import { describe, expect, it, vi } from "vitest";
import type { RenderFrame } from "../src/physics/frame";
import { createCamera } from "../src/render/camera";
import { linkProgram } from "../src/render/gl-program";
import { drawShadedBlob } from "../src/render/webgl-particles";
import {
  drawWater,
  type WaterBackdropCanvas,
} from "../src/render/webgl-water";
import {
  backdropCanvasFactory,
  glFixture,
  particleFrame,
} from "./webgl-render-fixture";

const camera = createCamera(640, 480);
const WALLS = [-2, 0, 2, 0, 2, 0, 2, 3];
const SPEED_LOCATION = 3;

function backdropFactory() {
  const factory = backdropCanvasFactory();
  // The mock GL accepts any upload source, so the fake stands in for a canvas.
  const create = factory.create as unknown as () => WaterBackdropCanvas;
  return { ...factory, create };
}

function wallFrame(segments: readonly number[]): RenderFrame {
  return { ...particleFrame(), rigidSegments: new Float32Array(segments) };
}

function draw(
  fixture: ReturnType<typeof glFixture>,
  frame: RenderFrame = particleFrame(),
  createBackdrop: () => WaterBackdropCanvas | undefined = () => undefined,
): boolean {
  return drawWater(fixture.canvas, frame, camera, 1, true, createBackdrop);
}

/** Counts texture uploads from a canvas source (the 6-argument form). */
function backdropUploads(gl: WebGL2RenderingContext): number {
  const calls: readonly (readonly unknown[])[] = vi.mocked(gl.texImage2D).mock
    .calls;
  return calls.filter(
    (call) => call.length === 6 && call[4] === gl.UNSIGNED_BYTE,
  ).length;
}

function withSpeedLocation(fixture: ReturnType<typeof glFixture>) {
  vi.mocked(fixture.gl.getAttribLocation).mockImplementation((_program, name) =>
    name === "aSpeed" ? SPEED_LOCATION : 0,
  );
  return fixture;
}

describe("drawWater", () => {
  it("draws with two programs, a 2-attachment target, points then the shade quad", () => {
    // Arrange
    const fixture = glFixture();
    const { gl } = fixture;

    // Act
    const drawn = draw(fixture);

    // Assert
    expect(drawn).toBe(true);
    expect(gl.createProgram).toHaveBeenCalledTimes(2);
    expect(gl.drawBuffers).toHaveBeenCalledWith([
      gl.COLOR_ATTACHMENT0,
      gl.COLOR_ATTACHMENT1,
    ]);
    const modes = vi.mocked(gl.drawArrays).mock.calls.map((call) => call[0]);
    expect(modes).toEqual([gl.POINTS, gl.TRIANGLE_STRIP]);
  });

  it("uploads the speed lane to the water speed buffer", () => {
    // Arrange
    const fixture = withSpeedLocation(glFixture());
    const frame: RenderFrame = {
      ...particleFrame(),
      maybeParticleSpeeds: new Float32Array([1, 8]),
    };

    // Act
    const drawn = draw(fixture, frame);

    // Assert
    expect(drawn).toBe(true);
    const speedUploads = fixture.uploads.filter(
      (upload) => upload.values.join() === "1,8",
    );
    expect(speedUploads).toHaveLength(1);
    expect(fixture.gl.bufferData).toHaveBeenCalledWith(
      fixture.gl.ARRAY_BUFFER,
      new Float32Array([1, 8]),
      fixture.gl.DYNAMIC_DRAW,
    );
    expect(fixture.gl.enableVertexAttribArray).toHaveBeenCalledWith(
      SPEED_LOCATION,
    );
  });

  it("draws zero foam from a constant attribute when the speed lane is absent", () => {
    // Arrange
    const fixture = withSpeedLocation(glFixture());

    // Act
    const drawn = draw(fixture);

    // Assert
    expect(drawn).toBe(true);
    expect(fixture.gl.disableVertexAttribArray).toHaveBeenCalledWith(
      SPEED_LOCATION,
    );
    expect(fixture.gl.vertexAttrib1f).toHaveBeenCalledWith(SPEED_LOCATION, 0);
  });

  it.each([
    ["a single draw buffer", { maybeMaxDrawBuffers: 1 }],
    ["no highp fragment precision", { maybeHighpPrecision: 0 }],
    ["an incomplete 2-attachment target", { maybeIncompleteMrt: true }],
  ] as const)(
    "falls back with %s while shaded blob still draws on the same canvas",
    (_name, options) => {
      // Arrange
      const fixture = glFixture(options);

      // Act
      const water = draw(fixture);
      const blob = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

      // Assert
      expect([water, blob]).toEqual([false, true]);
    },
  );

  it("falls back together with shaded blob when no color-buffer extension exists", () => {
    // Arrange
    const fixture = glFixture({ maybeExtensions: [] });

    // Act
    const water = draw(fixture);
    const blob = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Assert
    expect([water, blob]).toEqual([false, false]);
  });

  it("falls back while the context is lost and rebuilds two programs on restore", () => {
    // Arrange
    const fixture = glFixture();
    draw(fixture);
    fixture.lose();
    const whileLost = draw(fixture);

    // Act
    fixture.restore();
    const restored = draw(fixture);

    // Assert
    expect([whileLost, restored]).toEqual([false, true]);
    expect(fixture.gl.createProgram).toHaveBeenCalledTimes(4);
  });

  it("uploads the walls backdrop once for a run of identical frames", () => {
    // Arrange
    const fixture = glFixture();
    const backdrop = backdropFactory();

    // Act
    for (let frame = 0; frame < 4; frame++)
      draw(fixture, wallFrame(WALLS), backdrop.create);

    // Assert
    expect(backdropUploads(fixture.gl)).toBe(1);
  });

  it("uploads the backdrop once more after a context restore", () => {
    // Arrange
    const fixture = glFixture();
    const backdrop = backdropFactory();
    for (let frame = 0; frame < 4; frame++)
      draw(fixture, wallFrame(WALLS), backdrop.create);
    fixture.lose();
    fixture.restore();

    // Act
    draw(fixture, wallFrame(WALLS), backdrop.create);
    draw(fixture, wallFrame(WALLS), backdrop.create);

    // Assert
    expect(backdropUploads(fixture.gl)).toBe(2);
  });

  it("never uploads a backdrop while the walls move every frame", () => {
    // Arrange
    const fixture = glFixture();
    const backdrop = backdropFactory();

    // Act
    for (let frame = 0; frame < 5; frame++)
      draw(fixture, wallFrame([frame * 0.1, 0, 2, 0]), backdrop.create);

    // Assert
    expect(backdropUploads(fixture.gl)).toBe(0);
  });

  it("returns false without a canvas", () => {
    // Arrange
    const frame = particleFrame();

    // Act
    const drawn = drawWater(undefined, frame, camera, 1);

    // Assert
    expect(drawn).toBe(false);
  });

  it("creates no water resources on a canvas that only draws shaded blob", () => {
    // Arrange
    const fixture = glFixture();

    // Act
    drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Assert
    expect(fixture.gl.createProgram).toHaveBeenCalledTimes(2);
    expect(fixture.gl.getUniformLocation).toHaveBeenCalledTimes(8);
  });
});

describe("water GL cleanup", () => {
  it("frees every water GL object when the 2-attachment target is incomplete", () => {
    // Arrange
    const fixture = glFixture({ maybeIncompleteMrt: true });

    // Act
    const drawn = draw(fixture);

    // Assert
    const { gl } = fixture;
    expect(drawn).toBe(false);
    expect(gl.deleteProgram).toHaveBeenCalledTimes(2);
    expect(gl.deleteVertexArray).toHaveBeenCalledTimes(2);
    expect(gl.deleteBuffer).toHaveBeenCalledTimes(2);
    expect(gl.deleteFramebuffer).toHaveBeenCalledTimes(1);
    expect(gl.deleteTexture).toHaveBeenCalledTimes(3);
  });

  it("frees the objects it made when a vertex array cannot be created", () => {
    // Arrange
    const fixture = glFixture();
    // A lost context returns null even though the DOM typings omit it.
    const lostVao = null as unknown as WebGLVertexArrayObject;
    vi.mocked(fixture.gl.createVertexArray).mockReturnValueOnce(lostVao);

    // Act
    const drawn = draw(fixture);

    // Assert
    const { gl } = fixture;
    expect(drawn).toBe(false);
    expect(gl.deleteProgram).toHaveBeenCalledTimes(2);
    expect(gl.deleteFramebuffer).toHaveBeenCalledTimes(1);
  });

  it("deletes the compiled vertex shader when the fragment stage fails", () => {
    // Arrange
    const { gl } = glFixture();
    vi.mocked(gl.getShaderParameter)
      .mockReturnValueOnce(true)
      .mockReturnValueOnce(false);

    // Act
    const program = linkProgram(gl, "vertex", "fragment");

    // Assert
    const [vertexShader] = vi.mocked(gl.createShader).mock.results;
    expect(program).toBeUndefined();
    expect(gl.deleteShader).toHaveBeenCalledWith(vertexShader?.value);
    expect(gl.createProgram).not.toHaveBeenCalled();
  });
});

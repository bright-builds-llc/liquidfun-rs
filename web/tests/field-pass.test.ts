import { describe, expect, it, vi } from "vitest";
import { createCamera } from "../src/render/camera";
import {
  createFieldTarget,
  particleSurfaceFor,
  resizeFieldTarget,
  supportsHalfFloatTargets,
  watchParticleSurface,
} from "../src/render/field-pass";
import { drawShadedBlob } from "../src/render/webgl-particles";
import { glFixture, particleFrame } from "./webgl-render-fixture";

const camera = createCamera(640, 480);

describe("float-or-half render target acceptance (D-13)", () => {
  it.each([
    [["EXT_color_buffer_float"], true],
    [["EXT_color_buffer_half_float"], true],
    [[], false],
  ] as const)("with extensions %j reports %s", (extensions, expected) => {
    // Arrange
    const fixture = glFixture({ maybeExtensions: extensions });

    // Act
    const supported = supportsHalfFloatTargets(fixture.gl);

    // Assert
    expect(supported).toBe(expected);
  });

  it("draws shaded blob into a framebuffer when only the half-float extension is exposed", () => {
    // Arrange
    const fixture = glFixture({
      maybeExtensions: ["EXT_color_buffer_half_float"],
    });

    // Act
    const drawn = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Assert
    expect(drawn).toBe(true);
    expect(fixture.gl.bindFramebuffer).toHaveBeenCalledWith(
      fixture.gl.FRAMEBUFFER,
      expect.objectContaining({ id: expect.any(Number) }),
    );
  });

  it("falls back without building programs when neither color-buffer extension is exposed", () => {
    // Arrange
    const fixture = glFixture({ maybeExtensions: [] });

    // Act
    const drawn = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Assert
    expect(drawn).toBe(false);
    expect(fixture.gl.createProgram).not.toHaveBeenCalled();
  });

  it("falls back on an incomplete field framebuffer and stays off without rebuilding programs", () => {
    // Arrange
    const fixture = glFixture({ maybeFramebufferStatus: () => 0 });
    const first = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Act
    const second = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Assert
    expect([first, second]).toEqual([false, false]);
    expect(fixture.gl.createProgram).toHaveBeenCalledTimes(2);
  });
});

describe("field targets", () => {
  it("allocates RGBA16F half-float storage for every attachment", () => {
    // Arrange
    const { gl } = glFixture();
    const target = createFieldTarget(gl, 2);
    if (target === undefined) throw new Error("target was not created");

    // Act
    resizeFieldTarget(gl, target, 4, 4);

    // Assert
    expect(vi.mocked(gl.texImage2D).mock.calls).toEqual([
      [gl.TEXTURE_2D, 0, gl.RGBA16F, 4, 4, 0, gl.RGBA, gl.HALF_FLOAT, null],
      [gl.TEXTURE_2D, 0, gl.RGBA16F, 4, 4, 0, gl.RGBA, gl.HALF_FLOAT, null],
    ]);
  });

  it("attaches two textures, enables both draw buffers and records the size when complete", () => {
    // Arrange
    const { gl } = glFixture();
    const target = createFieldTarget(gl, 2);
    if (target === undefined) throw new Error("target was not created");

    // Act
    const complete = resizeFieldTarget(gl, target, 4, 4);

    // Assert
    expect(complete).toBe(true);
    expect(
      vi.mocked(gl.framebufferTexture2D).mock.calls.map((call) => call[1]),
    ).toEqual([gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1]);
    expect(gl.drawBuffers).toHaveBeenCalledWith([
      gl.COLOR_ATTACHMENT0,
      gl.COLOR_ATTACHMENT1,
    ]);
    expect([target.width, target.height]).toEqual([4, 4]);
  });

  it("never sets draw buffers for a single-attachment target", () => {
    // Arrange
    const { gl } = glFixture();
    const target = createFieldTarget(gl, 1);
    if (target === undefined) throw new Error("target was not created");

    // Act
    resizeFieldTarget(gl, target, 4, 4);

    // Assert
    expect(gl.drawBuffers).not.toHaveBeenCalled();
  });

  it("keeps the previous size when the framebuffer is incomplete", () => {
    // Arrange
    const { gl } = glFixture({ maybeFramebufferStatus: () => 0 });
    const target = createFieldTarget(gl, 2);
    if (target === undefined) throw new Error("target was not created");

    // Act
    const complete = resizeFieldTarget(gl, target, 4, 4);

    // Assert
    expect(complete).toBe(false);
    expect([target.width, target.height]).toEqual([0, 0]);
  });
});

type Repaint = { readonly drawn: boolean; readonly programs: number };

function watchWithDraw(fixture: ReturnType<typeof glFixture>) {
  const repaints: Repaint[] = [];
  const detach = watchParticleSurface(fixture.canvas, () => {
    const drawn = drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);
    const programs = vi.mocked(fixture.gl.createProgram).mock.calls.length;
    repaints.push({ drawn, programs });
  });
  return { repaints, detach };
}

describe("ordered loss and restore repaint (D-14)", () => {
  it("repaints once after a lost context, and that repaint falls back", () => {
    // Arrange
    const fixture = glFixture();
    drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);
    const { repaints } = watchWithDraw(fixture);

    // Act
    fixture.lose();

    // Assert
    expect(repaints).toEqual([{ drawn: false, programs: 2 }]);
  });

  it("repaints after restore with freshly built programs", () => {
    // Arrange
    const fixture = glFixture();
    drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);
    const { repaints } = watchWithDraw(fixture);
    fixture.lose();

    // Act
    fixture.restore();

    // Assert
    expect(repaints.at(-1)).toEqual({ drawn: true, programs: 4 });
    expect(repaints).toHaveLength(2);
  });

  it("resets before the repaint on restore when watched before the first draw", () => {
    // Arrange
    const fixture = glFixture();
    const { repaints } = watchWithDraw(fixture);
    drawShadedBlob(fixture.canvas, particleFrame(), camera, 1);

    // Act
    fixture.restore();

    // Assert
    expect(repaints).toEqual([{ drawn: true, programs: 4 }]);
  });

  it("stops repainting after detach on lost and restored events", () => {
    // Arrange
    const fixture = glFixture();
    const repaint = vi.fn();
    const detach = watchParticleSurface(fixture.canvas, repaint);

    // Act
    detach();
    fixture.lose();
    fixture.restore();

    // Assert
    expect(repaint).not.toHaveBeenCalled();
  });
});

describe("per-pass availability", () => {
  it("keeps a failed pass unavailable while another pass on the surface is created", () => {
    // Arrange
    const surface = particleSurfaceFor(glFixture().canvas);
    const failedKey = Symbol("failed"),
      otherKey = Symbol("other");
    const failing = vi.fn(() => undefined);
    surface.pass(failedKey, failing);

    // Act
    const retried = surface.pass(failedKey, () => "late");
    const other = surface.pass(otherKey, () => "water");

    // Assert
    expect(retried).toBeUndefined();
    expect(failing).toHaveBeenCalledTimes(1);
    expect(other).toBe("water");
  });

  it("rebuilds a failed pass after a lost and restored context", () => {
    // Arrange
    const fixture = glFixture();
    const surface = particleSurfaceFor(fixture.canvas);
    const key = Symbol("pass");
    surface.pass(key, () => undefined);

    // Act
    fixture.lose();
    fixture.restore();
    const rebuilt = surface.pass(key, () => "rebuilt");

    // Assert
    expect(rebuilt).toBe("rebuilt");
  });
});

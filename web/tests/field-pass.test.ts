import { describe, expect, it, vi } from "vitest";
import { createCamera } from "../src/render/camera";
import {
  createFieldTarget,
  resizeFieldTarget,
  supportsHalfFloatTargets,
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

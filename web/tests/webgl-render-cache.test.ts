import { describe, expect, it, vi } from "vitest";
import { createCamera, projectPoint } from "../src/render/camera";
import { drawShadedBlob } from "../src/render/webgl-particles";
import { glFixture, particleFrame } from "./webgl-render-fixture";

describe("actual WebGL submission caches", () => {
  it.each(["radius", "color"])(
    "updates current positions and the changed %s lane without disturbing the other metadata",
    (lane) => {
      // Arrange
      const fixture = glFixture(),
        camera = createCamera(640, 480),
        frame = particleFrame();
      drawShadedBlob(fixture.canvas, frame, camera, 1);
      const before = fixture.uploads.length;

      // Act
      frame.particlePositions[0] = 2;
      if (lane === "radius") frame.particleRadii[0] = 0.4;
      else frame.particleColors[0] = 77;
      drawShadedBlob(fixture.canvas, frame, camera, 1);

      // Assert
      expect(fixture.uploads).toHaveLength(before + 2);
      expect(fixture.uploads.at(-2)?.values[0]).toBe(
        Math.fround(projectPoint(camera, { x: 2, y: 1 }).x),
      );
      expect(fixture.uploads.at(-1)?.values[0]).toBe(
        lane === "radius"
          ? Math.fround(frame.particleRadii[0]! * camera.scale)
          : Math.fround(77 / 255),
      );
    },
  );

  it("does not clone cache snapshots during a changing-count stream and refreshes full lanes on return to an earlier count", () => {
    // Arrange
    const fixture = glFixture(),
      camera = createCamera(640, 480),
      first = particleFrame();
    drawShadedBlob(fixture.canvas, first, camera, 1);
    const radiusCopies = vi.spyOn(Uint32Array.prototype, "slice");
    const colorCopies = vi.spyOn(Uint8Array.prototype, "slice");
    const before = fixture.uploads.length;

    // Act
    try {
      const smaller = {
        ...first,
        particleCount: 1,
        particlePositions: new Float32Array([3, 4]),
        particleRadii: new Float32Array([0.4]),
        particleColors: new Uint8Array([77, 50, 30, 255]),
      };
      drawShadedBlob(fixture.canvas, smaller, camera, 1);
      drawShadedBlob(fixture.canvas, first, camera, 1);

      // Assert
      expect(radiusCopies).not.toHaveBeenCalled();
      expect(colorCopies).not.toHaveBeenCalled();
      expect(fixture.uploads).toHaveLength(before + 6);
      expect(fixture.uploads.at(-2)?.values).toEqual(
        Array.from(first.particleRadii, (radius) =>
          Math.fround(radius * camera.scale),
        ),
      );
      expect(fixture.uploads.at(-1)?.values).toEqual(
        Array.from(first.particleColors, (color) => Math.fround(color / 255)),
      );
    } finally {
      radiusCopies.mockRestore();
      colorCopies.mockRestore();
    }
  });

  it("uploads all current positions each frame while retaining exact unchanged radius/color lanes", () => {
    // Arrange
    const fixture = glFixture(),
      camera = createCamera(640, 480),
      frame = particleFrame();
    drawShadedBlob(fixture.canvas, frame, camera, 1);
    const before = fixture.uploads.length;

    // Act
    frame.particlePositions[0] = 2;
    drawShadedBlob(fixture.canvas, { ...frame, stepIndex: 2 }, camera, 1);

    // Assert
    expect(fixture.uploads).toHaveLength(before + 1);
    const projected = projectPoint(camera, { x: 2, y: 1 });
    expect(fixture.uploads.at(-1)?.values).toEqual([
      Math.fround(projected.x),
      Math.fround(projected.y),
      Math.fround(projectPoint(camera, { x: 1, y: 2 }).x),
      Math.fround(projectPoint(camera, { x: 1, y: 2 }).y),
    ]);
    expect(
      vi
        .mocked(fixture.gl.drawArrays)
        .mock.calls.filter((call) => call[0] === fixture.gl.POINTS),
    ).toEqual([
      [fixture.gl.POINTS, 0, 2],
      [fixture.gl.POINTS, 0, 2],
    ]);
    expect(fixture.gl.getUniformLocation).toHaveBeenCalledTimes(8);
  });

  it("refreshes radius metadata on camera zoom and color metadata on an in-place row change", () => {
    // Arrange
    const fixture = glFixture(),
      frame = particleFrame(),
      camera = createCamera(640, 480);
    drawShadedBlob(fixture.canvas, frame, camera, 1);
    const before = fixture.uploads.length;

    // Act
    frame.particleColors[0] = 77;
    const zoomed = createCamera(640, 480, { zoom: 2, panX: 0, panY: 0 });
    drawShadedBlob(fixture.canvas, frame, zoomed, 1);

    // Assert
    expect(fixture.uploads).toHaveLength(before + 3);
    expect(fixture.uploads.at(-2)?.values).toEqual(
      Array.from(frame.particleRadii, (radius) =>
        Math.fround(radius * zoomed.scale),
      ),
    );
    expect(fixture.uploads.at(-1)?.values[0]).toBe(Math.fround(77 / 255));
  });

  it("submits every row after a particle-count change", () => {
    // Arrange
    const fixture = glFixture(),
      camera = createCamera(640, 480),
      frame = particleFrame();
    drawShadedBlob(fixture.canvas, frame, camera, 1);

    // Act
    const one = {
      ...frame,
      particleCount: 1,
      particlePositions: frame.particlePositions.slice(0, 2),
      particleRadii: frame.particleRadii.slice(0, 1),
      particleColors: frame.particleColors.slice(0, 4),
    };
    drawShadedBlob(fixture.canvas, one, camera, 1);

    // Assert
    expect(fixture.uploads.at(-3)?.values).toHaveLength(2);
    expect(fixture.uploads.at(-2)?.values).toHaveLength(1);
    expect(fixture.uploads.at(-1)?.values).toHaveLength(4);
    expect(fixture.gl.drawArrays).toHaveBeenCalledWith(fixture.gl.POINTS, 0, 1);
  });

  it("owns metadata separately for separate canvases", () => {
    // Arrange
    const first = glFixture(),
      second = glFixture(),
      camera = createCamera(640, 480),
      frame = particleFrame();

    // Act
    drawShadedBlob(first.canvas, frame, camera, 1);
    drawShadedBlob(second.canvas, frame, camera, 1);
    drawShadedBlob(first.canvas, frame, camera, 1);

    // Assert
    expect(first.gl.createProgram).toHaveBeenCalledTimes(2);
    expect(second.gl.createProgram).toHaveBeenCalledTimes(2);
    expect(first.gl.getUniformLocation).toHaveBeenCalledTimes(8);
    expect(second.gl.getUniformLocation).toHaveBeenCalledTimes(8);
  });

  it("falls back while lost and recreates every resource and metadata lane after restoration", () => {
    // Arrange
    const fixture = glFixture(),
      frame = particleFrame(),
      camera = createCamera(640, 480);
    drawShadedBlob(fixture.canvas, frame, camera, 1);

    // Act
    const event = fixture.lose();
    const lost = drawShadedBlob(fixture.canvas, frame, camera, 1);
    fixture.restore();
    const before = fixture.uploads.length;
    const restored = drawShadedBlob(fixture.canvas, frame, camera, 1);

    // Assert
    expect(event.defaultPrevented).toBe(true);
    expect(lost).toBe(false);
    expect(restored).toBe(true);
    expect(fixture.uploads).toHaveLength(before + 4);
    expect(fixture.gl.getUniformLocation).toHaveBeenCalledTimes(16);
  });
});

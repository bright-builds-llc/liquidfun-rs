import { createCanvas, Path2D } from "@napi-rs/canvas";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { RenderFrame } from "../src/physics/frame";
import { createCamera } from "../src/render/camera";
import { drawRenderFrame } from "../src/render/canvas";

const segments = new Float32Array(
  Array.from({ length: 40 }, (_, index) => [
    -2 + index * 0.08,
    1,
    1,
    4 + index * 0.01,
  ]).flat(),
);

function frame(stepIndex: number): RenderFrame {
  return {
    stepIndex,
    particleCount: 1,
    rigidShapeCount: 40,
    maxSpeed: 0,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array([0, 2]),
    particleColors: new Uint8Array([20, 180, 220, 180]),
    particleRadii: new Float32Array([0.3]),
    rigidSegments: segments.slice(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
  };
}

function surface() {
  const canvas = createCanvas(640, 480);
  const context = canvas.getContext(
    "2d",
  ) as unknown as CanvasRenderingContext2D;
  return { canvas, context };
}

function factory() {
  const create = vi.fn(function () {
    return new Path2D();
  });
  vi.stubGlobal("Path2D", create);
  return create;
}

afterEach(() => vi.unstubAllGlobals());

describe("stable wall foreground", () => {
  it("applies current native transforms and stroke styles to reused paths", () => {
    // Arrange
    factory();
    const direct = surface(),
      cached = surface(),
      camera = createCamera(640, 480);
    for (let step = 0; step < 2; step++) {
      drawRenderFrame(direct.context, frame(step), camera, "solid");
      drawRenderFrame(
        cached.context,
        frame(step),
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        1,
      );
    }

    // Act
    for (const target of [direct, cached]) {
      target.context.setTransform(1, 0, 0, 1, 7, 3);
      target.context.lineCap = "round";
      target.context.setLineDash([3, 2]);
    }
    drawRenderFrame(direct.context, frame(2), camera, "solid");
    drawRenderFrame(
      cached.context,
      frame(2),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Assert
    expect(cached.context.getImageData(0, 0, 640, 480).data).toEqual(
      direct.context.getImageData(0, 0, 640, 480).data,
    );
  });

  it("detects camera values changed on the same camera object", () => {
    // Arrange
    factory();
    const direct = surface(),
      cached = surface(),
      camera = createCamera(640, 480);
    drawRenderFrame(
      cached.context,
      frame(0),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      cached.context,
      frame(1),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Act
    Object.assign(camera, { offsetX: camera.offsetX + 20 });
    drawRenderFrame(direct.context, frame(2), camera, "solid");
    drawRenderFrame(
      cached.context,
      frame(2),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Assert
    expect(cached.context.getImageData(0, 0, 640, 480).data).toEqual(
      direct.context.getImageData(0, 0, 640, 480).data,
    );
  });

  it("invalidates stroke/mode changes and reset even when the geometry is identical", () => {
    // Arrange
    const create = factory();
    const target = surface(),
      camera = createCamera(640, 480);
    drawRenderFrame(
      target.context,
      frame(10),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      target.context,
      frame(11),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    expect(create).toHaveBeenCalledTimes(40);

    // Act
    drawRenderFrame(
      target.context,
      frame(12),
      camera,
      "circle-wireframe",
      0.9,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      target.context,
      frame(13),
      camera,
      "circle-wireframe",
      0.9,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      target.context,
      frame(0),
      camera,
      "circle-wireframe",
      0.9,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Assert
    expect(create).toHaveBeenCalledTimes(80);
  });

  it("preserves DPR backing resolution and keeps current circle labels outside the cached layer", () => {
    // Arrange
    factory();
    const direct = surface(),
      cached = surface(),
      camera = createCamera(320, 240);
    direct.context.setTransform(2, 0, 0, 2, 0, 0);
    cached.context.setTransform(2, 0, 0, 2, 0, 0);
    const text = vi.spyOn(cached.context, "fillText");

    // Act
    for (let step = 0; step < 4; step++) {
      const current = {
        ...frame(step),
        rigidCircles: new Float32Array([0, 2, 1]),
        circleLabels: [`${step}`],
      };
      drawRenderFrame(direct.context, current, camera, "solid");
      drawRenderFrame(
        cached.context,
        current,
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        2,
      );
    }

    // Assert
    expect(text).toHaveBeenCalledTimes(4);
    expect(text.mock.calls.at(-1)?.[0]).toBe("3");
    const expected = direct.context.getImageData(0, 0, 640, 480).data;
    const actual = cached.context.getImageData(0, 0, 640, 480).data;
    let maxDifference = 0;
    for (let index = 0; index < expected.length; index++)
      maxDifference = Math.max(
        maxDifference,
        Math.abs(expected[index]! - actual[index]!),
      );
    expect(maxDifference).toBe(0);
  });

  it("invalidates a lost/restored context and never shares a foreground across surfaces", () => {
    // Arrange
    const create = factory(),
      first = surface(),
      second = surface(),
      camera = createCamera(640, 480);
    const listeners = new Map<string, () => void>();
    Object.defineProperty(first.canvas, "addEventListener", {
      value: (name: string, listener: () => void) =>
        listeners.set(name, listener),
    });

    // Act
    for (let step = 0; step < 2; step++) {
      drawRenderFrame(
        first.context,
        frame(step),
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        1,
      );
      drawRenderFrame(
        second.context,
        frame(step),
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        1,
      );
    }
    listeners.get("contextlost")?.();
    listeners.get("contextrestored")?.();
    drawRenderFrame(
      first.context,
      frame(2),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      first.context,
      frame(3),
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Assert
    expect(create).toHaveBeenCalledTimes(120);
  });

  it("reuses stable copied geometry and preserves overlapping stroke pixels", () => {
    // Arrange
    const create = factory(),
      direct = surface(),
      cached = surface();
    const camera = createCamera(640, 480);

    // Act
    for (let step = 0; step < 4; step++) {
      drawRenderFrame(direct.context, frame(step), camera, "solid");
      drawRenderFrame(
        cached.context,
        frame(step),
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        1,
      );
    }

    // Assert
    expect(create).toHaveBeenCalledTimes(40);
    const expected = direct.context.getImageData(0, 0, 640, 480).data;
    const actual = cached.context.getImageData(0, 0, 640, 480).data;
    let maxDifference = 0;
    for (let index = 0; index < expected.length; index++)
      maxDifference = Math.max(
        maxDifference,
        Math.abs(expected[index]! - actual[index]!),
      );
    expect(maxDifference).toBe(0);
  });

  it("keeps moving rigid geometry on the direct path", () => {
    // Arrange
    const create = factory(),
      target = surface(),
      camera = createCamera(640, 480);

    // Act
    for (let step = 0; step < 12; step++) {
      const moving = frame(step);
      moving.rigidSegments[0] = step;
      drawRenderFrame(
        target.context,
        moving,
        camera,
        "solid",
        0.3,
        Infinity,
        false,
        true,
        undefined,
        1,
      );
    }

    // Assert
    expect(create).not.toHaveBeenCalled();
  });

  it("invalidates an in-place geometry change and a camera change", () => {
    // Arrange
    factory();
    const direct = surface(),
      cached = surface(),
      camera = createCamera(640, 480);
    const current = frame(1);
    drawRenderFrame(
      cached.context,
      current,
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );
    drawRenderFrame(
      cached.context,
      { ...current, stepIndex: 2 },
      camera,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Act
    current.rigidSegments[0] = 3;
    const moved = createCamera(640, 480, { zoom: 1.2, panX: 3, panY: -4 });
    drawRenderFrame(direct.context, current, moved, "solid");
    drawRenderFrame(
      cached.context,
      { ...current, stepIndex: 3 },
      moved,
      "solid",
      0.3,
      Infinity,
      false,
      true,
      undefined,
      1,
    );

    // Assert
    expect(cached.context.getImageData(0, 0, 640, 480).data).toEqual(
      direct.context.getImageData(0, 0, 640, 480).data,
    );
  });
});

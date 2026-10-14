import { describe, expect, it } from "vitest";
import type { RenderFrame } from "../src/physics/frame";
import { createCamera } from "../src/render/camera";
import type { FieldSize } from "../src/render/field-pass";
import { createWaterBackdrop } from "../src/render/water-backdrop";
import { backdropCanvasFactory, particleFrame } from "./webgl-render-fixture";

const camera = createCamera(640, 480);
const size: FieldSize = { width: 640, height: 480, pixelRatio: 1 };

function wallFrame(segments: readonly number[]): RenderFrame {
  return { ...particleFrame(), rigidSegments: new Float32Array(segments) };
}

const WALLS = [-2, 0, 2, 0, 2, 0, 2, 3];

function callsNamed(
  calls: readonly { readonly name: string }[],
  name: string,
): number {
  return calls.filter((call) => call.name === name).length;
}

describe("water backdrop (D-07 refinement)", () => {
  it("does not rasterize the first frame with segments", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);

    // Act
    backdrop.update(wallFrame(WALLS), camera, size);

    // Assert
    expect([backdrop.matchesView, backdrop.revision]).toEqual([false, 0]);
    expect(factory.create).not.toHaveBeenCalled();
  });

  it("rasterizes the walls once the second identical frame arrives", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);
    backdrop.update(wallFrame(WALLS), camera, size);

    // Act
    backdrop.update(wallFrame(WALLS), camera, size);

    // Assert
    expect([backdrop.matchesView, backdrop.revision]).toEqual([true, 1]);
    const fills = factory.calls.filter((call) => call.name === "fillRect");
    const strokes = factory.calls.filter((call) => call.name === "stroke");
    expect(fills.map((call) => call.fillStyle)).toEqual(["#071018"]);
    expect(strokes.map((call) => call.strokeStyle)).toEqual(["#94A3B8"]);
    expect(backdrop.maybeRaster).toMatchObject({ width: 640, height: 480 });
  });

  it("does not redraw while the walls and view stay the same", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);
    backdrop.update(wallFrame(WALLS), camera, size);
    backdrop.update(wallFrame(WALLS), camera, size);
    const drawnCalls = factory.calls.length;

    // Act
    backdrop.update(wallFrame(WALLS), camera, size);
    backdrop.update(wallFrame(WALLS), camera, size);

    // Assert
    expect([backdrop.matchesView, backdrop.revision]).toEqual([true, 1]);
    expect(factory.calls.length).toBe(drawnCalls);
  });

  it("never rasterizes segments that move every frame", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);
    const views: boolean[] = [];

    // Act
    for (let step = 0; step < 5; step++) {
      backdrop.update(wallFrame([step * 0.1, 0, 2, 0]), camera, size);
      views.push(backdrop.matchesView);
    }

    // Assert
    expect(backdrop.revision).toBe(0);
    expect(views).toEqual([false, false, false, false, false]);
    expect(callsNamed(factory.calls, "stroke")).toBe(0);
  });

  it("stops matching on a camera change and redraws once the new view is stable", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);
    backdrop.update(wallFrame(WALLS), camera, size);
    backdrop.update(wallFrame(WALLS), camera, size);
    const zoomed = createCamera(640, 480, { zoom: 2, panX: 0, panY: 0 });

    // Act
    backdrop.update(wallFrame(WALLS), zoomed, size);
    const matchedAfterChange = backdrop.matchesView;
    backdrop.update(wallFrame(WALLS), zoomed, size);

    // Assert
    expect(matchedAfterChange).toBe(false);
    expect([backdrop.matchesView, backdrop.revision]).toEqual([true, 2]);
  });

  it("stops matching on a field-size change and redraws once it is stable", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);
    backdrop.update(wallFrame(WALLS), camera, size);
    backdrop.update(wallFrame(WALLS), camera, size);
    const sharper: FieldSize = { width: 1280, height: 960, pixelRatio: 2 };

    // Act
    backdrop.update(wallFrame(WALLS), camera, sharper);
    const matchedAfterChange = backdrop.matchesView;
    backdrop.update(wallFrame(WALLS), camera, sharper);

    // Assert
    expect(matchedAfterChange).toBe(false);
    expect([backdrop.matchesView, backdrop.revision]).toEqual([true, 2]);
    expect(backdrop.maybeRaster).toMatchObject({ width: 1280, height: 960 });
  });

  it("uses the procedural backdrop for a frame without segments", () => {
    // Arrange
    const factory = backdropCanvasFactory();
    const backdrop = createWaterBackdrop(factory.create);

    // Act
    backdrop.update(wallFrame([]), camera, size);
    backdrop.update(wallFrame([]), camera, size);

    // Assert
    expect([backdrop.matchesView, backdrop.revision]).toEqual([false, 0]);
    expect(factory.create).not.toHaveBeenCalled();
  });

  it("stays procedural without an error when no canvas can be created", () => {
    // Arrange
    const backdrop = createWaterBackdrop(() => undefined);

    // Act
    const update = () => {
      for (let step = 0; step < 3; step++)
        backdrop.update(wallFrame(WALLS), camera, size);
    };

    // Assert
    expect(update).not.toThrow();
    expect([backdrop.matchesView, backdrop.revision]).toEqual([false, 0]);
    expect(backdrop.maybeRaster).toBeUndefined();
  });
});

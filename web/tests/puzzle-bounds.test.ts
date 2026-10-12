import { describe, expect, it } from "vitest";

import { PUZZLE_IDS, type PuzzleId } from "../src/catalog/puzzles";
import { SCENE_IDS } from "../src/catalog/scenes";
import { worldBoundsForViewport } from "../src/catalog/portrait-bounds";
import {
  PUZZLE_PORTRAIT_BOUNDS,
  PUZZLE_VIEW_BOUNDS,
  worldBoundsForPuzzle,
  worldBoundsForTarget,
} from "../src/catalog/puzzle-bounds";
import {
  IDENTITY_CAMERA_VIEW,
  createCamera,
  projectPoint,
  type WorldBounds,
} from "../src/render/camera";

const IPHONE = { width: 390, height: 844 } as const;
const DESKTOP = { width: 1280, height: 800 } as const;
const MIN_WIDTH_FRACTION = 0.85;
const MIN_HEIGHT_FRACTION = 0.33;

type LevelGeometry = {
  readonly cupInterior: WorldBounds;
  readonly buildArea: WorldBounds;
  readonly walls: WorldBounds;
  readonly spawn: WorldBounds;
};

/**
 * Level geometry the frames must contain, hardcoded from the Rust level spec
 * and cross-checked once against the generated wasm (Phase 38 Plan 03 probe,
 * `target/phase38/level-probe.txt`).
 */
const LEVEL_GEOMETRY: Record<PuzzleId, LevelGeometry> = {
  "first-pour": {
    // levelInfo().cupInterior() = [1.6, 0, 3.4, 1.8] m (f32).
    cupInterior: { minX: 1.6, minY: 0, maxX: 3.4, maxY: 1.8 },
    // levelInfo().buildArea() = grid [-20, 44, 40, 84] x gridMeters 0.05 m.
    buildArea: { minX: -1, minY: 2.2, maxX: 2, maxY: 4.2 },
    // captureFrame().rigidSegments() extents: x -6..6, y -0.5..9.
    walls: { minX: -6, minY: -0.5, maxX: 6, maxY: 9 },
    // captureFrame().particlePositions() extents (x -2.94..-0.66,
    // y 4.86..7.86, 1989 particles) widened by the particle radius.
    spawn: { minX: -2.96, minY: 4.84, maxX: -0.64, maxY: 7.9 },
  },
};

/** Same fit rule as `fittedFrameFraction` in portrait-bounds.test.ts. */
function fittedFrameFraction(
  id: PuzzleId,
  viewport: { readonly width: number; readonly height: number },
): { readonly width: number; readonly height: number } {
  const bounds = worldBoundsForPuzzle(id, viewport.width, viewport.height);
  const camera = createCamera(
    viewport.width,
    viewport.height,
    IDENTITY_CAMERA_VIEW,
    bounds,
  );
  const topLeft = projectPoint(camera, { x: bounds.minX, y: bounds.maxY });
  const bottomRight = projectPoint(camera, { x: bounds.maxX, y: bounds.minY });
  return {
    width: (bottomRight.x - topLeft.x) / viewport.width,
    height: (bottomRight.y - topLeft.y) / viewport.height,
  };
}

function contains(outer: WorldBounds, inner: WorldBounds): boolean {
  return (
    outer.minX <= inner.minX &&
    outer.minY <= inner.minY &&
    outer.maxX >= inner.maxX &&
    outer.maxY >= inner.maxY
  );
}

describe("puzzle camera frames", () => {
  it("uses the portrait frame on an iPhone", () => {
    // Arrange
    const id = "first-pour" as const;

    // Act
    const bounds = worldBoundsForPuzzle(id, IPHONE.width, IPHONE.height);

    // Assert
    expect(bounds).toEqual(PUZZLE_PORTRAIT_BOUNDS[id]);
  });

  it("uses the landscape frame on a wide or empty viewport", () => {
    // Arrange
    const id = "first-pour" as const;

    // Act
    const landscape = worldBoundsForPuzzle(id, IPHONE.height, IPHONE.width);
    const empty = worldBoundsForPuzzle(id, 0, 0);

    // Assert
    expect(landscape).toEqual(PUZZLE_VIEW_BOUNDS[id]);
    expect(empty).toEqual(PUZZLE_VIEW_BOUNDS[id]);
  });

  it("fills most of an iPhone canvas for every puzzle", () => {
    // Arrange
    const ids = PUZZLE_IDS;

    // Act
    const fractions = ids.map((id) => ({ id, ...fittedFrameFraction(id, IPHONE) }));

    // Assert
    for (const fraction of fractions) {
      expect(fraction.width, fraction.id).toBeGreaterThan(MIN_WIDTH_FRACTION);
      expect(fraction.height, fraction.id).toBeGreaterThan(MIN_HEIGHT_FRACTION);
    }
  });

  it("contains the cup, build area, walls and spawn in both frames", () => {
    // Arrange
    const frames = PUZZLE_IDS.flatMap((id) => [
      { id, orientation: "landscape", bounds: PUZZLE_VIEW_BOUNDS[id] },
      { id, orientation: "portrait", bounds: PUZZLE_PORTRAIT_BOUNDS[id] },
    ]);

    // Act
    const outside = frames.flatMap(({ id, orientation, bounds }) =>
      Object.entries(LEVEL_GEOMETRY[id])
        .filter(([, box]) => !contains(bounds, box))
        .map(([name]) => `${id} ${orientation} ${name}`),
    );

    // Assert
    expect(outside).toEqual([]);
  });

  it("delegates every scene to worldBoundsForViewport unchanged", () => {
    // Arrange
    const viewports = [IPHONE, DESKTOP];

    // Act
    const mismatches = SCENE_IDS.flatMap((id) =>
      viewports
        .filter(
          ({ width, height }) =>
            JSON.stringify(worldBoundsForTarget({ kind: "scene", id }, width, height)) !==
            JSON.stringify(worldBoundsForViewport(id, width, height)),
        )
        .map(({ width, height }) => `${id} ${width}x${height}`),
    );

    // Assert
    expect(mismatches).toEqual([]);
  });

  it("returns the puzzle landscape frame on a desktop viewport", () => {
    // Arrange
    const target = { kind: "puzzle", id: "first-pour" } as const;

    // Act
    const bounds = worldBoundsForTarget(target, DESKTOP.width, DESKTOP.height);

    // Assert
    expect(bounds).toEqual(PUZZLE_VIEW_BOUNDS["first-pour"]);
  });

  it("returns the puzzle portrait frame on an iPhone viewport", () => {
    // Arrange
    const target = { kind: "puzzle", id: "first-pour" } as const;

    // Act
    const bounds = worldBoundsForTarget(target, IPHONE.width, IPHONE.height);

    // Assert
    expect(bounds).toEqual(PUZZLE_PORTRAIT_BOUNDS["first-pour"]);
  });
});

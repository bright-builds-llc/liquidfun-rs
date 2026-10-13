import { describe, expect, it, vi } from "vitest";

import {
  decodeInspection,
  inspectPlacements,
  type RawPlacementInspection,
} from "../src/physics/puzzle-inspector";
import type { PuzzlePlacement } from "../src/puzzle/placements";

const INVALID_INSPECTION_MESSAGE = "Invalid placement inspection";

/** One finite plank outline (x, y pairs in vertex order). */
const PLANK_OUTLINE = [0.1, 3.0, 0.7, 3.2, 0.68, 3.26, 0.08, 3.06];

function outlineLanes(...outlines: number[][]): Float32Array {
  return new Float32Array(outlines.flat());
}

function fakeRawInspection(statuses: number[], outlines: Float32Array): RawPlacementInspection {
  return {
    statuses: () => new Uint8Array(statuses),
    outlines: () => outlines,
    free: vi.fn(),
  };
}

const REFERENCE_PLANK: PuzzlePlacement = { kind: "plank", x: 8, y: 64, angle: 23 };

describe("decodeInspection", () => {
  it("decodes statuses in input order", () => {
    // Arrange
    const outlines = outlineLanes(PLANK_OUTLINE, new Array<number>(8).fill(0));

    // Act
    const inspection = decodeInspection(2, new Uint8Array([0, 1]), outlines);

    // Assert
    expect(inspection.statuses).toEqual([0, 1]);
  });

  it("decodes a geometric outline into points in vertex order", () => {
    // Arrange
    const outlines = outlineLanes(PLANK_OUTLINE, new Array<number>(8).fill(0));

    // Act
    const inspection = decodeInspection(2, new Uint8Array([0, 1]), outlines);

    // Assert
    const [x0, y0, x1, y1, x2, y2, x3, y3] = Array.from(outlines.subarray(0, 8));
    expect(inspection.outlines[0]).toEqual([
      { x: x0, y: y0 },
      { x: x1, y: y1 },
      { x: x2, y: y2 },
      { x: x3, y: y3 },
    ]);
  });

  it("decodes a non-geometric status to an undefined outline", () => {
    // Arrange
    const outlines = outlineLanes(PLANK_OUTLINE, new Array<number>(8).fill(0));

    // Act
    const inspection = decodeInspection(2, new Uint8Array([0, 1]), outlines);

    // Assert
    expect(inspection.outlines[1]).toBeUndefined();
  });

  it("ignores non-finite floats in a non-geometric outline", () => {
    // Arrange
    const outlines = outlineLanes(new Array<number>(8).fill(Number.NaN));

    // Act
    const inspection = decodeInspection(1, new Uint8Array([4]), outlines);

    // Assert
    expect(inspection.outlines).toEqual([undefined]);
  });

  it("decodes an empty inspection", () => {
    // Arrange
    const statuses = new Uint8Array([]);

    // Act
    const inspection = decodeInspection(0, statuses, new Float32Array([]));

    // Assert
    expect(inspection).toEqual({ statuses: [], outlines: [] });
  });

  const invalidCases: ReadonlyArray<[string, number, Uint8Array, Float32Array]> = [
    ["a short status lane", 2, new Uint8Array([0]), outlineLanes(PLANK_OUTLINE, PLANK_OUTLINE)],
    ["a short outline lane", 1, new Uint8Array([0]), new Float32Array(PLANK_OUTLINE.slice(0, 7))],
    ["a long outline lane", 1, new Uint8Array([0]), outlineLanes(PLANK_OUTLINE, [0])],
    ["an unknown status code", 1, new Uint8Array([13]), outlineLanes(PLANK_OUTLINE)],
    [
      "a NaN in a geometric outline",
      1,
      new Uint8Array([0]),
      outlineLanes([Number.NaN, ...PLANK_OUTLINE.slice(1)]),
    ],
    [
      "an Infinity in a geometric outline",
      1,
      new Uint8Array([11]),
      outlineLanes([...PLANK_OUTLINE.slice(0, 7), Number.POSITIVE_INFINITY]),
    ],
    ["a negative count", -1, new Uint8Array([]), new Float32Array([])],
    ["a fractional count", 0.5, new Uint8Array([]), new Float32Array([])],
  ];

  it.each(invalidCases)("%s throws invalid placement inspection", (_name, count, statuses, outlines) => {
    // Arrange: lanes come from the table row.

    // Act
    const act = () => decodeInspection(count, statuses, outlines);

    // Assert
    expect(act).toThrow(INVALID_INSPECTION_MESSAGE);
  });
});

describe("inspectPlacements", () => {
  it("passes the level id and encoded lanes to the raw inspector", () => {
    // Arrange
    const raw = fakeRawInspection([0], outlineLanes(PLANK_OUTLINE));
    const inspect = vi.fn(() => raw);

    // Act
    inspectPlacements(inspect, "first-pour", [REFERENCE_PLANK]);

    // Assert
    expect(inspect).toHaveBeenCalledWith(
      "first-pour",
      new Uint8Array([1]),
      new Int32Array([8, 64, 23]),
    );
  });

  it("returns the decoded inspection", () => {
    // Arrange
    const raw = fakeRawInspection([0], outlineLanes(PLANK_OUTLINE));

    // Act
    const inspection = inspectPlacements(() => raw, "first-pour", [REFERENCE_PLANK]);

    // Assert
    expect(inspection.statuses).toEqual([0]);
  });

  it("frees the raw result once after a successful decode", () => {
    // Arrange
    const raw = fakeRawInspection([0], outlineLanes(PLANK_OUTLINE));

    // Act
    inspectPlacements(() => raw, "first-pour", [REFERENCE_PLANK]);

    // Assert
    expect(raw.free).toHaveBeenCalledTimes(1);
  });

  it("frees the raw result once when decoding throws", () => {
    // Arrange
    const raw = fakeRawInspection([13], outlineLanes(PLANK_OUTLINE));

    // Act
    const act = () => inspectPlacements(() => raw, "first-pour", [REFERENCE_PLANK]);

    // Assert
    expect(act).toThrow(INVALID_INSPECTION_MESSAGE);
    expect(raw.free).toHaveBeenCalledTimes(1);
  });

  it("rejects a non-integer placement before calling the inspector", () => {
    // Arrange
    const inspect = vi.fn(() => fakeRawInspection([], new Float32Array([])));
    const placement: PuzzlePlacement = { kind: "plank", x: 8.5, y: 64, angle: 0 };

    // Act
    const act = () => inspectPlacements(inspect, "first-pour", [placement]);

    // Assert
    expect(act).toThrow("Invalid puzzle placement");
    expect(inspect).not.toHaveBeenCalled();
  });
});

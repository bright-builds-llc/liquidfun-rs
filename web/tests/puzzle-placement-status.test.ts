import { describe, expect, it } from "vitest";

import {
  PLACEMENT_STATUS,
  isGeometricStatus,
  isPlacementStatusCode,
  placementReason,
  type PlacementStatusCode,
} from "../src/puzzle/placement-status";

describe("PLACEMENT_STATUS", () => {
  it("mirrors the Rust PlacementStatus wire codes 0 through 12", () => {
    // Act
    const codes = Object.entries(PLACEMENT_STATUS);

    // Assert
    expect(codes).toEqual([
      ["valid", 0],
      ["unknownKind", 1],
      ["kindNotAllowed", 2],
      ["overBudget", 3],
      ["angleOutOfRange", 4],
      ["positionOutOfRange", 5],
      ["outsideBuildArea", 6],
      ["overlapsWall", 7],
      ["overlapsCupWall", 8],
      ["insideCup", 9],
      ["overlapsWater", 10],
      ["overlapsTool", 11],
      ["overlapsNoPlace", 12],
    ]);
  });
});

describe("isPlacementStatusCode", () => {
  it.each([0, 6, 12])("accepts %i", (value) => {
    expect(isPlacementStatusCode(value)).toBe(true);
  });

  it.each([13, -1, 1.5, Number.NaN])("rejects %s", (value) => {
    expect(isPlacementStatusCode(value)).toBe(false);
  });
});

describe("isGeometricStatus", () => {
  it.each([1, 4, 5] as const)("is false for non-geometric status %i", (code) => {
    expect(isGeometricStatus(code)).toBe(false);
  });

  it.each([0, 2, 3, 6, 7, 8, 9, 10, 11, 12] as const)(
    "is true for geometric status %i",
    (code) => {
      expect(isGeometricStatus(code)).toBe(true);
    },
  );
});

describe("placementReason", () => {
  it.each<[PlacementStatusCode, string]>([
    [1, "Not allowed in this level"],
    [2, "Not allowed in this level"],
    [3, "Over the tool limit"],
    [4, "Can't be placed there"],
    [5, "Can't be placed there"],
    [6, "Outside the build area"],
    [7, "Overlaps a wall"],
    [8, "Overlaps the cup"],
    [9, "Inside the cup"],
    [10, "Overlaps the water"],
    [11, "Overlaps another tool"],
    [12, "In a no-build zone"],
  ])("maps status %i to the UI-SPEC phrase", (code, phrase) => {
    expect(placementReason(code)).toBe(phrase);
  });

  it("throws for the valid status, which has no reason", () => {
    // Act
    const act = () => placementReason(PLACEMENT_STATUS.valid);

    // Assert
    expect(act).toThrow();
  });
});

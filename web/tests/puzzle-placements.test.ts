import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  EMPTY_PLACEMENTS,
  PUZZLE_TEST_PLACEMENTS_GLOBAL,
  encodePlacementLanes,
  maybeReadTestPlacements,
  type PuzzlePlacement,
} from "../src/puzzle/placements";

const WEB_SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");
const SOLUTION_PLANK: PuzzlePlacement = { kind: "plank", x: 8, y: 64, angle: 23 };

function seam(value: unknown): object {
  return { [PUZZLE_TEST_PLACEMENTS_GLOBAL]: value };
}

describe("encodePlacementLanes", () => {
  it("encodes no placements as empty lanes", () => {
    // Act
    const lanes = encodePlacementLanes(EMPTY_PLACEMENTS);

    // Assert
    expect([lanes.kinds.length, lanes.poses.length]).toEqual([0, 0]);
  });

  it("encodes the plank kind code into the kinds lane", () => {
    // Act
    const lanes = encodePlacementLanes([SOLUTION_PLANK]);

    // Assert
    expect(Array.from(lanes.kinds)).toEqual([1]);
  });

  it("encodes x, y and angle into the poses lane", () => {
    // Act
    const lanes = encodePlacementLanes([SOLUTION_PLANK]);

    // Assert
    expect(Array.from(lanes.poses)).toEqual([8, 64, 23]);
  });

  it.each([
    ["a fractional x", { x: 8.7 }],
    ["a NaN y", { y: Number.NaN }],
    ["an infinite angle", { angle: Number.POSITIVE_INFINITY }],
    ["an x above the i32 range", { x: 2 ** 31 }],
    ["an x below the i32 range", { x: -(2 ** 31) - 1 }],
  ])("rejects %s instead of coercing it", (_label, override) => {
    // Arrange
    const placement = { ...SOLUTION_PLANK, ...override };

    // Act
    const act = () => encodePlacementLanes([placement]);

    // Assert
    expect(act).toThrow("Invalid puzzle placement");
  });

  it("accepts both ends of the i32 range", () => {
    // Arrange
    const placement = { ...SOLUTION_PLANK, x: -(2 ** 31), y: 2 ** 31 - 1 };

    // Act
    const lanes = encodePlacementLanes([placement]);

    // Assert
    expect(Array.from(lanes.poses)).toEqual([-(2 ** 31), 2 ** 31 - 1, 23]);
  });

  it("rejects an unknown tool kind", () => {
    // Arrange
    const placement = { ...SOLUTION_PLANK, kind: "valve" } as unknown as PuzzlePlacement;

    // Act
    const act = () => encodePlacementLanes([placement]);

    // Assert
    expect(act).toThrow("Invalid puzzle placement");
  });
});

describe("maybeReadTestPlacements", () => {
  it("returns undefined when the seam global is absent", () => {
    // Act
    const maybePlacements = maybeReadTestPlacements({}, "first-pour");

    // Assert
    expect(maybePlacements).toBeUndefined();
  });

  it("returns the placements for the current level", () => {
    // Arrange
    const source = seam({ level: "first-pour", placements: [{ ...SOLUTION_PLANK }] });

    // Act
    const maybePlacements = maybeReadTestPlacements(source, "first-pour");

    // Assert
    expect(maybePlacements).toEqual([SOLUTION_PLANK]);
  });

  it("copies placements instead of returning the input objects", () => {
    // Arrange
    const input = { ...SOLUTION_PLANK, extra: "ignored" };
    const source = seam({ level: "first-pour", placements: [input] });

    // Act
    const maybePlacements = maybeReadTestPlacements(source, "first-pour");

    // Assert
    expect(maybePlacements?.[0]).not.toBe(input);
  });

  it("rejects placements that name another level", () => {
    // Arrange
    const source = seam({ level: "second-pour", placements: [] });

    // Act
    const act = () => maybeReadTestPlacements(source, "first-pour");

    // Assert
    expect(act).toThrow("Test placements name another level");
  });

  it.each([
    ["null", null],
    ["a string", "plank 8 64 23"],
    ["missing placements", { level: "first-pour" }],
    ["object placements", { level: "first-pour", placements: {} }],
    ["a non-string level", { level: 1, placements: [] }],
    [
      "a placement missing angle",
      { level: "first-pour", placements: [{ kind: "plank", x: 8, y: 64 }] },
    ],
    [
      "a placement with a string x",
      { level: "first-pour", placements: [{ kind: "plank", x: "8", y: 64, angle: 23 }] },
    ],
    ["a null placement", { level: "first-pour", placements: [null] }],
  ])("rejects a malformed seam: %s", (_label, value) => {
    // Arrange
    const source = seam(value);

    // Act
    const act = () => maybeReadTestPlacements(source, "first-pour");

    // Assert
    expect(act).toThrow("Invalid test placements");
  });

  it("applies the lane checks to seam placements", () => {
    // Arrange
    const source = seam({
      level: "first-pour",
      placements: [{ kind: "plank", x: 8.5, y: 64, angle: 23 }],
    });

    // Act
    const act = () => maybeReadTestPlacements(source, "first-pour");

    // Assert
    expect(act).toThrow("Invalid puzzle placement");
  });

  it("never reads placements from the URL or storage", () => {
    // Arrange
    const source = readFileSync(join(WEB_SRC, "puzzle/placements.ts"), "utf8");
    const forbidden = ["location", "URLSearchParams", "localStorage", "sessionStorage", "document."];

    // Act
    const found = forbidden.filter((token) => source.includes(token));

    // Assert
    expect(found).toEqual([]);
  });
});

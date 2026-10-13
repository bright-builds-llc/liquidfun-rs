import { describe, expect, it, vi } from "vitest";

import { parsePuzzleLevelInfo, type RawPuzzleLevelInfo } from "../src/physics/puzzle-session";

const INVALID_INFO_MESSAGE = "Invalid puzzle level info";

type LevelInfoOverrides = {
  toolKinds?: Uint8Array;
  toolCounts?: Uint32Array;
  brimCount?: number;
  parTools?: number;
};

/** first-pour as `levelInfo()` reports it after Phase 39 Plan 01. */
function fakeLevelInfo(overrides: LevelInfoOverrides = {}): RawPuzzleLevelInfo {
  const toolKinds = overrides.toolKinds ?? new Uint8Array([1]);
  const toolCounts = overrides.toolCounts ?? new Uint32Array([2]);
  const brimCount = overrides.brimCount ?? 810;
  const parTools = overrides.parTools ?? 1;
  return {
    levelId: () => "first-pour",
    title: () => "First Pour",
    targetCount: () => 540,
    fullCount: () => 900,
    brimCount: () => brimCount,
    parTools: () => parTools,
    settleSteps: () => 30,
    stepLimit: () => 720,
    cupInterior: () => new Float32Array([1.6, 0, 3.4, 1.8]),
    buildArea: () => new Int32Array([-20, 44, 40, 84]),
    gridMeters: () => 0.05,
    toolKinds: () => toolKinds,
    toolCounts: () => toolCounts,
    free: vi.fn(),
  };
}

describe("parsePuzzleLevelInfo stars and tools", () => {
  it("parses the tool budget from the kind and count lanes", () => {
    // Arrange
    const raw = fakeLevelInfo();

    // Act
    const info = parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(info.toolBudget).toEqual([{ kind: "plank", count: 2 }]);
  });

  it("parses the brim count", () => {
    // Arrange
    const raw = fakeLevelInfo();

    // Act
    const info = parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(info.brimCount).toBe(810);
  });

  it("parses the par tool count", () => {
    // Arrange
    const raw = fakeLevelInfo();

    // Act
    const info = parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(info.parTools).toBe(1);
  });

  it("keeps the build area in integer grid cells", () => {
    // Arrange
    const raw = fakeLevelInfo();

    // Act
    const info = parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(info.buildAreaCells).toEqual([-20, 44, 40, 84]);
  });

  it("accepts a par of zero tools", () => {
    // Arrange
    const raw = fakeLevelInfo({ parTools: 0 });

    // Act
    const info = parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(info.parTools).toBe(0);
  });

  const invalidCases: ReadonlyArray<[string, LevelInfoOverrides]> = [
    ["an unknown kind code", { toolKinds: new Uint8Array([2]) }],
    ["misaligned kind and count lanes", { toolCounts: new Uint32Array([2, 1]) }],
    [
      "a duplicate kind",
      { toolKinds: new Uint8Array([1, 1]), toolCounts: new Uint32Array([1, 1]) },
    ],
    ["empty tool lanes", { toolKinds: new Uint8Array([]), toolCounts: new Uint32Array([]) }],
    ["a count of zero", { toolCounts: new Uint32Array([0]) }],
    ["a count above sixteen", { toolCounts: new Uint32Array([17]) }],
    ["a brim equal to the target", { brimCount: 540 }],
    ["a brim above full", { brimCount: 901 }],
    ["a par above the total budget", { parTools: 3 }],
    ["a fractional par", { parTools: 0.5 }],
    ["a negative par", { parTools: -1 }],
  ];

  it.each(invalidCases)("%s throws invalid level info", (_name, overrides) => {
    // Arrange
    const raw = fakeLevelInfo(overrides);

    // Act
    const act = () => parsePuzzleLevelInfo(raw, "first-pour");

    // Assert
    expect(act).toThrow(INVALID_INFO_MESSAGE);
  });
});

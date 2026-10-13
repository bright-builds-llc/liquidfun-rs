import { describe, expect, it } from "vitest";

import {
  EMPTY_PROGRESS,
  PROGRESS_STORAGE_KEY,
  maybeLevelStars,
  mergeProgress,
  parseProgress,
  serializeProgress,
  withPlacements,
  withStars,
  type ProgressDocument,
  type SavedPlacements,
} from "../src/puzzle/progress";
import type { LevelStars } from "../src/puzzle/stars";

const LEVEL_HASH = "0123456789abcdef";
const SOLUTION_TOOL = { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 };
const SAVED: SavedPlacements = { levelHash: LEVEL_HASH, tools: [SOLUTION_TOOL] };
const NO_STARS_RECORD: LevelStars = { win: false, brim: false, thrifty: false, bestPeakHeldPct: 0 };

function raw(value: unknown): string {
  return JSON.stringify(value);
}

function parsedOk(maybeRaw: string | null): ProgressDocument {
  const parsed = parseProgress(maybeRaw);
  if (parsed.kind !== "ok") {
    throw new Error(`Expected an ok parse, got ${parsed.reason}`);
  }
  return parsed.progress;
}

function firstPourDocument(stars: LevelStars, maybePlacements?: unknown): string {
  const level = maybePlacements === undefined ? stars : { ...stars, placements: maybePlacements };
  return raw({ version: 1, levels: { "first-pour": level } });
}

const WON_BRIM: LevelStars = { win: true, brim: true, thrifty: false, bestPeakHeldPct: 90 };

describe("parseProgress", () => {
  it("uses the documented storage key", () => {
    // Assert
    expect(PROGRESS_STORAGE_KEY).toBe("liquidfun.puzzle-progress.v1");
  });

  it("reads a missing value as empty progress", () => {
    // Act
    const parsed = parseProgress(null);

    // Assert
    expect(parsed).toEqual({ kind: "ok", progress: EMPTY_PROGRESS });
  });

  it("reports corrupt JSON as unusable", () => {
    // Act
    const parsed = parseProgress("{");

    // Assert
    expect(parsed).toEqual({ kind: "unusable", reason: "json" });
  });

  const level = (fields: Record<string, unknown>): string =>
    raw({ version: 1, levels: { "first-pour": fields } });

  it.each([
    ["an array", "[]"],
    ["a number", "1"],
    ["null", "null"],
    ["a missing version", raw({ levels: {} })],
    ["a string version", raw({ version: "1", levels: {} })],
    ["a fractional version", raw({ version: 1.5, levels: {} })],
    ["version 0", raw({ version: 0, levels: {} })],
    ["missing levels", raw({ version: 1 })],
    ["array levels", raw({ version: 1, levels: [] })],
    ["a string win", level({ win: "yes", brim: false, thrifty: false, bestPeakHeldPct: 0 })],
    ["a missing thrifty", level({ win: true, brim: false, bestPeakHeldPct: 50 })],
    ["a negative fill", level({ win: true, brim: false, thrifty: false, bestPeakHeldPct: -1 })],
    ["a fill above 100", level({ win: true, brim: false, thrifty: false, bestPeakHeldPct: 101 })],
    ["a string fill", level({ win: true, brim: false, thrifty: false, bestPeakHeldPct: "50" })],
    ["an array level", raw({ version: 1, levels: { "first-pour": [] } })],
    ["a star without a win", level({ win: false, brim: true, thrifty: false, bestPeakHeldPct: 0 })],
  ])("reports %s as a shape failure", (_name, input) => {
    // Act
    const parsed = parseProgress(input);

    // Assert
    expect(parsed).toEqual({ kind: "unusable", reason: "shape" });
  });

  it.each([
    ["version 2", raw({ version: 2, levels: {} })],
    ["version 7 without levels", raw({ version: 7 })],
  ])("reports %s as a future version", (_name, input) => {
    // Act
    const parsed = parseProgress(input);

    // Assert
    expect(parsed).toEqual({ kind: "unusable", reason: "future-version" });
  });

  it("reads the stars of a known level", () => {
    // Arrange
    const progress = parsedOk(firstPourDocument(WON_BRIM));

    // Act
    const maybeStars = maybeLevelStars(progress, "first-pour");

    // Assert
    expect(maybeStars).toEqual(WON_BRIM);
  });

  it("has no best for a level that was never won", () => {
    // Arrange
    const progress = parsedOk(firstPourDocument(NO_STARS_RECORD, SAVED));

    // Act
    const maybeStars = maybeLevelStars(progress, "first-pour");

    // Assert
    expect(maybeStars).toBeUndefined();
  });

  it("has no best for an absent level", () => {
    // Act
    const maybeStars = maybeLevelStars(EMPTY_PROGRESS, "thick-syrup");

    // Assert
    expect(maybeStars).toBeUndefined();
  });

  it.each([
    ["a string", "tools"],
    ["a missing levelHash", { tools: [] }],
    ["a malformed levelHash", { levelHash: "XYZ", tools: [] }],
    ["non-array tools", { levelHash: LEVEL_HASH, tools: {} }],
  ])("drops placements that are %s and keeps the stars", (_name, placements) => {
    // Act
    const progress = parsedOk(firstPourDocument(WON_BRIM, placements));

    // Assert
    expect(progress.levels["first-pour"]).toEqual({ ...WON_BRIM, maybePlacements: undefined });
  });

  it("keeps valid placements with their raw tools", () => {
    // Arrange
    const tools = [SOLUTION_TOOL, { kind: "unchecked" }];

    // Act
    const progress = parsedOk(firstPourDocument(WON_BRIM, { levelHash: LEVEL_HASH, tools }));

    // Assert
    expect(progress.levels["first-pour"]?.maybePlacements).toEqual({ levelHash: LEVEL_HASH, tools });
  });

  it("writes unknown level ids back unchanged", () => {
    // Arrange
    const input = { version: 1, levels: { "future-level": { anything: [1, 2] } } };

    // Act
    const written = serializeProgress(parsedOk(raw(input)));

    // Assert
    expect(written).toContain('"future-level":{"anything":[1,2]}');
    expect(JSON.parse(written)).toEqual(input);
  });

  it("writes unknown top-level keys back unchanged", () => {
    // Arrange
    const input = { version: 1, levels: {}, note: "x" };

    // Act
    const written = serializeProgress(parsedOk(raw(input)));

    // Assert
    expect(JSON.parse(written)).toEqual(input);
  });

  it("keeps a __proto__ level id as plain data", () => {
    // Arrange
    const input = '{"version":1,"levels":{"__proto__":{"polluted":true}}}';

    // Act
    const written = serializeProgress(parsedOk(input));

    // Assert
    expect(({} as Record<string, unknown>)["polluted"]).toBeUndefined();
    const levels = (JSON.parse(written) as { levels: object }).levels;
    expect(Object.hasOwn(levels, "__proto__")).toBe(true);
  });
});

describe("serializeProgress", () => {
  it("writes known ids in registry order, then unknown ids, then extra keys", () => {
    // Arrange
    const input = raw({
      version: 1,
      extra: true,
      levels: {
        "zz-level": 1,
        "thick-syrup": WON_BRIM,
        "first-pour": { ...WON_BRIM, placements: SAVED, ignored: 3 },
      },
    });

    // Act
    const written = serializeProgress(parsedOk(input));

    // Assert
    expect(written).toBe(
      raw({
        version: 1,
        levels: {
          "first-pour": { ...WON_BRIM, placements: SAVED },
          "thick-syrup": WON_BRIM,
          "zz-level": 1,
        },
        extra: true,
      }),
    );
  });

  it("round-trips stars, placements, unknown ids and extra keys", () => {
    // Arrange
    const progress = parsedOk(
      raw({
        version: 1,
        levels: { "first-pour": { ...WON_BRIM, placements: SAVED }, "jelly-pad": WON_BRIM, other: [1] },
        note: { nested: "x" },
      }),
    );

    // Act
    const reparsed = parseProgress(serializeProgress(progress));

    // Assert
    expect(reparsed).toEqual({ kind: "ok", progress });
  });
});

describe("withStars and withPlacements", () => {
  it("never lowers a star or the best fill", () => {
    // Arrange
    const first = withStars(EMPTY_PROGRESS, "first-pour", {
      win: true,
      brim: false,
      thrifty: false,
      bestPeakHeldPct: 61,
    });

    // Act
    const second = withStars(first, "first-pour", {
      win: true,
      brim: true,
      thrifty: false,
      bestPeakHeldPct: 40,
    });

    // Assert
    expect(maybeLevelStars(second, "first-pour")).toEqual({
      win: true,
      brim: true,
      thrifty: false,
      bestPeakHeldPct: 61,
    });
  });

  it("ignores a loss on a solved level", () => {
    // Arrange
    const solved = withStars(EMPTY_PROGRESS, "first-pour", WON_BRIM);

    // Act
    const after = withStars(solved, "first-pour", NO_STARS_RECORD);

    // Assert
    expect(maybeLevelStars(after, "first-pour")).toEqual(WON_BRIM);
  });

  it("keeps saved placements when stars change", () => {
    // Arrange
    const saved = withPlacements(EMPTY_PROGRESS, "first-pour", SAVED);

    // Act
    const after = withStars(saved, "first-pour", WON_BRIM);

    // Assert
    expect(after.levels["first-pour"]).toEqual({ ...WON_BRIM, maybePlacements: SAVED });
  });

  it("keeps stars when placements change", () => {
    // Arrange
    const solved = withStars(EMPTY_PROGRESS, "first-pour", WON_BRIM);

    // Act
    const after = withPlacements(solved, "first-pour", SAVED);

    // Assert
    expect(after.levels["first-pour"]).toEqual({ ...WON_BRIM, maybePlacements: SAVED });
  });

  it("removes placements and keeps stars", () => {
    // Arrange
    const saved = withPlacements(withStars(EMPTY_PROGRESS, "first-pour", WON_BRIM), "first-pour", SAVED);

    // Act
    const after = withPlacements(saved, "first-pour", undefined);

    // Assert
    expect(after.levels["first-pour"]).toEqual({ ...WON_BRIM, maybePlacements: undefined });
  });

  it("stores placements of an unplayed level with zero stars", () => {
    // Act
    const after = withPlacements(EMPTY_PROGRESS, "jelly-pad", SAVED);

    // Assert
    expect(after.levels["jelly-pad"]).toEqual({ ...NO_STARS_RECORD, maybePlacements: SAVED });
  });

  it("leaves the input document unchanged", () => {
    // Arrange
    const before = withStars(EMPTY_PROGRESS, "first-pour", WON_BRIM);
    const snapshot = serializeProgress(before);

    // Act
    withPlacements(withStars(before, "first-pour", { ...WON_BRIM, thrifty: true }), "first-pour", SAVED);

    // Assert
    expect(serializeProgress(before)).toBe(snapshot);
    expect(serializeProgress(EMPTY_PROGRESS)).toBe('{"version":1,"levels":{}}');
  });
});

describe("mergeProgress", () => {
  it("never lowers stars from another tab", () => {
    // Arrange
    const stored = withStars(EMPTY_PROGRESS, "first-pour", {
      win: true,
      brim: true,
      thrifty: false,
      bestPeakHeldPct: 70,
    });
    const local = withStars(EMPTY_PROGRESS, "first-pour", {
      win: true,
      brim: false,
      thrifty: true,
      bestPeakHeldPct: 90,
    });

    // Act
    const merged = mergeProgress(stored, local);

    // Assert
    expect(maybeLevelStars(merged, "first-pour")).toEqual({
      win: true,
      brim: true,
      thrifty: true,
      bestPeakHeldPct: 90,
    });
  });

  it("keeps levels present on only one side", () => {
    // Arrange
    const stored = withStars(EMPTY_PROGRESS, "first-pour", WON_BRIM);
    const local = withStars(EMPTY_PROGRESS, "float-up", WON_BRIM);

    // Act
    const merged = mergeProgress(stored, local);

    // Assert
    expect(Object.keys(merged.levels)).toEqual(["first-pour", "float-up"]);
  });

  it("prefers stored placements, then local ones", () => {
    // Arrange
    const other: SavedPlacements = { levelHash: LEVEL_HASH, tools: [] };
    const stored = withPlacements(EMPTY_PROGRESS, "first-pour", SAVED);
    const local = withPlacements(withPlacements(EMPTY_PROGRESS, "first-pour", other), "jelly-pad", other);

    // Act
    const merged = mergeProgress(stored, local);

    // Assert
    expect(merged.levels["first-pour"]?.maybePlacements).toEqual(SAVED);
    expect(merged.levels["jelly-pad"]?.maybePlacements).toEqual(other);
  });

  it("takes unknown ids and extra keys from stored", () => {
    // Arrange
    const stored = parsedOk(raw({ version: 1, levels: { later: 1 }, note: "stored" }));
    const local = parsedOk(raw({ version: 1, levels: { sooner: 2 }, note: "local" }));

    // Act
    const merged = mergeProgress(stored, local);

    // Assert
    expect(JSON.parse(serializeProgress(merged))).toEqual({
      version: 1,
      levels: { later: 1 },
      note: "stored",
    });
  });
});

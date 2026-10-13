import { describe, expect, it } from "vitest";

import type { PuzzleId } from "../src/catalog/puzzles";
import { createStarStore } from "../src/puzzle/star-store";
import {
  NO_STARS,
  mergeLevelStars,
  runStars,
  starNames,
  type LevelStars,
  type RunStars,
} from "../src/puzzle/stars";

const ALL_STARS: RunStars = { win: true, brim: true, thrifty: true };

describe("runStars", () => {
  it("earns nothing on a loss even when Brim latched", () => {
    // Arrange
    const input = { won: false, brimLatched: true, placementsAtPlay: 0, parTools: 1 };

    // Act
    const stars = runStars(input);

    // Assert
    expect(stars).toEqual(NO_STARS);
  });

  it("earns every star on a won Brim run within par", () => {
    // Arrange
    const input = { won: true, brimLatched: true, placementsAtPlay: 1, parTools: 1 };

    // Act
    const stars = runStars(input);

    // Assert
    expect(stars).toEqual(ALL_STARS);
  });

  it("withholds Thrifty when placements exceed par", () => {
    // Arrange
    const input = { won: true, brimLatched: true, placementsAtPlay: 2, parTools: 1 };

    // Act
    const stars = runStars(input);

    // Assert
    expect(stars).toEqual({ win: true, brim: true, thrifty: false });
  });

  it("withholds Brim when it did not latch", () => {
    // Arrange
    const input = { won: true, brimLatched: false, placementsAtPlay: 0, parTools: 1 };

    // Act
    const stars = runStars(input);

    // Assert
    expect(stars).toEqual({ win: true, brim: false, thrifty: true });
  });

  it("reads par from its input, not a constant", () => {
    // Arrange
    const input = { won: true, brimLatched: false, placementsAtPlay: 2, parTools: 2 };

    // Act
    const stars = runStars(input);

    // Assert
    expect(stars.thrifty).toBe(true);
  });
});

describe("mergeLevelStars", () => {
  it("records nothing for a loss with no previous best", () => {
    expect(mergeLevelStars(undefined, NO_STARS, 0)).toBeUndefined();
  });

  it("keeps the previous best unchanged after a loss", () => {
    // Arrange
    const previous: LevelStars = { win: true, brim: false, thrifty: true, bestPeakHeldPct: 70 };

    // Act
    const merged = mergeLevelStars(previous, NO_STARS, 95);

    // Assert
    expect(merged).toEqual(previous);
  });

  it("records the first win with its peak held percent", () => {
    // Arrange
    const run: RunStars = { win: true, brim: false, thrifty: true };

    // Act
    const merged = mergeLevelStars(undefined, run, 76);

    // Assert
    expect(merged).toEqual({ win: true, brim: false, thrifty: true, bestPeakHeldPct: 76 });
  });

  it("ORs each criterion so a later run keeps Brim and adds Thrifty", () => {
    // Arrange
    const previous: LevelStars = { win: true, brim: true, thrifty: false, bestPeakHeldPct: 90 };
    const run: RunStars = { win: true, brim: false, thrifty: true };

    // Act
    const merged = mergeLevelStars(previous, run, 70);

    // Assert
    expect(merged).toEqual({ win: true, brim: true, thrifty: true, bestPeakHeldPct: 90 });
  });

  it("keeps the larger peak held percent", () => {
    // Arrange
    const previous: LevelStars = { win: true, brim: false, thrifty: false, bestPeakHeldPct: 70 };
    const run: RunStars = { win: true, brim: false, thrifty: false };

    // Act
    const merged = mergeLevelStars(previous, run, 82);

    // Assert
    expect(merged?.bestPeakHeldPct).toBe(82);
  });

  it("never lowers a stored star across a weaker win", () => {
    // Arrange
    const previous: LevelStars = { win: true, brim: true, thrifty: true, bestPeakHeldPct: 95 };
    const run: RunStars = { win: true, brim: false, thrifty: false };

    // Act
    const merged = mergeLevelStars(previous, run, 61);

    // Assert
    expect(merged).toEqual(previous);
  });

  it("rejects a peak held percent outside 0 to 100", () => {
    expect(() => mergeLevelStars(undefined, ALL_STARS, 101)).toThrow(
      "Invalid peak held percent",
    );
  });
});

describe("starNames", () => {
  it("lists latched stars in Win, Brim, Thrifty order", () => {
    expect(starNames({ win: true, brim: false, thrifty: true })).toEqual(["Win", "Thrifty"]);
  });

  it("lists every star when all latched", () => {
    expect(starNames(ALL_STARS)).toEqual(["Win", "Brim", "Thrifty"]);
  });

  it("lists nothing when none latched", () => {
    expect(starNames(NO_STARS)).toEqual([]);
  });
});

describe("createStarStore", () => {
  it("returns undefined for a level with no recorded run", () => {
    expect(createStarStore().get("first-pour")).toBeUndefined();
  });

  it("stores the merge of a recorded run", () => {
    // Arrange
    const store = createStarStore();
    const run: RunStars = { win: true, brim: true, thrifty: false };

    // Act
    store.record("first-pour", run, 90);

    // Assert
    expect(store.get("first-pour")).toEqual({
      win: true,
      brim: true,
      thrifty: false,
      bestPeakHeldPct: 90,
    });
  });

  it("returns the stored best from record", () => {
    // Arrange
    const store = createStarStore();
    store.record("first-pour", { win: true, brim: true, thrifty: false }, 90);

    // Act
    const best = store.record("first-pour", { win: true, brim: false, thrifty: true }, 70);

    // Assert
    expect(best).toEqual({ win: true, brim: true, thrifty: true, bestPeakHeldPct: 90 });
  });

  it("records nothing for a loss", () => {
    // Arrange
    const store = createStarStore();

    // Act
    const best = store.record("first-pour", NO_STARS, 0);

    // Assert
    expect(best).toBeUndefined();
    expect(store.get("first-pour")).toBeUndefined();
  });

  it("throws for a level id outside PUZZLE_IDS", () => {
    // Arrange
    const store = createStarStore();
    const unknown = "__proto__" as PuzzleId;

    // Act
    const record = () => store.record(unknown, ALL_STARS, 100);

    // Assert
    expect(record).toThrow("Unknown puzzle level");
  });

  it("snapshots to a null-prototype object", () => {
    // Arrange
    const store = createStarStore();
    store.record("first-pour", ALL_STARS, 95);

    // Act
    const snapshot = store.snapshot();

    // Assert
    expect(Object.getPrototypeOf(snapshot)).toBeNull();
  });

  it("snapshots only recorded level ids", () => {
    // Arrange
    const store = createStarStore();
    store.record("first-pour", NO_STARS, 0);

    // Act
    const snapshot = store.snapshot();

    // Assert
    expect(Object.keys(snapshot)).toEqual([]);
  });

  it("snapshots to a JSON-safe shape", () => {
    // Arrange
    const store = createStarStore();
    store.record("first-pour", { win: true, brim: false, thrifty: true }, 76);

    // Act
    const snapshot = store.snapshot();

    // Assert
    expect(JSON.parse(JSON.stringify(snapshot))).toEqual({
      "first-pour": { win: true, brim: false, thrifty: true, bestPeakHeldPct: 76 },
    });
    expect(JSON.parse(JSON.stringify(snapshot))).toEqual(snapshot);
  });

  it("copies entries so snapshot edits do not reach the store", () => {
    // Arrange
    const store = createStarStore();
    store.record("first-pour", { win: true, brim: false, thrifty: false }, 61);
    const snapshot = store.snapshot() as Record<string, { brim: boolean }>;

    // Act
    const entry = snapshot["first-pour"];
    if (entry !== undefined) {
      entry.brim = true;
    }

    // Assert
    expect(store.get("first-pour")?.brim).toBe(false);
  });

  it("keeps two stores independent", () => {
    // Arrange
    const first = createStarStore();
    const second = createStarStore();

    // Act
    first.record("first-pour", ALL_STARS, 95);

    // Assert
    expect(second.get("first-pour")).toBeUndefined();
  });
});

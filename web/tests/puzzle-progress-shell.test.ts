import { createRoot } from "solid-js";
import { afterEach, describe, expect, it } from "vitest";

import {
  PROGRESS_PROBE_KEY,
  UNAVAILABLE_PROGRESS_STORAGE,
  createPuzzleProgress,
  type ProgressStorageProvider,
  type PuzzleProgress,
} from "../src/player/puzzle-progress";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import {
  EMPTY_PROGRESS,
  PROGRESS_STORAGE_KEY,
  maybeLevelStars,
  parseProgress,
  serializeProgress,
  type ProgressDocument,
} from "../src/puzzle/progress";
import type { LevelStars } from "../src/puzzle/stars";
import { createFakeStorage, type FakeStorage } from "./progress-storage-fixture";

const LEVEL_HASH = "0123456789abcdef";
const OTHER_HASH = "fedcba9876543210";
const PLANK: PuzzlePlacement = { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 };
const WON_THRIFTY: LevelStars = { win: true, brim: false, thrifty: true, bestPeakHeldPct: 72 };
const UNUSABLE_SEEDS = [
  ["corrupt JSON", "{"],
  ["wrong shape", '{"version":1,"levels":[]}'],
  ["future version", '{"version":2,"levels":{}}'],
] as const;

let disposers: (() => void)[] = [];

afterEach(() => {
  for (const dispose of disposers) {
    dispose();
  }
  disposers = [];
});

function open(provider: ProgressStorageProvider): PuzzleProgress {
  return createRoot((dispose) => {
    disposers.push(dispose);
    return createPuzzleProgress(provider);
  });
}

function openFake(fake: FakeStorage): PuzzleProgress {
  return open(() => fake.storage);
}

function storedDocument(fake: FakeStorage): ProgressDocument {
  const parsed = parseProgress(fake.values.get(PROGRESS_STORAGE_KEY) ?? null);
  if (parsed.kind !== "ok") {
    throw new Error(`Stored progress is unusable: ${parsed.reason}`);
  }
  return parsed.progress;
}

function firstPourSeed(stars: LevelStars): string {
  return JSON.stringify({ version: 1, levels: { "first-pour": stars } });
}

function progressKeyWrites(fake: FakeStorage): string[] {
  return fake.writes.filter((key) => key === PROGRESS_STORAGE_KEY);
}

describe("puzzle progress startup", () => {
  it("degrades with empty progress when the provider throws", () => {
    // Arrange
    const provider: ProgressStorageProvider = () => {
      throw new DOMException("blocked", "SecurityError");
    };

    // Act
    const progress = open(provider);

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(progress.current()).toBe(EMPTY_PROGRESS);
  });

  it("degrades when the provider returns null", () => {
    // Arrange
    const provider = UNAVAILABLE_PROGRESS_STORAGE;

    // Act
    const progress = open(provider);

    // Assert
    expect(progress.degraded()).toBe(true);
  });

  it("never calls a throwing provider again", () => {
    // Arrange
    let calls = 0;
    const progress = open(() => {
      calls += 1;
      throw new DOMException("blocked", "SecurityError");
    });

    // Act
    progress.recordStars("first-pour", WON_THRIFTY);
    progress.savePlacements("first-pour", LEVEL_HASH, [PLANK]);

    // Assert
    expect(calls).toBe(1);
  });

  it("degrades without touching the progress key when the probe setItem throws", () => {
    // Arrange
    const fake = createFakeStorage();
    fake.failSetItem(true);

    // Act
    const progress = openFake(fake);
    progress.recordStars("first-pour", WON_THRIFTY);

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(fake.writes).toEqual([PROGRESS_PROBE_KEY]);
  });

  it("degrades when the probe removeItem throws", () => {
    // Arrange
    const fake = createFakeStorage();
    fake.failRemoveItem(true);

    // Act
    const progress = openFake(fake);

    // Assert
    expect(progress.degraded()).toBe(true);
  });

  it("degrades when the startup getItem throws", () => {
    // Arrange
    const fake = createFakeStorage();
    fake.failGetItem(true);

    // Act
    const progress = openFake(fake);

    // Assert
    expect(progress.degraded()).toBe(true);
  });

  it.each(UNUSABLE_SEEDS)("leaves a %s value byte-for-byte unchanged", (_label, seed) => {
    // Arrange
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: seed });
    const progress = openFake(fake);

    // Act
    progress.recordStars("first-pour", WON_THRIFTY);
    progress.savePlacements("first-pour", LEVEL_HASH, [PLANK]);

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(fake.values.get(PROGRESS_STORAGE_KEY)).toBe(seed);
    expect(progressKeyWrites(fake)).toEqual([]);
  });
});

describe("puzzle progress with healthy storage", () => {
  it("is not degraded and removes the probe key", () => {
    // Arrange
    const fake = createFakeStorage();

    // Act
    const progress = openFake(fake);

    // Assert
    expect(progress.degraded()).toBe(false);
    expect(fake.values.has(PROGRESS_PROBE_KEY)).toBe(false);
  });

  it("writes a won level's stars and reflects them in current()", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);

    // Act
    progress.recordStars("first-pour", WON_THRIFTY);

    // Assert
    const stored = storedDocument(fake);
    expect(fake.values.get(PROGRESS_STORAGE_KEY)).toBe(serializeProgress(stored));
    expect(maybeLevelStars(stored, "first-pour")).toEqual(WON_THRIFTY);
    expect(maybeLevelStars(progress.current(), "first-pour")).toEqual(WON_THRIFTY);
  });

  it("merges stars another tab wrote after startup", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    fake.values.set(
      PROGRESS_STORAGE_KEY,
      firstPourSeed({ win: true, brim: true, thrifty: false, bestPeakHeldPct: 95 }),
    );
    const union: LevelStars = { win: true, brim: true, thrifty: true, bestPeakHeldPct: 95 };

    // Act
    const returned = progress.recordStars("first-pour", {
      win: true,
      brim: false,
      thrifty: true,
      bestPeakHeldPct: 60,
    });

    // Assert
    expect(returned).toEqual(union);
    expect(maybeLevelStars(storedDocument(fake), "first-pour")).toEqual(union);
  });

  it("degrades without writing when the re-read finds a future version", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    const future = '{"version":2,"levels":{}}';
    fake.values.set(PROGRESS_STORAGE_KEY, future);

    // Act
    const returned = progress.recordStars("first-pour", WON_THRIFTY);

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(progressKeyWrites(fake)).toEqual([]);
    expect(fake.values.get(PROGRESS_STORAGE_KEY)).toBe(future);
    expect(returned).toEqual(WON_THRIFTY);
  });

  it("degrades on a mid-session quota error and stops writing", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    progress.recordStars("first-pour", WON_THRIFTY);
    fake.failSetItem(true);
    const better: LevelStars = { win: true, brim: true, thrifty: false, bestPeakHeldPct: 90 };

    // Act
    const returned = progress.recordStars("first-pour", better);
    progress.recordStars("thick-syrup", WON_THRIFTY);

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(returned).toEqual({ win: true, brim: true, thrifty: true, bestPeakHeldPct: 90 });
    expect(progressKeyWrites(fake)).toHaveLength(2);
  });

  it("writes back unknown level ids unchanged", () => {
    // Arrange
    const fake = createFakeStorage({
      [PROGRESS_STORAGE_KEY]: '{"version":1,"levels":{"future-level":{"x":1}}}',
    });
    const progress = openFake(fake);

    // Act
    progress.recordStars("first-pour", WON_THRIFTY);

    // Assert
    expect(fake.values.get(PROGRESS_STORAGE_KEY)).toContain('"future-level":{"x":1}');
  });
});

describe("puzzle progress placements", () => {
  it("restores saved placements for the same level hash", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    progress.savePlacements("first-pour", LEVEL_HASH, [PLANK]);

    // Act
    const maybeRestored = progress.maybeRestore("first-pour", LEVEL_HASH);

    // Assert
    expect(maybeRestored).toEqual([PLANK]);
  });

  it("discards placements saved for another level hash and keeps the stars", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    progress.recordStars("first-pour", WON_THRIFTY);
    progress.savePlacements("first-pour", LEVEL_HASH, [PLANK]);

    // Act
    const maybeRestored = progress.maybeRestore("first-pour", OTHER_HASH);

    // Assert
    expect(maybeRestored).toBeUndefined();
    const stored = storedDocument(fake);
    expect(stored.levels["first-pour"]?.maybePlacements).toBeUndefined();
    expect(maybeLevelStars(stored, "first-pour")).toEqual(WON_THRIFTY);
  });

  it("returns undefined without writing when nothing was saved", () => {
    // Arrange
    const fake = createFakeStorage();
    const progress = openFake(fake);
    const writesBefore = fake.writes.length;

    // Act
    const maybeRestored = progress.maybeRestore("first-pour", LEVEL_HASH);

    // Assert
    expect(maybeRestored).toBeUndefined();
    expect(fake.writes).toHaveLength(writesBefore);
  });

  it("keeps placements in memory when degraded", () => {
    // Arrange
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: "{" });
    const progress = openFake(fake);

    // Act
    progress.savePlacements("first-pour", LEVEL_HASH, [PLANK]);
    const maybeRestored = progress.maybeRestore("first-pour", LEVEL_HASH);

    // Assert
    expect(maybeRestored).toEqual([PLANK]);
    expect(progressKeyWrites(fake)).toEqual([]);
  });
});

describe("unavailable progress storage", () => {
  it("creates a degraded progress without throwing", () => {
    // Arrange
    const create = (): PuzzleProgress => open(UNAVAILABLE_PROGRESS_STORAGE);

    // Act
    const progress = create();

    // Assert
    expect(progress.degraded()).toBe(true);
    expect(() => progress.recordStars("first-pour", WON_THRIFTY)).not.toThrow();
  });
});

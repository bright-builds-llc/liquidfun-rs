import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { inspectPlacementLanes, loadPuzzleSession } from "../src/physics/puzzle-loader";
import { applyPuzzleEdit } from "../src/player/puzzle-editing";
import type { PuzzleHost } from "../src/player/puzzle-host";
import {
  beginPuzzleRoute,
  editPuzzle,
  leavePuzzle,
  retryPuzzle,
  tryAgainPuzzle,
} from "../src/player/puzzle-lifecycle";
import { placementsInOrder } from "../src/puzzle/editor";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import {
  EMPTY_PROGRESS,
  PROGRESS_STORAGE_KEY,
  maybeLevelStars,
  parseProgress,
  serializeProgress,
  withPlacements,
  withStars,
  type ProgressDocument,
} from "../src/puzzle/progress";
import type { PuzzleState } from "../src/puzzle/reduce-puzzle";
import type { LevelStars } from "../src/puzzle/stars";
import {
  createHost,
  fakeGenerated,
  fakeRawInspection,
  flushMicrotasks,
  stubPuzzleGlobals,
} from "./puzzle-host-fixture";
import { createFakeStorage, type FakeStorage } from "./progress-storage-fixture";

vi.mock("../src/physics/puzzle-loader", () => ({
  loadPuzzleSession: vi.fn(),
  buildPuzzleSessionSync: vi.fn(),
  inspectPlacementLanes: vi.fn(),
}));

/** The fixture fake's `levelInfo().levelHash()`. */
const LIVE_HASH = "0123456789abcdef";
const OTHER_HASH = "fedcba9876543210";
const PLANK: PuzzlePlacement = { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 };
const SEAM_PLANK: PuzzlePlacement = { kind: "plank", x: -4, y: 60, angle: 2, openTick: 0 };
const STARS: LevelStars = { win: true, brim: true, thrifty: false, bestPeakHeldPct: 90 };

const loader = vi.mocked(loadPuzzleSession);
const inspector = vi.mocked(inspectPlacementLanes);

function seededText(levelHash: string, tools: readonly unknown[]): string {
  const withLevelStars = withStars(EMPTY_PROGRESS, "first-pour", STARS);
  return serializeProgress(withPlacements(withLevelStars, "first-pour", { levelHash, tools }));
}

function seededStorage(levelHash: string, tools: readonly unknown[]): FakeStorage {
  return createFakeStorage({ [PROGRESS_STORAGE_KEY]: seededText(levelHash, tools) });
}

function storedDocument(fake: FakeStorage): ProgressDocument {
  const parsed = parseProgress(fake.values.get(PROGRESS_STORAGE_KEY) ?? null);
  if (parsed.kind !== "ok") {
    throw new Error(`Stored progress is unusable: ${parsed.reason}`);
  }
  return parsed.progress;
}

async function enter(fake: FakeStorage): Promise<PuzzleHost> {
  const { host } = createHost({ maybeStorage: () => fake.storage });
  await beginPuzzleRoute(host, "first-pour");
  return host;
}

function toolsNow(host: PuzzleHost): readonly PuzzlePlacement[] {
  return placementsInOrder(host.puzzle.current.editor);
}

function endedWith(host: PuzzleHost): PuzzleState {
  return {
    kind: "ended",
    stepIndex: 840,
    latch: {
      outcome: "lost",
      outcomeStep: 720,
      decidedStep: 720,
      goalCount: 0,
      peakHeldCount: 0,
      maybeBrimStep: undefined,
      goalTarget: 540,
      goalFull: 900,
      brimCount: 810,
      stepLimit: 720,
    },
    tailEndStep: 840,
    held: true,
    editor: host.puzzle.current.editor,
    speed: host.puzzle.current.speed,
  };
}

beforeEach(() => {
  stubPuzzleGlobals();
  loader.mockReset();
  loader.mockImplementation(async () => fakeGenerated());
  inspector.mockReset();
  inspector.mockImplementation((_level, kinds) => fakeRawInspection(Array.from(kinds, () => 0)));
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("restore on level entry", () => {
  it("opens building with the placements saved for the live level hash", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);

    // Act
    const host = await enter(fake);

    // Assert
    expect(host.puzzle.current.kind).toBe("building");
    expect(toolsNow(host)).toEqual([PLANK]);
  });

  it("inspects the restored placements on load", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);

    // Act
    await enter(fake);

    // Assert
    const [, kinds, poses] = inspector.mock.calls.at(-1) ?? [];
    expect(Array.from(kinds ?? [])).toEqual([1]);
    expect(Array.from(poses ?? [])).toEqual([8, 64, 23, 0]);
  });

  it("still builds the preview session with empty lanes", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);

    // Act
    await enter(fake);

    // Assert
    const lanes = loader.mock.calls[0]?.[1];
    expect(lanes?.kinds).toHaveLength(0);
    expect(lanes?.poses).toHaveLength(0);
  });

  it("opens with no tools when the saved level hash differs", async () => {
    // Arrange
    const fake = seededStorage(OTHER_HASH, [PLANK]);

    // Act
    const host = await enter(fake);

    // Assert
    expect(host.puzzle.current.kind).toBe("building");
    expect(toolsNow(host)).toEqual([]);
  });

  it("removes placements saved for another level hash and keeps the stars", async () => {
    // Arrange
    const fake = seededStorage(OTHER_HASH, [PLANK]);

    // Act
    await enter(fake);

    // Assert
    const stored = storedDocument(fake);
    expect(stored.levels["first-pour"]?.maybePlacements).toBeUndefined();
    expect(maybeLevelStars(stored, "first-pour")).toEqual(STARS);
  });

  it("discards a saved list with a malformed entry", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [{ ...PLANK, x: 1.5 }]);

    // Act
    const host = await enter(fake);

    // Assert
    expect(toolsNow(host)).toEqual([]);
    expect(storedDocument(fake).levels["first-pour"]?.maybePlacements).toBeUndefined();
  });

  it("discards a saved list with more than 16 entries", async () => {
    // Arrange
    const seventeen = Array.from({ length: 17 }, (_, index) => ({ ...PLANK, x: index }));
    const fake = seededStorage(LIVE_HASH, seventeen);

    // Act
    const host = await enter(fake);

    // Assert
    expect(toolsNow(host)).toEqual([]);
    expect(storedDocument(fake).levels["first-pour"]?.maybePlacements).toBeUndefined();
  });

  it("lets the test seam win and leaves the saved record unchanged", async () => {
    // Arrange
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: [SEAM_PLANK] },
    });
    const fake = seededStorage(LIVE_HASH, [PLANK]);
    const before = fake.values.get(PROGRESS_STORAGE_KEY);

    // Act
    const host = await enter(fake);

    // Assert
    expect(toolsNow(host)).toEqual([SEAM_PLANK]);
    expect(fake.values.get(PROGRESS_STORAGE_KEY)).toBe(before);
  });
});

describe("in-session rebuilds never restore", () => {
  it("Try again keeps the cleared editor after a restored entry", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);
    const host = await enter(fake);
    applyPuzzleEdit(host, { kind: "clear" }, "commit");
    host.setView({ kind: "failure", maybeFrame: undefined, maybeDetails: undefined });

    // Act
    tryAgainPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(host.puzzle.current.kind).toBe("building");
    expect(toolsNow(host)).toEqual([]);
  });

  it("Retry keeps the editor the run had", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);
    const host = await enter(fake);
    applyPuzzleEdit(host, { kind: "clear" }, "commit");
    host.puzzle.current = endedWith(host);

    // Act
    retryPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(host.puzzle.current.kind).toBe("running");
    expect(toolsNow(host)).toEqual([]);
  });

  it("Edit keeps the editor the run had", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);
    const host = await enter(fake);
    applyPuzzleEdit(host, { kind: "clear" }, "commit");
    host.puzzle.current = endedWith(host);

    // Act
    editPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(host.puzzle.current.kind).toBe("building");
    expect(toolsNow(host)).toEqual([]);
  });
});

describe("restore after leaving", () => {
  it("restores the latest saved placements when the level is entered again", async () => {
    // Arrange
    const fake = seededStorage(LIVE_HASH, [PLANK]);
    const host = await enter(fake);
    leavePuzzle(host);
    host.puzzle.progress.savePlacements("first-pour", LIVE_HASH, [SEAM_PLANK]);

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(toolsNow(host)).toEqual([SEAM_PLANK]);
  });

  it("restores from memory without storage writes when storage is degraded", async () => {
    // Arrange
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: "{" });
    const host = await enter(fake);
    host.puzzle.progress.savePlacements("first-pour", LIVE_HASH, [PLANK]);
    leavePuzzle(host);

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(host.puzzle.progress.degraded()).toBe(true);
    expect(toolsNow(host)).toEqual([PLANK]);
    expect(fake.writes.filter((key) => key === PROGRESS_STORAGE_KEY)).toEqual([]);
  });
});

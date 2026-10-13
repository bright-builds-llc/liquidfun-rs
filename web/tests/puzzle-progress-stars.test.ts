import type { Setter } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildPuzzleSessionSync,
  inspectPlacementLanes,
  loadPuzzleSession,
} from "../src/physics/puzzle-loader";
import type { PuzzleHost } from "../src/player/puzzle-host";
import { beginPuzzleRoute, createPuzzleController } from "../src/player/puzzle-lifecycle";
import { UNAVAILABLE_PROGRESS_STORAGE } from "../src/player/puzzle-progress";
import { playPuzzle } from "../src/player/puzzle-run";
import { createPuzzleRuntime } from "../src/player/puzzle-runtime";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import {
  PROGRESS_STORAGE_KEY,
  maybeLevelStars,
  parseProgress,
  type ProgressDocument,
} from "../src/puzzle/progress";
import { reducePuzzle, type PuzzleState } from "../src/puzzle/reduce-puzzle";
import type { LevelStars } from "../src/puzzle/stars";
import {
  createHost,
  fakeGenerated,
  fakeRawInspection,
  flushMicrotasks,
  lossAtLimit,
  stubPuzzleGlobals,
  winWithBrimFrom,
  type AnimationFrames,
  type FakePuzzleScalars,
} from "./puzzle-host-fixture";
import { createFakeStorage, type FakeStorage } from "./progress-storage-fixture";

vi.mock("../src/physics/puzzle-loader", () => ({
  loadPuzzleSession: vi.fn(),
  buildPuzzleSessionSync: vi.fn(),
  inspectPlacementLanes: vi.fn(),
}));

const ONE_PLANK: readonly PuzzlePlacement[] = [{ kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 }];
const ZERO_QUAD = [
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
] as const;
const LOOP_FRAME_LIMIT = 400;
const STORED_BRIM: LevelStars = { win: true, brim: true, thrifty: false, bestPeakHeldPct: 90 };

const loader = vi.mocked(loadPuzzleSession);
const syncBuild = vi.mocked(buildPuzzleSessionSync);
const inspector = vi.mocked(inspectPlacementLanes);
let frames: AnimationFrames;

function seeded(stars: LevelStars): string {
  return JSON.stringify({ version: 1, levels: { "first-pour": stars } });
}

function storedDocument(fake: FakeStorage): ProgressDocument {
  const parsed = parseProgress(fake.values.get(PROGRESS_STORAGE_KEY) ?? null);
  if (parsed.kind !== "ok") {
    throw new Error(`Stored progress is unusable: ${parsed.reason}`);
  }
  return parsed.progress;
}

/** Enters building on first-pour with one plank and a committed valid inspection. */
async function enterBuilding(fake: FakeStorage): Promise<PuzzleHost> {
  vi.stubGlobal("window", {
    __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: ONE_PLANK },
  });
  loader.mockResolvedValueOnce(fakeGenerated());
  const { host } = createHost({ maybeStorage: () => fake.storage });
  await beginPuzzleRoute(host, "first-pour");
  const { editor } = host.puzzle.current;
  host.puzzle.current = reducePuzzle(host.puzzle.current, {
    kind: "inspected",
    inspection: {
      revision: editor.revision,
      statuses: editor.tools.map(() => 0),
      outlines: editor.tools.map(() => ZERO_QUAD),
    },
    mode: "commit",
  });
  return host;
}

async function runToEnd(host: PuzzleHost, script: (stepIndex: number) => FakePuzzleScalars) {
  syncBuild.mockReturnValueOnce(fakeGenerated("First Pour", script));
  playPuzzle(host);
  frames.runUntilIdle(LOOP_FRAME_LIMIT);
  await flushMicrotasks();
}

/** Records the persisted first-pour stars each time an ended state is published. */
function recordProgressAtEndedPublish(host: PuzzleHost): (LevelStars | undefined)[] {
  const seen: (LevelStars | undefined)[] = [];
  const original = host.puzzle.setState;
  const wrapped = (next: PuzzleState) => {
    if (next.kind === "ended") {
      seen.push(maybeLevelStars(host.puzzle.progress.current(), "first-pour"));
    }
    return original(next);
  };
  (host.puzzle as { setState: Setter<PuzzleState> }).setState = wrapped as Setter<PuzzleState>;
  return seen;
}

beforeEach(() => {
  frames = stubPuzzleGlobals();
  loader.mockReset();
  syncBuild.mockReset();
  inspector.mockReset();
  inspector.mockImplementation((_level, kinds) => fakeRawInspection(Array.from(kinds, () => 0)));
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("star store hydration", () => {
  it("returns persisted won stars before any run", () => {
    // Arrange
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: seeded(STORED_BRIM) });

    // Act
    const { host } = createHost({ maybeStorage: () => fake.storage });

    // Assert
    expect(host.puzzle.starStore.get("first-pour")).toEqual(STORED_BRIM);
  });

  it("does not hydrate a record that was never won", () => {
    // Arrange
    const placementsOnly = { win: false, brim: false, thrifty: false, bestPeakHeldPct: 0 };
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: seeded(placementsOnly) });

    // Act
    const { host } = createHost({ maybeStorage: () => fake.storage });

    // Assert
    expect(host.puzzle.starStore.get("first-pour")).toBeUndefined();
  });

  it("still builds a degraded empty runtime without a progress argument", () => {
    // Arrange
    const create = () => createPuzzleRuntime();

    // Act
    const runtime = create();

    // Assert
    expect(runtime.progress.degraded()).toBe(true);
    expect(runtime.starStore.snapshot()).toEqual({});
  });
});

describe("star write-through", () => {
  it("persists a won run before the ended state is published", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterBuilding(fake);
    const seenAtPublish = recordProgressAtEndedPublish(host);

    // Act
    await runToEnd(host, winWithBrimFrom(2));

    // Assert
    expect(storedDocument(fake).levels["first-pour"]?.win).toBe(true);
    expect(seenAtPublish[0]?.win).toBe(true);
  });

  it("persists no stars for a lost run, only Play's placement save", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterBuilding(fake);

    // Act
    await runToEnd(host, lossAtLimit);

    // Assert
    const state = host.puzzle.current;
    expect(state.kind === "ended" && state.latch.outcome).toBe("lost");
    expect(fake.writes.filter((key) => key === PROGRESS_STORAGE_KEY)).toHaveLength(1);
    expect(maybeLevelStars(storedDocument(fake), "first-pour")).toBeUndefined();
  });

  it("absorbs stars another tab wrote after startup", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterBuilding(fake);
    fake.values.set(
      PROGRESS_STORAGE_KEY,
      seeded({ win: true, brim: false, thrifty: false, bestPeakHeldPct: 95 }),
    );
    const union: LevelStars = { win: true, brim: true, thrifty: true, bestPeakHeldPct: 95 };

    // Act
    await runToEnd(host, winWithBrimFrom(2));

    // Assert
    expect(host.puzzle.starStore.get("first-pour")).toEqual(union);
    expect(host.puzzle.maybeBestStars()).toEqual(union);
  });
});

describe("controller progressDegraded", () => {
  it("is false with healthy storage", () => {
    // Arrange
    const fake = createFakeStorage();
    const { host } = createHost({ maybeStorage: () => fake.storage });

    // Act
    const degraded = createPuzzleController(host).progressDegraded();

    // Assert
    expect(degraded).toBe(false);
  });

  it("is true with unavailable storage", () => {
    // Arrange
    const { host } = createHost({ maybeStorage: UNAVAILABLE_PROGRESS_STORAGE });

    // Act
    const degraded = createPuzzleController(host).progressDegraded();

    // Assert
    expect(degraded).toBe(true);
  });
});

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildPuzzleSessionSync,
  inspectPlacementLanes,
  loadPuzzleSession,
} from "../src/physics/puzzle-loader";
import { applyPuzzleEdit, inspectNow } from "../src/player/puzzle-editing";
import type { PuzzleHost } from "../src/player/puzzle-host";
import { beginPuzzleRoute } from "../src/player/puzzle-lifecycle";
import { playPuzzle } from "../src/player/puzzle-run";
import { saveEditorPlacements } from "../src/player/puzzle-saved-placements";
import { placementsInOrder } from "../src/puzzle/editor";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import {
  EMPTY_PROGRESS,
  PROGRESS_STORAGE_KEY,
  parseProgress,
  serializeProgress,
  withPlacements,
  type ProgressDocument,
} from "../src/puzzle/progress";
import type { BuildingEditEvent } from "../src/puzzle/reduce-puzzle";
import {
  createHost,
  fakeGenerated,
  fakeLevelInfo,
  fakeRawInspection,
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
const PLACE_PLANK = { kind: "place", toolKind: "plank", x: 10, y: 64 } as const;
const PLACED: PuzzlePlacement = { kind: "plank", x: 10, y: 64, angle: 0, openTick: 0 };
const VALVE: PuzzlePlacement = { kind: "valve", x: 10, y: 64, angle: 0, openTick: 4 };

const loader = vi.mocked(loadPuzzleSession);
const syncBuild = vi.mocked(buildPuzzleSessionSync);
const inspector = vi.mocked(inspectPlacementLanes);
/** Status the fake inspector reports for every placement. */
let reportedStatus: number;

function progressWrites(fake: FakeStorage): number {
  return fake.writes.filter((key) => key === PROGRESS_STORAGE_KEY).length;
}

function storedDocument(fake: FakeStorage): ProgressDocument {
  const parsed = parseProgress(fake.values.get(PROGRESS_STORAGE_KEY) ?? null);
  if (parsed.kind !== "ok") {
    throw new Error(`Stored progress is unusable: ${parsed.reason}`);
  }
  return parsed.progress;
}

function storedTools(fake: FakeStorage, id: "first-pour" | "thick-syrup"): readonly unknown[] {
  return storedDocument(fake).levels[id]?.maybePlacements?.tools ?? ["<none>"];
}

async function enterWith(
  fake: FakeStorage,
  maybeSeam: readonly PuzzlePlacement[] | undefined = undefined,
): Promise<PuzzleHost> {
  if (maybeSeam !== undefined) {
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: maybeSeam },
    });
  }
  const { host } = createHost({ maybeStorage: () => fake.storage });
  await beginPuzzleRoute(host, "first-pour");
  return host;
}

/** Enters first-pour and places one plank (one committed save). */
async function enterWithPlacedPlank(fake: FakeStorage): Promise<PuzzleHost> {
  const host = await enterWith(fake);
  applyPuzzleEdit(host, PLACE_PLANK, "commit");
  return host;
}

function thickSyrupGenerated() {
  const generated = fakeGenerated("Thick Syrup");
  generated.levelInfo.mockImplementation(() => ({
    ...fakeLevelInfo("Thick Syrup"),
    levelId: () => "thick-syrup",
  }));
  return generated;
}

beforeEach(() => {
  stubPuzzleGlobals();
  reportedStatus = 0;
  loader.mockReset();
  loader.mockImplementation(async () => fakeGenerated());
  syncBuild.mockReset();
  syncBuild.mockImplementation(() => fakeGenerated());
  inspector.mockReset();
  inspector.mockImplementation((_level, kinds) =>
    fakeRawInspection(Array.from(kinds, () => reportedStatus)),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("saving committed edits", () => {
  it("writes the placed tool once with the live level hash", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWith(fake);

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(progressWrites(fake)).toBe(1);
    const saved = storedDocument(fake).levels["first-pour"]?.maybePlacements;
    expect(saved?.levelHash).toBe(LIVE_HASH);
    expect(saved?.tools).toEqual(placementsInOrder(host.puzzle.current.editor));
  });

  it.each<[string, BuildingEditEvent, readonly PuzzlePlacement[]]>([
    ["rotate", { kind: "rotate", direction: 1 }, [{ ...PLACED, angle: 1 }]],
    ["nudge", { kind: "nudge", dx: 1, dy: 0 }, [{ ...PLACED, x: 11 }]],
    ["remove", { kind: "remove" }, []],
    ["clear", { kind: "clear" }, []],
  ])("writes the new list after a committed %s", async (_name, event, expected) => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWithPlacedPlank(fake);

    // Act
    applyPuzzleEdit(host, event, "commit");

    // Assert
    expect(progressWrites(fake)).toBe(2);
    expect(storedTools(fake, "first-pour")).toEqual(expected);
  });

  it("writes the new open time after a committed valve time change", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWith(fake, [VALVE]);
    applyPuzzleEdit(host, { kind: "select", maybeId: 1 }, "commit");
    const writesBefore = progressWrites(fake);

    // Act
    applyPuzzleEdit(host, { kind: "setValveTime", direction: 1 }, "commit");

    // Assert
    expect(progressWrites(fake)).toBe(writesBefore + 1);
    expect(storedTools(fake, "first-pour")).toEqual([{ ...VALVE, openTick: 5 }]);
  });

  it("writes nothing on live drag frames and once on drag end", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWithPlacedPlank(fake);
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 9, y: 64 }, "live");
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 8, y: 64 }, "live");
    const writesAfterDrag = progressWrites(fake);

    // Act
    inspectNow(host);

    // Assert
    expect(writesAfterDrag).toBe(1);
    expect(progressWrites(fake)).toBe(2);
    expect(storedTools(fake, "first-pour")).toEqual([{ ...PLACED, x: 8 }]);
  });

  it("does not rewrite after commits that leave the placements unchanged", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWithPlacedPlank(fake);

    // Act
    applyPuzzleEdit(host, { kind: "select", maybeId: undefined }, "commit");
    applyPuzzleEdit(host, { kind: "arm", toolKind: "plank" }, "commit");
    applyPuzzleEdit(host, { kind: "disarm" }, "commit");

    // Assert
    expect(host.puzzle.current.editor.maybeArmedKind).toBeUndefined();
    expect(progressWrites(fake)).toBe(1);
  });

  it("saves an invalid placement too", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWith(fake);
    reportedStatus = 11;

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(host.puzzle.current.editor.maybeCommittedInspection?.statuses).toEqual([11]);
    expect(storedTools(fake, "first-pour")).toEqual([PLACED]);
  });
});

describe("saving at Play and across levels", () => {
  it("Play saves restored placements that no edit saved yet", async () => {
    // Arrange
    const seeded = withPlacements(EMPTY_PROGRESS, "first-pour", {
      levelHash: LIVE_HASH,
      tools: [PLACED],
    });
    const fake = createFakeStorage({ [PROGRESS_STORAGE_KEY]: serializeProgress(seeded) });
    const host = await enterWith(fake);
    const writesBefore = progressWrites(fake);

    // Act
    playPuzzle(host);

    // Assert
    expect(host.puzzle.current.kind).toBe("running");
    expect(writesBefore).toBe(0);
    expect(progressWrites(fake)).toBe(1);
    expect(storedTools(fake, "first-pour")).toEqual([PLACED]);
  });

  it("saves a second level whose placements match the first level's", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWithPlacedPlank(fake);
    applyPuzzleEdit(host, { kind: "clear" }, "commit");
    loader.mockImplementationOnce(async () => thickSyrupGenerated());
    await beginPuzzleRoute(host, "thick-syrup");

    // Act
    applyPuzzleEdit(host, { kind: "arm", toolKind: "plank" }, "commit");

    // Assert
    expect(storedTools(fake, "first-pour")).toEqual([]);
    expect(storedTools(fake, "thick-syrup")).toEqual([]);
  });

  it("saves nothing and does not throw before a level loads", () => {
    // Arrange
    const fake = createFakeStorage();
    const { host } = createHost({ maybeStorage: () => fake.storage });

    // Act
    const save = () => saveEditorPlacements(host);

    // Assert
    expect(save).not.toThrow();
    expect(progressWrites(fake)).toBe(0);
  });

  it("degrades on a storage failure without reaching failScene", async () => {
    // Arrange
    const fake = createFakeStorage();
    const host = await enterWith(fake);
    fake.failSetItem(true);

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(host.failScene).not.toHaveBeenCalled();
    expect(host.puzzle.progress.degraded()).toBe(true);
    expect(placementsInOrder(host.puzzle.current.editor)).toEqual([PLACED]);
  });
});

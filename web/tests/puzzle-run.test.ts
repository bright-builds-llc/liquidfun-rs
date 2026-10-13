import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildPuzzleSessionSync,
  inspectPlacementLanes,
  loadPuzzleSession,
} from "../src/physics/puzzle-loader";
import type { PuzzleHost } from "../src/player/puzzle-host";
import { beginPuzzleRoute, leavePuzzle } from "../src/player/puzzle-lifecycle";
import {
  cycleSpeed,
  editPuzzle,
  playPuzzle,
  retryPuzzle,
  stopPuzzle,
} from "../src/player/puzzle-run";
import type { PlacementStatusCode } from "../src/puzzle/placement-status";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import { reducePuzzle } from "../src/puzzle/reduce-puzzle";
import { fnv1a64Lanes, hex16 } from "../src/puzzle/frame-hash";
import {
  createHost,
  fakeGenerated,
  fakeRawInspection,
  flushMicrotasks,
  lossAtLimit,
  particlePositionsAt,
  stubPuzzleGlobals,
  winWithBrimFrom,
  winWithoutBrimFrom,
  type AnimationFrames,
} from "./puzzle-host-fixture";

vi.mock("../src/physics/puzzle-loader", () => ({
  loadPuzzleSession: vi.fn(),
  buildPuzzleSessionSync: vi.fn(),
  inspectPlacementLanes: vi.fn(),
}));

const SEAM_PLACEMENTS: readonly PuzzlePlacement[] = [
  { kind: "plank", x: 12, y: 64, angle: 0 },
  { kind: "plank", x: 8, y: 64, angle: 23 },
];
const ONE_PLANK: readonly PuzzlePlacement[] = [{ kind: "plank", x: 8, y: 64, angle: 23 }];
const ZERO_QUAD = [
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
] as const;
const LOOP_FRAME_LIMIT = 400;

const loader = vi.mocked(loadPuzzleSession);
const syncBuild = vi.mocked(buildPuzzleSessionSync);
const inspector = vi.mocked(inspectPlacementLanes);
let frames: AnimationFrames;

function stubSeam(placements: readonly PuzzlePlacement[]): void {
  vi.stubGlobal("window", {
    __liquidfunPuzzleTestPlacements: { level: "first-pour", placements },
  });
}

/** Stores a committed inspection for the current revision, as the shell would. */
function commitInspection(host: PuzzleHost, status: PlacementStatusCode): void {
  const { editor } = host.puzzle.current;
  host.puzzle.current = reducePuzzle(host.puzzle.current, {
    kind: "inspected",
    inspection: {
      revision: editor.revision,
      statuses: editor.tools.map(() => status),
      outlines: editor.tools.map(() => ZERO_QUAD),
    },
    mode: "commit",
  });
}

async function enterBuilding(placements: readonly PuzzlePlacement[] = ONE_PLANK) {
  stubSeam(placements);
  const preview = fakeGenerated();
  loader.mockResolvedValueOnce(preview);
  const { host } = createHost();
  await beginPuzzleRoute(host, "first-pour");
  commitInspection(host, 0);
  return { host, preview };
}

function lanesOf(call: unknown[] | undefined) {
  const lanes = call?.[1] as { kinds: Uint8Array; poses: Int32Array } | undefined;
  return { kinds: Array.from(lanes?.kinds ?? [99]), poses: Array.from(lanes?.poses ?? [99]) };
}

async function winOnce(host: PuzzleHost): Promise<void> {
  syncBuild.mockReturnValueOnce(fakeGenerated("First Pour", winWithBrimFrom(2)));
  playPuzzle(host);
  frames.runUntilIdle(LOOP_FRAME_LIMIT);
  await flushMicrotasks();
}

beforeEach(() => {
  frames = stubPuzzleGlobals();
  loader.mockReset();
  syncBuild.mockReset();
  inspector.mockReset();
  inspector.mockImplementation((_level, kinds) =>
    fakeRawInspection(Array.from(kinds, () => 0)),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("preview session", () => {
  it("builds the step-0 preview with empty lanes and seeds the editor from the seam", async () => {
    // Arrange
    stubSeam(SEAM_PLACEMENTS);
    loader.mockResolvedValueOnce(fakeGenerated());
    const { host } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const state = host.puzzle.state();
    expect(state.kind).toBe("building");
    expect(lanesOf(loader.mock.calls[0])).toEqual({ kinds: [], poses: [] });
    expect(state.editor.tools.map((tool) => tool.placement)).toEqual(SEAM_PLACEMENTS);
  });

  it("publishes the level info and the stored best on load", async () => {
    // Act
    const { host } = await enterBuilding();

    // Assert
    expect(host.puzzle.maybeLevelInfo()?.parTools).toBe(1);
    expect(host.puzzle.maybeBestStars()).toBeUndefined();
  });
});

describe("Play", () => {
  it("builds the run session from canonical placements, frees the preview and runs", async () => {
    // Arrange
    const { host, preview } = await enterBuilding(SEAM_PLACEMENTS);
    syncBuild.mockReturnValueOnce(fakeGenerated());

    // Act
    playPuzzle(host);

    // Assert
    expect(lanesOf(syncBuild.mock.calls[0])).toEqual({
      kinds: [1, 1],
      poses: [8, 64, 23, 12, 64, 0],
    });
    expect(preview.free).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state().kind).toBe("running");
    expect(host.view().kind).toBe("playing");
    expect(frames.callbacks.size).toBe(1);
  });

  it("does nothing while an inspection reports an invalid tool", async () => {
    // Arrange
    const { host, preview } = await enterBuilding();
    commitInspection(host, 11);

    // Act
    playPuzzle(host);

    // Assert
    expect(syncBuild).not.toHaveBeenCalled();
    expect(preview.free).not.toHaveBeenCalled();
    expect(host.puzzle.current.kind).toBe("building");
  });

  it("routes a throwing run build to failScene without throwing", async () => {
    // Arrange
    const { host } = await enterBuilding();
    const buildError = new Error("build failed");
    syncBuild.mockImplementationOnce(() => {
      throw buildError;
    });

    // Act
    const play = () => playPuzzle(host);

    // Assert
    expect(play).not.toThrow();
    expect(host.failScene).toHaveBeenCalledWith(buildError);
  });
});

describe("Stop", () => {
  it("returns to building on a fresh empty preview with the tools kept", async () => {
    // Arrange
    const { host } = await enterBuilding(SEAM_PLACEMENTS);
    const run = fakeGenerated();
    syncBuild.mockReturnValueOnce(run).mockReturnValueOnce(fakeGenerated());
    playPuzzle(host);
    const placements = host.puzzle.current.editor.tools.map((tool) => tool.placement);

    // Act
    stopPuzzle(host);

    // Assert
    const state = host.puzzle.state();
    expect(state.kind).toBe("building");
    expect(state.editor.tools.map((tool) => tool.placement)).toEqual(placements);
    expect(lanesOf(syncBuild.mock.calls[1])).toEqual({ kinds: [], poses: [] });
    expect(run.free).toHaveBeenCalledTimes(1);
    expect(frames.callbacks.size).toBe(0);
    expect(host.view().kind).toBe("paused");
  });

  it("records the latched Win and Thrifty stars when stopped after a win", async () => {
    // Arrange
    const { host } = await enterBuilding();
    syncBuild
      .mockReturnValueOnce(fakeGenerated("First Pour", winWithoutBrimFrom(2)))
      .mockReturnValueOnce(fakeGenerated());
    playPuzzle(host);
    frames.runNext();
    frames.runNext();

    // Act
    stopPuzzle(host);

    // Assert
    expect(host.puzzle.maybeBestStars()).toEqual({
      win: true,
      brim: false,
      thrifty: true,
      bestPeakHeldPct: 66,
    });
  });

  it("records nothing when stopped before a win", async () => {
    // Arrange
    const { host } = await enterBuilding();
    syncBuild.mockReturnValueOnce(fakeGenerated()).mockReturnValueOnce(fakeGenerated());
    playPuzzle(host);
    frames.runNext();
    frames.runNext();

    // Act
    stopPuzzle(host);

    // Assert
    expect(host.puzzle.starStore.get("first-pour")).toBeUndefined();
    expect(host.puzzle.maybeBestStars()).toBeUndefined();
  });
});

describe("decision and stars", () => {
  it("records the run's stars and peak held percent once when a won run is decided", async () => {
    // Arrange
    const { host } = await enterBuilding();
    const record = vi.spyOn(host.puzzle.starStore, "record");

    // Act
    await winOnce(host);

    // Assert
    expect(host.puzzle.current.kind).toBe("ended");
    expect(record).toHaveBeenCalledTimes(1);
    expect(host.puzzle.maybeBestStars()).toEqual({
      win: true,
      brim: true,
      thrifty: true,
      bestPeakHeldPct: 90,
    });
  });

  it("keeps the best stars after a later losing Retry", async () => {
    // Arrange
    const { host } = await enterBuilding();
    await winOnce(host);
    loader.mockResolvedValueOnce(fakeGenerated("First Pour", lossAtLimit));

    // Act
    retryPuzzle(host);
    await flushMicrotasks();
    frames.runUntilIdle(LOOP_FRAME_LIMIT);

    // Assert
    const state = host.puzzle.current;
    expect(state.kind === "ended" && state.latch.outcome).toBe("lost");
    expect(host.puzzle.maybeBestStars()).toEqual({
      win: true,
      brim: true,
      thrifty: true,
      bestPeakHeldPct: 90,
    });
  });

  it("publishes the held frame's FNV-1a 64 hash once the run is held", async () => {
    // Arrange
    const { host } = await enterBuilding();
    const expected = hex16(
      fnv1a64Lanes([particlePositionsAt(122), new Float32Array(), new Float32Array()]),
    );

    // Act
    await winOnce(host);

    // Assert
    expect(host.puzzle.current.kind === "ended" && host.puzzle.current.held).toBe(true);
    expect(host.puzzle.maybeHeldHash()).toBe(expected);
  });
});

describe("Retry and Edit", () => {
  it("Retry rebuilds with the run lanes, lands running and clears the held hash", async () => {
    // Arrange
    const { host } = await enterBuilding(SEAM_PLACEMENTS);
    await winOnce(host);
    loader.mockResolvedValueOnce(fakeGenerated());

    // Act
    retryPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(lanesOf(loader.mock.calls[1])).toEqual({ kinds: [1, 1], poses: [8, 64, 23, 12, 64, 0] });
    expect(host.puzzle.state().kind).toBe("running");
    expect(host.puzzle.state().editor.tools).toHaveLength(2);
    expect(host.puzzle.maybeHeldHash()).toBeUndefined();
  });

  it("Edit rebuilds an empty preview, lands building and clears the held hash", async () => {
    // Arrange
    const { host } = await enterBuilding(SEAM_PLACEMENTS);
    await winOnce(host);
    loader.mockResolvedValueOnce(fakeGenerated());

    // Act
    editPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(lanesOf(loader.mock.calls[1])).toEqual({ kinds: [], poses: [] });
    expect(host.puzzle.state().kind).toBe("building");
    expect(host.puzzle.state().editor.tools).toHaveLength(2);
    expect(host.puzzle.maybeHeldHash()).toBeUndefined();
  });

  it("Play clears a previous held hash", async () => {
    // Arrange
    const { host } = await enterBuilding();
    host.puzzle.setMaybeHeldHash("0123456789abcdef");
    syncBuild.mockReturnValueOnce(fakeGenerated());

    // Act
    playPuzzle(host);

    // Assert
    expect(host.puzzle.maybeHeldHash()).toBeUndefined();
  });
});

describe("page session", () => {
  it("keeps the star store across leavePuzzle and a new route entry", async () => {
    // Arrange
    const { host } = await enterBuilding();
    await winOnce(host);
    loader.mockResolvedValueOnce(fakeGenerated());

    // Act
    leavePuzzle(host);
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(host.puzzle.maybeBestStars()?.win).toBe(true);
    expect(host.puzzle.maybeHeldHash()).toBeUndefined();
  });

  it("cycles the speed and resets it to 1× on route entry", async () => {
    // Arrange
    const { host } = await enterBuilding();
    cycleSpeed(host);
    cycleSpeed(host);
    const cycled = host.puzzle.state().speed;
    loader.mockResolvedValueOnce(fakeGenerated());

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(cycled).toBe(4);
    expect(host.puzzle.state().speed).toBe(1);
  });

  it("leavePuzzle clears the level info but not the store", async () => {
    // Arrange
    const { host } = await enterBuilding();
    await winOnce(host);

    // Act
    leavePuzzle(host);

    // Assert
    expect(host.puzzle.maybeLevelInfo()).toBeUndefined();
    expect(host.puzzle.starStore.get("first-pour")?.win).toBe(true);
  });
});

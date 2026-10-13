import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  buildPuzzleSessionSync,
  inspectPlacementLanes,
  loadPuzzleSession,
} from "../src/physics/puzzle-loader";
import { applyPuzzleEdit, inspectNow } from "../src/player/puzzle-editing";
import type { PuzzleHost } from "../src/player/puzzle-host";
import { beginPuzzleRoute, createPuzzleController } from "../src/player/puzzle-lifecycle";
import { playPuzzle, stopPuzzle } from "../src/player/puzzle-run";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import type { PuzzleState } from "../src/puzzle/reduce-puzzle";
import {
  createHost,
  fakeGenerated,
  fakeRawInspection,
  stubPuzzleGlobals,
  type AnimationFrames,
} from "./puzzle-host-fixture";

vi.mock("../src/physics/puzzle-loader", () => ({
  loadPuzzleSession: vi.fn(),
  buildPuzzleSessionSync: vi.fn(),
  inspectPlacementLanes: vi.fn(),
}));

const PLACE_PLANK = { kind: "place", toolKind: "plank", x: 10, y: 64 } as const;
const TWO_PLANKS: readonly PuzzlePlacement[] = [
  { kind: "plank", x: 8, y: 64, angle: 0 },
  { kind: "plank", x: 20, y: 64, angle: 0 },
];

const loader = vi.mocked(loadPuzzleSession);
const syncBuild = vi.mocked(buildPuzzleSessionSync);
const inspector = vi.mocked(inspectPlacementLanes);
let frames: AnimationFrames;
/** Status the fake inspector reports for every placement. */
let reportedStatus: number;

async function enterBuilding(placements: readonly PuzzlePlacement[] = []): Promise<PuzzleHost> {
  vi.stubGlobal("window", {
    __liquidfunPuzzleTestPlacements: { level: "first-pour", placements },
  });
  loader.mockResolvedValueOnce(fakeGenerated());
  const { host } = createHost();
  await beginPuzzleRoute(host, "first-pour");
  inspector.mockClear();
  vi.mocked(host.drawSceneFrame).mockClear();
  return host;
}

function selectFirst(host: PuzzleHost): void {
  applyPuzzleEdit(host, { kind: "select", maybeId: 1 }, "commit");
}

beforeEach(() => {
  frames = stubPuzzleGlobals();
  reportedStatus = 0;
  loader.mockReset();
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

describe("committed edits", () => {
  it("dispatches, inspects in creation order, redraws once, publishes and announces", async () => {
    // Arrange
    const host = await enterBuilding();

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    const { editor } = host.puzzle.state();
    expect(Array.from(inspector.mock.calls[0]?.[2] ?? [])).toEqual([10, 64, 0]);
    expect(editor.tools).toHaveLength(1);
    expect(editor.maybeCommittedInspection?.revision).toBe(editor.revision);
    expect(host.drawSceneFrame).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state()).toBe(host.puzzle.current);
    expect(host.puzzle.announcement()).toBe("Plank placed.");
  });

  it("announces the reason for an overlapping placement", async () => {
    // Arrange
    const host = await enterBuilding();
    reportedStatus = 11;

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(host.puzzle.announcement()).toBe("Plank placed. Overlaps another tool.");
  });

  it("ignores edits outside building", async () => {
    // Arrange
    const host = await enterBuilding();
    playPuzzle(host);
    const running = host.puzzle.current;
    inspector.mockClear();

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(host.puzzle.current).toBe(running);
    expect(inspector).not.toHaveBeenCalled();
    expect(host.puzzle.announcement()).toBe("");
  });

  it("routes an inspector failure to failScene", async () => {
    // Arrange
    const host = await enterBuilding();
    const inspectError = new Error("inspect failed");
    inspector.mockImplementationOnce(() => {
      throw inspectError;
    });

    // Act
    applyPuzzleEdit(host, PLACE_PLANK, "commit");

    // Assert
    expect(host.failScene).toHaveBeenCalledWith(inspectError);
  });
});

describe("live edits (D-02, UI-D14)", () => {
  it("publishes at once and coalesces inspections to one animation frame", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);

    // Act
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 9, y: 64 }, "live");
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 10, y: 64 }, "live");

    // Assert
    expect(host.puzzle.state().editor.tools[0]?.placement.x).toBe(10);
    expect(frames.callbacks.size).toBe(1);
    expect(inspector).not.toHaveBeenCalled();
  });

  it("runs the coalesced live inspection without touching the committed snapshot", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    const committed = host.puzzle.current.editor.maybeCommittedInspection;
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 9, y: 64 }, "live");
    reportedStatus = 11;

    // Act
    frames.runNext();

    // Assert
    const { editor } = host.puzzle.current;
    expect(inspector).toHaveBeenCalledTimes(1);
    expect(editor.maybeInspection?.revision).toBe(editor.revision);
    expect(editor.maybeCommittedInspection).toBe(committed);
    expect(host.puzzle.announcement()).toBe("");
    expect(host.drawSceneFrame).toHaveBeenCalledTimes(1);
  });

  it("cancels the pending live frame when a commit inspects now", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 9, y: 64 }, "live");

    // Act
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 10, y: 64 }, "commit");

    // Assert
    const { editor } = host.puzzle.current;
    expect(frames.callbacks.size).toBe(0);
    expect(inspector).toHaveBeenCalledTimes(1);
    expect(editor.maybeCommittedInspection?.revision).toBe(editor.revision);
  });

  it("inspectNow commits the inspection after a drag", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    applyPuzzleEdit(host, { kind: "move", id: 1, x: 9, y: 64 }, "live");

    // Act
    inspectNow(host);

    // Assert
    const { editor } = host.puzzle.state();
    expect(editor.maybeCommittedInspection?.revision).toBe(editor.revision);
    expect(frames.callbacks.size).toBe(0);
  });
});

describe("building entry", () => {
  it("commits an inspection before the building state is first published", async () => {
    // Arrange
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: TWO_PLANKS },
    });
    loader.mockResolvedValueOnce(fakeGenerated());
    const { host } = createHost();
    const published: PuzzleState[] = [];
    vi.spyOn(host.puzzle, "setState").mockImplementation((value) => {
      published.push(value as PuzzleState);
      return value as PuzzleState;
    });

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const firstBuilding = published.find((state) => state.kind === "building");
    expect(firstBuilding?.editor.maybeCommittedInspection?.statuses).toEqual([0, 0]);
  });

  it("commits an inspection and announces when a run is stopped", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    playPuzzle(host);
    inspector.mockClear();

    // Act
    stopPuzzle(host);

    // Assert
    const { editor } = host.puzzle.state();
    expect(inspector).toHaveBeenCalledTimes(1);
    expect(editor.maybeCommittedInspection?.revision).toBe(editor.revision);
    expect(host.puzzle.announcement()).toBe("Stopped. Your tools are kept.");
  });
});

describe("controller editing commands", () => {
  it("places at the build-area centre with angle 0 and selects it", async () => {
    // Arrange
    const host = await enterBuilding();
    const controller = createPuzzleController(host);

    // Act
    controller.placeAtCenter("plank");

    // Assert
    const { editor } = host.puzzle.state();
    expect(editor.tools.map((tool) => tool.placement)).toEqual([
      { kind: "plank", x: 10, y: 64, angle: 0 },
    ]);
    expect(editor.maybeSelectedId).toBe(editor.tools[0]?.id);
  });

  it("toggleArm arms, then disarms with a cancel announcement", async () => {
    // Arrange
    const host = await enterBuilding();
    const controller = createPuzzleController(host);
    controller.toggleArm("plank");
    const armed = host.puzzle.state().editor.maybeArmedKind;

    // Act
    controller.toggleArm("plank");

    // Assert
    expect(armed).toBe("plank");
    expect(host.puzzle.state().editor.maybeArmedKind).toBeUndefined();
    expect(host.puzzle.announcement()).toBe("Placement canceled.");
  });

  it("rotateSelected, removeSelected and clear announce their edits", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    const controller = createPuzzleController(host);
    selectFirst(host);
    const announcements: string[] = [];

    // Act
    controller.rotateSelected(-1);
    announcements.push(host.puzzle.announcement());
    controller.removeSelected();
    announcements.push(host.puzzle.announcement());
    controller.clear();
    announcements.push(host.puzzle.announcement());

    // Assert
    expect(announcements).toEqual(["Rotated to −15°.", "Plank removed.", "All tools cleared."]);
  });
});

describe("key commands (D-23)", () => {
  it("nudges the selected tool and reports the key handled", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    selectFirst(host);

    // Act
    const handled = createPuzzleController(host).command({ kind: "nudge", dx: -1, dy: 0 });

    // Assert
    expect(handled).toBe(true);
    expect(host.puzzle.state().editor.tools[0]?.placement.x).toBe(7);
  });

  it("leaves a nudge unhandled without a selection", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);

    // Act
    const handled = createPuzzleController(host).command({ kind: "nudge", dx: -1, dy: 0 });

    // Assert
    expect(handled).toBe(false);
  });

  it("togglePlay plays in building and stops while running", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    const controller = createPuzzleController(host);

    // Act
    const played = controller.command({ kind: "togglePlay" });
    const phaseAfterPlay = host.puzzle.state().kind;
    const stopped = controller.command({ kind: "togglePlay" });

    // Assert
    expect([played, phaseAfterPlay, stopped]).toEqual([true, "running", true]);
    expect(host.puzzle.state().kind).toBe("building");
  });

  it("togglePlay is handled but does not play while a tool is invalid", async () => {
    // Arrange
    reportedStatus = 8;
    const host = await enterBuilding(TWO_PLANKS);

    // Act
    const handled = createPuzzleController(host).command({ kind: "togglePlay" });

    // Assert
    expect(handled).toBe(true);
    expect(host.puzzle.state().kind).toBe("building");
    expect(syncBuild).not.toHaveBeenCalled();
  });

  it("cycles the selection and lets focus leave after the last tool", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    const controller = createPuzzleController(host);

    // Act
    const next = { kind: "cycleSelection", direction: 1 } as const;
    const results = [1, 2, 3].map(() => controller.command(next));

    // Assert
    expect(results).toEqual([true, true, false]);
    expect(host.puzzle.state().editor.maybeSelectedId).toBe(2);
    expect(host.puzzle.announcement()).toBe("Plank 2 of 2 selected.");
  });

  it("escape disarms and deselects in building", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    const controller = createPuzzleController(host);
    selectFirst(host);
    controller.toggleArm("plank");

    // Act
    const handled = controller.command({ kind: "escape" });

    // Assert
    const { editor } = host.puzzle.state();
    expect(handled).toBe(true);
    expect(editor.maybeArmedKind).toBeUndefined();
    expect(editor.maybeSelectedId).toBeUndefined();
  });

  it("handles no command while loading", () => {
    // Arrange
    const { host } = createHost();
    const controller = createPuzzleController(host);

    // Act
    const handled = [
      controller.command({ kind: "togglePlay" }),
      controller.command({ kind: "escape" }),
      controller.command({ kind: "remove" }),
    ];

    // Assert
    expect(handled).toEqual([false, false, false]);
  });

  it("handles no command once ended", async () => {
    // Arrange
    const host = await enterBuilding(TWO_PLANKS);
    selectFirst(host);
    const { editor, speed } = host.puzzle.current;
    host.puzzle.current = {
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
      editor,
      speed,
    };
    const controller = createPuzzleController(host);

    // Act
    const handled = [
      controller.command({ kind: "togglePlay" }),
      controller.command({ kind: "nudge", dx: 1, dy: 0 }),
      controller.command({ kind: "escape" }),
      controller.command({ kind: "cycleSelection", direction: 1 }),
    ];

    // Assert
    expect(handled).toEqual([false, false, false, false]);
  });
});

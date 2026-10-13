import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import {
  EMPTY_EDITOR,
  canonicalPlacements,
  editorFromPlacements,
  type BuildingEditor,
  type EditorInspection,
} from "../src/puzzle/editor";
import type { PlacementStatusCode, Quad } from "../src/puzzle/placement-status";
import { encodePlacementLanes } from "../src/puzzle/placements";
import {
  INITIAL_PUZZLE_STATE,
  canPlay,
  reducePuzzle,
  type PuzzleEditEvent,
  type PuzzleEvent,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";

const BUDGET = [{ kind: "plank", count: 2 }] as const;
const QUAD: Quad = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 0.1 },
  { x: 0, y: 0.1 },
];

function status(overrides: Partial<PuzzleFrameStatus> = {}): PuzzleFrameStatus {
  return {
    goalCount: 0,
    goalTarget: 540,
    goalFull: 900,
    outcome: "running",
    stepLimit: 720,
    peakHeldCount: 0,
    brimCount: 810,
    ...overrides,
  };
}

function building(): PuzzleState {
  return reducePuzzle(INITIAL_PUZZLE_STATE, {
    kind: "loaded",
    stepIndex: 0,
    status: status(),
    budget: BUDGET,
  });
}

function apply(state: PuzzleState, events: readonly PuzzleEvent[]): PuzzleState {
  return events.reduce(reducePuzzle, state);
}

function inspection(
  editor: BuildingEditor,
  statuses: readonly PlacementStatusCode[],
): EditorInspection {
  return { revision: editor.revision, statuses, outlines: statuses.map(() => QUAD) };
}

function inspected(
  state: PuzzleState,
  statuses: readonly PlacementStatusCode[],
  mode: "commit" | "live",
): PuzzleEvent {
  return { kind: "inspected", inspection: inspection(state.editor, statuses), mode };
}

/** Building with one plank at (10, 64), selected, and no inspection yet. */
function buildingWithTool(): PuzzleState {
  return reducePuzzle(building(), { kind: "place", toolKind: "plank", x: 10, y: 64 });
}

function committed(state: PuzzleState, statuses: readonly PlacementStatusCode[]): PuzzleState {
  return reducePuzzle(state, inspected(state, statuses, "commit"));
}

function runningWithTool(): PuzzleState {
  return reducePuzzle(committed(buildingWithTool(), [0]), { kind: "play" });
}

function endedState(): PuzzleState {
  const lost = status({ outcome: "lost", maybeOutcomeStep: 720, maybeDecidedStep: 720 });
  return apply(runningWithTool(), [
    { kind: "stepped", stepIndex: 719, status: status() },
    { kind: "stepped", stepIndex: 720, status: lost },
  ]);
}

function liveOnly(): PuzzleState {
  const state = buildingWithTool();
  return reducePuzzle(state, inspected(state, [0], "live"));
}

function staleCommit(): PuzzleState {
  return reducePuzzle(committed(buildingWithTool(), [0]), { kind: "nudge", dx: 1, dy: 0 });
}

function bytes(view: ArrayBufferView): readonly number[] {
  return Array.from(new Uint8Array(view.buffer, view.byteOffset, view.byteLength));
}

const EDIT_EVENTS: readonly PuzzleEditEvent[] = [
  { kind: "arm", toolKind: "plank" },
  { kind: "disarm" },
  { kind: "place", toolKind: "plank", x: 10, y: 64 },
  { kind: "select", maybeId: undefined },
  { kind: "move", id: 1, x: 12, y: 64 },
  { kind: "rotate", direction: 1 },
  { kind: "nudge", dx: 1, dy: 0 },
  { kind: "remove" },
  { kind: "clear" },
  {
    kind: "inspected",
    inspection: { revision: 1, statuses: [0], outlines: [QUAD] },
    mode: "commit",
  },
];

describe("reducePuzzle edit events", () => {
  it("places, selects and disarms a tool while building", () => {
    // Arrange
    const state = reducePuzzle(building(), { kind: "arm", toolKind: "plank" });

    // Act
    const next = reducePuzzle(state, { kind: "place", toolKind: "plank", x: 10, y: 64 });

    // Assert
    expect(next.editor.tools).toEqual([
      { id: 1, placement: { kind: "plank", x: 10, y: 64, angle: 0 } },
    ]);
    expect(next.editor.maybeSelectedId).toBe(1);
    expect(next.editor.maybeArmedKind).toBeUndefined();
  });

  it("returns the same building state when an edit changes nothing", () => {
    // Arrange
    const state = building();

    // Act
    const next = reducePuzzle(state, { kind: "remove" });

    // Assert
    expect(next).toBe(state);
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["running", runningWithTool],
    ["ended", endedState],
  ] as const)("ignores every edit event while %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const results = EDIT_EVENTS.map((event) => reducePuzzle(state, event));

    // Assert
    expect(results.every((next) => next === state)).toBe(true);
  });

  it("a live inspection updates only the live snapshot (UI-D14)", () => {
    // Arrange
    const state = buildingWithTool();

    // Act
    const next = reducePuzzle(state, inspected(state, [8], "live"));

    // Assert
    expect(next.editor.maybeInspection?.statuses).toEqual([8]);
    expect(next.editor.maybeCommittedInspection).toBeUndefined();
  });

  it("a commit inspection updates the live and committed snapshots", () => {
    // Arrange
    const state = buildingWithTool();

    // Act
    const next = committed(state, [0]);

    // Assert
    expect(next.editor.maybeInspection?.statuses).toEqual([0]);
    expect(next.editor.maybeCommittedInspection?.statuses).toEqual([0]);
  });
});

describe("canPlay and play", () => {
  it("accepts play with zero tools", () => {
    // Arrange
    const state = building();

    // Act
    const next = reducePuzzle(state, { kind: "play" });

    // Assert
    expect(canPlay(state)).toBe(true);
    expect(next.kind).toBe("running");
  });

  it.each([
    ["no inspection", buildingWithTool],
    ["only a live inspection", liveOnly],
    ["an invalid committed inspection", () => committed(buildingWithTool(), [8])],
    ["a stale committed inspection", staleCommit],
  ] as const)("ignores play with one tool and %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "play" });

    // Assert
    expect(canPlay(state)).toBe(false);
    expect(next).toBe(state);
  });

  it("plays after a fresh all-valid commit and clears selection and arming", () => {
    // Arrange
    const armed = reducePuzzle(committed(buildingWithTool(), [0]), {
      kind: "arm",
      toolKind: "plank",
    });

    // Act
    const next = reducePuzzle(armed, { kind: "play" });

    // Assert
    expect(next).toMatchObject({ kind: "running", stepIndex: 0 });
    expect(next.editor.tools).toEqual(armed.editor.tools);
    expect(next.editor.maybeSelectedId).toBeUndefined();
    expect(next.editor.maybeArmedKind).toBeUndefined();
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["running", runningWithTool],
    ["ended", endedState],
  ] as const)("cannot play while %s", (_label, makeState) => {
    // Act
    const playable = canPlay(makeState());

    // Assert
    expect(playable).toBe(false);
  });
});

describe("reducePuzzle stop", () => {
  it("returns a run to building with the run's editor and the step-0 status", () => {
    // Arrange
    const state = runningWithTool();
    const preview = status();

    // Act
    const next = reducePuzzle(state, { kind: "stop", stepIndex: 0, status: preview });

    // Assert
    expect(next).toEqual({ kind: "building", status: preview, editor: state.editor, speed: 1 });
    expect(next.editor).toBe(state.editor);
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["building", buildingWithTool],
    ["ended", endedState],
  ] as const)("ignores stop while %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "stop", stepIndex: 0, status: status() });

    // Assert
    expect(next).toBe(state);
  });

  it.each([
    ["not at step 0", 3, status()],
    ["already latched", 0, status({ outcome: "won", maybeOutcomeStep: 0, peakHeldCount: 540 })],
  ] as const)("rejects a stop preview %s", (_label, stepIndex, preview) => {
    // Act
    const act = () => reducePuzzle(runningWithTool(), { kind: "stop", stepIndex, status: preview });

    // Assert
    expect(act).toThrow("Puzzle level did not start at step 0");
  });
});

describe("reducePuzzle setSpeed", () => {
  it.each([
    ["building", buildingWithTool],
    ["running", runningWithTool],
    ["ended", endedState],
  ] as const)("changes the speed while %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "setSpeed", speed: 4 });

    // Assert
    expect(next).toEqual({ ...state, speed: 4 });
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE, 4],
    ["building at the same speed", buildingWithTool, 1],
  ] as const)("ignores setSpeed while %s", (_label, makeState, speed) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "setSpeed", speed });

    // Assert
    expect(next).toBe(state);
  });
});

describe("edit-history fingerprint (D-27 level 1)", () => {
  function iteration(state: PuzzleState): PuzzleState {
    const maybeExisting = state.editor.tools[0];
    const removePrevious: readonly PuzzleEvent[] =
      maybeExisting === undefined
        ? []
        : [{ kind: "select", maybeId: maybeExisting.id }, { kind: "remove" }];
    const edited = apply(state, [
      ...removePrevious,
      { kind: "place", toolKind: "plank", x: 8, y: 64 },
      { kind: "rotate", direction: -1 },
      { kind: "remove" },
      { kind: "place", toolKind: "plank", x: 8, y: 64 },
      { kind: "rotate", direction: -1 },
    ]);
    return apply(committed(edited, [0]), [
      { kind: "play" },
      { kind: "stop", stepIndex: 0, status: status() },
    ]);
  }

  it("three place, remove, re-place and Play/Stop rounds equal one clean placement", () => {
    // Arrange
    const clean = [{ kind: "plank", x: 8, y: 64, angle: 23 }] as const;
    const start = building();

    // Act
    const after = iteration(iteration(iteration(start)));

    // Assert
    expect(after.kind).toBe("building");
    expect(canonicalPlacements(after.editor)).toEqual(clean);
    const lanes = encodePlacementLanes(canonicalPlacements(after.editor));
    const cleanLanes = encodePlacementLanes(canonicalPlacements(editorFromPlacements(clean)));
    expect(bytes(lanes.kinds)).toEqual(bytes(cleanLanes.kinds));
    expect(bytes(lanes.poses)).toEqual(bytes(cleanLanes.poses));
  });
});

describe("reducePuzzle reset editor", () => {
  it("seeds an empty editor when no placements are given", () => {
    // Act
    const next = reducePuzzle(building(), { kind: "reset", placements: [], speed: 1 });

    // Assert
    expect(next.editor).toEqual(EMPTY_EDITOR);
  });
});

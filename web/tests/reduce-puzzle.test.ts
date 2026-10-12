import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import {
  INITIAL_PUZZLE_STATE,
  SETTLE_TAIL_STEPS,
  puzzleStepBudget,
  reducePuzzle,
  remainingSimulationSteps,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";

function status(overrides: Partial<PuzzleFrameStatus> = {}): PuzzleFrameStatus {
  return {
    goalCount: 0,
    goalTarget: 540,
    goalFull: 900,
    outcome: "running",
    stepLimit: 720,
    ...overrides,
  };
}

const WON_STATUS = status({ outcome: "won", maybeOutcomeStep: 201, goalCount: 689 });
const LOST_STATUS = status({ outcome: "lost", maybeOutcomeStep: 720, goalCount: 0 });

function building(): PuzzleState {
  return reducePuzzle(INITIAL_PUZZLE_STATE, { kind: "loaded", stepIndex: 0, status: status() });
}

function running(stepIndex = 0, frameStatus = status()): PuzzleState {
  return { kind: "running", stepIndex, status: frameStatus };
}

function endedWon(): PuzzleState {
  return reducePuzzle(running(200), { kind: "stepped", stepIndex: 201, status: WON_STATUS });
}

function endedLost(): PuzzleState {
  return reducePuzzle(running(719), { kind: "stepped", stepIndex: 720, status: LOST_STATUS });
}

describe("reducePuzzle loading", () => {
  it("moves a step-0 level from loading to building", () => {
    // Arrange
    const loadedStatus = status();

    // Act
    const next = reducePuzzle(INITIAL_PUZZLE_STATE, {
      kind: "loaded",
      stepIndex: 0,
      status: loadedStatus,
    });

    // Assert
    expect(next).toEqual({ kind: "building", status: loadedStatus });
  });

  it("rejects a loaded level that did not start at step 0", () => {
    // Arrange
    const event = { kind: "loaded", stepIndex: 1, status: status() } as const;

    // Act
    const act = () => reducePuzzle(INITIAL_PUZZLE_STATE, event);

    // Assert
    expect(act).toThrow("Puzzle level did not start at step 0");
  });

  it("rejects a loaded level whose outcome already latched", () => {
    // Arrange
    const event = { kind: "loaded", stepIndex: 0, status: WON_STATUS } as const;

    // Act
    const act = () => reducePuzzle(INITIAL_PUZZLE_STATE, event);

    // Assert
    expect(act).toThrow("Puzzle level did not start at step 0");
  });

  it("rejects a loaded event outside loading", () => {
    // Arrange
    const state = building();

    // Act
    const act = () => reducePuzzle(state, { kind: "loaded", stepIndex: 0, status: status() });

    // Assert
    expect(act).toThrow("Puzzle level loaded outside loading");
  });
});

describe("reducePuzzle play", () => {
  it("starts running from building at step 0", () => {
    // Arrange
    const state = building();

    // Act
    const next = reducePuzzle(state, { kind: "play" });

    // Assert
    expect(next).toEqual({ kind: "running", stepIndex: 0, status: status() });
  });

  it.each([
    ["running", () => running(10)],
    ["ended", endedWon],
  ] as const)("ignores play while %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "play" });

    // Assert
    expect(next).toBe(state);
  });
});

describe("reducePuzzle stepping", () => {
  it("tracks the step index and status while the outcome is running", () => {
    // Arrange
    const frameStatus = status({ goalCount: 12 });

    // Act
    const next = reducePuzzle(running(4), { kind: "stepped", stepIndex: 5, status: frameStatus });

    // Assert
    expect(next).toEqual({ kind: "running", stepIndex: 5, status: frameStatus });
  });

  it("latches a win at its outcome step", () => {
    // Arrange
    const state = running(200);

    // Act
    const next = reducePuzzle(state, { kind: "stepped", stepIndex: 201, status: WON_STATUS });

    // Assert
    expect(next).toEqual({
      kind: "ended",
      stepIndex: 201,
      latch: {
        outcome: "won",
        outcomeStep: 201,
        goalCount: 689,
        goalTarget: 540,
        goalFull: 900,
        stepLimit: 720,
      },
      tailEndStep: 201 + SETTLE_TAIL_STEPS,
      held: false,
    });
  });

  it("rejects a latch first seen after its outcome step", () => {
    // Arrange
    const state = running(202);

    // Act
    const act = () => reducePuzzle(state, { kind: "stepped", stepIndex: 203, status: WON_STATUS });

    // Assert
    expect(act).toThrow("Puzzle latch was not observed at its outcome step");
  });

  it("keeps the latch during the settle tail while the count changes", () => {
    // Arrange
    const state = endedWon();
    const later = { ...WON_STATUS, goalCount: 702 };

    // Act
    const next = reducePuzzle(state, { kind: "stepped", stepIndex: 250, status: later });

    // Assert
    expect(next).toEqual({ ...state, stepIndex: 250, held: false });
  });

  it("holds once the settle tail reaches its end step", () => {
    // Arrange
    const state = endedWon();

    // Act
    const next = reducePuzzle(state, { kind: "stepped", stepIndex: 321, status: WON_STATUS });

    // Assert
    expect(next).toMatchObject({ kind: "ended", stepIndex: 321, held: true });
  });

  it("rejects a step past the end of the settle tail", () => {
    // Arrange
    const state = reducePuzzle(endedWon(), { kind: "stepped", stepIndex: 321, status: WON_STATUS });

    // Act
    const act = () => reducePuzzle(state, { kind: "stepped", stepIndex: 322, status: WON_STATUS });

    // Assert
    expect(act).toThrow("Puzzle stepped past the settle tail");
  });

  it("rejects an outcome that changes after the latch", () => {
    // Arrange
    const state = endedWon();
    const flipped = status({ outcome: "lost", maybeOutcomeStep: 720 });

    // Act
    const act = () => reducePuzzle(state, { kind: "stepped", stepIndex: 230, status: flipped });

    // Assert
    expect(act).toThrow("Puzzle outcome changed after the latch");
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["building", building],
  ] as const)("rejects a step while %s", (_label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const act = () => reducePuzzle(state, { kind: "stepped", stepIndex: 1, status: status() });

    // Assert
    expect(act).toThrow("Puzzle stepped outside a run");
  });
});

describe("reducePuzzle retry, edit and reset", () => {
  it("retry from ended waits for a fresh build that runs", () => {
    // Arrange
    const state = endedLost();

    // Act
    const next = reducePuzzle(state, { kind: "retry" });

    // Assert
    expect(next).toEqual({ kind: "loading", pendingRun: true });
  });

  it("a build after retry lands in running at step 0", () => {
    // Arrange
    const state = reducePuzzle(endedLost(), { kind: "retry" });

    // Act
    const next = reducePuzzle(state, { kind: "loaded", stepIndex: 0, status: status() });

    // Assert
    expect(next).toEqual({ kind: "running", stepIndex: 0, status: status() });
  });

  it("edit from ended waits for a fresh build that does not run", () => {
    // Arrange
    const state = endedWon();

    // Act
    const next = reducePuzzle(state, { kind: "edit" });

    // Assert
    expect(next).toEqual({ kind: "loading", pendingRun: false });
  });

  it("a build after edit lands in building", () => {
    // Arrange
    const state = reducePuzzle(endedWon(), { kind: "edit" });

    // Act
    const next = reducePuzzle(state, { kind: "loaded", stepIndex: 0, status: status() });

    // Assert
    expect(next).toEqual({ kind: "building", status: status() });
  });

  it.each([
    ["retry", "building", building],
    ["retry", "running", () => running(3)],
    ["edit", "building", building],
    ["edit", "running", () => running(3)],
  ] as const)("ignores %s while %s", (kind, _label, makeState) => {
    // Arrange
    const state = makeState();

    // Act
    const next = reducePuzzle(state, { kind });

    // Assert
    expect(next).toBe(state);
  });

  it.each([
    ["loading", () => ({ kind: "loading", pendingRun: true }) as const],
    ["building", building],
    ["running", () => running(3)],
    ["ended", endedWon],
  ] as const)("reset from %s returns to loading without a pending run", (_label, makeState) => {
    // Arrange
    const state: PuzzleState = makeState();

    // Act
    const next = reducePuzzle(state, { kind: "reset" });

    // Assert
    expect(next).toEqual({ kind: "loading", pendingRun: false });
  });
});

describe("remainingSimulationSteps", () => {
  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE, undefined],
    ["building", building, 720],
    ["running at step 100", () => running(100), 620],
    ["ended won at 201", endedWon, 519],
    [
      "ended won later in the tail",
      () => reducePuzzle(endedWon(), { kind: "stepped", stepIndex: 300, status: WON_STATUS }),
      519,
    ],
    ["ended lost", endedLost, 0],
  ] as const)("reads %s from frame data", (_label, makeState, expected) => {
    // Arrange
    const state: PuzzleState = makeState();

    // Act
    const remaining = remainingSimulationSteps(state);

    // Assert
    expect(remaining).toBe(expected);
  });
});

describe("puzzleStepBudget", () => {
  it("allows every requested step while running", () => {
    // Act
    const budget = puzzleStepBudget(running(10), 4);

    // Assert
    expect(budget).toBe(4);
  });

  it("clamps the requested steps to the settle tail end", () => {
    // Arrange
    const state = reducePuzzle(endedWon(), { kind: "stepped", stepIndex: 319, status: WON_STATUS });

    // Act
    const budget = puzzleStepBudget(state, 4);

    // Assert
    expect(budget).toBe(2);
  });

  it("allows no steps once held", () => {
    // Arrange
    const state = reducePuzzle(endedWon(), { kind: "stepped", stepIndex: 321, status: WON_STATUS });

    // Act
    const budget = puzzleStepBudget(state, 4);

    // Assert
    expect(budget).toBe(0);
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["building", building],
  ] as const)("allows no steps while %s", (_label, makeState) => {
    // Act
    const budget = puzzleStepBudget(makeState(), 4);

    // Assert
    expect(budget).toBe(0);
  });
});

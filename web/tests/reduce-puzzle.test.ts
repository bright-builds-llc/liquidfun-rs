import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import { EMPTY_EDITOR, editorFromPlacements } from "../src/puzzle/editor";
import {
  INITIAL_PUZZLE_STATE,
  SETTLE_TAIL_STEPS,
  isDeciding,
  puzzleStepBudget,
  reducePuzzle,
  remainingSimulationSteps,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";

// Synthetic first-pour-like numbers: win latch 201, Brim latch and decision 262.
const WIN_STEP = 201;
const BRIM_STEP = 262;
const BUDGET = [{ kind: "plank", count: 2 }] as const;

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

const WIN_STATUS = status({
  outcome: "won",
  maybeOutcomeStep: WIN_STEP,
  goalCount: 689,
  peakHeldCount: 541,
});
const BRIM_STATUS = status({
  outcome: "won",
  maybeOutcomeStep: WIN_STEP,
  goalCount: 920,
  peakHeldCount: 811,
  maybeBrimStep: BRIM_STEP,
  maybeDecidedStep: BRIM_STEP,
});
const LIMIT_WIN_STATUS = status({
  outcome: "won",
  maybeOutcomeStep: WIN_STEP,
  goalCount: 700,
  peakHeldCount: 700,
  maybeDecidedStep: 720,
});
const LOST_STATUS = status({
  outcome: "lost",
  maybeOutcomeStep: 720,
  goalCount: 0,
  maybeDecidedStep: 720,
});

function building(): PuzzleState {
  return reducePuzzle(INITIAL_PUZZLE_STATE, {
    kind: "loaded",
    stepIndex: 0,
    status: status(),
    budget: BUDGET,
  });
}

function running(stepIndex = 0, frameStatus = status()): PuzzleState {
  return { kind: "running", stepIndex, status: frameStatus, editor: EMPTY_EDITOR, speed: 1 };
}

function deciding(stepIndex = BRIM_STEP - 1): PuzzleState {
  return running(stepIndex, WIN_STATUS);
}

function endedWon(): PuzzleState {
  return reducePuzzle(deciding(), { kind: "stepped", stepIndex: BRIM_STEP, status: BRIM_STATUS });
}

function endedLost(): PuzzleState {
  return reducePuzzle(running(719), { kind: "stepped", stepIndex: 720, status: LOST_STATUS });
}

function stepped(state: PuzzleState, stepIndex: number, frameStatus: PuzzleFrameStatus) {
  return reducePuzzle(state, { kind: "stepped", stepIndex, status: frameStatus });
}

describe("reducePuzzle loading", () => {
  it("resets into loading with the given placements and speed", () => {
    // Arrange
    const placements = [{ kind: "plank", x: 8, y: 64, angle: 23 }] as const;

    // Act
    const next = reducePuzzle(building(), { kind: "reset", placements, speed: 2 });

    // Assert
    expect(next).toEqual({
      kind: "loading",
      pendingRun: false,
      editor: editorFromPlacements(placements),
      speed: 2,
    });
  });

  it("moves a step-0 level from loading to building with the level budget", () => {
    // Arrange
    const loadedStatus = status();

    // Act
    const next = reducePuzzle(INITIAL_PUZZLE_STATE, {
      kind: "loaded",
      stepIndex: 0,
      status: loadedStatus,
      budget: BUDGET,
    });

    // Assert
    expect(next).toEqual({
      kind: "building",
      status: loadedStatus,
      editor: { ...EMPTY_EDITOR, budget: BUDGET },
      speed: 1,
    });
  });

  it.each([
    ["did not start at step 0", 1, status()],
    ["already latched", 0, WIN_STATUS],
  ] as const)("rejects a loaded level that %s", (_label, stepIndex, loadedStatus) => {
    // Arrange
    const event = { kind: "loaded", stepIndex, status: loadedStatus, budget: BUDGET } as const;

    // Act
    const act = () => reducePuzzle(INITIAL_PUZZLE_STATE, event);

    // Assert
    expect(act).toThrow("Puzzle level did not start at step 0");
  });

  it("rejects a loaded event outside loading", () => {
    // Arrange
    const event = { kind: "loaded", stepIndex: 0, status: status(), budget: BUDGET } as const;

    // Act
    const act = () => reducePuzzle(building(), event);

    // Assert
    expect(act).toThrow("Puzzle level loaded outside loading");
  });
});

describe("reducePuzzle play", () => {
  it("starts running at step 0 from building with zero tools", () => {
    // Arrange
    const state = building();

    // Act
    const next = reducePuzzle(state, { kind: "play" });

    // Assert
    expect(next).toEqual({
      kind: "running",
      stepIndex: 0,
      status: status(),
      editor: state.editor,
      speed: 1,
    });
  });

  it.each([
    ["running", () => running(10)],
    ["deciding", () => deciding()],
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

describe("reducePuzzle running and the D-10 decision", () => {
  it("tracks the step index and status while the outcome is running", () => {
    // Arrange
    const frameStatus = status({ goalCount: 12 });

    // Act
    const next = stepped(running(4), 5, frameStatus);

    // Assert
    expect(next).toEqual({ ...running(5, frameStatus) });
  });

  it("keeps a won run running and deciding until its decided step", () => {
    // Arrange
    const state = running(WIN_STEP - 1);

    // Act
    const next = stepped(state, WIN_STEP, WIN_STATUS);

    // Assert
    expect(next).toEqual(running(WIN_STEP, WIN_STATUS));
    expect(isDeciding(next)).toBe(true);
  });

  it("keeps deciding while the live count changes", () => {
    // Arrange
    const later = { ...WIN_STATUS, goalCount: 900, peakHeldCount: 760 };

    // Act
    const next = stepped(deciding(230), 231, later);

    // Assert
    expect(next).toEqual(running(231, later));
  });

  it("ends a won run at the Brim latch with the full latch and tail", () => {
    // Arrange
    const state = deciding();

    // Act
    const next = stepped(state, BRIM_STEP, BRIM_STATUS);

    // Assert
    expect(next).toEqual({
      kind: "ended",
      stepIndex: BRIM_STEP,
      latch: {
        outcome: "won",
        outcomeStep: WIN_STEP,
        decidedStep: BRIM_STEP,
        goalCount: 920,
        peakHeldCount: 811,
        maybeBrimStep: BRIM_STEP,
        goalTarget: 540,
        goalFull: 900,
        brimCount: 810,
        stepLimit: 720,
      },
      tailEndStep: BRIM_STEP + SETTLE_TAIL_STEPS,
      held: false,
      editor: EMPTY_EDITOR,
      speed: 1,
    });
  });

  it("ends a won run without Brim at the step limit", () => {
    // Arrange
    const state = deciding(719);

    // Act
    const next = stepped(state, 720, LIMIT_WIN_STATUS);

    // Assert
    expect(next).toMatchObject({
      kind: "ended",
      latch: { outcome: "won", decidedStep: 720, maybeBrimStep: undefined, peakHeldCount: 700 },
      tailEndStep: 720 + SETTLE_TAIL_STEPS,
    });
  });

  it("ends a lost run at its loss latch as in Phase 38", () => {
    // Act
    const next = endedLost();

    // Assert
    expect(next).toMatchObject({
      kind: "ended",
      stepIndex: 720,
      latch: { outcome: "lost", outcomeStep: 720, decidedStep: 720, goalCount: 0 },
      tailEndStep: 720 + SETTLE_TAIL_STEPS,
    });
  });

  it("decides a win and Brim latched on the same step at once", () => {
    // Arrange
    const sameStep = { ...BRIM_STATUS, maybeOutcomeStep: BRIM_STEP };

    // Act
    const next = stepped(running(BRIM_STEP - 1), BRIM_STEP, sameStep);

    // Assert
    expect(next).toMatchObject({ kind: "ended", latch: { outcomeStep: BRIM_STEP } });
  });

  it("rejects a win latch first seen after its outcome step", () => {
    // Act
    const act = () => stepped(running(202), 203, WIN_STATUS);

    // Assert
    expect(act).toThrow("Puzzle latch was not observed at its outcome step");
  });

  it.each([
    ["a won run", deciding(BRIM_STEP), BRIM_STEP + 1, BRIM_STATUS],
    [
      "a lost run without a decided step",
      running(719),
      720,
      status({ outcome: "lost", maybeOutcomeStep: 720 }),
    ],
  ] as const)("rejects %s decided off its decided step", (_label, state, stepIndex, frameStatus) => {
    // Act
    const act = () => stepped(state, stepIndex, frameStatus);

    // Assert
    expect(act).toThrow("Puzzle decision was not observed at its decided step");
  });

  it.each([
    ["flips to lost", LOST_STATUS, "Puzzle outcome changed after the latch"],
    ["drops back to running", status({ goalCount: 300 }), "Puzzle outcome changed after the latch"],
    [
      "moves its win step",
      { ...WIN_STATUS, maybeOutcomeStep: 230 },
      "Puzzle latch was not observed at its outcome step",
    ],
  ] as const)("rejects a deciding run whose outcome %s", (_label, frameStatus, message) => {
    // Arrange
    const state = deciding(719);

    // Act
    const act = () => stepped(state, 720, frameStatus);

    // Assert
    expect(act).toThrow(message);
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["building", building],
  ] as const)("rejects a step while %s", (_label, makeState) => {
    // Act
    const act = () => stepped(makeState(), 1, status());

    // Assert
    expect(act).toThrow("Puzzle stepped outside a run");
  });
});

describe("reducePuzzle settle tail", () => {
  it("keeps the latch during the settle tail while the count changes", () => {
    // Arrange
    const state = endedWon();
    const later = { ...BRIM_STATUS, goalCount: 702 };

    // Act
    const next = stepped(state, 300, later);

    // Assert
    expect(next).toEqual({ ...state, stepIndex: 300, held: false });
  });

  it("holds once the settle tail reaches its end step", () => {
    // Act
    const next = stepped(endedWon(), BRIM_STEP + SETTLE_TAIL_STEPS, BRIM_STATUS);

    // Assert
    expect(next).toMatchObject({ kind: "ended", stepIndex: 382, held: true });
  });

  it("rejects a step past the end of the settle tail", () => {
    // Arrange
    const state = stepped(endedWon(), 382, BRIM_STATUS);

    // Act
    const act = () => stepped(state, 383, BRIM_STATUS);

    // Assert
    expect(act).toThrow("Puzzle stepped past the settle tail");
  });

  it.each([
    ["the outcome", LOST_STATUS],
    ["the outcome step", { ...BRIM_STATUS, maybeOutcomeStep: 202 }],
    ["the decided step", { ...BRIM_STATUS, maybeDecidedStep: 263 }],
    ["the Brim step", { ...BRIM_STATUS, maybeBrimStep: 263 }],
  ] as const)("rejects a tail frame that changes %s", (_label, frameStatus) => {
    // Act
    const act = () => stepped(endedWon(), 300, frameStatus);

    // Assert
    expect(act).toThrow("Puzzle outcome changed after the latch");
  });
});

describe("reducePuzzle retry, edit and reset", () => {
  it.each([
    ["retry", true],
    ["edit", false],
  ] as const)("%s from ended carries the editor and speed into loading", (kind, pendingRun) => {
    // Arrange
    const editor = editorFromPlacements([{ kind: "plank", x: 8, y: 64, angle: 23 }], BUDGET);
    const state: PuzzleState = { ...endedLost(), editor, speed: 4 };

    // Act
    const next = reducePuzzle(state, { kind });

    // Assert
    expect(next).toEqual({ kind: "loading", pendingRun, editor, speed: 4 });
  });

  it("a build after retry lands in running at step 0", () => {
    // Arrange
    const state = reducePuzzle(endedLost(), { kind: "retry" });

    // Act
    const next = reducePuzzle(state, {
      kind: "loaded",
      stepIndex: 0,
      status: status(),
      budget: BUDGET,
    });

    // Assert
    expect(next).toMatchObject({ kind: "running", stepIndex: 0, status: status() });
  });

  it("a build after edit lands in building", () => {
    // Arrange
    const state = reducePuzzle(endedWon(), { kind: "edit" });

    // Act
    const next = reducePuzzle(state, {
      kind: "loaded",
      stepIndex: 0,
      status: status(),
      budget: BUDGET,
    });

    // Assert
    expect(next).toMatchObject({ kind: "building", status: status() });
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
});

describe("remainingSimulationSteps", () => {
  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE, undefined],
    ["building", building, 720],
    ["running at step 100", () => running(100), 620],
    ["deciding at step 230", () => deciding(230), 490],
    ["ended won at the decided step 262", endedWon, 458],
    ["ended won later in the tail", () => stepped(endedWon(), 300, BRIM_STATUS), 458],
    ["ended lost", endedLost, 0],
  ] as const)("reads %s from frame data", (_label, makeState, expected) => {
    // Act
    const remaining = remainingSimulationSteps(makeState());

    // Assert
    expect(remaining).toBe(expected);
  });
});

describe("puzzleStepBudget", () => {
  it.each([
    ["running", () => running(10)],
    ["deciding", () => deciding()],
  ] as const)("allows every requested step while %s", (_label, makeState) => {
    // Act
    const budget = puzzleStepBudget(makeState(), 4);

    // Assert
    expect(budget).toBe(4);
  });

  it("clamps the requested steps to the settle tail end", () => {
    // Arrange
    const state = stepped(endedWon(), 380, BRIM_STATUS);

    // Act
    const budget = puzzleStepBudget(state, 4);

    // Assert
    expect(budget).toBe(2);
  });

  it("allows no steps once held", () => {
    // Arrange
    const state = stepped(endedWon(), 382, BRIM_STATUS);

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

import type { PuzzleFrameStatus } from "../physics/frame";

/** Steps simulated after the latch so water visibly settles (2 s at 60 Hz) (D-08). */
export const SETTLE_TAIL_STEPS = 120;

/** Rust outcome and goal readout captured at the outcome step (D-08). */
export type PuzzleLatch = {
  readonly outcome: "won" | "lost";
  readonly outcomeStep: number;
  readonly goalCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly stepLimit: number;
};

/**
 * Puzzle player phase (D-06). `pendingRun` makes the next `loaded` land in
 * running (Retry) instead of building.
 */
export type PuzzleState =
  | { readonly kind: "loading"; readonly pendingRun: boolean }
  | { readonly kind: "building"; readonly status: PuzzleFrameStatus }
  | {
      readonly kind: "running";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    }
  | {
      readonly kind: "ended";
      readonly stepIndex: number;
      readonly latch: PuzzleLatch;
      readonly tailEndStep: number;
      readonly held: boolean;
    };

/** Phase name of a puzzle state. */
export type PuzzlePhase = PuzzleState["kind"];

/** Inputs to the puzzle phase machine; `stepped` carries one fixed step's frame data. */
export type PuzzleEvent =
  | { readonly kind: "reset" }
  | {
      readonly kind: "loaded";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    }
  | { readonly kind: "play" }
  | {
      readonly kind: "stepped";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    }
  | { readonly kind: "retry" }
  | { readonly kind: "edit" };

/** State before any level has been built. */
export const INITIAL_PUZZLE_STATE: PuzzleState = { kind: "loading", pendingRun: false };

/**
 * Pure puzzle phase transition (D-06, D-07, D-08). Ignored events return the
 * same state object; contract violations throw so the shell fails closed.
 */
export function reducePuzzle(state: PuzzleState, event: PuzzleEvent): PuzzleState {
  switch (event.kind) {
    case "reset":
      return INITIAL_PUZZLE_STATE;
    case "loaded":
      return reduceLoaded(state, event.stepIndex, event.status);
    case "play":
      return state.kind === "building"
        ? { kind: "running", stepIndex: 0, status: state.status }
        : state;
    case "stepped":
      return reduceStepped(state, event.stepIndex, event.status);
    case "retry":
      return state.kind === "ended" ? { kind: "loading", pendingRun: true } : state;
    case "edit":
      return state.kind === "ended" ? { kind: "loading", pendingRun: false } : state;
    default: {
      const unreachable: never = event;
      return unreachable;
    }
  }
}

function reduceLoaded(
  state: PuzzleState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  if (state.kind !== "loading") {
    throw new Error("Puzzle level loaded outside loading");
  }
  if (stepIndex !== 0 || status.outcome !== "running") {
    throw new Error("Puzzle level did not start at step 0");
  }

  return state.pendingRun
    ? { kind: "running", stepIndex: 0, status }
    : { kind: "building", status };
}

function reduceStepped(
  state: PuzzleState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  if (state.kind === "running") {
    return stepRunning(stepIndex, status);
  }
  if (state.kind === "ended") {
    return stepEnded(state, stepIndex, status);
  }

  throw new Error("Puzzle stepped outside a run");
}

function stepRunning(stepIndex: number, status: PuzzleFrameStatus): PuzzleState {
  if (status.outcome === "running") {
    return { kind: "running", stepIndex, status };
  }
  if (status.maybeOutcomeStep !== stepIndex) {
    throw new Error("Puzzle latch was not observed at its outcome step");
  }

  const tailEndStep = stepIndex + SETTLE_TAIL_STEPS;
  return {
    kind: "ended",
    stepIndex,
    latch: {
      outcome: status.outcome,
      outcomeStep: stepIndex,
      goalCount: status.goalCount,
      goalTarget: status.goalTarget,
      goalFull: status.goalFull,
      stepLimit: status.stepLimit,
    },
    tailEndStep,
    held: false,
  };
}

function stepEnded(
  state: Extract<PuzzleState, { kind: "ended" }>,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  const { latch } = state;
  if (status.outcome !== latch.outcome || status.maybeOutcomeStep !== latch.outcomeStep) {
    throw new Error("Puzzle outcome changed after the latch");
  }
  if (stepIndex > state.tailEndStep) {
    throw new Error("Puzzle stepped past the settle tail");
  }

  return { ...state, stepIndex, held: stepIndex >= state.tailEndStep };
}

/** Simulation steps left on the timer, from frame data only (D-09). */
export function remainingSimulationSteps(state: PuzzleState): number | undefined {
  switch (state.kind) {
    case "loading":
      return undefined;
    case "building":
      return state.status.stepLimit;
    case "running":
      return Math.max(0, state.status.stepLimit - state.stepIndex);
    case "ended":
      return Math.max(0, state.latch.stepLimit - state.latch.outcomeStep);
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

/** Fixed steps the loop may run now: all requested while running, clamped to the tail end once ended, zero otherwise (D-08). */
export function puzzleStepBudget(state: PuzzleState, requestedSteps: number): number {
  const requested = Math.max(0, requestedSteps);
  if (state.kind === "running") {
    return requested;
  }
  if (state.kind === "ended" && !state.held) {
    return Math.min(requested, state.tailEndStep - state.stepIndex);
  }

  return 0;
}

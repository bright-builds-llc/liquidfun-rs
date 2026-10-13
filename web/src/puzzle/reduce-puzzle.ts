import type { PuzzleFrameStatus } from "../physics/frame";
import { applyEditEvent, type PuzzleEditEvent } from "./edit-event";
import {
  EMPTY_EDITOR,
  allPlacementsValid,
  clearInteraction,
  editorFromPlacements,
  stepSelectedValveTime,
  withBudget,
  type BuildingEditor,
  type ToolAllowance,
} from "./editor";
import type { PuzzlePlacement } from "./placements";
import { INITIAL_PUZZLE_SPEED, type PuzzleSpeed } from "./speed";
import { maxOpenTick } from "./valve-format";

/** Steps simulated after the decision so water visibly settles (2 s at 60 Hz) (D-08, D-10). */
export const SETTLE_TAIL_STEPS = 120;

/**
 * Rust outcome, star latches and goal readout captured at the decision step
 * (D-10). `outcomeStep` is the win or loss latch; `decidedStep` is the Brim
 * latch or the step limit for a win, and the step limit for a loss.
 */
export type PuzzleLatch = {
  readonly outcome: "won" | "lost";
  readonly outcomeStep: number;
  readonly decidedStep: number;
  readonly goalCount: number;
  readonly peakHeldCount: number;
  readonly maybeBrimStep: number | undefined;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly brimCount: number;
  readonly stepLimit: number;
};

/** Fields every phase carries: the building editor (placements of the run once running) and the speed (D-24, D-25). */
type Shared = { readonly editor: BuildingEditor; readonly speed: PuzzleSpeed };

/**
 * Puzzle player phase (D-06, D-25). `pendingRun` makes the next `loaded` land
 * in running (Retry) instead of building.
 */
export type PuzzleState =
  | ({ readonly kind: "loading"; readonly pendingRun: boolean } & Shared)
  | ({ readonly kind: "building"; readonly status: PuzzleFrameStatus } & Shared)
  | ({
      readonly kind: "running";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    } & Shared)
  | ({
      readonly kind: "ended";
      readonly stepIndex: number;
      readonly latch: PuzzleLatch;
      readonly tailEndStep: number;
      readonly held: boolean;
    } & Shared);

/** Phase name of a puzzle state. */
export type PuzzlePhase = PuzzleState["kind"];

export type { PuzzleEditEvent } from "./edit-event";

/**
 * Moves the selected valve's open time by one half-second step while
 * building, clamped to the level's step limit (Phase 40 D-05).
 */
export type SetValveTimeEvent = { readonly kind: "setValveTime"; readonly direction: 1 | -1 };

/** Every committed building-phase edit the shell applies and announces. */
export type BuildingEditEvent = PuzzleEditEvent | SetValveTimeEvent;

/** Inputs to the puzzle phase machine; `stepped` carries one fixed step's frame data. */
export type PuzzleEvent =
  | {
      readonly kind: "reset";
      readonly placements: readonly PuzzlePlacement[];
      readonly speed: PuzzleSpeed;
    }
  | {
      readonly kind: "loaded";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
      readonly budget: readonly ToolAllowance[];
    }
  | { readonly kind: "play" }
  | {
      readonly kind: "stepped";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    }
  | {
      readonly kind: "stop";
      readonly stepIndex: number;
      readonly status: PuzzleFrameStatus;
    }
  | { readonly kind: "retry" }
  | { readonly kind: "edit" }
  | { readonly kind: "setSpeed"; readonly speed: PuzzleSpeed }
  | SetValveTimeEvent
  | PuzzleEditEvent;

type RunningState = Extract<PuzzleState, { kind: "running" }>;
type EndedState = Extract<PuzzleState, { kind: "ended" }>;

/** State before any level has been built. */
export const INITIAL_PUZZLE_STATE: PuzzleState = {
  kind: "loading",
  pendingRun: false,
  editor: EMPTY_EDITOR,
  speed: INITIAL_PUZZLE_SPEED,
};

/** Play gate (D-20): building with zero tools or a fresh, all-valid committed inspection. */
export function canPlay(state: PuzzleState): boolean {
  if (state.kind !== "building") {
    return false;
  }

  return state.editor.tools.length === 0 || allPlacementsValid(state.editor);
}

/** Won but not yet decided: the "Filled!" in-progress state (D-10). */
export function isDeciding(state: PuzzleState): boolean {
  return state.kind === "running" && state.status.outcome === "won";
}

/**
 * Pure puzzle phase transition (D-06, D-08, D-10, D-20, D-21, D-25). Ignored
 * events return the same state object; contract violations throw so the
 * shell fails closed.
 */
export function reducePuzzle(state: PuzzleState, event: PuzzleEvent): PuzzleState {
  switch (event.kind) {
    case "reset":
      return {
        kind: "loading",
        pendingRun: false,
        editor: editorFromPlacements(event.placements),
        speed: event.speed,
      };
    case "loaded":
      return reduceLoaded(state, event.stepIndex, event.status, event.budget);
    case "play":
      return reducePlay(state);
    case "stepped":
      return reduceStepped(state, event.stepIndex, event.status);
    case "stop":
      return reduceStop(state, event.stepIndex, event.status);
    case "retry":
    case "edit":
      return reduceRebuild(state, event.kind === "retry");
    case "setSpeed":
      return state.kind === "loading" || state.speed === event.speed
        ? state
        : { ...state, speed: event.speed };
    case "setValveTime":
      return reduceValveTime(state, event.direction);
    case "arm":
    case "disarm":
    case "place":
    case "select":
    case "move":
    case "rotate":
    case "nudge":
    case "remove":
    case "clear":
    case "inspected":
      return reduceEdit(state, event);
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
  budget: readonly ToolAllowance[],
): PuzzleState {
  if (state.kind !== "loading") {
    throw new Error("Puzzle level loaded outside loading");
  }
  requireStepZero(stepIndex, status);

  const editor = withBudget(state.editor, budget);
  const { speed } = state;
  return state.pendingRun
    ? { kind: "running", stepIndex: 0, status, editor: clearInteraction(editor), speed }
    : { kind: "building", status, editor, speed };
}

function requireStepZero(stepIndex: number, status: PuzzleFrameStatus): void {
  if (stepIndex !== 0 || status.outcome !== "running") {
    throw new Error("Puzzle level did not start at step 0");
  }
}

function reducePlay(state: PuzzleState): PuzzleState {
  if (state.kind !== "building" || !canPlay(state)) {
    return state;
  }

  return {
    kind: "running",
    stepIndex: 0,
    status: state.status,
    editor: clearInteraction(state.editor),
    speed: state.speed,
  };
}

/** Retry or Edit: ended → loading, keeping the editor and speed (D-21). */
function reduceRebuild(state: PuzzleState, pendingRun: boolean): PuzzleState {
  if (state.kind !== "ended") {
    return state;
  }

  return { kind: "loading", pendingRun, editor: state.editor, speed: state.speed };
}

/** running → building with the run's editor on a fresh step-0 preview (D-21). */
function reduceStop(
  state: PuzzleState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  if (state.kind !== "running") {
    return state;
  }
  requireStepZero(stepIndex, status);

  return { kind: "building", status, editor: state.editor, speed: state.speed };
}

function reduceEdit(state: PuzzleState, event: PuzzleEditEvent): PuzzleState {
  if (state.kind !== "building") {
    return state;
  }

  const editor = applyEditEvent(state.editor, event);
  return editor === state.editor ? state : { ...state, editor };
}

function reduceValveTime(state: PuzzleState, direction: 1 | -1): PuzzleState {
  if (state.kind !== "building") {
    return state;
  }

  const maxTick = maxOpenTick(state.status.stepLimit);
  const editor = stepSelectedValveTime(state.editor, direction, maxTick);
  return editor === state.editor ? state : { ...state, editor };
}

function reduceStepped(
  state: PuzzleState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  if (state.kind === "running") {
    return stepRunning(state, stepIndex, status);
  }
  if (state.kind === "ended") {
    return stepEnded(state, stepIndex, status);
  }

  throw new Error("Puzzle stepped outside a run");
}

/** Outcome latch step of a non-running frame; checks it against the previous frame (T-39-21). */
function latchedOutcomeStep(
  state: RunningState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): number {
  const previous = state.status;
  const expectedStep =
    previous.outcome === "running" ? stepIndex : previous.maybeOutcomeStep;
  if (previous.outcome !== "running" && previous.outcome !== status.outcome) {
    throw new Error("Puzzle outcome changed after the latch");
  }
  if (status.maybeOutcomeStep === undefined || status.maybeOutcomeStep !== expectedStep) {
    throw new Error("Puzzle latch was not observed at its outcome step");
  }

  return status.maybeOutcomeStep;
}

function stepRunning(
  state: RunningState,
  stepIndex: number,
  status: PuzzleFrameStatus,
): PuzzleState {
  if (status.outcome === "running") {
    if (state.status.outcome !== "running") {
      throw new Error("Puzzle outcome changed after the latch");
    }
    return { ...state, stepIndex, status };
  }

  const outcomeStep = latchedOutcomeStep(state, stepIndex, status);
  const maybeDecidedStep = status.maybeDecidedStep;
  // A won run keeps running ("Filled!" deciding) until Brim or the step limit (D-10).
  if (maybeDecidedStep === undefined && status.outcome === "won") {
    return { ...state, stepIndex, status };
  }
  if (maybeDecidedStep !== stepIndex) {
    throw new Error("Puzzle decision was not observed at its decided step");
  }

  return {
    kind: "ended",
    stepIndex,
    latch: {
      outcome: status.outcome,
      outcomeStep,
      decidedStep: stepIndex,
      goalCount: status.goalCount,
      peakHeldCount: status.peakHeldCount,
      maybeBrimStep: status.maybeBrimStep,
      goalTarget: status.goalTarget,
      goalFull: status.goalFull,
      brimCount: status.brimCount,
      stepLimit: status.stepLimit,
    },
    tailEndStep: stepIndex + SETTLE_TAIL_STEPS,
    held: false,
    editor: state.editor,
    speed: state.speed,
  };
}

function stepEnded(state: EndedState, stepIndex: number, status: PuzzleFrameStatus): PuzzleState {
  const { latch } = state;
  const unchanged =
    status.outcome === latch.outcome &&
    status.maybeOutcomeStep === latch.outcomeStep &&
    status.maybeDecidedStep === latch.decidedStep &&
    status.maybeBrimStep === latch.maybeBrimStep;
  if (!unchanged) {
    throw new Error("Puzzle outcome changed after the latch");
  }
  if (stepIndex > state.tailEndStep) {
    throw new Error("Puzzle stepped past the settle tail");
  }

  return { ...state, stepIndex, held: stepIndex >= state.tailEndStep };
}

/** Simulation steps left on the timer, from frame data only; frozen at the decided step once ended (D-09, UI-D16). */
export function remainingSimulationSteps(state: PuzzleState): number | undefined {
  switch (state.kind) {
    case "loading":
      return undefined;
    case "building":
      return state.status.stepLimit;
    case "running":
      return Math.max(0, state.status.stepLimit - state.stepIndex);
    case "ended":
      return Math.max(0, state.latch.stepLimit - state.latch.decidedStep);
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

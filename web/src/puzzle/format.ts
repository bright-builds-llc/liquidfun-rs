import type { PuzzleId } from "../catalog/puzzles";
import { remainingSimulationSteps, type PuzzleState } from "./reduce-puzzle";

/** Fixed steps per tenth of a simulated second (60 Hz). */
const STEPS_PER_TENTH = 6;
const TENTHS_PER_MINUTE = 600;
const LOADING_TIMER_TEXT = "-:--.-";

/** One fill-meter and glass-cup marker at a fraction of the full capacity (D-13). */
export type GoalTick = { readonly fraction: number; readonly label: string };

/** Remaining simulation time as `m:ss.t`, floored to tenths with integer math only (D-09). */
export function formatSimulationTime(remainingSteps: number): string {
  const tenths = Math.floor(Math.max(0, remainingSteps) / STEPS_PER_TENTH);
  const minutes = Math.floor(tenths / TENTHS_PER_MINUTE);
  const seconds = Math.floor((tenths % TENTHS_PER_MINUTE) / 10);
  return `${minutes}:${String(seconds).padStart(2, "0")}.${tenths % 10}`;
}

/** Whole percent of capacity, rounded down and capped at 100 (D-10). */
export function floorPercent(count: number, full: number): number {
  if (!(full > 0)) {
    throw new Error("Puzzle capacity must be positive");
  }

  return Math.min(100, Math.floor((count * 100) / full));
}

/** Data-driven tick list: ★ Win at the target, Full at capacity (D-13; Phase 39 appends ★ Brim). */
export function goalTicks(targetCount: number, fullCount: number): readonly GoalTick[] {
  return [
    { fraction: targetCount / fullCount, label: "★ Win" },
    { fraction: 1, label: "Full" },
  ];
}

/** Goal sentence shown in the result slot before the outcome latches. */
export function goalText(targetCount: number, fullCount: number): string {
  return `Fill the cup to ${floorPercent(targetCount, fullCount)}% before time runs out.`;
}

/** Timer chip text: full limit in building, live while running, frozen at the outcome step (D-09). */
export function timerText(state: PuzzleState): string {
  const maybeRemaining = remainingSimulationSteps(state);
  return maybeRemaining === undefined
    ? LOADING_TIMER_TEXT
    : formatSimulationTime(maybeRemaining);
}

/** Fill meter chip contents and meter semantics (D-11). */
export type FillMeterModel = {
  readonly percentText: string;
  readonly targetText: string;
  readonly fillPercent: number;
  readonly ticks: readonly GoalTick[];
  readonly maybeValueNow: number | undefined;
  readonly maybeValueText: string | undefined;
};

const LOADING_FILL_METER: FillMeterModel = {
  percentText: "--%",
  targetText: " / --%",
  fillPercent: 0,
  ticks: [],
  maybeValueNow: undefined,
  maybeValueText: undefined,
};

type GoalCounts = {
  readonly goalCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
};

/** Counts the HUD shows: the live frame status, or the latch once ended (UI-D5). */
function maybeShownCounts(state: PuzzleState): GoalCounts | undefined {
  switch (state.kind) {
    case "loading":
      return undefined;
    case "building":
    case "running":
      return state.status;
    case "ended":
      return state.latch;
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

/** Fill meter model: placeholders while loading, live counts while running, frozen latch once ended. */
export function fillMeterModel(state: PuzzleState): FillMeterModel {
  const maybeCounts = maybeShownCounts(state);
  if (maybeCounts === undefined) {
    return LOADING_FILL_METER;
  }

  const fill = floorPercent(maybeCounts.goalCount, maybeCounts.goalFull);
  const target = floorPercent(maybeCounts.goalTarget, maybeCounts.goalFull);
  return {
    percentText: `${fill}%`,
    targetText: ` / ${target}%`,
    fillPercent: fill,
    ticks: goalTicks(maybeCounts.goalTarget, maybeCounts.goalFull),
    maybeValueNow: fill,
    maybeValueText: `${fill}% full, target ${target}%`,
  };
}

/** Result slot contents (D-14). */
export type ResultCardModel =
  | { readonly kind: "loading"; readonly message: string }
  | { readonly kind: "goal"; readonly message: string }
  | {
      readonly kind: "outcome";
      readonly outcome: "won" | "lost";
      readonly outcomeWord: "Filled!" | "Out of time:";
      readonly scoreText: string;
      readonly text: string;
    };

/** Result slot model; outcome copy uses the latched, rounded-down numbers (D-10, D-14). */
export function resultCardModel(state: PuzzleState): ResultCardModel {
  switch (state.kind) {
    case "loading":
      return { kind: "loading", message: "Loading level…" };
    case "building":
    case "running":
      return {
        kind: "goal",
        message: goalText(state.status.goalTarget, state.status.goalFull),
      };
    case "ended": {
      const { latch } = state;
      const outcomeWord = latch.outcome === "won" ? "Filled!" : "Out of time:";
      const scoreText = `${floorPercent(latch.goalCount, latch.goalFull)}% of ${floorPercent(latch.goalTarget, latch.goalFull)}%`;
      return {
        kind: "outcome",
        outcome: latch.outcome,
        outcomeWord,
        scoreText,
        text: `${outcomeWord} ${scoreText}`,
      };
    }
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

/** Result actions: Next link state and which action is the accented primary (D-15). */
export type ResultActionsModel = {
  readonly next: {
    readonly enabled: boolean;
    readonly label: "Next";
    readonly ariaLabel: string;
    readonly maybeTitle: string | undefined;
    readonly maybeHref: string | undefined;
  };
  readonly primary: "next" | "retry";
};

/** Next is a link to the following level, or disabled with "No next level yet" (D-15). */
export function resultActionsModel(maybeNextId: PuzzleId | undefined): ResultActionsModel {
  if (maybeNextId === undefined) {
    return {
      next: {
        enabled: false,
        label: "Next",
        ariaLabel: "Next: No next level yet",
        maybeTitle: "No next level yet",
        maybeHref: undefined,
      },
      primary: "retry",
    };
  }

  return {
    next: {
      enabled: true,
      label: "Next",
      ariaLabel: "Next level",
      maybeTitle: undefined,
      maybeHref: `#/puzzle/${maybeNextId}`,
    },
    primary: "next",
  };
}

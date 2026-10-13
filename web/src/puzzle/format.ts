import { puzzleHash } from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";
import { buildingMessage } from "./editor-format";
import { remainingSimulationSteps, type PuzzleState } from "./reduce-puzzle";

/** Fixed steps per tenth of a simulated second (60 Hz). */
const STEPS_PER_TENTH = 6;
const TENTHS_PER_MINUTE = 600;
const LOADING_TIMER_TEXT = "-:--.-";

/** One fill-meter and glass-cup marker at a fraction of the full capacity (D-13). */
export type GoalTick = { readonly fraction: number; readonly label: string };

/** Fill-meter tick: `star` for ★ ticks, `reached` once its latch happened this run (D-10). */
export type MeterTick = GoalTick & { readonly star: boolean; readonly reached: boolean };

const WIN_TICK_LABEL = "★ Win";
const BRIM_TICK_LABEL = "★ Brim";

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

/** Data-driven tick list: ★ Win at the target, ★ Brim at the Brim count, Full at capacity (D-13, D-16). */
export function goalTicks(
  targetCount: number,
  fullCount: number,
  brimCount: number,
): readonly GoalTick[] {
  return [
    { fraction: targetCount / fullCount, label: WIN_TICK_LABEL },
    { fraction: brimCount / fullCount, label: BRIM_TICK_LABEL },
    { fraction: 1, label: "Full" },
  ];
}

/** Goal sentence shown in the result slot before the outcome latches. */
export function goalText(targetCount: number, fullCount: number): string {
  return `Fill the cup to ${floorPercent(targetCount, fullCount)}% before time runs out.`;
}

/** "Going for Brim at 90%": the Brim count as a rounded-down percent of capacity. */
function brimPendingText(brimCount: number, fullCount: number): string {
  return `Going for Brim at ${floorPercent(brimCount, fullCount)}%`;
}

/** Timer chip text: full limit in building, live while running, frozen at the decided step (D-09, UI-D16). */
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
  readonly ticks: readonly MeterTick[];
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

/** What the meter shows: the count, the level scalars and which ★ latches happened. */
type MeterInputs = {
  readonly shownCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly brimCount: number;
  readonly won: boolean;
  readonly brimLatched: boolean;
};

/**
 * Live frame counts while building or running; once ended, the latch with a
 * won run frozen at its peak-held count (UI-D5, UI-D16).
 */
function maybeMeterInputs(state: PuzzleState): MeterInputs | undefined {
  switch (state.kind) {
    case "loading":
      return undefined;
    case "building":
    case "running": {
      const { status } = state;
      return {
        shownCount: status.goalCount,
        goalTarget: status.goalTarget,
        goalFull: status.goalFull,
        brimCount: status.brimCount,
        won: status.outcome === "won",
        brimLatched: status.maybeBrimStep !== undefined,
      };
    }
    case "ended": {
      const { latch } = state;
      const won = latch.outcome === "won";
      return {
        shownCount: won ? latch.peakHeldCount : latch.goalCount,
        goalTarget: latch.goalTarget,
        goalFull: latch.goalFull,
        brimCount: latch.brimCount,
        won,
        brimLatched: latch.maybeBrimStep !== undefined,
      };
    }
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

function meterTicks(inputs: MeterInputs): readonly MeterTick[] {
  return goalTicks(inputs.goalTarget, inputs.goalFull, inputs.brimCount).map((tick) => {
    const reached =
      (tick.label === WIN_TICK_LABEL && inputs.won) ||
      (tick.label === BRIM_TICK_LABEL && inputs.brimLatched);
    return { ...tick, star: tick.label.startsWith("★"), reached };
  });
}

/** Fill meter model: placeholders while loading, live counts while running, frozen once decided. */
export function fillMeterModel(state: PuzzleState): FillMeterModel {
  const maybeInputs = maybeMeterInputs(state);
  if (maybeInputs === undefined) {
    return LOADING_FILL_METER;
  }

  const fill = floorPercent(maybeInputs.shownCount, maybeInputs.goalFull);
  const target = floorPercent(maybeInputs.goalTarget, maybeInputs.goalFull);
  return {
    percentText: `${fill}%`,
    targetText: ` / ${target}%`,
    fillPercent: fill,
    ticks: meterTicks(maybeInputs),
    maybeValueNow: fill,
    maybeValueText: `${fill}% full, target ${target}%`,
  };
}

/** Result slot contents (D-10, D-14, D-16). */
export type ResultCardModel =
  | { readonly kind: "loading"; readonly message: string }
  | {
      readonly kind: "goal";
      readonly message: string;
      readonly tone: "status" | "hint" | "goal";
    }
  | {
      readonly kind: "deciding";
      readonly outcomeWord: "Filled!";
      readonly pendingText: string;
      readonly text: string;
    }
  | {
      readonly kind: "outcome";
      readonly outcome: "won" | "lost";
      readonly outcomeWord: "Filled!" | "Out of time:";
      readonly scoreText: string;
      readonly text: string;
    };

/**
 * Result slot model. Building copy reads only the committed inspection
 * (UI-D14); a won run shows "Filled!" until its decision, then scores the
 * peak-held count; a loss scores the latched count (D-10, D-14, D-16).
 */
export function resultCardModel(state: PuzzleState): ResultCardModel {
  switch (state.kind) {
    case "loading":
      return { kind: "loading", message: "Loading level…" };
    case "building": {
      const goal = goalText(state.status.goalTarget, state.status.goalFull);
      const message = buildingMessage(state.editor, goal);
      return { kind: "goal", message: message.text, tone: message.tone };
    }
    case "running": {
      const { status } = state;
      if (status.outcome !== "won") {
        const message = goalText(status.goalTarget, status.goalFull);
        return { kind: "goal", message, tone: "goal" };
      }
      const pendingText = brimPendingText(status.brimCount, status.goalFull);
      const text = `Filled! ${pendingText}`;
      return { kind: "deciding", outcomeWord: "Filled!", pendingText, text };
    }
    case "ended": {
      const { latch } = state;
      const won = latch.outcome === "won";
      const outcomeWord = won ? "Filled!" : "Out of time:";
      const scoreCount = won ? latch.peakHeldCount : latch.goalCount;
      const scoreText = `${floorPercent(scoreCount, latch.goalFull)}% of ${floorPercent(latch.goalTarget, latch.goalFull)}%`;
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
      maybeHref: puzzleHash(maybeNextId),
    },
    primary: "next",
  };
}

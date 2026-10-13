import { committedPlacementsValid, selectedValve } from "./editor";
import { statusLine, trayItems, type TrayItemModel } from "./editor-format";
import { floorPercent } from "./format";
import type { PuzzleToolKind } from "./placements";
import type { PuzzleState } from "./reduce-puzzle";
import { speedLabel, type PuzzleSpeed } from "./speed";
import type { StarThresholds } from "./star-format";
import { runStars, type RunStars } from "./stars";
import { maxOpenTick, valveSecondsText } from "./valve-format";

/** Play/Stop, Speed and Clear contents (D-21, D-24, TOOL-04, TOOL-07). */
export type TransportModel = {
  readonly mode: "play" | "stop";
  readonly playEnabled: boolean;
  readonly playAccent: boolean;
  readonly maybeDisabledReason: string | undefined;
  readonly speed: PuzzleSpeed;
  readonly speedText: string;
  readonly speedAccessibleName: string;
  readonly speedTitle: string;
  readonly speedEnabled: boolean;
  readonly clearEnabled: boolean;
};

type PlayControl = Pick<
  TransportModel,
  "mode" | "playEnabled" | "playAccent" | "maybeDisabledReason"
>;

const DISABLED_PLAY: PlayControl = {
  mode: "play",
  playEnabled: false,
  playAccent: false,
  maybeDisabledReason: undefined,
};
const ACCENTED_PLAY: PlayControl = { ...DISABLED_PLAY, playEnabled: true, playAccent: true };
const ENABLED_STOP: PlayControl = { ...DISABLED_PLAY, mode: "stop", playEnabled: true };

/** Play while building reads only the committed snapshot, so live drags never flicker it (UI-D14). */
function playControl(state: PuzzleState): PlayControl {
  switch (state.kind) {
    case "building": {
      const { editor } = state;
      if (editor.tools.length === 0 || committedPlacementsValid(editor)) {
        return ACCENTED_PLAY;
      }
      return { ...DISABLED_PLAY, maybeDisabledReason: statusLine(editor) };
    }
    case "running":
      return ENABLED_STOP;
    case "loading":
    case "ended":
      return DISABLED_PLAY;
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

/** A failed level keeps its Play or Stop face but disables it; only Try again leaves the failure. */
function failedControl(state: PuzzleState): PlayControl {
  return state.kind === "running" ? { ...ENABLED_STOP, playEnabled: false } : DISABLED_PLAY;
}

/**
 * Transport model for every phase; running (including deciding) offers Stop.
 * While `failed`, Play, Stop and Clear are disabled (the shell ignores them too).
 */
export function transportModel(state: PuzzleState, failed = false): TransportModel {
  const speedText = speedLabel(state.speed);
  return {
    ...(failed ? failedControl(state) : playControl(state)),
    speed: state.speed,
    speedText,
    speedAccessibleName: `Speed ${speedText}`,
    speedTitle: `Speed ${speedText}. Press to change.`,
    speedEnabled: state.kind !== "loading",
    clearEnabled: !failed && state.kind === "building" && state.editor.tools.length > 0,
  };
}

/** Tray items from the editor budget and placed kinds; disabled outside building (D-17). */
export function trayModel(state: PuzzleState): readonly TrayItemModel<PuzzleToolKind>[] {
  const { editor } = state;
  return trayItems(
    editor.budget,
    editor.tools.map((tool) => tool.placement.kind),
    editor.maybeArmedKind,
    state.kind === "building",
  );
}

/** Selection bar is live only while building with a selected tool (D-19). */
export function selectionBarEnabled(state: PuzzleState): boolean {
  return state.kind === "building" && state.editor.maybeSelectedId !== undefined;
}

/** Valve open-time stepper contents; always mounted, live only for a selected valve while building (D-05). */
export type ValveStepperModel = {
  readonly enabled: boolean;
  readonly earlierEnabled: boolean;
  readonly laterEnabled: boolean;
  readonly text: string;
};

const DISABLED_VALVE_STEPPER: ValveStepperModel = {
  enabled: false,
  earlierEnabled: false,
  laterEnabled: false,
  text: "\u2013",
};

/** Stepper model; each direction is disabled at its clamp (`1..=stepLimit/30`). */
export function valveStepperModel(state: PuzzleState): ValveStepperModel {
  if (state.kind !== "building") {
    return DISABLED_VALVE_STEPPER;
  }
  const maybeValve = selectedValve(state.editor);
  if (maybeValve === undefined) {
    return DISABLED_VALVE_STEPPER;
  }

  const { openTick } = maybeValve.placement;
  return {
    enabled: true,
    earlierEnabled: openTick > 1,
    laterEnabled: openTick < maxOpenTick(state.status.stepLimit),
    text: valveSecondsText(openTick),
  };
}

/** This run's stars: none before a run, live latches while running, the latch once ended (D-13). */
export function runStarsForState(state: PuzzleState, parTools: number): RunStars | undefined {
  switch (state.kind) {
    case "loading":
    case "building":
      return undefined;
    case "running":
      return runStars({
        won: state.status.outcome === "won",
        brimLatched: state.status.maybeBrimStep !== undefined,
        placementsAtPlay: state.editor.tools.length,
        parTools,
      });
    case "ended":
      return runStars({
        won: state.latch.outcome === "won",
        brimLatched: state.latch.maybeBrimStep !== undefined,
        placementsAtPlay: state.editor.tools.length,
        parTools,
      });
    default: {
      const unreachable: never = state;
      return unreachable;
    }
  }
}

/** Star-row thresholds as whole percents from the frame or latch, never TS constants (D-13). */
export function starThresholdsFor(
  state: PuzzleState,
  parTools: number,
): StarThresholds | undefined {
  if (state.kind === "loading") {
    return undefined;
  }

  const counts = state.kind === "ended" ? state.latch : state.status;
  return {
    targetPct: floorPercent(counts.goalTarget, counts.goalFull),
    brimPct: floorPercent(counts.brimCount, counts.goalFull),
    parTools,
  };
}

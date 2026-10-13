import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import { type PlacementStatusCode, type Quad } from "../src/puzzle/placement-status";
import {
  fillMeterModel,
  formatSimulationTime,
  resultCardModel,
  timerText,
} from "../src/puzzle/format";
import {
  runStarsForState,
  selectionBarEnabled,
  starThresholdsFor,
  transportModel,
  trayModel,
} from "../src/puzzle/hud-format";
import {
  INITIAL_PUZZLE_STATE,
  reducePuzzle,
  type PuzzleEvent,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";

const WIN_STEP = 201;
const BRIM_STEP = 262;
const PAR_TOOLS = 1;
const QUAD: Quad = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 0.1 },
  { x: 0, y: 0.1 },
];
const EMPTY_HINT =
  "Fill the cup to 60% before time runs out. Pick a tool from the tray to start.";
const STATUS_LINE = "Overlaps the cup. Move or remove that tool to play.";

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
  goalCount: 700,
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
const LOST_STATUS = status({
  outcome: "lost",
  maybeOutcomeStep: 720,
  goalCount: 300,
  peakHeldCount: 200,
  maybeDecidedStep: 720,
});

function inspected(
  state: PuzzleState,
  statuses: readonly PlacementStatusCode[],
  mode: "commit" | "live",
): PuzzleEvent {
  const inspection = {
    revision: state.editor.revision,
    statuses,
    outlines: statuses.map(() => QUAD),
  };
  return { kind: "inspected", inspection, mode };
}

function building(): PuzzleState {
  return reducePuzzle(INITIAL_PUZZLE_STATE, {
    kind: "loaded",
    stepIndex: 0,
    status: status(),
    budget: [{ kind: "plank", count: 2 }],
  });
}

function withTool(statuses: readonly PlacementStatusCode[] = [0]): PuzzleState {
  const placed = reducePuzzle(building(), { kind: "place", toolKind: "plank", x: 10, y: 64 });
  return reducePuzzle(placed, inspected(placed, statuses, "commit"));
}

function running(stepIndex: number, frameStatus = status()): PuzzleState {
  const started = reducePuzzle(withTool(), { kind: "play" });
  if (started.kind !== "running") {
    throw new Error("Test fixture did not start running");
  }
  return stepIndex === 0 ? started : { ...started, stepIndex, status: frameStatus };
}

function deciding(): PuzzleState {
  return reducePuzzle(running(WIN_STEP - 1), {
    kind: "stepped",
    stepIndex: WIN_STEP,
    status: WIN_STATUS,
  });
}

function endedWon(): PuzzleState {
  return reducePuzzle(running(BRIM_STEP - 1, WIN_STATUS), {
    kind: "stepped",
    stepIndex: BRIM_STEP,
    status: BRIM_STATUS,
  });
}

function endedLost(): PuzzleState {
  return reducePuzzle(running(719), { kind: "stepped", stepIndex: 720, status: LOST_STATUS });
}

describe("committed snapshot rule (UI-D14)", () => {
  it("a live move and live invalid inspection change neither the card nor Play", () => {
    // Arrange
    const state = withTool([0]);
    const cardBefore = resultCardModel(state);
    const moved = reducePuzzle(state, { kind: "move", id: 1, x: 30, y: 64 });

    // Act
    const dragging = reducePuzzle(moved, inspected(moved, [8], "live"));

    // Assert
    expect(dragging.editor.maybeInspection?.statuses).toEqual([8]);
    expect(resultCardModel(dragging)).toEqual(cardBefore);
    expect(transportModel(dragging).playEnabled).toBe(true);
  });

  it("the following commit inspection shows the status line and disables Play", () => {
    // Arrange
    const moved = reducePuzzle(withTool([0]), { kind: "move", id: 1, x: 30, y: 64 });
    const dragging = reducePuzzle(moved, inspected(moved, [8], "live"));

    // Act
    const committedState = reducePuzzle(dragging, inspected(dragging, [8], "commit"));

    // Assert
    expect(resultCardModel(committedState)).toEqual({
      kind: "goal",
      tone: "status",
      message: STATUS_LINE,
    });
    expect(transportModel(committedState).playEnabled).toBe(false);
  });
});

describe("fillMeterModel ticks and decision freeze", () => {
  it.each([
    ["building", building, [false, false, false]],
    ["running", () => running(100, status({ goalCount: 300 })), [false, false, false]],
    ["deciding", deciding, [true, false, false]],
    ["ended won with Brim", endedWon, [true, true, false]],
    ["ended lost", endedLost, [false, false, false]],
  ] as const)("marks reached ★ ticks while %s", (_label, makeState, expected) => {
    // Act
    const ticks = fillMeterModel(makeState()).ticks;

    // Assert
    expect(ticks.map((tick) => tick.reached)).toEqual(expected);
    expect(ticks.map((tick) => tick.star)).toEqual([true, true, false]);
    expect(ticks.map((tick) => tick.label)).toEqual(["★ Win", "★ Brim", "Full"]);
  });

  it("freezes a won run at the peak-held percent", () => {
    // Act
    const model = fillMeterModel(endedWon());

    // Assert
    expect(model).toMatchObject({
      percentText: "90%",
      fillPercent: 90,
      maybeValueNow: 90,
      maybeValueText: "90% full, target 60%",
    });
  });

  it("freezes a lost run at its latched goal count", () => {
    // Act
    const model = fillMeterModel(endedLost());

    // Assert
    expect(model.percentText).toBe("33%");
  });

  it("follows the live count while deciding", () => {
    // Act
    const model = fillMeterModel(deciding());

    // Assert
    expect(model.percentText).toBe("77%");
  });
});

describe("timerText at the decision step", () => {
  it.each([
    ["ended won at decided step 262", endedWon, "0:07.6"],
    ["ended lost", endedLost, "0:00.0"],
    ["running at step 230", () => running(230), formatSimulationTime(490)],
  ] as const)("shows %s as %s", (_label, makeState, expected) => {
    // Act
    const text = timerText(makeState());

    // Assert
    expect(text).toBe(expected);
  });
});

describe("resultCardModel phases", () => {
  it("shows Filled! and the Brim target while deciding", () => {
    // Act
    const model = resultCardModel(deciding());

    // Assert
    expect(model).toEqual({
      kind: "deciding",
      outcomeWord: "Filled!",
      pendingText: "Going for Brim at 90%",
      text: "Filled! Going for Brim at 90%",
    });
  });

  it("scores a won run with the peak-held percent", () => {
    // Act
    const model = resultCardModel(endedWon());

    // Assert
    expect(model).toMatchObject({ kind: "outcome", outcome: "won", text: "Filled! 90% of 60%" });
  });

  it("shows the status line while building with an invalid tool", () => {
    // Act
    const model = resultCardModel(withTool([8]));

    // Assert
    expect(model).toEqual({ kind: "goal", tone: "status", message: STATUS_LINE });
  });

  it("shows the empty hint while building with zero tools", () => {
    // Act
    const model = resultCardModel(building());

    // Assert
    expect(model).toEqual({ kind: "goal", tone: "hint", message: EMPTY_HINT });
  });

  it("shows the goal while running before the win", () => {
    // Act
    const model = resultCardModel(running(50));

    // Assert
    expect(model).toEqual({
      kind: "goal",
      tone: "goal",
      message: "Fill the cup to 60% before time runs out.",
    });
  });
});

describe("transportModel", () => {
  it.each([
    ["zero tools", building],
    ["valid committed tools", () => withTool([0])],
  ] as const)("enables an accented Play while building with %s", (_label, makeState) => {
    // Act
    const model = transportModel(makeState());

    // Assert
    expect(model).toMatchObject({
      mode: "play",
      playEnabled: true,
      playAccent: true,
      maybeDisabledReason: undefined,
    });
  });

  it("disables Play with the status line as the reason for an invalid tool", () => {
    // Act
    const model = transportModel(withTool([8]));

    // Assert
    expect(model).toMatchObject({
      mode: "play",
      playEnabled: false,
      playAccent: false,
      maybeDisabledReason: STATUS_LINE,
    });
  });

  it.each([
    ["running", () => running(10)],
    ["deciding", deciding],
  ] as const)("offers an enabled default Stop while %s", (_label, makeState) => {
    // Act
    const model = transportModel(makeState());

    // Assert
    expect(model).toMatchObject({ mode: "stop", playEnabled: true, playAccent: false });
  });

  it.each([
    ["ended", endedWon],
    ["loading", () => INITIAL_PUZZLE_STATE],
  ] as const)("disables Play while %s", (_label, makeState) => {
    // Act
    const model = transportModel(makeState());

    // Assert
    expect(model).toMatchObject({ mode: "play", playEnabled: false, playAccent: false });
  });

  it.each([
    ["building", () => withTool([0]), "play"],
    ["running", () => running(10), "stop"],
  ] as const)("disables Play/Stop and Clear while failed in %s", (_label, makeState, mode) => {
    // Act
    const model = transportModel(makeState(), true);

    // Assert
    expect(model).toMatchObject({
      mode,
      playEnabled: false,
      playAccent: false,
      clearEnabled: false,
    });
  });

  it("describes the speed with text, name and title", () => {
    // Arrange
    const state = reducePuzzle(building(), { kind: "setSpeed", speed: 2 });

    // Act
    const model = transportModel(state);

    // Assert
    expect(model).toMatchObject({
      speed: 2,
      speedText: "2×",
      speedAccessibleName: "Speed 2×",
      speedTitle: "Speed 2×. Press to change.",
    });
  });

  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE, false, false],
    ["building with zero tools", building, true, false],
    ["building with a tool", () => withTool([0]), true, true],
    ["running", () => running(10), true, false],
    ["ended", endedLost, true, false],
  ] as const)("enables Speed and Clear correctly while %s", (_label, makeState, speed, clear) => {
    // Act
    const model = transportModel(makeState());

    // Assert
    expect([model.speedEnabled, model.clearEnabled]).toEqual([speed, clear]);
  });
});

describe("trayModel and selectionBarEnabled", () => {
  it("counts remaining tools from the editor budget while building", () => {
    // Act
    const items = trayModel(withTool([0]));

    // Assert
    expect(items).toMatchObject([
      { kind: "plank", remaining: 1, countText: "×1", armed: false, disabled: false },
    ]);
  });

  it("disables the tray outside building", () => {
    // Act
    const items = trayModel(running(10));

    // Assert
    expect(items.map((item) => item.disabled)).toEqual([true]);
  });

  it.each([
    ["building with a selected tool", () => withTool([0]), true],
    ["building with nothing selected", building, false],
    ["running", () => running(10), false],
  ] as const)("enables the selection bar only while %s", (_label, makeState, expected) => {
    // Act
    const enabled = selectionBarEnabled(makeState());

    // Assert
    expect(enabled).toBe(expected);
  });
});

describe("runStarsForState and starThresholdsFor", () => {
  it.each([
    ["loading", () => INITIAL_PUZZLE_STATE],
    ["building", building],
  ] as const)("has no run stars while %s", (_label, makeState) => {
    // Act
    const stars = runStarsForState(makeState(), PAR_TOOLS);

    // Assert
    expect(stars).toBeUndefined();
  });

  it.each([
    ["running before the win", () => running(10), { win: false, brim: false, thrifty: false }],
    ["deciding", deciding, { win: true, brim: false, thrifty: true }],
    ["ended won with Brim", endedWon, { win: true, brim: true, thrifty: true }],
    ["ended lost", endedLost, { win: false, brim: false, thrifty: false }],
  ] as const)("reflects the latches while %s", (_label, makeState, expected) => {
    // Act
    const stars = runStarsForState(makeState(), PAR_TOOLS);

    // Assert
    expect(stars).toEqual(expected);
  });

  it("withholds Thrifty when the run used more tools than par", () => {
    // Act
    const stars = runStarsForState(endedWon(), 0);

    // Assert
    expect(stars).toEqual({ win: true, brim: true, thrifty: false });
  });

  it.each([
    ["building", building],
    ["ended", endedWon],
  ] as const)("reads whole-percent thresholds while %s", (_label, makeState) => {
    // Act
    const thresholds = starThresholdsFor(makeState(), PAR_TOOLS);

    // Assert
    expect(thresholds).toEqual({ targetPct: 60, brimPct: 90, parTools: 1 });
  });

  it("has no thresholds while loading", () => {
    // Act
    const thresholds = starThresholdsFor(INITIAL_PUZZLE_STATE, PAR_TOOLS);

    // Assert
    expect(thresholds).toBeUndefined();
  });
});

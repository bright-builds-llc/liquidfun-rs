import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import {
  fillMeterModel,
  floorPercent,
  formatSimulationTime,
  goalText,
  goalTicks,
  resultActionsModel,
  resultCardModel,
  timerText,
} from "../src/puzzle/format";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import {
  INITIAL_PUZZLE_STATE,
  SETTLE_TAIL_STEPS,
  type PuzzleLatch,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";

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

function building(goalCount = 0): PuzzleState {
  return { kind: "building", status: status({ goalCount }), editor: EMPTY_EDITOR, speed: 1 };
}

function running(stepIndex: number, goalCount = 0): PuzzleState {
  return {
    kind: "running",
    stepIndex,
    status: status({ goalCount }),
    editor: EMPTY_EDITOR,
    speed: 1,
  };
}

function ended(latch: Partial<PuzzleLatch> = {}, stepIndex?: number): PuzzleState {
  const full: PuzzleLatch = {
    outcome: "won",
    outcomeStep: 201,
    decidedStep: 201,
    goalCount: 689,
    peakHeldCount: 689,
    maybeBrimStep: undefined,
    goalTarget: 540,
    goalFull: 900,
    brimCount: 810,
    stepLimit: 720,
    ...latch,
  };
  return {
    kind: "ended",
    stepIndex: stepIndex ?? full.decidedStep,
    latch: full,
    tailEndStep: full.decidedStep + SETTLE_TAIL_STEPS,
    held: false,
    editor: EMPTY_EDITOR,
    speed: 1,
  };
}

describe("formatSimulationTime", () => {
  it.each([
    [720, "0:12.0"],
    [519, "0:08.6"],
    [0, "0:00.0"],
    [5, "0:00.0"],
    [6, "0:00.1"],
    [3600, "1:00.0"],
    [-10, "0:00.0"],
  ])("formats %i remaining steps as %s", (steps, expected) => {
    // Act
    const text = formatSimulationTime(steps);

    // Assert
    expect(text).toBe(expected);
  });
});

describe("floorPercent", () => {
  it.each([
    [689, 900, 76],
    [540, 900, 60],
    [539, 900, 59],
    [0, 900, 0],
    [900, 900, 100],
    [950, 900, 100],
  ])("rounds %i of %i down to %i%%", (count, full, expected) => {
    // Act
    const percent = floorPercent(count, full);

    // Assert
    expect(percent).toBe(expected);
  });

  it("rejects a capacity of zero", () => {
    // Act
    const act = () => floorPercent(10, 0);

    // Assert
    expect(act).toThrow("Puzzle capacity must be positive");
  });
});

describe("goal copy and ticks", () => {
  it("lists the win, Brim and full ticks in order", () => {
    // Act
    const ticks = goalTicks(540, 900, 810);

    // Assert
    expect(ticks).toEqual([
      { fraction: 0.6, label: "★ Win" },
      { fraction: 0.9, label: "★ Brim" },
      { fraction: 1, label: "Full" },
    ]);
  });

  it("states the goal as a rounded-down target percentage", () => {
    // Act
    const text = goalText(540, 900);

    // Assert
    expect(text).toBe("Fill the cup to 60% before time runs out.");
  });
});

describe("timerText", () => {
  it.each([
    ["loading", INITIAL_PUZZLE_STATE, "-:--.-"],
    ["building", building(), "0:12.0"],
    ["running at step 100", running(100), "0:10.3"],
    ["ended won at 201 later in the tail", ended({}, 300), "0:08.6"],
  ] as const)("shows %s as %s", (_label, state, expected) => {
    // Act
    const text = timerText(state);

    // Assert
    expect(text).toBe(expected);
  });
});

describe("fillMeterModel", () => {
  it("shows placeholders and no ticks while loading", () => {
    // Act
    const model = fillMeterModel(INITIAL_PUZZLE_STATE);

    // Assert
    expect(model).toEqual({
      percentText: "--%",
      targetText: " / --%",
      fillPercent: 0,
      ticks: [],
      maybeValueNow: undefined,
      maybeValueText: undefined,
    });
  });

  it("shows the empty cup, target and ticks while building", () => {
    // Act
    const model = fillMeterModel(building());

    // Assert
    expect(model).toEqual({
      percentText: "0%",
      targetText: " / 60%",
      fillPercent: 0,
      ticks: [
        { fraction: 0.6, label: "★ Win", star: true, reached: false },
        { fraction: 0.9, label: "★ Brim", star: true, reached: false },
        { fraction: 1, label: "Full", star: false, reached: false },
      ],
      maybeValueNow: 0,
      maybeValueText: "0% full, target 60%",
    });
  });

  it("follows the live count while running", () => {
    // Act
    const model = fillMeterModel(running(300, 486));

    // Assert
    expect(model.percentText).toBe("54%");
  });

  it("freezes at the latched count once ended", () => {
    // Arrange
    const state = ended({ goalCount: 689 }, 320);

    // Act
    const model = fillMeterModel(state);

    // Assert
    expect(model).toMatchObject({
      percentText: "76%",
      fillPercent: 76,
      maybeValueNow: 76,
      maybeValueText: "76% full, target 60%",
    });
  });
});

describe("resultCardModel", () => {
  it("shows the loading message while loading", () => {
    // Act
    const model = resultCardModel(INITIAL_PUZZLE_STATE);

    // Assert
    expect(model).toEqual({ kind: "loading", message: "Loading level…" });
  });

  it("shows the empty hint while building with zero tools", () => {
    // Act
    const model = resultCardModel(building());

    // Assert
    expect(model).toEqual({
      kind: "goal",
      tone: "hint",
      message: "Fill the cup to 60% before time runs out. Pick a tool from the tray to start.",
    });
  });

  it("shows the goal while running", () => {
    // Act
    const model = resultCardModel(running(50, 120));

    // Assert
    expect(model).toEqual({
      kind: "goal",
      tone: "goal",
      message: "Fill the cup to 60% before time runs out.",
    });
  });

  it("reports a win with rounded-down fill and target", () => {
    // Act
    const model = resultCardModel(ended());

    // Assert
    expect(model).toEqual({
      kind: "outcome",
      outcome: "won",
      outcomeWord: "Filled!",
      scoreText: "76% of 60%",
      text: "Filled! 76% of 60%",
    });
  });

  it.each([
    ["the empty first-pour run", { goalCount: 0, goalTarget: 540 }, "Out of time: 0% of 60%"],
    ["the requirement example", { goalCount: 486, goalTarget: 720 }, "Out of time: 54% of 80%"],
    ["a loss at or above the target", { goalCount: 549, goalTarget: 540 }, "Out of time: 61% of 60%"],
  ] as const)("reports %s as a loss", (_label, counts, expected) => {
    // Arrange
    const state = ended({ outcome: "lost", outcomeStep: 720, decidedStep: 720, ...counts });

    // Act
    const model = resultCardModel(state);

    // Assert
    expect(model).toMatchObject({ kind: "outcome", outcomeWord: "Out of time:", text: expected });
  });
});

describe("resultActionsModel", () => {
  it("turns Next into an All levels link to the grid on the last level", () => {
    // Act
    const model = resultActionsModel(undefined);

    // Assert
    expect(model).toEqual({
      next: {
        enabled: true,
        label: "All levels",
        ariaLabel: "All levels",
        maybeTitle: undefined,
        maybeHref: "#/puzzles",
      },
      primary: "next",
    });
  });

  it("links Next to the next level and promotes it", () => {
    // Act
    const model = resultActionsModel("first-pour");

    // Assert
    expect(model).toEqual({
      next: {
        enabled: true,
        label: "Next",
        ariaLabel: "Next level",
        maybeTitle: undefined,
        maybeHref: "#/puzzle/first-pour",
      },
      primary: "next",
    });
  });
});

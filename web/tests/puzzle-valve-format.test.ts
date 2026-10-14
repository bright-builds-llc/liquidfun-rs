import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import { editorFromPlacements, selectTool } from "../src/puzzle/editor";
import { valveStepperModel } from "../src/puzzle/hud-format";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../src/puzzle/reduce-puzzle";
import {
  maxOpenTick,
  openStepFor,
  valveCountdownText,
  valveSecondsText,
} from "../src/puzzle/valve-format";

const BUDGET = [
  { kind: "plank", count: 2 },
  { kind: "valve", count: 1 },
] as const;

function status(): PuzzleFrameStatus {
  return {
    goalCount: 0,
    goalTarget: 540,
    goalFull: 900,
    outcome: "running",
    stepLimit: 720,
    peakHeldCount: 0,
    brimCount: 810,
  };
}

function buildingWith(placement: PuzzlePlacement, selected = true): PuzzleState {
  const editor = editorFromPlacements([placement], BUDGET);
  return {
    kind: "building",
    status: status(),
    editor: selected ? selectTool(editor, 1) : editor,
    speed: 1,
  };
}

function valveAt(openTick: number): PuzzlePlacement {
  return { kind: "valve", x: 10, y: 64, angle: 0, openTick };
}

const DISABLED = {
  enabled: false,
  earlierEnabled: false,
  laterEnabled: false,
  text: "–",
};

describe("valve time arithmetic", () => {
  it("allows whole half-seconds within the step limit", () => {
    expect(maxOpenTick(720)).toBe(24);
  });

  it("opens at thirty steps per tick", () => {
    expect(openStepFor(4)).toBe(120);
  });

  it("caps the open tick at the u8 maximum above 7,650 steps", () => {
    expect(maxOpenTick(9_000)).toBe(255);
  });

  it("keeps 255 at exactly 7,650 steps", () => {
    expect(maxOpenTick(7_650)).toBe(255);
  });
});

describe("valveSecondsText", () => {
  it.each([
    [4, "2.0 s"],
    [1, "0.5 s"],
    [24, "12.0 s"],
  ])("formats tick %i as %s", (openTick, text) => {
    expect(valveSecondsText(openTick)).toBe(text);
  });
});

describe("valveCountdownText", () => {
  it.each([
    [0, "2.0 s"],
    [61, "1.0 s"],
    [119, "0.1 s"],
  ])("counts down from step %i as %s, rounding up to a tenth", (stepIndex, text) => {
    expect(valveCountdownText(120, stepIndex)).toBe(text);
  });
});

describe("valveStepperModel", () => {
  it("enables both directions for a selected valve at 2.0 s", () => {
    // Act
    const model = valveStepperModel(buildingWith(valveAt(4)));

    // Assert
    expect(model).toEqual({
      enabled: true,
      earlierEnabled: true,
      laterEnabled: true,
      text: "2.0 s",
    });
  });

  it("disables earlier at half a second", () => {
    // Act
    const model = valveStepperModel(buildingWith(valveAt(1)));

    // Assert
    expect(model.earlierEnabled).toBe(false);
  });

  it("disables later at the step limit", () => {
    // Act
    const model = valveStepperModel(buildingWith(valveAt(24)));

    // Assert
    expect(model.laterEnabled).toBe(false);
  });

  it("is disabled with a placeholder when no valve is selected", () => {
    // Arrange
    const states = [
      buildingWith(valveAt(4), false),
      buildingWith({ kind: "plank", x: 10, y: 64, angle: 0, openTick: 0 }),
    ];

    // Act
    const models = states.map(valveStepperModel);

    // Assert
    expect(models).toEqual([DISABLED, DISABLED]);
  });

  it("is disabled outside building", () => {
    // Arrange
    const building = buildingWith(valveAt(4));
    const running: PuzzleState = {
      ...building,
      kind: "running",
      stepIndex: 0,
      status: status(),
    };

    // Act
    const models = [INITIAL_PUZZLE_STATE, running].map(valveStepperModel);

    // Assert
    expect(models).toEqual([DISABLED, DISABLED]);
  });
});

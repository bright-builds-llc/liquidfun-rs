import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import { puzzleDebugAttributes } from "../src/puzzle/debug-attributes";
import {
  EMPTY_EDITOR,
  applyInspection,
  armKind,
  editorFromPlacements,
  selectTool,
  type BuildingEditor,
} from "../src/puzzle/editor";
import type { Quad } from "../src/puzzle/placement-status";
import type { PuzzleLatch, PuzzleState } from "../src/puzzle/reduce-puzzle";

const QUAD: Quad = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 0.25 },
  { x: 0, y: 0.25 },
];
const HELD_HASH = "0123456789abcdef";

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

function twoPlanks(): BuildingEditor {
  const editor = editorFromPlacements(
    [
      { kind: "plank", x: 30, y: 84, angle: 0, openTick: 0 },
      { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 },
    ],
    [{ kind: "plank", count: 3 }],
  );
  const inspected = applyInspection(
    editor,
    { revision: editor.revision, statuses: [0, 0], outlines: [QUAD, QUAD] },
    "commit",
  );
  return armKind(selectTool(inspected, 2), "plank");
}

function latch(): PuzzleLatch {
  return {
    outcome: "won",
    outcomeStep: 262,
    decidedStep: 382,
    goalCount: 810,
    peakHeldCount: 812,
    maybeBrimStep: 382,
    goalTarget: 540,
    goalFull: 900,
    brimCount: 810,
    stepLimit: 720,
  };
}

function ended(): PuzzleState {
  return {
    kind: "ended",
    stepIndex: 502,
    latch: latch(),
    tailEndStep: 502,
    held: true,
    editor: twoPlanks(),
    speed: 4,
  };
}

describe("puzzleDebugAttributes", () => {
  it("describes a building editor with canonical placements and the selection index", () => {
    // Arrange
    const state: PuzzleState = { kind: "building", status: status(), editor: twoPlanks(), speed: 2 };

    // Act
    const attributes = puzzleDebugAttributes(state, undefined);

    // Assert
    expect(attributes).toEqual({
      "data-puzzle-placements": "plank:8,64,23;plank:30,84,0",
      "data-puzzle-selected": "1",
      "data-puzzle-armed": "plank",
      "data-puzzle-speed": "2",
      "data-puzzle-play-enabled": "true",
      "data-puzzle-deciding": "false",
    });
  });

  it("writes valves as kind:x,y,angle,openTick and keeps the plank form", () => {
    // Arrange
    const editor = editorFromPlacements(
      [
        { kind: "valve", x: -16, y: 87, angle: 0, openTick: 4 },
        { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 },
      ],
      [
        { kind: "plank", count: 1 },
        { kind: "valve", count: 1 },
      ],
    );
    const state: PuzzleState = { kind: "building", status: status(), editor, speed: 1 };

    // Act
    const attributes = puzzleDebugAttributes(state, undefined);

    // Assert
    expect(attributes["data-puzzle-placements"]).toBe("plank:8,64,23;valve:-16,87,0,4");
  });

  it("uses empty strings for an empty building editor", () => {
    // Arrange
    const state: PuzzleState = { kind: "building", status: status(), editor: EMPTY_EDITOR, speed: 1 };

    // Act
    const attributes = puzzleDebugAttributes(state, undefined);

    // Assert
    expect(attributes["data-puzzle-placements"]).toBe("");
    expect(attributes["data-puzzle-selected"]).toBe("");
    expect(attributes["data-puzzle-armed"]).toBe("");
  });

  it("reports the deciding state of a won run without a decision", () => {
    // Arrange
    const state: PuzzleState = {
      kind: "running",
      stepIndex: 270,
      status: status({ outcome: "won", maybeOutcomeStep: 262 }),
      editor: twoPlanks(),
      speed: 1,
    };

    // Act
    const attributes = puzzleDebugAttributes(state, undefined);

    // Assert
    expect(attributes["data-puzzle-deciding"]).toBe("true");
    expect(attributes["data-puzzle-play-enabled"]).toBe("false");
  });

  it("reports the latched steps and peak once ended", () => {
    // Arrange
    const state = ended();

    // Act
    const attributes = puzzleDebugAttributes(state, undefined);

    // Assert
    expect(attributes["data-puzzle-outcome-step"]).toBe("262");
    expect(attributes["data-puzzle-decided-step"]).toBe("382");
    expect(attributes["data-puzzle-peak-held"]).toBe("812");
    expect(attributes).not.toHaveProperty("data-puzzle-held-hash");
  });

  it("adds the held hash when provided", () => {
    // Arrange
    const state = ended();

    // Act
    const attributes = puzzleDebugAttributes(state, HELD_HASH);

    // Assert
    expect(attributes["data-puzzle-held-hash"]).toBe(HELD_HASH);
  });
});

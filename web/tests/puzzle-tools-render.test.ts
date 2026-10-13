import { describe, expect, it } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import {
  applyInspection,
  editorFromPlacements,
  selectTool,
  type BuildingEditor,
} from "../src/puzzle/editor";
import type { PlacementStatusCode, Quad } from "../src/puzzle/placement-status";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../src/puzzle/reduce-puzzle";
import { createCamera, IDENTITY_CAMERA_VIEW } from "../src/render/camera";
import { drawPuzzleTools, toolOverlay, type OverlayTool } from "../src/render/puzzle-tools";

const QUAD_A: Quad = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 0.25 },
  { x: 0, y: 0.25 },
];
const QUAD_B: Quad = [
  { x: 2, y: 1 },
  { x: 3, y: 1 },
  { x: 3, y: 1.25 },
  { x: 2, y: 1.25 },
];
const PLANKS: readonly PuzzlePlacement[] = [
  { kind: "plank", x: 10, y: 2, angle: 0 },
  { kind: "plank", x: 50, y: 22, angle: 0 },
];
const CAMERA = createCamera(400, 300, IDENTITY_CAMERA_VIEW, {
  minX: 0,
  minY: 0,
  maxX: 4,
  maxY: 3,
});

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

function inspectedEditor(
  statuses: readonly PlacementStatusCode[],
  outlines: readonly (Quad | undefined)[] = [QUAD_A, QUAD_B],
): BuildingEditor {
  const editor = editorFromPlacements(PLANKS);
  return applyInspection(editor, { revision: editor.revision, statuses, outlines }, "live");
}

function building(editor: BuildingEditor): PuzzleState {
  return { kind: "building", status: status(), editor, speed: 1 };
}

type Recorded =
  | { readonly kind: "call"; readonly name: string; readonly args: readonly unknown[] }
  | { readonly kind: "set"; readonly name: string; readonly value: unknown };

/** Fake 2D context that records every method call and property assignment in order. */
function recordingContext(maybeThrowOn?: string) {
  const log: Recorded[] = [];
  const target: Record<string, unknown> = {};
  const context = new Proxy(target, {
    get(_target, name: string) {
      return (...args: unknown[]) => {
        log.push({ kind: "call", name, args });
        if (name === maybeThrowOn) {
          throw new Error(`draw failed in ${name}`);
        }
      };
    },
    set(_target, name: string, value: unknown) {
      log.push({ kind: "set", name, value });
      return true;
    },
  });
  return { context: context as unknown as CanvasRenderingContext2D, log };
}

function calls(log: readonly Recorded[], name: string): number {
  return log.filter((entry) => entry.kind === "call" && entry.name === name).length;
}

/** Compact draw script: strokes and fills with the style that was current, plus dash changes. */
function drawScript(log: readonly Recorded[]): readonly string[] {
  const style: Record<string, unknown> = {};
  const script: string[] = [];
  for (const entry of log) {
    if (entry.kind === "set") {
      style[entry.name] = entry.value;
      continue;
    }
    if (entry.name === "stroke") {
      script.push(`stroke ${String(style.strokeStyle)} ${String(style.lineWidth)}`);
    } else if (entry.name === "fill") {
      script.push(`fill ${String(style.fillStyle)}`);
    } else if (entry.name === "setLineDash") {
      script.push(`dash ${JSON.stringify(entry.args[0])}`);
    }
  }
  return script;
}

describe("toolOverlay", () => {
  it("styles building tools from the live inspection and the selection", () => {
    // Arrange
    const editor = selectTool(inspectedEditor([0, 9]), 2);

    // Act
    const overlay = toolOverlay(building(editor));

    // Assert
    expect(overlay).toEqual([
      { quad: QUAD_A, style: "normal" },
      { quad: QUAD_B, style: "selectedInvalid" },
    ]);
  });

  it("follows a live inspection even while the committed snapshot is valid", () => {
    // Arrange
    const base = editorFromPlacements(PLANKS);
    const committed = applyInspection(
      base,
      { revision: base.revision, statuses: [0, 0], outlines: [QUAD_A, QUAD_B] },
      "commit",
    );
    const live = applyInspection(
      committed,
      { revision: committed.revision, statuses: [11, 0], outlines: [QUAD_A, QUAD_B] },
      "live",
    );

    // Act
    const overlay = toolOverlay(building(live));

    // Assert
    expect(overlay[0]?.style).toBe("invalid");
  });

  it("marks a selected valid tool as selected", () => {
    // Arrange
    const editor = selectTool(inspectedEditor([0, 0]), 1);

    // Act
    const overlay = toolOverlay(building(editor));

    // Assert
    expect(overlay.map((tool) => tool.style)).toEqual(["selected", "normal"]);
  });

  it("omits tools whose outline is undefined", () => {
    // Arrange
    const editor = inspectedEditor([4, 0], [undefined, QUAD_B]);

    // Act
    const overlay = toolOverlay(building(editor));

    // Assert
    expect(overlay).toEqual([{ quad: QUAD_B, style: "normal" }]);
  });

  it("draws nothing for a stale inspection whose length no longer matches", () => {
    // Arrange
    const inspected = inspectedEditor([0, 0]);
    const editor: BuildingEditor = {
      ...inspected,
      tools: [...inspected.tools, { id: 3, placement: { kind: "plank", x: 1, y: 1, angle: 0 } }],
    };

    // Act
    const overlay = toolOverlay(building(editor));

    // Assert
    expect(overlay).toEqual([]);
  });

  it("locks every outlined tool while running", () => {
    // Arrange
    const editor = selectTool(inspectedEditor([0, 9]), 2);
    const running: PuzzleState = {
      kind: "running",
      stepIndex: 40,
      status: status(),
      editor,
      speed: 1,
    };

    // Act
    const overlay = toolOverlay(running);

    // Assert
    expect(overlay).toEqual([
      { quad: QUAD_A, style: "locked" },
      { quad: QUAD_B, style: "locked" },
    ]);
  });

  it("locks every outlined tool once ended", () => {
    // Arrange
    const ended: PuzzleState = {
      kind: "ended",
      stepIndex: 400,
      latch: {
        outcome: "won",
        outcomeStep: 300,
        decidedStep: 380,
        goalCount: 700,
        peakHeldCount: 700,
        maybeBrimStep: undefined,
        goalTarget: 540,
        goalFull: 900,
        brimCount: 810,
        stepLimit: 720,
      },
      tailEndStep: 500,
      held: false,
      editor: inspectedEditor([0, 0]),
      speed: 1,
    };

    // Act
    const overlay = toolOverlay(ended);

    // Assert
    expect(overlay.map((tool) => tool.style)).toEqual(["locked", "locked"]);
  });

  it("draws nothing while loading", () => {
    // Arrange
    const loading: PuzzleState = { ...INITIAL_PUZZLE_STATE, editor: inspectedEditor([0, 0]) };

    // Act
    const overlay = toolOverlay(loading);

    // Assert
    expect(overlay).toEqual([]);
  });
});

describe("drawPuzzleTools", () => {
  const SELECTED_INVALID: readonly OverlayTool[] = [{ quad: QUAD_B, style: "selectedInvalid" }];

  it("draws halo, keyline, red fill and dashed red stroke for a selected invalid tool", () => {
    // Arrange
    const { context, log } = recordingContext();

    // Act
    drawPuzzleTools(context, CAMERA, SELECTED_INVALID);

    // Assert
    expect(drawScript(log)).toEqual([
      "stroke rgba(244, 247, 250, 0.40) 8",
      "stroke rgba(7, 16, 24, 0.80) 4",
      "fill rgba(90, 36, 48, 0.92)",
      "dash [6,4]",
      "stroke #F07178 2",
      "dash []",
    ]);
  });

  it("draws a normal tool with keyline, slate fill and solid stroke only", () => {
    // Arrange
    const { context, log } = recordingContext();

    // Act
    drawPuzzleTools(context, CAMERA, [{ quad: QUAD_A, style: "normal" }]);

    // Assert
    expect(drawScript(log)).toEqual([
      "stroke rgba(7, 16, 24, 0.80) 4",
      "fill rgba(51, 65, 85, 0.92)",
      "stroke #CBD5E1 2",
    ]);
  });

  it("projects the outline corners into CSS pixels with round joins", () => {
    // Arrange
    const { context, log } = recordingContext();

    // Act
    drawPuzzleTools(context, CAMERA, [{ quad: QUAD_A, style: "locked" }]);

    // Assert
    const moves = log.filter(
      (entry) => entry.kind === "call" && (entry.name === "moveTo" || entry.name === "lineTo"),
    );
    expect(moves.slice(0, 4).map((entry) => (entry.kind === "call" ? entry.args : []))).toEqual([
      [CAMERA.offsetX, CAMERA.offsetY + 3 * CAMERA.scale],
      [CAMERA.offsetX + CAMERA.scale, CAMERA.offsetY + 3 * CAMERA.scale],
      [CAMERA.offsetX + CAMERA.scale, CAMERA.offsetY + 2.75 * CAMERA.scale],
      [CAMERA.offsetX, CAMERA.offsetY + 2.75 * CAMERA.scale],
    ]);
    expect(log).toContainEqual({ kind: "set", name: "lineJoin", value: "round" });
  });

  it("saves and restores the context once even when a draw call throws", () => {
    // Arrange
    const { context, log } = recordingContext("fill");

    // Act
    const draw = () => drawPuzzleTools(context, CAMERA, SELECTED_INVALID);

    // Assert
    expect(draw).toThrow("draw failed in fill");
    expect(calls(log, "save")).toBe(1);
    expect(calls(log, "restore")).toBe(1);
  });

  it("does not touch the context when there are no tools", () => {
    // Arrange
    const { context, log } = recordingContext();

    // Act
    drawPuzzleTools(context, CAMERA, []);

    // Assert
    expect(log).toEqual([]);
  });
});

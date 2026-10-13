/**
 * Puzzle tool overlay (D-22, UI-D13).
 *
 * `toolOverlay` is pure: it turns the puzzle state and the latest inspector
 * outlines into styled quads. `drawPuzzleTools` paints them in world space
 * between the particle layer and the glass cup. Line widths are fixed CSS
 * pixels, so tools keep the same weight at every camera scale.
 */

import type { PuzzleState } from "../puzzle/reduce-puzzle";
import type { Quad } from "../puzzle/placement-status";
import { projectPoint, type Camera } from "./camera";

/** Draw style of one tool; locked tools look normal and appear while running and ended. */
export type ToolStyle = "normal" | "selected" | "invalid" | "selectedInvalid" | "locked";

/** One tool outline with its draw style. */
export type OverlayTool = { readonly quad: Quad; readonly style: ToolStyle };

type ToolPaint = {
  readonly halo: boolean;
  readonly fill: string;
  readonly stroke: string;
  readonly dashed: boolean;
};

const HALO_STROKE = "rgba(244, 247, 250, 0.40)";
const KEYLINE_STROKE = "rgba(7, 16, 24, 0.80)";
const HALO_WIDTH_PX = 8;
const KEYLINE_WIDTH_PX = 4;
const STROKE_WIDTH_PX = 2;

const NORMAL_FILL = "rgba(51, 65, 85, 0.92)";
const INVALID_FILL = "rgba(90, 36, 48, 0.92)";
const NORMAL_STROKE = "#CBD5E1";
const SELECTED_STROKE = "#F4F7FA";
const INVALID_STROKE = "#F07178";

const NORMAL_PAINT: ToolPaint = {
  halo: false,
  fill: NORMAL_FILL,
  stroke: NORMAL_STROKE,
  dashed: false,
};
const INVALID_PAINT: ToolPaint = {
  halo: false,
  fill: INVALID_FILL,
  stroke: INVALID_STROKE,
  dashed: true,
};

const PAINTS: Readonly<Record<ToolStyle, ToolPaint>> = {
  normal: NORMAL_PAINT,
  locked: NORMAL_PAINT,
  selected: { ...NORMAL_PAINT, halo: true, stroke: SELECTED_STROKE },
  invalid: INVALID_PAINT,
  selectedInvalid: { ...INVALID_PAINT, halo: true },
};

function buildingStyle(selected: boolean, valid: boolean): ToolStyle {
  if (selected) {
    return valid ? "selected" : "selectedInvalid";
  }

  return valid ? "normal" : "invalid";
}

/**
 * Tools to draw for a puzzle state. Reads the latest (live or commit)
 * inspection so the tint follows a drag at once (UI-D14). Tools without an
 * outline are omitted, and a stale inspection draws nothing.
 */
export function toolOverlay(state: PuzzleState): readonly OverlayTool[] {
  if (state.kind === "loading") {
    return [];
  }

  const { editor } = state;
  const maybeInspection = editor.maybeInspection;
  const stale =
    maybeInspection === undefined ||
    maybeInspection.outlines.length !== editor.tools.length ||
    maybeInspection.statuses.length !== editor.tools.length;
  if (stale) {
    return [];
  }

  const locked = state.kind !== "building";
  const overlay: OverlayTool[] = [];
  editor.tools.forEach((tool, index) => {
    const maybeQuad = maybeInspection.outlines[index];
    if (maybeQuad === undefined) {
      return;
    }
    const style = locked
      ? "locked"
      : buildingStyle(tool.id === editor.maybeSelectedId, maybeInspection.statuses[index] === 0);
    overlay.push({ quad: maybeQuad, style });
  });
  return overlay;
}

function traceQuad(context: CanvasRenderingContext2D, camera: Camera, quad: Quad): void {
  const [first, ...rest] = quad;
  const start = projectPoint(camera, first);
  context.beginPath();
  context.moveTo(start.x, start.y);
  for (const corner of rest) {
    const point = projectPoint(camera, corner);
    context.lineTo(point.x, point.y);
  }
  context.closePath();
}

function strokePass(context: CanvasRenderingContext2D, color: string, width: number): void {
  context.strokeStyle = color;
  context.lineWidth = width;
  context.stroke();
}

function drawTool(context: CanvasRenderingContext2D, camera: Camera, tool: OverlayTool): void {
  const paint = PAINTS[tool.style];
  traceQuad(context, camera, tool.quad);
  if (paint.halo) {
    strokePass(context, HALO_STROKE, HALO_WIDTH_PX);
  }
  strokePass(context, KEYLINE_STROKE, KEYLINE_WIDTH_PX);
  context.fillStyle = paint.fill;
  context.fill();
  if (paint.dashed) {
    context.setLineDash([6, 4]);
  }
  strokePass(context, paint.stroke, STROKE_WIDTH_PX);
  if (paint.dashed) {
    context.setLineDash([]);
  }
}

/** Draws tools after the particles and before the glass cup (D-22); restores every canvas state it changes. */
export function drawPuzzleTools(
  context: CanvasRenderingContext2D,
  camera: Camera,
  tools: readonly OverlayTool[],
): void {
  if (tools.length === 0) {
    return;
  }

  context.save();
  try {
    context.lineJoin = "round";
    for (const tool of tools) {
      drawTool(context, camera, tool);
    }
  } finally {
    context.restore();
  }
}

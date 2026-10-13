/**
 * Puzzle tool overlay (D-22, UI-D13).
 *
 * `toolOverlay` is pure: it turns the puzzle state and the latest inspector
 * outlines into styled quads. `drawPuzzleTools` paints them in world space
 * between the particle layer and the glass cup. Line widths are fixed CSS
 * pixels, so tools keep the same weight at every camera scale.
 */

import type { PuzzlePlacement } from "../puzzle/placements";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import type { Quad } from "../puzzle/placement-status";
import { openStepFor, valveCountdownText, valveSecondsText } from "../puzzle/valve-format";
import { projectPoint, type Camera } from "./camera";

/**
 * Draw style of one tool; locked tools look normal and appear while running
 * and ended. `open` is a valve whose gate has gone (Phase 40 D-06).
 */
export type ToolStyle = "normal" | "selected" | "invalid" | "selectedInvalid" | "locked" | "open";

/** One tool outline with its draw style and an optional valve time label. */
export type OverlayTool = {
  readonly quad: Quad;
  readonly style: ToolStyle;
  readonly maybeLabel: string | undefined;
};

type ToolPaint = {
  readonly halo: boolean;
  readonly keyline: boolean;
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
  keyline: true,
  fill: NORMAL_FILL,
  stroke: NORMAL_STROKE,
  dashed: false,
};
const INVALID_PAINT: ToolPaint = {
  halo: false,
  keyline: true,
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
  // A faint outline with no fill or keyline, matching the destroyed gate body.
  open: {
    halo: false,
    keyline: false,
    fill: "rgba(0, 0, 0, 0)",
    stroke: "rgba(203, 213, 225, 0.35)",
    dashed: true,
  },
};

const LABEL_FONT = "600 12px system-ui, sans-serif";
const LABEL_FILL = "#F4F7FA";
const LABEL_KEYLINE_WIDTH_PX = 3;

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

  const overlay: OverlayTool[] = [];
  editor.tools.forEach((tool, index) => {
    const maybeQuad = maybeInspection.outlines[index];
    if (maybeQuad === undefined) {
      return;
    }
    if (state.kind === "building") {
      const selected = tool.id === editor.maybeSelectedId;
      const style = buildingStyle(selected, maybeInspection.statuses[index] === 0);
      overlay.push({ quad: maybeQuad, style, maybeLabel: buildingLabel(tool.placement) });
      return;
    }
    overlay.push({ quad: maybeQuad, ...runStyle(tool.placement, state.stepIndex) });
  });
  return overlay;
}

/** A valve shows its open time while building; planks carry no label. */
function buildingLabel(placement: PuzzlePlacement): string | undefined {
  return placement.kind === "valve" ? valveSecondsText(placement.openTick) : undefined;
}

/**
 * Running or ended: tools are locked, a valve counts down from the frame step
 * and is drawn open from its open step on (D-06).
 */
function runStyle(
  placement: PuzzlePlacement,
  stepIndex: number,
): Pick<OverlayTool, "style" | "maybeLabel"> {
  if (placement.kind !== "valve") {
    return { style: "locked", maybeLabel: undefined };
  }

  const openStep = openStepFor(placement.openTick);
  return stepIndex >= openStep
    ? { style: "open", maybeLabel: undefined }
    : { style: "locked", maybeLabel: valveCountdownText(openStep, stepIndex) };
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
  if (paint.keyline) {
    strokePass(context, KEYLINE_STROKE, KEYLINE_WIDTH_PX);
  }
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

/** Centroid of the projected quad corners, in canvas pixels. */
function projectedCentroid(camera: Camera, quad: Quad): { x: number; y: number } {
  const points = quad.map((corner) => projectPoint(camera, corner));
  const sum = points.reduce((total, point) => ({ x: total.x + point.x, y: total.y + point.y }), {
    x: 0,
    y: 0,
  });
  return { x: sum.x / points.length, y: sum.y / points.length };
}

/**
 * Draws valve time labels at their quad centroids, above the tools (D-06).
 * Text size is fixed CSS pixels; restores every canvas state it changes.
 */
export function drawToolLabels(
  context: CanvasRenderingContext2D,
  camera: Camera,
  tools: readonly OverlayTool[],
): void {
  if (tools.every((tool) => tool.maybeLabel === undefined)) {
    return;
  }

  context.save();
  try {
    context.font = LABEL_FONT;
    context.textAlign = "center";
    context.textBaseline = "middle";
    context.lineJoin = "round";
    context.strokeStyle = KEYLINE_STROKE;
    context.lineWidth = LABEL_KEYLINE_WIDTH_PX;
    context.fillStyle = LABEL_FILL;
    for (const { quad, maybeLabel } of tools) {
      if (maybeLabel === undefined) {
        continue;
      }
      const center = projectedCentroid(camera, quad);
      context.strokeText(maybeLabel, center.x, center.y);
      context.fillText(maybeLabel, center.x, center.y);
    }
  } finally {
    context.restore();
  }
}

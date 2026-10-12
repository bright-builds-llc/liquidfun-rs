/**
 * Glass-cup canvas overlay for puzzle levels (D-12, D-13).
 *
 * The pure helpers turn a level's cup interior and goal counts into a
 * validated `GlassCup`; `drawGlassCup` paints it in world space above the
 * particle layer. Line widths and pixel offsets are fixed CSS pixels, so the
 * overlay keeps the same weight at every camera scale.
 */

import { projectPoint, type Camera, type Point, type WorldBounds } from "./camera";

/** One goal tick, as a fraction of the cup's full capacity (structurally equal to puzzle/format.ts GoalTick). */
export type GlassTick = { readonly fraction: number; readonly label: string };

/** Validated glass geometry: the cup interior, the target fraction and its ticks. */
export type GlassCup = {
  readonly interior: WorldBounds;
  readonly targetFraction: number;
  readonly ticks: readonly GlassTick[];
};

const INVALID_GLASS_MESSAGE = "Invalid glass cup";
const TAU = 2 * Math.PI;

const TINT_FILL = "rgba(186, 230, 253, 0.08)";
const GLASS_STROKE = "rgba(224, 242, 254, 0.75)";
const HIGHLIGHT_STROKE = "rgba(255, 255, 255, 0.22)";
const SHADOW_STROKE = "rgba(7, 16, 24, 0.65)";
const TARGET_AMBER = "#E6C35C";
const STAR_PREFIX = "★";

const TICK_LENGTH_PX = 8;
const STAR_GAP_PX = 6;
const STAR_OUTER_RADIUS_PX = 4;
const STAR_INNER_RADIUS_PX = 1.8;
const STAR_VERTEX_COUNT = 10;

function isUnitFraction(value: number): boolean {
  return Number.isFinite(value) && value >= 0 && value <= 1;
}

function isValidInterior(interior: WorldBounds): boolean {
  const { minX, minY, maxX, maxY } = interior;
  return [minX, minY, maxX, maxY].every(Number.isFinite) && minX < maxX && minY < maxY;
}

/** Builds a validated glass cup; throws `Invalid glass cup` on non-finite or inconsistent input. */
export function glassCup(
  interior: WorldBounds,
  targetCount: number,
  fullCount: number,
  ticks: readonly GlassTick[],
): GlassCup {
  const validCounts =
    Number.isFinite(fullCount) &&
    fullCount > 0 &&
    Number.isFinite(targetCount) &&
    targetCount >= 0 &&
    targetCount <= fullCount;
  const validTicks = ticks.every((tick) => isUnitFraction(tick.fraction));
  if (!isValidInterior(interior) || !validCounts || !validTicks) {
    throw new Error(INVALID_GLASS_MESSAGE);
  }

  return { interior, targetFraction: targetCount / fullCount, ticks };
}

/** Linear interior height for a capacity fraction (visual aid, not a physics claim; D-12). */
export function glassHeightAt(glass: GlassCup, fraction: number): number {
  const { minY, maxY } = glass.interior;
  return minY + fraction * (maxY - minY);
}

/** World y of the etched target line. */
export function targetLineY(glass: GlassCup): number {
  return glassHeightAt(glass, glass.targetFraction);
}

function strokeSegment(context: CanvasRenderingContext2D, start: Point, end: Point): void {
  context.beginPath();
  context.moveTo(start.x, start.y);
  context.lineTo(end.x, end.y);
  context.stroke();
}

function fillStar(context: CanvasRenderingContext2D, center: Point): void {
  context.beginPath();
  for (let index = 0; index < STAR_VERTEX_COUNT; index++) {
    const angle = (index * TAU) / STAR_VERTEX_COUNT - TAU / 4;
    const radius = index % 2 === 0 ? STAR_OUTER_RADIUS_PX : STAR_INNER_RADIUS_PX;
    const x = center.x + radius * Math.cos(angle);
    const y = center.y + radius * Math.sin(angle);
    if (index === 0) context.moveTo(x, y);
    else context.lineTo(x, y);
  }
  context.closePath();
  context.fill();
}

function drawTint(context: CanvasRenderingContext2D, camera: Camera, interior: WorldBounds): void {
  const topLeft = projectPoint(camera, { x: interior.minX, y: interior.maxY });
  const bottomRight = projectPoint(camera, { x: interior.maxX, y: interior.minY });
  context.fillStyle = TINT_FILL;
  context.fillRect(topLeft.x, topLeft.y, bottomRight.x - topLeft.x, bottomRight.y - topLeft.y);
}

function drawEdges(context: CanvasRenderingContext2D, camera: Camera, interior: WorldBounds): void {
  const corners = [
    { x: interior.minX, y: interior.maxY },
    { x: interior.minX, y: interior.minY },
    { x: interior.maxX, y: interior.minY },
    { x: interior.maxX, y: interior.maxY },
  ].map((corner) => projectPoint(camera, corner));
  context.strokeStyle = GLASS_STROKE;
  context.lineWidth = 1.5;
  context.lineJoin = "round";
  context.setLineDash([]);
  context.beginPath();
  corners.forEach((corner, index) => {
    if (index === 0) context.moveTo(corner.x, corner.y);
    else context.lineTo(corner.x, corner.y);
  });
  context.stroke();
}

function drawHighlight(context: CanvasRenderingContext2D, camera: Camera, interior: WorldBounds): void {
  const width = interior.maxX - interior.minX;
  const height = interior.maxY - interior.minY;
  const x = interior.minX + 0.12 * width;
  context.strokeStyle = HIGHLIGHT_STROKE;
  context.lineWidth = 3;
  context.lineCap = "round";
  strokeSegment(
    context,
    projectPoint(camera, { x, y: interior.minY + 0.85 * height }),
    projectPoint(camera, { x, y: interior.minY + 0.15 * height }),
  );
}

function drawTargetLine(context: CanvasRenderingContext2D, camera: Camera, glass: GlassCup): void {
  const y = targetLineY(glass);
  const start = projectPoint(camera, { x: glass.interior.minX, y });
  const end = projectPoint(camera, { x: glass.interior.maxX, y });

  context.strokeStyle = SHADOW_STROKE;
  context.lineWidth = 1;
  strokeSegment(context, { x: start.x, y: start.y + 1 }, { x: end.x, y: end.y + 1 });

  context.strokeStyle = TARGET_AMBER;
  context.globalAlpha = 0.95;
  context.lineWidth = 1.5;
  context.setLineDash([6, 4]);
  context.lineCap = "butt";
  strokeSegment(context, start, end);
}

function drawTicks(context: CanvasRenderingContext2D, camera: Camera, glass: GlassCup): void {
  context.setLineDash([]);
  context.globalAlpha = 1;
  context.lineWidth = 2;
  for (const tick of glass.ticks) {
    const inner = projectPoint(camera, {
      x: glass.interior.minX,
      y: glassHeightAt(glass, tick.fraction),
    });
    const outer = { x: inner.x - TICK_LENGTH_PX, y: inner.y };
    const starred = tick.label.startsWith(STAR_PREFIX);
    context.strokeStyle = starred ? TARGET_AMBER : GLASS_STROKE;
    strokeSegment(context, inner, outer);
    if (!starred) continue;
    context.fillStyle = TARGET_AMBER;
    fillStar(context, { x: outer.x - STAR_GAP_PX, y: outer.y });
  }
}

/** Draws the glass overlay above the particle layer; restores every canvas state it changes. */
export function drawGlassCup(
  context: CanvasRenderingContext2D,
  camera: Camera,
  glass: GlassCup,
): void {
  context.save();
  try {
    drawTint(context, camera, glass.interior);
    drawEdges(context, camera, glass.interior);
    drawHighlight(context, camera, glass.interior);
    drawTargetLine(context, camera, glass);
    drawTicks(context, camera, glass);
  } finally {
    context.restore();
  }
}

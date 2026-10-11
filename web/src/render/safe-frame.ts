import { VIEWPORT_INSET, type ViewportSize } from "./camera";

/** CSS-pixel rectangle in canvas coordinates. */
export interface ScreenRect {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
}

/** Result of fitting a world aspect around HUD obstacles. */
export interface SafeFrame {
  readonly frame: ScreenRect;
  readonly usedFallback: boolean;
}

/** Clearance kept between the fitted frame and every HUD obstacle. */
export const SAFE_FRAME_GAP_PX = 8;

/** Below this fraction of the unobstructed scale, obstacles are ignored. */
export const SAFE_FRAME_FALLBACK_RATIO = 0.5;

/** Candidates scoring within this fraction of the best are treated as ties. */
export const SAFE_FRAME_SCORE_TOLERANCE = 0.005;

/** Edge tolerance used when comparing measured rectangles. */
const SAME_RECT_TOLERANCE_PX = 0.5;

type Candidate = {
  readonly rect: ScreenRect;
  readonly score: number;
};

/**
 * Finds the obstacle-free rectangle that allows the largest world scale.
 *
 * Near-best ties prefer the rectangle centred nearest the viewport centre,
 * then the lowest top, then the lowest left, so obstacle order cannot change
 * the result. When nothing fits, or the best fit is below half the
 * unobstructed scale, the full viewport is returned with `usedFallback`.
 */
export function safeFrame(
  viewport: ViewportSize,
  obstacles: readonly ScreenRect[],
  worldAspect: number,
): SafeFrame {
  const full: ScreenRect = {
    left: 0,
    top: 0,
    right: viewport.width,
    bottom: viewport.height,
  };
  const inflated = inflatedObstacles(full, obstacles);
  const candidates = obstacleFreeCandidates(full, inflated, worldAspect);
  const bestScore = candidates.reduce(
    (best, candidate) => Math.max(best, candidate.score),
    0,
  );
  const fullScore = frameScore(full, worldAspect);
  if (
    candidates.length === 0 ||
    bestScore < SAFE_FRAME_FALLBACK_RATIO * fullScore
  ) {
    return { frame: full, usedFallback: true };
  }

  const threshold = bestScore * (1 - SAFE_FRAME_SCORE_TOLERANCE);
  const ties = candidates.filter((candidate) => candidate.score >= threshold);
  const centreX = viewport.width / 2;
  const centreY = viewport.height / 2;
  const [chosen] = [...ties].sort((a, b) =>
    compareCandidates(a.rect, b.rect, centreX, centreY),
  );
  if (chosen === undefined) {
    return { frame: full, usedFallback: true };
  }

  return { frame: chosen.rect, usedFallback: false };
}

/** Converts a client rectangle into the canvas's CSS-pixel coordinates. */
export function screenRectInCanvas(
  rect: ScreenRect,
  canvasLeft: number,
  canvasTop: number,
): ScreenRect {
  return {
    left: rect.left - canvasLeft,
    top: rect.top - canvasTop,
    right: rect.right - canvasLeft,
    bottom: rect.bottom - canvasTop,
  };
}

/** Order-sensitive equality with half-pixel tolerance on every edge. */
export function sameScreenRects(
  a: readonly ScreenRect[],
  b: readonly ScreenRect[],
): boolean {
  if (a.length !== b.length) {
    return false;
  }

  return a.every((rect, index) => {
    const other = b[index];
    return other !== undefined && sameScreenRect(rect, other);
  });
}

function sameScreenRect(a: ScreenRect, b: ScreenRect): boolean {
  return (
    Math.abs(a.left - b.left) <= SAME_RECT_TOLERANCE_PX &&
    Math.abs(a.top - b.top) <= SAME_RECT_TOLERANCE_PX &&
    Math.abs(a.right - b.right) <= SAME_RECT_TOLERANCE_PX &&
    Math.abs(a.bottom - b.bottom) <= SAME_RECT_TOLERANCE_PX
  );
}

function inflatedObstacles(
  full: ScreenRect,
  obstacles: readonly ScreenRect[],
): ScreenRect[] {
  return obstacles
    .map((obstacle) => ({
      left: Math.max(full.left, obstacle.left - SAFE_FRAME_GAP_PX),
      top: Math.max(full.top, obstacle.top - SAFE_FRAME_GAP_PX),
      right: Math.min(full.right, obstacle.right + SAFE_FRAME_GAP_PX),
      bottom: Math.min(full.bottom, obstacle.bottom + SAFE_FRAME_GAP_PX),
    }))
    .filter((rect) => rect.right > rect.left && rect.bottom > rect.top);
}

function sortedEdges(values: readonly number[]): number[] {
  return [...new Set(values)].sort((a, b) => a - b);
}

function obstacleFreeCandidates(
  full: ScreenRect,
  obstacles: readonly ScreenRect[],
  worldAspect: number,
): Candidate[] {
  const xs = sortedEdges([
    full.left,
    full.right,
    ...obstacles.flatMap((obstacle) => [obstacle.left, obstacle.right]),
  ]);
  const ys = sortedEdges([
    full.top,
    full.bottom,
    ...obstacles.flatMap((obstacle) => [obstacle.top, obstacle.bottom]),
  ]);
  const candidates: Candidate[] = [];
  for (const [i, left] of xs.entries()) {
    for (const right of xs.slice(i + 1)) {
      for (const [j, top] of ys.entries()) {
        for (const bottom of ys.slice(j + 1)) {
          const rect = { left, top, right, bottom };
          const score = frameScore(rect, worldAspect);
          if (score <= 0 || obstacles.some((o) => overlaps(rect, o))) {
            continue;
          }
          candidates.push({ rect, score });
        }
      }
    }
  }

  return candidates;
}

/** World scale (pixels per unit of world height) a rectangle allows. */
function frameScore(rect: ScreenRect, worldAspect: number): number {
  const innerWidth = rect.right - rect.left - 2 * VIEWPORT_INSET;
  const innerHeight = rect.bottom - rect.top - 2 * VIEWPORT_INSET;
  return Math.min(innerWidth / worldAspect, innerHeight);
}

function overlaps(a: ScreenRect, b: ScreenRect): boolean {
  return (
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
  );
}

function compareCandidates(
  a: ScreenRect,
  b: ScreenRect,
  centreX: number,
  centreY: number,
): number {
  const distance =
    centreDistanceSquared(a, centreX, centreY) -
    centreDistanceSquared(b, centreX, centreY);
  if (distance !== 0) {
    return distance;
  }
  if (a.top !== b.top) {
    return a.top - b.top;
  }

  return a.left - b.left;
}

function centreDistanceSquared(
  rect: ScreenRect,
  centreX: number,
  centreY: number,
): number {
  const dx = (rect.left + rect.right) / 2 - centreX;
  const dy = (rect.top + rect.bottom) / 2 - centreY;
  return dx * dx + dy * dy;
}

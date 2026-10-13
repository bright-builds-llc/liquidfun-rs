import type { Point } from "../render/camera";
import type { Quad } from "./placement-status";

/** Pointer slop around a tool outline in CSS pixels (D-18). */
export const HIT_SLOP_CSS_PX = 22;

/** A placed tool and its latest outline; tools without an outline cannot be hit. */
export type HitCandidate = { readonly id: number; readonly maybeQuad: Quad | undefined };

function cross(origin: Point, a: Point, b: Point): number {
  return (a.x - origin.x) * (b.y - origin.y) - (a.y - origin.y) * (b.x - origin.x);
}

function distanceToSegment(point: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lengthSquared = dx * dx + dy * dy;
  const t =
    lengthSquared > 0
      ? Math.min(1, Math.max(0, ((point.x - a.x) * dx + (point.y - a.y) * dy) / lengthSquared))
      : 0;
  return Math.hypot(point.x - (a.x + t * dx), point.y - (a.y + t * dy));
}

function quadEdges(quad: Quad): readonly (readonly [Point, Point])[] {
  return [
    [quad[0], quad[1]],
    [quad[1], quad[2]],
    [quad[2], quad[3]],
    [quad[3], quad[0]],
  ];
}

/** 0 inside a convex quad of either winding; otherwise the nearest edge distance in meters. */
export function distanceToQuad(point: Point, quad: Quad): number {
  const edges = quadEdges(quad);
  const sides = edges.map(([a, b]) => cross(a, b, point));
  if (sides.every((side) => side >= 0) || sides.every((side) => side <= 0)) {
    return 0;
  }

  return Math.min(...edges.map(([a, b]) => distanceToSegment(point, a, b)));
}

type Hit = { readonly id: number; readonly distance: number };

function preferHit(best: Hit, next: Hit, maybeSelectedId: number | undefined): Hit {
  if (next.distance !== best.distance) {
    return next.distance < best.distance ? next : best;
  }
  if (best.id === maybeSelectedId) {
    return best;
  }
  if (next.id === maybeSelectedId) {
    return next;
  }

  return next.id > best.id ? next : best;
}

/** Nearest tool within `HIT_SLOP_CSS_PX / pixelsPerMeter` meters; ties → selected, then highest id (D-18). */
export function pickTool(
  point: Point,
  candidates: readonly HitCandidate[],
  pixelsPerMeter: number,
  maybeSelectedId: number | undefined,
): number | undefined {
  if (!Number.isFinite(pixelsPerMeter) || pixelsPerMeter <= 0) {
    return undefined;
  }

  const slopMeters = HIT_SLOP_CSS_PX / pixelsPerMeter;
  let maybeBest: Hit | undefined;
  for (const candidate of candidates) {
    if (candidate.maybeQuad === undefined) {
      continue;
    }
    const hit = { id: candidate.id, distance: distanceToQuad(point, candidate.maybeQuad) };
    if (hit.distance > slopMeters) {
      continue;
    }
    maybeBest = maybeBest === undefined ? hit : preferHit(maybeBest, hit, maybeSelectedId);
  }

  return maybeBest?.id;
}

/**
 * Puzzle camera frames, in meters.
 *
 * Puzzle levels are not scenes (D-03), so their frames live here rather than
 * in `PORTRAIT_VIEW_BOUNDS`. Each level has one landscape and one portrait
 * rectangle, picked by orientation only, so the camera never refits while a
 * level moves between building, running and ended (D-20). Both rectangles
 * contain every wall, the liquid spawn, the cup and the build area. Neither
 * pads for HUD chrome: the safe frame fits the world around HUD obstacles.
 *
 * The portrait frames keep every wall. first-pour's 12 m width limits its
 * phone height fill (D-25); Phase 40 levels are authored about 8 m wide and
 * 10-11 m tall so their portrait frames fill most of a phone (LEVEL-04).
 */

import type { WorldBounds } from "../render/camera";
import { worldBoundsForViewport } from "./portrait-bounds";
import type { PuzzleId } from "./puzzles";
import type { SceneId } from "./scenes";

/**
 * Landscape frames: every wall, the spawn, the cup and the build area, plus a
 * 0.2 m stroke margin (D-20).
 */
export const PUZZLE_VIEW_BOUNDS: Readonly<Record<PuzzleId, WorldBounds>> = {
  "first-pour": { minX: -6.2, minY: -0.7, maxX: 6.2, maxY: 9.2 },
  "thick-syrup": { minX: -4.2, minY: -0.7, maxX: 4.2, maxY: 10.2 },
  "jelly-pad": { minX: -4.2, minY: -0.7, maxX: 4.2, maxY: 11.2 },
  "timed-gate": { minX: -4.7, minY: -0.7, maxX: 4.7, maxY: 10.7 },
};

/**
 * Portrait phone frames with strict wall containment and a 0.1 m stroke
 * margin (D-25, D-22). At 390x844, first-pour fills the full width and about
 * 34% of the height; thick-syrup about 60%, jelly-pad about 66% and
 * timed-gate about 51%. No HUD
 * padding; the safe frame handles chrome.
 */
export const PUZZLE_PORTRAIT_BOUNDS: Readonly<Record<PuzzleId, WorldBounds>> = {
  "first-pour": { minX: -6.1, minY: -0.6, maxX: 6.1, maxY: 9.1 },
  "thick-syrup": { minX: -4.1, minY: -0.6, maxX: 4.1, maxY: 10.1 },
  "jelly-pad": { minX: -4.1, minY: -0.6, maxX: 4.1, maxY: 11.1 },
  "timed-gate": { minX: -4.6, minY: -0.6, maxX: 4.6, maxY: 10.6 },
};

/**
 * World rectangle for one puzzle at the canvas's current aspect, using the
 * same portrait rule as `worldBoundsForViewport`.
 */
export function worldBoundsForPuzzle(
  id: PuzzleId,
  viewportWidth: number,
  viewportHeight: number,
): WorldBounds {
  if (!(viewportHeight > viewportWidth) || viewportWidth <= 0) {
    return PUZZLE_VIEW_BOUNDS[id];
  }

  return PUZZLE_PORTRAIT_BOUNDS[id];
}

/**
 * World rectangle for any playable target. Scenes delegate to
 * `worldBoundsForViewport` unchanged; puzzles use this registry by
 * orientation (D-20).
 */
export function worldBoundsForTarget(
  target:
    | { readonly kind: "scene"; readonly id: SceneId }
    | { readonly kind: "puzzle"; readonly id: PuzzleId },
  viewportWidth: number,
  viewportHeight: number,
): WorldBounds {
  if (target.kind === "scene") {
    return worldBoundsForViewport(target.id, viewportWidth, viewportHeight);
  }

  return worldBoundsForPuzzle(target.id, viewportWidth, viewportHeight);
}

import {
  WAVE_MACHINE_VIEW_BOUNDS,
  worldBoundsForScene,
  type SceneId,
} from "./scenes";
import type { WorldBounds } from "../render/camera";

/**
 * Portrait phone frames, in meters.
 *
 * A tall iPhone canvas contain-fits whatever rectangle it is given. The shared
 * 12 m by 9 m frame therefore becomes a short band with a large empty margin.
 * These rectangles sit on each scene's action so that band fills more of the
 * screen. Wide scenes keep enough width to stay recognizable, which still
 * leaves some vertical margin. Landscape viewports keep `worldBoundsForScene`.
 */

/**
 * Inner basin plus the falling ball, with headroom so the ball starts
 * below the phone title.
 */
const FALLING_BALL_BASIN = {
  minX: -2.45,
  minY: -0.45,
  maxX: 2.45,
  maxY: 10.1,
} as const;

/** Soup pool, wide enough for the floating bits and tight above the water. */
const SOUP_BASIN = {
  minX: -1.7,
  minY: -0.3,
  maxX: 1.7,
  maxY: 2.5,
} as const;

export const PORTRAIT_VIEW_BOUNDS: Record<SceneId, WorldBounds> = {
  "wave-machine": WAVE_MACHINE_VIEW_BOUNDS,
  // Leading water and the obstacle. The far left wall stays off-screen.
  "dam-break": { minX: -2.2, minY: -0.25, maxX: 3.4, maxY: 6.9 },
  fountain: { minX: -1.9, minY: -0.45, maxX: 1.9, maxY: 5.4 },
  "float-or-sink": { minX: -2.2, minY: -0.55, maxX: 2.2, maxY: 6.9 },
  "color-mixer": { minX: -2.15, minY: -0.4, maxX: 2.15, maxY: 4.1 },
  "jelly-drop": { minX: -3.85, minY: 0.35, maxX: 3.85, maxY: 7.7 },
  "water-wheel": { minX: -4.15, minY: 0.05, maxX: 1.9, maxY: 5.55 },
  particles: FALLING_BALL_BASIN,
  "liquid-timer": { minX: -2.15, minY: -0.12, maxX: 2.15, maxY: 4.35 },
  "surface-tension": FALLING_BALL_BASIN,
  "elastic-particles": FALLING_BALL_BASIN,
  "rigid-particles": FALLING_BALL_BASIN,
  soup: SOUP_BASIN,
  "soup-stirrer": SOUP_BASIN,
  impulse: { minX: -2.25, minY: -0.25, maxX: 2.25, maxY: 5.15 },
  "theo-jansen": { minX: -6.4, minY: -0.35, maxX: 6.4, maxY: 13.1 },
  "liquid-tumbler": { minX: -0.042, minY: -0.006, maxX: 0.042, maxY: 0.128 },
};

/** World rectangle fitted for one scene at the canvas's current aspect. */
export function worldBoundsForViewport(
  id: SceneId,
  viewportWidth: number,
  viewportHeight: number,
): WorldBounds {
  if (!(viewportHeight > viewportWidth) || viewportWidth <= 0) {
    return worldBoundsForScene(id);
  }

  return PORTRAIT_VIEW_BOUNDS[id];
}

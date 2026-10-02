import {
  THEO_JANSEN_VIEW_BOUNDS,
  WASHING_MACHINE_VIEW_BOUNDS,
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
 * These rectangles sit on each scene's action, including its walls, so that
 * band fills more of the screen without clipping the container. Wide scenes
 * keep enough width to stay recognizable, which still leaves some vertical
 * margin. Landscape viewports keep `worldBoundsForScene`.
 */

/**
 * Inner faces of the shared tall basin (x = ±5.5, y = 0..8) plus a margin
 * so the wall stroke stays on the canvas.
 */
const TALL_BASIN = {
  minX: -5.7,
  minY: -0.2,
  maxX: 5.7,
  maxY: 8.2,
} as const;

/**
 * Floor out to x = ±4, side walls, and the falling ball at y = 8.5, with
 * margin so those walls stay on the canvas.
 */
const WALLED_FALLING_BALL = {
  minX: -4.2,
  minY: -0.2,
  maxX: 4.2,
  maxY: 8.8,
} as const;

/**
 * Tesla valve portrait frame.
 *
 * The landscape frame is the valve itself. On a phone the flow sliders and
 * transport sit on the bottom of the canvas, so this rectangle adds empty
 * space below the drain and a little above the inlet. The valve then stays
 * clear of that chrome while the frame still fills the width.
 */
const TESLA_VALVE_PORTRAIT = {
  minX: -0.9,
  minY: -1.04,
  maxX: 0.9,
  maxY: 3.25,
} as const;

/** Soup floor and slanted walls out to (±4, 3), plus a small margin. */
const SOUP_BASIN = {
  minX: -4.2,
  minY: -0.2,
  maxX: 4.2,
  maxY: 3.3,
} as const;

export const PORTRAIT_VIEW_BOUNDS: Record<SceneId, WorldBounds> = {
  "wave-machine": WAVE_MACHINE_VIEW_BOUNDS,
  "dam-break": TALL_BASIN,
  fountain: TALL_BASIN,
  "float-or-sink": TALL_BASIN,
  "color-mixer": TALL_BASIN,
  "jelly-drop": { minX: -3.85, minY: 0.35, maxX: 3.85, maxY: 7.7 },
  "water-wheel": TALL_BASIN,
  particles: WALLED_FALLING_BALL,
  "liquid-timer": { minX: -2.15, minY: -0.12, maxX: 2.15, maxY: 4.35 },
  "surface-tension": WALLED_FALLING_BALL,
  "elastic-particles": WALLED_FALLING_BALL,
  "rigid-particles": WALLED_FALLING_BALL,
  soup: SOUP_BASIN,
  "soup-stirrer": SOUP_BASIN,
  impulse: { minX: -2.25, minY: -0.25, maxX: 2.25, maxY: 5.15 },
  "theo-jansen": THEO_JANSEN_VIEW_BOUNDS,
  "liquid-tumbler": { minX: -0.042, minY: -0.006, maxX: 0.042, maxY: 0.128 },
  "drawing-particles": { minX: -4.2, minY: -2.2, maxX: 4.2, maxY: 6.2 },
  sparky: { minX: -22, minY: -1, maxX: 22, maxY: 42 },
  "hydraulic-fountain": { minX: -2.58, minY: -0.3, maxX: 2.58, maxY: 3.2 },
  "wave-tank": { minX: -0.12, minY: -0.16, maxX: 4.16, maxY: 1.04 },
  "liquid-bubbler": { minX: -0.71, minY: -0.16, maxX: 1.12, maxY: 2.48 },
  "stacked-drip": { minX: -0.86, minY: -0.16, maxX: 1.68, maxY: 3.28 },
  "washing-machine": WASHING_MACHINE_VIEW_BOUNDS,
  "tesla-valve": TESLA_VALVE_PORTRAIT,
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

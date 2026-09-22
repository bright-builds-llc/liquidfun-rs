import type { TiltDebug, TiltGravity } from "../input/tilt-gravity";

const BOX = 48;
const CENTER = BOX / 2;
const SHAFT = 14;
const HEAD = 7;

export type ArrowPoint = readonly [number, number];

export type GravityArrowGeometry = {
  readonly x1: number;
  readonly y1: number;
  readonly x2: number;
  readonly y2: number;
  readonly head: readonly [ArrowPoint, ArrowPoint, ArrowPoint];
};

/**
 * Screen-space arrow for a world gravity vector.
 *
 * World +y is up and SVG +y is down, so the arrow flips the world y axis.
 */
export function gravityArrowGeometry(
  gravity: TiltGravity,
): GravityArrowGeometry | undefined {
  const screenX = gravity.x;
  const screenY = -gravity.y;
  const length = Math.hypot(screenX, screenY);
  if (!Number.isFinite(length) || length < 1e-3) {
    return undefined;
  }

  const x = screenX / length;
  const y = screenY / length;
  const x2 = CENTER + x * SHAFT;
  const y2 = CENTER + y * SHAFT;
  const baseX = CENTER + x * (SHAFT - HEAD);
  const baseY = CENTER + y * (SHAFT - HEAD);
  const side = HEAD * 0.65;
  const perpendicularX = -y;
  const perpendicularY = x;
  return {
    x1: CENTER - x * (SHAFT - HEAD),
    y1: CENTER - y * (SHAFT - HEAD),
    x2,
    y2,
    head: [
      [x2, y2],
      [baseX + perpendicularX * side, baseY + perpendicularY * side],
      [baseX - perpendicularX * side, baseY - perpendicularY * side],
    ],
  };
}

/**
 * Returns an arrow while accelerometer gravity is on and a live sample exists.
 *
 * The canvas arrow is independent of the debug-info readout.
 */
export function maybeGravityArrow(
  tiltGravityEnabled: boolean,
  tiltDebug: TiltDebug,
): GravityArrowGeometry | undefined {
  if (!tiltGravityEnabled || tiltDebug.kind !== "live") {
    return undefined;
  }
  return gravityArrowGeometry(tiltDebug.gravity);
}

export function formatArrowPoints(
  points: readonly ArrowPoint[],
): string {
  return points.map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`).join(" ");
}

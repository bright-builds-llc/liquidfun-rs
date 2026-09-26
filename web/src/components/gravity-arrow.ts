import {
  STANDARD_GRAVITY,
  type TiltDebug,
  type TiltGravity,
} from "../input/tilt-gravity";

/** Applied magnitude, in m/s², at which the arrow reaches its full length by default. */
export const FULL_GRAVITY = STANDARD_GRAVITY;

const BOX = 48;
const CENTER = BOX / 2;
const SHAFT = 14;
const HEAD = 7;
const STROKE = 2.5;
const MIN_DRAWABLE_MAGNITUDE = 1e-3;

export type ArrowPoint = readonly [number, number];

export type GravityArrowGeometry = {
  readonly x1: number;
  readonly y1: number;
  readonly x2: number;
  readonly y2: number;
  readonly strokeWidth: number;
  readonly head: readonly [ArrowPoint, ArrowPoint, ArrowPoint];
};

/**
 * Screen-space arrow for a world gravity vector.
 *
 * World +y is up and SVG +y is down, so the arrow flips the world y axis.
 * Length scales with the applied magnitude and reaches its full size at
 * `maybeFullLengthMagnitude`. That reference is one standard g, or the gravity
 * slider magnitude when accelerometer gravity is scaled to the slider. A phone
 * lying flat draws only the leftover accelerometer noise.
 */
export function gravityArrowGeometry(
  gravity: TiltGravity,
  maybeFullLengthMagnitude?: number,
): GravityArrowGeometry | undefined {
  const screenX = gravity.x;
  const screenY = -gravity.y;
  const magnitude = Math.hypot(screenX, screenY);
  const fullLength = arrowFullLength(maybeFullLengthMagnitude);
  if (!Number.isFinite(magnitude) || magnitude < MIN_DRAWABLE_MAGNITUDE) {
    return undefined;
  }

  const strength = Math.min(magnitude / fullLength, 1);
  const x = screenX / magnitude;
  const y = screenY / magnitude;
  const shaft = SHAFT * strength;
  const head = HEAD * strength;
  const x2 = CENTER + x * shaft;
  const y2 = CENTER + y * shaft;
  const baseX = CENTER + x * (shaft - head);
  const baseY = CENTER + y * (shaft - head);
  const side = head * 0.65;
  const perpendicularX = -y;
  const perpendicularY = x;
  return {
    x1: CENTER - x * (shaft - head),
    y1: CENTER - y * (shaft - head),
    x2,
    y2,
    strokeWidth: STROKE * strength,
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
  return gravityArrowGeometry(tiltDebug.gravity, tiltDebug.fullLengthMagnitude);
}

function arrowFullLength(maybeFullLengthMagnitude: number | undefined): number {
  if (
    maybeFullLengthMagnitude === undefined ||
    !Number.isFinite(maybeFullLengthMagnitude) ||
    maybeFullLengthMagnitude <= 0
  ) {
    return FULL_GRAVITY;
  }
  return maybeFullLengthMagnitude;
}

export function formatArrowPoints(
  points: readonly ArrowPoint[],
): string {
  return points.map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`).join(" ");
}

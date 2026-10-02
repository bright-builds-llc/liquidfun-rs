import type { SceneControl } from "./scenes";

/** Stopped. The source pours nothing. */
export const FLOW_RATE_MIN = 0;
/**
 * Fast pour, in particles per second.
 *
 * The inlet distributes each frame's burst across distinct grid positions.
 * The maximum creates 48 particles per frame at 60 Hz.
 */
export const FLOW_RATE_MAX = 2880;
export const FLOW_RATE_STEP = 30;
/** A steady stream that still leaves room in the valve. */
export const FLOW_RATE_DEFAULT = 1440;
export const FLOW_RATE_TICKS = [
  FLOW_RATE_MIN,
  FLOW_RATE_DEFAULT,
  FLOW_RATE_MAX,
] as const;

/** Live HUD slider. Changing the pour does not rebuild the valve. */
export const FLOW_RATE_CONTROL: SceneControl = {
  id: "flow-rate",
  label: "Flow rate",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: FLOW_RATE_MIN,
  max: FLOW_RATE_MAX,
  step: FLOW_RATE_STEP,
  defaultValue: FLOW_RATE_DEFAULT,
  unit: "particles/s",
  scale: "linear",
  ticks: FLOW_RATE_TICKS,
};

/** Reverse. The valve is flipped so the loops redirect the incoming stream. */
export const FLOW_DIRECTION_MIN = -1;
/** Forward. Water follows the central passage between alternating loops. */
export const FLOW_DIRECTION_MAX = 1;
export const FLOW_DIRECTION_STEP = 2;
export const FLOW_DIRECTION_DEFAULT = FLOW_DIRECTION_MAX;
export const FLOW_DIRECTION_TICKS = [
  FLOW_DIRECTION_MIN,
  FLOW_DIRECTION_MAX,
] as const;

/** Live HUD slider. Flipping direction keeps the chosen flow rate. */
export const FLOW_DIRECTION_CONTROL: SceneControl = {
  id: "flow-direction",
  label: "Flow direction",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: FLOW_DIRECTION_MIN,
  max: FLOW_DIRECTION_MAX,
  step: FLOW_DIRECTION_STEP,
  defaultValue: FLOW_DIRECTION_DEFAULT,
  unit: "direction",
  scale: "linear",
  ticks: FLOW_DIRECTION_TICKS,
};

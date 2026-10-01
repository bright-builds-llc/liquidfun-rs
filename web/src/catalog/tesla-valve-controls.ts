import type { SceneControl } from "./scenes";

/** Stopped. The source pours nothing. */
export const FLOW_RATE_MIN = 0;
/**
 * Fast pour, in particles per second.
 *
 * Three columns can accept one particle each per frame, so this stays under
 * the spawn overlap limit at 60 Hz.
 */
export const FLOW_RATE_MAX = 720;
export const FLOW_RATE_STEP = 30;
/** A steady stream that still leaves room in the valve. */
export const FLOW_RATE_DEFAULT = 180;
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

/** Reverse. The valve is flipped so the ramps hold water back. */
export const FLOW_DIRECTION_MIN = -1;
/** Forward. Water slides off the ramps and falls through. */
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

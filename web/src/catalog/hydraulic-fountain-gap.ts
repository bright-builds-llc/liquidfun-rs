import type { SceneControl } from "./scenes";

/** Narrowest opening that still lets the squeezed water through. */
export const GAP_CENTIMETERS_MIN = 5;
/** Widest opening that still leaves a plate sealed against each wall. */
export const GAP_CENTIMETERS_MAX = 80;
export const GAP_CENTIMETERS_STEP = 1;
/** Authored opening between the two platforms. */
export const GAP_CENTIMETERS_DEFAULT = 20;
export const GAP_CENTIMETERS_TICKS = [
  GAP_CENTIMETERS_MIN,
  GAP_CENTIMETERS_DEFAULT,
  GAP_CENTIMETERS_MAX,
] as const;

/** Live HUD slider. Changing it moves the platforms and does not rebuild the pool. */
export const GAP_CONTROL: SceneControl = {
  id: "gap",
  label: "Gap",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: GAP_CENTIMETERS_MIN,
  max: GAP_CENTIMETERS_MAX,
  step: GAP_CENTIMETERS_STEP,
  defaultValue: GAP_CENTIMETERS_DEFAULT,
  unit: "cm",
  scale: "linear",
  ticks: GAP_CENTIMETERS_TICKS,
};

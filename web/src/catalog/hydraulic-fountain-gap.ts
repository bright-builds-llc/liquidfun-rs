import type { SceneControl } from "./scenes";

/** Narrowest opening on the slider. */
export const GAP_CENTIMETERS_MIN = 1.5;
/** Widest opening. A particle is 7.5 cm across, so this is the setting that still lets water through. */
export const GAP_CENTIMETERS_MAX = 9;
export const GAP_CENTIMETERS_STEP = 0.1;
/** Starts at the wide end so the squeeze still has a hole. */
export const GAP_CENTIMETERS_DEFAULT = 9;
export const GAP_CENTIMETERS_TICKS = [1.5, 5, 9] as const;

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

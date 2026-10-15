import type { SceneControl } from "./scenes";

/** No pull. The puddle lies flat. */
export const MAGNET_STRENGTH_MIN = 0;
/** Tallest crown. Tuned so the pull never flings liquid out of the dish. */
export const MAGNET_STRENGTH_MAX = 100;
export const MAGNET_STRENGTH_STEP = 1;
/** Five to nine spikes over the resting magnet. */
export const MAGNET_STRENGTH_DEFAULT = 65;
export const MAGNET_STRENGTH_TICKS = [
  MAGNET_STRENGTH_MIN,
  MAGNET_STRENGTH_DEFAULT,
  MAGNET_STRENGTH_MAX,
] as const;

/** Live HUD slider with step buttons. Changing it does not rebuild the scene. */
export const MAGNET_STRENGTH_CONTROL: SceneControl = {
  id: "magnet-strength",
  label: "Magnet strength",
  kind: "range",
  recreates: false,
  surface: "hud",
  widget: "spinner",
  min: MAGNET_STRENGTH_MIN,
  max: MAGNET_STRENGTH_MAX,
  step: MAGNET_STRENGTH_STEP,
  defaultValue: MAGNET_STRENGTH_DEFAULT,
  unit: "%",
  scale: "linear",
  ticks: MAGNET_STRENGTH_TICKS,
};

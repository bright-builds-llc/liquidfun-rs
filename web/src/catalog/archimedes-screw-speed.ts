import type { SceneControl } from "./scenes";

/** Stopped. The vanes hold still. */
export const SCREW_SPEED_MIN = 0;
/**
 * Fast lift, in revolutions per minute.
 *
 * One step of vane travel stays under half a particle diameter.
 */
export const SCREW_SPEED_MAX = 60;
export const SCREW_SPEED_STEP = 1;
/** Steady lift. Pockets reach the basin within the README clip. */
export const SCREW_SPEED_DEFAULT = 30;
export const SCREW_SPEED_TICKS = [
  SCREW_SPEED_MIN,
  SCREW_SPEED_DEFAULT,
  SCREW_SPEED_MAX,
] as const;

/** Live HUD slider with step buttons. Changing it does not rebuild the screw. */
export const SCREW_SPEED_CONTROL: SceneControl = {
  id: "screw-speed",
  label: "Screw speed",
  kind: "range",
  recreates: false,
  surface: "hud",
  widget: "spinner",
  min: SCREW_SPEED_MIN,
  max: SCREW_SPEED_MAX,
  step: SCREW_SPEED_STEP,
  defaultValue: SCREW_SPEED_DEFAULT,
  unit: "rpm",
  scale: "linear",
  ticks: SCREW_SPEED_TICKS,
};

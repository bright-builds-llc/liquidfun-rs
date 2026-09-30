import type { SceneControl } from "./scenes";

/** Stopped. The drum holds still. */
export const DRUM_SPEED_MIN = 0;
/**
 * Fast spin, in revolutions per minute.
 *
 * The wall is thick enough that one step at this rate stays inside the drum.
 */
export const DRUM_SPEED_MAX = 48;
export const DRUM_SPEED_STEP = 1;
/** Gentle tumble. The ribs lift the water without pinning it to the wall. */
export const DRUM_SPEED_DEFAULT = 20;
export const DRUM_SPEED_TICKS = [
  DRUM_SPEED_MIN,
  DRUM_SPEED_DEFAULT,
  DRUM_SPEED_MAX,
] as const;

/** Live HUD slider with step buttons. Changing it does not rebuild the drum. */
export const DRUM_SPEED_CONTROL: SceneControl = {
  id: "drum-speed",
  label: "Drum speed",
  kind: "range",
  recreates: false,
  surface: "hud",
  widget: "spinner",
  min: DRUM_SPEED_MIN,
  max: DRUM_SPEED_MAX,
  step: DRUM_SPEED_STEP,
  defaultValue: DRUM_SPEED_DEFAULT,
  unit: "rpm",
  scale: "linear",
  ticks: DRUM_SPEED_TICKS,
};

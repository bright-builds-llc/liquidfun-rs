import type { SceneControl } from "./scenes";

/** Stopped. The tank does not rock until the slider moves. */
export const WAVE_MACHINE_SPEED_MIN = 0;
/**
 * Ten times the pinned Wave Machine amplitude.
 *
 * `1` matches `0.05 * cos(t) * π` from testWaveMachine.js.
 */
export const WAVE_MACHINE_SPEED_MAX = 10;
export const WAVE_MACHINE_SPEED_STEP = 0.1;
export const WAVE_MACHINE_SPEED_DEFAULT = 0;
/** Canonical token for the original pinned motor amplitude. */
export const WAVE_MACHINE_SPEED_ORIGINAL = "1.0";
export const WAVE_MACHINE_SPEED_TICKS = [
  WAVE_MACHINE_SPEED_MIN,
  5,
  WAVE_MACHINE_SPEED_MAX,
] as const;

/** Live HUD slider. Changing it does not rebuild the tank. */
export const WAVE_MACHINE_SPEED_CONTROL: SceneControl = {
  id: "wave-speed",
  label: "Wave speed",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: WAVE_MACHINE_SPEED_MIN,
  max: WAVE_MACHINE_SPEED_MAX,
  step: WAVE_MACHINE_SPEED_STEP,
  defaultValue: WAVE_MACHINE_SPEED_DEFAULT,
  unit: "×",
  scale: "linear",
  ticks: WAVE_MACHINE_SPEED_TICKS,
};

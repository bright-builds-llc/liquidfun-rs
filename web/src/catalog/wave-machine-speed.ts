import type { SceneControl } from "./scenes";

/** Stopped. The tank holds still. */
export const WAVE_MACHINE_SPEED_MIN = 0;
/**
 * Ten times the pinned Wave Machine rocking frequency.
 *
 * `1` matches `0.05 * cos(t) * π` from testWaveMachine.js and keeps that tilt.
 */
export const WAVE_MACHINE_SPEED_MAX = 10;
export const WAVE_MACHINE_SPEED_STEP = 0.1;
export const WAVE_MACHINE_SPEED_DEFAULT = 1;
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

/** Level. The tank does not rock. */
export const WAVE_MACHINE_TILT_MIN = 0;
/**
 * Widest HUD tilt that still fits the Wave Machine frame.
 *
 * `9` is the pinned peak, `0.05 * π` radians.
 */
export const WAVE_MACHINE_TILT_MAX = 30;
export const WAVE_MACHINE_TILT_STEP = 1;
export const WAVE_MACHINE_TILT_DEFAULT = 9;
export const WAVE_MACHINE_TILT_TICKS = [
  WAVE_MACHINE_TILT_MIN,
  WAVE_MACHINE_TILT_DEFAULT,
  WAVE_MACHINE_TILT_MAX,
] as const;

/** Live HUD slider for the peak rocking angle. Changing it does not rebuild the tank. */
export const WAVE_MACHINE_TILT_CONTROL: SceneControl = {
  id: "wave-tilt",
  label: "Wave tilt",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: WAVE_MACHINE_TILT_MIN,
  max: WAVE_MACHINE_TILT_MAX,
  step: WAVE_MACHINE_TILT_STEP,
  defaultValue: WAVE_MACHINE_TILT_DEFAULT,
  unit: "°",
  scale: "linear",
  ticks: WAVE_MACHINE_TILT_TICKS,
};

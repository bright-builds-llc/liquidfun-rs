import type { SceneControl } from "./scenes";

/** Narrowest platform. The fixed floor still reaches the far wall. */
export const WAVE_TANK_WIDTH_MIN = 2;
/** Widest platform that still leaves a fixed channel. */
export const WAVE_TANK_WIDTH_MAX = 12;
export const WAVE_TANK_WIDTH_STEP = 0.5;
/** A few meters of paddle on a 50 m pool. */
export const WAVE_TANK_WIDTH_DEFAULT = 4;
export const WAVE_TANK_WIDTH_TICKS = [
  WAVE_TANK_WIDTH_MIN,
  WAVE_TANK_WIDTH_DEFAULT,
  WAVE_TANK_WIDTH_MAX,
] as const;

/** Rebuilding slider. Changing the width recreates the tank. */
export const WAVE_TANK_WIDTH_CONTROL: SceneControl = {
  id: "platform-width",
  label: "Platform width",
  kind: "range",
  recreates: true,
  surface: "hud",
  min: WAVE_TANK_WIDTH_MIN,
  max: WAVE_TANK_WIDTH_MAX,
  step: WAVE_TANK_WIDTH_STEP,
  defaultValue: WAVE_TANK_WIDTH_DEFAULT,
  unit: "m",
  scale: "linear",
  ticks: WAVE_TANK_WIDTH_TICKS,
};

/** Stopped. The platform holds still. */
export const WAVE_TANK_SPEED_MIN = 0;
/** Four times the original rise-and-fall rate. */
export const WAVE_TANK_SPEED_MAX = 4;
export const WAVE_TANK_SPEED_STEP = 0.1;
/** Six tenths of the original unit rate. */
export const WAVE_TANK_SPEED_DEFAULT = 0.6;
export const WAVE_TANK_SPEED_TICKS = [
  WAVE_TANK_SPEED_MIN,
  WAVE_TANK_SPEED_DEFAULT,
  WAVE_TANK_SPEED_MAX,
] as const;

/** Live HUD slider. Changing it does not rebuild the pool. */
export const WAVE_TANK_SPEED_CONTROL: SceneControl = {
  id: "platform-speed",
  label: "Platform speed",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: WAVE_TANK_SPEED_MIN,
  max: WAVE_TANK_SPEED_MAX,
  step: WAVE_TANK_SPEED_STEP,
  defaultValue: WAVE_TANK_SPEED_DEFAULT,
  unit: "×",
  scale: "linear",
  ticks: WAVE_TANK_SPEED_TICKS,
};

/** Level. The platform does not rise. */
export const WAVE_TANK_AMPLITUDE_MIN = 0;
/** Tallest stroke that still stays under the 12 m side walls. */
export const WAVE_TANK_AMPLITUDE_MAX = 4;
export const WAVE_TANK_AMPLITUDE_STEP = 0.1;
/** About half a meter, enough to send a wave down the pool. */
export const WAVE_TANK_AMPLITUDE_DEFAULT = 0.5;
export const WAVE_TANK_AMPLITUDE_TICKS = [
  WAVE_TANK_AMPLITUDE_MIN,
  WAVE_TANK_AMPLITUDE_DEFAULT,
  WAVE_TANK_AMPLITUDE_MAX,
] as const;

/** Live HUD slider for the platform stroke. Changing it does not rebuild the pool. */
export const WAVE_TANK_AMPLITUDE_CONTROL: SceneControl = {
  id: "platform-amplitude",
  label: "Platform amplitude",
  kind: "range",
  recreates: false,
  surface: "hud",
  min: WAVE_TANK_AMPLITUDE_MIN,
  max: WAVE_TANK_AMPLITUDE_MAX,
  step: WAVE_TANK_AMPLITUDE_STEP,
  defaultValue: WAVE_TANK_AMPLITUDE_DEFAULT,
  unit: "m",
  scale: "linear",
  ticks: WAVE_TANK_AMPLITUDE_TICKS,
};

/** Level plate. Water sits until the platform rises. */
export const WAVE_TANK_SLANT_MIN = 0;
/** Steep enough that water runs off, still a short ramp on a 4 m paddle. */
export const WAVE_TANK_SLANT_MAX = 30;
export const WAVE_TANK_SLANT_STEP = 1;
/** A gentle slope so the pool starts already sliding toward the spill edge. */
export const WAVE_TANK_SLANT_DEFAULT = 10;
export const WAVE_TANK_SLANT_TICKS = [
  WAVE_TANK_SLANT_MIN,
  WAVE_TANK_SLANT_DEFAULT,
  WAVE_TANK_SLANT_MAX,
] as const;

/** Rebuilding slider. Changing the slant recreates the tank. */
export const WAVE_TANK_SLANT_CONTROL: SceneControl = {
  id: "platform-slant",
  label: "Platform slant",
  kind: "range",
  recreates: true,
  surface: "hud",
  min: WAVE_TANK_SLANT_MIN,
  max: WAVE_TANK_SLANT_MAX,
  step: WAVE_TANK_SLANT_STEP,
  defaultValue: WAVE_TANK_SLANT_DEFAULT,
  unit: "°",
  scale: "linear",
  ticks: WAVE_TANK_SLANT_TICKS,
};

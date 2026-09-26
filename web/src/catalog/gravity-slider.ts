import type { SceneControl } from "./scenes";

/** Lowest shared gravity-slider magnitude, in m/s² downward. */
export const GRAVITY_SLIDER_MIN = 2;
/** Former Dam Break High gravity, in m/s² downward. */
export const GRAVITY_SLIDER_FORMER_HIGH = 16;
/** Five times the former High gravity. Keep in sync with the wasm slider cap. */
export const GRAVITY_SLIDER_MAX = GRAVITY_SLIDER_FORMER_HIGH * 5;
export const GRAVITY_SLIDER_STEP = 1;
/** Documented normal gravity, in m/s² downward. */
export const GRAVITY_SLIDER_DEFAULT = 10;
/** Labeled marks on the logarithmic gravity slider. */
export const GRAVITY_SLIDER_TICKS = [
  GRAVITY_SLIDER_MIN,
  GRAVITY_SLIDER_DEFAULT,
  GRAVITY_SLIDER_FORMER_HIGH,
  GRAVITY_SLIDER_MAX,
] as const;

/** Logarithmic gravity slider. Changing it rebuilds the scene at that magnitude. */
export const GRAVITY_CONTROL: SceneControl = {
  id: "gravity",
  label: "Gravity",
  kind: "range",
  recreates: true,
  min: GRAVITY_SLIDER_MIN,
  max: GRAVITY_SLIDER_MAX,
  step: GRAVITY_SLIDER_STEP,
  defaultValue: GRAVITY_SLIDER_DEFAULT,
  unit: "m/s²",
  scale: "logarithmic",
  ticks: GRAVITY_SLIDER_TICKS,
};

export function withGravitySlider(
  controls: readonly SceneControl[],
): readonly SceneControl[] {
  return [...controls, GRAVITY_CONTROL];
}

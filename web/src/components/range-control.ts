import type { SceneControl } from "../catalog/scenes";

export type RangeControl = Extract<SceneControl, { kind: "range" }>;

const STEP_TOLERANCE_RATIO = 1e-4;
/** Linear thumb positions used to sample a logarithmic magnitude. */
export const LOG_SLIDER_POSITION_MAX = 1000;

export type SliderBounds = {
  readonly min: number;
  readonly max: number;
  readonly step: number;
};

export type RangeTick = {
  readonly label: string;
  readonly ratio: number;
};

type RangeBounds = Pick<RangeControl, "min" | "max" | "step" | "scale">;

/** Canonical slider token, or undefined when the text is off the allowed steps. */
export function maybeParseRangeControlValue(
  control: Pick<RangeControl, "min" | "max" | "step">,
  raw: string,
): string | undefined {
  if (raw.length === 0 || raw.trim() !== raw) {
    return undefined;
  }

  const value = Number(raw);
  if (!Number.isFinite(value) || control.step <= 0) {
    return undefined;
  }

  const stepsFromMinimum = Math.round((value - control.min) / control.step);
  const snapped = control.min + stepsFromMinimum * control.step;
  const tolerance = control.step * STEP_TOLERANCE_RATIO;
  if (
    snapped < control.min - tolerance ||
    snapped > control.max + tolerance ||
    Math.abs(snapped - value) > tolerance
  ) {
    return undefined;
  }

  return snapped.toFixed(stepDecimalPlaces(control.step));
}

/** Slider position for a fresh control or a previously applied value. */
export function initialRangeValue(
  control: RangeControl,
  maybeValues: Readonly<Record<string, string>> | undefined,
): string {
  const maybeCurrent = maybeValues?.[control.id];
  if (maybeCurrent !== undefined) {
    const maybeAccepted = maybeParseRangeControlValue(control, maybeCurrent);
    if (maybeAccepted !== undefined) {
      return maybeAccepted;
    }
  }

  const maybeDefault = maybeParseRangeControlValue(
    control,
    String(control.defaultValue),
  );
  return maybeDefault ?? String(control.defaultValue);
}

/** Visible magnitude, such as `10 m/s²`. */
export function formatRangeReadout(value: string, unit: string): string {
  return `${value} ${unit}`;
}

/** Spoken slider value. Keeps the superscript unit out of the accessible name. */
export function formatRangeValueText(value: string, unit: string): string {
  if (unit === "m/s²") {
    return `${value} meters per second squared`;
  }
  if (unit === "×") {
    return `${value} times the original wave speed`;
  }
  if (unit === "°") {
    return `${value} degrees`;
  }

  return formatRangeReadout(value, unit);
}

/** Native range bounds. Logarithmic controls sample the exponent, not the magnitude. */
export function sliderBounds(control: RangeBounds): SliderBounds {
  if (control.scale === "logarithmic") {
    return { min: 0, max: LOG_SLIDER_POSITION_MAX, step: 1 };
  }

  return { min: control.min, max: control.max, step: control.step };
}

/** Thumb position for a magnitude on this control's scale. */
export function sliderPositionForMagnitude(
  control: RangeBounds,
  magnitude: number,
): number {
  if (control.scale !== "logarithmic") {
    return magnitude;
  }

  const ratio = logarithmicRatio(control.min, control.max, magnitude);
  return Math.round(ratio * LOG_SLIDER_POSITION_MAX);
}

/** Magnitude token for a native range position, snapped to the control step. */
export function maybeMagnitudeForSliderPosition(
  control: RangeBounds,
  rawPosition: string,
): string | undefined {
  if (control.scale !== "logarithmic") {
    return maybeParseRangeControlValue(control, rawPosition);
  }

  const position = Number(rawPosition);
  if (!Number.isFinite(position)) {
    return undefined;
  }
  if (position < 0 || position > LOG_SLIDER_POSITION_MAX) {
    return undefined;
  }

  const ratio = position / LOG_SLIDER_POSITION_MAX;
  const magnitude =
    control.min * Math.pow(control.max / control.min, ratio);
  return snapMagnitude(control, magnitude);
}

/** Tick marks positioned on the same scale as the thumb. */
export function rangeTickMarks(control: RangeControl): readonly RangeTick[] {
  return control.ticks.map((tick) => ({
    label: formatTickLabel(tick, control.step),
    ratio: tickRatio(control, tick),
  }));
}

function tickRatio(control: RangeBounds, tick: number): number {
  if (control.max === control.min) {
    return 0;
  }
  if (control.scale === "logarithmic") {
    return logarithmicRatio(control.min, control.max, tick);
  }

  return (tick - control.min) / (control.max - control.min);
}

function logarithmicRatio(min: number, max: number, value: number): number {
  if (value <= min || min <= 0 || max <= min) {
    return 0;
  }
  if (value >= max) {
    return 1;
  }

  return Math.log(value / min) / Math.log(max / min);
}

function snapMagnitude(control: RangeBounds, magnitude: number): string | undefined {
  const stepsFromMinimum = Math.round((magnitude - control.min) / control.step);
  const snapped = control.min + stepsFromMinimum * control.step;
  return maybeParseRangeControlValue(
    control,
    snapped.toFixed(stepDecimalPlaces(control.step)),
  );
}

function formatTickLabel(tick: number, step: number): string {
  return tick.toFixed(stepDecimalPlaces(step));
}

function stepDecimalPlaces(step: number): number {
  const text = String(step);
  const dot = text.indexOf(".");
  if (dot === -1) {
    return 0;
  }

  return text.length - dot - 1;
}

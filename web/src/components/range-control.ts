import type { SceneControl } from "../catalog/scenes";

export type RangeControl = Extract<SceneControl, { kind: "range" }>;

const STEP_TOLERANCE_RATIO = 1e-4;

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

  return formatRangeReadout(value, unit);
}

function stepDecimalPlaces(step: number): number {
  const text = String(step);
  const dot = text.indexOf(".");
  if (dot === -1) {
    return 0;
  }

  return text.length - dot - 1;
}

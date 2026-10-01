/**
 * One-segment world scale, in the style of the Google Maps scale bar.
 *
 * World length is meters. The bar shows the largest 1, 2, or 5 × 10ⁿ distance
 * that fits {@link SCALE_LEGEND_MAX_WIDTH_PX}, then labels it in millimeters,
 * centimeters, meters, or kilometers. Zoom changes pixels per meter, so the
 * same function shortens or lengthens the labeled distance.
 */

export const SCALE_LEGEND_MAX_WIDTH_PX = 100;

const NICE_STEPS = [1, 2, 5, 10] as const;

/** A single scale tick: bar width in CSS pixels and the length it represents. */
export type ScaleLegendMark = {
  readonly widthPx: number;
  readonly label: string;
  readonly accessibleLabel: string;
  readonly meters: number;
};

/**
 * Chooses the one scale tick for the current view.
 *
 * `pixelsPerMeter` is the camera scale: CSS pixels per world meter. Returns
 * undefined when the scale cannot be drawn.
 */
export function scaleLegend(
  pixelsPerMeter: number,
  maxWidthPx = SCALE_LEGEND_MAX_WIDTH_PX,
): ScaleLegendMark | undefined {
  if (!Number.isFinite(pixelsPerMeter) || pixelsPerMeter <= 0) {
    return undefined;
  }
  if (!Number.isFinite(maxWidthPx) || maxWidthPx <= 0) {
    return undefined;
  }

  const maxMeters = maxWidthPx / pixelsPerMeter;
  const maybeMeters = niceMetersAtMost(maxMeters);
  if (maybeMeters === undefined) {
    return undefined;
  }

  const widthPx = Math.min(
    maxWidthPx,
    Math.max(1, Math.round((maybeMeters / maxMeters) * maxWidthPx)),
  );

  return {
    meters: maybeMeters,
    widthPx,
    ...formatWorldLength(maybeMeters),
  };
}

function niceMetersAtMost(maxMeters: number): number | undefined {
  if (!Number.isFinite(maxMeters) || maxMeters <= 0) {
    return undefined;
  }

  const exponent = Math.floor(Math.log10(maxMeters));
  if (!Number.isFinite(exponent)) {
    return undefined;
  }

  const magnitude = 10 ** exponent;
  let chosen = magnitude;
  for (const step of NICE_STEPS) {
    const candidate = step * magnitude;
    if (candidate <= maxMeters * (1 + 1e-9)) {
      chosen = candidate;
    }
  }

  if (!Number.isFinite(chosen) || chosen <= 0) {
    return undefined;
  }

  return chosen;
}

function formatWorldLength(meters: number): {
  readonly label: string;
  readonly accessibleLabel: string;
} {
  if (meters >= 1000) {
    return lengthLabel(meters / 1000, "km", "kilometer", "kilometers");
  }
  if (meters >= 1) {
    return lengthLabel(meters, "m", "meter", "meters");
  }
  if (meters >= 0.01) {
    return lengthLabel(meters * 100, "cm", "centimeter", "centimeters");
  }
  return lengthLabel(meters * 1000, "mm", "millimeter", "millimeters");
}

function lengthLabel(
  count: number,
  symbol: string,
  singular: string,
  plural: string,
): { readonly label: string; readonly accessibleLabel: string } {
  const shown = formatCount(count);
  const spokenUnit = shown === "1" ? singular : plural;
  return {
    label: `${shown} ${symbol}`,
    accessibleLabel: `${shown} ${spokenUnit}`,
  };
}

function formatCount(value: number): string {
  const rounded = Math.round(value);
  if (Math.abs(value - rounded) <= 1e-6 * Math.max(1, Math.abs(value))) {
    return String(rounded);
  }

  return String(Math.round(value * 10) / 10);
}

export const WIREFRAME_STROKE_WIDTH_MIN = 0.1;
export const WIREFRAME_STROKE_WIDTH_MAX = 1.5;
export const WIREFRAME_STROKE_WIDTH_STEP = 0.1;
export const DEFAULT_WIREFRAME_STROKE_WIDTH = 0.3;
export const WIREFRAME_STROKE_WIDTH_STORAGE_KEY =
  "liquidfun.wireframe-stroke-width.v2";

type StrokeWidthStorage = Pick<Storage, "getItem" | "setItem">;
export type StrokeWidthStorageProvider = () => StrokeWidthStorage;

const SNAP_TOLERANCE = 1e-4;

/** Parses one persisted or slider-provided stroke width on the allowed steps. */
export function maybeParseWireframeStrokeWidth(
  maybeValue: string | null,
): number | undefined {
  if (maybeValue === null) {
    return undefined;
  }

  const value = Number(maybeValue);
  if (!Number.isFinite(value)) {
    return undefined;
  }

  const stepsFromMinimum = Math.round(
    (value - WIREFRAME_STROKE_WIDTH_MIN) / WIREFRAME_STROKE_WIDTH_STEP,
  );
  const snapped =
    WIREFRAME_STROKE_WIDTH_MIN +
    stepsFromMinimum * WIREFRAME_STROKE_WIDTH_STEP;
  if (
    snapped < WIREFRAME_STROKE_WIDTH_MIN - SNAP_TOLERANCE ||
    snapped > WIREFRAME_STROKE_WIDTH_MAX + SNAP_TOLERANCE ||
    Math.abs(snapped - value) > SNAP_TOLERANCE
  ) {
    return undefined;
  }

  return Number(snapped.toFixed(2));
}

/** Formats an allowlisted width for the slider readout. */
export function formatWireframeStrokeWidth(width: number): string {
  return width.toFixed(2);
}

/** Loads a stroke preference while containing unavailable browser storage. */
export function loadWireframeStrokeWidth(
  storageProvider: StrokeWidthStorageProvider,
): number {
  try {
    return (
      maybeParseWireframeStrokeWidth(
        storageProvider().getItem(WIREFRAME_STROKE_WIDTH_STORAGE_KEY),
      ) ?? DEFAULT_WIREFRAME_STROKE_WIDTH
    );
  } catch {
    return DEFAULT_WIREFRAME_STROKE_WIDTH;
  }
}

/** Persists a stroke preference without turning storage failure into app failure. */
export function persistWireframeStrokeWidth(
  storageProvider: StrokeWidthStorageProvider,
  width: number,
): void {
  const maybeWidth = maybeParseWireframeStrokeWidth(String(width));
  if (maybeWidth === undefined) {
    return;
  }

  try {
    storageProvider().setItem(
      WIREFRAME_STROKE_WIDTH_STORAGE_KEY,
      String(maybeWidth),
    );
  } catch {
    // The in-memory preference remains valid when storage is unavailable.
  }
}

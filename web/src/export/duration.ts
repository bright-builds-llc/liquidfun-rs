export const DEFAULT_SVG_EXPORT_SECONDS = 10;
export const MIN_SVG_EXPORT_SECONDS = 1;
export const MAX_SVG_EXPORT_SECONDS = 30;
export const SVG_SAMPLE_HZ = 20;
export const PHYSICS_STEP_HZ = 60;
/** Fixed engine steps between SVG samples. Must stay inside the 1–4 advance limit. */
export const STEPS_PER_SVG_SAMPLE = PHYSICS_STEP_HZ / SVG_SAMPLE_HZ;

/** Whole seconds from 1 through 30. Partial text does not select a duration. */
export function maybeParseSvgExportSeconds(raw: string): number | undefined {
  const trimmed = raw.trim();
  if (!/^\d+$/.test(trimmed)) {
    return undefined;
  }

  const value = Number(trimmed);
  if (
    !Number.isSafeInteger(value) ||
    value < MIN_SVG_EXPORT_SECONDS ||
    value > MAX_SVG_EXPORT_SECONDS
  ) {
    return undefined;
  }

  return value;
}

/** Samples from the initial frame through the requested duration, inclusive. */
export function sampleCountForDuration(durationSeconds: number): number {
  return durationSeconds * SVG_SAMPLE_HZ + 1;
}

/** Percent complete for a sampling progress readout. */
export function exportProgressPercent(completed: number, total: number): number {
  if (!Number.isFinite(completed) || !Number.isFinite(total) || total <= 0) {
    return 0;
  }

  return Math.min(100, Math.floor((completed / total) * 100));
}

/** One-decimal seconds for the completion readout. */
export function formatGenerationSeconds(elapsedMs: number): string {
  return `${(elapsedMs / 1000).toFixed(1)} s`;
}

/** Download name for one scene clip. */
export function svgExportFileName(sceneId: string, durationSeconds: number): string {
  return `${sceneId}-${durationSeconds}s.svg`;
}

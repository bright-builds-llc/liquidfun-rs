export const STEP_SECONDS = 1 / 60;
export const MAX_STEPS_PER_FRAME = 4;
export const MAX_DELTA_SECONDS = MAX_STEPS_PER_FRAME * STEP_SECONDS;

/** Convert elapsed wall time into a 0–4 engine-step budget. */
export function acceptedStepCount(elapsedSeconds: number): number {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return 0;
  }

  const clamped = Math.min(elapsedSeconds, MAX_DELTA_SECONDS);
  return Math.min(MAX_STEPS_PER_FRAME, Math.floor(clamped / STEP_SECONDS));
}

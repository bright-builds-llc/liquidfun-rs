export const STEP_SECONDS = 1 / 60;
export const MAX_STEPS_PER_FRAME = 4;
export const MAX_DELTA_SECONDS = MAX_STEPS_PER_FRAME * STEP_SECONDS;
/** Leave a few milliseconds after physics so the canvas can paint on this vsync. */
export const FRAME_STEP_BUDGET_MS = 12;

export type AccumulatedStepTime = {
  readonly stepCount: number;
  readonly remainderSeconds: number;
};

/** Convert elapsed wall time into a 0–4 engine-step budget. */
export function acceptedStepCount(elapsedSeconds: number): number {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return 0;
  }

  const clamped = Math.min(elapsedSeconds, MAX_DELTA_SECONDS);
  return Math.min(MAX_STEPS_PER_FRAME, Math.floor(clamped / STEP_SECONDS));
}

/** Accumulate fractional frame time while retaining the four-step cap. */
export function accumulateStepTime(
  remainderSeconds: number,
  elapsedSeconds: number,
): AccumulatedStepTime {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return { stepCount: 0, remainderSeconds };
  }

  const accumulatedSeconds = Math.min(
    remainderSeconds + elapsedSeconds,
    MAX_DELTA_SECONDS,
  );
  const stepCount = acceptedStepCount(accumulatedSeconds);
  return {
    stepCount,
    remainderSeconds: accumulatedSeconds - stepCount * STEP_SECONDS,
  };
}

/** Puts steps the frame budget did not run back into the accumulator. */
export function restoreUnrunSteps(
  remainderSeconds: number,
  requested: number,
  ran: number,
): number {
  const unrun = Math.max(0, requested - ran);
  return remainderSeconds + unrun * STEP_SECONDS;
}

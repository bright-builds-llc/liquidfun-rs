/**
 * Valve open-time arithmetic and copy (Phase 40 D-05, D-06). Pure: every
 * value derives from the placement's open tick and the frame step index.
 */

/** Fixed steps per half-second open tick; mirrors Rust `valve::STEPS_PER_OPEN_TICK`. */
export const STEPS_PER_OPEN_TICK = 30;

/** Fixed steps per tenth of a second at 60 Hz. */
const STEPS_PER_TENTH = 6;

/** Latest open tick a level allows: the whole half-seconds within its step limit. */
export function maxOpenTick(stepLimit: number): number {
  return Math.floor(stepLimit / STEPS_PER_OPEN_TICK);
}

/** Simulation step at which a valve with this open tick is destroyed. */
export function openStepFor(openTick: number): number {
  return openTick * STEPS_PER_OPEN_TICK;
}

/** Open time as seconds with one decimal, for example "2.0 s". */
export function valveSecondsText(openTick: number): string {
  return `${(openTick / 2).toFixed(1)} s`;
}

/** Time left until the open step, rounded up to a tenth of a second. */
export function valveCountdownText(openStep: number, stepIndex: number): string {
  const tenths = Math.ceil((openStep - stepIndex) / STEPS_PER_TENTH);
  return `${(tenths / 10).toFixed(1)} s`;
}

import { STEP_SECONDS } from "../physics/clock";

/** Fast-forward multiplier for puzzle runs (D-24). */
export type PuzzleSpeed = 1 | 2 | 4;

/** Speeds in cycle order. */
export const PUZZLE_SPEEDS: readonly PuzzleSpeed[] = [1, 2, 4];

/** Speed on a new route; it then persists across runs in the page session (D-24). */
export const INITIAL_PUZZLE_SPEED: PuzzleSpeed = 1;

/** True for 1, 2 or 4. */
export function isPuzzleSpeed(value: number): value is PuzzleSpeed {
  return value === 1 || value === 2 || value === 4;
}

/** Cycles 1 → 2 → 4 → 1. */
export function nextPuzzleSpeed(speed: PuzzleSpeed): PuzzleSpeed {
  switch (speed) {
    case 1:
      return 2;
    case 2:
      return 4;
    case 4:
      return 1;
    default: {
      const unreachable: never = speed;
      return unreachable;
    }
  }
}

/** Button text such as "2×" (U+00D7). */
export function speedLabel(speed: PuzzleSpeed): string {
  return `${speed}×`;
}

/** Fixed steps requested this frame (D-24); the loop still steps one `nextFrame(1)` per capture. */
export function speedStepRequest(stepCount: number, speed: PuzzleSpeed): number {
  if (!Number.isSafeInteger(stepCount) || stepCount < 0) {
    throw new Error(`Invalid step count: ${stepCount}`);
  }

  return stepCount * speed;
}

/** Restores unrun steps as wall time: (max(0, budget − ran) / speed) × STEP_SECONDS. */
export function restoreUnrunSpeedSteps(
  remainderSeconds: number,
  budget: number,
  ran: number,
  speed: PuzzleSpeed,
): number {
  const unrun = Math.max(0, budget - ran);
  return remainderSeconds + (unrun / speed) * STEP_SECONDS;
}

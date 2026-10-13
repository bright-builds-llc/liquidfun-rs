import type { PuzzleFrameSound } from "../physics/frame";

/** Sound aggregates folded over every fixed step of one presented frame (Phase 42 D-07, Research Pattern 3). */
export type PresentedSoundSignals = {
  /** Fixed steps folded; triggers divide splash contacts by it so 2x and 4x are not louder. */
  readonly steps: number;
  readonly splashContacts: number;
  readonly impactCount: number;
  /** Largest impact impulse of any folded step (N·s). */
  readonly impactImpulse: number;
};

/** Signals before any step of a presented frame has been folded. */
export const NO_SOUND_SIGNALS: PresentedSoundSignals = {
  steps: 0,
  splashContacts: 0,
  impactCount: 0,
  impactImpulse: 0,
};

/** Adds one fixed step; total and non-throwing so the frame loop never fails on sound data. */
export function foldSoundSignals(
  accumulated: PresentedSoundSignals,
  maybeSound: PuzzleFrameSound | undefined,
): PresentedSoundSignals {
  if (maybeSound === undefined) {
    return { ...accumulated, steps: accumulated.steps + 1 };
  }

  return {
    steps: accumulated.steps + 1,
    splashContacts: accumulated.splashContacts + maybeSound.splashContacts,
    impactCount: accumulated.impactCount + maybeSound.impactCount,
    impactImpulse: Math.max(accumulated.impactImpulse, maybeSound.impactImpulse),
  };
}

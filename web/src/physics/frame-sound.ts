import type {
  PuzzleFrameSound,
  PuzzleFrameStatus,
  RawProofFrame,
} from "./frame";
import { invalidFrame, parseNonNegativeInteger } from "./frame-scalars";

function parseImpulse(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    return invalidFrame();
  }

  return value;
}

/**
 * Parses the read-only per-step sound aggregates (Phase 42 D-07), fail closed.
 *
 * The three getters are all-or-nothing and required on puzzle frames: a
 * scene frame (no `maybePuzzle`) must report none of them, and a puzzle frame
 * must report all three. Counts are non-negative integers, the impulse is
 * finite and non-negative, and no impulse may appear without a counted
 * impact, mirroring `FrameData::set_puzzle_sound` in Rust.
 */
export function parsePuzzleSound(
  rawFrame: RawProofFrame,
  maybePuzzle: PuzzleFrameStatus | undefined,
): PuzzleFrameSound | undefined {
  const maybeSplashContacts = rawFrame.splashContacts?.();
  const maybeImpactCount = rawFrame.impactCount?.();
  const maybeImpactImpulse = rawFrame.impactImpulse?.();
  const presentCount = [
    maybeSplashContacts,
    maybeImpactCount,
    maybeImpactImpulse,
  ].filter((value) => value !== undefined).length;

  if (maybePuzzle === undefined) {
    return presentCount === 0 ? undefined : invalidFrame();
  }

  if (
    maybeSplashContacts === undefined ||
    maybeImpactCount === undefined ||
    maybeImpactImpulse === undefined
  ) {
    return invalidFrame();
  }

  const splashContacts = parseNonNegativeInteger(maybeSplashContacts);
  const impactCount = parseNonNegativeInteger(maybeImpactCount);
  const impactImpulse = parseImpulse(maybeImpactImpulse);
  if (impactCount === 0 && impactImpulse !== 0) {
    return invalidFrame();
  }

  return { splashContacts, impactCount, impactImpulse };
}

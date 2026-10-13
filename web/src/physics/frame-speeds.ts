import type { RawProofFrame } from "./frame";
import { invalidFrame } from "./frame-scalars";

/**
 * Parses the optional per-particle speed lane (Phase 43 D-10), fail closed.
 *
 * Absent (no getter, or the getter returns undefined) stays absent. A present
 * lane must be a Float32Array of exactly `particleCount` finite, non-negative
 * values, mirroring `FrameData::set_particle_speeds` in Rust. The same array
 * instance is returned without a copy.
 */
export function parseParticleSpeeds(
  rawFrame: RawProofFrame,
  particleCount: number,
): Float32Array | undefined {
  const maybeSpeeds: unknown = rawFrame.particleSpeeds?.();
  if (maybeSpeeds === undefined) {
    return undefined;
  }

  if (
    !(maybeSpeeds instanceof Float32Array) ||
    maybeSpeeds.length !== particleCount
  ) {
    return invalidFrame();
  }

  for (const speed of maybeSpeeds) {
    if (!Number.isFinite(speed) || speed < 0) {
      return invalidFrame();
    }
  }

  return maybeSpeeds;
}

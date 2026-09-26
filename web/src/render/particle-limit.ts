/** Highest particle count the frame protocol can carry. */
export const MAX_RENDERED_PARTICLE_LIMIT = 16384;

/** Default draw cap for scenes whose fills stay under a few thousand particles. */
export const DEFAULT_RENDERED_PARTICLE_LIMIT = 4000;

/** Draw cap applied when a scene starts. The tumbler fill is above the default. */
export function initialRenderedParticleLimit(sceneId: string): number {
  if (sceneId === "liquid-tumbler") {
    return MAX_RENDERED_PARTICLE_LIMIT;
  }

  return DEFAULT_RENDERED_PARTICLE_LIMIT;
}

/**
 * Step through the particle list so a draw cap still covers the whole liquid.
 *
 * A cap at or above the live count keeps every particle, in order.
 */
export function particleDrawStride(
  particleCount: number,
  maxRenderedParticles: number,
): number {
  if (maxRenderedParticles <= 0 || particleCount <= maxRenderedParticles) {
    return 1;
  }

  return Math.ceil(particleCount / maxRenderedParticles);
}

/** Parses a typed particle cap. Empty or partial text does not change the cap. */
export function maybeParseRenderedParticleLimit(
  raw: string,
): number | undefined {
  const trimmed = raw.trim();
  if (!/^\d+$/.test(trimmed)) {
    return undefined;
  }

  const value = Number(trimmed);
  if (
    !Number.isSafeInteger(value) ||
    value > MAX_RENDERED_PARTICLE_LIMIT
  ) {
    return undefined;
  }

  return value;
}

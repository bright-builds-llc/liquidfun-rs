/** Highest particle count the frame protocol can carry. */
export const MAX_RENDERED_PARTICLE_LIMIT = 10240;

/** Default draw cap, above the current scene budgets. */
export const DEFAULT_RENDERED_PARTICLE_LIMIT = 4000;

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

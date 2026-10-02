import type { RenderFrame } from "../physics/frame";

const ALL_CHANGED = { radiiChanged: true, colorsChanged: true } as const;
const NEITHER_CHANGED = { radiiChanged: false, colorsChanged: false } as const;
const RADII_CHANGED = { radiiChanged: true, colorsChanged: false } as const;
const COLORS_CHANGED = { radiiChanged: false, colorsChanged: true } as const;

function equal(
  first: Uint32Array | Uint8Array,
  second: Uint32Array | Uint8Array,
): boolean {
  if (first.length > second.length) return false;
  for (let index = 0; index < first.length; index++)
    if (first[index] !== second[index]) return false;
  return true;
}

/** Reuses exact GPU metadata for stable counts; changing counts take the full-upload path. */
export function createParticleMetadataCache() {
  let radiiSnapshot = new Uint32Array(0);
  let colorsSnapshot = new Uint8Array(0);
  let maybeScale: number | undefined;
  let previousCount = -1;
  let snapshotCurrent = false;
  return {
    changes(frame: RenderFrame, scale: number) {
      if (previousCount !== frame.particleCount) {
        const first = previousCount === -1;
        previousCount = frame.particleCount;
        snapshotCurrent = false;
        // Volatile streams cannot reuse the GPU lanes, so avoid scanning/copying them.
        if (!first) return ALL_CHANGED;
      }
      const radii = new Uint32Array(
        frame.particleRadii.buffer,
        frame.particleRadii.byteOffset,
        frame.particleRadii.length,
      );
      const radiiChanged =
        !snapshotCurrent ||
        maybeScale !== scale ||
        !equal(radii, radiiSnapshot);
      const colorsChanged =
        !snapshotCurrent || !equal(frame.particleColors, colorsSnapshot);
      if (radiiChanged) {
        if (radiiSnapshot.length < radii.length)
          radiiSnapshot = new Uint32Array(
            Math.max(radii.length, radiiSnapshot.length * 2),
          );
        radiiSnapshot.set(radii);
      }
      if (colorsChanged) {
        if (colorsSnapshot.length < frame.particleColors.length)
          colorsSnapshot = new Uint8Array(
            Math.max(frame.particleColors.length, colorsSnapshot.length * 2),
          );
        colorsSnapshot.set(frame.particleColors);
      }
      maybeScale = scale;
      snapshotCurrent = true;
      return radiiChanged
        ? colorsChanged
          ? ALL_CHANGED
          : RADII_CHANGED
        : colorsChanged
          ? COLORS_CHANGED
          : NEITHER_CHANGED;
    },
  };
}

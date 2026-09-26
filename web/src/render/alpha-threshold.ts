/** Alpha cutoff that snaps a blurred particle mask into a solid silhouette. */
export const MASK_ALPHA_THRESHOLD = 128;

/**
 * Turns a blurred coverage mask into a binary alpha image.
 *
 * RGB is forced to white so the mask can clip a separate color buffer.
 */
export function thresholdAlpha(
  pixels: Uint8ClampedArray,
  threshold = MASK_ALPHA_THRESHOLD,
): void {
  for (let index = 0; index < pixels.length; index += 4) {
    const coverage = pixels[index + 3] ?? 0;
    const on = coverage >= threshold;
    pixels[index] = 255;
    pixels[index + 1] = 255;
    pixels[index + 2] = 255;
    pixels[index + 3] = on ? 255 : 0;
  }
}

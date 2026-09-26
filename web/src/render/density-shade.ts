/**
 * Darkens particle color where kernels overlap.
 *
 * An isolated particle peaks at 1. Playground water rests near 1.6 radii, so a
 * normal interior stays close to full brightness. Tighter piles, such as the
 * bottom of a container, climb toward the packed end and approach the floor.
 */

/** Kernel sum where darkening begins. A lone particle center stays below this. */
export const DENSITY_SHADE_START = 1.15;

/** Kernel sum that reaches the darkest packed shade. */
export const DENSITY_SHADE_END = 2.8;

/** Brightness kept where particles are packed well past a single kernel. */
export const DENSITY_SHADE_FLOOR = 0.46;

function clamp01(value: number): number {
  if (value < 0) {
    return 0;
  }
  if (value > 1) {
    return 1;
  }
  return value;
}

function smoothstep(edge0: number, edge1: number, value: number): number {
  if (!Number.isFinite(value)) {
    return 0;
  }

  const span = edge1 - edge0;
  if (span <= 0) {
    return value >= edge1 ? 1 : 0;
  }

  const t = clamp01((value - edge0) / span);
  return t * t * (3 - 2 * t);
}

/** Maps an accumulated kernel sum to a brightness multiplier in `(floor, 1]`. */
export function densityShade(density: number): number {
  if (!Number.isFinite(density)) {
    return 1;
  }

  const packed = smoothstep(DENSITY_SHADE_START, DENSITY_SHADE_END, density);
  return 1 - packed * (1 - DENSITY_SHADE_FLOOR);
}

function scaleChannel(channel: number | undefined, shade: number): number {
  return Math.round((channel ?? 0) * shade);
}

/**
 * Multiplies straight-alpha color pixels by the shade at each pixel center.
 *
 * Transparent pixels are left alone so a mask can still clip the silhouette.
 */
export function shadePackedColor(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
  densityAt: (x: number, y: number) => number,
): void {
  if (width <= 0 || height <= 0 || pixels.length < width * height * 4) {
    return;
  }

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const index = (x + y * width) * 4;
      const alpha = pixels[index + 3] ?? 0;
      if (alpha === 0) {
        continue;
      }

      const shade = densityShade(densityAt(x + 0.5, y + 0.5));
      pixels[index] = scaleChannel(pixels[index], shade);
      pixels[index + 1] = scaleChannel(pixels[index + 1], shade);
      pixels[index + 2] = scaleChannel(pixels[index + 2], shade);
    }
  }
}

/** Applies `shadePackedColor` to one offscreen color buffer. */
export function shadeContext(
  context: CanvasRenderingContext2D,
  maybeDensity: ((x: number, y: number) => number) | undefined,
): void {
  if (maybeDensity === undefined) {
    return;
  }

  const width = context.canvas.width;
  const height = context.canvas.height;
  if (width <= 0 || height <= 0) {
    return;
  }

  const image = context.getImageData(0, 0, width, height);
  shadePackedColor(image.data, width, height, maybeDensity);
  context.putImageData(image, 0, 0);
}

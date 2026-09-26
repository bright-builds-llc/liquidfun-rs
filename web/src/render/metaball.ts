import type { RenderFrame } from "../physics/frame";
import { MASK_ALPHA_THRESHOLD, thresholdAlpha } from "./alpha-threshold";
import type { Camera } from "./camera";
import { collectFieldSamples, maybeFieldDensity } from "./contour";
import { shadeContext } from "./density-shade";
import { eachProjectedParticle } from "./projected-particle";
import { compositeMasked, scratchContext } from "./scratch-canvas";

/** Blur, in particle radii, that smooths the beaded edge before the cutoff. */
const BLUR_RADIUS_FACTOR = 0.8;

/** CSS blur spreads about three radii past the disc, so color must cover that halo. */
const COLOR_BLEED = 3;

const MIN_BLUR_PX = 0.75;
const TAU = Math.PI * 2;

function maxProjectedRadius(
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
): number {
  let maxRadius = 0;
  eachProjectedParticle(
    frame,
    camera,
    maxRenderedParticles,
    (x, y, radius) => {
      void x;
      void y;
      if (radius > maxRadius) {
        maxRadius = radius;
      }
    },
  );
  return maxRadius;
}

function drawDiscs(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
  radiusPadding: number,
  colored: boolean,
): void {
  eachProjectedParticle(
    frame,
    camera,
    maxRenderedParticles,
    (x, y, radius, red, green, blue, alpha) => {
      context.fillStyle = colored
        ? `rgba(${red}, ${green}, ${blue}, ${alpha})`
        : "#FFFFFF";
      context.beginPath();
      context.arc(x, y, radius + radiusPadding, 0, TAU);
      context.fill();
    },
  );
}

/**
 * Draws particles as one soft surface.
 *
 * White discs are blurred and thresholded into a mask. Particle colors are
 * darkened where kernels overlap, then clipped to that mask.
 */
export function paintMetaball(
  target: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
  densityShading = true,
): void {
  const width = camera.viewport.width;
  const height = camera.viewport.height;
  const maxRadius = maxProjectedRadius(frame, camera, maxRenderedParticles);
  if (maxRadius <= 0 || width <= 0 || height <= 0) {
    return;
  }

  const blur = Math.max(MIN_BLUR_PX, maxRadius * BLUR_RADIUS_FACTOR);
  const color = scratchContext("metaball-color", width, height);
  const sharp = scratchContext("metaball-sharp", width, height);
  const blurred = scratchContext("metaball-blur", width, height);
  color.clearRect(0, 0, color.canvas.width, color.canvas.height);
  sharp.clearRect(0, 0, sharp.canvas.width, sharp.canvas.height);
  drawDiscs(color, frame, camera, maxRenderedParticles, blur * COLOR_BLEED, true);
  if (densityShading) {
    shadeContext(
      color,
      maybeFieldDensity(
        collectFieldSamples(frame, camera, maxRenderedParticles),
        color.canvas.width,
        color.canvas.height,
      ),
    );
  }
  drawDiscs(sharp, frame, camera, maxRenderedParticles, 0, false);

  blurred.clearRect(0, 0, blurred.canvas.width, blurred.canvas.height);
  blurred.filter = `blur(${blur}px)`;
  blurred.drawImage(sharp.canvas, 0, 0);
  blurred.filter = "none";
  const image = blurred.getImageData(0, 0, blurred.canvas.width, blurred.canvas.height);
  thresholdAlpha(image.data, MASK_ALPHA_THRESHOLD);
  blurred.putImageData(image, 0, 0);
  compositeMasked(target, color.canvas, blurred.canvas, width, height);
}

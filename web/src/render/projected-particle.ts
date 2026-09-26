import type { RenderFrame } from "../physics/frame";
import { projectPoint, projectRadius, type Camera } from "./camera";
import { particleDrawStride } from "./particle-limit";

const INVALID_RENDER_FRAME_MESSAGE = "Invalid renderer frame";

function valueAt(values: Float32Array | Uint8Array, index: number): number {
  const value = values[index];
  if (value === undefined) {
    throw new Error(INVALID_RENDER_FRAME_MESSAGE);
  }

  return value;
}

/** Visits each particle that the active draw cap still includes. */
export function eachProjectedParticle(
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
  visit: (
    x: number,
    y: number,
    radius: number,
    red: number,
    green: number,
    blue: number,
    alpha: number,
  ) => void,
): void {
  const stride = particleDrawStride(frame.particleCount, maxRenderedParticles);
  for (
    let particleIndex = 0;
    particleIndex < frame.particleCount && maxRenderedParticles > 0;
    particleIndex += stride
  ) {
    const positionIndex = particleIndex * 2;
    const colorIndex = particleIndex * 4;
    const center = projectPoint(camera, {
      x: valueAt(frame.particlePositions, positionIndex),
      y: valueAt(frame.particlePositions, positionIndex + 1),
    });
    visit(
      center.x,
      center.y,
      projectRadius(camera, valueAt(frame.particleRadii, particleIndex)),
      valueAt(frame.particleColors, colorIndex),
      valueAt(frame.particleColors, colorIndex + 1),
      valueAt(frame.particleColors, colorIndex + 2),
      valueAt(frame.particleColors, colorIndex + 3) / 255,
    );
  }
}

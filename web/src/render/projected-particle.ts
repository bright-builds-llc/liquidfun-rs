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

/** Packs every current position, with a straight-line full-metadata path for volatile streams. */
export function fillProjectedParticleArrays(
  frame: RenderFrame,
  camera: Camera,
  positions: Float32Array,
  radii: Float32Array,
  colors: Float32Array,
  writeRadii: boolean,
  writeColors: boolean,
): number {
  let count = 0;
  const full = (
    x: number,
    y: number,
    radius: number,
    red: number,
    green: number,
    blue: number,
    alpha: number,
  ) => {
    const positionIndex = count * 2,
      colorIndex = count * 4;
    positions[positionIndex] = x;
    positions[positionIndex + 1] = y;
    radii[count] = radius;
    colors[colorIndex] = red / 255;
    colors[colorIndex + 1] = green / 255;
    colors[colorIndex + 2] = blue / 255;
    colors[colorIndex + 3] = alpha;
    count += 1;
  };
  const positionOnly = (x: number, y: number) => {
    const index = count * 2;
    positions[index] = x;
    positions[index + 1] = y;
    count += 1;
  };
  const partial = (
    x: number,
    y: number,
    radius: number,
    red: number,
    green: number,
    blue: number,
    alpha: number,
  ) => {
    const positionIndex = count * 2,
      colorIndex = count * 4;
    positions[positionIndex] = x;
    positions[positionIndex + 1] = y;
    if (writeRadii) radii[count] = radius;
    if (writeColors) {
      colors[colorIndex] = red / 255;
      colors[colorIndex + 1] = green / 255;
      colors[colorIndex + 2] = blue / 255;
      colors[colorIndex + 3] = alpha;
    }
    count += 1;
  };
  const visit =
    writeRadii && writeColors
      ? full
      : writeRadii || writeColors
        ? partial
        : positionOnly;
  eachProjectedParticle(frame, camera, Number.POSITIVE_INFINITY, visit);
  return count;
}

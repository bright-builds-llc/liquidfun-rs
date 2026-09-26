import type { RenderFrame } from "../physics/frame";
import { projectPoint, projectRadius, type Camera } from "../render/camera";
import { particleDrawStride } from "../render/particle-limit";

const INVALID_EXPORT_FRAME_MESSAGE = "Invalid export frame";

export type ProjectedParticle = {
  readonly x: number;
  readonly y: number;
  readonly radius: number;
  readonly red: number;
  readonly green: number;
  readonly blue: number;
  readonly alpha: number;
};

export type ProjectedSegment = {
  readonly x1: number;
  readonly y1: number;
  readonly x2: number;
  readonly y2: number;
};

export type ProjectedBody = {
  readonly x: number;
  readonly y: number;
  readonly radius: number;
  readonly label: string;
};

export type ProjectedSample = {
  readonly particles: readonly ProjectedParticle[];
  readonly segments: readonly ProjectedSegment[];
  readonly bodies: readonly ProjectedBody[];
};

function valueAt(values: Float32Array | Uint8Array, index: number): number {
  const value = values[index];
  if (value === undefined || !Number.isFinite(value)) {
    throw new Error(INVALID_EXPORT_FRAME_MESSAGE);
  }

  return value;
}

/** Projects one validated frame with the same camera the canvas uses. */
export function projectRenderSample(
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
): ProjectedSample {
  const stride = particleDrawStride(frame.particleCount, maxRenderedParticles);
  const particles: ProjectedParticle[] = [];
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
    particles.push({
      x: center.x,
      y: center.y,
      radius: projectRadius(camera, valueAt(frame.particleRadii, particleIndex)),
      red: valueAt(frame.particleColors, colorIndex),
      green: valueAt(frame.particleColors, colorIndex + 1),
      blue: valueAt(frame.particleColors, colorIndex + 2),
      alpha: valueAt(frame.particleColors, colorIndex + 3) / 255,
    });
  }

  const segments: ProjectedSegment[] = [];
  for (
    let segmentIndex = 0;
    segmentIndex < frame.rigidSegments.length;
    segmentIndex += 4
  ) {
    const start = projectPoint(camera, {
      x: valueAt(frame.rigidSegments, segmentIndex),
      y: valueAt(frame.rigidSegments, segmentIndex + 1),
    });
    const end = projectPoint(camera, {
      x: valueAt(frame.rigidSegments, segmentIndex + 2),
      y: valueAt(frame.rigidSegments, segmentIndex + 3),
    });
    segments.push({ x1: start.x, y1: start.y, x2: end.x, y2: end.y });
  }

  const bodies: ProjectedBody[] = [];
  for (
    let circleIndex = 0;
    circleIndex < frame.rigidCircles.length;
    circleIndex += 3
  ) {
    const center = projectPoint(camera, {
      x: valueAt(frame.rigidCircles, circleIndex),
      y: valueAt(frame.rigidCircles, circleIndex + 1),
    });
    bodies.push({
      x: center.x,
      y: center.y,
      radius: projectRadius(camera, valueAt(frame.rigidCircles, circleIndex + 2)),
      label: frame.circleLabels[circleIndex / 3] ?? "",
    });
  }

  return { particles, segments, bodies };
}

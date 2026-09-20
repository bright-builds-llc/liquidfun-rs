const MAX_PARTICLE_COUNT = 10240;
const MAX_RIGID_SEGMENT_COUNT = 16;
const MAX_RIGID_CIRCLE_COUNT = 8;
const PARTICLE_POSITION_STRIDE = 2;
const PARTICLE_COLOR_STRIDE = 4;
const PARTICLE_RADIUS_STRIDE = 1;
const RIGID_SEGMENT_STRIDE = 4;
const RIGID_CIRCLE_STRIDE = 3;
const INVALID_FRAME_MESSAGE = "Invalid Rust/WASM frame";

/** Generated frame methods consumed by the validated browser boundary. */
export interface RawProofFrame {
  stepIndex(): number;
  particleCount(): number;
  rigidShapeCount(): number;
  particlePositions(): Float32Array;
  particleColors(): Uint8Array;
  particleRadii(): Float32Array;
  rigidSegments(): Float32Array;
  rigidCircles(): Float32Array;
  free(): void;
}

/** Bounded JavaScript-owned numeric lanes ready for rendering. */
export interface RenderFrame {
  readonly stepIndex: number;
  readonly particleCount: number;
  readonly rigidShapeCount: number;
  readonly particlePositions: Float32Array;
  readonly particleColors: Uint8Array;
  readonly particleRadii: Float32Array;
  readonly rigidSegments: Float32Array;
  readonly rigidCircles: Float32Array;
}

function invalidFrame(): never {
  throw new Error(INVALID_FRAME_MESSAGE);
}

function parseNonNegativeInteger(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0) {
    return invalidFrame();
  }

  return value;
}

function parseBoundedCount(value: number, maximum: number): number {
  const count = parseNonNegativeInteger(value);
  if (count > maximum) {
    return invalidFrame();
  }

  return count;
}

function checkedExpectedLength(count: number, stride: number): number {
  const expectedLength = count * stride;
  if (!Number.isSafeInteger(expectedLength)) {
    return invalidFrame();
  }

  return expectedLength;
}

function requireLength(
  values: Float32Array | Uint8Array,
  count: number,
  stride: number,
): void {
  if (values.length !== checkedExpectedLength(count, stride)) {
    invalidFrame();
  }
}

function parseLaneCount(length: number, stride: number, maximum: number): number {
  if (!Number.isSafeInteger(length) || length % stride !== 0) {
    return invalidFrame();
  }

  return parseBoundedCount(length / stride, maximum);
}

function requireFinite(values: Float32Array): void {
  for (const value of values) {
    if (!Number.isFinite(value)) {
      invalidFrame();
    }
  }
}

function requirePositive(values: Float32Array): void {
  requireFinite(values);
  for (const value of values) {
    if (value <= 0) {
      invalidFrame();
    }
  }
}

function requirePositiveCircleRadii(circles: Float32Array): void {
  for (
    let radiusIndex = RIGID_CIRCLE_STRIDE - 1;
    radiusIndex < circles.length;
    radiusIndex += RIGID_CIRCLE_STRIDE
  ) {
    const radius = circles[radiusIndex];
    if (radius === undefined || radius <= 0) {
      invalidFrame();
    }
  }
}

/**
 * Parses one generated frame into bounded renderer-ready copied lanes.
 *
 * The generated boxed-slice getters already return JavaScript-owned arrays, so
 * this function validates and returns those arrays without retaining the Rust
 * frame wrapper.
 */
export function parseRenderFrame(rawFrame: RawProofFrame): RenderFrame {
  const stepIndex = parseNonNegativeInteger(rawFrame.stepIndex());
  const particleCount = parseBoundedCount(
    rawFrame.particleCount(),
    MAX_PARTICLE_COUNT,
  );
  const rigidShapeCount = parseBoundedCount(
    rawFrame.rigidShapeCount(),
    MAX_RIGID_SEGMENT_COUNT + MAX_RIGID_CIRCLE_COUNT,
  );

  const particlePositions = rawFrame.particlePositions();
  const particleColors = rawFrame.particleColors();
  const particleRadii = rawFrame.particleRadii();
  const rigidSegments = rawFrame.rigidSegments();
  const rigidCircles = rawFrame.rigidCircles();

  if (
    !(particlePositions instanceof Float32Array) ||
    !(particleColors instanceof Uint8Array) ||
    !(particleRadii instanceof Float32Array) ||
    !(rigidSegments instanceof Float32Array) ||
    !(rigidCircles instanceof Float32Array)
  ) {
    return invalidFrame();
  }

  requireLength(
    particlePositions,
    particleCount,
    PARTICLE_POSITION_STRIDE,
  );
  requireLength(particleColors, particleCount, PARTICLE_COLOR_STRIDE);
  requireLength(particleRadii, particleCount, PARTICLE_RADIUS_STRIDE);

  const rigidSegmentCount = parseLaneCount(
    rigidSegments.length,
    RIGID_SEGMENT_STRIDE,
    MAX_RIGID_SEGMENT_COUNT,
  );
  const rigidCircleCount = parseLaneCount(
    rigidCircles.length,
    RIGID_CIRCLE_STRIDE,
    MAX_RIGID_CIRCLE_COUNT,
  );
  if (rigidShapeCount !== rigidSegmentCount + rigidCircleCount) {
    return invalidFrame();
  }

  requireFinite(particlePositions);
  requirePositive(particleRadii);
  requireFinite(rigidSegments);
  requireFinite(rigidCircles);
  requirePositiveCircleRadii(rigidCircles);

  return {
    stepIndex,
    particleCount,
    rigidShapeCount,
    particlePositions,
    particleColors,
    particleRadii,
    rigidSegments,
    rigidCircles,
  };
}

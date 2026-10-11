const MAX_PARTICLE_COUNT = 16384;
const MAX_RIGID_SEGMENT_COUNT = 512;
const MAX_RIGID_CIRCLE_COUNT = 48;
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
  maxSpeed(): number;
  stuckCandidateCount(): number;
  bodyContactCount(): number;
  particlePositions(): Float32Array;
  particleColors(): Uint8Array;
  particleRadii(): Float32Array;
  rigidSegments(): Float32Array;
  rigidCircles(): Float32Array;
  circleLabels(): readonly string[];
  /** Puzzle scalars (D-17); generated getters return undefined on scene sessions. */
  goalCount?(): number | undefined;
  goalTarget?(): number | undefined;
  goalFull?(): number | undefined;
  outcome?(): number | undefined;
  outcomeStep?(): number | undefined;
  stepLimit?(): number | undefined;
  free(): void;
}

/** Puzzle outcome latched per fixed step in Rust. */
export type PuzzleOutcome = "running" | "won" | "lost";

/** Per-step puzzle readout decided in Rust (D-17). Absent on scene frames. */
export interface PuzzleFrameStatus {
  readonly goalCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly outcome: PuzzleOutcome;
  readonly maybeOutcomeStep?: number;
  readonly stepLimit: number;
}

/** Bounded JavaScript-owned numeric lanes ready for rendering. */
export interface RenderFrame {
  readonly stepIndex: number;
  readonly particleCount: number;
  readonly rigidShapeCount: number;
  readonly maxSpeed: number;
  readonly stuckCandidateCount: number;
  readonly bodyContactCount: number;
  readonly particlePositions: Float32Array;
  readonly particleColors: Uint8Array;
  readonly particleRadii: Float32Array;
  readonly rigidSegments: Float32Array;
  readonly rigidCircles: Float32Array;
  readonly circleLabels: readonly string[];
  readonly maybePuzzle?: PuzzleFrameStatus;
}

const PUZZLE_OUTCOMES: readonly PuzzleOutcome[] = ["running", "won", "lost"];

function invalidFrame(): never {
  throw new Error(INVALID_FRAME_MESSAGE);
}

function parseNonNegativeInteger(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0) {
    return invalidFrame();
  }

  return value;
}

function parseNonNegativeFinite(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
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

function parsePositiveInteger(value: number): number {
  const parsed = parseNonNegativeInteger(value);
  if (parsed < 1) {
    return invalidFrame();
  }

  return parsed;
}

function parseOutcomeStep(
  outcome: PuzzleOutcome,
  maybeOutcomeStep: number | undefined,
  stepIndex: number,
  stepLimit: number,
): number | undefined {
  if (outcome === "running") {
    return maybeOutcomeStep === undefined ? undefined : invalidFrame();
  }

  if (maybeOutcomeStep === undefined) {
    return invalidFrame();
  }

  const outcomeStep = parseNonNegativeInteger(maybeOutcomeStep);
  if (outcomeStep > stepIndex || outcomeStep > stepLimit) {
    return invalidFrame();
  }

  if (outcome === "lost" && outcomeStep !== stepLimit) {
    return invalidFrame();
  }

  return outcomeStep;
}

/**
 * Parses the optional D-17 puzzle scalars, failing closed on partial or
 * inconsistent values. Returns undefined for scene frames.
 */
function parsePuzzleStatus(
  rawFrame: RawProofFrame,
  stepIndex: number,
  particleCount: number,
): PuzzleFrameStatus | undefined {
  const maybeGoalCount = rawFrame.goalCount?.();
  const maybeGoalTarget = rawFrame.goalTarget?.();
  const maybeGoalFull = rawFrame.goalFull?.();
  const maybeOutcome = rawFrame.outcome?.();
  const maybeOutcomeStep = rawFrame.outcomeStep?.();
  const maybeStepLimit = rawFrame.stepLimit?.();
  if (
    maybeGoalCount === undefined &&
    maybeGoalTarget === undefined &&
    maybeGoalFull === undefined &&
    maybeOutcome === undefined &&
    maybeOutcomeStep === undefined &&
    maybeStepLimit === undefined
  ) {
    return undefined;
  }

  if (
    maybeGoalCount === undefined ||
    maybeGoalTarget === undefined ||
    maybeGoalFull === undefined ||
    maybeOutcome === undefined ||
    maybeStepLimit === undefined
  ) {
    return invalidFrame();
  }

  const goalCount = parseNonNegativeInteger(maybeGoalCount);
  const goalTarget = parsePositiveInteger(maybeGoalTarget);
  const goalFull = parseNonNegativeInteger(maybeGoalFull);
  const stepLimit = parsePositiveInteger(maybeStepLimit);
  if (goalFull < goalTarget || goalCount > particleCount) {
    return invalidFrame();
  }

  const outcome = PUZZLE_OUTCOMES[parseNonNegativeInteger(maybeOutcome)];
  if (outcome === undefined) {
    return invalidFrame();
  }

  const outcomeStep = parseOutcomeStep(
    outcome,
    maybeOutcomeStep,
    stepIndex,
    stepLimit,
  );
  return {
    goalCount,
    goalTarget,
    goalFull,
    outcome,
    ...(outcomeStep === undefined ? {} : { maybeOutcomeStep: outcomeStep }),
    stepLimit,
  };
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
  const maxSpeed = parseNonNegativeFinite(rawFrame.maxSpeed());
  const stuckCandidateCount = parseNonNegativeInteger(
    rawFrame.stuckCandidateCount(),
  );
  const bodyContactCount = parseNonNegativeInteger(
    rawFrame.bodyContactCount(),
  );

  const particlePositions = rawFrame.particlePositions();
  const particleColors = rawFrame.particleColors();
  const particleRadii = rawFrame.particleRadii();
  const rigidSegments = rawFrame.rigidSegments();
  const rigidCircles = rawFrame.rigidCircles();
  const circleLabels = rawFrame.circleLabels();

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
  if (
    !Array.isArray(circleLabels) ||
    circleLabels.length !== rigidCircleCount ||
    circleLabels.some((label) => typeof label !== "string")
  ) {
    return invalidFrame();
  }

  const maybePuzzle = parsePuzzleStatus(rawFrame, stepIndex, particleCount);
  return {
    stepIndex,
    particleCount,
    rigidShapeCount,
    maxSpeed,
    stuckCandidateCount,
    bodyContactCount,
    particlePositions,
    particleColors,
    particleRadii,
    rigidSegments,
    rigidCircles,
    circleLabels,
    ...(maybePuzzle === undefined ? {} : { maybePuzzle }),
  };
}

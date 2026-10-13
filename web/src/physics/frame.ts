import { parsePuzzleStatus } from "./frame-puzzle";
import { parsePuzzleSound } from "./frame-sound";
import { parseParticleSpeeds } from "./frame-speeds";
import { invalidFrame, parseNonNegativeInteger } from "./frame-scalars";

const MAX_PARTICLE_COUNT = 16384;
const MAX_RIGID_SEGMENT_COUNT = 512;
const MAX_RIGID_CIRCLE_COUNT = 48;
const PARTICLE_POSITION_STRIDE = 2;
const PARTICLE_COLOR_STRIDE = 4;
const PARTICLE_RADIUS_STRIDE = 1;
const RIGID_SEGMENT_STRIDE = 4;
const RIGID_CIRCLE_STRIDE = 3;

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
  /** Largest count held for the settle window at or before the step limit (D-08). */
  peakHeldCount?(): number | undefined;
  /** The level's Brim count (D-09). */
  brimCount?(): number | undefined;
  /** Step Brim latched; undefined until it latches (D-09). */
  brimStep?(): number | undefined;
  /** Step the run was decided; undefined until decided (D-10). */
  decidedStep?(): number | undefined;
  /** Read-only sound aggregates for the last fixed step (Phase 42 D-07); undefined on scene sessions. */
  splashContacts?(): number | undefined;
  impactCount?(): number | undefined;
  impactImpulse?(): number | undefined;
  /** Per-particle speeds in m/s when the capture asked for them (Phase 43 D-10); undefined otherwise. */
  particleSpeeds?(): Float32Array | undefined;
  free(): void;
}

/** Puzzle outcome latched per fixed step in Rust. */
export type PuzzleOutcome = "running" | "won" | "lost";

/** Per-step puzzle readout decided in Rust (D-17, D-11). Absent on scene frames. */
export interface PuzzleFrameStatus {
  readonly goalCount: number;
  readonly goalTarget: number;
  readonly goalFull: number;
  readonly outcome: PuzzleOutcome;
  readonly maybeOutcomeStep?: number;
  readonly stepLimit: number;
  /**
   * Peak held count: the largest count held for the settle window at or
   * before the step limit (D-08). Never bounded by the live particle count.
   */
  readonly peakHeldCount: number;
  /** Count the peak held count must reach for the Brim star (D-09). */
  readonly brimCount: number;
  /** Step Brim latched; present only on won runs that reached Brim (D-09). */
  readonly maybeBrimStep?: number;
  /** Step the run was decided; present only once decided (D-10). */
  readonly maybeDecidedStep?: number;
}

/** Read-only per-step sound aggregates on puzzle frames (Phase 42 D-07). */
export interface PuzzleFrameSound {
  /** Particle-fixture contacts present this step that were absent the previous step. */
  readonly splashContacts: number;
  /** Non-sensor rigid contacts that began touching this step. */
  readonly impactCount: number;
  /** Largest summed normal impulse among those contacts (N·s); 0 when none. */
  readonly impactImpulse: number;
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
  /** Present on every puzzle frame, absent on scene frames (Phase 42 D-07). */
  readonly maybeSound?: PuzzleFrameSound;
  /** Per-particle speeds in m/s, index-aligned with particlePositions; present only while Water is requested (Phase 43 D-11). Never feeds physics. */
  readonly maybeParticleSpeeds?: Float32Array;
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
  const maybeSound = parsePuzzleSound(rawFrame, maybePuzzle);
  const maybeParticleSpeeds = parseParticleSpeeds(rawFrame, particleCount);
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
    ...(maybeSound === undefined ? {} : { maybeSound }),
    ...(maybeParticleSpeeds === undefined ? {} : { maybeParticleSpeeds }),
  };
}

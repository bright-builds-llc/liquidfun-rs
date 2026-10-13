import { describe, expect, it } from "vitest";

import { parseRenderFrame, type RawProofFrame } from "../src/physics/frame";

const PARTICLE_COUNT = 4;
const INVALID_FRAME_MESSAGE = "Invalid Rust/WASM frame";

type SoundValues = {
  splashContacts: number | undefined;
  impactCount: number | undefined;
  impactImpulse: number | undefined;
};

function sceneRawFrame(): RawProofFrame {
  return {
    stepIndex: () => 10,
    particleCount: () => PARTICLE_COUNT,
    rigidShapeCount: () => 0,
    maxSpeed: () => 0.5,
    stuckCandidateCount: () => 0,
    bodyContactCount: () => 0,
    particlePositions: () =>
      new Float32Array([0, 0, 0.1, 0, 0.2, 0, 0.3, 0]),
    particleColors: () => new Uint8Array(PARTICLE_COUNT * 4).fill(200),
    particleRadii: () => new Float32Array(PARTICLE_COUNT).fill(0.05),
    rigidSegments: () => new Float32Array(),
    rigidCircles: () => new Float32Array(),
    circleLabels: () => [],
    free: () => undefined,
  };
}

/** A running puzzle frame at step 10: target 3, brim 4, full 4. */
function puzzleWithoutSound(): RawProofFrame {
  return {
    ...sceneRawFrame(),
    goalCount: () => 2,
    goalTarget: () => 3,
    goalFull: () => 4,
    outcome: () => 0,
    outcomeStep: () => undefined,
    stepLimit: () => 600,
    peakHeldCount: () => 0,
    brimCount: () => 4,
    brimStep: () => undefined,
    decidedStep: () => undefined,
  };
}

function withSound(base: RawProofFrame, values: SoundValues): RawProofFrame {
  return {
    ...base,
    splashContacts: () => values.splashContacts,
    impactCount: () => values.impactCount,
    impactImpulse: () => values.impactImpulse,
  };
}

function puzzleRawFrame(overrides: Partial<SoundValues> = {}): RawProofFrame {
  return withSound(puzzleWithoutSound(), {
    splashContacts: 3,
    impactCount: 1,
    impactImpulse: 0.25,
    ...overrides,
  });
}

describe("parseRenderFrame sound scalars", () => {
  it("scene frame without sound getters has no maybeSound", () => {
    // Arrange
    const rawFrame = sceneRawFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect("maybeSound" in frame).toBe(false);
  });

  it("puzzle frame parses its sound scalars", () => {
    // Arrange
    const rawFrame = puzzleRawFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybeSound).toStrictEqual({
      splashContacts: 3,
      impactCount: 1,
      impactImpulse: 0.25,
    });
  });

  it("puzzle frame whose sound getters all return undefined throws", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      splashContacts: undefined,
      impactCount: undefined,
      impactImpulse: undefined,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("puzzle frame without sound getters throws", () => {
    // Arrange
    const rawFrame = puzzleWithoutSound();

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it.each<[string, Partial<SoundValues>]>([
    ["only splash", { impactCount: undefined, impactImpulse: undefined }],
    ["only impact count", { splashContacts: undefined, impactImpulse: undefined }],
    ["only impulse", { splashContacts: undefined, impactCount: undefined }],
    ["missing splash", { splashContacts: undefined }],
    ["missing impact count", { impactCount: undefined }],
    ["missing impulse", { impactImpulse: undefined }],
  ])("partial sound set (%s) throws", (_label, overrides) => {
    // Arrange
    const rawFrame = puzzleRawFrame(overrides);

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("sound getters on a frame without puzzle scalars throw", () => {
    // Arrange
    const rawFrame = withSound(sceneRawFrame(), {
      splashContacts: 0,
      impactCount: 0,
      impactImpulse: 0,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it.each<[string, Partial<SoundValues>]>([
    ["negative splash", { splashContacts: -1 }],
    ["non-integer splash", { splashContacts: 1.5 }],
    ["NaN splash", { splashContacts: Number.NaN }],
    ["negative impact count", { impactCount: -1 }],
    ["non-integer impact count", { impactCount: 0.5 }],
    ["NaN impact count", { impactCount: Number.NaN }],
    ["negative impulse", { impactImpulse: -0.25 }],
    ["NaN impulse", { impactImpulse: Number.NaN }],
    ["infinite impulse", { impactImpulse: Number.POSITIVE_INFINITY }],
    ["impulse without an impact", { impactCount: 0, impactImpulse: 0.5 }],
  ])("invalid sound value (%s) throws", (_label, overrides) => {
    // Arrange
    const rawFrame = puzzleRawFrame(overrides);

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("zero impacts with zero impulse parse", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ impactCount: 0, impactImpulse: 0 });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybeSound).toStrictEqual({
      splashContacts: 3,
      impactCount: 0,
      impactImpulse: 0,
    });
  });
});

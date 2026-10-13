import { describe, expect, it } from "vitest";

import { parseRenderFrame, type RawProofFrame } from "../src/physics/frame";

const PARTICLE_COUNT = 2;
const INVALID_FRAME_MESSAGE = "Invalid Rust/WASM frame";

function sceneRawFrame(): RawProofFrame {
  return {
    stepIndex: () => 10,
    particleCount: () => PARTICLE_COUNT,
    rigidShapeCount: () => 0,
    maxSpeed: () => 1.5,
    stuckCandidateCount: () => 0,
    bodyContactCount: () => 0,
    particlePositions: () => new Float32Array([0, 0, 0.1, 0]),
    particleColors: () => new Uint8Array(PARTICLE_COUNT * 4).fill(200),
    particleRadii: () => new Float32Array(PARTICLE_COUNT).fill(0.05),
    rigidSegments: () => new Float32Array(),
    rigidCircles: () => new Float32Array(),
    circleLabels: () => [],
    free: () => undefined,
  };
}

function withSpeeds(lane: unknown): RawProofFrame {
  return {
    ...sceneRawFrame(),
    particleSpeeds: () => lane as Float32Array | undefined,
  };
}

describe("parseRenderFrame particle speeds", () => {
  it("frame without a particleSpeeds getter has no maybeParticleSpeeds", () => {
    // Arrange
    const raw = sceneRawFrame();

    // Act
    const frame = parseRenderFrame(raw);

    // Assert
    expect("maybeParticleSpeeds" in frame).toBe(false);
  });

  it("getter returning undefined leaves maybeParticleSpeeds absent", () => {
    // Arrange
    const raw = withSpeeds(undefined);

    // Act
    const frame = parseRenderFrame(raw);

    // Assert
    expect("maybeParticleSpeeds" in frame).toBe(false);
  });

  it("valid lane parses to the same Float32Array", () => {
    // Arrange
    const lane = new Float32Array([0, 1.5]);
    const raw = withSpeeds(lane);

    // Act
    const frame = parseRenderFrame(raw);

    // Assert
    expect(frame.maybeParticleSpeeds).toBe(lane);
    expect(Array.from(frame.maybeParticleSpeeds ?? [])).toEqual([0, 1.5]);
  });

  it.each([
    ["a plain array", [0, 1.5]],
    ["a Float64Array", new Float64Array([0, 1.5])],
    ["a shorter lane", new Float32Array([0])],
    ["a longer lane", new Float32Array([0, 1.5, 2])],
    ["NaN", new Float32Array([0, Number.NaN])],
    ["Infinity", new Float32Array([Number.POSITIVE_INFINITY, 0])],
    ["a negative speed", new Float32Array([0, -0.5])],
  ])("rejects %s", (_label, lane) => {
    // Arrange
    const raw = withSpeeds(lane);

    // Act
    const parse = () => parseRenderFrame(raw);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });
});

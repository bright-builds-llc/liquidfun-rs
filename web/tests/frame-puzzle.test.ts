import { describe, expect, it } from "vitest";

import { parseRenderFrame, type RawProofFrame } from "../src/physics/frame";

const PARTICLE_COUNT = 4;
const INVALID_FRAME_MESSAGE = "Invalid Rust/WASM frame";

type PuzzleValues = {
  stepIndex: number;
  goalCount: number | undefined;
  goalTarget: number | undefined;
  goalFull: number | undefined;
  outcome: number | undefined;
  outcomeStep: number | undefined;
  stepLimit: number | undefined;
};

function defaultPuzzleValues(): PuzzleValues {
  return {
    stepIndex: 10,
    goalCount: 2,
    goalTarget: 3,
    goalFull: 4,
    outcome: 0,
    outcomeStep: undefined,
    stepLimit: 600,
  };
}

function sceneRawFrame(stepIndex: number): RawProofFrame {
  return {
    stepIndex: () => stepIndex,
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

function puzzleRawFrame(overrides: Partial<PuzzleValues> = {}): RawProofFrame {
  const values = { ...defaultPuzzleValues(), ...overrides };
  return {
    ...sceneRawFrame(values.stepIndex),
    goalCount: () => values.goalCount,
    goalTarget: () => values.goalTarget,
    goalFull: () => values.goalFull,
    outcome: () => values.outcome,
    outcomeStep: () => values.outcomeStep,
    stepLimit: () => values.stepLimit,
  };
}

describe("parseRenderFrame puzzle scalars", () => {
  it("scene frame without puzzle methods has no maybePuzzle", () => {
    // Arrange
    const rawFrame = sceneRawFrame(10);

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect("maybePuzzle" in frame).toBe(false);
  });

  it("puzzle methods all returning undefined is a scene frame", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      goalCount: undefined,
      goalTarget: undefined,
      goalFull: undefined,
      outcome: undefined,
      outcomeStep: undefined,
      stepLimit: undefined,
    });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect("maybePuzzle" in frame).toBe(false);
  });

  it("running puzzle frame parses", () => {
    // Arrange
    const rawFrame = puzzleRawFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toStrictEqual({
      goalCount: 2,
      goalTarget: 3,
      goalFull: 4,
      outcome: "running",
      stepLimit: 600,
    });
  });

  it("won puzzle frame carries its outcome step", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ outcome: 1, outcomeStep: 9 });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toMatchObject({
      outcome: "won",
      maybeOutcomeStep: 9,
    });
  });

  it("lost puzzle frame latches at the step limit", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      stepIndex: 600,
      outcome: 2,
      outcomeStep: 600,
    });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toMatchObject({
      outcome: "lost",
      maybeOutcomeStep: 600,
    });
  });

  it("rejects partial puzzle scalars", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ goalFull: undefined });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects an outcome code outside 0..2", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ outcome: 3 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects an outcome step while running", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ outcome: 0, outcomeStep: 5 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a win without an outcome step", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ outcome: 1, outcomeStep: undefined });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a loss whose outcome step is not the step limit", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      stepIndex: 600,
      outcome: 2,
      outcomeStep: 599,
      stepLimit: 600,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a goal target above the full count", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ goalTarget: 5, goalFull: 4 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a zero goal target", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ goalTarget: 0 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a goal count above the particle count", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ goalCount: PARTICLE_COUNT + 1 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects an outcome step after the frame step", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      stepIndex: 10,
      outcome: 1,
      outcomeStep: 11,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a zero step limit", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ stepLimit: 0 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects a non-integer goal count", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({ goalCount: 1.5 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });
});

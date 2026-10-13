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
  peakHeldCount: number | undefined;
  brimCount: number | undefined;
  brimStep: number | undefined;
  decidedStep: number | undefined;
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
    peakHeldCount: 0,
    brimCount: 4,
    brimStep: undefined,
    decidedStep: undefined,
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
    peakHeldCount: () => values.peakHeldCount,
    brimCount: () => values.brimCount,
    brimStep: () => values.brimStep,
    decidedStep: () => values.decidedStep,
    splashContacts: () => 0,
    impactCount: () => 0,
    impactImpulse: () => 0,
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
    const rawFrame: RawProofFrame = {
      ...puzzleRawFrame({
        goalCount: undefined,
        goalTarget: undefined,
        goalFull: undefined,
        outcome: undefined,
        outcomeStep: undefined,
        stepLimit: undefined,
        peakHeldCount: undefined,
        brimCount: undefined,
      }),
      splashContacts: () => undefined,
      impactCount: () => undefined,
      impactImpulse: () => undefined,
    };

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
      peakHeldCount: 0,
      brimCount: 4,
    });
  });

  it("won puzzle frame carries its outcome step", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      outcome: 1,
      outcomeStep: 9,
      peakHeldCount: 3,
    });

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
      decidedStep: 600,
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

const FIRST_POUR_STARS: Partial<PuzzleValues> = {
  goalTarget: 540,
  goalFull: 900,
  stepLimit: 720,
  peakHeldCount: 120,
  brimCount: 810,
};

function starRawFrame(overrides: Partial<PuzzleValues> = {}): RawProofFrame {
  return puzzleRawFrame({ ...FIRST_POUR_STARS, ...overrides });
}

const WON_WITHOUT_BRIM: Partial<PuzzleValues> = {
  stepIndex: 230,
  outcome: 1,
  outcomeStep: 201,
  peakHeldCount: 600,
};

const WON_AT_BRIM: Partial<PuzzleValues> = {
  stepIndex: 262,
  outcome: 1,
  outcomeStep: 201,
  peakHeldCount: 811,
  brimStep: 262,
  decidedStep: 262,
};

const LOST_AT_LIMIT: Partial<PuzzleValues> = {
  stepIndex: 720,
  outcome: 2,
  outcomeStep: 720,
  peakHeldCount: 0,
  decidedStep: 720,
};

describe("parseRenderFrame D-11 star scalars", () => {
  it("running frame carries peak held and brim counts without steps", () => {
    // Arrange
    const rawFrame = starRawFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toStrictEqual({
      goalCount: 2,
      goalTarget: 540,
      goalFull: 900,
      outcome: "running",
      stepLimit: 720,
      peakHeldCount: 120,
      brimCount: 810,
    });
  });

  it("won frame before the step limit without Brim is not decided", () => {
    // Arrange
    const rawFrame = starRawFrame(WON_WITHOUT_BRIM);

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toMatchObject({
      outcome: "won",
      peakHeldCount: 600,
    });
    expect(frame.maybePuzzle).not.toHaveProperty("maybeBrimStep");
    expect(frame.maybePuzzle).not.toHaveProperty("maybeDecidedStep");
  });

  it("won frame without Brim is decided at the step limit", () => {
    // Arrange
    const rawFrame = starRawFrame({
      ...WON_WITHOUT_BRIM,
      stepIndex: 720,
      decidedStep: 720,
    });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle?.maybeDecidedStep).toBe(720);
  });

  it("won frame with Brim is decided at the Brim step", () => {
    // Arrange
    const rawFrame = starRawFrame(WON_AT_BRIM);

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toMatchObject({
      peakHeldCount: 811,
      maybeBrimStep: 262,
      maybeDecidedStep: 262,
    });
  });

  it("lost frame is decided at the step limit", () => {
    // Arrange
    const rawFrame = starRawFrame(LOST_AT_LIMIT);

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.maybePuzzle).toMatchObject({
      outcome: "lost",
      peakHeldCount: 0,
      maybeDecidedStep: 720,
    });
  });

  it("peak held count above the live particle count still parses", () => {
    // Arrange
    const rawFrame = starRawFrame({ ...WON_WITHOUT_BRIM, peakHeldCount: 700 });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.particleCount).toBe(PARTICLE_COUNT);
    expect(frame.maybePuzzle?.peakHeldCount).toBe(700);
  });

  it.each<[string, Partial<PuzzleValues>]>([
    ["a missing peak held count", { peakHeldCount: undefined }],
    ["a missing brim count", { brimCount: undefined }],
    ["a non-integer peak held count", { peakHeldCount: 120.5 }],
    ["a brim count at the target", { brimCount: 540 }],
    ["a brim count above the full count", { brimCount: 901 }],
    ["a Brim step on a running frame", { brimStep: 100 }],
    ["a Brim step on a lost frame", { ...LOST_AT_LIMIT, brimStep: 700 }],
    ["a Brim step before the outcome step", { ...WON_AT_BRIM, brimStep: 200, decidedStep: 200 }],
    ["a Brim step after the frame step", { ...WON_AT_BRIM, brimStep: 263, decidedStep: 263 }],
    ["a Brim step after the step limit", { ...WON_AT_BRIM, stepIndex: 730, brimStep: 725, decidedStep: 725 }],
    ["a Brim step with peak below brim", { ...WON_AT_BRIM, peakHeldCount: 700 }],
    ["peak at brim without a Brim step", { ...WON_WITHOUT_BRIM, peakHeldCount: 811 }],
    ["a win with peak below the target", { ...WON_WITHOUT_BRIM, peakHeldCount: 539 }],
    ["a running frame with peak at the target", { peakHeldCount: 540 }],
    ["a decided step while running", { decidedStep: 100 }],
    ["a lost frame without a decided step", { ...LOST_AT_LIMIT, decidedStep: undefined }],
    ["a decided step other than the Brim step", { ...WON_AT_BRIM, stepIndex: 300, decidedStep: 300 }],
    ["a decided step before the step limit without Brim", { ...WON_WITHOUT_BRIM, decidedStep: 230 }],
    ["a won frame past the limit without a decided step", { ...WON_WITHOUT_BRIM, stepIndex: 730 }],
  ])("rejects %s", (_name, overrides) => {
    // Arrange
    const rawFrame = starRawFrame(overrides);

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });

  it("rejects star scalars on a frame without goal fields", () => {
    // Arrange
    const rawFrame = puzzleRawFrame({
      goalCount: undefined,
      goalTarget: undefined,
      goalFull: undefined,
      outcome: undefined,
      stepLimit: undefined,
      peakHeldCount: 0,
      brimCount: 810,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow(INVALID_FRAME_MESSAGE);
  });
});

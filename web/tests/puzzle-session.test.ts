import { describe, expect, it, vi } from "vitest";

import type { RawProofFrame } from "../src/physics/frame";
import {
  createPuzzleSession,
  PUZZLE_SCENE_INPUT_MESSAGE,
  type GeneratedPuzzleSession,
  type PuzzleSession,
  type RawPuzzleLevelInfo,
} from "../src/physics/puzzle-session";

const PARTICLE_COUNT = 4;
const FAILED_MESSAGE = "Rust/WASM session failed";
const DISPOSED_MESSAGE = "Rust/WASM session is disposed";
const INVALID_INFO_MESSAGE = "Invalid puzzle level info";

type LevelInfoValues = {
  levelId: string;
  title: string;
  targetCount: number;
  fullCount: number;
  settleSteps: number;
  stepLimit: number;
  cupInterior: Float32Array;
  buildArea: Int32Array;
  gridMeters: number;
};

function firstPourValues(): LevelInfoValues {
  return {
    levelId: "first-pour",
    title: "First Pour",
    targetCount: 540,
    fullCount: 900,
    settleSteps: 30,
    stepLimit: 720,
    cupInterior: new Float32Array([1.6, 0, 3.4, 1.8]),
    buildArea: new Int32Array([-20, 44, 40, 84]),
    gridMeters: 0.05,
  };
}

function fakeLevelInfo(overrides: Partial<LevelInfoValues> = {}) {
  const values = { ...firstPourValues(), ...overrides };
  return {
    levelId: () => values.levelId,
    title: () => values.title,
    targetCount: () => values.targetCount,
    fullCount: () => values.fullCount,
    brimCount: () => 810,
    parTools: () => 1,
    settleSteps: () => values.settleSteps,
    stepLimit: () => values.stepLimit,
    cupInterior: () => values.cupInterior,
    buildArea: () => values.buildArea,
    gridMeters: () => values.gridMeters,
    toolKinds: () => new Uint8Array([1]),
    toolCounts: () => new Uint32Array([2]),
    free: vi.fn(),
  } satisfies RawPuzzleLevelInfo;
}

function puzzleRawFrame(stepIndex: number): RawProofFrame {
  return {
    stepIndex: () => stepIndex,
    particleCount: () => PARTICLE_COUNT,
    rigidShapeCount: () => 0,
    maxSpeed: () => 0.5,
    stuckCandidateCount: () => 0,
    bodyContactCount: () => 0,
    particlePositions: () => new Float32Array([0, 0, 0.1, 0, 0.2, 0, 0.3, 0]),
    particleColors: () => new Uint8Array(PARTICLE_COUNT * 4).fill(200),
    particleRadii: () => new Float32Array(PARTICLE_COUNT).fill(0.05),
    rigidSegments: () => new Float32Array(),
    rigidCircles: () => new Float32Array(),
    circleLabels: () => [],
    goalCount: () => 0,
    goalTarget: () => 3,
    goalFull: () => 4,
    outcome: () => 0,
    outcomeStep: () => undefined,
    stepLimit: () => 720,
    peakHeldCount: () => 0,
    brimCount: () => 4,
    brimStep: () => undefined,
    decidedStep: () => undefined,
    free: vi.fn(),
  };
}

function fakeGeneratedSession(
  levelInfo: RawPuzzleLevelInfo = fakeLevelInfo(),
) {
  let stepIndex = 0;
  return {
    advance: vi.fn((stepCount: number) => {
      stepIndex += stepCount;
    }),
    captureFrame: vi.fn(() => puzzleRawFrame(stepIndex)),
    applyControl: vi.fn(() => true),
    applyAction: vi.fn(),
    pointerAction: vi.fn(),
    setGravity: vi.fn(),
    restoreAuthoredGravity: vi.fn(),
    levelInfo: vi.fn(() => levelInfo),
    free: vi.fn(),
  } satisfies GeneratedPuzzleSession;
}

describe("createPuzzleSession frames", () => {
  it("captureFrame returns the untouched step-0 frame without advancing", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    const frame = session.captureFrame();

    // Assert
    expect(frame.stepIndex).toBe(0);
    expect(frame.maybePuzzle?.outcome).toBe("running");
    expect(generated.advance).not.toHaveBeenCalled();
  });

  it("nextFrame(1) advances one step then captures once", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    const frame = session.nextFrame(1);

    // Assert
    expect(generated.advance).toHaveBeenCalledWith(1);
    expect(generated.captureFrame).toHaveBeenCalledTimes(1);
    expect(frame.stepIndex).toBe(1);
  });

  it("nextFrame accepts four steps", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    const frame = session.nextFrame(4);

    // Assert
    expect(frame.stepIndex).toBe(4);
  });

  it.each([0, 5, 1.5])(
    "nextFrame(%s) poisons the session and frees it once",
    (stepCount) => {
      // Arrange
      const generated = fakeGeneratedSession();
      const session = createPuzzleSession(generated, "first-pour");

      // Act
      const act = () => session.nextFrame(stepCount);

      // Assert
      expect(act).toThrow(FAILED_MESSAGE);
      expect(generated.advance).not.toHaveBeenCalled();
      expect(generated.free).toHaveBeenCalledTimes(1);
      expect(() => session.captureFrame()).toThrow(DISPOSED_MESSAGE);
    },
  );

  it("a throwing generated captureFrame poisons the session", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const cause = new Error("capture broke");
    generated.captureFrame.mockImplementation(() => {
      throw cause;
    });
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    const act = () => session.captureFrame();

    // Assert
    expect(act).toThrow(FAILED_MESSAGE);
    expect(generated.free).toHaveBeenCalledTimes(1);
  });

  it("frees each captured raw frame", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const rawFrame = puzzleRawFrame(0);
    generated.captureFrame.mockReturnValue(rawFrame);
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    session.captureFrame();

    // Assert
    expect(rawFrame.free).toHaveBeenCalledTimes(1);
  });
});

describe("createPuzzleSession scene input", () => {
  const sceneInputs: ReadonlyArray<
    [string, (session: PuzzleSession) => unknown]
  > = [
    ["applyControl", (session) => session.applyControl("speed", "1")],
    ["applyAction", (session) => session.applyAction("reset")],
    ["pointerAction", (session) => session.pointerAction("poke", 0, 0)],
    ["setGravity", (session) => session.setGravity(0, -10)],
    ["restoreAuthoredGravity", (session) => session.restoreAuthoredGravity()],
  ];

  it.each(sceneInputs)("%s throws without calling wasm", (name, call) => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    const act = () => call(session);

    // Assert
    expect(act).toThrow(PUZZLE_SCENE_INPUT_MESSAGE);
    expect(generated[name as keyof typeof generated]).not.toHaveBeenCalled();
  });
});

describe("createPuzzleSession level info", () => {
  it("parses first-pour level info into meters", () => {
    // Arrange
    const generated = fakeGeneratedSession();

    // Act
    const { levelInfo } = createPuzzleSession(generated, "first-pour");

    // Assert
    expect(levelInfo).toMatchObject({
      levelId: "first-pour",
      title: "First Pour",
      targetCount: 540,
      fullCount: 900,
      settleSteps: 30,
      stepLimit: 720,
    });
    expect(levelInfo.gridMeters).toBeCloseTo(0.05);
    expect(levelInfo.cupInterior.minX).toBeCloseTo(1.6);
    expect(levelInfo.cupInterior.minY).toBe(0);
    expect(levelInfo.cupInterior.maxX).toBeCloseTo(3.4);
    expect(levelInfo.cupInterior.maxY).toBeCloseTo(1.8);
    expect(levelInfo.buildArea.minX).toBeCloseTo(-1);
    expect(levelInfo.buildArea.minY).toBeCloseTo(2.2);
    expect(levelInfo.buildArea.maxX).toBeCloseTo(2);
    expect(levelInfo.buildArea.maxY).toBeCloseTo(4.2);
  });

  it("frees the raw level info exactly once", () => {
    // Arrange
    const rawInfo = fakeLevelInfo();
    const generated = fakeGeneratedSession(rawInfo);

    // Act
    createPuzzleSession(generated, "first-pour");

    // Assert
    expect(rawInfo.free).toHaveBeenCalledTimes(1);
    expect(generated.free).not.toHaveBeenCalled();
  });

  it("a level id mismatch fails closed and frees both objects", () => {
    // Arrange
    const rawInfo = fakeLevelInfo({ levelId: "other" });
    const generated = fakeGeneratedSession(rawInfo);

    // Act
    const act = () => createPuzzleSession(generated, "first-pour");

    // Assert
    expect(act).toThrow(INVALID_INFO_MESSAGE);
    expect(rawInfo.free).toHaveBeenCalledTimes(1);
    expect(generated.free).toHaveBeenCalledTimes(1);
  });

  const invalidCases: ReadonlyArray<[string, Partial<LevelInfoValues>]> = [
    ["target above full", { targetCount: 901 }],
    ["zero full count", { fullCount: 0 }],
    ["NaN in cup interior", { cupInterior: new Float32Array([1.6, Number.NaN, 3.4, 1.8]) }],
    ["short cup interior", { cupInterior: new Float32Array([1.6, 0, 3.4]) }],
    ["inverted cup interior", { cupInterior: new Float32Array([3.4, 0, 1.6, 1.8]) }],
    ["inverted build area", { buildArea: new Int32Array([40, 44, -20, 84]) }],
    ["short build area", { buildArea: new Int32Array([-20, 44, 40]) }],
    ["zero grid size", { gridMeters: 0 }],
    ["empty title", { title: "" }],
    ["fractional step limit", { stepLimit: 1.5 }],
  ];

  it.each(invalidCases)("%s throws invalid level info", (_name, overrides) => {
    // Arrange
    const rawInfo = fakeLevelInfo(overrides);
    const generated = fakeGeneratedSession(rawInfo);

    // Act
    const act = () => createPuzzleSession(generated, "first-pour");

    // Assert
    expect(act).toThrow(INVALID_INFO_MESSAGE);
    expect(rawInfo.free).toHaveBeenCalledTimes(1);
    expect(generated.free).toHaveBeenCalledTimes(1);
  });
});

describe("createPuzzleSession dispose", () => {
  it("rejects frames after dispose", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    session.dispose();

    // Assert
    expect(() => session.captureFrame()).toThrow(DISPOSED_MESSAGE);
    expect(() => session.nextFrame()).toThrow(DISPOSED_MESSAGE);
  });

  it("a second dispose is a no-op", () => {
    // Arrange
    const generated = fakeGeneratedSession();
    const session = createPuzzleSession(generated, "first-pour");

    // Act
    session.dispose();
    session.dispose();

    // Assert
    expect(generated.free).toHaveBeenCalledTimes(1);
  });
});

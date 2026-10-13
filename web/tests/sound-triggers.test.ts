import { describe, expect, it } from "vitest";

import { NO_SOUND_SIGNALS, type PresentedSoundSignals } from "../src/audio/sound-signals";
import {
  BURST_QUIET_SECONDS,
  INITIAL_TRIGGER_MEMORY,
  decideVoices,
  trackWithoutClock,
  type TriggerMemory,
} from "../src/audio/triggers";
import {
  IMPACT_FULL_IMPULSE,
  IMPACT_MIN_STRENGTH,
  SPLASH_FULL_CONTACTS_PER_STEP,
  SPLASH_MIN_CONTACTS_PER_STEP,
  impactStrength,
  plinkFrequency,
  splashIntensity,
} from "../src/audio/voices";
import type { PuzzleFrameStatus, RenderFrame } from "../src/physics/frame";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import { INITIAL_PUZZLE_STATE, type PuzzleLatch, type PuzzleState } from "../src/puzzle/reduce-puzzle";

const GOAL_TARGET = 540;
const FAST_MPS = 2;

function status(overrides: Partial<PuzzleFrameStatus> = {}): PuzzleFrameStatus {
  return {
    goalCount: 0,
    goalTarget: GOAL_TARGET,
    goalFull: 900,
    outcome: "running",
    stepLimit: 720,
    peakHeldCount: 0,
    brimCount: 810,
    ...overrides,
  };
}

const WON_STATUS = status({ outcome: "won", maybeOutcomeStep: 201, goalCount: 600, peakHeldCount: 541 });

function latch(outcome: "won" | "lost"): PuzzleLatch {
  return {
    outcome,
    outcomeStep: outcome === "won" ? 201 : 720,
    decidedStep: outcome === "won" ? 262 : 720,
    goalCount: outcome === "won" ? 900 : 0,
    peakHeldCount: outcome === "won" ? 811 : 0,
    maybeBrimStep: outcome === "won" ? 262 : undefined,
    goalTarget: GOAL_TARGET,
    goalFull: 900,
    brimCount: 810,
    stepLimit: 720,
  };
}

function running(stepIndex: number, frameStatus = status()): PuzzleState {
  return { kind: "running", stepIndex, status: frameStatus, editor: EMPTY_EDITOR, speed: 1 };
}

function ended(stepIndex: number, outcome: "won" | "lost", held: boolean): PuzzleState {
  return {
    kind: "ended",
    stepIndex,
    latch: latch(outcome),
    tailEndStep: stepIndex + 100,
    held,
    editor: EMPTY_EDITOR,
    speed: 1,
  };
}

function building(): PuzzleState {
  return { kind: "building", status: status(), editor: EMPTY_EDITOR, speed: 1 };
}

function frame(stepIndex: number, goalCount = 0, frameStatus = status({ goalCount })): RenderFrame {
  return {
    stepIndex,
    particleCount: 0,
    rigidShapeCount: 0,
    maxSpeed: FAST_MPS,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array(),
    particleColors: new Uint8Array(),
    particleRadii: new Float32Array(),
    rigidSegments: new Float32Array(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
    maybePuzzle: frameStatus,
  };
}

function signals(overrides: Partial<PresentedSoundSignals> = {}): PresentedSoundSignals {
  return { ...NO_SOUND_SIGNALS, steps: 1, ...overrides };
}

const LOUD = signals({ splashContacts: SPLASH_FULL_CONTACTS_PER_STEP * 2 });
const SPLASH = signals({ splashContacts: SPLASH_MIN_CONTACTS_PER_STEP + 4 });

function memoryAt(stepIndex: number, overrides: Partial<TriggerMemory> = {}): TriggerMemory {
  return { ...INITIAL_TRIGGER_MEMORY, maybeLastStepIndex: stepIndex, ...overrides };
}

describe("decideVoices outside a run", () => {
  it.each([
    ["building", building],
    ["loading", () => INITIAL_PUZZLE_STATE],
  ] as const)("is silent while %s and resets the memory", (_, makeState) => {
    // Act
    const decision = decideVoices(memoryAt(40, { lastGoalCount: 12, wasWon: true }), {
      state: makeState(),
      frame: frame(0),
      signals: LOUD,
      nowSeconds: 3,
    });

    // Assert
    expect(decision).toEqual({ memory: INITIAL_TRIGGER_MEMORY, requests: [] });
  });

  it("is silent once the ended outcome is held", () => {
    // Act
    const decision = decideVoices(memoryAt(400), {
      state: ended(401, "lost", true),
      frame: frame(401, 20),
      signals: LOUD,
      nowSeconds: 3,
    });

    // Assert
    expect(decision.requests).toEqual([]);
  });
});

describe("decideVoices plinks", () => {
  it("requests one plink at the new fill when the goal count rises", () => {
    // Act
    const decision = decideVoices(memoryAt(50, { lastGoalCount: 10 }), {
      state: running(51),
      frame: frame(51, 14),
      signals: signals(),
      nowSeconds: 1,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "plink", frequency: plinkFrequency(14, GOAL_TARGET) }]);
  });

  it.each([10, 7])("requests no plink when the goal count goes from 10 to %i", (goalCount) => {
    // Act
    const decision = decideVoices(memoryAt(50, { lastGoalCount: 10 }), {
      state: running(51),
      frame: frame(51, goalCount),
      signals: signals(),
      nowSeconds: 1,
    });

    // Assert
    expect(decision.requests).toEqual([]);
  });
});

describe("decideVoices win chime", () => {
  it("requests only the chime on the first won frame", () => {
    // Act
    const decision = decideVoices(memoryAt(200, { lastGoalCount: 500 }), {
      state: running(201, WON_STATUS),
      frame: frame(201, 600, WON_STATUS),
      signals: { ...LOUD, impactCount: 2, impactImpulse: 1 },
      nowSeconds: 4,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "chime" }]);
  });

  it("is silent on later frames of the same won run", () => {
    // Arrange
    const first = decideVoices(memoryAt(200), {
      state: running(201, WON_STATUS),
      frame: frame(201, 600, WON_STATUS),
      signals: LOUD,
      nowSeconds: 4,
    });
    const deciding = decideVoices(first.memory, {
      state: running(202, WON_STATUS),
      frame: frame(202, 650, WON_STATUS),
      signals: LOUD,
      nowSeconds: 4.5,
    });

    // Act
    const tail = decideVoices(deciding.memory, {
      state: ended(262, "won", false),
      frame: frame(262, 900, WON_STATUS),
      signals: LOUD,
      nowSeconds: 5,
    });

    // Assert
    expect([deciding.requests, tail.requests]).toEqual([[], []]);
  });

  it("chimes again after the step index goes back for a new run", () => {
    // Act
    const decision = decideVoices(memoryAt(300, { wasWon: true }), {
      state: running(201, WON_STATUS),
      frame: frame(201, 600, WON_STATUS),
      signals: signals(),
      nowSeconds: 9,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "chime" }]);
  });

  it("treats a repeated step index as a new run", () => {
    // Act
    const decision = decideVoices(memoryAt(201, { wasWon: true, lastGoalCount: 600 }), {
      state: running(201, WON_STATUS),
      frame: frame(201, 600, WON_STATUS),
      signals: signals(),
      nowSeconds: 9,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "chime" }]);
  });
});

describe("decideVoices splash and impact", () => {
  it("keeps splash and plink voices in a lost settle tail", () => {
    // Act
    const decision = decideVoices(memoryAt(720, { lastGoalCount: 3, maybeLastSplashAt: 7.9 }), {
      state: ended(721, "lost", false),
      frame: frame(721, 5),
      signals: SPLASH,
      nowSeconds: 8,
    });

    // Assert
    expect(decision.requests.map((request) => request.kind)).toEqual(["splash", "plink"]);
  });

  it("requests a splash at the mapped intensity", () => {
    // Act
    const decision = decideVoices(memoryAt(10, { maybeLastSplashAt: 0.9 }), {
      state: running(11),
      frame: frame(11),
      signals: SPLASH,
      nowSeconds: 1,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "splash", intensity: splashIntensity(SPLASH, FAST_MPS) }]);
  });

  it("adds a burst impact to the first splash after a quiet interval", () => {
    // Arrange
    const intensity = splashIntensity(SPLASH, FAST_MPS) ?? 0;

    // Act
    const decision = decideVoices(memoryAt(10, { maybeLastSplashAt: 1 }), {
      state: running(11),
      frame: frame(11),
      signals: SPLASH,
      nowSeconds: 1 + BURST_QUIET_SECONDS,
    });

    // Assert
    expect(decision.requests).toEqual([
      { kind: "impact", strength: Math.max(IMPACT_MIN_STRENGTH, intensity) },
      { kind: "splash", intensity },
    ]);
  });

  it("adds no burst impact to a splash soon after another", () => {
    // Arrange
    const first = decideVoices(INITIAL_TRIGGER_MEMORY, {
      state: running(11),
      frame: frame(11),
      signals: SPLASH,
      nowSeconds: 2,
    });

    // Act
    const second = decideVoices(first.memory, {
      state: running(12),
      frame: frame(12),
      signals: SPLASH,
      nowSeconds: 2.1,
    });

    // Assert
    expect(second.requests.map((request) => request.kind)).toEqual(["splash"]);
  });

  it("requests a rigid impact at its mapped strength", () => {
    // Arrange
    const rigid = signals({ impactCount: 1, impactImpulse: IMPACT_FULL_IMPULSE / 2 });

    // Act
    const decision = decideVoices(memoryAt(10), {
      state: running(11),
      frame: frame(11),
      signals: rigid,
      nowSeconds: 1,
    });

    // Assert
    expect(decision.requests).toEqual([{ kind: "impact", strength: impactStrength(rigid) }]);
  });

  it("merges a rigid impact and a burst onset into one stronger impact", () => {
    // Arrange
    const both = { ...SPLASH, impactCount: 1, impactImpulse: IMPACT_FULL_IMPULSE };

    // Act
    const decision = decideVoices(memoryAt(10), {
      state: running(11),
      frame: frame(11),
      signals: both,
      nowSeconds: 5,
    });

    // Assert
    const impacts = decision.requests.filter((request) => request.kind === "impact");
    expect(impacts).toEqual([{ kind: "impact", strength: 1 }]);
  });

  it("orders requests impact, splash, plink", () => {
    // Act
    const decision = decideVoices(memoryAt(10, { lastGoalCount: 1 }), {
      state: running(11),
      frame: frame(11, 2),
      signals: SPLASH,
      nowSeconds: 5,
    });

    // Assert
    expect(decision.requests.map((request) => request.kind)).toEqual(["impact", "splash", "plink"]);
  });

  it("records the step, goal count and splash time", () => {
    // Act
    const decision = decideVoices(memoryAt(10, { lastGoalCount: 1 }), {
      state: running(11),
      frame: frame(11, 2),
      signals: SPLASH,
      nowSeconds: 5,
    });

    // Assert
    expect(decision.memory).toEqual({
      maybeLastStepIndex: 11,
      lastGoalCount: 2,
      wasWon: false,
      maybeLastSplashAt: 5,
    });
  });
});

describe("trackWithoutClock", () => {
  it("tracks the step and goal count but not the splash time", () => {
    // Act
    const memory = trackWithoutClock(memoryAt(10, { lastGoalCount: 3, maybeLastSplashAt: 1.5 }), {
      state: running(11),
      frame: frame(11, 9),
    });

    // Assert
    expect(memory).toEqual({ maybeLastStepIndex: 11, lastGoalCount: 9, wasWon: false, maybeLastSplashAt: 1.5 });
  });

  it("leaves an unset splash time unset", () => {
    // Act
    const memory = trackWithoutClock(INITIAL_TRIGGER_MEMORY, { state: running(1), frame: frame(1, 4) });

    // Assert
    expect(memory.maybeLastSplashAt).toBeUndefined();
  });

  it("marks the win so a later won frame does not chime", () => {
    // Arrange
    const tracked = trackWithoutClock(memoryAt(200), {
      state: running(201, WON_STATUS),
      frame: frame(201, 600, WON_STATUS),
    });

    // Act
    const decision = decideVoices(tracked, {
      state: running(202, WON_STATUS),
      frame: frame(202, 610, WON_STATUS),
      signals: LOUD,
      nowSeconds: 3,
    });

    // Assert
    expect([tracked.wasWon, decision.requests]).toEqual([true, []]);
  });

  it.each([
    ["building", building, 0],
    ["loading", () => INITIAL_PUZZLE_STATE, 0],
    ["a repeated step", () => running(40), 40],
  ] as const)("resets the memory for %s", (_, makeState, stepIndex) => {
    // Arrange
    const memory = memoryAt(40, { lastGoalCount: 12, wasWon: true, maybeLastSplashAt: 2 });

    // Act
    const tracked = trackWithoutClock(memory, { state: makeState(), frame: frame(stepIndex) });

    // Assert
    const expected =
      stepIndex === 0 ? INITIAL_TRIGGER_MEMORY : { ...INITIAL_TRIGGER_MEMORY, maybeLastStepIndex: 40 };
    expect(tracked).toEqual(expected);
  });
});

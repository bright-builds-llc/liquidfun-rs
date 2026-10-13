import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  MAX_STEPS_PER_FRAME,
  STEP_SECONDS,
  accumulateStepTime,
} from "../src/physics/clock";
import type { PuzzleFrameStatus, RenderFrame } from "../src/physics/frame";
import type { PuzzleSession } from "../src/physics/puzzle-session";
import { createFrameClock, type FrameClock } from "../src/player/frame-loop";
import type { FrameObservation } from "../src/player/observe";
import { schedulePuzzleFrame, type PuzzleLoopDeps } from "../src/player/puzzle-loop";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import {
  SETTLE_TAIL_STEPS,
  reducePuzzle,
  type PuzzleEvent,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";
import type { PuzzleSpeed } from "../src/puzzle/speed";
import { createCamera } from "../src/render/camera";

const LATCH_STEP = 9;
// Each frame is 100 ms apart, so the accumulator always requests the 4-step cap.
const FRAME_MS = 100;
// 35 ms makes the accumulator request two fixed steps on the first stepping frame.
const TWO_STEP_FRAME_MS = 35;

type ScriptedSteps = { readonly winStep: number; readonly decideStep: number };

type Published = { frame: RenderFrame; observation: FrameObservation; state: PuzzleState };

function frameAt(stepIndex: number, maybePuzzle: PuzzleFrameStatus | undefined): RenderFrame {
  return {
    stepIndex,
    particleCount: 1,
    rigidShapeCount: 0,
    maxSpeed: 0,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array([0, stepIndex]),
    particleColors: new Uint8Array([77, 163, 255, 255]),
    particleRadii: new Float32Array([0.05]),
    rigidSegments: new Float32Array(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
    ...(maybePuzzle === undefined ? {} : { maybePuzzle }),
  };
}

function statusAt(stepIndex: number, steps: ScriptedSteps): PuzzleFrameStatus {
  const { winStep, decideStep } = steps;
  const base = {
    goalCount: stepIndex,
    goalTarget: 540,
    goalFull: 900,
    stepLimit: 720,
    brimCount: 810,
  };
  if (stepIndex < winStep) {
    return { ...base, outcome: "running", peakHeldCount: 0 };
  }
  const won = { ...base, outcome: "won", maybeOutcomeStep: winStep } as const;
  // A win stays undecided ("Filled!") until Brim latches at the decide step (D-10).
  return stepIndex < decideStep
    ? { ...won, peakHeldCount: 600 }
    : { ...won, peakHeldCount: 810, maybeBrimStep: decideStep, maybeDecidedStep: decideStep };
}

type HarnessOptions = {
  readonly steps?: ScriptedSteps;
  readonly omitStatus?: boolean;
  readonly speed?: PuzzleSpeed;
  /** Simulated wall time each `nextFrame` call takes. */
  readonly stepCostMs?: number;
};

function fakeSession(steps: ScriptedSteps, options: HarnessOptions) {
  let step = 0;
  const nextFrame = vi.fn((count = 1) => {
    step += count;
    nowMs += options.stepCostMs ?? 0;
    return frameAt(step, options.omitStatus === true ? undefined : statusAt(step, steps));
  });
  const session = { nextFrame } as unknown as PuzzleSession;
  return { session, nextFrame };
}

function harness(options: HarnessOptions = {}) {
  const steps = options.steps ?? { winStep: LATCH_STEP, decideStep: LATCH_STEP };
  const { session, nextFrame } = fakeSession(steps, options);
  const clock: FrameClock = createFrameClock();
  clock.maybeCamera = createCamera(390, 844);
  const context = {} as CanvasRenderingContext2D;
  let state: PuzzleState = {
    kind: "running",
    stepIndex: 0,
    status: statusAt(0, steps),
    editor: EMPTY_EDITOR,
    speed: options.speed ?? 1,
  };
  let current = true;
  const published: Published[] = [];
  const dispatched: PuzzleEvent[] = [];
  const deps: PuzzleLoopDeps = {
    maybeSession: () => session,
    current: () => state,
    dispatch: (event) => {
      dispatched.push(event);
      state = reducePuzzle(state, event);
      return state;
    },
    publish: (frame, observation, publishedState) => {
      published.push({ frame, observation, state: publishedState });
    },
    drawSceneFrame: vi.fn(),
    fail: vi.fn(),
    isCurrent: () => current,
  };
  return {
    clock,
    context,
    deps,
    dispatched,
    nextFrame,
    published,
    supersede: () => {
      current = false;
    },
  };
}

let callbacks: Map<number, FrameRequestCallback>;
let rafId: number;
let timestamp: number;
let frameMs: number;
let nowMs: number;
let documentStub: { hidden: boolean };

function runFrame(): void {
  const entry = callbacks.entries().next();
  if (entry.done === true) {
    throw new Error("No animation frame is scheduled");
  }
  const [id, callback] = entry.value;
  callbacks.delete(id);
  callback(timestamp);
  timestamp += frameMs;
}

function runUntilIdle(limit: number): number {
  let frames = 0;
  while (callbacks.size > 0 && frames < limit) {
    runFrame();
    frames += 1;
  }
  return frames;
}

beforeEach(() => {
  callbacks = new Map();
  rafId = 0;
  timestamp = 0;
  frameMs = FRAME_MS;
  nowMs = 0;
  documentStub = { hidden: false };
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    rafId += 1;
    callbacks.set(rafId, callback);
    return rafId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  vi.stubGlobal("document", documentStub);
  vi.stubGlobal("performance", { now: () => nowMs });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("schedulePuzzleFrame", () => {
  it("steps the session one fixed step per call and at most four per frame", () => {
    // Arrange
    const { clock, context, deps, nextFrame } = harness();
    schedulePuzzleFrame(clock, context, deps);
    runFrame();

    // Act
    const callsBefore = nextFrame.mock.calls.length;
    runFrame();
    const callsInFrame = nextFrame.mock.calls.length - callsBefore;
    runUntilIdle(200);

    // Assert
    expect(callsInFrame).toBe(4);
    expect(nextFrame.mock.calls.every((call) => call.length === 1 && call[0] === 1)).toBe(true);
  });

  it("latches the goal count and outcome step at the outcome step while stepping continues", () => {
    // Arrange
    const { clock, context, deps, published } = harness();
    schedulePuzzleFrame(clock, context, deps);

    // Act
    runUntilIdle(200);

    // Assert
    const ended = published.map((entry) => entry.state).filter((state) => state.kind === "ended");
    expect(ended.length).toBeGreaterThan(1);
    for (const state of ended) {
      expect(state.kind === "ended" && state.latch.goalCount).toBe(LATCH_STEP);
      expect(state.kind === "ended" && state.latch.outcomeStep).toBe(LATCH_STEP);
    }
  });

  it("stops exactly at the end of the settle tail and holds without scheduling", () => {
    // Arrange
    const { clock, context, deps, nextFrame, published } = harness();
    schedulePuzzleFrame(clock, context, deps);

    // Act
    runUntilIdle(200);

    // Assert
    const last = published.at(-1);
    expect(last?.state).toMatchObject({
      kind: "ended",
      stepIndex: LATCH_STEP + SETTLE_TAIL_STEPS,
      held: true,
    });
    expect(nextFrame).toHaveBeenCalledTimes(LATCH_STEP + SETTLE_TAIL_STEPS);
    expect(callbacks.size).toBe(0);
    expect(clock.maybeAnimationFrameId).toBeUndefined();
  });

  it("publishes once per presented frame with the frame of the batch's last step", () => {
    // Arrange
    const { clock, context, deps, published } = harness();
    schedulePuzzleFrame(clock, context, deps);
    runFrame();

    // Act
    runFrame();
    runFrame();

    // Assert
    expect(published.map((entry) => entry.frame.stepIndex)).toEqual([4, 8]);
    expect(published.map((entry) => entry.observation.stepIndex)).toEqual([4, 8]);
    expect(deps.drawSceneFrame).toHaveBeenCalledTimes(2);
  });

  it("fails and stops scheduling when a frame has no goal status", () => {
    // Arrange
    const { clock, context, deps } = harness({ omitStatus: true });
    schedulePuzzleFrame(clock, context, deps);
    runFrame();

    // Act
    runFrame();

    // Assert
    expect(deps.fail).toHaveBeenCalledTimes(1);
    expect(vi.mocked(deps.fail).mock.calls[0]?.[0]).toMatchObject({
      message: "Puzzle frame is missing its goal status",
    });
    expect(callbacks.size).toBe(0);
  });

  it("returns without stepping or rescheduling once superseded", () => {
    // Arrange
    const { clock, context, deps, nextFrame, supersede } = harness();
    schedulePuzzleFrame(clock, context, deps);
    runFrame();
    supersede();

    // Act
    runFrame();

    // Assert
    expect(nextFrame).not.toHaveBeenCalled();
    expect(callbacks.size).toBe(0);
  });

  it("runs no steps while the tab is hidden and reschedules with the timestamp reset", () => {
    // Arrange
    const { clock, context, deps, nextFrame } = harness();
    schedulePuzzleFrame(clock, context, deps);
    runFrame();
    documentStub.hidden = true;

    // Act
    runFrame();

    // Assert
    expect(nextFrame).not.toHaveBeenCalled();
    expect(clock.maybeLastTimestamp).toBeUndefined();
    expect(callbacks.size).toBe(1);
  });
});

describe("schedulePuzzleFrame fast-forward (D-24)", () => {
  it("multiplies the frame's step request by the speed and dispatches every step", () => {
    // Arrange
    frameMs = TWO_STEP_FRAME_MS;
    const { clock, context, deps, dispatched, nextFrame } = harness({ speed: 4 });
    schedulePuzzleFrame(clock, context, deps);
    runFrame();

    // Act
    runFrame();

    // Assert
    expect(nextFrame).toHaveBeenCalledTimes(8);
    expect(dispatched.filter((event) => event.kind === "stepped")).toHaveLength(8);
  });

  it("ends with the same latch, decision and tail end at 1x, 2x and 4x", () => {
    // Arrange
    const steps = { winStep: 6, decideStep: 9 };
    const speeds: readonly PuzzleSpeed[] = [1, 2, 4];

    // Act
    const endings = speeds.map((speed) => {
      callbacks.clear();
      const { clock, context, deps, nextFrame, published } = harness({ steps, speed });
      schedulePuzzleFrame(clock, context, deps);
      runUntilIdle(200);
      const last = published.at(-1)?.state;
      return {
        outcomeStep: last?.kind === "ended" ? last.latch.outcomeStep : undefined,
        decidedStep: last?.kind === "ended" ? last.latch.decidedStep : undefined,
        stepIndex: last?.kind === "ended" ? last.stepIndex : undefined,
        held: last?.kind === "ended" && last.held,
        steps: nextFrame.mock.calls.length,
      };
    });

    // Assert
    const expected = {
      outcomeStep: 6,
      decidedStep: 9,
      stepIndex: 9 + SETTLE_TAIL_STEPS,
      held: true,
      steps: 9 + SETTLE_TAIL_STEPS,
    };
    expect(endings).toEqual([expected, expected, expected]);
  });

  it("restores steps the frame budget left unrun at wall-time scale", () => {
    // Arrange
    frameMs = TWO_STEP_FRAME_MS;
    // 2.5 ms per step crosses the 12 ms budget after the fifth step.
    const { clock, context, deps, nextFrame } = harness({ speed: 4, stepCostMs: 2.5 });
    schedulePuzzleFrame(clock, context, deps);
    runFrame();
    const accumulated = accumulateStepTime(0, TWO_STEP_FRAME_MS / 1000);

    // Act
    runFrame();

    // Assert
    expect(accumulated.stepCount).toBe(2);
    expect(nextFrame).toHaveBeenCalledTimes(5);
    expect(clock.frameRemainderSeconds).toBeCloseTo(
      accumulated.remainderSeconds + ((8 - 5) / 4) * STEP_SECONDS,
      12,
    );
  });

  it("captures one fixed step per call at 4x under the unchanged four-step cap", () => {
    // Arrange
    const { clock, context, deps, nextFrame } = harness({ speed: 4 });
    schedulePuzzleFrame(clock, context, deps);
    runFrame();

    // Act
    runFrame();
    const callsInFrame = nextFrame.mock.calls.length;
    runUntilIdle(200);

    // Assert
    expect(MAX_STEPS_PER_FRAME).toBe(4);
    expect(callsInFrame).toBe(MAX_STEPS_PER_FRAME * 4);
    expect(nextFrame.mock.calls.every((call) => call.length === 1 && call[0] === 1)).toBe(true);
  });
});

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { PuzzleFrameStatus, RenderFrame } from "../src/physics/frame";
import type { PuzzleSession } from "../src/physics/puzzle-session";
import { createFrameClock, type FrameClock } from "../src/player/frame-loop";
import type { FrameObservation } from "../src/player/observe";
import { schedulePuzzleFrame, type PuzzleLoopDeps } from "../src/player/puzzle-loop";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import {
  SETTLE_TAIL_STEPS,
  reducePuzzle,
  type PuzzleState,
} from "../src/puzzle/reduce-puzzle";
import { createCamera } from "../src/render/camera";

const LATCH_STEP = 9;
// Each frame is 100 ms apart, so the accumulator always requests the 4-step cap.
const FRAME_MS = 100;

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

function statusAt(stepIndex: number, latchStep: number): PuzzleFrameStatus {
  const base = {
    goalCount: stepIndex,
    goalTarget: 540,
    goalFull: 900,
    stepLimit: 720,
    brimCount: 810,
  };
  // The win and Brim latch on the same step, so the run is decided there (D-10).
  return stepIndex < latchStep
    ? { ...base, outcome: "running", peakHeldCount: 0 }
    : {
        ...base,
        outcome: "won",
        maybeOutcomeStep: latchStep,
        peakHeldCount: 810,
        maybeBrimStep: latchStep,
        maybeDecidedStep: latchStep,
      };
}

function fakeSession(options: { latchStep: number; omitStatus?: boolean }) {
  let step = 0;
  const nextFrame = vi.fn((count = 1) => {
    step += count;
    return frameAt(step, options.omitStatus === true ? undefined : statusAt(step, options.latchStep));
  });
  const session = { nextFrame } as unknown as PuzzleSession;
  return { session, nextFrame };
}

function harness(options: { latchStep?: number; omitStatus?: boolean } = {}) {
  const { session, nextFrame } = fakeSession({
    latchStep: options.latchStep ?? LATCH_STEP,
    ...(options.omitStatus === undefined ? {} : { omitStatus: options.omitStatus }),
  });
  const clock: FrameClock = createFrameClock();
  clock.maybeCamera = createCamera(390, 844);
  const context = {} as CanvasRenderingContext2D;
  let state: PuzzleState = {
    kind: "running",
    stepIndex: 0,
    status: statusAt(0, LATCH_STEP),
    editor: EMPTY_EDITOR,
    speed: 1,
  };
  let current = true;
  const published: Published[] = [];
  const deps: PuzzleLoopDeps = {
    maybeSession: () => session,
    current: () => state,
    dispatch: (event) => {
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
let documentStub: { hidden: boolean };

function runFrame(): void {
  const entry = callbacks.entries().next();
  if (entry.done === true) {
    throw new Error("No animation frame is scheduled");
  }
  const [id, callback] = entry.value;
  callbacks.delete(id);
  callback(timestamp);
  timestamp += FRAME_MS;
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
  documentStub = { hidden: false };
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    rafId += 1;
    callbacks.set(rafId, callback);
    return rafId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  vi.stubGlobal("document", documentStub);
  vi.stubGlobal("performance", { now: () => 0 });
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

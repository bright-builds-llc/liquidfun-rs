import { createRoot, createSignal, type Setter } from "solid-js";
import { vi } from "vitest";

import type { RawProofFrame } from "../src/physics/frame";
import type { RawPlacementInspection } from "../src/physics/puzzle-inspector";
import type {
  GeneratedPuzzleSession,
  RawPuzzleLevelInfo,
} from "../src/physics/puzzle-session";
import { createFrameClock } from "../src/player/frame-loop";
import type { PuzzleHost } from "../src/player/puzzle-host";
import { createPuzzleRuntime } from "../src/player/puzzle-runtime";
import type { PlayerView } from "../src/player/view";
import { createCamera } from "../src/render/camera";

/** Particles in every fake frame; a live goal count never exceeds it. */
export const PARTICLE_COUNT = 4;

/** Puzzle scalars a fake frame reports at one step. */
export type FakePuzzleScalars = {
  readonly outcome: 0 | 1 | 2;
  readonly outcomeStep: number | undefined;
  readonly peakHeldCount: number;
  readonly brimStep: number | undefined;
  readonly decidedStep: number | undefined;
};

/** A run that never latches. */
export const RUNNING_SCALARS: FakePuzzleScalars = {
  outcome: 0,
  outcomeStep: undefined,
  peakHeldCount: 0,
  brimStep: undefined,
  decidedStep: undefined,
};

/** Win and Brim latch at `step`, so the run is decided there (D-10). */
export function winWithBrimFrom(step: number): (stepIndex: number) => FakePuzzleScalars {
  return (stepIndex) =>
    stepIndex < step
      ? RUNNING_SCALARS
      : { outcome: 1, outcomeStep: step, peakHeldCount: 811, brimStep: step, decidedStep: step };
}

/** Win latches at `step` without Brim, so the run keeps deciding until the step limit. */
export function winWithoutBrimFrom(step: number): (stepIndex: number) => FakePuzzleScalars {
  return (stepIndex) =>
    stepIndex < step
      ? RUNNING_SCALARS
      : {
          outcome: 1,
          outcomeStep: step,
          peakHeldCount: 600,
          brimStep: undefined,
          decidedStep: undefined,
        };
}

/** Loss at the 720-step limit. */
export function lossAtLimit(stepIndex: number): FakePuzzleScalars {
  return stepIndex < 720
    ? RUNNING_SCALARS
    : { outcome: 2, outcomeStep: 720, peakHeldCount: 0, brimStep: undefined, decidedStep: 720 };
}

export function fakeLevelInfo(title: string): RawPuzzleLevelInfo {
  return {
    levelId: () => "first-pour",
    title: () => title,
    targetCount: () => 540,
    fullCount: () => 900,
    brimCount: () => 810,
    parTools: () => 1,
    settleSteps: () => 30,
    stepLimit: () => 720,
    cupInterior: () => new Float32Array([1.6, 0, 3.4, 1.8]),
    buildArea: () => new Int32Array([-20, 44, 40, 84]),
    gridMeters: () => 0.05,
    toolKinds: () => new Uint8Array([1]),
    toolCounts: () => new Uint32Array([2]),
    levelHash: () => "0123456789abcdef",
    free: vi.fn(),
  };
}

/** Particle lane for a step; it changes every step so frame hashes differ. */
export function particlePositionsAt(stepIndex: number): Float32Array {
  return new Float32Array([0, stepIndex, 0.1, 0, 0.2, 0, 0.3, 0]);
}

export function rawFrame(
  stepIndex: number,
  scalars: FakePuzzleScalars = RUNNING_SCALARS,
): RawProofFrame {
  return {
    stepIndex: () => stepIndex,
    particleCount: () => PARTICLE_COUNT,
    rigidShapeCount: () => 0,
    maxSpeed: () => 0,
    stuckCandidateCount: () => 0,
    bodyContactCount: () => 0,
    particlePositions: () => particlePositionsAt(stepIndex),
    particleColors: () => new Uint8Array(PARTICLE_COUNT * 4).fill(200),
    particleRadii: () => new Float32Array(PARTICLE_COUNT).fill(0.05),
    rigidSegments: () => new Float32Array(),
    rigidCircles: () => new Float32Array(),
    circleLabels: () => [],
    goalCount: () => 0,
    goalTarget: () => 540,
    goalFull: () => 900,
    outcome: () => scalars.outcome,
    outcomeStep: () => scalars.outcomeStep,
    stepLimit: () => 720,
    peakHeldCount: () => scalars.peakHeldCount,
    brimCount: () => 810,
    brimStep: () => scalars.brimStep,
    decidedStep: () => scalars.decidedStep,
    free: () => undefined,
  };
}

/** Generated session fake; `script` decides the puzzle scalars per step. */
export function fakeGenerated(
  title = "First Pour",
  script: (stepIndex: number) => FakePuzzleScalars = () => RUNNING_SCALARS,
) {
  let step = 0;
  const generated = {
    advance: vi.fn((count: number) => {
      step += count;
    }),
    captureFrame: vi.fn(() => rawFrame(step, script(step))),
    applyControl: vi.fn(() => false),
    applyAction: vi.fn(),
    pointerAction: vi.fn(),
    setGravity: vi.fn(),
    restoreAuthoredGravity: vi.fn(),
    levelInfo: vi.fn(() => fakeLevelInfo(title)),
    free: vi.fn(),
  } satisfies GeneratedPuzzleSession;
  return generated;
}

/** Raw inspector result with one status per placement and zeroed outlines. */
export function fakeRawInspection(statuses: readonly number[]): RawPlacementInspection {
  return {
    statuses: () => new Uint8Array(statuses),
    outlines: () => new Float32Array(statuses.length * 8),
    free: vi.fn(),
  };
}

function setter<T>(): Setter<T> {
  return vi.fn() as unknown as Setter<T>;
}

export function createHost() {
  return createRoot((dispose) => {
    const [view, setView] = createSignal<PlayerView>({ kind: "fallback" });
    const clock = createFrameClock();
    clock.viewportWidth = 390;
    clock.viewportHeight = 844;
    clock.cameraView = { zoom: 2, panX: 10, panY: -4 };
    const canvas = { dataset: {} as DOMStringMap } as HTMLCanvasElement;
    const host: PuzzleHost = {
      generation: 0,
      clock,
      maybeCanvas: canvas,
      maybeContext: {} as CanvasRenderingContext2D,
      maybeSession: undefined,
      maybeCanvasPointer: undefined,
      tiltBinding: { request: 0, stop: vi.fn(), maybeLive: undefined },
      constructionValues: { width: "3" },
      view,
      setView,
      refreshCamera: () => {
        clock.maybeCamera = createCamera(
          clock.viewportWidth,
          clock.viewportHeight,
          clock.cameraView,
          clock.worldBounds,
        );
      },
      drawSceneFrame: vi.fn(),
      failScene: vi.fn(),
      setMaybeDebugFrame: setter(),
      setStepsThisFrame: setter(),
      setFpsTicks: setter(),
      setPanEnabled: setter(),
      setTiltGravityEnabled: setter(),
      setTiltDebug: setter(),
      puzzle: createPuzzleRuntime(),
    };
    return { host, canvas, dispose };
  });
}

export function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => {
    resolve = yes;
  });
  return { promise, resolve };
}

export async function flushMicrotasks(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

/** Stubbed animation frames: callbacks run only when a test asks. */
export type AnimationFrames = {
  readonly callbacks: Map<number, FrameRequestCallback>;
  /** Runs the oldest pending callback; each call advances the clock 100 ms (the 4-step cap). */
  runNext(): void;
  /** Runs callbacks until none are pending or `limit` ran; returns how many ran. */
  runUntilIdle(limit: number): number;
};

const FRAME_MS = 100;

/** Stubs rAF, document, performance and window; undo with `vi.unstubAllGlobals()`. */
export function stubPuzzleGlobals(): AnimationFrames {
  const callbacks = new Map<number, FrameRequestCallback>();
  let rafId = 0;
  let timestamp = 0;
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    rafId += 1;
    callbacks.set(rafId, callback);
    return rafId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  vi.stubGlobal("document", { hidden: false });
  vi.stubGlobal("performance", { now: () => 0 });
  vi.stubGlobal("window", {});

  function runNext(): void {
    const entry = callbacks.entries().next();
    if (entry.done === true) {
      throw new Error("No animation frame is scheduled");
    }
    const [id, callback] = entry.value;
    callbacks.delete(id);
    callback(timestamp);
    timestamp += FRAME_MS;
  }

  return {
    callbacks,
    runNext,
    runUntilIdle(limit) {
      let ran = 0;
      while (callbacks.size > 0 && ran < limit) {
        runNext();
        ran += 1;
      }
      return ran;
    },
  };
}

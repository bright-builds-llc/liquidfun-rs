import { vi } from "vitest";
import type {
  LiveSceneSession,
  WorkerAdvance,
  WorkerTiming,
} from "../src/physics/live-session";
import type { RenderFrame } from "../src/physics/frame";
import { createFrameClock, type FrameLoopDeps } from "../src/player/frame-loop";
import { observeFrame } from "../src/player/observe";
import type { PlayerView } from "../src/player/view";
import { createCamera } from "../src/render/camera";
import type { SceneRuntime } from "../src/player/scene-runtime";
import { createTiltBinding } from "../src/input/tilt-binding";

export function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

export function testFrame(stepIndex: number, side = 1): RenderFrame {
  return {
    stepIndex,
    particleCount: 1,
    rigidShapeCount: 1,
    maxSpeed: 2,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array([0, 1]),
    particleColors: new Uint8Array([77, 163, 255, 255]),
    particleRadii: new Float32Array([0.005]),
    rigidSegments: new Float32Array([side, 0, side, 1]),
    rigidCircles: new Float32Array(),
    circleLabels: [],
  };
}

export function workerLoopFixture() {
  let step = 0,
    side = 1;
  const timing: WorkerTiming = {
    advanceMs: 30,
    captureMs: 1,
    parseMs: 0.1,
    roundTripMs: 32,
  };
  const requests: {
    count: number;
    result: ReturnType<typeof deferred<WorkerAdvance>>;
  }[] = [];
  const owner: LiveSceneSession = {
    backend: "worker",
    maybeLastTiming: timing,
    advanceBudgeted: vi.fn((count) => {
      const result = deferred<WorkerAdvance>();
      requests.push({ count, result });
      return result.promise;
    }),
    nextFrame: vi.fn(async (count = 1) => {
      step += count;
      return testFrame(step, side);
    }),
    captureFrame: vi.fn(async () => testFrame(step, side)),
    advanceOnly: vi.fn(async (count) => {
      step += count;
    }),
    applyControl: vi.fn(async () => false),
    applyAction: vi.fn(async () => undefined),
    pointerAction: vi.fn(async () => undefined),
    setGravity: vi.fn(async () => undefined),
    restoreAuthoredGravity: vi.fn(async () => undefined),
    dispose: vi.fn(),
  };
  const clock = createFrameClock();
  clock.maybeCamera = createCamera(390, 844);
  const context = {} as CanvasRenderingContext2D;
  let active: LiveSceneSession | undefined = owner;
  let view: PlayerView = {
    kind: "playing",
    frame: observeFrame(testFrame(0), undefined, 0),
  };
  let rafId = 0;
  const callbacks = new Map<number, FrameRequestCallback>();
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    callbacks.set(++rafId, callback);
    return rafId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  vi.stubGlobal("document", { hidden: false });
  vi.stubGlobal("window", { matchMedia: () => ({ matches: false }) });
  const deps: FrameLoopDeps = {
    view: () => view,
    setView: (next) => {
      view = next;
    },
    maybeSession: () => active,
    route: () => ({ kind: "scene", id: "tesla-valve" }),
    startScene: vi.fn(),
    fail: vi.fn(),
    drawSceneFrame: vi.fn(),
    setMaybeDebugFrame: vi.fn(),
    setStepsThisFrame: vi.fn(),
    setMaybePixelsPerMeter: vi.fn(),
    setFpsTicks: vi.fn(),
  };
  const runtime = {
    generation: 1,
    constructionValues: {},
    maybeSession: owner,
    maybeContext: context,
    clock,
    tiltBinding: createTiltBinding(),
    route: deps.route,
    view: deps.view,
    setView: deps.setView,
    setTiltDebug: vi.fn(),
    gravitySliderMagnitude: () => 10,
    setMaybeDebugFrame: deps.setMaybeDebugFrame,
    setStepsThisFrame: deps.setStepsThisFrame,
    setFpsTicks: deps.setFpsTicks,
    drawSceneFrame: deps.drawSceneFrame,
    frameDeps: () => deps,
    failScene: deps.fail,
  } as unknown as SceneRuntime;
  return {
    owner,
    clock,
    context,
    deps,
    runtime,
    requests,
    view: () => view,
    pause: () => {
      view = {
        kind: "paused",
        frame: observeFrame(testFrame(step), undefined, 0),
      };
    },
    setSide: (next: number) => {
      side = next;
    },
    abandon: () => {
      active = undefined;
    },
    raf: (timestamp: number) => {
      const pending = [...callbacks.values()];
      callbacks.clear();
      pending.forEach((callback) => callback(timestamp));
    },
    finish: (index: number, ran: number, maybeFrame?: RenderFrame) => {
      const request = requests[index];
      if (request === undefined) throw new Error("Missing step request");
      step += ran;
      request.result.resolve({
        frame: maybeFrame ?? testFrame(step, side),
        ran,
        timing,
      });
    },
  };
}

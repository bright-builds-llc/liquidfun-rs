import { appendFpsTick, type FpsTick } from "../components/fps-meter";
import type { SceneId } from "../catalog/scenes";
import {
  FRAME_STEP_BUDGET_MS,
  accumulateStepTime,
  restoreUnrunSteps,
} from "../physics/clock";
import type { RenderFrame } from "../physics/frame";
import type { SceneSession } from "../physics/session";
import { observeFrame } from "./observe";
import { maybeReadySceneId } from "./runtime";
import { prefersReducedMotion, isUsableViewport } from "./viewport";
import { maybeObservedFrame, type PlayerView } from "./view";
import { resizeCanvasBackingStore } from "../render/canvas";
import {
  IDENTITY_CAMERA_VIEW,
  WORLD_BOUNDS,
  type Camera,
  type CameraView,
  type WorldBounds,
} from "../render/camera";

const MILLISECONDS_PER_SECOND = 1000;

export type FrameClock = {
  maybeAnimationFrameId: number | undefined;
  maybeLastTimestamp: number | undefined;
  frameRemainderSeconds: number;
  maybePreviousFrame: RenderFrame | undefined;
  viewportWidth: number;
  viewportHeight: number;
  maybeResizeObserver: ResizeObserver | undefined;
  maybeCamera: Camera | undefined;
  cameraView: CameraView;
  worldBounds: WorldBounds;
};

export type FrameLoopDeps = {
  view: () => PlayerView;
  fail: (error: unknown) => void;
  maybeSession: () => SceneSession | undefined;
  route: () => Parameters<typeof maybeReadySceneId>[0];
  startScene: (id: SceneId) => void;
  drawSceneFrame: (
    context: CanvasRenderingContext2D,
    frame: RenderFrame,
    camera: Camera,
  ) => void;
  setMaybeDebugFrame: (frame: RenderFrame | undefined) => void;
  setStepsThisFrame: (steps: number) => void;
  setFpsTicks: (
    updater: (ticks: readonly FpsTick[]) => readonly FpsTick[],
  ) => void;
  setView: (view: PlayerView) => void;
};

export function createFrameClock(): FrameClock {
  return {
    maybeAnimationFrameId: undefined,
    maybeLastTimestamp: undefined,
    frameRemainderSeconds: 0,
    maybePreviousFrame: undefined,
    viewportWidth: 0,
    viewportHeight: 0,
    maybeResizeObserver: undefined,
    maybeCamera: undefined,
    cameraView: IDENTITY_CAMERA_VIEW,
    worldBounds: WORLD_BOUNDS,
  };
}

export function cancelPendingFrame(clock: FrameClock): void {
  if (clock.maybeAnimationFrameId === undefined) {
    return;
  }

  cancelAnimationFrame(clock.maybeAnimationFrameId);
  clock.maybeAnimationFrameId = undefined;
}

export function disconnectResizeObserver(clock: FrameClock): void {
  const maybeObserver = clock.maybeResizeObserver;
  clock.maybeResizeObserver = undefined;
  maybeObserver?.disconnect();
}

export function scheduleFrame(
  clock: FrameClock,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
): void {
  clock.maybeAnimationFrameId = requestAnimationFrame((timestamp) => {
    clock.maybeAnimationFrameId = undefined;
    if (deps.view().kind !== "playing") {
      return;
    }

    if (document.hidden) {
      clock.maybeLastTimestamp = undefined;
      scheduleFrame(clock, context, deps);
      return;
    }

    const maybeOwnedSession = deps.maybeSession();
    if (maybeOwnedSession === undefined) {
      deps.fail(new Error("Scene session owner is unavailable"));
      return;
    }

    const camera = clock.maybeCamera;
    if (camera === undefined) {
      deps.fail(new Error("Canvas camera is unavailable"));
      return;
    }

    if (clock.maybeLastTimestamp === undefined) {
      clock.frameRemainderSeconds = 0;
      clock.maybeLastTimestamp = timestamp;
      scheduleFrame(clock, context, deps);
      return;
    }

    const elapsedSeconds =
      (timestamp - clock.maybeLastTimestamp) / MILLISECONDS_PER_SECOND;
    const stepTime = accumulateStepTime(
      clock.frameRemainderSeconds,
      elapsedSeconds,
    );
    clock.frameRemainderSeconds = stepTime.remainderSeconds;
    clock.maybeLastTimestamp = timestamp;
    if (stepTime.stepCount === 0) {
      scheduleFrame(clock, context, deps);
      return;
    }
    try {
      const stepStarted = performance.now();
      let ran = 1;
      let frame = maybeOwnedSession.nextFrame(1);
      while (
        ran < stepTime.stepCount &&
        performance.now() - stepStarted < FRAME_STEP_BUDGET_MS
      ) {
        frame = maybeOwnedSession.nextFrame(1);
        ran += 1;
      }
      clock.frameRemainderSeconds = restoreUnrunSteps(
        clock.frameRemainderSeconds,
        stepTime.stepCount,
        ran,
      );
      deps.drawSceneFrame(context, frame, camera);
      const observation = observeFrame(
        frame,
        clock.maybePreviousFrame,
        maybeObservedFrame(deps.view())?.movedFrameCount ?? 0,
      );
      clock.maybePreviousFrame = frame;
      deps.setMaybeDebugFrame(frame);
      deps.setStepsThisFrame(ran);
      deps.setFpsTicks((ticks) =>
        appendFpsTick(ticks, {
          timeMs: timestamp,
          simSteps: ran,
        }),
      );
      deps.setView({ kind: "playing", frame: observation });
      scheduleFrame(clock, context, deps);
    } catch (error) {
      deps.fail(error);
    }
  });
}

export function presentOwnedFrame(
  clock: FrameClock,
  ownedSession: SceneSession,
  context: CanvasRenderingContext2D,
  resetObservation: boolean,
  deps: FrameLoopDeps,
): void {
  const camera = clock.maybeCamera;
  if (camera === undefined) {
    deps.fail(new Error("Canvas camera is unavailable"));
    return;
  }

  const frame = ownedSession.nextFrame();
  deps.drawSceneFrame(context, frame, camera);
  const observation = observeFrame(
    frame,
    resetObservation ? undefined : clock.maybePreviousFrame,
    resetObservation ? 0 : maybeObservedFrame(deps.view())?.movedFrameCount ?? 0,
  );
  clock.maybePreviousFrame = frame;
  deps.setMaybeDebugFrame(frame);
  deps.setStepsThisFrame(1);
  deps.setFpsTicks((ticks) =>
    appendFpsTick(ticks, {
      timeMs: performance.now(),
      simSteps: 1,
    }),
  );

  if (prefersReducedMotion()) {
    deps.setView({ kind: "paused", frame: observation });
    return;
  }

  deps.setView({ kind: "playing", frame: observation });
  scheduleFrame(clock, context, deps);
}

export function connectResizeObserver(
  clock: FrameClock,
  canvas: HTMLCanvasElement,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
): void {
  disconnectResizeObserver(clock);
  clock.maybeResizeObserver = new ResizeObserver(() => {
    const resizedBounds = canvas.getBoundingClientRect();
    if (!isUsableViewport(resizedBounds.width, resizedBounds.height)) {
      return;
    }

    try {
      clock.viewportWidth = resizedBounds.width;
      clock.viewportHeight = resizedBounds.height;
      const resizedCamera = resizeCanvasBackingStore(
        canvas,
        clock.viewportWidth,
        clock.viewportHeight,
        window.devicePixelRatio,
        clock.cameraView,
        clock.worldBounds,
      );
      clock.maybeCamera = resizedCamera;
      if (resizedCamera === undefined) {
        return;
      }
      const maybeReadyId = maybeReadySceneId(deps.route());
      if (deps.maybeSession() === undefined && maybeReadyId !== undefined) {
        deps.startScene(maybeReadyId);
        return;
      }

      const maybeFrame = clock.maybePreviousFrame;
      if (maybeFrame !== undefined) {
        deps.drawSceneFrame(context, maybeFrame, resizedCamera);
      }
    } catch (error) {
      deps.fail(error);
    }
  });
  clock.maybeResizeObserver.observe(canvas);
}

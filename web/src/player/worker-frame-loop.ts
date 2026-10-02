import { appendFpsTick } from "../components/fps-meter";
import {
  MAX_DELTA_SECONDS,
  STEP_SECONDS,
  acceptedStepCount,
  restoreUnrunSteps,
} from "../physics/clock";
import type { RenderFrame } from "../physics/frame";
import type { LiveSceneSession, WorkerTiming } from "../physics/live-session";
import type { FrameClock, FrameLoopDeps } from "./frame-loop";
import { observeFrame } from "./observe";
import { maybeObservedFrame } from "./view";
import { prefersReducedMotion } from "./viewport";

type Flight = {
  readonly clockEpoch: number;
  readonly publicationEpoch: number;
  readonly requested: number;
};
type WorkerClock = {
  readonly owner: LiveSceneSession;
  clockEpoch: number;
  publicationEpoch: number;
  mutationEpoch: number;
  holds: number;
  maybeFlight: Flight | undefined;
  maybeWallTime: number | undefined;
  maybeRafTimestamp: number | undefined;
  unpublishedSteps: number;
  maybeHeldFrame: RenderFrame | undefined;
};
const states = new WeakMap<FrameClock, WorkerClock>();

function stateFor(clock: FrameClock, owner: LiveSceneSession): WorkerClock {
  let state = states.get(clock);
  if (state === undefined || state.owner !== owner) {
    state = {
      owner,
      clockEpoch: 0,
      publicationEpoch: 0,
      mutationEpoch: 0,
      holds: 0,
      maybeFlight: undefined,
      maybeWallTime: undefined,
      maybeRafTimestamp: undefined,
      unpublishedSteps: 0,
      maybeHeldFrame: undefined,
    };
    states.set(clock, state);
    clock.frameRemainderSeconds = 0;
  }
  return state;
}

function current(
  clock: FrameClock,
  state: WorkerClock,
  deps: FrameLoopDeps,
): boolean {
  return states.get(clock) === state && deps.maybeSession() === state.owner;
}

function accumulate(clock: FrameClock, state: WorkerClock): void {
  const now = performance.now();
  if (state.maybeWallTime !== undefined)
    clock.frameRemainderSeconds = Math.min(
      MAX_DELTA_SECONDS,
      clock.frameRemainderSeconds +
        Math.max(0, now - state.maybeWallTime) / 1000,
    );
  state.maybeWallTime = now;
}

/** Invalidates late paints/debt without pretending an accepted in-flight step can be undone. */
export function cancelWorkerFrames(clock: FrameClock): void {
  const state = states.get(clock);
  if (state === undefined) return;
  state.clockEpoch += 1;
  state.publicationEpoch += 1;
  state.maybeWallTime = undefined;
  state.unpublishedSteps = 0;
  clock.frameRemainderSeconds = 0;
}

export function resetWorkerTime(
  clock: FrameClock,
  owner: LiveSceneSession,
): void {
  const state = stateFor(clock, owner);
  state.clockEpoch += 1;
  state.maybeWallTime = undefined;
  state.unpublishedSteps = 0;
  clock.frameRemainderSeconds = 0;
}

export function takeUnpublishedWorkerSteps(
  clock: FrameClock,
  owner: LiveSceneSession,
): number {
  const state = stateFor(clock, owner),
    steps = state.unpublishedSteps;
  state.unpublishedSteps = 0;
  return steps;
}

/** A geometry/control barrier prevents pre-mutation snapshots from overwriting refreshed state. */
export function beginWorkerMutation(
  clock: FrameClock,
  owner: LiveSceneSession,
) {
  const state = stateFor(clock, owner),
    epoch = ++state.mutationEpoch;
  state.publicationEpoch += 1;
  state.maybeHeldFrame = undefined;
  state.holds += 1;
  let released = false;
  return {
    latest: () => states.get(clock) === state && state.mutationEpoch === epoch,
    release(context: CanvasRenderingContext2D, deps: FrameLoopDeps) {
      if (released) return;
      released = true;
      state.holds -= 1;
      dispatch(clock, state, context, deps);
    },
  };
}

export function publishWorkerFrame(
  clock: FrameClock,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
  frame: RenderFrame,
  ran: number,
  kind: "playing" | "paused",
  resetObservation = false,
  maybeTiming?: WorkerTiming,
): void {
  const camera = clock.maybeCamera;
  if (camera === undefined) throw new Error("Canvas camera is unavailable");
  const state = states.get(clock);
  if (document.hidden) {
    if (state !== undefined) state.maybeHeldFrame = frame;
  } else {
    if (state !== undefined) state.maybeHeldFrame = undefined;
    deps.drawSceneFrame(context, frame, camera);
  }
  const observation = observeFrame(
    frame,
    resetObservation ? undefined : clock.maybePreviousFrame,
    resetObservation
      ? 0
      : (maybeObservedFrame(deps.view())?.movedFrameCount ?? 0),
  );
  clock.maybePreviousFrame = frame;
  deps.setMaybeDebugFrame(frame);
  deps.setStepsThisFrame(ran);
  deps.setView({ kind, frame: observation });
  if (maybeTiming !== undefined) deps.maybeOnWorkerTiming?.(maybeTiming);
  if (document.hidden) return;
  deps.setFpsTicks((ticks) =>
    appendFpsTick(ticks, {
      timeMs: performance.now(),
      simSteps: ran,
      ...(state?.maybeRafTimestamp === undefined
        ? {}
        : { maybeRafTimestampMs: state.maybeRafTimestamp }),
    }),
  );
}

/** Paints a fresh hidden control snapshot on visibility without advancing its owner. */
export function publishHeldWorkerFrame(
  clock: FrameClock,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
): void {
  const state = states.get(clock),
    view = deps.view();
  if (
    state === undefined ||
    !current(clock, state, deps) ||
    document.hidden ||
    state.maybeHeldFrame === undefined ||
    (view.kind !== "playing" && view.kind !== "paused")
  )
    return;
  publishWorkerFrame(clock, context, deps, state.maybeHeldFrame, 0, view.kind);
}

function dispatch(
  clock: FrameClock,
  state: WorkerClock,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
): void {
  if (
    !current(clock, state, deps) ||
    state.maybeFlight !== undefined ||
    state.holds > 0 ||
    deps.view().kind !== "playing" ||
    document.hidden
  )
    return;
  accumulate(clock, state);
  const requested = acceptedStepCount(clock.frameRemainderSeconds);
  if (requested === 0) return;
  clock.frameRemainderSeconds -= requested * STEP_SECONDS;
  const flight: Flight = {
    requested,
    clockEpoch: state.clockEpoch,
    publicationEpoch: state.publicationEpoch,
  };
  state.maybeFlight = flight;
  void state.owner
    .advanceBudgeted(requested)
    .then((result) => {
      if (state.maybeFlight !== flight) return;
      state.maybeFlight = undefined;
      if (!current(clock, state, deps)) return;
      if (document.hidden) {
        cancelWorkerFrames(clock);
        return;
      }
      const settlesPause =
        flight.clockEpoch !== state.clockEpoch &&
        state.publicationEpoch === flight.publicationEpoch + 1 &&
        state.holds === 0 &&
        deps.view().kind === "paused";
      if (settlesPause) {
        publishWorkerFrame(
          clock,
          context,
          deps,
          result.frame,
          result.ran,
          "paused",
          false,
          result.timing,
        );
        return;
      }
      if (flight.clockEpoch === state.clockEpoch) {
        accumulate(clock, state);
        clock.frameRemainderSeconds = Math.min(
          MAX_DELTA_SECONDS,
          restoreUnrunSteps(clock.frameRemainderSeconds, requested, result.ran),
        );
        state.unpublishedSteps += result.ran;
      }
      if (
        flight.clockEpoch === state.clockEpoch &&
        flight.publicationEpoch === state.publicationEpoch &&
        state.holds === 0 &&
        deps.view().kind === "playing"
      ) {
        const ran = state.unpublishedSteps;
        state.unpublishedSteps = 0;
        publishWorkerFrame(
          clock,
          context,
          deps,
          result.frame,
          ran,
          "playing",
          false,
          result.timing,
        );
      }
      // Debt that accrued while the worker ran is admitted promptly, not delayed another vsync.
      dispatch(clock, state, context, deps);
    })
    .catch((error: unknown) => {
      if (state.maybeFlight === flight) state.maybeFlight = undefined;
      if (current(clock, state, deps)) deps.fail(error);
    });
}

export function scheduleWorkerFrame(
  clock: FrameClock,
  context: CanvasRenderingContext2D,
  deps: FrameLoopDeps,
  owner: LiveSceneSession,
): void {
  if (clock.maybeAnimationFrameId !== undefined) return;
  const state = stateFor(clock, owner);
  clock.maybeAnimationFrameId = requestAnimationFrame((timestamp) => {
    clock.maybeAnimationFrameId = undefined;
    if (!current(clock, state, deps) || deps.view().kind !== "playing") return;
    if (document.hidden) {
      cancelWorkerFrames(clock);
    } else {
      state.maybeRafTimestamp = timestamp;
      clock.maybeLastTimestamp = timestamp;
      accumulate(clock, state);
      dispatch(clock, state, context, deps);
    }
    scheduleWorkerFrame(clock, context, deps, owner);
  });
}

/** One exact initial/manual step; a paused view never resumes because a reply arrived. */
export function presentWorkerOwnedFrame(
  clock: FrameClock,
  owner: LiveSceneSession,
  context: CanvasRenderingContext2D,
  resetObservation: boolean,
  deps: FrameLoopDeps,
): Promise<void> {
  const state = stateFor(clock, owner),
    publicationEpoch = state.publicationEpoch;
  const before = deps.view().kind;
  state.holds += 1;
  const step = async () => {
    if (before === "paused") {
      // A step starts from settled paused state, including any previously accepted in-flight work.
      const settled = await owner.captureFrame();
      if (
        !current(clock, state, deps) ||
        publicationEpoch !== state.publicationEpoch ||
        document.hidden
      )
        return;
      publishWorkerFrame(clock, context, deps, settled, 0, "paused");
    }
    const frame = await owner.nextFrame(1);
    if (
      !current(clock, state, deps) ||
      publicationEpoch !== state.publicationEpoch ||
      document.hidden
    )
      return;
    const kind =
      deps.view().kind === "paused" ||
      (before === "paused" && deps.view().kind !== "playing") ||
      prefersReducedMotion()
        ? "paused"
        : "playing";
    publishWorkerFrame(
      clock,
      context,
      deps,
      frame,
      1,
      kind,
      resetObservation,
      owner.maybeLastTiming,
    );
    if (kind === "playing") scheduleWorkerFrame(clock, context, deps, owner);
  };
  return step()
    .catch((error: unknown) => {
      if (current(clock, state, deps)) deps.fail(error);
    })
    .finally(() => {
      state.holds -= 1;
      dispatch(clock, state, context, deps);
    });
}

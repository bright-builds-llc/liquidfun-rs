import {
  NO_SOUND_SIGNALS,
  foldSoundSignals,
  type PresentedSoundSignals,
} from "../audio/sound-signals";
import { FRAME_STEP_BUDGET_MS, accumulateStepTime } from "../physics/clock";
import type { PuzzleFrameStatus, RenderFrame } from "../physics/frame";
import type { PuzzleSession } from "../physics/puzzle-session";
import { puzzleStepBudget, type PuzzleEvent, type PuzzleState } from "../puzzle/reduce-puzzle";
import { restoreUnrunSpeedSteps, speedStepRequest } from "../puzzle/speed";
import type { Camera } from "../render/camera";
import type { FrameClock } from "./frame-loop";
import { observeFrame, type FrameObservation } from "./observe";

const MILLISECONDS_PER_SECOND = 1000;
const MISSING_STATUS_MESSAGE = "Puzzle frame is missing its goal status";

/** What the puzzle frame loop reads and drives; built by `puzzle-lifecycle.ts`. */
export type PuzzleLoopDeps = {
  readonly maybeSession: () => PuzzleSession | undefined;
  readonly current: () => PuzzleState;
  /** Folds one event into the non-reactive mirror and returns the new state. */
  readonly dispatch: (event: PuzzleEvent) => PuzzleState;
  /**
   * Publishes one presented frame: Solid puzzle state plus the player view.
   * `signals` fold every fixed step of this presented frame (Phase 42 D-07).
   */
  readonly publish: (
    frame: RenderFrame,
    observation: FrameObservation,
    state: PuzzleState,
    signals: PresentedSoundSignals,
  ) => void;
  readonly drawSceneFrame: (
    context: CanvasRenderingContext2D,
    frame: RenderFrame,
    camera: Camera,
  ) => void;
  readonly fail: (error: unknown) => void;
  /** False once a newer build, route change or dispose superseded this loop. */
  readonly isCurrent: () => boolean;
};

/** Goal status of a puzzle frame; a scene-shaped frame fails closed. */
export function requirePuzzleStatus(frame: RenderFrame): PuzzleFrameStatus {
  const maybeStatus = frame.maybePuzzle;
  if (maybeStatus === undefined) {
    throw new Error(MISSING_STATUS_MESSAGE);
  }

  return maybeStatus;
}

function isHeld(state: PuzzleState): boolean {
  return state.kind === "ended" && state.held;
}

/**
 * Runs the puzzle on fixed steps (D-08, D-11). Each step is captured on its
 * own so the latch is observed at its outcome step; the settle tail is clamped
 * and the loop stops scheduling once held. Fast-forward (D-24) only scales
 * how many steps a frame requests; the 12 ms budget still bounds the work and
 * unrun steps return as wall time. The rAF id lives in
 * `clock.maybeAnimationFrameId`, so `cancelPendingFrame` stops this loop too.
 */
export function schedulePuzzleFrame(
  clock: FrameClock,
  context: CanvasRenderingContext2D,
  deps: PuzzleLoopDeps,
): void {
  let movedFrameCount = 0;

  function presentSteps(session: PuzzleSession, camera: Camera, requested: number): void {
    let state = deps.current();
    const { speed } = state;
    const budget = puzzleStepBudget(state, requested);
    if (budget === 0) {
      requestNext();
      return;
    }

    const started = performance.now();
    let ran = 0;
    let signals = NO_SOUND_SIGNALS;
    // One step per capture, so the frame at the outcome step is never skipped (Pitfall 3).
    const stepOnce = (): RenderFrame => {
      const stepped = session.nextFrame(1);
      const status = requirePuzzleStatus(stepped);
      state = deps.dispatch({ kind: "stepped", stepIndex: stepped.stepIndex, status });
      signals = foldSoundSignals(signals, stepped.maybeSound);
      ran += 1;
      return stepped;
    };
    let frame = stepOnce();
    while (ran < budget && !isHeld(state) && performance.now() - started < FRAME_STEP_BUDGET_MS) {
      frame = stepOnce();
    }

    clock.frameRemainderSeconds = restoreUnrunSpeedSteps(
      clock.frameRemainderSeconds,
      budget,
      ran,
      speed,
    );
    deps.drawSceneFrame(context, frame, camera);
    const observation = observeFrame(frame, clock.maybePreviousFrame, movedFrameCount);
    movedFrameCount = observation.movedFrameCount;
    clock.maybePreviousFrame = frame;
    deps.publish(frame, observation, state, signals);
    if (!isHeld(state)) {
      requestNext();
    }
  }

  function onFrame(timestamp: number): void {
    clock.maybeAnimationFrameId = undefined;
    if (!deps.isCurrent() || puzzleStepBudget(deps.current(), 1) === 0) {
      return;
    }

    if (document.hidden) {
      clock.maybeLastTimestamp = undefined;
      requestNext();
      return;
    }

    const maybeSession = deps.maybeSession();
    if (maybeSession === undefined) {
      deps.fail(new Error("Puzzle session owner is unavailable"));
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
      requestNext();
      return;
    }

    const elapsedSeconds = (timestamp - clock.maybeLastTimestamp) / MILLISECONDS_PER_SECOND;
    const stepTime = accumulateStepTime(clock.frameRemainderSeconds, elapsedSeconds);
    clock.frameRemainderSeconds = stepTime.remainderSeconds;
    clock.maybeLastTimestamp = timestamp;
    try {
      const requested = speedStepRequest(stepTime.stepCount, deps.current().speed);
      presentSteps(maybeSession, camera, requested);
    } catch (error) {
      deps.fail(error);
    }
  }

  function requestNext(): void {
    clock.maybeAnimationFrameId = requestAnimationFrame(onFrame);
  }

  requestNext();
}

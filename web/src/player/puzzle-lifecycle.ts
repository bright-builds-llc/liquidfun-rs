import { puzzleTitleForId } from "../catalog/puzzle-levels";
import { worldBoundsForPuzzle } from "../catalog/puzzle-bounds";
import type { PuzzleId } from "../catalog/puzzles";
import { changeTiltGravity } from "../input/tilt-binding";
import { loadPuzzleSession } from "../physics/puzzle-loader";
import { createPuzzleSession } from "../physics/puzzle-session";
import type { PuzzleController } from "../puzzle/controller";
import { goalTicks } from "../puzzle/format";
import {
  EMPTY_PLACEMENTS,
  encodePlacementLanes,
  maybeReadTestPlacements,
} from "../puzzle/placements";
import {
  INITIAL_PUZZLE_STATE,
  reducePuzzle,
  type PuzzleEvent,
  type PuzzleState,
} from "../puzzle/reduce-puzzle";
import { IDENTITY_CAMERA_VIEW } from "../render/camera";
import { glassCup } from "../render/glass-cup";
import { cancelPendingFrame } from "./frame-loop";
import { isStaleGeneration, nextGeneration } from "./generation";
import { observeFrame } from "./observe";
import { requirePuzzleStatus, schedulePuzzleFrame } from "./puzzle-loop";
import type { PuzzleRuntime } from "./puzzle-runtime";
import type { SceneRuntime } from "./scene-runtime";
import { maybeObservedFrame } from "./view";

/** Scene runtime fields the puzzle lifecycle uses, plus the puzzle bag. */
export type PuzzleHost = Pick<
  SceneRuntime,
  | "generation"
  | "clock"
  | "maybeCanvas"
  | "maybeContext"
  | "maybeSession"
  | "maybeWorkerInitialization"
  | "maybeCanvasPointer"
  | "tiltBinding"
  | "constructionValues"
  | "view"
  | "setView"
  | "refreshCamera"
  | "drawSceneFrame"
  | "failScene"
  | "setMaybeDebugFrame"
  | "setStepsThisFrame"
  | "setFpsTicks"
  | "setPanEnabled"
  | "setTiltGravityEnabled"
  | "setTiltDebug"
> & { readonly puzzle: PuzzleRuntime };

function incrementGeneration(host: PuzzleHost): number {
  host.generation = nextGeneration(host.generation);
  return host.generation;
}

function dispatch(host: PuzzleHost, event: PuzzleEvent): PuzzleState {
  host.puzzle.current = reducePuzzle(host.puzzle.current, event);
  return host.puzzle.current;
}

function publishState(host: PuzzleHost, state: PuzzleState): void {
  host.puzzle.setState(state);
}

/** Releases the live session; a throwing dispose propagates to the caller's failure path. */
function disposeCurrentSession(host: PuzzleHost): void {
  host.maybeWorkerInitialization?.abort();
  delete host.maybeWorkerInitialization;
  host.maybeCanvasPointer?.cancel();
  const maybeOwned = host.maybeSession;
  host.maybeSession = undefined;
  host.puzzle.maybeOwned = undefined;
  host.clock.maybeLastTimestamp = undefined;
  maybeOwned?.dispose();
}

function viewKindFor(state: PuzzleState): "playing" | "paused" {
  const stepping = state.kind === "running" || (state.kind === "ended" && !state.held);
  return stepping ? "playing" : "paused";
}

function startLoop(host: PuzzleHost, context: CanvasRenderingContext2D): void {
  host.clock.maybeLastTimestamp = undefined;
  host.clock.frameRemainderSeconds = 0;
  const started = host.generation;
  const owned = host.puzzle.maybeOwned;
  schedulePuzzleFrame(host.clock, context, {
    maybeSession: () => host.puzzle.maybeOwned,
    current: () => host.puzzle.current,
    dispatch: (event) => dispatch(host, event),
    publish: (_frame, observation, state) => {
      publishState(host, state);
      host.setView({ kind: viewKindFor(state), frame: observation });
    },
    drawSceneFrame: host.drawSceneFrame,
    fail: host.failScene,
    isCurrent: () => host.generation === started && host.puzzle.maybeOwned === owned,
  });
}

/** Builds a fresh session for `id` and presents its step-0 frame; never rejects. */
async function buildPuzzle(host: PuzzleHost, id: PuzzleId): Promise<void> {
  const started = incrementGeneration(host);
  const canvas = host.maybeCanvas;
  const context = host.maybeContext;
  if (canvas === undefined || context === undefined) {
    host.failScene(new Error("Canvas element is unavailable"));
    return;
  }

  try {
    cancelPendingFrame(host.clock);
    disposeCurrentSession(host);
    const generated = await loadPuzzleSession(id, encodePlacementLanes(host.puzzle.placements));
    if (isStaleGeneration(started, host.generation)) {
      generated.free();
      return;
    }

    const owned = createPuzzleSession(generated, id);
    const info = owned.levelInfo;
    const expectedTitle = puzzleTitleForId(id);
    if (info.title !== expectedTitle) {
      owned.dispose();
      throw new Error(
        `Level title mismatch: expected "${expectedTitle}", wasm reported "${info.title}"`,
      );
    }

    host.maybeSession = owned;
    host.puzzle.maybeOwned = owned;
    host.puzzle.maybeGlass = glassCup(
      info.cupInterior,
      info.targetCount,
      info.fullCount,
      goalTicks(info.targetCount, info.fullCount),
    );
    canvas.dataset.simulationBackend = "direct";
    // Step 0 from a never-advanced world (D-07, Pitfall 2).
    const frame = owned.captureFrame();
    const status = requirePuzzleStatus(frame);
    const state = dispatch(host, { kind: "loaded", stepIndex: frame.stepIndex, status });
    const maybeCamera = host.clock.maybeCamera;
    if (maybeCamera !== undefined) {
      host.drawSceneFrame(context, frame, maybeCamera);
    }
    host.clock.maybePreviousFrame = frame;
    publishState(host, state);
    host.setView({ kind: viewKindFor(state), frame: observeFrame(frame, undefined, 0) });
    if (state.kind === "running") {
      startLoop(host, context);
    }
  } catch (error) {
    if (!isStaleGeneration(started, host.generation)) {
      host.failScene(error);
    }
  }
}

/**
 * Enters a puzzle route (D-04, D-16, D-22): stops tilt, turns pan off,
 * resets the camera, reads the test seam once and builds a fresh session.
 * Every failure reaches `host.failScene`; the promise never rejects.
 */
export async function beginPuzzleRoute(host: PuzzleHost, id: PuzzleId): Promise<void> {
  void changeTiltGravity(
    host.tiltBinding,
    false,
    () => undefined,
    () => undefined,
    host.setTiltGravityEnabled,
    host.setTiltDebug,
  ).catch(host.failScene);
  // Supersede any in-flight build even if entry fails below.
  incrementGeneration(host);
  try {
    cancelPendingFrame(host.clock);
    disposeCurrentSession(host);
    host.setPanEnabled(false);
    host.clock.cameraView = IDENTITY_CAMERA_VIEW;
    host.constructionValues = {};
    host.clock.maybePreviousFrame = undefined;
    host.setMaybeDebugFrame(undefined);
    host.setStepsThisFrame(0);
    host.setFpsTicks([]);
    host.clock.worldBounds = worldBoundsForPuzzle(
      id,
      host.clock.viewportWidth,
      host.clock.viewportHeight,
    );
    host.refreshCamera();
    host.puzzle.setMaybeLevelId(id);
    host.puzzle.maybeGlass = undefined;
    publishState(host, dispatch(host, { kind: "reset" }));
    host.setView({ kind: "loading" });
    host.puzzle.placements = maybeReadTestPlacements(window, id) ?? EMPTY_PLACEMENTS;
  } catch (error) {
    host.failScene(error);
    return;
  }

  await buildPuzzle(host, id);
}

/**
 * Resize-observer entry point: begins the route only when this level has not
 * been entered yet. The session is absent during every build and after a
 * failure, so restarting there would drop Retry's pending run or hide the
 * failure view. Route changes and Try again own every later rebuild.
 */
export function startPuzzleUnlessLoading(host: PuzzleHost, id: PuzzleId): void {
  if (host.puzzle.maybeLevelId() === id) {
    return;
  }

  void beginPuzzleRoute(host, id);
}

/** building -> running on the same never-advanced session (D-07). */
export function playPuzzle(host: PuzzleHost): void {
  const context = host.maybeContext;
  const maybeOwned = host.puzzle.maybeOwned;
  if (host.puzzle.current.kind !== "building" || context === undefined) {
    return;
  }
  if (maybeOwned === undefined || host.maybeSession !== maybeOwned) {
    return;
  }

  const state = dispatch(host, { kind: "play" });
  publishState(host, state);
  const maybeFrame = maybeObservedFrame(host.view());
  if (maybeFrame !== undefined) {
    host.setView({ kind: "playing", frame: maybeFrame });
  }
  startLoop(host, context);
}

function rebuildFromEnded(host: PuzzleHost, event: "retry" | "edit"): void {
  const maybeId = host.puzzle.maybeLevelId();
  if (host.puzzle.current.kind !== "ended" || maybeId === undefined) {
    return;
  }

  publishState(host, dispatch(host, { kind: event }));
  host.setView({ kind: "loading" });
  void buildPuzzle(host, maybeId);
}

/** ended -> running on a fresh session with the same placements (D-06). */
export function retryPuzzle(host: PuzzleHost): void {
  rebuildFromEnded(host, "retry");
}

/** ended -> building on a fresh step-0 session with the same placements (D-06). */
export function editPuzzle(host: PuzzleHost): void {
  rebuildFromEnded(host, "edit");
}

/** After a failure, rebuilds the level into building. */
export function tryAgainPuzzle(host: PuzzleHost): void {
  const maybeId = host.puzzle.maybeLevelId();
  if (host.view().kind !== "failure" || maybeId === undefined) {
    return;
  }

  publishState(host, dispatch(host, { kind: "reset" }));
  host.setView({ kind: "loading" });
  void buildPuzzle(host, maybeId);
}

/**
 * Clears puzzle state on leaving a puzzle route. The session itself is
 * disposed by the scene lifecycle (`beginScene` / `abandonScene`).
 */
export function leavePuzzle(host: PuzzleHost): void {
  cancelPendingFrame(host.clock);
  host.puzzle.maybeGlass = undefined;
  host.puzzle.maybeOwned = undefined;
  host.puzzle.placements = EMPTY_PLACEMENTS;
  host.puzzle.current = INITIAL_PUZZLE_STATE;
  host.puzzle.setState(INITIAL_PUZZLE_STATE);
  host.puzzle.setMaybeLevelId(undefined);
}

/** Lazy controller: reads `host.puzzle` only when an accessor or command runs. */
export function createPuzzleController(host: PuzzleHost): PuzzleController {
  return {
    maybeLevelId: () => host.puzzle.maybeLevelId(),
    state: () => host.puzzle.state(),
    play: () => playPuzzle(host),
    retry: () => retryPuzzle(host),
    edit: () => editPuzzle(host),
    tryAgain: () => tryAgainPuzzle(host),
  };
}

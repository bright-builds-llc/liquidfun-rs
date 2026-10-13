import { worldBoundsForPuzzle } from "../catalog/puzzle-bounds";
import type { PuzzleId } from "../catalog/puzzles";
import { changeTiltGravity } from "../input/tilt-binding";
import type { PuzzleController } from "../puzzle/controller";
import { placementsInOrder } from "../puzzle/editor";
import type { PuzzleKeyCommand } from "../puzzle/keyboard";
import { EMPTY_PLACEMENTS, maybeReadTestPlacements } from "../puzzle/placements";
import { INITIAL_PUZZLE_STATE, canPlay } from "../puzzle/reduce-puzzle";
import { INITIAL_PUZZLE_SPEED } from "../puzzle/speed";
import { IDENTITY_CAMERA_VIEW } from "../render/camera";
import { cancelPendingFrame } from "./frame-loop";
import {
  applyKeyCommand,
  applyPuzzleEdit,
  cancelPendingInspection,
  placeAtCenter,
  stepValveTime,
  toggleArm,
} from "./puzzle-editing";
import {
  disposeCurrentSession,
  dispatch,
  incrementGeneration,
  ownsLiveSession,
  publishState,
  type PuzzleHost,
} from "./puzzle-host";
import {
  buildPuzzle,
  cycleSpeed,
  editPuzzle,
  playPuzzle,
  retryPuzzle,
  stopPuzzle,
} from "./puzzle-run";

export type { PuzzleHost } from "./puzzle-host";
export { editPuzzle, playPuzzle, retryPuzzle, stopPuzzle } from "./puzzle-run";

/** Clears per-level puzzle signals and any pending inspection; the page-session star store is kept (D-14). */
function clearLevelSignals(host: PuzzleHost): void {
  cancelPendingInspection(host);
  host.puzzle.setMaybeLevelInfo(undefined);
  host.puzzle.setMaybeHeldHash(undefined);
  host.puzzle.setAnnouncement("");
}

/**
 * Enters a puzzle route (D-04, D-16, D-22): stops tilt, turns pan off,
 * resets the camera, reads the test seam once into the editor and builds a
 * fresh preview session. Speed starts at 1× (D-24). Every failure reaches
 * `host.failScene`; the promise never rejects.
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
    clearLevelSignals(host);
    host.puzzle.setMaybeBestStars(host.puzzle.starStore.get(id));
    const placements = maybeReadTestPlacements(window, id) ?? EMPTY_PLACEMENTS;
    publishState(host, dispatch(host, { kind: "reset", placements, speed: INITIAL_PUZZLE_SPEED }));
    host.setView({ kind: "loading" });
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

/** After a failure, rebuilds the level into building with the same tools and speed. */
export function tryAgainPuzzle(host: PuzzleHost): void {
  const maybeId = host.puzzle.maybeLevelId();
  if (host.view().kind !== "failure" || maybeId === undefined) {
    return;
  }

  try {
    const { editor, speed } = host.puzzle.current;
    clearLevelSignals(host);
    const placements = placementsInOrder(editor);
    publishState(host, dispatch(host, { kind: "reset", placements, speed }));
    host.setView({ kind: "loading" });
  } catch (error) {
    host.failScene(error);
    return;
  }
  void buildPuzzle(host, maybeId);
}

/**
 * Failure cleanup for the puzzle shell: cancels a pending live inspection and
 * releases the puzzle's claim on the session the scene lifecycle disposes.
 * Try again or a route change builds the next session.
 */
export function releaseFailedPuzzle(host: PuzzleHost): void {
  cancelPendingInspection(host);
  host.puzzle.maybeOwned = undefined;
}

/**
 * Clears puzzle state on leaving a puzzle route. The session itself is
 * disposed by the scene lifecycle (`beginScene` / `abandonScene`); the star
 * store survives for the page session (D-14).
 */
export function leavePuzzle(host: PuzzleHost): void {
  cancelPendingFrame(host.clock);
  host.puzzle.maybeEditorInput?.detach();
  host.puzzle.maybeEditorInput = undefined;
  host.puzzle.maybeGlass = undefined;
  host.puzzle.maybeOwned = undefined;
  host.puzzle.current = INITIAL_PUZZLE_STATE;
  host.puzzle.setState(INITIAL_PUZZLE_STATE);
  host.puzzle.setMaybeLevelId(undefined);
  clearLevelSignals(host);
  host.puzzle.setMaybeBestStars(undefined);
}

/**
 * Space toggles Play/Stop (D-21, D-23): handled in building and running, Play
 * only when allowed. Unhandled in the failure view, so Space keeps its default.
 */
function togglePlay(host: PuzzleHost): boolean {
  const current = host.puzzle.current;
  if (!ownsLiveSession(host)) {
    return false;
  }
  if (current.kind === "building") {
    if (canPlay(current)) {
      playPuzzle(host);
    }
    return true;
  }
  if (current.kind === "running") {
    stopPuzzle(host);
    return true;
  }

  return false;
}

/** Applies a D-23 key command; true when handled so the caller calls `preventDefault`. */
export function applyPuzzleCommand(host: PuzzleHost, command: PuzzleKeyCommand): boolean {
  return command.kind === "togglePlay" ? togglePlay(host) : applyKeyCommand(host, command);
}

/** Lazy controller: reads `host.puzzle` only when an accessor or command runs. */
export function createPuzzleController(host: PuzzleHost): PuzzleController {
  return {
    maybeLevelId: () => host.puzzle.maybeLevelId(),
    state: () => host.puzzle.state(),
    maybeLevelInfo: () => host.puzzle.maybeLevelInfo(),
    maybeBestStars: () => host.puzzle.maybeBestStars(),
    maybeHeldHash: () => host.puzzle.maybeHeldHash(),
    announcement: () => host.puzzle.announcement(),
    play: () => playPuzzle(host),
    stop: () => stopPuzzle(host),
    retry: () => retryPuzzle(host),
    edit: () => editPuzzle(host),
    tryAgain: () => tryAgainPuzzle(host),
    cycleSpeed: () => cycleSpeed(host),
    clear: () => applyPuzzleEdit(host, { kind: "clear" }, "commit"),
    toggleArm: (kind) => toggleArm(host, kind),
    placeAtCenter: (kind) => placeAtCenter(host, kind),
    rotateSelected: (direction) => applyPuzzleEdit(host, { kind: "rotate", direction }, "commit"),
    removeSelected: () => applyPuzzleEdit(host, { kind: "remove" }, "commit"),
    stepValveTime: (direction) => {
      stepValveTime(host, direction);
    },
    command: (command) => applyPuzzleCommand(host, command),
  };
}

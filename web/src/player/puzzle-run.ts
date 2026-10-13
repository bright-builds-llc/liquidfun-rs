import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzleFrameStatus, RenderFrame } from "../physics/frame";
import { buildPuzzleSessionSync, loadPuzzleSession } from "../physics/puzzle-loader";
import type { PuzzleLevelInfo, PuzzleSession } from "../physics/puzzle-session";
import { canonicalPlacements } from "../puzzle/editor";
import { stoppedAnnouncement } from "../puzzle/editor-format";
import { floorPercent } from "../puzzle/format";
import { fnv1a64Lanes, hex16 } from "../puzzle/frame-hash";
import { runStarsForState } from "../puzzle/hud-format";
import { EMPTY_PLACEMENTS, encodePlacementLanes } from "../puzzle/placements";
import { canPlay, type PuzzleEvent, type PuzzleState } from "../puzzle/reduce-puzzle";
import { nextPuzzleSpeed } from "../puzzle/speed";
import { NO_STARS, starNames, type RunStars } from "../puzzle/stars";
import { cancelPendingFrame } from "./frame-loop";
import { isStaleGeneration } from "./generation";
import { cancelPendingInspection, commitInspection } from "./puzzle-editing";
import {
  adoptPuzzleSession,
  disposeCurrentSession,
  dispatch,
  incrementGeneration,
  presentStepZero,
  publishState,
  viewKindFor,
  type PuzzleHost,
} from "./puzzle-host";
import { requirePuzzleStatus, schedulePuzzleFrame } from "./puzzle-loop";

type RunningState = Extract<PuzzleState, { kind: "running" }>;
type PuzzleLanes = ReturnType<typeof encodePlacementLanes>;

/** Lanes for a session build: the run's canonical placements (D-27) or none for a preview. */
function lanesFor(state: PuzzleState, run: boolean): PuzzleLanes {
  return encodePlacementLanes(run ? canonicalPlacements(state.editor) : EMPTY_PLACEMENTS);
}

function requireLevelInfo(host: PuzzleHost): PuzzleLevelInfo {
  const maybeInfo = host.puzzle.maybeLevelInfo();
  if (maybeInfo === undefined) {
    throw new Error("Puzzle level info is unavailable");
  }

  return maybeInfo;
}

/** Captures a never-advanced frame and checks it is a running step 0 (D-07). */
function captureStepZero(owned: PuzzleSession): {
  readonly frame: RenderFrame;
  readonly status: PuzzleFrameStatus;
} {
  const frame = owned.captureFrame();
  const status = requirePuzzleStatus(frame);
  if (frame.stepIndex !== 0 || status.outcome !== "running") {
    throw new Error("Puzzle level did not start at step 0");
  }

  return { frame, status };
}

function recordStars(host: PuzzleHost, stars: RunStars, peakHeldPct: number): void {
  const maybeId = host.puzzle.maybeLevelId();
  if (maybeId === undefined) {
    throw new Error("Puzzle level is unavailable");
  }

  host.puzzle.starStore.record(maybeId, stars, peakHeldPct);
  host.puzzle.setMaybeBestStars(host.puzzle.starStore.get(maybeId));
}

/** Records a won run's stars once, on the running → ended decision (D-10, D-14). */
export function recordDecision(host: PuzzleHost, before: PuzzleState, after: PuzzleState): void {
  if (before.kind !== "running" || after.kind !== "ended" || after.latch.outcome !== "won") {
    return;
  }

  const maybeStars = runStarsForState(after, requireLevelInfo(host).parTools);
  const { latch } = after;
  recordStars(host, maybeStars ?? NO_STARS, floorPercent(latch.peakHeldCount, latch.goalFull));
}

/** Records the stars already latched when a run is stopped (D-15); returns them for the announcement. */
export function recordStop(host: PuzzleHost, running: RunningState): RunStars {
  const stars = runStarsForState(running, requireLevelInfo(host).parTools) ?? NO_STARS;
  if (stars.win) {
    const { status } = running;
    recordStars(host, stars, floorPercent(status.peakHeldCount, status.goalFull));
  }

  return stars;
}

/** Publishes the held frame's FNV-1a 64 hash once per run (D-27). */
export function publishHeldHash(host: PuzzleHost, frame: RenderFrame, state: PuzzleState): void {
  if (state.kind !== "ended" || !state.held || host.puzzle.maybeHeldHash() !== undefined) {
    return;
  }

  const hash = fnv1a64Lanes([frame.particlePositions, frame.rigidSegments, frame.rigidCircles]);
  host.puzzle.setMaybeHeldHash(hex16(hash));
}

function dispatchRecording(host: PuzzleHost, event: PuzzleEvent): PuzzleState {
  const before = host.puzzle.current;
  const after = dispatch(host, event);
  recordDecision(host, before, after);
  return after;
}

function startLoop(host: PuzzleHost, context: CanvasRenderingContext2D): void {
  host.clock.maybeLastTimestamp = undefined;
  host.clock.frameRemainderSeconds = 0;
  const started = host.generation;
  const owned = host.puzzle.maybeOwned;
  schedulePuzzleFrame(host.clock, context, {
    maybeSession: () => host.puzzle.maybeOwned,
    current: () => host.puzzle.current,
    dispatch: (event) => dispatchRecording(host, event),
    publish: (frame, observation, state) => {
      publishState(host, state);
      publishHeldHash(host, frame, state);
      host.setView({ kind: viewKindFor(state), frame: observation });
    },
    drawSceneFrame: host.drawSceneFrame,
    fail: host.failScene,
    isCurrent: () => host.generation === started && host.puzzle.maybeOwned === owned,
  });
}

/**
 * Builds a fresh session for `id` and presents its step-0 frame; never
 * rejects. A pending Retry builds the run's placements; every other build is
 * an empty preview (Research Pattern 3).
 */
export async function buildPuzzle(host: PuzzleHost, id: PuzzleId): Promise<void> {
  const started = incrementGeneration(host);
  const context = host.maybeContext;
  if (host.maybeCanvas === undefined || context === undefined) {
    host.failScene(new Error("Canvas element is unavailable"));
    return;
  }

  try {
    cancelPendingFrame(host.clock);
    cancelPendingInspection(host);
    disposeCurrentSession(host);
    const current = host.puzzle.current;
    const run = current.kind === "loading" && current.pendingRun;
    const generated = await loadPuzzleSession(id, lanesFor(current, run));
    if (isStaleGeneration(started, host.generation)) {
      generated.free();
      return;
    }

    const owned = adoptPuzzleSession(host, generated, id);
    const info = owned.levelInfo;
    const { frame, status } = captureStepZero(owned);
    dispatch(host, { kind: "loaded", stepIndex: frame.stepIndex, status, budget: info.toolBudget });
    host.puzzle.setMaybeLevelInfo(info);
    host.puzzle.setMaybeBestStars(host.puzzle.starStore.get(id));
    // Building is published only with a committed inspection, so Play never lags a load (D-20).
    const state = commitInspection(host);
    presentStepZero(host, context, frame, state);
    if (state.kind === "running") {
      startLoop(host, context);
    }
  } catch (error) {
    if (!isStaleGeneration(started, host.generation)) {
      host.failScene(error);
    }
  }
}

/** building → running on a fresh run session built synchronously from canonical placements (D-21). */
export function playPuzzle(host: PuzzleHost): void {
  const context = host.maybeContext;
  const maybeId = host.puzzle.maybeLevelId();
  const current = host.puzzle.current;
  if (current.kind !== "building" || !canPlay(current)) {
    return;
  }
  if (context === undefined || maybeId === undefined || host.puzzle.maybeOwned === undefined) {
    return;
  }

  incrementGeneration(host);
  try {
    cancelPendingFrame(host.clock);
    cancelPendingInspection(host);
    host.puzzle.setMaybeHeldHash(undefined);
    const generated = buildPuzzleSessionSync(maybeId, lanesFor(current, true));
    const { frame } = captureStepZero(adoptPuzzleSession(host, generated, maybeId));
    const state = dispatch(host, { kind: "play" });
    presentStepZero(host, context, frame, state);
    startLoop(host, context);
  } catch (error) {
    host.failScene(error);
  }
}

/** running → building on a fresh empty preview with the placements kept; records latched stars (D-15, D-21). */
export function stopPuzzle(host: PuzzleHost): void {
  const context = host.maybeContext;
  const maybeId = host.puzzle.maybeLevelId();
  const current = host.puzzle.current;
  if (current.kind !== "running" || context === undefined || maybeId === undefined) {
    return;
  }

  incrementGeneration(host);
  try {
    cancelPendingFrame(host.clock);
    cancelPendingInspection(host);
    const stars = recordStop(host, current);
    host.puzzle.setMaybeHeldHash(undefined);
    const generated = buildPuzzleSessionSync(maybeId, lanesFor(current, false));
    const { frame, status } = captureStepZero(adoptPuzzleSession(host, generated, maybeId));
    dispatch(host, { kind: "stop", stepIndex: frame.stepIndex, status });
    const state = commitInspection(host);
    presentStepZero(host, context, frame, state);
    host.puzzle.setAnnouncement(stoppedAnnouncement(starNames(stars)));
  } catch (error) {
    host.failScene(error);
  }
}

function rebuildFromEnded(host: PuzzleHost, event: "retry" | "edit"): void {
  const maybeId = host.puzzle.maybeLevelId();
  if (host.puzzle.current.kind !== "ended" || maybeId === undefined) {
    return;
  }

  try {
    host.puzzle.setMaybeHeldHash(undefined);
    publishState(host, dispatch(host, { kind: event }));
    host.setView({ kind: "loading" });
  } catch (error) {
    host.failScene(error);
    return;
  }
  void buildPuzzle(host, maybeId);
}

/** ended → running on a fresh session with the run's placements (D-06, D-21). */
export function retryPuzzle(host: PuzzleHost): void {
  rebuildFromEnded(host, "retry");
}

/** ended → building on a fresh empty preview with the placements kept (D-06, D-21). */
export function editPuzzle(host: PuzzleHost): void {
  rebuildFromEnded(host, "edit");
}

/** Cycles 1× → 2× → 4× → 1× (D-24); ignored while loading. */
export function cycleSpeed(host: PuzzleHost): void {
  try {
    const speed = nextPuzzleSpeed(host.puzzle.current.speed);
    publishState(host, dispatch(host, { kind: "setSpeed", speed }));
  } catch (error) {
    host.failScene(error);
  }
}

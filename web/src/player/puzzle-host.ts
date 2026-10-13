import { puzzleTitleForId } from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";
import type { RenderFrame } from "../physics/frame";
import {
  createPuzzleSession,
  type GeneratedPuzzleSession,
  type PuzzleSession,
} from "../physics/puzzle-session";
import { goalTicks } from "../puzzle/format";
import { reducePuzzle, type PuzzleEvent, type PuzzleState } from "../puzzle/reduce-puzzle";
import { glassCup } from "../render/glass-cup";
import { nextGeneration } from "./generation";
import { observeFrame } from "./observe";
import type { PuzzleRuntime } from "./puzzle-runtime";
import type { SceneRuntime } from "./scene-runtime";

/** Scene runtime fields the puzzle shell uses, plus the puzzle bag. */
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

/** Supersedes every in-flight build and loop; returns the new generation. */
export function incrementGeneration(host: PuzzleHost): number {
  host.generation = nextGeneration(host.generation);
  return host.generation;
}

/** Folds one event into the non-reactive mirror and returns the new state. */
export function dispatch(host: PuzzleHost, event: PuzzleEvent): PuzzleState {
  host.puzzle.current = reducePuzzle(host.puzzle.current, event);
  return host.puzzle.current;
}

/** Publishes the Solid puzzle state. */
export function publishState(host: PuzzleHost, state: PuzzleState): void {
  host.puzzle.setState(state);
}

/** Releases the live session; a throwing dispose propagates to the caller's failure path. */
export function disposeCurrentSession(host: PuzzleHost): void {
  host.maybeWorkerInitialization?.abort();
  delete host.maybeWorkerInitialization;
  host.maybeCanvasPointer?.cancel();
  const maybeOwned = host.maybeSession;
  host.maybeSession = undefined;
  host.puzzle.maybeOwned = undefined;
  host.clock.maybeLastTimestamp = undefined;
  maybeOwned?.dispose();
}

/** Player view kind for a puzzle phase: playing while steps still run. */
export function viewKindFor(state: PuzzleState): "playing" | "paused" {
  const stepping = state.kind === "running" || (state.kind === "ended" && !state.held);
  return stepping ? "playing" : "paused";
}

/**
 * Wraps a generated session for `id`, checks its title, disposes the previous
 * session and installs the new one with its glass overlay. The generated
 * session is freed on every failure before the throw.
 */
export function adoptPuzzleSession(
  host: PuzzleHost,
  generated: GeneratedPuzzleSession,
  id: PuzzleId,
): PuzzleSession {
  const canvas = host.maybeCanvas;
  if (canvas === undefined) {
    generated.free();
    throw new Error("Canvas element is unavailable");
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

  try {
    disposeCurrentSession(host);
  } catch (error) {
    owned.dispose();
    throw error;
  }
  host.maybeSession = owned;
  host.puzzle.maybeOwned = owned;
  host.puzzle.maybeGlass = glassCup(
    info.cupInterior,
    info.targetCount,
    info.fullCount,
    goalTicks(info.targetCount, info.fullCount, info.brimCount),
  );
  canvas.dataset.simulationBackend = "direct";
  return owned;
}

/** Draws a never-advanced step-0 frame (D-07) and publishes `state` with it. */
export function presentStepZero(
  host: PuzzleHost,
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  state: PuzzleState,
): void {
  const maybeCamera = host.clock.maybeCamera;
  if (maybeCamera !== undefined) {
    host.drawSceneFrame(context, frame, maybeCamera);
  }
  host.clock.maybePreviousFrame = frame;
  publishState(host, state);
  host.setView({ kind: viewKindFor(state), frame: observeFrame(frame, undefined, 0) });
}

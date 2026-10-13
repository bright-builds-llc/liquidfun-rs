import { createSignal, type Accessor, type Setter } from "solid-js";

import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import type { PuzzleLevelInfo, PuzzleSession } from "../physics/puzzle-session";
import { maybeLevelStars } from "../puzzle/progress";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../puzzle/reduce-puzzle";
import { createStarStore, type StarStore } from "../puzzle/star-store";
import type { LevelStars } from "../puzzle/stars";
import type { GlassCup } from "../render/glass-cup";
import type { PuzzleEditorInput } from "./puzzle-editor-input";
import {
  UNAVAILABLE_PROGRESS_STORAGE,
  createPuzzleProgress,
  type PuzzleProgress,
} from "./puzzle-progress";

/** Mutable puzzle state shared by the lifecycle, loop, surface and UI. */
export type PuzzleRuntime = {
  readonly state: Accessor<PuzzleState>;
  readonly setState: Setter<PuzzleState>;
  readonly maybeLevelId: Accessor<PuzzleId | undefined>;
  readonly setMaybeLevelId: Setter<PuzzleId | undefined>;
  /** Parsed info of the loaded level; undefined until a level loads. */
  readonly maybeLevelInfo: Accessor<PuzzleLevelInfo | undefined>;
  readonly setMaybeLevelInfo: Setter<PuzzleLevelInfo | undefined>;
  /** Best-of stars for the current level (D-14). */
  readonly maybeBestStars: Accessor<LevelStars | undefined>;
  readonly setMaybeBestStars: Setter<LevelStars | undefined>;
  /** 16 hex digits of the held frame's FNV-1a 64 hash, set once per run (D-27). */
  readonly maybeHeldHash: Accessor<string | undefined>;
  readonly setMaybeHeldHash: Setter<string | undefined>;
  /** Latest committed editor announcement for the polite live region (UI-D14). */
  readonly announcement: Accessor<string>;
  readonly setAnnouncement: Setter<string>;
  /** Best-of stars: persisted wins plus this page session (D-09, D-14). */
  readonly starStore: StarStore;
  /** Per-browser progress; hydrates the star store (D-09). */
  readonly progress: PuzzleProgress;
  /** Non-reactive mirror the loop folds per-step events into; Solid state is published once per frame. */
  current: PuzzleState;
  maybeGlass: GlassCup | undefined;
  maybeOwned: PuzzleSession | undefined;
  /** Pending coalesced live inspection (one per animation frame, D-02). */
  maybeInspectionFrame: number | undefined;
  /** Building-only canvas editor pointer; separate from the scene pointer (Pitfall 6). */
  maybeEditorInput: PuzzleEditorInput | undefined;
};

function hydratedStarStore(progress: PuzzleProgress): StarStore {
  const starStore = createStarStore();
  for (const id of PUZZLE_IDS) {
    const maybeStars = maybeLevelStars(progress.current(), id);
    if (maybeStars !== undefined) {
      starStore.record(id, maybeStars, maybeStars.bestPeakHeldPct);
    }
  }
  return starStore;
}

/**
 * Creates an idle puzzle runtime; call inside a Solid root so the signals have an owner.
 * Without a progress argument the runtime is degraded: nothing persists (D-14).
 */
export function createPuzzleRuntime(
  progress: PuzzleProgress = createPuzzleProgress(UNAVAILABLE_PROGRESS_STORAGE),
): PuzzleRuntime {
  const [state, setState] = createSignal<PuzzleState>(INITIAL_PUZZLE_STATE);
  const [maybeLevelId, setMaybeLevelId] = createSignal<PuzzleId | undefined>(undefined);
  const [maybeLevelInfo, setMaybeLevelInfo] = createSignal<PuzzleLevelInfo | undefined>(
    undefined,
  );
  const [maybeBestStars, setMaybeBestStars] = createSignal<LevelStars | undefined>(undefined);
  const [maybeHeldHash, setMaybeHeldHash] = createSignal<string | undefined>(undefined);
  const [announcement, setAnnouncement] = createSignal("");
  return {
    state,
    setState,
    maybeLevelId,
    setMaybeLevelId,
    maybeLevelInfo,
    setMaybeLevelInfo,
    maybeBestStars,
    setMaybeBestStars,
    maybeHeldHash,
    setMaybeHeldHash,
    announcement,
    setAnnouncement,
    starStore: hydratedStarStore(progress),
    progress,
    current: INITIAL_PUZZLE_STATE,
    maybeGlass: undefined,
    maybeOwned: undefined,
    maybeInspectionFrame: undefined,
    maybeEditorInput: undefined,
  };
}

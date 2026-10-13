import { createSignal, type Accessor, type Setter } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzleLevelInfo, PuzzleSession } from "../physics/puzzle-session";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../puzzle/reduce-puzzle";
import { createStarStore, type StarStore } from "../puzzle/star-store";
import type { LevelStars } from "../puzzle/stars";
import type { GlassCup } from "../render/glass-cup";

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
  /** Page-session best-of stars; survives route changes but not a reload (D-14). */
  readonly starStore: StarStore;
  /** Non-reactive mirror the loop folds per-step events into; Solid state is published once per frame. */
  current: PuzzleState;
  maybeGlass: GlassCup | undefined;
  maybeOwned: PuzzleSession | undefined;
  /** Pending coalesced live inspection (one per animation frame, D-02). */
  maybeInspectionFrame: number | undefined;
};

/** Creates an idle puzzle runtime; call inside a Solid root so the signals have an owner. */
export function createPuzzleRuntime(): PuzzleRuntime {
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
    starStore: createStarStore(),
    current: INITIAL_PUZZLE_STATE,
    maybeGlass: undefined,
    maybeOwned: undefined,
    maybeInspectionFrame: undefined,
  };
}

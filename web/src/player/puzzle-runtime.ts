import { createSignal, type Accessor, type Setter } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzleSession } from "../physics/puzzle-session";
import { EMPTY_PLACEMENTS, type PuzzlePlacement } from "../puzzle/placements";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../puzzle/reduce-puzzle";
import type { GlassCup } from "../render/glass-cup";

/** Mutable puzzle state shared by the lifecycle, loop, surface and UI. */
export type PuzzleRuntime = {
  readonly state: Accessor<PuzzleState>;
  readonly setState: Setter<PuzzleState>;
  readonly maybeLevelId: Accessor<PuzzleId | undefined>;
  readonly setMaybeLevelId: Setter<PuzzleId | undefined>;
  /** Non-reactive mirror the loop folds per-step events into; Solid state is published once per frame. */
  current: PuzzleState;
  /** Retained across Retry/Edit; read once per route start (D-04, D-22). */
  placements: readonly PuzzlePlacement[];
  maybeGlass: GlassCup | undefined;
  maybeOwned: PuzzleSession | undefined;
};

/** Creates an idle puzzle runtime; call inside a Solid root so the signals have an owner. */
export function createPuzzleRuntime(): PuzzleRuntime {
  const [state, setState] = createSignal<PuzzleState>(INITIAL_PUZZLE_STATE);
  const [maybeLevelId, setMaybeLevelId] = createSignal<PuzzleId | undefined>(undefined);
  return {
    state,
    setState,
    maybeLevelId,
    setMaybeLevelId,
    current: INITIAL_PUZZLE_STATE,
    placements: EMPTY_PLACEMENTS,
    maybeGlass: undefined,
    maybeOwned: undefined,
  };
}

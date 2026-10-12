import { PUZZLE_IDS, type PuzzleId } from "./puzzles";

/** Level titles available before wasm loads; checked against `levelInfo().title` at load (D-26). */
export const PUZZLE_TITLES: Readonly<Record<PuzzleId, string>> = {
  "first-pour": "First Pour",
};

/** First registry level. The level-select entry point and default puzzle link. */
export const FIRST_PUZZLE_ID: PuzzleId = firstPuzzleId();

function firstPuzzleId(): PuzzleId {
  const maybeId = PUZZLE_IDS[0];
  if (maybeId === undefined) {
    throw new Error("The puzzle registry is empty.");
  }

  return maybeId;
}

/** Display title for an allowlisted level id. */
export function puzzleTitleForId(id: PuzzleId): string {
  return PUZZLE_TITLES[id];
}

/** Canonical playground hash for a level. */
export function puzzleHash(id: PuzzleId): string {
  return `#/puzzle/${id}`;
}

/** Next level in registry order, or undefined after the last one (D-15). */
export function maybeNextPuzzleId(id: PuzzleId): PuzzleId | undefined {
  return PUZZLE_IDS[PUZZLE_IDS.indexOf(id) + 1];
}

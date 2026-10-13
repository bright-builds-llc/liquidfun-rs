import { PUZZLE_IDS, type PuzzleId } from "./puzzles";

/** Level titles available before wasm loads; checked against `levelInfo().title` at load (D-26). */
export const PUZZLE_TITLES: Readonly<Record<PuzzleId, string>> = {
  "first-pour": "First Pour",
  "thick-syrup": "Thick Syrup",
  "jelly-pad": "Jelly Pad",
  "float-up": "Float Up",
  "timed-gate": "Timed Gate",
  "grand-finale": "Grand Finale",
};

/**
 * One-line teaching captions, shown on each level's caption leaf before wasm
 * loads (D-21). Unlike titles, they are not cross-checked against Rust.
 */
export const PUZZLE_CAPTIONS: Readonly<Record<PuzzleId, string>> = {
  "first-pour": "Water runs downhill. Tilt a plank to pour it into the cup.",
  "thick-syrup": "Syrup is thick and slow. Give it a steep slide.",
  "jelly-pad": "Jelly never counts, but water slides across it. Drop the water onto the pad.",
  "float-up": "The ball floats. Fill its well and it rises to plug the drain.",
  "timed-gate": "The valve opens on a timer. Catch the water first, then let the syrup out.",
  "grand-finale": "Finale: drop water on the jelly, catch it with the valve, then free the syrup.",
};

/** First-level placement hint (LEVEL-03), shown on the caption leaf until dismissed. */
export const FIRST_LEVEL_HINT = "Tap the plank in the tray, then tap the canvas to place it.";

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

/** One-line teaching caption for an allowlisted level id. */
export function puzzleCaptionForId(id: PuzzleId): string {
  return PUZZLE_CAPTIONS[id];
}

/** Canonical playground hash for a level. */
export function puzzleHash(id: PuzzleId): string {
  return `#/puzzle/${id}`;
}

/** Next level in registry order, or undefined after the last one (D-15). */
export function maybeNextPuzzleId(id: PuzzleId): PuzzleId | undefined {
  return PUZZLE_IDS[PUZZLE_IDS.indexOf(id) + 1];
}

import { FIRST_LEVEL_HINT, FIRST_PUZZLE_ID, puzzleCaptionForId } from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";

/** Content of the caption HUD leaf: one line of text and whether Dismiss shows. */
export type CaptionModel = {
  readonly text: string;
  readonly showDismiss: boolean;
};

/**
 * The first level shows the placement hint until it is dismissed (D-19,
 * D-20); every other case shows the level caption (D-21).
 */
export function captionModel(levelId: PuzzleId, hintDismissed: boolean): CaptionModel {
  if (levelId === FIRST_PUZZLE_ID && !hintDismissed) {
    return { text: FIRST_LEVEL_HINT, showDismiss: true };
  }
  return { text: puzzleCaptionForId(levelId), showDismiss: false };
}

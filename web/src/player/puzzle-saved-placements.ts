import type { PuzzleId } from "../catalog/puzzles";
import { placementsInOrder } from "../puzzle/editor";
import { savedPlacementsFor } from "../puzzle/progress-view";
import { dispatch, type PuzzleHost } from "./puzzle-host";

/**
 * Restores a level's saved placements once per route entry (D-18). Runs in
 * `buildPuzzle` after the session is adopted (so the live level hash exists)
 * and before `loaded`, while the state is still a non-run `loading`; Retry,
 * Edit, Stop and Try again never set the flag (Pitfall 5). Publishing is left
 * to `buildPuzzle`, which publishes after `loaded` and the committed inspection.
 */
export function restorePendingPlacements(host: PuzzleHost, id: PuzzleId, levelHash: string): void {
  const current = host.puzzle.current;
  if (host.puzzle.maybePendingRestoreId !== id || current.kind !== "loading" || current.pendingRun) {
    return;
  }

  host.puzzle.maybePendingRestoreId = undefined;
  const maybeRestored = host.puzzle.progress.maybeRestore(id, levelHash);
  if (maybeRestored !== undefined) {
    dispatch(host, { kind: "reset", placements: maybeRestored, speed: current.speed });
  }
}

/**
 * Saves the editor's placements in creation order with the live level hash
 * (D-16). Skips when no level is loaded or nothing changed since the last
 * save for this level. Storage failures degrade inside the progress shell
 * and never reach the edit path.
 */
export function saveEditorPlacements(host: PuzzleHost): void {
  const maybeId = host.puzzle.maybeLevelId();
  const maybeInfo = host.puzzle.maybeLevelInfo();
  if (maybeId === undefined || maybeInfo === undefined) {
    return;
  }

  const placements = placementsInOrder(host.puzzle.current.editor);
  const { levelHash } = maybeInfo;
  const tools = JSON.stringify(savedPlacementsFor(levelHash, placements).tools);
  const key = `${maybeId}|${levelHash}|${tools}`;
  if (key === host.puzzle.maybeLastSavedKey) {
    return;
  }

  host.puzzle.maybeLastSavedKey = key;
  host.puzzle.progress.savePlacements(maybeId, levelHash, placements);
}

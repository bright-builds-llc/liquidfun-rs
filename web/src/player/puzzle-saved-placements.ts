import type { PuzzleId } from "../catalog/puzzles";
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

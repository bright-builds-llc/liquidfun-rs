import type { Accessor } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzleState } from "./reduce-puzzle";

/** What the puzzle panel reads and commands; implemented by `player/puzzle-lifecycle.ts`. */
export interface PuzzleController {
  /** Level of the current puzzle route, undefined outside puzzle routes. */
  readonly maybeLevelId: Accessor<PuzzleId | undefined>;
  /** Current puzzle phase, published once per presented frame. */
  readonly state: Accessor<PuzzleState>;
  /** building -> running on the same never-advanced session (D-07). */
  play(): void;
  /** ended -> running on a fresh session with the same placements (D-06). */
  retry(): void;
  /** ended -> building on a fresh step-0 session with the same placements (D-06). */
  edit(): void;
  /** After a failure, rebuilds the level into building. */
  tryAgain(): void;
}

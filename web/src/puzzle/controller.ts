import type { Accessor } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzleLevelInfo } from "../physics/puzzle-session";
import type { PuzzleKeyCommand } from "./keyboard";
import type { PuzzleToolKind } from "./placements";
import type { PuzzleState } from "./reduce-puzzle";
import type { LevelStars } from "./stars";

/** What the puzzle panel reads and commands; implemented by `player/puzzle-lifecycle.ts`. */
export interface PuzzleController {
  /** Level of the current puzzle route, undefined outside puzzle routes. */
  readonly maybeLevelId: Accessor<PuzzleId | undefined>;
  /** Current puzzle phase, published once per presented frame. */
  readonly state: Accessor<PuzzleState>;
  /** Parsed info of the loaded level; undefined until it loads. */
  readonly maybeLevelInfo: Accessor<PuzzleLevelInfo | undefined>;
  /** Best-of stars for the current level in this page session (D-14). */
  readonly maybeBestStars: Accessor<LevelStars | undefined>;
  /** 16 hex digits of the held frame's FNV-1a 64 hash, set once when held (D-27). */
  readonly maybeHeldHash: Accessor<string | undefined>;
  /** Latest committed editor announcement for the polite live region (UI-D14). */
  readonly announcement: Accessor<string>;
  /** True when progress cannot be saved in this browser (D-15). */
  readonly progressDegraded: Accessor<boolean>;
  /** Shared on/off state of the HUD toggle and the drawer checkbox (Phase 42 D-01/D-02). */
  readonly soundEnabled: Accessor<boolean>;
  /** False when Web Audio is missing or construction failed (D-05). */
  readonly soundAvailable: Accessor<boolean>;
  /** True after a runtime audio failure this session (D-16). */
  readonly soundFailed: Accessor<boolean>;
  /** Call from the click/change handler; turning on unlocks audio there (D-04). */
  setSoundEnabled(on: boolean): void;
  /** building → running on a fresh run session built from the placements (D-21). */
  play(): void;
  /** running → building on a fresh preview with the placements kept (D-21). */
  stop(): void;
  /** ended → running on a fresh session with the same placements (D-06). */
  retry(): void;
  /** ended → building on a fresh step-0 preview with the same placements (D-06). */
  edit(): void;
  /** After a failure, rebuilds the level into building. */
  tryAgain(): void;
  /** Cycles the run speed 1× → 2× → 4× (D-24). */
  cycleSpeed(): void;
  /** Removes every tool in building, without confirmation (D-21). */
  clear(): void;
  /** Arms a tray kind, or disarms it when already armed (D-17). */
  toggleArm(kind: PuzzleToolKind): void;
  /** Places one tool at the build-area centre and selects it (D-23). */
  placeAtCenter(kind: PuzzleToolKind): void;
  /** Rotates the selected tool one τ/24 step; +1 is counterclockwise (D-19). */
  rotateSelected(direction: 1 | -1): void;
  /** Removes the selected tool (D-19). */
  removeSelected(): void;
  /** Moves the selected valve's open time by 0.5 s while building (D-05). */
  stepValveTime(direction: 1 | -1): void;
  /** Applies a D-23 key command; true when handled so the caller calls preventDefault. */
  command(command: PuzzleKeyCommand): boolean;
}

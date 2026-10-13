import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import { mergeLevelStars, type LevelStars, type RunStars } from "./stars";

/** Null-prototype `{levelId → LevelStars}` object; Phase 41 serializes it as-is (D-14). */
export type StarStoreSnapshot = { readonly [levelId: string]: LevelStars };

/** In-memory best-of stars for one page session. Each runtime owns its own store. */
export interface StarStore {
  /** The stored best for a level, or undefined before its first win. */
  get(levelId: PuzzleId): LevelStars | undefined;
  /** Merges a run; returns the stored best (D-14, D-15). Throws "Unknown puzzle level" for ids outside PUZZLE_IDS. */
  record(levelId: PuzzleId, run: RunStars, peakHeldPct: number): LevelStars | undefined;
  /** A fresh JSON-safe copy holding only recorded levels. */
  snapshot(): StarStoreSnapshot;
}

const KNOWN_LEVEL_IDS: ReadonlySet<string> = new Set(PUZZLE_IDS);

function copyLevelStars(stars: LevelStars): LevelStars {
  return {
    win: stars.win,
    brim: stars.brim,
    thrifty: stars.thrifty,
    bestPeakHeldPct: stars.bestPeakHeldPct,
  };
}

/** Creates an empty Map-backed store with allowlisted keys (T-39-09). */
export function createStarStore(): StarStore {
  const bestByLevel = new Map<PuzzleId, LevelStars>();

  return {
    get(levelId) {
      const maybeBest = bestByLevel.get(levelId);
      return maybeBest === undefined ? undefined : copyLevelStars(maybeBest);
    },

    record(levelId, run, peakHeldPct) {
      if (!KNOWN_LEVEL_IDS.has(levelId)) {
        throw new Error(`Unknown puzzle level: ${String(levelId)}`);
      }

      const maybeBest = mergeLevelStars(bestByLevel.get(levelId), run, peakHeldPct);
      if (maybeBest === undefined) {
        return undefined;
      }

      bestByLevel.set(levelId, copyLevelStars(maybeBest));
      return copyLevelStars(maybeBest);
    },

    snapshot() {
      const snapshot = Object.create(null) as Record<string, LevelStars>;
      for (const [levelId, best] of bestByLevel) {
        snapshot[levelId] = copyLevelStars(best);
      }
      return snapshot;
    },
  };
}

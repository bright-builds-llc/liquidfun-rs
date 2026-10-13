/** Stars one run earned (D-13). Every star requires a win. */
export type RunStars = { readonly win: boolean; readonly brim: boolean; readonly thrifty: boolean };

/** A run that earned nothing, such as any loss. */
export const NO_STARS: RunStars = { win: false, brim: false, thrifty: false };

/** Best result per criterion across runs, plus the best peak-held fill (D-14). */
export type LevelStars = RunStars & { readonly bestPeakHeldPct: number };

/** What a run reports for scoring; thresholds come from the level, never TS constants (D-13). */
export type RunStarsInput = {
  readonly won: boolean;
  readonly brimLatched: boolean;
  readonly placementsAtPlay: number;
  readonly parTools: number;
};

/** Win = won; Brim = won and Brim latched; Thrifty = won within par placements at Play (D-13). */
export function runStars(input: RunStarsInput): RunStars {
  if (!input.won) {
    return NO_STARS;
  }

  return {
    win: true,
    brim: input.brimLatched,
    thrifty: input.placementsAtPlay <= input.parTools,
  };
}

/** Per-criterion OR merge and max %, never lowers (D-14); undefined stays undefined unless the run won. */
export function mergeLevelStars(
  maybePrevious: LevelStars | undefined,
  run: RunStars,
  peakHeldPct: number,
): LevelStars | undefined {
  if (!Number.isFinite(peakHeldPct) || peakHeldPct < 0 || peakHeldPct > 100) {
    throw new Error(`Invalid peak held percent: ${peakHeldPct}`);
  }

  if (!run.win) {
    return maybePrevious;
  }

  if (maybePrevious === undefined) {
    return { ...run, bestPeakHeldPct: peakHeldPct };
  }

  return {
    win: true,
    brim: maybePrevious.brim || run.brim,
    thrifty: maybePrevious.thrifty || run.thrifty,
    bestPeakHeldPct: Math.max(maybePrevious.bestPeakHeldPct, peakHeldPct),
  };
}

/** Display name of one star criterion. */
export type StarName = "Win" | "Brim" | "Thrifty";

/** Names of the stars a run latched, in the fixed Win, Brim, Thrifty order. */
export function starNames(run: RunStars): readonly StarName[] {
  const names: StarName[] = [];
  if (run.win) {
    names.push("Win");
  }
  if (run.brim) {
    names.push("Brim");
  }
  if (run.thrifty) {
    names.push("Thrifty");
  }
  return names;
}

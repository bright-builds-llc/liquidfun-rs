import { PUZZLE_CAPTIONS, PUZZLE_TITLES, puzzleHash } from "../catalog/puzzle-levels";
import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import { maybeParsePlacement, type PuzzlePlacement } from "./placements";
import { maybeLevelStars, type ProgressDocument, type SavedPlacements } from "./progress";
import { NO_STARS, type LevelStars, type RunStars } from "./stars";

/** Mirrors Rust `MAX_PLACEMENTS`; longer saved lists are discarded (Pitfall 4). */
export const MAX_SAVED_PLACEMENTS = 16;

/** A level is solved once its Win star is earned (D-09). */
export function isSolved(progress: ProgressDocument, id: PuzzleId): boolean {
  return progress.levels[id]?.win === true;
}

/**
 * Index 0 always; index i when level i−1 is solved; every level when degraded (D-10).
 * A level solved through a deep link also stays unlocked so its card can reopen it.
 */
export function unlockedLevelIds(progress: ProgressDocument, degraded: boolean): ReadonlySet<PuzzleId> {
  if (degraded) {
    return new Set(PUZZLE_IDS);
  }

  return new Set(
    PUZZLE_IDS.filter((id, index) => {
      const maybePreviousId = PUZZLE_IDS[index - 1];
      return maybePreviousId === undefined || isSolved(progress, id) || isSolved(progress, maybePreviousId);
    }),
  );
}

/** Number of levels with a Win star. */
export function solvedCount(progress: ProgressDocument): number {
  return PUZZLE_IDS.filter((id) => isSolved(progress, id)).length;
}

/** Everything one grid card renders; built only from TS metadata and numbers (T-41-08). */
export type LevelCardModel = {
  readonly id: PuzzleId;
  readonly number: number;
  readonly title: string;
  readonly caption: string;
  readonly stars: RunStars;
  readonly maybeBestFillPct: number | undefined;
  readonly locked: boolean;
  readonly solved: boolean;
  readonly isNextUp: boolean;
  /** "Solve {previous title} to unlock", "Best fill {pct}%" or "Not solved yet" (UI-D21). */
  readonly footer: string;
  /** "Level {n}, {title}, locked" for the locked card's group name. */
  readonly lockedLabel: string;
  readonly href: string;
};

function cardFooter(
  locked: boolean,
  maybePreviousId: PuzzleId | undefined,
  maybeStars: LevelStars | undefined,
): string {
  if (locked) {
    if (maybePreviousId === undefined) {
      throw new Error("The first level is never locked");
    }
    return `Solve ${PUZZLE_TITLES[maybePreviousId]} to unlock`;
  }
  if (maybeStars !== undefined) {
    return `Best fill ${Math.floor(maybeStars.bestPeakHeldPct)}%`;
  }
  return "Not solved yet";
}

function cardModel(
  progress: ProgressDocument,
  unlocked: ReadonlySet<PuzzleId>,
  id: PuzzleId,
  index: number,
): Omit<LevelCardModel, "isNextUp"> {
  const number = index + 1;
  const title = PUZZLE_TITLES[id];
  const maybeStars = maybeLevelStars(progress, id);
  const locked = !unlocked.has(id);
  const stars = maybeStars ?? NO_STARS;

  return {
    id,
    number,
    title,
    caption: PUZZLE_CAPTIONS[id],
    stars: { win: stars.win, brim: stars.brim, thrifty: stars.thrifty },
    maybeBestFillPct: maybeStars === undefined ? undefined : Math.floor(maybeStars.bestPeakHeldPct),
    locked,
    solved: maybeStars !== undefined,
    footer: cardFooter(locked, PUZZLE_IDS[index - 1], maybeStars),
    lockedLabel: `Level ${number}, ${title}, locked`,
    href: puzzleHash(id),
  };
}

/** One card per registry level, in order; the first unlocked unsolved card is next up (UI-D22). */
export function levelCardModels(progress: ProgressDocument, degraded: boolean): readonly LevelCardModel[] {
  const unlocked = unlockedLevelIds(progress, degraded);
  const cards = PUZZLE_IDS.map((id, index) => cardModel(progress, unlocked, id, index));
  const nextUpIndex = cards.findIndex((model) => !model.locked && !model.solved);

  return cards.map((model, index) => ({ ...model, isNextUp: index === nextUpIndex }));
}

/** What level entry does with a level's saved placements (D-18). */
export type RestoreChoice =
  | { readonly kind: "none" }
  | { readonly kind: "restore"; readonly placements: readonly PuzzlePlacement[] }
  | { readonly kind: "discard" };

/** Restores only for the live level hash, at most the lane cap, and when every entry parses strictly. */
export function restoreChoice(progress: ProgressDocument, id: PuzzleId, liveHash: string): RestoreChoice {
  const maybeSaved = progress.levels[id]?.maybePlacements;
  if (maybeSaved === undefined) {
    return { kind: "none" };
  }
  if (maybeSaved.levelHash !== liveHash || maybeSaved.tools.length > MAX_SAVED_PLACEMENTS) {
    return { kind: "discard" };
  }

  const placements: PuzzlePlacement[] = [];
  for (const entry of maybeSaved.tools) {
    const maybePlacement = maybeParsePlacement(entry);
    if (maybePlacement === undefined) {
      return { kind: "discard" };
    }
    placements.push(maybePlacement);
  }

  return { kind: "restore", placements };
}

/** Saved form of the editor's placements, as fresh records under the live level hash. */
export function savedPlacementsFor(levelHash: string, placements: readonly PuzzlePlacement[]): SavedPlacements {
  return {
    levelHash,
    tools: placements.map(({ kind, x, y, angle, openTick }) => ({ kind, x, y, angle, openTick })),
  };
}

import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import { mergeLevelStars, type LevelStars } from "./stars";

/** One key holds the whole document (D-13). */
export const PROGRESS_STORAGE_KEY = "liquidfun.puzzle-progress.v1";

/** The only document version this build reads and writes. */
export const PROGRESS_VERSION = 1;

/** Rust `levelInfo().levelHash()` form (D-17). */
export const LEVEL_HASH_PATTERN = /^[0-9a-f]{16}$/;

/** Last placements for a level; `tools` stay raw until restore validates them (D-18). */
export type SavedPlacements = { readonly levelHash: string; readonly tools: readonly unknown[] };

/** Stars plus optional saved placements for one known level. */
export type LevelProgress = LevelStars & { readonly maybePlacements: SavedPlacements | undefined };

type LevelRecords = Partial<Record<PuzzleId, LevelProgress>>;
type RawRecords = Record<string, unknown>;

/** Parsed v1 document. Every record is null-prototype. */
export type ProgressDocument = {
  readonly levels: Readonly<LevelRecords>;
  /** Raw JSON of level ids this build does not know, written back verbatim (D-13). */
  readonly unknownLevels: Readonly<RawRecords>;
  /** Raw top-level keys other than `version` and `levels`, written back verbatim. */
  readonly extra: Readonly<RawRecords>;
};

/** Why a stored value cannot be used; each turns on degraded mode (D-14). */
export type UnusableReason = "json" | "shape" | "future-version";

/** Result of reading the stored text. */
export type ParsedProgress =
  | { readonly kind: "ok"; readonly progress: ProgressDocument }
  | { readonly kind: "unusable"; readonly reason: UnusableReason };

const KNOWN_LEVEL_IDS: ReadonlySet<string> = new Set(PUZZLE_IDS);
const NO_LEVEL_STARS: LevelStars = { win: false, brim: false, thrifty: false, bestPeakHeldPct: 0 };

function emptyLevels(): LevelRecords {
  return Object.create(null) as LevelRecords;
}

function emptyRaw(): RawRecords {
  return Object.create(null) as RawRecords;
}

/** No progress yet: no levels, no unknown ids, no extra keys. */
export const EMPTY_PROGRESS: ProgressDocument = Object.freeze({
  levels: Object.freeze(emptyLevels()),
  unknownLevels: Object.freeze(emptyRaw()),
  extra: Object.freeze(emptyRaw()),
});

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isKnownLevelId(id: string): id is PuzzleId {
  return KNOWN_LEVEL_IDS.has(id);
}

function unusable(reason: UnusableReason): ParsedProgress {
  return { kind: "unusable", reason };
}

function copyStars(stars: LevelStars): LevelStars {
  return {
    win: stars.win,
    brim: stars.brim,
    thrifty: stars.thrifty,
    bestPeakHeldPct: stars.bestPeakHeldPct,
  };
}

function parsePlacements(value: unknown): SavedPlacements | undefined {
  if (!isPlainRecord(value)) {
    return undefined;
  }

  const levelHash = value["levelHash"];
  const tools = value["tools"];
  if (typeof levelHash !== "string" || !LEVEL_HASH_PATTERN.test(levelHash) || !Array.isArray(tools)) {
    return undefined;
  }

  return { levelHash, tools: [...(tools as readonly unknown[])] };
}

/** Strict star fields; unknown fields in the record are ignored and not written back. */
function parseLevel(value: unknown): LevelProgress | undefined {
  if (!isPlainRecord(value)) {
    return undefined;
  }

  const { win, brim, thrifty, bestPeakHeldPct } = value;
  if (typeof win !== "boolean" || typeof brim !== "boolean" || typeof thrifty !== "boolean") {
    return undefined;
  }
  if (typeof bestPeakHeldPct !== "number" || !Number.isFinite(bestPeakHeldPct)) {
    return undefined;
  }
  if (bestPeakHeldPct < 0 || bestPeakHeldPct > 100) {
    return undefined;
  }
  // Every star requires a win, as in `mergeLevelStars`.
  if ((brim || thrifty) && !win) {
    return undefined;
  }

  // D-18: a bad placements subtree is discarded silently and stars are kept, so it never triggers degraded mode (unlike the D-14 star fields).
  const maybePlacements = parsePlacements(value["placements"]);
  return { win, brim, thrifty, bestPeakHeldPct, maybePlacements };
}

type ParsedLevels = { readonly levels: LevelRecords; readonly unknownLevels: RawRecords };

function parseLevels(value: Record<string, unknown>): ParsedLevels | undefined {
  const levels = emptyLevels();
  const unknownLevels = emptyRaw();
  // Object.entries reads own keys only, so a JSON "__proto__" key stays plain data.
  for (const [id, record] of Object.entries(value)) {
    if (!isKnownLevelId(id)) {
      unknownLevels[id] = record;
      continue;
    }

    const maybeLevel = parseLevel(record);
    if (maybeLevel === undefined) {
      return undefined;
    }
    levels[id] = maybeLevel;
  }

  return { levels, unknownLevels };
}

function parseExtra(value: Record<string, unknown>): RawRecords {
  const extra = emptyRaw();
  for (const [key, entry] of Object.entries(value)) {
    if (key !== "version" && key !== "levels") {
      extra[key] = entry;
    }
  }
  return extra;
}

function parseJson(text: string): { readonly value: unknown } | undefined {
  try {
    return { value: JSON.parse(text) as unknown };
  } catch {
    // The caller reports this as the typed `json` reason.
    return undefined;
  }
}

/** Parses the stored text fail-closed (D-13, D-14); a missing value is empty progress. */
export function parseProgress(maybeRaw: string | null): ParsedProgress {
  if (maybeRaw === null) {
    return { kind: "ok", progress: EMPTY_PROGRESS };
  }

  const maybeJson = parseJson(maybeRaw);
  if (maybeJson === undefined) {
    return unusable("json");
  }

  const value = maybeJson.value;
  if (!isPlainRecord(value)) {
    return unusable("shape");
  }

  const version = value["version"];
  if (typeof version === "number" && Number.isInteger(version) && version > PROGRESS_VERSION) {
    return unusable("future-version");
  }

  const rawLevels = value["levels"];
  if (version !== PROGRESS_VERSION || !isPlainRecord(rawLevels)) {
    return unusable("shape");
  }

  const maybeLevels = parseLevels(rawLevels);
  if (maybeLevels === undefined) {
    return unusable("shape");
  }

  return {
    kind: "ok",
    progress: { ...maybeLevels, extra: parseExtra(value) },
  };
}

function serializeLevel(level: LevelProgress): Record<string, unknown> {
  const stars: Record<string, unknown> = copyStars(level);
  if (level.maybePlacements === undefined) {
    return stars;
  }

  const { levelHash, tools } = level.maybePlacements;
  return { ...stars, placements: { levelHash, tools } };
}

/** Writes `{version: 1, levels}` with known ids in registry order, then unknown ids, then extra keys. */
export function serializeProgress(progress: ProgressDocument): string {
  const levels = emptyRaw();
  for (const id of PUZZLE_IDS) {
    const maybeLevel = progress.levels[id];
    if (maybeLevel !== undefined) {
      levels[id] = serializeLevel(maybeLevel);
    }
  }
  for (const [id, record] of Object.entries(progress.unknownLevels)) {
    levels[id] = record;
  }

  const output = emptyRaw();
  output["version"] = PROGRESS_VERSION;
  output["levels"] = levels;
  for (const [key, entry] of Object.entries(progress.extra)) {
    output[key] = entry;
  }

  return JSON.stringify(output);
}

/** The only place stars combine; delegates to `mergeLevelStars`, so nothing is ever lowered (D-09). */
function mergeStarFields(maybeA: LevelStars | undefined, maybeB: LevelStars | undefined): LevelStars {
  const maybeWonA = maybeA?.win === true ? maybeA : undefined;
  const maybeWonB = maybeB?.win === true ? maybeB : undefined;
  if (maybeWonA !== undefined && maybeWonB !== undefined) {
    return copyStars(mergeLevelStars(maybeWonA, maybeWonB, maybeWonB.bestPeakHeldPct) ?? maybeWonA);
  }

  const maybeWon = maybeWonA ?? maybeWonB;
  return copyStars(maybeWon ?? NO_LEVEL_STARS);
}

function withLevel(progress: ProgressDocument, id: PuzzleId, level: LevelProgress): ProgressDocument {
  const levels = emptyLevels();
  Object.assign(levels, progress.levels);
  levels[id] = level;
  return { levels, unknownLevels: progress.unknownLevels, extra: progress.extra };
}

/** Merges a run's stars into a level without lowering any (D-09); a loss leaves the document as is. */
export function withStars(progress: ProgressDocument, id: PuzzleId, stars: LevelStars): ProgressDocument {
  if (!stars.win) {
    return progress;
  }

  const maybeLevel = progress.levels[id];
  const merged = mergeStarFields(maybeLevel, stars);
  return withLevel(progress, id, { ...merged, maybePlacements: maybeLevel?.maybePlacements });
}

/** Replaces a level's saved placements and keeps its stars; an absent level starts with zero stars. */
export function withPlacements(
  progress: ProgressDocument,
  id: PuzzleId,
  maybeSaved: SavedPlacements | undefined,
): ProgressDocument {
  const stars = copyStars(progress.levels[id] ?? NO_LEVEL_STARS);
  return withLevel(progress, id, { ...stars, maybePlacements: maybeSaved });
}

/**
 * Combines the freshly read stored document with this tab's progress: stars
 * never go down, stored placements win, and unknown data comes from stored.
 */
export function mergeProgress(stored: ProgressDocument, local: ProgressDocument): ProgressDocument {
  const levels = emptyLevels();
  for (const id of PUZZLE_IDS) {
    const maybeStored = stored.levels[id];
    const maybeLocal = local.levels[id];
    if (maybeStored === undefined && maybeLocal === undefined) {
      continue;
    }

    levels[id] = {
      ...mergeStarFields(maybeStored, maybeLocal),
      maybePlacements: maybeStored?.maybePlacements ?? maybeLocal?.maybePlacements,
    };
  }

  return { levels, unknownLevels: stored.unknownLevels, extra: stored.extra };
}

/** Persisted best for a level, or undefined before its first win (matches StarStore.get). */
export function maybeLevelStars(progress: ProgressDocument, id: PuzzleId): LevelStars | undefined {
  const maybeLevel = progress.levels[id];
  if (maybeLevel === undefined || !maybeLevel.win) {
    return undefined;
  }

  return copyStars(maybeLevel);
}

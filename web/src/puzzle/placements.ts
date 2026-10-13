import type { PuzzleId } from "../catalog/puzzles";

/** Tool kind codes for the `kinds` lane; mirrors Rust `tools.rs` and `TOOL_CODES` in scripts/puzzle-replay/fixture.ts. */
export const PUZZLE_TOOL_CODES = { plank: 1, valve: 2 } as const;

/** One placeable tool kind. */
export type PuzzleToolKind = keyof typeof PUZZLE_TOOL_CODES;

/** One placement in grid units: x/y in 0.05 m cells, angle in τ/24 steps. */
export type PuzzlePlacement = {
  readonly kind: PuzzleToolKind;
  readonly x: number;
  readonly y: number;
  readonly angle: number;
  /** Half-second open tick; 0 except for valves (Phase 40 D-02). */
  readonly openTick: number;
};

/** Open tick of a newly placed valve: 2.0 s (Phase 40 D-05). */
export const DEFAULT_VALVE_OPEN_TICK = 4;

/** Open tick a newly placed tool starts with: the valve default, otherwise 0. */
export function defaultOpenTick(kind: PuzzleToolKind): number {
  return kind === "valve" ? DEFAULT_VALVE_OPEN_TICK : 0;
}

/**
 * Parallel wasm input lanes: one kind code and four pose integers
 * `(x, y, angle, openTick)` per placement; mirrors Rust `POSE_STRIDE = 4`.
 */
export type PlacementLanes = { readonly kinds: Uint8Array; readonly poses: Int32Array };

/** No placements, the Phase 38 default for natural runs. */
export const EMPTY_PLACEMENTS: readonly PuzzlePlacement[] = [];

const POSE_STRIDE = 4;
const I32_MIN = -2_147_483_648;
const I32_MAX = 2_147_483_647;
const INVALID_PLACEMENT_MESSAGE = "Invalid puzzle placement";
const INVALID_SEAM_MESSAGE = "Invalid test placements";

function isToolKind(kind: unknown): kind is PuzzleToolKind {
  return typeof kind === "string" && Object.hasOwn(PUZZLE_TOOL_CODES, kind);
}

function isPoseInteger(value: number): boolean {
  return Number.isSafeInteger(value) && value >= I32_MIN && value <= I32_MAX;
}

/** Fails closed so typed arrays never coerce NaN, fractions or out-of-range values (Phase 37 WR-01). */
function checkPlacement(placement: PuzzlePlacement): void {
  const valid =
    isToolKind(placement.kind) &&
    isPoseInteger(placement.x) &&
    isPoseInteger(placement.y) &&
    isPoseInteger(placement.angle) &&
    isPoseInteger(placement.openTick);
  if (!valid) {
    throw new Error(INVALID_PLACEMENT_MESSAGE);
  }
}

/** Copies placements into wasm lanes after integer, i32-range and kind checks (Phase 37 WR-01). */
export function encodePlacementLanes(placements: readonly PuzzlePlacement[]): PlacementLanes {
  const kinds = new Uint8Array(placements.length);
  const poses = new Int32Array(placements.length * POSE_STRIDE);
  placements.forEach((placement, index) => {
    checkPlacement(placement);
    kinds[index] = PUZZLE_TOOL_CODES[placement.kind];
    poses.set(
      [placement.x, placement.y, placement.angle, placement.openTick],
      index * POSE_STRIDE,
    );
  });

  return { kinds, poses };
}

/** Window global set only by Playwright `addInitScript`; read once at puzzle start (D-22). */
export const PUZZLE_TEST_PLACEMENTS_GLOBAL = "__liquidfunPuzzleTestPlacements";

function invalidSeam(): never {
  throw new Error(INVALID_SEAM_MESSAGE);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readSeamNumber(entry: Record<string, unknown>, key: string): number {
  const value = entry[key];
  return typeof value === "number" ? value : invalidSeam();
}

/** An absent `openTick` reads 0 so Phase 39 plank seams stay valid. */
function readSeamOpenTick(entry: Record<string, unknown>): number {
  return entry["openTick"] === undefined ? 0 : readSeamNumber(entry, "openTick");
}

/** Builds a fresh placement from untrusted seam data; never returns the input object. */
function parseSeamPlacement(entry: unknown): PuzzlePlacement {
  if (!isRecord(entry) || typeof entry["kind"] !== "string") {
    return invalidSeam();
  }

  const kind = entry["kind"];
  const x = readSeamNumber(entry, "x");
  const y = readSeamNumber(entry, "y");
  const angle = readSeamNumber(entry, "angle");
  const openTick = readSeamOpenTick(entry);
  if (!isToolKind(kind)) {
    throw new Error(INVALID_PLACEMENT_MESSAGE);
  }

  const placement: PuzzlePlacement = { kind, x, y, angle, openTick };
  checkPlacement(placement);
  return placement;
}

/** Test-only placements from `source[PUZZLE_TEST_PLACEMENTS_GLOBAL]`, undefined when absent; throws when present but invalid. */
export function maybeReadTestPlacements(
  source: object,
  levelId: PuzzleId,
): readonly PuzzlePlacement[] | undefined {
  const maybeSeam: unknown = Reflect.get(source, PUZZLE_TEST_PLACEMENTS_GLOBAL);
  if (maybeSeam === undefined) {
    return undefined;
  }
  if (!isRecord(maybeSeam) || typeof maybeSeam["level"] !== "string") {
    return invalidSeam();
  }
  if (maybeSeam["level"] !== levelId) {
    throw new Error("Test placements name another level");
  }

  const placements = maybeSeam["placements"];
  if (!Array.isArray(placements)) {
    return invalidSeam();
  }

  return placements.map(parseSeamPlacement);
}

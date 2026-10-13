import type { PuzzleId } from "../catalog/puzzles";
import {
  isGeometricStatus,
  isPlacementStatusCode,
  type PlacementInspection,
  type PlacementStatusCode,
  type Quad,
} from "../puzzle/placement-status";
import { encodePlacementLanes, type PuzzlePlacement } from "../puzzle/placements";
import type { Point } from "../render/camera";

/** Floats per outline in the inspector lane: four `(x, y)` world-space corners. */
const OUTLINE_STRIDE = 8;
const INVALID_INSPECTION_MESSAGE = "Invalid placement inspection";

/** Generated `PlacementInspection` surface. */
export type RawPlacementInspection = {
  statuses(): Uint8Array;
  outlines(): Float32Array;
  free(): void;
};

/** Generated `inspectPuzzlePlacements` signature. */
export type RawInspectFn = (
  levelId: string,
  kinds: Uint8Array,
  poses: Int32Array,
) => RawPlacementInspection;

function invalidInspection(): never {
  throw new Error(INVALID_INSPECTION_MESSAGE);
}

function decodeStatus(value: number | undefined): PlacementStatusCode {
  if (value === undefined || !isPlacementStatusCode(value)) {
    return invalidInspection();
  }

  return value;
}

function decodeCorner(outlines: Float32Array, offset: number): Point {
  const x = outlines[offset];
  const y = outlines[offset + 1];
  if (x === undefined || y === undefined || !Number.isFinite(x) || !Number.isFinite(y)) {
    return invalidInspection();
  }

  return { x, y };
}

function decodeOutline(outlines: Float32Array, index: number): Quad {
  const start = index * OUTLINE_STRIDE;
  return [
    decodeCorner(outlines, start),
    decodeCorner(outlines, start + 2),
    decodeCorner(outlines, start + 4),
    decodeCorner(outlines, start + 6),
  ];
}

/** Decodes inspector lanes (D-02), failing closed with "Invalid placement inspection". */
export function decodeInspection(
  count: number,
  statuses: Uint8Array,
  outlines: Float32Array,
): PlacementInspection {
  if (!Number.isSafeInteger(count) || count < 0) {
    return invalidInspection();
  }
  if (statuses.length !== count || outlines.length !== count * OUTLINE_STRIDE) {
    return invalidInspection();
  }

  const codes = Array.from(statuses, decodeStatus);
  return {
    statuses: codes,
    outlines: codes.map((code, index) =>
      isGeometricStatus(code) ? decodeOutline(outlines, index) : undefined,
    ),
  };
}

/** Runs the stateless Rust inspector for `placements` in the given (creation) order and frees its result. */
export function inspectPlacements(
  inspect: RawInspectFn,
  levelId: PuzzleId,
  placements: readonly PuzzlePlacement[],
): PlacementInspection {
  const lanes = encodePlacementLanes(placements);
  const raw = inspect(levelId, lanes.kinds, lanes.poses);
  try {
    return decodeInspection(placements.length, raw.statuses(), raw.outlines());
  } finally {
    raw.free();
  }
}

import type { Point } from "../render/camera";

/** Rust `validate.rs` PlacementStatus wire codes (D-01). Never renumber. */
export const PLACEMENT_STATUS = {
  valid: 0,
  unknownKind: 1,
  kindNotAllowed: 2,
  overBudget: 3,
  angleOutOfRange: 4,
  positionOutOfRange: 5,
  outsideBuildArea: 6,
  overlapsWall: 7,
  overlapsCupWall: 8,
  insideCup: 9,
  overlapsWater: 10,
  overlapsTool: 11,
  overlapsNoPlace: 12,
} as const;

/** One inspector status code. */
export type PlacementStatusCode = (typeof PLACEMENT_STATUS)[keyof typeof PLACEMENT_STATUS];

/** Four world-space corners of one placed tool's outline. */
export type Quad = readonly [Point, Point, Point, Point];

/** Decoded inspector answer for one editor revision; `outlines[i]` is undefined for non-geometric statuses. */
export type PlacementInspection = {
  readonly statuses: readonly PlacementStatusCode[];
  readonly outlines: readonly (Quad | undefined)[];
};

const MAX_STATUS_CODE = PLACEMENT_STATUS.overlapsNoPlace;

/** Player-facing short reasons for every invalid status (D-20, UI-SPEC). */
const REASONS: Readonly<Record<Exclude<PlacementStatusCode, 0>, string>> = {
  1: "Not allowed in this level",
  2: "Not allowed in this level",
  3: "Over the tool limit",
  4: "Can't be placed there",
  5: "Can't be placed there",
  6: "Outside the build area",
  7: "Overlaps a wall",
  8: "Overlaps the cup",
  9: "Inside the cup",
  10: "Overlaps the water",
  11: "Overlaps another tool",
  12: "In a no-build zone",
};

/** True for an integer status the Rust inspector can return. */
export function isPlacementStatusCode(value: number): value is PlacementStatusCode {
  return Number.isInteger(value) && value >= 0 && value <= MAX_STATUS_CODE;
}

/** False for statuses whose outline lane is zeros (unknown kind, angle or position out of range). */
export function isGeometricStatus(code: PlacementStatusCode): boolean {
  return (
    code !== PLACEMENT_STATUS.unknownKind &&
    code !== PLACEMENT_STATUS.angleOutOfRange &&
    code !== PLACEMENT_STATUS.positionOutOfRange
  );
}

/** Short reason phrase for an invalid status; the valid status has none. */
export function placementReason(code: PlacementStatusCode): string {
  if (code === PLACEMENT_STATUS.valid) {
    throw new Error("A valid placement has no reason");
  }

  return REASONS[code];
}

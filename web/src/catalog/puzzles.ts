/**
 * Allowlisted puzzle level ids. Mirrors `LEVEL_IDS` in
 * crates/liquidfun-wasm/src/puzzle.rs and never joins `SCENE_IDS`.
 */
export const PUZZLE_IDS = ["first-pour"] as const;

/** One allowlisted puzzle level id. */
export type PuzzleId = (typeof PUZZLE_IDS)[number];

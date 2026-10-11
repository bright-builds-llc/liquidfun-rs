---
phase: 37-puzzle-core-in-the-bridge
plan: "03"
subsystem: puzzle-core
tags: [rust, puzzle, goal, outcome-latch, level-registry, GOAL-02]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-11T23:44:58.060Z"
requires:
  - liquidfun::math::Vec2 (new, is_valid; unchanged)
  - crate::scene::parse_scene_id and SessionError::UnknownScene (unchanged)
provides:
  - crate::puzzle::goal::{GoalRect, count_in_goal, fill_ratio, fill_margin_count, Outcome, GoalTracker, PuzzleStatus}
  - crate::puzzle::{LevelId, LEVEL_IDS, parse_level_id}
  - web/src/catalog/puzzles.ts PUZZLE_IDS / PuzzleId (not imported yet)
affects:
  - 37-05/06 (level builder and PuzzleHooks call count_in_goal and GoalTracker::observe)
  - 37-07 (newPuzzle maps parse_level_id None to SessionError::UnknownLevel, removes the temporary dead_code allow)
  - Phase 38 (routing reads PUZZLE_IDS)
tech-stack:
  added: []
  patterns:
    - pure functional core generic over the group key so tests use u8 instead of opaque ParticleGroupId
    - include_str! mirror test keeps a TS id list in sync with a Rust allowlist
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/goal.rs
    - crates/liquidfun-wasm/src/puzzle/goal/tests.rs
    - crates/liquidfun-wasm/src/puzzle/tests.rs
    - web/src/catalog/puzzles.ts
  modified:
    - crates/liquidfun-wasm/src/lib.rs
key-decisions:
  - "GoalTracker latches on the first win or loss; the win is checked before the loss, so a hold that completes exactly at step_limit wins, and a first observation past the limit loses at step_limit"
  - "count_in_goal is generic over the group key (u8 in tests, ParticleGroupId in production), uses strict containment (edges and NaN are outside) and a saturating u32 count, and allocates nothing"
  - "Levels have their own LevelId allowlist and PUZZLE_IDS mirror, and tests show they are disjoint from the 25 SCENE_IDS in both directions"
requirements: [GOAL-02]
duration: 14min
completed: 2026-10-11
---

# Phase 37 Plan 03: Pure goal counter, outcome latch and level registry Summary

**A side-effect-free `puzzle::goal` core (strict cup counting, clamped fill, +5% margin, step-latched `GoalTracker`) plus a separate `LevelId` allowlist with its `PUZZLE_IDS` TS mirror. 32 unit tests cover it, all in Arrange/Act/Assert form.**

## Performance

- **Duration:** about 14 min (23:31:04Z to 23:44:58Z), most of it in two full `cargo test -p liquidfun-wasm` runs of about 3 min each
- **Tasks:** 3 of 3
- **Files:** 6 (5 created, 1 modified)

## Accomplishments

- `GoalRect` with `is_valid` (finite corners, min strictly below max) and `contains_strictly`. A centre on any edge, or a NaN centre, is outside.
- `count_in_goal<G: PartialEq>` zips positions with group ids, counts only groups in the counted set and only centres strictly inside, and saturates.
- `fill_ratio` returns `min(count / full, 1.0)` and returns `0.0` when full is 0. `fill_margin_count` is `full.div_ceil(20)`. Neither uses an `as` cast or `unwrap`.
- `Outcome` maps to codes 0/1/2 with `maybe_step`, matching D-17 and the TS parser from Plan 02. `GoalTracker::new` rejects a zero target, a zero settle window and `settle > limit`. `observe` latches on the first win or loss.
- `PuzzleStatus` is a plain-data struct that Plans 06 and 07 will fill.
- `LevelId::FirstPour`, `LEVEL_IDS`, `as_str` and `parse_level_id` (exact match, `Option`).
- `web/src/catalog/puzzles.ts` exports `PUZZLE_IDS = ["first-pour"]` and is not imported anywhere yet. `scenes.ts`, `scene-records.ts`, `portrait-bounds.ts` and the README plans are untouched.

## Task Commits

1. **Task 1: Pure goal counter, fill and step-latched GoalTracker.** TDD RED: the build failed on unresolved imports. GREEN: 26 tests passed.
1. **Task 2: Level allowlist and PUZZLE_IDS mirror.** TDD RED: the build failed on the missing `puzzles.ts` and unresolved imports. GREEN: 6 tests passed, and `bun run typecheck` was clean.
1. **Task 3: Local checks and commit.** `1b6887a67` (`feat(37-03): add pure goal counter, outcome latch and level registry`). It contains exactly the six plan files.

The plan asks for one commit, so all three tasks share `1b6887a67`.

## Test Counts

- `cargo test -p liquidfun-wasm --lib puzzle::goal`: 26 passed (the 24 from the plan plus 2 added, see Deviations).
- `cargo test -p liquidfun-wasm --lib puzzle::tests`: 6 passed.
- `cargo test -p liquidfun-wasm --all-features`: 350 passed, 1 ignored (the Plan 01 spike).

## Check Results (`target/phase37/checks-03.results`, second run)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | exit=0 |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| `cargo test -p liquidfun-wasm --all-features` | exit=0 |
| `bun scripts/bright-builds-check.ts all` | exit=0 |
| `cd web && bun run typecheck` | exit=0 |
| `git diff --stat 958d6fad7 -- crates/liquidfun` is empty | exit=0 |

The first run had `fmt exit=1`, caused only by rustfmt line wrapping in `goal.rs` and `goal/tests.rs`. I ran `cargo fmt --all` and reran the whole script, and every check passed. The first run's results were overwritten by the rerun, so only the passing results are in the file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Two extra tests to cover items unused under `cfg(test)`**
- **Found during:** Task 1
- **Issue:** The plan's `#![cfg_attr(not(test), allow(dead_code))]` covers only non-test builds. In the lib test build, `GoalRect::min`/`max` were never called and `PuzzleStatus` was never constructed, so the build gave 2 dead-code warnings, which clippy `-D warnings` rejects.
- **Fix:** I added `goal_rect_exposes_its_corners` and `puzzle_status_carries_the_latched_outcome` (both AAA) instead of widening the allow attribute.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/goal/tests.rs
- **Commit:** 1b6887a67

### Other notes

- `generated_by` is `gsd-execute-phase`, as the orchestrator asked. The plan's output section says `gsd-executor`.
- I did not mark GOAL-02 complete in REQUIREMENTS.md. Phase verification owns that, and the tracker is not wired into a session until Plans 06 and 07.

## Known Stubs

- `puzzle.rs` line 2 has a temporary `#![cfg_attr(not(test), allow(dead_code))]`, as the plan intends. Plan 37-07 removes it when `newPuzzle` wires the module.
- `PUZZLE_IDS` is not imported anywhere yet. Phase 38 owns routing.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle.rs, crates/liquidfun-wasm/src/puzzle/goal.rs, crates/liquidfun-wasm/src/puzzle/goal/tests.rs (400 lines, 26 `// Arrange`), crates/liquidfun-wasm/src/puzzle/tests.rs, web/src/catalog/puzzles.ts
- FOUND: commit 1b6887a67
- FOUND: target/phase37/checks-03.results (7 lines, all exit=0)

---
phase: 37-puzzle-core-in-the-bridge
plan: "07"
subsystem: wasm-bridge
tags: [rust, wasm-bindgen, puzzle, frame, GOAL-02]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-12T00:42:16.240Z"
requires:
  - SessionSource::Puzzle and SessionCore::create_puzzle (Plan 06)
  - Placements::parse, PlacementError, GRID_METERS, REFERENCE_PLANK_POSE (Plan 05)
  - PuzzleStatus, Outcome::code / maybe_step (Plan 03)
  - TS parsing of the optional frame getters (Plan 02)
provides:
  - ProofSession.newPuzzle(levelId, kinds, poses) and ProofSession.levelInfo()
  - "#[wasm_bindgen] PuzzleLevelInfo (re-exported from lib.rs)"
  - ProofFrame goalCount / goalTarget / goalFull / outcome / outcomeStep / stepLimit (undefined on scene frames)
  - FrameData::set_puzzle_status (fail-closed, including observed_step == step_index)
  - SessionError::{UnknownLevel, InvalidPlacement(PlacementError), NotAPuzzle}
  - crate-root build_puzzle_core helper for native boundary tests
affects:
  - 37-08 (replay harness reads the frame getters and levelInfo through build_puzzle_core / newPuzzle)
  - 38 (player shell consumes newPuzzle, levelInfo and the frame scalars)
tech-stack:
  added: []
  patterns:
    - second #[wasm_bindgen] impl block in a child module (frame/puzzle.rs) reading the parent's private field
    - private module + pub use chain to keep private_interfaces / missing_docs clean
key-files:
  created:
    - crates/liquidfun-wasm/src/frame/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/info.rs
    - crates/liquidfun-wasm/src/puzzle/api_tests.rs
  modified:
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/goal.rs
    - crates/liquidfun-wasm/src/puzzle/tools.rs
key-decisions:
  - "Every captured puzzle frame checks hooks observed_step == frame step index (D-14); a drift fails capture with FrameCaptureFailed"
  - "parse_level_id now searches LEVEL_IDS by as_str, so the allowlist array is the single source of level ids"
  - "Test-only puzzle helpers are #[cfg(test)] (fill_ratio, fill_margin_count, ToolKind::token/from_token, Placement::kind/grid/angle, Placements::empty); Plan 08's replay module is cfg(test), so it still reaches them"
  - "capture_frame label and puzzle-status attachment moved into SessionCore::annotate_frame to stay under clippy's 100-line limit"
requirements: [GOAL-02]
duration: 11min
completed: 2026-10-12
---

# Phase 37 Plan 07: newPuzzle, levelInfo and frame puzzle scalars Summary

**The puzzle core now reaches JavaScript. `ProofSession.newPuzzle(levelId, kinds, poses)` builds a level from copied typed-array lanes, and `levelInfo()` returns a `PuzzleLevelInfo`. Every puzzle frame carries six validated goal scalars that are `undefined` on scene frames. No temporary dead-code allow remains.**

## Performance

- **Duration:** about 11 min (2026-10-12T00:31Z to 00:42Z), including one full check run of about 4 min
- **Tasks:** 3 of 3
- **Files:** 10 (3 created, 7 modified)

## Commit

| Task | Commit | Description |
| --- | --- | --- |
| 1-3 | `bb34c4c04` | feat(37-07): expose newPuzzle, levelInfo and frame puzzle scalars |

The plan specifies one commit in Task 3. Tasks 1 and 2 have no commit step, so all code changes landed together after the full check run.

## Generated d.ts (verified after `bun scripts/web-build.ts wasm`)

```ts
// ProofSession
static newPuzzle(level_id: string, kinds: Uint8Array, poses: Int32Array): ProofSession;
levelInfo(): PuzzleLevelInfo;
// ProofFrame
goalCount(): number | undefined;
goalTarget(): number | undefined;
goalFull(): number | undefined;
outcome(): number | undefined;
outcomeStep(): number | undefined;
stepLimit(): number | undefined;
// export class PuzzleLevelInfo
levelId(), title(), targetCount(), fullCount(), settleSteps(), stepLimit(): number,
cupInterior(): Float32Array, buildArea(): Int32Array, gridMeters(), toolKinds(): Uint8Array,
toolCounts(): Uint32Array;
```

## What Was Built

- **Frame scalars (D-17):** `FrameData` has `maybe_puzzle: Option<PuzzleStatus>`, set to `None` in `new`, so the 7-argument constructor and all 9 `FrameDiagnostics::at_rest()` call sites are unchanged. `set_puzzle_status` rejects each of these with `InvalidDiagnostic`:
  - `target_count == 0`
  - `target_count > full_count`
  - `step_limit == 0`
  - `goal_count > particles`
  - an outcome step greater than `step_limit` or greater than `observed_step`
  - `observed_step != step_index`

  A particle count that does not fit in `u32` returns `ArithmeticOverflow`. `capture_frame` attaches the status only when the hooks report one.
- **Constructor and info (D-16, D-24):** `build_puzzle_core` runs `parse_level_id`, then `UnknownLevel`, then `Placements::parse`, then `InvalidPlacement(..)`, then `create_puzzle`. `SessionCore::level_info` returns `NotAPuzzle` for scene sources. `PuzzleLevelInfo` wraps `&'static LevelSpec`.
- **Cleanup:** removed the four `Temporary until Plan 37-07` markers: the inner allow in puzzle.rs, the allow in scene.rs, and the two-line marker on `SessionSource::Puzzle` plus the one on `create_puzzle` in session.rs.

## Dead-code Items After Wiring

| Item | Resolution |
| --- | --- |
| `LEVEL_IDS` | Wired: `parse_level_id` searches it by `as_str` |
| `goal::fill_ratio` | `#[cfg(test)]` (only goal tests read it; the player computes fill from frame scalars) |
| `goal::fill_margin_count` | `#[cfg(test)]` (used by `maybe_margin_win_step` and Plan 08's `check_contract`, both test-only) |
| `ToolKind::token`, `ToolKind::from_token` | `#[cfg(test)]` (fixture tokens for the cfg(test) replay module) |
| `Placement::kind`, `grid`, `angle` | `#[cfg(test)]` (only tool tests read them) |
| `Placements::empty` | `#[cfg(test)]` (production parses empty lanes through `parse`) |

No `allow(dead_code)` was added. The only remaining one in `frame.rs` is the pre-existing allow on `FrameDiagnostics::at_rest`.

## Tests

- `frame::puzzle::tests` (6): scene frames give None; running, won and lost scalars; inconsistent statuses are rejected; an out-of-sync step counter is rejected.
- `puzzle::api_tests` (8), including `first_pour_frames_and_level_info_report_the_goal_contract`. It builds first-pour through `build_puzzle_core`, checks every `levelInfo` field against `level_spec(FirstPour)`, then checks frames 0 to 10 for running scalars and `step_index`. The other tests cover:
  - The reference plank adds 4 rigid segments.
  - A scene frame has no puzzle scalars.
  - Scene `level_info` returns `NotAPuzzle`.
  - `dam-break` and `First-Pour` give `UnknownLevel`, and `first-pour` as a scene gives `UnknownScene`.
  - Budget, angle and lane errors map to `InvalidPlacement`.
  - Error messages are bounded.
  - Function-pointer checks confirm `new_puzzle` and `level_info` exist.

## Check Results (`target/phase37/checks-07.results`)

| Check | Result |
| --- | --- |
| fmt --check | exit=0 |
| clippy -p liquidfun-wasm --all-targets --all-features -D warnings | exit=0 |
| build --all-targets --all-features | exit=0 |
| cargo test -p liquidfun-wasm --all-features | exit=0 (411 passed, 3 ignored) |
| bright-builds-check all | exit=0 (file-lengths findings=0) |
| web-build wasm | exit=0 |
| web typecheck + Vitest (frame, frame-puzzle, session, worker-messages) | exit=0 (64 tests) |
| crates/liquidfun unchanged vs phase base | exit=0 |

The 25/25 fingerprint gate was not re-run, as the plan intends: the scene arm is untouched and scene hooks return `None`. Plan 08 runs the final gate.

File lengths: lib.rs 311, session.rs 545, frame.rs 542, puzzle/info.rs 107, puzzle/api_tests.rs 219 (all at or below 628).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `capture_frame` exceeded clippy's too_many_lines (104 of 100)**
- **Found during:** Task 2 clippy
- **Fix:** Moved the circle-label and puzzle-status attachment into `SessionCore::annotate_frame`. Behavior is unchanged.
- **Files modified:** crates/liquidfun-wasm/src/session.rs
- **Commit:** bb34c4c04

**2. [Rule 3 - Blocking] clippy type_complexity on the six-scalar tuple in api_tests**
- **Fix:** Added a `PuzzleScalars` type alias.
- **Commit:** bb34c4c04

**3. Additional dead-code items beyond the plan's expected list.** `fill_margin_count`, `ToolKind::token` and `Placement::kind/grid/angle` were also unused in production. Each was handled the same way, with `#[cfg(test)]`.

## Known Stubs

None.

## Threat Flags

None. The new surface (`newPuzzle`, `levelInfo`, frame getters) is the one covered by T-37-26 to T-37-30, and each mitigation is implemented and tested.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/frame/puzzle.rs, crates/liquidfun-wasm/src/puzzle/info.rs, crates/liquidfun-wasm/src/puzzle/api_tests.rs
- FOUND: commit bb34c4c04
- `git grep "Temporary until Plan 37-" -- crates/liquidfun-wasm/src` prints nothing

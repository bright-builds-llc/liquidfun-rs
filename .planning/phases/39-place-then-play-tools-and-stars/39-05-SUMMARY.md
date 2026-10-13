---
phase: 39-place-then-play-tools-and-stars
plan: "05"
subsystem: wasm-puzzle-core
tags: [rust, wasm, puzzle, peak-held, brim, frame, fail-closed, D-07, D-08, D-09, D-10, D-11, D-27]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:43:18Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: "39-01 LevelSpec.brim_count (810 for first-pour), validate_for_build; 39-04 TS D-11 parse rules"
provides:
  - crates/liquidfun-wasm/src/puzzle/peak.rs PeakHeldTracker (new/observe/peak/maybe_brim_step) and decided_step
  - PuzzleStatus peak_held_count, brim_count, maybe_brim_step, maybe_decided_step
  - ProofFrame getters peakHeldCount, brimCount, brimStep, decidedStep (undefined for scenes)
  - frame/puzzle.rs status_is_consistent (single Rust rule set mirrored by web/src/physics/frame-puzzle.ts)
affects:
  - 39-06 (rebuilt wasm package exposes the four getters the TS parser now requires)
  - 39-07 reducer and 39-08 replay (read peak held, Brim and decision step from these frames)
tech-stack:
  added: []
  patterns:
    - integer-only per-step tracker next to GoalTracker in PuzzleHooks::on_after_step
    - one consistency function (status_is_consistent) owns every puzzle status rule; set_puzzle_status only delegates
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/peak.rs
    - crates/liquidfun-wasm/src/puzzle/peak/tests.rs
    - crates/liquidfun-wasm/src/frame/puzzle/tests.rs
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/goal.rs
    - crates/liquidfun-wasm/src/puzzle/goal/tests.rs
    - crates/liquidfun-wasm/src/puzzle/hooks.rs
    - crates/liquidfun-wasm/src/puzzle/builder.rs
    - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/frame/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/session_tests.rs
key-decisions:
  - "Peak held is a separate PeakHeldTracker beside the unchanged GoalTracker; a 200-sequence test proves the first step with peak >= target equals the GoalTracker win step"
  - "decided_step: Lost -> its latch step, Won with Brim -> Brim step, Won without Brim -> step_limit once observed >= step_limit, else None"
  - "status_is_consistent adds: target < brim <= full, won iff peak >= target, Brim iff peak >= brim, Brim step only on won runs within win..min(observed, limit), loss exactly at the limit, decided equals decided_step(..); peak never compared with particle count"
  - "D-07 measured (release and debug agree): reference peak held 541 @201, 811 @262, 1331 @600, 1361 @720; Brim 810 latches at step 262 and decides the run there"
requirements-completed: []
duration: 22min
completed: 2026-10-13
---

# Phase 39 Plan 05: Peak Held Fill, Brim Latch and D-11 Frame Scalars Summary

**Rust now tracks the peak held count, the Brim latch and the decision step on every fixed step. Every puzzle frame carries `peakHeldCount`, `brimCount`, `brimStep` and `decidedStep`, checked fail-closed by a single `status_is_consistent` rule set. Permuted placement order gives identical fingerprints, and the reference solution clears Brim 810 by 521 particles at step 600.**

## Performance

- **Duration:** about 22 min
- **Started:** 2026-10-13T00:20:46Z
- **Completed:** 2026-10-13T00:42:37Z
- **Tasks:** 2 of 2
- **Files modified:** 12 (3 created, 9 modified)

## Accomplishments

- `puzzle/peak.rs`: `PeakHeldTracker` keeps a `VecDeque` window bounded at `settle_steps`. When the window is full, the peak becomes the larger of the current peak and the window minimum. Brim latches at the first step where the peak reaches `brim_count`. Steps after `step_limit` are ignored. The tracker uses integers only and never panics. `new` returns `None` unless `1 <= settle_steps <= step_limit` and `brim_count >= 1`.
- `decided_step(outcome, maybe_brim_step, observed_step, step_limit)` implements D-10.
- `PuzzleHooks::on_after_step` calls `self.peak.observe(..)` right after `self.tracker.observe(..)`. `maybe_puzzle_status` fills the four new `PuzzleStatus` fields. The builder creates the tracker inside `validated_trackers` and fails with `SceneError::Geometry` on an invalid spec.
- The `GoalTracker` logic is untouched: `git diff 424735672 -- puzzle/goal.rs | grep -c "fn observe"` returns 0.
- `frame/puzzle.rs` adds the four getters and `status_is_consistent`, which is split into `counts_are_consistent`, `steps_are_consistent` and `stars_are_consistent`. `frame.rs` `set_puzzle_status` now only delegates, so frame.rs shrank from 542 to 534 lines.
- The frame puzzle tests moved to `frame/puzzle/tests.rs` (373 lines) and now cover the following:
  - all four scalars are `None` on scene frames
  - a Brim frame reports `(811, 810, 262, 262)`
  - a won run at the limit without Brim decides at 600
  - a peak above the particle count is accepted
  - 14 labelled rejection cases, split between two tests
- `session_tests.rs` (367 lines) gains four tests:
  - `permuted_placements_give_identical_fingerprints`
  - `reference_peak_clears_brim_with_margin`
  - `reference_run_decides_at_the_brim_step`
  - `empty_run_has_no_peak_and_decides_at_the_limit`

  All run in the existing lib test binary.

## D-07 Measured Values (first-pour, brim_count 810, full_count 900)

From `LIQUIDFUN_PUZZLE_RECORD=1 cargo test -p liquidfun-wasm --release --lib puzzle:: -- --nocapture`. The debug build printed identical values.

| Quantity | Value |
|---|---|
| Peak held @201 (the recorded win step) | 541 (target 540) |
| Peak held @262 | 811 |
| Peak held @600 (`step_limit − 120`) | 1331, which is 521 above brim 810. The required margin is 45 (`900.div_ceil(20)`). |
| Peak held @720 (`step_limit`) | 1361 |
| Brim step | 262 |
| Decided step | 262 |
| Empty run | peak 0, no Brim, lost and decided at 720 |

These match the research measurements exactly. The reference solution is a 3-star solution, because it wins, reaches Brim, and uses 1 tool with par 1.

Fixtures replay unchanged: `first-pour-solution.txt` won at 201 with final count 689, and `first-pour-empty.txt` lost at 720 with final count 0, in the release profile. `git diff --quiet 424735672 -- crates/liquidfun-wasm/src/puzzle/fixtures` passes.

## Task Commits

1. **Task 1: PeakHeldTracker, Brim latch and decided step** - `989d89834` (feat)
1. **Task 2: Frame getters, consistency rules, permuted-order fingerprint and D-07 margin** - `26715f0fb` (feat)

TDD note: each task landed as one feat commit with tests and implementation together, rather than separate RED and GREEN commits. Plans 39-01 and 39-04 did the same.

## Verification

- `cargo fmt --all --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0, but this checks workspace default members only. `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` is the meaningful run, and it exits 0.
- `cargo test -p liquidfun-wasm` (debug): 475 passed, 0 failed, 3 ignored. The 3 ignored tests existed before this plan.
- `cargo test -p liquidfun-wasm --release --lib puzzle::`: 153 passed, 2 ignored. `puzzle::peak` alone runs 9 tests, including the 200-sequence GoalTracker equivalence test, which sees both winning and losing sequences.
- `git diff --stat 424735672 -- crates/liquidfun` is empty, so the engine crate is untouched.
- `bun scripts/bright-builds-check.ts all`: `findings=0`.
- Acceptance greps match: `pub(crate) struct PeakHeldTracker`, `peak.observe` in hooks.rs, `js_name = decidedStep` and `js_name = peakHeldCount` in frame/puzzle.rs, and `status_is_consistent` in frame.rs.

## Decisions Made

See `key-decisions` in the frontmatter. The Rust rule list matches the TS rules in 39-04 one for one.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Clippy `match_same_arms` in `decided_step`**
- **Found during:** Task 1
- **Issue:** `-D warnings` rejected separate `Running => None` and `Won, None => None` arms.
- **Fix:** Merged them into one final arm after the guarded step-limit arm. Behavior is unchanged.
- **Commit:** `989d89834`

**2. [Rule 3 - Blocking] Clippy `too_many_lines` on the star rejection test**
- **Found during:** Task 2
- **Issue:** The 14-case rejection test reached 111 lines, above the limit of 100.
- **Fix:** Split it into `set_puzzle_status_rejects_star_counts_inconsistent_with_the_peak` and `set_puzzle_status_rejects_star_steps_out_of_order`. All cases are still covered.
- **Commit:** `26715f0fb`

**3. [Plan-sanctioned] Frame puzzle tests moved to `frame/puzzle/tests.rs`**
- The plan allowed this move if the file would grow past 450 lines. The module was moved up front so that `frame/puzzle.rs` holds only getters and rules (159 lines).

**4. [Additions] Extra coverage beyond the bullets**
- Added `new_rejects_invalid_configurations` (peak tracker), a split decided-step test, `a_won_run_at_the_limit_without_brim_is_decided_at_the_limit` (frame), and a "loss before the limit" rejection case for the `Lost{step}` ⇒ `step == step_limit` rule.

## Issues Encountered

None.

## Known Stubs

None.

## Threat Flags

None. T-39-14 is mitigated: one Rust rule set, and any violation fails capture with `InvalidDiagnostic`. T-39-15 is mitigated: the window is bounded by `settle_steps`, the tracker uses integers only, and it has no panics. T-39-16 is mitigated: scene getters return `None` and the engine crate diff is empty.

## Next Phase Readiness

- Plan 06 must rebuild `web/src/generated/liquidfun-wasm` so the TS parser from 39-04 receives the four getters. Until then, real puzzle frames fail to parse in the web player. This plan did not rebuild the package, as instructed.
- D-12 (replay fixtures asserting `peak_held` and `brim_step`) is not part of this plan.

## Self-Check: PASSED

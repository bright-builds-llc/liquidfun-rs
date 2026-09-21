---
phase: 22-observability-shell
plan: "01"
subsystem: observability-tooling
tags: [xtask, playground, dam-break-bench, utc-stamp, file-length]

requires:
  - phase: 21-playground-leftover-cleanup
    provides: cargo xtask playground dam-break-bench stdout Markdown pair
provides:
  - playground.rs dispatcher plus playground/{error,counts,identity,pair,stamp}.rs
  - exclusive UTC stamp minting under target/dam-break-perf
affects:
  - 22-02 pair persistence
  - 22-03 profile command
  - 22-04 timer command

tech-stack:
  added: []
  patterns:
    - foo.rs plus foo/ playground split with no playground/mod.rs
    - exclusive create_dir stamp mint with one-second bump and 8-attempt fail-closed

key-files:
  created:
    - tools/xtask/src/playground/error.rs
    - tools/xtask/src/playground/counts.rs
    - tools/xtask/src/playground/identity.rs
    - tools/xtask/src/playground/pair.rs
    - tools/xtask/src/playground/stamp.rs
  modified:
    - tools/xtask/src/playground.rs

key-decisions:
  - "Keep dam-break-bench as the only playground subcommand; pair stays stdout-only until 22-02."
  - "Mint stamps with exclusive create_dir under target/dam-break-perf, bumping one Unix second on AlreadyExists, capped at 8 attempts."
  - "Honor LIQUIDFUN_XTASK_GIT as a Command program path for git rev-parse HEAD."
  - "Do not mark PERF-PAIR complete in this plan; persistence of pair.json/pair.md is 22-02."

patterns-established:
  - "Playground xtask is a dispatcher in playground.rs with focused playground/*.rs children and no playground/mod.rs."
  - "Evidence stamps are generated from Unix seconds, never from argv, and never overwrite an existing stamp directory."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-20T23:44:41Z

duration: 31min
completed: 2026-09-20
---

# Phase 22 Plan 01: Playground Split and Exclusive Stamp Summary

**Split the playground xtask into a dispatcher plus focused modules, and landed exclusive UTC `target/dam-break-perf/<YYYY-MM-DDTHH-MM-SSZ>/` minting that never clobbers an existing stamp.**

## Performance

- **Duration:** 31 min
- **Started:** 2026-09-20T23:13:12Z
- **Completed:** 2026-09-20T23:44:41Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Replaced the 540-line `playground.rs` with a dispatcher and `playground/{error,counts,identity,pair}.rs` (no `playground/mod.rs`); every file is well under 628 lines.
- Kept `cargo xtask playground dam-break-bench` as the only subcommand with the same `--warmup`/`--steps` defaults (60/600) and stdout Markdown pair output.
- Routed `git rev-parse HEAD` through `LIQUIDFUN_XTASK_GIT` when set so later fake-git tests can inject a program path.
- Added TDD-covered `format_utc_stamp` / `mint_exclusive_stamp`: exclusive `create_dir`, bump one second on `AlreadyExists`, fail closed after 8 occupied stamps, no `remove_dir_all`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Split playground.rs into foo.rs plus foo/ without behavior change** - `f757472` (refactor)
2. **Task 2 RED: TDD exclusive UTC stamp minting** - `ae90837` (test)
3. **Task 2 GREEN: TDD exclusive UTC stamp minting** - `b251594` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

- `tools/xtask/src/playground.rs` — dispatcher: USAGE, module declarations, `run()` match, `PlaygroundError` re-export
- `tools/xtask/src/playground/error.rs` — closed `PlaygroundError` with usage/upstream mapping
- `tools/xtask/src/playground/counts.rs` — locked 60/600 defaults and flag parsing
- `tools/xtask/src/playground/identity.rs` — host identity and repository-root walk; `LIQUIDFUN_XTASK_GIT`
- `tools/xtask/src/playground/pair.rs` — unprofiled pair orchestration (CMake extra target, native `--release` bench, Markdown stdout)
- `tools/xtask/src/playground/stamp.rs` — filename-safe UTC format plus exclusive mint

## Decisions Made

- Followed D-05: only `dam-break-bench` is live; no profile/timer commands in this plan.
- Followed D-01: stamps live under `target/dam-break-perf/`, never `target/v12-native-perf/`, never argv override.
- Left stamp APIs unused by `pair::run` so 22-02 owns persistence and the Rust/C++ ratio.
- Did not mark REQUIREMENTS `PERF-PAIR` complete: this plan prepares the split and stamp core; `pair.json`/`pair.md` remain 22-02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xtask has no lib target**
- **Found during:** Task 1 verification
- **Issue:** Plan command `cargo test -p xtask --lib playground` fails with `no library targets found in package xtask`.
- **Fix:** Ran `cargo test -p xtask playground -- --test-threads=1` against the binary unit tests (12 passed after Task 2).
- **Files modified:** none (verification command only)
- **Verification:** 12 playground unit tests pass
- **Committed in:** n/a

**2. [Rule 3 - Blocking] Clippy denied integer casts in the civil-date core**
- **Found during:** Task 2 GREEN
- **Issue:** `clippy::cast_possible_wrap`, `cast_possible_truncation`, `cast_sign_loss`, and `cast_lossless` failed `-D warnings` on Hinnant day-to-date conversions.
- **Fix:** Replaced `as` casts with `try_from` / `i64::from` and `let...else` fallbacks.
- **Files modified:** `tools/xtask/src/playground/stamp.rs`
- **Verification:** `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0
- **Committed in:** `b251594`

**3. [Rule 2 - Missing Critical] Allow unused stamp APIs until 22-02**
- **Found during:** Task 2 GREEN
- **Issue:** `format_utc_stamp` / `mint_exclusive_stamp` are intentionally unused by `pair::run` this plan, so the non-test xtask binary warned `dead_code`.
- **Fix:** Module-level `#![allow(dead_code)]` with a comment that 22-02 is the first production caller.
- **Files modified:** `tools/xtask/src/playground/stamp.rs`
- **Verification:** clippy `-D warnings` on xtask passes
- **Committed in:** `b251594`

***

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical)
**Impact on plan:** Verification command adapted to the binary crate; stamp minting still matches D-01. No scope creep.

## Issues Encountered

None beyond the verification-command mismatch and Clippy cast lints documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 22-02: persist `pair.json`/`pair.md` into a minted stamp, add Rust/C++ ratio, keep stdout Markdown.
- Do not wire profile/timer commands yet.
- Do not run a live Dam Break pair, cmake, or samply as a 22-01 gate.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-20*

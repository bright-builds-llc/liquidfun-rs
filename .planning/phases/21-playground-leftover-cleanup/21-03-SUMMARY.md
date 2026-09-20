---
phase: 21-playground-leftover-cleanup
plan: "03"
subsystem: testing
tags: [file-lengths, upstream_cli, xtask, foo.rs-plus-foo, Bright-Builds]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: named WASM scene modules plus existing foo.rs plus foo/tests.rs splits
  - phase: 21-playground-leftover-cleanup
    provides: leftover cleanup D-07 D-08 D-09 D-10 file-length gate
provides:
  - named scene files confirmed under 628 physical lines
  - upstream_cli.rs plus verify/configure/build/failures child modules
  - bun scripts/bright-builds-check.ts file-lengths and all exit 0
affects: [21-04 player-smoke and independent review]

tech-stack:
  added: []
  patterns:
    - xtask integration tests use foo.rs plus foo/ with #[path] like inventory_cli
    - Named WASM scene parents stay unedited when already under the 628-line gate

key-files:
  created:
    - tools/xtask/tests/upstream_cli/verify.rs
    - tools/xtask/tests/upstream_cli/configure.rs
    - tools/xtask/tests/upstream_cli/build.rs
    - tools/xtask/tests/upstream_cli/failures.rs
  modified:
    - tools/xtask/tests/upstream_cli.rs

key-decisions:
  - "Confirm color_mixer.rs, dam_break.rs, and water_wheel.rs at 346/360/426 and leave them unedited."
  - "Split upstream_cli with #[path] child modules like inventory_cli rather than a TSV exception."
  - "Keep build_accepts_the_registered_playground_dam_break_bench as a fake-cmake registration test; do not run C++ timing."

patterns-established:
  - "Integration-test crate roots declare child modules with #[path = \"foo/bar.rs\"] so Cargo does not treat tests/foo/ as another crate."
  - "Bright Builds file-lengths is satisfied by splitting oversized tests, not by excepting them."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-09-20T19-45-21
generated_at: 2026-09-20T20:26:30Z

duration: 4min
completed: 2026-09-20
---

# Phase 21 Plan 03: Scene File-Lengths and upstream_cli Split Summary

**Confirmed named WASM scene files at 346/360/426 lines and split `upstream_cli` into `foo.rs` plus `foo/` child modules so Bright Builds `file-lengths` and `all` exit 0 without a TSV exception or xtask CLI change.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-20T20:22:43Z
- **Completed:** 2026-09-20T20:26:30Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Confirmed `color_mixer.rs` (346), `dam_break.rs` (360), and `water_wheel.rs` (426) are under the 628-line gate and already declare `#[cfg(test)] mod tests;`. Did not edit those parents or their `*/tests.rs` children.
- Split `tools/xtask/tests/upstream_cli.rs` (was 642 lines) into a 225-line parent plus `verify.rs`, `configure.rs`, `build.rs`, and `failures.rs`.
- Kept every former `#[test]` name, `CARGO_BIN_EXE_xtask`, fake-tool env keys, argv vectors, and `build_accepts_the_registered_playground_dam_break_bench`.
- `cargo test -p xtask --test upstream_cli -- --test-threads=1` passed 19 tests. `bun scripts/bright-builds-check.ts file-lengths` and `all` both exit 0 with findings=0.

## Task Commits

Each task was committed atomically:

1. **Task 1: Confirm named scene file-lengths then split upstream_cli tests** - `7820067` (refactor)
2. **Task 2: Prove xtask tests and Bright Builds all exit 0** - no additional tree changes; verification-only

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `tools/xtask/tests/upstream_cli.rs` — fixtures, FakeTools, helpers, and `#[path]` child mods
- `tools/xtask/tests/upstream_cli/verify.rs` — `verify_*` identity/dirty/submodule tests
- `tools/xtask/tests/upstream_cli/configure.rs` — `configure_*` preset/digest/usage/process tests
- `tools/xtask/tests/upstream_cli/build.rs` — build target registration including playground-dam-break-bench
- `tools/xtask/tests/upstream_cli/failures.rs` — LF `.gitattributes` and Clang deprecated-copy coverage

## Decisions Made

- Left named scene modules unedited because each parent is already ≤ 628 (D-07/D-08).
- Used `#[path = "upstream_cli/*.rs"]` like `inventory_cli.rs` because an integration-test crate root's `mod verify;` would resolve to `tests/verify.rs`, not `tests/upstream_cli/verify.rs`. File layout remains `foo.rs` plus `foo/` with no `mod.rs` (D-09, rust.md).
- Did not create `.bright-builds-rules-checks.tsv` and did not edit `scripts/bright-builds-check.ts` (T-21-03-01, T-21-03-04).
- Kept the playground Dam Break bench as a fake-cmake registration test and did not run `just playground-dam-break-bench` (D-09/D-10, T-21-03-05).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared child modules with `#[path]` instead of implicit `mod verify;`**
- **Found during:** Task 1 (split upstream_cli tests)
- **Issue:** The plan preferred implicit `mod verify;` and assumed rustc would look in `tests/upstream_cli/verify.rs`. Integration-test files are crate roots, so implicit `mod verify;` would look for `tests/verify.rs`. The live `inventory_cli.rs` pattern already uses `#[path]`.
- **Fix:** Declared `#[path = "upstream_cli/{verify,configure,build,failures}.rs"]` plus `mod` lines. Layout remains `foo.rs` plus `foo/`; no `upstream_cli/mod.rs`.
- **Files modified:** `tools/xtask/tests/upstream_cli.rs`
- **Verification:** `rg -n "^mod verify;|^mod configure;|^mod build;|^mod failures;"` matches all four; 19 `upstream_cli` tests pass.
- **Committed in:** `7820067` (Task 1)

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for the split to compile. No CLI behavior, argv, env-var, or scene-recipe change.

## Issues Encountered

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Bright Builds `file-lengths` and `all` exit 0 on this tree. Named scene files stay under 628 without recipe changes.
- Plan 21-04 can run `just web-player-smoke` and independent AI review. Do not treat C++ Dam Break timing as a gate.

---
*Phase: 21-playground-leftover-cleanup*
*Completed: 2026-09-20*

## Self-Check: PASSED

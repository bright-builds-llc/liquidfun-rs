---
phase: 23-baseline-pair-and-named-audit
plan: "01"
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T03:34:17Z
subsystem: observability-tooling
tags: [xtask, playground-cli, file-length, fake-tools]

requires:
  - phase: 22-observability-shell
    provides: fake-tool playground CLI tests for pair, profile, and timers
provides:
  - playground_cli.rs dispatcher plus playground_cli/{support,pair,profile,timers,bundle,heap}.rs
  - empty bundle.rs and heap.rs stubs for exclusive later-plan ownership
affects:
  - 23-03 audit-bundle CLI tests
  - 23-06 heap CLI tests

tech-stack:
  added: []
  patterns:
    - integration-test crate root uses #[path] into foo/ children because tests/foo.rs is a crate root
    - foo.rs plus foo/ with no playground_cli/mod.rs and no playground_cli/main.rs

key-files:
  created:
    - tools/xtask/tests/playground_cli/support.rs
    - tools/xtask/tests/playground_cli/pair.rs
    - tools/xtask/tests/playground_cli/profile.rs
    - tools/xtask/tests/playground_cli/timers.rs
    - tools/xtask/tests/playground_cli/bundle.rs
    - tools/xtask/tests/playground_cli/heap.rs
  modified:
    - tools/xtask/tests/playground_cli.rs

key-decisions:
  - "Keep LIQUIDFUN_XTASK_* program-path injection in support.rs; never sh -c."
  - "Integration-test crate roots need #[path] to playground_cli/*.rs; Cargo does not auto-map tests/foo.rs to tests/foo/."
  - "Do not mark PERF-AUDIT or PERF-HEAP complete in this plan; this split only makes room for later exclusive tests."
  - "No .bright-builds-rules-checks.tsv exact-file exception; every split file is well under 628 lines."

patterns-established:
  - "playground_cli.rs is a dispatcher of path-mapped child modules; helpers live in support.rs as pub(super)."
  - "23-03 owns only bundle.rs and 23-06 owns only heap.rs until those plans add tests."

requirements-completed: []

duration: 8min
completed: 2026-09-21
---

# Phase 23 Plan 01: Playground CLI Test Split Summary

**Split the 622-line `playground_cli` fake-tool integration test into a crate-root dispatcher plus `playground_cli/` children so 23-03 and 23-06 can add bundle and heap cases without hitting the 628-line Bright Builds cap.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-21T03:26:20Z
- **Completed:** 2026-09-21T03:34:17Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Replaced the 622-line `tests/playground_cli.rs` with a dispatcher and `playground_cli/{support,pair,profile,timers,bundle,heap}.rs`.
- Kept existing fake-tool pair, profile, and timers coverage behavior-identical, including `LIQUIDFUN_XTASK_*` program-path injection and `LIQUIDFUN_XTASK_STAMP_UNIX=1000000000`.
- Left empty `bundle.rs` / `heap.rs` stubs for exclusive later-plan ownership; no `dam-break-audit-bundle`, `dam-break-heap`, or `dhat-heap` strings.
- Confirmed every split file is ≤628 physical lines with room for 23-03 and 23-06 cases.

## Task Commits

Each task was committed atomically:

1. **Task 1: Split playground_cli.rs into foo.rs plus foo/ without behavior change** - `eded773` (refactor)
2. **Task 2: Enforce the 628-line cap on every split test file** - no commit (verification-only; counts recorded below)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

- `tools/xtask/tests/playground_cli.rs` — crate-root dispatcher: module docs plus `#[path]` `mod` lines only
- `tools/xtask/tests/playground_cli/support.rs` — `RepositoryFixture`, fake-tool compile, stamp helpers, `LIQUIDFUN_XTASK_*` injection
- `tools/xtask/tests/playground_cli/pair.rs` — unprofiled pair persistence and just alias tests
- `tools/xtask/tests/playground_cli/profile.rs` — samply profile persistence, fail-closed missing samply, just alias
- `tools/xtask/tests/playground_cli/timers.rs` — timers.json without pair/profile artifacts
- `tools/xtask/tests/playground_cli/bundle.rs` — stub module docs for 23-03
- `tools/xtask/tests/playground_cli/heap.rs` — stub module docs for 23-06

## Physical line counts

Counted with Python `sum(1 for _ in path.open())` including blanks and comments:

| File | Lines | Cap |
| --- | ---: | ---: |
| `tools/xtask/tests/playground_cli.rs` | 14 | 628 |
| `tools/xtask/tests/playground_cli/support.rs` | 304 | 628 |
| `tools/xtask/tests/playground_cli/pair.rs` | 96 | 628 |
| `tools/xtask/tests/playground_cli/profile.rs` | 186 | 628 |
| `tools/xtask/tests/playground_cli/timers.rs` | 48 | 628 |
| `tools/xtask/tests/playground_cli/bundle.rs` | 1 | 628 |
| `tools/xtask/tests/playground_cli/heap.rs` | 1 | 628 |

No `.bright-builds-rules-checks.tsv` row was added. `playground_cli/mod.rs` does not exist.

## Decisions Made

- Followed D-04/D-12 test-room requirement: split before adding bundle or heap cases.
- Kept `LIQUIDFUN_XTASK_*` as `Command` program paths in `support.rs`; never `sh -c`.
- Added `#[path = "playground_cli/*.rs"]` on the crate-root `mod` lines so Cargo finds children under `foo/` without converting the crate to `main.rs` or adding `mod.rs`.
- Did not mark REQUIREMENTS `PERF-AUDIT` or `PERF-HEAP` complete: this plan only splits tests; named audit and heap remain later plans.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Integration-test crate roots do not auto-map `mod foo` to `foo/`**
- **Found during:** Task 1 verification
- **Issue:** `mod bundle;` in `tests/playground_cli.rs` looks for `tests/bundle.rs`, not `tests/playground_cli/bundle.rs`, because the file is a Cargo integration-test crate root.
- **Fix:** Added `#[path = "playground_cli/<child>.rs"]` on each `mod` line, matching `upstream_cli.rs` / `phase11_evidence_cli.rs`. The plan's `^mod bundle;` acceptance regex still matches.
- **Files modified:** `tools/xtask/tests/playground_cli.rs`
- **Verification:** `cargo test -p xtask --test playground_cli -- --test-threads=1` exits 0 (8 passed)
- **Committed in:** `eded773`

**2. [Rule 3 - Blocking] Child test modules needed `use std::fs`**
- **Found during:** Task 1 implementation
- **Issue:** After the move, `pair.rs` / `profile.rs` / `timers.rs` call `fs::read` without the crate-root imports.
- **Fix:** Added `use std::fs;` in those three files.
- **Files modified:** `tools/xtask/tests/playground_cli/pair.rs`, `profile.rs`, `timers.rs`
- **Verification:** same 8-test run
- **Committed in:** `eded773`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Required for the split to compile under Cargo's integration-test crate-root rules. No new playground subcommands and no file-length TSV exception.

## Issues Encountered

None beyond the crate-root `#[path]` mapping documented above.

## Known Stubs

Intentional empty modules so later plans own a single file:

- `tools/xtask/tests/playground_cli/bundle.rs:1` — `//! Audit-bundle CLI tests land in 23-03.`
- `tools/xtask/tests/playground_cli/heap.rs:1` — `//! Heap CLI tests land in 23-06.`

These stubs contain no `#[test]` and no `dam-break-audit-bundle` / `dhat-heap` strings. They do not block this plan's goal.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-02: copy-only same-HEAD audit-bundle command and thin just alias.
- 23-03 can add bundle CLI tests only in `playground_cli/bundle.rs`.
- 23-06 can add heap CLI tests only in `playground_cli/heap.rs`.
- Do not add `dam-break-audit-bundle` or `dam-break-heap` in 23-01 leftovers.
- Do not run live Dam Break, cmake, host samply, or `bun scripts/bright-builds-check.ts all` until 23-09.

## Self-Check: PASSED

***
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*

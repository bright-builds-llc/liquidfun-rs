---
phase: 23-baseline-pair-and-named-audit
plan: "02"
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T03:58:24Z
subsystem: observability-tooling
tags: [xtask, playground-cli, audit-bundle, exclusive-stamp]

requires:
  - phase: 22-observability-shell
    provides: exclusive stamp mint, unprofiled pair.json, samply profile stamp
  - phase: 23-baseline-pair-and-named-audit
    provides: playground_cli split so 23-03 can own bundle CLI tests
provides:
  - copy-only same-HEAD dam-break-audit-bundle command
  - parse_stamp_name allowlist YYYY-MM-DDTHH-MM-SSZ
  - just playground-dam-break-audit-bundle one-line alias
affects:
  - 23-03 fake-stamp audit-bundle CLI tests
  - 23-07 live pair/profile/bundle stamps for the named audit

tech-stack:
  added: []
  patterns:
    - copy-only exclusive audit-bundle stamp with fs::copy, never rename or remove_dir_all
    - playground/bundle.rs plus bundle/ops.rs to stay under the 628-line cap

key-files:
  created:
    - tools/xtask/src/playground/bundle.rs
    - tools/xtask/src/playground/bundle/ops.rs
  modified:
    - tools/xtask/src/playground.rs
    - tools/xtask/src/playground/stamp.rs
    - tools/xtask/src/main.rs
    - justfile

key-decisions:
  - "Copy pair.json, pair.md, rust.json.gz, and *syms* sidecars with fs::copy into a new exclusive stamp; never move or overwrite sources."
  - "Split bundle helpers into bundle/ops.rs so every playground file stays ≤628 physical lines."
  - "Do not mark PERF-AUDIT complete in this plan; the named-function audit remains 23-07."
  - "Run playground unit tests as cargo test -p xtask playground::bundle because xtask has no lib target."

patterns-established:
  - "just playground-dam-break-audit-bundle is a one-line cargo xtask printer with no cmake, samply, or dhat flags."
  - "audit-bundle-identity.json keeps timing_authority unprofiled_wall_clock and marks the profile blob not_timing_authority true."
  - "Stamp CLI flags join only as evidence_dir.join(parse_stamp_name(raw))."

requirements-completed: []

duration: 14min
completed: 2026-09-21
---

# Phase 23 Plan 02: Copy-Only Audit Bundle Summary

**Copy-only same-HEAD `dam-break-audit-bundle` mints a new exclusive stamp with `pair.json`, `pair.md`, `rust.json.gz`, and `*syms*` sidecars, plus a one-line `just` alias.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-09-21T03:44:20Z
- **Completed:** 2026-09-21T03:58:24Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `parse_stamp_name` allowlisting `YYYY-MM-DDTHH-MM-SSZ` and rejecting `/`, `\`, `..`, `:`, NUL, and empty names.
- Implemented copy-only same-HEAD bundling: `fs::copy` of pair/profile artifacts into a new exclusive stamp, with `audit-bundle-identity.json` marking the profile blob `not_timing_authority`.
- Wired `just playground-dam-break-audit-bundle` as a one-line `cargo xtask playground dam-break-audit-bundle` alias with no cmake, samply, or dhat flags.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: TDD stamp-name parse and copy-only bundle tests** - `3975e06` (test)
2. **Task 1 GREEN: implement stamp-name parse and copy-only audit bundle** - `2fa7e28` (feat)
3. **Task 2: Wire dispatcher USAGE and the one-line just alias** - `eb739e6` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## RED excerpt

Plan command `cargo test -p xtask --lib playground::bundle` failed first (no library target):

```text
error: no library targets found in package `xtask`
```

Binary-unit RED (`cargo test -p xtask playground::bundle` / `playground::stamp`) then failed 5 bundle tests and 5 stamp-name tests against `todo!()` stubs:

```text
test playground::bundle::tests::copy_audit_bundle_copies_pair_profile_and_syms_without_moving_sources ... FAILED
test playground::stamp::tests::parse_stamp_name_accepts_filename_safe_utc ... FAILED
not yet implemented: copy_audit_bundle
not yet implemented: parse_stamp_name
test result: FAILED. 0 passed; 5 failed
```

GREEN: 5 bundle tests and 11 stamp tests passed; `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0.

## Files Created/Modified

- `tools/xtask/src/playground/bundle.rs` — module root, `fs::copy` helper, unit tests under `target/xtask-test-fixtures/`
- `tools/xtask/src/playground/bundle/ops.rs` — scan/validate/copy orchestration, identity JSON, duplicated `LIQUIDFUN_XTASK_STAMP_UNIX` read
- `tools/xtask/src/playground/stamp.rs` — `evidence_dir` and `parse_stamp_name`
- `tools/xtask/src/playground.rs` — `mod bundle;`, dispatcher arm, USAGE, missing-subcommand text
- `tools/xtask/src/main.rs` — playground help mentions audit bundle
- `justfile` — `playground-dam-break-audit-bundle` one-line alias after timers

## Physical line counts

Counted with Python `sum(1 for _ in path.open())` including blanks and comments:

| File | Lines | Cap |
| --- | ---: | ---: |
| `tools/xtask/src/playground.rs` | 68 | 628 |
| `tools/xtask/src/playground/bundle.rs` | 343 | 628 |
| `tools/xtask/src/playground/bundle/ops.rs` | 431 | 628 |
| `tools/xtask/src/playground/stamp.rs` | 362 | 628 |

## Decisions Made

- Keep copy-only exclusive mint: sources stay in place; destination is a new stamp. Copied `pair.json` retains `timing_authority: unprofiled_wall_clock` and is not rewritten with samply ratios.
- Split helpers into `bundle/ops.rs` after the combined file exceeded 628 lines. `fs::copy` stays in `bundle.rs` so the plan grep still matches.
- Honor `LIQUIDFUN_XTASK_STAMP_UNIX` inside `bundle/ops.rs` without editing `pair.rs`.
- Git status/rev-parse use `Command::new(LIQUIDFUN_XTASK_GIT else git)` argv lists, never `sh -c`. Dirty worktrees warn and set optional `worktree_dirty`; they do not hard-fail.
- Do not mark REQUIREMENTS `PERF-AUDIT` complete: this plan ships the bundle command; named-function notes remain 23-07.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xtask has no lib target**
- **Found during:** Task 1 RED verification
- **Issue:** Plan command `cargo test -p xtask --lib playground::bundle` fails with `no library targets found in package xtask` (same as 22-01/22-02/22-03).
- **Fix:** Ran `cargo test -p xtask playground::bundle -- --test-threads=1` and `playground::stamp` against binary unit tests.
- **Files modified:** none (verification command only)
- **Verification:** RED 5+5 failed; GREEN 5 bundle + 11 stamp passed
- **Committed in:** n/a

**2. [Rule 3 - Blocking] bundle.rs exceeded the 628-line cap after GREEN**
- **Found during:** Task 1 GREEN
- **Issue:** Combined implementation plus tests reached 763 physical lines.
- **Fix:** Split helpers into `tools/xtask/src/playground/bundle/ops.rs` (`foo.rs` plus `foo/`). Kept `fs::copy` in `bundle.rs`.
- **Files modified:** `tools/xtask/src/playground/bundle.rs`, `tools/xtask/src/playground/bundle/ops.rs`
- **Verification:** both files ≤628; clippy `-D warnings` on xtask exits 0; unit tests pass
- **Committed in:** `2fa7e28`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Required for the plan's own 628-line cap and the xtask binary-only test layout. No live Dam Break, no `dam-break-heap`, and no samply flags in just.

## Issues Encountered

None beyond the `--lib` filter and file-length split documented above.

## Known Stubs

None. `todo!()` RED stubs were replaced in GREEN.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-03: fake-stamp CLI tests in `tools/xtask/tests/playground_cli/bundle.rs` only.
- Do not add `dam-break-heap` here (23-05).
- Do not run live Dam Break, cmake, or host samply until 23-07/23-09.
- Do not treat samply durations or the copied gzip as the 3× number.

## Self-Check: PASSED

***
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*

---
phase: 22-observability-shell
plan: "04"
subsystem: observability-tooling
tags: [xtask, playground, samply, dam-break-profile, not-timing-authority]

requires:
  - phase: 22-observability-shell
    provides: workspace [profile.profiling] plus fail-closed samply 0.13.1 argv helpers
provides:
  - just playground-dam-break-profile one-line alias
  - dam-break-profile spawn that writes rust.json.gz and profile-identity.json
  - fake samply CLI success and missing-path fail-closed coverage
affects:
  - 22-05 dam-break-timers
  - PERF-PROFILE
  - Phase 23 named-function audit

tech-stack:
  added: []
  patterns:
    - Command::new(LIQUIDFUN_XTASK_SAMPLY else samply).args(samply_record_argv)
    - cargo build --profile profiling then wrap the profiling binary, never samply record cargo
    - success tests inject fake samply; fail-closed tests use a nonexistent program path

key-files:
  created: []
  modified:
    - justfile
    - tools/xtask/src/playground.rs
    - tools/xtask/src/playground/profile.rs
    - tools/xtask/src/main.rs
    - tools/xtask/tests/playground_cli.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs

key-decisions:
  - "just playground-dam-break-profile is a one-line cargo xtask alias with no cmake or samply flags."
  - "Profile cargo uses --profile profiling and never cargo run --release; samply argv never starts with cargo."
  - "Missing samply (LIQUIDFUN_XTASK_SAMPLY at a nonexistent path) fails closed with 0.13.1 install text and writes no rust.json.gz."
  - "profile-identity.json is kind samply_cpu with not_timing_authority true; the stamp has no pair.json."

patterns-established:
  - "Default dam-break-profile path is Rust-only: cargo build --profile profiling then samply wrap, no cmake."
  - "Success CLI tests inject fake samply; fail-closed tests use a missing LIQUIDFUN_XTASK_SAMPLY path."
  - "Identity sets warmup_included_in_samples true because the wrap records the existing binary including warmup."

requirements-completed: [PERF-PROFILE]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T00:25:12Z

duration: 9min
completed: 2026-09-21
---

# Phase 22 Plan 04: Dam Break Profile Capture Summary

**Thin `just playground-dam-break-profile` rebuilds `[profile.profiling]` `dam-break-bench`, wraps samply 0.13.1 `--save-only`, and writes `rust.json.gz` plus `not_timing_authority` identity into a new stamp.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-09-21T00:16:27Z
- **Completed:** 2026-09-21T00:25:12Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `just playground-dam-break-profile` as a one-line alias for `cargo xtask playground dam-break-profile` with no cmake, ninja, `-g`, samply, or `--cpp-debuginfo` flags.
- Wired `profile::run`: verify samply 0.13.1, `cargo build -p liquidfun-wasm --bin dam-break-bench --profile profiling`, mint an exclusive stamp, then `Command::new(samply).args(samply_record_argv)`.
- Persist `rust.json.gz` (samply `-o` only) plus `profile-identity.json` (`kind: samply_cpu`, `not_timing_authority: true`, `cargo_profile: profiling`, `samply_version: 0.13.1`, exact command). No `pair.json`.
- Proved the success path with fake samply. Missing `LIQUIDFUN_XTASK_SAMPLY` fails closed with install text and writes no placeholder gzip.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement dam-break-profile command and thin just alias** - `c13eac0` (feat)
2. **Task 2 RED: Fake samply CLI success and missing-samply fail-closed** - `e2e6e5b` (test)
3. **Task 2 GREEN: Fake samply CLI success and missing-samply fail-closed** - `524158f` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

- `justfile` — thin `playground-dam-break-profile` alias; unprofiled bench recipe unchanged
- `tools/xtask/src/playground.rs` — dispatch `dam-break-profile` to `profile::run`
- `tools/xtask/src/playground/profile.rs` — samply/cargo spawn, identity persist, CARGO_TARGET_DIR-aware binary path
- `tools/xtask/src/main.rs` — playground usage mentions pair or CPU profile
- `tools/xtask/tests/playground_cli.rs` — fake-samply success, missing-path fail-closed, stamp collision
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` — fake samply 0.13.1 and `--profile profiling` cargo stub

## Decisions Made

- Keep `just` as a one-line printer (D-06/D-07). xtask owns samply lookup, cargo `--profile profiling`, stamp mint, and identity write.
- Default profile path is Rust-only. Do not call cmake, do not implement `--cpp-debuginfo`, do not wrap `cargo run --release`.
- Wrap the existing `dam-break-bench` binary (research A1). Identity records `warmup_included_in_samples: true`. Do not use `--pid`.
- Missing samply is a production lookup failure (`LIQUIDFUN_XTASK_SAMPLY` then `"samply"`). Success tests inject the fake; the fail-closed test points at `/nonexistent/liquidfun-samply-missing`.
- Mark `PERF-PROFILE` complete. Live host samply / Instruments remain optional developer proof, not definition of done.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `Path::strip_prefix` `map_or_else` closure arity**
- **Found during:** Task 1 (clippy/build)
- **Issue:** `map_or_else(|| ..., |relative| ...)` does not compile; the error mapper must take the `StripPrefixError`.
- **Fix:** Match on `strip_prefix` `Ok`/`Err`.
- **Files modified:** `tools/xtask/src/playground/profile.rs`
- **Verification:** `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0
- **Committed in:** `c13eac0`

**2. [Rule 2 - Missing Critical] Require nonempty `rust.json.gz` after samply success**
- **Found during:** Task 1 (T-22-04-01)
- **Issue:** A zero-exit samply that wrote nothing would still mint identity.
- **Fix:** After a successful samply spawn, require `rust.json.gz` to exist and be nonempty before writing `profile-identity.json`. Do not write a placeholder gzip.
- **Files modified:** `tools/xtask/src/playground/profile.rs`
- **Verification:** Fake samply writes 19 bytes; CLI success test asserts `gzip.len() >= 16`
- **Committed in:** `c13eac0`

***

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes keep fail-closed identity/gzip semantics. No scope creep.

## Issues Encountered

None beyond the `map_or_else` compile error documented above. The missing-samply CLI test already passed in the TDD RED run because Task 1 implemented production fail-closed lookup before the fake existed.

## User Setup Required

None - no external service configuration required. Optional live `samply` 0.13.1 remains a host tool (`cargo install --locked samply --version 0.13.1`); tests do not require it.

## Next Phase Readiness

- Ready for 22-05: `dam-break-timers` on a separate diagnostic path. Do not call `step_profiled` from `dam-break-bench`.
- Keep `just playground-dam-break-bench` as the unprofiled 3× authority. Profiled wall times must never enter `pair.json`.
- Do not run live samply or Instruments as a 22-04 gate.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-21*

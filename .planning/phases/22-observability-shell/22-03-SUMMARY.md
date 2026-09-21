---
phase: 22-observability-shell
plan: "03"
subsystem: observability-tooling
tags: [xtask, playground, samply, profile.profiling, fail-closed]

requires:
  - phase: 22-observability-shell
    provides: playground.rs dispatcher plus exclusive UTC stamp minting
provides:
  - workspace [profile.profiling] inheriting release with debug = true and strip = false
  - pure samply 0.13.1 parse/require/argv helpers in playground/profile.rs
affects:
  - 22-04 dam-break-profile spawn
  - PERF-PROFILE

tech-stack:
  added:
    - samply 0.13.1 argv contract (host tool, not a crate dependency)
  patterns:
    - named Cargo [profile.profiling] instead of CARGO_PROFILE_RELEASE_DEBUG
    - Vec<OsString> samply flags starting at record; program is a separate Command

key-files:
  created:
    - tools/xtask/src/playground/profile.rs
  modified:
    - Cargo.toml
    - tools/xtask/src/playground.rs

key-decisions:
  - "Declare workspace [profile.profiling] with inherits = release, debug = true, strip = false; do not create [profile.release] or set CARGO_PROFILE_RELEASE_DEBUG."
  - "samply_record_argv is flags only: record --save-only --unstable-presymbolicate -o <gz> -- <profiling binary> --warmup N --steps M; never cargo."
  - "Missing or non-0.13.1 samply is PlaygroundError kind samply with cargo install --locked, brew install samply, and samply setup; no skip wording."
  - "Do not mark PERF-PROFILE complete in this plan; dam-break-profile spawn and just alias are 22-04."

patterns-established:
  - "Profiling helpers are a pure functional core (version string in, argv/error out) with no Command spawn until 22-04."
  - "Default --release stays the unprofiled gate; debuginfo lives only on the named profiling profile."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T00:12:25Z

duration: 8min
completed: 2026-09-21
---

# Phase 22 Plan 03: Profiling Profile and Fail-Closed Samply Argv Summary

**Workspace `[profile.profiling]` inherits release with full debuginfo, and xtask now has a tested fail-closed samply 0.13.1 argv/version core that never spawns samply or mutates `--release`.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-21T00:04:15Z
- **Completed:** 2026-09-21T00:12:25Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added workspace `[profile.profiling]` (`inherits = "release"`, `debug = true`, `strip = false`) without a `[profile.release]` debug override and without `CARGO_PROFILE_RELEASE_DEBUG`.
- Landed TDD-covered `parse_samply_version`, `require_samply_version` / `require_samply_0_13_1`, `missing_samply_error`, `samply_record_argv`, and `profiling_dam_break_bench_bin` in `playground/profile.rs`.
- Locked argv is `record --save-only --unstable-presymbolicate -o <rust.json.gz> -- <target/profiling/dam-break-bench> --warmup 60 --steps 600`; no `cargo` element; Windows uses `.exe`.
- Fail-closed missing/wrong-version copy includes pin `0.13.1`, `cargo install --locked samply --version 0.13.1`, `brew install samply`, and `samply setup`, and does not mention skipping.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: TDD samply argv, version pin, and missing-tool copy** - `1bfd86f` (test)
2. **Task 2 GREEN: TDD profiling profile and samply helpers** - `f158177` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## RED excerpt

Plan command `cargo test -p xtask --lib playground::profile` failed first (no library target):

```text
error: no library targets found in package `xtask`
```

Binary-unit RED (`cargo test -p xtask playground::profile`) then failed 4 of 7 tests against stubs:

```text
test playground::profile::tests::parse_samply_version_reads_pinned_0_13_1 ... FAILED
test playground::profile::tests::require_samply_version_rejects_other_version_with_install_text ... FAILED
test playground::profile::tests::missing_samply_error_includes_install_text_and_not_skip ... FAILED
test playground::profile::tests::samply_record_argv_matches_locked_flag_sequence ... FAILED
test result: FAILED. 3 passed; 4 failed
```

GREEN: 7 passed; `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0.

## Files Created/Modified

- `Cargo.toml` — workspace `[profile.profiling]` table; default `--release` unchanged
- `tools/xtask/src/playground.rs` — `mod profile;` only; still no `dam-break-profile` match arm
- `tools/xtask/src/playground/profile.rs` — pure samply 0.13.1 helpers and unit tests; no process spawn

## Decisions Made

- Followed D-08: named profiling profile, not `CARGO_PROFILE_RELEASE_DEBUG` or `[profile.release] debug = true`.
- Followed D-09: missing/wrong samply is `Err` with install text; no skip path and no placeholder gzip.
- Argv builder omits the program path so 22-04 can `Command::new(samply).args(argv)` without joining a shell string.
- Left `profile::run()` and just aliases for 22-04. Did not mark REQUIREMENTS `PERF-PROFILE` complete.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xtask has no lib target**
- **Found during:** Task 1 RED verification
- **Issue:** Plan command `cargo test -p xtask --lib playground::profile` fails with `no library targets found in package xtask` (same as 22-01/22-02).
- **Fix:** Ran `cargo test -p xtask playground::profile -- --test-threads=1` against the binary unit tests.
- **Files modified:** none (verification command only)
- **Verification:** RED 4 failed / 3 passed; GREEN 7 passed
- **Committed in:** n/a

**2. [Rule 3 - Blocking] Clippy denied `let...else` on Option in parse_samply_version**
- **Found during:** Task 2 GREEN
- **Issue:** `clippy::question_mark` under `-D warnings` required `let line = maybe_line?;` instead of `let Some(line) = maybe_line else { return None; }`.
- **Fix:** Used `?` for the empty-line Option path; kept `let...else` on `require_samply_version` where the fallback is `missing_samply_error()`.
- **Files modified:** `tools/xtask/src/playground/profile.rs`
- **Verification:** clippy `-D warnings` on xtask exits 0; 7 unit tests pass
- **Committed in:** `f158177`

**3. [Rule 2 - Missing Critical] Allow unused profile APIs until 22-04**
- **Found during:** Task 1 RED / Task 2 GREEN
- **Issue:** Helpers are intentionally unused by `dam-break-bench`; without an allow, non-test xtask would warn `dead_code`.
- **Fix:** Module-level `#![allow(dead_code)]` with a comment that 22-04 is the first production caller.
- **Files modified:** `tools/xtask/src/playground/profile.rs`
- **Verification:** clippy `-D warnings` on xtask passes
- **Committed in:** `1bfd86f`

***

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical)
**Impact on plan:** Verification command adapted to the binary crate; Cargo profile and fail-closed samply helpers still match D-08/D-09. No scope creep.

## Issues Encountered

None beyond the verification-command mismatch and Clippy `question_mark` lint documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 22-04: wire `dam-break-profile`, thin `just` alias, fake-samply CLI, and `profile-identity.json`.
- Do not copy profiled timings into `pair.json`.
- Do not run real samply or `cargo build --profile profiling` as a 22-03 gate.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-21*

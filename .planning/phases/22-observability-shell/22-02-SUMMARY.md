---
phase: 22-observability-shell
plan: "02"
subsystem: observability-tooling
tags: [xtask, playground, dam-break-bench, pair-json, unprofiled-wall-clock]

requires:
  - phase: 22-observability-shell
    provides: playground.rs dispatcher plus exclusive UTC stamp minting
provides:
  - dated pair.json and pair.md with rust_over_cpp_ratio under target/dam-break-perf
  - fake cmake/cargo/cpp CLI coverage for unprofiled pair persist and stamp collision
affects:
  - 22-03 profile command
  - 22-04 timer command
  - PERF-PAIR

tech-stack:
  added: []
  patterns:
    - exclusive mint then write pair.json/pair.md; never create_dir_all on an existing stamp
    - LIQUIDFUN_XTASK_CARGO else CARGO else cargo via Command::new(path).args
    - test-only LIQUIDFUN_XTASK_STAMP_UNIX as a u64 integer, never a path

key-files:
  created:
    - tools/xtask/tests/playground_cli.rs
  modified:
    - tools/xtask/src/playground/pair.rs
    - tools/xtask/src/playground/stamp.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs

key-decisions:
  - "Persist pair.json and pair.md only after exclusive stamp mint and a finite C++ wall_ms; zero C++ wall is a bench error with no files written."
  - "Keep just playground-dam-break-bench as the one-line cargo xtask alias; do not hide cmake or samply in just."
  - "Honor LIQUIDFUN_XTASK_CARGO else CARGO else cargo, and LIQUIDFUN_XTASK_STAMP_UNIX as test-only unix seconds."
  - "pair.json kind is unprofiled_pair with timing_authority unprofiled_wall_clock; omit samply/profile keys."

patterns-established:
  - "Unprofiled pair stdout Markdown bytes are also pair.md; pair.json is pretty serde_json with rust_over_cpp_ratio."
  - "Playground CLI proof uses fake cmake/cargo/cpp copied from the upstream_cli fake-tool pattern, not a live Dam Break pair."

requirements-completed: [PERF-PAIR]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T00:01:32Z

duration: 9min
completed: 2026-09-21
---

# Phase 22 Plan 02: Unprofiled Pair Persistence Summary

**Persist dated `pair.json` and `pair.md` (with Rust/C++ wall-ms ratio) from `just playground-dam-break-bench` into exclusive `target/dam-break-perf/<utc-stamp>/` stamps, proven with fake cmake/cargo/cpp.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-09-20T23:53:00Z
- **Completed:** 2026-09-21T00:01:32Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `rust_over_cpp_ratio`, `pair_report_json`, and a Markdown `Rust/C++ wall-ms ratio` line; C++ `wall_ms` of 0, negative, or non-finite is a bench error, not Inf.
- After both samples validate, mint an exclusive UTC stamp and write pretty `pair.json` (`kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`) plus `pair.md` matching the stdout table.
- Look up cargo as `LIQUIDFUN_XTASK_CARGO` else `CARGO` else `"cargo"` via `Command::new(program).args([...])`; keep `--release` and never call `step_profiled`.
- Proved persist, ratio 300.0, cmake `--target playground-dam-break-bench` without `-g`, and same-second stamp bump with fake tools. Left `just playground-dam-break-bench` as the one-line cargo xtask alias.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: TDD pair report JSON, Markdown ratio, and zero C++ wall error** - `e6021e0` (test)
2. **Task 1 GREEN: TDD pair report JSON, Markdown ratio, and zero C++ wall error** - `7a74330` (feat)
3. **Task 2: Persist pair.json/pair.md via exclusive stamp and fake-tool CLI** - `f28d9fc` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

- `tools/xtask/src/playground/pair.rs` — ratio, JSON shape, exclusive persist of `pair.json`/`pair.md`, cargo/stamp env lookup
- `tools/xtask/src/playground/stamp.rs` — removed 22-01 dead_code allow now that pair is the production caller
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` — fake cargo Dam Break JSON and fake `playground-dam-break-bench` C++ twin
- `tools/xtask/tests/playground_cli.rs` — fake-tool persist, stamp-collision, and justfile alias coverage

## Decisions Made

- Fail closed on non-finite/zero C++ wall **before** minting a stamp, so a bad sample never writes Inf or empty evidence.
- Inject `LIQUIDFUN_XTASK_STAMP_UNIX` as a `u64` only (reject `/`, `\`, `..`) so CLI tests can force same-second collision without a user stamp-name override.
- Keep cmake configure/build stdout in front of the Markdown table; `pair.md` bytes match `render_markdown`, and CLI tests assert stdout contains that table plus the unreviewed banner.
- Do not edit `justfile`, `docs/playground-dam-break-timing.md`, or `reference/performance/manifest.toml`. Do not run a live Dam Break pair as definition of done.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xtask has no lib target**
- **Found during:** Task 1 verification
- **Issue:** Plan command `cargo test -p xtask --lib playground::pair` fails with no library targets in package xtask (same as 22-01).
- **Fix:** Ran `cargo test -p xtask playground::pair` against the binary unit tests.
- **Files modified:** none (verification command only)
- **Verification:** 10 playground pair unit tests pass
- **Committed in:** n/a

**2. [Rule 1 - Bug] Clippy denied float equality in the ratio unit test**
- **Found during:** Task 1 GREEN
- **Issue:** `assert_eq!(ratio, 31_200.0 / 6_000.0)` triggered `clippy::float_cmp` under `-D warnings`.
- **Fix:** Compare with `f64::EPSILON` instead of `assert_eq!`.
- **Files modified:** `tools/xtask/src/playground/pair.rs`
- **Verification:** `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0
- **Committed in:** `7a74330`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Verification command adapted to the binary crate; ratio and persist behavior still match D-01/D-02/D-05. No scope creep.

## Issues Encountered

None beyond the verification-command mismatch and Clippy float comparison documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 22-03: `[profile.profiling]` plus `dam-break-profile` samply wrap. Do not copy profiled timings into `pair.json`.
- Keep `just playground-dam-break-bench` as the unprofiled 3× authority.
- Do not run a live Dam Break pair, samply, or Instruments as a 22-02 gate.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-21*

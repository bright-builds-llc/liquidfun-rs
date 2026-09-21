---
phase: 22-observability-shell
plan: "05"
subsystem: observability-tooling
tags: [xtask, playground, dam-break-timers, step_profiled, not-timing-authority]

requires:
  - phase: 22-observability-shell
    provides: exclusive stamp mint plus dam-break-profile dispatch
provides:
  - native dam-break-timers binary with SessionCore::advance_profiled
  - just playground-dam-break-timers one-line alias
  - timers.json persist into a new stamp with not_timing_authority
affects:
  - 22-06 isolation and BENCHMARKING note
  - PERF-TIMERS
  - Phase 23 named-function audit

tech-stack:
  added: []
  patterns:
    - SessionCore::advance_profiled loops one World::step_profiled; MAX_ADVANCE_STEPS stays 4
    - cargo run -p liquidfun-wasm --release --bin dam-break-timers, never dam-break-bench
    - Fake cargo prints step_profiled_parents JSON when args contain dam-break-timers

key-files:
  created:
    - crates/liquidfun-wasm/src/dam_break_timers.rs
    - crates/liquidfun-wasm/src/bin/dam_break_timers.rs
    - tools/xtask/src/playground/timers.rs
  modified:
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/Cargo.toml
    - tools/xtask/src/playground.rs
    - tools/xtask/src/main.rs
    - justfile
    - tools/xtask/tests/playground_cli.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs

key-decisions:
  - "dam-break-timers is a separate native binary; dam-break-bench still calls only ordinary advance/World::step."
  - "just playground-dam-break-timers is a one-line cargo xtask alias with no cmake or samply flags."
  - "A fake timers run writes timers.json into a new stamp with not_timing_authority true and no pair.json."
  - "MAX_ADVANCE_STEPS remains 4; measured steps loop advance_profiled(1)."

patterns-established:
  - "Parent-phase timers live on a sibling Dam Break process, never the unprofiled 3x gate binary."
  - "Aggregate maybe_common_parent durations only; include every DiagnosticProfileParent::ALL token."
  - "xtask mints an exclusive stamp after validating kind step_profiled_parents and boolean not_timing_authority."

requirements-completed: [PERF-TIMERS]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T00:41:27Z

duration: 13min
completed: 2026-09-21
---

# Phase 22 Plan 05: Dam Break Parent Timers Summary

**Separate `dam-break-timers` native binary loops `advance_profiled(1)` to emit Phase 12 parent walls (`particle_prepare` / `particle_solve` / `rigid_solve` plus `DiagnosticProfileParent::ALL`) into a new `timers.json` stamp; the unprofiled `dam-break-bench` gate stays on ordinary `World::step`.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-21T00:28:15Z
- **Completed:** 2026-09-21T00:41:27Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added native-only `SessionCore::advance_profiled` that reuses the `on_advance` hook then `World::step_profiled`, incrementing `step_index` by 1. `MAX_ADVANCE_STEPS` remains 4. `ProofSession` does not export `advance_profiled`.
- Added `dam-break-timers` (`test = false`, `bench = false`) plus `run_dam_break_timers`. Warm-up uses `advance(1)` loops; measured steps use `advance_profiled()` loops. Parents sum `maybe_common_parent` durations only.
- Added `just playground-dam-break-timers` as a one-line alias for `cargo xtask playground dam-break-timers` with no cmake or samply flags.
- Wired `timers::run`: `--release` `cargo run -p liquidfun-wasm --bin dam-break-timers`, require `kind == "step_profiled_parents"` and boolean `not_timing_authority`, mint a new stamp, write `timers.json` with host identity. No `pair.json`. No ratio.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Native timer binary and advance_profiled** - `82e04c6` (test)
2. **Task 1 GREEN: Native timer binary and advance_profiled** - `6ca1289` (feat)
3. **Task 2: xtask dam-break-timers persist, just alias, fake CLI** - `30c489a` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

- `crates/liquidfun-wasm/src/session.rs` — native-only `advance_profiled` wrapping `step_profiled`
- `crates/liquidfun-wasm/src/dam_break_timers.rs` — Dam Break recipe reuse, parent aggregation, JSON stdout, tests
- `crates/liquidfun-wasm/src/bin/dam_break_timers.rs` — argv parser calling `run_dam_break_timers`
- `crates/liquidfun-wasm/src/lib.rs` — native-only `dam_break_timers` module export
- `crates/liquidfun-wasm/Cargo.toml` — `[[bin]] name = "dam-break-timers"`
- `tools/xtask/src/playground.rs` — dispatch `dam-break-timers` to `timers::run`
- `tools/xtask/src/playground/timers.rs` — cargo spawn, JSON validate, exclusive stamp, `timers.json`
- `tools/xtask/src/main.rs` — playground usage mentions parent timers
- `justfile` — thin `playground-dam-break-timers` alias
- `tools/xtask/tests/playground_cli.rs` — fake-cargo stamp with `timers.json` and no `pair.json`
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` — fake cargo prints parent-timer JSON

## Decisions Made

- Keep `step_profiled` off `dam-break-bench` (D-10/D-11). The gate binary still uses `session.advance(1)` / `World::step`.
- `just` remains a one-line printer (D-06). xtask owns `--release` cargo, stamp mint, identity merge, and persist.
- Timer runs mint their own stamp (D-04). Fake CLI asserts `timers.json` exists and `pair.json` / `rust.json.gz` do not.
- Loop `advance_profiled()` once per measured step. Do not raise `MAX_ADVANCE_STEPS` and do not pass 600 as a single advance count (Pitfall 6).
- Mark `PERF-TIMERS` complete. Live 600-step timers are optional developer proof, not definition of done.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required. Timer tooling is proven with fake cargo; a live Dam Break timer run is optional.

## Next Phase Readiness

- Ready for 22-06: package isolation, one-line BENCHMARKING.md stamp note, and confirmation that public timing docs / `manifest.toml` stay unmodified.
- Keep `just playground-dam-break-bench` as the unprofiled 3× authority. Timer `wall_ms` must never enter `pair.json`.
- Do not run 600-step live timers as a 22-05 gate.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-21*

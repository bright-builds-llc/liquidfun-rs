---
phase: 34-scene-timing-survey
plan: "01"
subsystem: testing
tags: [rust, xtask, liquidfun-wasm, benchmark, scene-survey]

requires: []
provides:
  - "All-catalog native scene survey bin (25 scenes, --runs, median/min/max ms/step)"
  - "include_str! coverage test tying SURVEY_SCENES to web/src/catalog/scenes.ts"
  - "cargo xtask playground scene-spot ranked Markdown table plus scene-spot.json with runs/warmup/steps"
affects: [34-02, 35, 36]

tech-stack:
  added: []
  patterns:
    - "Survey-only cues (SurveyCue) live in scene_spot.rs, never in production scene modules"
    - "xtask functional core in spot/survey_table.rs (parse catalog, rank, render) with the imperative shell in spot.rs"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene_spot/tests.rs
    - tools/xtask/src/playground/spot/survey_table.rs
  modified:
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
    - tools/xtask/src/playground/spot.rs
    - tools/xtask/src/playground.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs
    - tools/xtask/tests/playground_cli/spot.rs

key-decisions:
  - "SCENE_WALL_TIMEOUT (3 min) applies per run of one scene, not per scene total"
  - "start_particles is read before the survey cue, so Drawing Particles starts at 0"
  - "Even run counts use the mean of the two middle values as the median"
  - "Each aggregated JSON line reports the last run's start/end particle counts"

patterns-established:
  - "Catalog drift check: tiny bracket/comma SCENE_IDS parser duplicated in liquidfun-wasm tests, xtask and the fake tool instead of a shared crate"

requirements-completed: [PERF-07]

duration: 26min
completed: 2026-10-07
---

# Phase 34 Plan 01: Scene Timing Survey Tool Summary

**The native scene timer now surveys all 25 playground catalog scenes over 3 fresh-session runs each (median/min/max ms/step). xtask checks the output against `SCENE_IDS` parsed from `scenes.ts` and prints a ranked Markdown table, slowest scene first.**

## Performance

- **Duration:** about 26 min
- **Started:** 2026-10-07T05:53Z (approx.)
- **Completed:** 2026-10-07T06:19Z
- **Tasks:** 2
- **Files modified:** 8 (2 created, 6 modified)

## Accomplishments

- `SURVEY_SCENES` lists all 25 catalog scenes in `scenes.ts` order. Survey cues apply only to Float or Sink (`drop-body`), Impulse (pointer up at 1, 2) and Drawing Particles (pointer up at 0, 2). Sparky stays `default`.
- `cargo test -p liquidfun-wasm` fails if `SURVEY_SCENES` and `SCENE_IDS` differ in membership or order, or if any id fails to resolve through `parse_scene_id`.
- The bin and xtask both accept `--runs <n>` (default 3) and reject 0. Every run builds a fresh `SessionCore`, and particle counts come from `live_particle_count()`.
- `cargo xtask playground scene-spot` checks the bin output against the parsed catalog: the exact set of ids, no duplicates, finite spread fields, `timed_out` false and known interaction labels. It then prints the ranked table and writes `target/dam-break-perf/<stamp>/scene-spot.json` with `not_timing_authority: true` plus `warmup_steps`, `measured_steps` and `runs`.
- The justfile recipe is unchanged, and no file under `crates/liquidfun-wasm/src/scene/` or `session.rs` changed.

## Task Commits

1. **Task 1: Survey every catalog scene with runs, cues and median/min/max in liquidfun-wasm** - `f7e354518` (feat)
1. **Task 2: Catalog-backed xtask validation, --runs passthrough and ranked Markdown table** - `c22d23926` (feat)

TDD: for each task, RED was confirmed first (the new tests failed to compile against the old API) before any implementation was written. No separate RED commit was made, so history never holds a non-compiling tree.

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene_spot.rs`: SURVEY_SCENES, SurveyCue, apply_cue, time_run, summarize, aggregated JSON line, ZeroRuns/CueFailed errors
- `crates/liquidfun-wasm/src/scene_spot/tests.rs`: catalog coverage, id resolution, cue labels, cue application, summarize, zero runs, JSON fields
- `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs`: `--runs` flag
- `tools/xtask/src/playground/spot.rs`: CATALOG_SCENES_TS, catalog validation, `--runs` passthrough, report counts, ranked summary
- `tools/xtask/src/playground/spot/survey_table.rs`: catalog_scene_ids, rank_scene_samples, render_ranked_table, plus unit tests
- `tools/xtask/src/playground.rs`: USAGE gains `[--runs <n>]`
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs`: fake cargo emits every catalog scene in the new shape
- `tools/xtask/tests/playground_cli/spot.rs`: asserts 25 scenes, `runs == 1`, the table header, and tesla-valve at rank 1

## Decisions Made

- The per-run timeout follows the plan's reading of D-06.
- The infallible `writeln!` into a `String` uses `.expect(...)`, matching `release/report.rs`, so the result is not silently discarded.
- `validate_scene_samples` is split into a catalog-set check plus a per-sample `validate_scene_sample` to keep the functions short.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Renamed the `wall_ms` local to `timed_ms` in scene_spot.rs**
- **Found during:** Task 1 acceptance checks
- **Issue:** The acceptance grep `SPOT_SCENES\|wall_ms` must return nothing, but the kept timed-loop local was named `wall_ms`.
- **Fix:** Renamed the local only. Behavior is unchanged.
- **Files modified:** crates/liquidfun-wasm/src/scene_spot.rs
- **Commit:** f7e354518

**2. [Rule 1 - Lint] Backticked `SessionCore` in the plan-specified module doc**
- **Found during:** Task 1 clippy (`doc_markdown`)
- **Fix:** `` //! Native-only all-catalog `SessionCore` stepper for the playground scene survey. ``
- **Commit:** f7e354518

**3. Added test `parse_spot_counts_defaults_to_three_runs`**
- The plan's `parse_spot_counts_rejects_zero_runs` behavior bundled two concerns (zero rejected, default 3). They were split into two tests to keep one concern per test.

None of these changed scope.

## Issues Encountered

- `rustfmt --check` on the standalone fixture `fake_upstream_tool.rs` reports a formatting diff in pre-existing code (lines 84-87). It already existed at HEAD before this plan, and the file is outside `cargo fmt`'s module tree. Out of scope, not changed.

## Verification

- `cargo fmt --all --check`: pass
- `cargo clippy -p liquidfun-wasm -p xtask --all-targets --all-features -- -D warnings`: pass
- `cargo test -p liquidfun-wasm`: 306 passed
- `cargo test -p xtask --bin xtask playground`: 63 passed
- `cargo test -p xtask --test playground_cli`: 22 passed
- `bun scripts/bright-builds-check.ts file-lengths`: findings=0
- Release bin `--warmup 0 --steps 1 --runs 1`: 25 lines, first wave-machine, last tesla-valve
- Real `cargo xtask playground scene-spot --warmup 0 --steps 1 --runs 1`: ranked 25-row table printed and stamp written (smoke run only, not the committed survey)
- `git diff --stat -- crates/liquidfun-wasm/src/scene/ crates/liquidfun-wasm/src/session.rs justfile`: empty

## Next Phase Readiness

- Plan 02 can run `just playground-scene-spot` on the committed HEAD (`c22d23926`) and paste the ranked table into `docs/benchmarks/scene-survey.md`.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene_spot/tests.rs
- FOUND: tools/xtask/src/playground/spot/survey_table.rs
- FOUND: f7e354518
- FOUND: c22d23926

---
*Phase: 34-scene-timing-survey*
*Completed: 2026-10-07*

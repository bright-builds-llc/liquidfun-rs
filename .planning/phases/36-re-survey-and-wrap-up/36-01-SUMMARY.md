---
phase: 36-re-survey-and-wrap-up
plan: "01"
subsystem: measurement-tooling
tags: [xtask, scene-spot, evidence-stamp, transparency, WR-01]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-10-11T02-24-50
generated_at: "2026-10-11T03:19:07.728Z"
requires:
  - Phase 35 scene-spot `--scene` filter (tools/xtask/src/playground/spot/args.rs)
provides:
  - scene-spot stamps that record `scene_filter` and `full_catalog`
  - a disclaimer chosen from the filter and shared by the stamp and the printed summary
affects:
  - 36-02 re-survey (full-catalog and filtered stamps are now self-describing)
tech-stack:
  added: []
  patterns:
    - one pure `disclaimer_for` value passed to both the stamp writer and the summary renderer (no read-back fallback)
key-files:
  created: []
  modified:
    - tools/xtask/src/playground/spot.rs
    - tools/xtask/tests/playground_cli/spot.rs
key-decisions:
  - "WR-01 fixed in measurement tooling only: filtered scene-spot stamps record scene_filter (request order) and full_catalog false with a requested-scenes-only disclaimer; full-catalog stamps keep the byte-identical original disclaimer and add scene_filter [] and full_catalog true"
requirements: [PERF-10]
duration: 39min
completed: 2026-10-11
---

# Phase 36 Plan 01: Record the scene filter in scene-spot stamps (WR-01) Summary

**A `--scene`-filtered `cargo xtask playground scene-spot` stamp now records `scene_filter` and `full_catalog: false` and says that it covers only the requested scenes. Full-catalog stamps keep the original disclaimer word for word and gain `scene_filter: []` and `full_catalog: true`.**

## Performance

- **Duration:** about 39 min (02:39:59Z to 03:19:07Z). About 23 min of that was the workspace test run.
- **Tasks:** 2 of 2
- **Files modified:** 2

## Accomplishments

- `FULL_DISCLAIMER` keeps the pre-fix text byte for byte. `FILTERED_DISCLAIMER` reads "Unreviewed local survey of the requested playground catalog scenes only (see scene_filter). ...".
- The pure function `disclaimer_for(&[String]) -> &'static str` selects the text. `run` computes the disclaimer once and passes that value to both `assemble_report` and `render_summary`, so the stamp and stdout cannot disagree.
- The stamp has two new fields: `"scene_filter": spot_args.scenes`, kept in request order, and `"full_catalog": spot_args.scenes.is_empty()`.
- The module doc no longer says "all-catalog". The file is 576 lines, under the 628-line limit.
- Two new unit tests cover `disclaimer_for`. The two existing CLI tests were extended to check `full_catalog`, `scene_filter`, the stamp disclaimer and the stdout disclaimer for both a full-catalog run and a filtered run.

## Task Commits

1. **Task 1: Record the scene filter and choose the disclaimer from it.** Tests were written first. The RED run failed: the unit tests did not compile because `FULL_DISCLAIMER`, `FILTERED_DISCLAIMER` and `disclaimer_for` did not exist yet, and both extended CLI tests failed their assertions. Then GREEN: all 23 spot unit tests and all 4 spot CLI tests pass. Task 1 was committed together with Task 2, as the plan asked for a single `fix(36)` commit.
1. **Task 2: Full local checks, smoke test of the real recipe, and the commit.** Commit `c4252c01b`: `fix(36): record scene filter in scene-spot stamps (WR-01)`.

## Check Results (target/phase36/checks-01.results, run on the committed tree)

| Check | Exit | Duration |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | 1 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | 53 s |
| `cargo build --workspace --all-targets --all-features` | 0 | 72 s |
| `bun scripts/bright-builds-check.ts all` | 0 | 1 s |
| `cargo test --workspace --all-features` | 0 | 1402 s |

## Real-Recipe Smoke (untimed, not evidence)

- Command: `cargo xtask playground scene-spot --scene particles --warmup 1 --steps 1 --runs 1`
- Stamp: `target/dam-break-perf/2026-10-11T03-18-57Z/scene-spot.json`
- Result: `full_catalog: false`, `scene_filter: ["particles"]`, exactly 1 scene, and the disclaimer contains "requested playground catalog scenes only". The printed summary shows the same disclaimer.
- The full-catalog recipe was not run; Plan 02 does all timing.

## Decisions Made

- I followed the plan rather than the 35-REVIEW suggestion to read the disclaimer back out of the report in `render_summary`. Passing the computed value directly means there is no fallback that could bring back the full-catalog text.

## Deviations from Plan

- None. The plan was executed as written. The TDD RED step was not committed separately, because the plan specifies exactly one `fix(36)` commit containing only the two xtask files.

## Issues Encountered

None.

## Threat Flags

None. T-36-01 is mitigated as planned: `disclaimer_for` is the single selection point, and both cases are pinned in unit and CLI tests.

## Next Phase Readiness

- Plan 36-02 can run the ABBA re-survey. Any filtered stamp now identifies itself in both the stamp and the summary.
- PERF-10 is not marked complete; Plan 36-03 owns that. No `v1.4` tag was created and no milestone archive was run (D-10).

## Self-Check: PASSED

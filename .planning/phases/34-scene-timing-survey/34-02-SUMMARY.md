---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 34-2026-10-07T05-23-23
generated_at: 2026-10-07T06:22:11Z
phase: 34-scene-timing-survey
plan: "02"
subsystem: docs
tags: [benchmark, scene-survey, docs]

requires:
  - phase: 34-01
    provides: "All-catalog scene survey tool and ranked table output"
provides:
  - "docs/benchmarks/scene-survey.md: committed ranked 25-scene survey with provenance"
  - "Pointer from docs/playground-scene-spot-check.md to the survey; five-scene table marked historical"
affects: [35, 36]

tech-stack:
  added: []
  patterns:
    - "Survey doc table pasted verbatim from xtask stdout, provenance copied from the stamp JSON"

key-files:
  created:
    - docs/benchmarks/scene-survey.md
  modified:
    - docs/playground-scene-spot-check.md

key-decisions:
  - "Recorded survey is stamp 2026-10-07T06-20-54Z on commit 96a3ac6a6 (clean tree, Plan 01 code); docs committed separately in 7174cd676"
  - "Top Phase 35 candidates by median ms/step: liquid-tumbler 24.440, tesla-valve 4.118, stacked-drip 2.640, washing-machine 2.104, particles 1.580"

requirements-completed: [PERF-07]

duration: 5min
completed: 2026-10-07
---

# Phase 34 Plan 02: Record Playground Scene Survey Summary

**One survey run on clean committed source (96a3ac6a6, Apple M4 Max, rustc 1.97.0) is now committed as a ranked 25-scene table in `docs/benchmarks/scene-survey.md`. liquid-tumbler is slowest by far at 24.440 ms/step median.**

## Performance

- **Duration:** about 5 min
- **Started:** 2026-10-07T06:20:25Z
- **Completed:** 2026-10-07T06:25Z
- **Tasks:** 2
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- Ran `just playground-scene-spot` once with the defaults (warmup 60, steps 120, runs 3) on a clean tree. `git status --porcelain -- crates tools web justfile Cargo.toml Cargo.lock` printed nothing before the run.
- Stamp `target/dam-break-perf/2026-10-07T06-20-54Z/scene-spot.json`: `git_head` matches the HEAD taken before the run, runs/warmup/steps are 3/60/120, there are 25 scenes, and none timed out. Only float-or-sink, impulse and drawing-particles show `scripted`.
- `docs/benchmarks/scene-survey.md` holds the disclaimer (local observations, not public speed claims; not a C++ pair, not Phase 12, not the Dam Break 3× number), the reproduce command, the recorded provenance, the verbatim ranked table and the "How to read" notes (cues `drop-body`, (1, 2), (0, 2); Sparky `default`).
- `docs/playground-scene-spot-check.md` now links to the survey, calls its five-scene sample historical, and describes the old five-scene recipe in the past tense.

Top 5 by median ms/step (local observations only):

1. liquid-tumbler: 24.440 (spread 24.435 to 24.648), 3800 particles
1. tesla-valve: 4.118 (spread 3.984 to 4.452), 0 to 2850 particles
1. stacked-drip: 2.640 (spread 2.630 to 2.650), 2000 particles
1. washing-machine: 2.104 (spread 2.055 to 2.581), 3168 particles
1. particles: 1.580 (spread 1.574 to 1.615), 4569 particles

## Task Commits

1. **Task 1: Run the survey on clean committed source**: no commit. The output lives under `target/` and the session scratchpad, and it measured commit `96a3ac6a6`.
1. **Task 2: Write scene-survey.md and point the old page at it**: `7174cd676` (docs)

## Files Created/Modified

- `docs/benchmarks/scene-survey.md`: ranked whole-catalog survey with provenance
- `docs/playground-scene-spot-check.md`: survey pointer, "historical" label, past-tense recipe description

## Decisions Made

- I took the table exactly as stdout printed it (three decimals). mdformat only re-padded the cells.
- I rewrapped the one edited paragraph in the old page to keep lines near 80 columns. The wording matches the plan.

## Deviations from Plan

None. The plan ran as written.

## Issues Encountered

- None. The run finished on the first attempt.
- Observation, not acted on: Fountain and Water Wheel now measure about 1.1 to 1.3 ms/step, against about 20 ms/step in the historical 2026-09-21 five-scene sample. The historical page keeps its numbers unchanged.

## Verification

- Task 1 automated check: printed `ok` for the newest stamp.
- `just markdown-check`: pass
- The table has 25 rows, equal to the SCENE_IDS count. Every catalog id appears in the doc.
- All required phrases are present. The old page contains `(benchmarks/scene-survey.md)` and `historical`.
- `git merge-base --is-ancestor 96a3ac6a6 HEAD` passes, and `git diff --quiet 96a3ac6a6 HEAD -- crates tools web justfile` exits 0.
- `git diff --stat HEAD~1 -- crates tools web justfile` after the docs commit: empty
- Rust pre-commit checks were not rerun because no Rust or code files changed.

## Next Phase Readiness

- Phase 35 can pick its targets from the ranked table. Phase 36 adds an "after" column to the same page.

## Self-Check: PASSED

- FOUND: docs/benchmarks/scene-survey.md
- FOUND: docs/playground-scene-spot-check.md
- FOUND: 7174cd676
- FOUND: 96a3ac6a6 (recorded commit)

---
*Phase: 34-scene-timing-survey*
*Completed: 2026-10-07*

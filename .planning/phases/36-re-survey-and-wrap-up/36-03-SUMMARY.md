---
phase: 36-re-survey-and-wrap-up
plan: "03"
subsystem: planning-records
tags: [wrap-up, future-work, requirements, PERF-10]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-10-11T02-24-50
generated_at: "2026-10-11T03:35:24.745Z"
requires:
  - 36-01 WR-01 fix (c4252c01b)
  - 36-02 published v1.4 before/after survey (e5640ea88)
provides:
  - REQUIREMENTS.md "Left over from v1.4" future-work list (D-09)
  - PERF-10 checked and Complete in traceability
affects:
  - v1.4 milestone archive (separate step after phase verification)
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - .planning/REQUIREMENTS.md
key-decisions:
  - "v1.4 leftovers recorded under REQUIREMENTS.md Future Requirements: presortedness test, A7, world backup clone, emission reallocation, grouped prepare_create load time, wasm32 native-only bins, support.rs:98 release warning, stale web-smoke spec; no Slower scenes and WR-01 already fixed, so neither is listed"
  - "PERF-10 marked Complete only after confirming the docs(36) survey commit e5640ea88 in git log; no v1.4 tag and no milestone archive (D-10)"
requirements: [PERF-10]
duration: 1min
completed: 2026-10-11
---

# Phase 36 Plan 03: Record v1.4 future work and complete PERF-10 Summary

**`.planning/REQUIREMENTS.md` now lists the five Phase 35 leftover ideas and the three pre-existing deferred items under `### Left over from v1.4`. PERF-10 is checked and reads Complete in traceability, backed by survey commit `e5640ea88`.**

## Performance

- **Duration:** about 1 min (03:35:06Z to 03:35:24Z)
- **Tasks:** 1 of 1
- **Files modified:** 1

## Accomplishments

- Preconditions confirmed in `git log`: `fix(36): record scene filter in scene-spot stamps (WR-01)` and `docs(36): publish v1.4 whole-catalog before/after scene survey`.
- Added `### Left over from v1.4` after the two original Future Requirements bullets, which are unchanged. It has eight one-line bullets, each citing its source:
  - from 35-PROFILES §Notes for Phase 36: the presortedness test (A2r particles cost +7.4% / +2.6%, quick 261010-rif), A7, the world backup clone, emission reallocation and the grouped `prepare_create` load time;
  - from Phase 35 deferred-items.md: the wasm32 native-only bins, the release-only warning at `support.rs:98` and the stale `web-smoke` spec.
- The syspolicyd launch-stall item was not copied, because it is fixed. WR-01 is not listed, because 36-01 committed the fix (`grep -c WR-01` prints 0). No scene was listed as Slower, because 36-02 classified none as Slower (24 Faster, 1 Within noise).
- Marked PERF-10 `[x]`, set its traceability row to `| PERF-10 | Phase 36 | Complete |` and updated the last-updated line to 2026-10-11.

## Task Commits

1. **Task 1: Record v1.4 future work and mark PERF-10 complete.** Commit `1cc89564a`, `docs(36): record v1.4 future work and complete PERF-10`. Only `.planning/REQUIREMENTS.md` is in it.

## Checks

- `bun scripts/bright-builds-check.ts all`: findings 0, exit 0.
- The plan's automated verify command printed VERIFY-OK.
- `grep -c "Optional controlled-host and cross-platform campaigns"` prints 1.
- `git tag -l 'v1.4*'` prints nothing.
- I did not run mdformat, because `.planning/**` is GSD-owned.

## Decisions Made

- I gave each bullet a short source citation in parentheses (35-PROFILES, deferred-items.md or the quick task), so that the list can be traced without restating the deferred-items text.

## Deviations from Plan

None. The plan was executed as written.

## Issues Encountered

None.

## Threat Flags

None. T-36-08 was mitigated: PERF-10 was marked Complete only after the survey commit was confirmed. T-36-09 was mitigated: there is no `v1.4` tag and no archive was run.

## Next Steps

- Next comes Phase 36 verification. After that, `/gsd-complete-milestone` archives v1.4 as a separate step.
- A `v1.4` git tag needs the user's explicit go-ahead (D-10). Nothing was tagged or pushed.

## Self-Check: PASSED

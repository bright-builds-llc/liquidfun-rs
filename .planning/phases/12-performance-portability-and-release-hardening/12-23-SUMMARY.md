---
phase: 12-performance-portability-and-release-hardening
plan: "23"
subsystem: ci
tags: [github-actions, performance, provenance, controlled-host, actionlint]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: Plan 12-08 paired performance runner, calibration, validation, and evidence schema
provides:
  - Candidate-bound scheduled and manual performance evidence workflow
  - Controlled-host and shared-runner evidence classification
  - Negative workflow contract tests for identity, ordering, pinning, and artifact cardinality
affects: [release-hardening, performance-evidence, release-aggregation]
tech-stack:
  added: []
  patterns:
    - Detached full-SHA workflow checkout before measurement
    - Identity-last publication before one candidate-scoped artifact upload
key-files:
  created:
    - .github/workflows/performance.yml
    - tools/xtask/tests/performance_workflow.rs
  modified: []
key-decisions:
  - "Acquire paired raw reports before calibration because the Plan 12-08 calibration command consumes that raw set; keep calibration before validation and publication."
  - "Treat scheduled shared-runner measurements only as trend_diagnostic evidence, while manual reviewed runs require the controlled runner label and secret-backed host identity."
patterns-established:
  - "Performance producer order: paired acquisition, calibration, validation, complete-set checks, manifest assembly, producer identity, read-only verification, single upload."
  - "Workflow contract tests reject mutations that weaken candidate identity, host identity, provenance, action pinning, ordering, or upload cardinality."
requirements-completed: [PERF-04, PERF-05, PERF-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T01:51:31Z
duration: 7min
completed: 2026-07-24
---

# Phase 12 Plan 23: Controlled Performance Evidence Workflow Summary

**A full-SHA, controlled-host-aware GitHub Actions producer now emits one validated fourteen-workload performance artifact with candidate, host, workflow, job, run, and payload identity.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-07-24T01:44:31Z
- **Completed:** 2026-07-24T01:51:31Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Added a pinned scheduled/manual workflow that checks out an exact detached candidate and verifies reviewed host identity before manual measurements.
- Kept scheduled shared-host measurements in a closed `trend_diagnostic` class that cannot claim release-reviewed status.
- Enforced complete 32-case, 14-workload, 150-sample evidence before manifest construction, identity-last publication, and one artifact upload.
- Added nine positive and negative contract tests covering candidate and host identity, evidence promotion, validation order, run provenance, action pinning, producer order, and upload cardinality.

## Task Commits

Each task was committed atomically:

1. **Task 1: Produce validated controlled-host performance artifacts** - `3fe338d` (feat)

## Files Created/Modified

- `.github/workflows/performance.yml` - Produces candidate-bound controlled or trend performance evidence and uploads one validated artifact set.
- `tools/xtask/tests/performance_workflow.rs` - Locks the workflow's trigger, identity, ordering, action pin, provenance, and upload contracts.

## Decisions Made

- Paired raw acquisition precedes calibration because calibration reads the raw report set produced by the paired run. Calibration remains mandatory before validation, identity publication, and upload.
- Manual evidence uses the exact `performance-controlled-linux-x64` runner label and a SHA-256 host identity matched against the `PERFORMANCE_CONTROLLED_HOST_IDENTITY` repository secret.
- Scheduled evidence runs on `ubuntu-24.04` and is permanently classified as non-release-reviewed trend evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected the impossible calibration-before-acquisition command order**

- **Found during:** Task 1 (Produce validated controlled-host performance artifacts)
- **Issue:** The plan's literal `calibrate → paired → validate` order conflicted with the Plan 12-08 runner: calibration consumes raw reports that only the paired command creates.
- **Fix:** Used `paired → calibrate → validate`, retaining the governing requirement that calibration completes before comparison validation, identity publication, and upload.
- **Files modified:** `.github/workflows/performance.yml`, `tools/xtask/tests/performance_workflow.rs`
- **Verification:** The positive contract and wrong-order negative test pass; actionlint and the complete ordered Rust gate pass.
- **Committed in:** `3fe338d`

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The correction makes the producer executable without weakening calibration, validation, provenance, or publication controls.

## Issues Encountered

- The initial TDD run failed all nine tests because the workflow did not yet exist, establishing the expected RED state without committing it.

## User Setup Required

- Manual reviewed runs require the repository secret `PERFORMANCE_CONTROLLED_HOST_IDENTITY` to contain the reviewed lowercase 64-character host identity. Scheduled trend runs do not require this secret.

## Known Stubs

None.

## Next Phase Readiness

- Release aggregation can consume one candidate-scoped artifact whose reviewed/trend disposition and complete producer provenance are explicit.
- A real manual reviewed run still depends on the controlled runner label and matching repository secret being configured.

## Self-Check

PASSED - both created implementation files, this summary, and task commit `3fe338d` exist.

***

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-24*

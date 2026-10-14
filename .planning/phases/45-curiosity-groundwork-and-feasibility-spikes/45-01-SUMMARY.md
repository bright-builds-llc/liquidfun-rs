---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "01"
subsystem: scene-fingerprints
tags: [fingerprints, baseline, scene-spot, prep-01]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T15:53:41.986Z"
requires: []
provides:
  - "docs/benchmarks/scene-fingerprints.json: pinned 25-scene id -> fingerprint baseline with provenance and growth rule"
  - "Reusable capture: target/dam-break-perf/2026-10-14T15-53-11Z/scene-spot.json at HEAD 04650ff5a"
affects: [45-02, 45-10]
tech-stack:
  added: []
  patterns:
    - "Baseline derived by jq from one validated scene-spot stamp named by the stderr `wrote` line"
key-files:
  created:
    - docs/benchmarks/scene-fingerprints.json
  modified: []
key-decisions:
  - "Phase 45 baseline pinned (f31ac087a) from scene-spot 2026-10-14T15-53-11Z at 04650ff5a (code identical to b151838be); 25/25 equal to target/phase43/after-final.txt"
  - "provenance.scene_spot_stamp is repo-relative (target/dam-break-perf/<stamp>), not the absolute path from the wrote line"
requirements-completed: []
duration: 2min
completed: 2026-10-14
---

# Phase 45 Plan 01: Pinned Scene Fingerprint Baseline Summary

**25 scene ids and end-state fingerprints pinned in catalog order in `docs/benchmarks/scene-fingerprints.json`, captured at HEAD before any Phase 45 code edit, with host and toolchain provenance and the protected-set growth rule.**

## Performance

- **Duration:** about 2 min
- **Started:** 2026-10-14T15:52:46Z
- **Completed:** 2026-10-14T15:53:47Z
- **Tasks:** 2
- **Files modified:** 1

## Capture Record

| Field | Value |
| --- | --- |
| Capture HEAD | `04650ff5ade210a0cbaf9770f38ef6fbfebad54a` |
| Milestone base | `b151838be180262eba58245b8e5b376b22de6276` (`git diff --stat` over crates/web/tools empty) |
| Stamp | `target/dam-break-perf/2026-10-14T15-53-11Z/scene-spot.json` (from the `wrote` line in `target/phase45/spot-01.err`) |
| Host | macos, aarch64, Apple M4 Max |
| Compiler | rustc 1.97.0 (2d8144b78 2026-07-07), release |
| Steps | 60 warmup, 120 measured, 3 runs (determinism check passed) |
| Baseline commit | `f31ac087a` |
| Cross-check | `diff target/phase45/base.txt target/phase43/after-final.txt` empty: 25/25 identical |
| Catalog order | `scenes[]` order equals `SCENE_IDS` in `web/src/catalog/scenes.ts` (wave-machine first, tesla-valve last) |

Plans 02 and 10 can reuse this stamp. Its `git_head` is 04650ff5a, so a reuse after later commits must be judged against code changes, not that hash.

## Task Commits

1. **Task 1: Preflight and capture the 25 fingerprints at HEAD** - no commit (gitignored logs in `target/phase45/` only: capture-head.txt, base-diff.txt, spot-01.{out,err,path}, spot-01-order.txt, catalog-order.txt, base.txt)
1. **Task 2: Derive and commit docs/benchmarks/scene-fingerprints.json** - `f31ac087a` (docs)

## Files Created/Modified

- `docs/benchmarks/scene-fingerprints.json` - kind `scene_fingerprint_baseline`, schema 1, growth_rule, provenance, 25 `{id, fingerprint}` records in catalog order

## Decisions Made

- `provenance.scene_spot_stamp` holds the repo-relative path `target/dam-break-perf/2026-10-14T15-53-11Z`. The `wrote` line prints an absolute path, and committing the plan's literal `dirname` would have put a home-directory path into the repository.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Stamp path made repo-relative**
- **Found during:** Task 2
- **Issue:** `scene-spot` prints `wrote /Users/.../target/dam-break-perf/<stamp>/scene-spot.json`, so `dirname` gives an absolute, host-specific path.
- **Fix:** Removed the `$PWD/` prefix before passing `--arg stamp`. `captured_at` (last path component) is unchanged.
- **Files modified:** docs/benchmarks/scene-fingerprints.json
- **Commit:** f31ac087a

Also, the orchestrator asked for `generated_by: gsd-execute-phase`, while the plan's output section said `gsd-executor`. I followed the orchestrator's value.

## Issues Encountered

None. Every preflight, jq validation, order and cross-check passed on the first capture.

## Next Phase Readiness

- The baseline exists for the Plan 02 gate (`just scene-fingerprint-gate`, still to be built) and the Plan 10 final-tree run.
- The baseline's `growth_rule` points to `docs/adding-a-scene.md`, which a later Phase 45 plan writes.

## Self-Check: PASSED

- FOUND: docs/benchmarks/scene-fingerprints.json
- FOUND: target/dam-break-perf/2026-10-14T15-53-11Z/scene-spot.json
- FOUND: commit f31ac087a

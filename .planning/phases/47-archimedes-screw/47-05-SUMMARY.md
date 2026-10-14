---
phase: 47-archimedes-screw
plan: "05"
subsystem: scene-fingerprint-baseline
tags: [scene, archimedes-screw, fingerprint-gate, baseline-growth]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: "2026-10-14T22:45:30Z"
requires:
  - "47-04: full-catalog capture target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json at 23541975c, gate PASS 26/26 with archimedes-screw added"
provides:
  - "docs/benchmarks/scene-fingerprints.json protects 27 ids, archimedes-screw last (86a28d366447bbeb)"
  - "Gate evidence target/phase47/fingerprint-gate-27.log: PASS 27/27 unchanged, no additions"
affects: [48]
tech-stack:
  added: []
  patterns:
    - "Reuse a saved full-catalog capture for baseline growth only when `git diff --quiet <capture> HEAD -- crates tools web` holds"
key-files:
  created: []
  modified:
    - docs/benchmarks/scene-fingerprints.json
key-decisions:
  - "47-05: baseline grown in single-file commit 4be8ffdab from the reused 47-04 capture 2026-10-14T22-42-42Z at 23541975c (crates/tools/web identical to HEAD c9d58840d); host fields unchanged"
  - "47-05: gate rerun against the same capture reports PASS 27/27 unchanged, added none; Phase 48 protects 27 ids including archimedes-screw"
requirements-completed: []
duration: about 2 min
completed: 2026-10-14
---

# Phase 47 Plan 05: Protect the Archimedes Screw Fingerprint Summary

**`docs/benchmarks/scene-fingerprints.json` now protects 27 scene ids. `archimedes-screw` (`86a28d366447bbeb`) was appended last in its own single-file commit, 4be8ffdab. Rerunning the id-join gate against the same capture reports PASS: 27/27 protected ids unchanged and no additions. Phase 48 therefore starts with the screw protected.**

## Performance

- **Duration:** about 2 min
- **Started:** 2026-10-14T22:44:31Z
- **Completed:** 2026-10-14T22:45Z
- **Tasks:** 2 of 2
- **Files:** 1 modified

## Accomplishments

### Task 1 (4be8ffdab): baseline growth in its own commit

- **Capture choice.** `git diff --quiet 23541975cd276880934f0044c732915d8c88d287 HEAD -- crates tools web` exited 0 at HEAD c9d58840d. Only `.planning/**` changed after the capture commit, so I reused the 47-04 full-catalog capture `target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json`. It has 27 scenes, `full_catalog` true and `git_head` 23541975c.
- **Pre-check.** `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json` reported PASS, `protected: 26/26 unchanged` and `added (accepted): archimedes-screw 86a28d366447bbeb` (log `target/phase47/fingerprint-gate-05-pre.log`).
- **Screw fingerprint.** The plan's `jq` command returned `86a28d366447bbeb`, which matches `^[0-9a-f]{16}$`.
- **Edit.**
  - Appended `{ "id": "archimedes-screw", "fingerprint": "86a28d366447bbeb" }` after `cartesian-diver`.
  - Set the provenance fields to the reused capture: `git_commit` `23541975cd276880934f0044c732915d8c88d287`, `scene_spot_stamp` `target/dam-break-perf/2026-10-14T22-42-42Z` and `captured_at` `2026-10-14T22-42-42Z`.
  - Left the remaining provenance fields unchanged, because the capture's `os`, `arch`, `cpu_brand`, `compiler`, warmup steps, measured steps and runs all match the existing values.
  - Kept 2-space formatting and the trailing newline; `jq --indent 2` output is byte-identical to the file.
- **Validation.**
  - `jq -e '.scenes | length == 27 and .[-1].id == "archimedes-screw" ...'` returned true.
  - The first 26 entries match those in `HEAD~1` with no diff.
  - `git show --name-only --format= HEAD` prints only `docs/benchmarks/scene-fingerprints.json`.

### Task 2: gate rerun, 27/27 unchanged (no tracked changes, no commit)

I ran `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json 2>&1 | tee target/phase47/fingerprint-gate-27.log`. A separate run without `tee` confirmed exit 0.

```
Scene fingerprint gate: PASS
- protected: 27/27 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 23541975cd276880934f0044c732915d8c88d287, 2026-10-14T22-42-42Z)
- observed: target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json (git 23541975cd276880934f0044c732915d8c88d287, full_catalog true)
```

- `git diff --quiet HEAD -- . ':(exclude).planning'` passes.
- `git log --oneline -3` shows the baseline commit 4be8ffdab as the most recent commit, ahead of this SUMMARY's separate docs commit.
- Phase 48 now protects 27 ids, including `archimedes-screw`.

## Decisions Made

- I reused the 47-04 capture instead of taking a fresh one. The plan allows this when the code trees are identical, and the provenance names that capture's commit and stamp.

## Deviations from Plan

None. The plan was executed as written.

## Known Stubs

None.

## Next Phase Readiness

Phase 48 runs the id-join gate against a baseline of 27 protected ids. Every protected id, `archimedes-screw` included, must stay unchanged, and any new scene id should appear under "added (accepted)".

## Self-Check: PASSED

- FOUND: commit 4be8ffdab, which changes only `docs/benchmarks/scene-fingerprints.json`
- FOUND: target/phase47/fingerprint-gate-27.log with PASS, 27/27 unchanged and added none
- FOUND: target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json

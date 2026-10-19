---
phase: 49-rayleigh-taylor-instability
plan: "05"
subsystem: scene-fingerprint-baseline
tags: [fingerprint-gate, baseline-growth, rayleigh-taylor, prep-01]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: "2026-10-19T03:15:44Z"
requires:
  - "49-04: full-catalog capture target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json at e495c30f5, gate PASS 28/28 with rayleigh-taylor ac59254e2231967c added"
provides:
  - "docs/benchmarks/scene-fingerprints.json protects 29 ids, rayleigh-taylor ac59254e2231967c last"
  - "Gate rerun PASS 29/29 unchanged, added none"
affects: [50]
tech-stack:
  added: []
  patterns:
    - "Reused capture only after git diff --quiet <capture-commit> HEAD -- crates tools web"
key-files:
  created: []
  modified:
    - docs/benchmarks/scene-fingerprints.json
key-decisions:
  - "49-05: baseline grown in single-file commit 9e7e61cfc from reused 49-04 capture 2026-10-19T03-11-46Z at e495c30f5 (crates/tools/web identical to HEAD); gate rerun PASS 29/29 unchanged, added none; later work protects rayleigh-taylor"
requirements-completed: []
requirements-advanced: [RT-02]
duration: 1min
completed: 2026-10-19
---

# Phase 49 Plan 05: Protect the Rayleigh–Taylor Fingerprint Summary

**The protected baseline now has 29 ids, with `rayleigh-taylor` `ac59254e2231967c` added last. Commit `9e7e61cfc` changes only that file. Rerunning the gate against the same capture reports 29/29 unchanged and no additions.**

## Performance

- **Duration:** about 1 min
- **Started:** 2026-10-19T03:15:01Z
- **Completed:** 2026-10-19T03:15:44Z
- **Tasks:** 2 (Task 1 has a commit; Task 2 changed no tracked file)
- **Files modified:** 1

## Task 1: Grow the baseline (D-20)

- **Capture reuse:** `git diff --stat e495c30f5 HEAD -- crates tools web` was empty and `git diff --quiet e495c30f55f5e9bd667a5aeb6cef561c69e5399e HEAD -- crates tools web` exited 0, so I reused the 49-04 full-catalog capture `target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json` instead of taking a new one. The capture reports `git_head` e495c30f5, `full_catalog true`, macos/aarch64, Apple M4 Max and rustc 1.97.0 (2d8144b78 2026-07-07). Those match the existing provenance, so `os`, `arch`, `cpu_brand` and `compiler` did not change.
- **Pre-check:** the gate reported `protected: 28/28 unchanged` and `added (accepted): rayleigh-taylor ac59254e2231967c`.
- **Fingerprint:** the jq read returns `ac59254e2231967c`, which matches `^[0-9a-f]{16}$`. It is also the last scene in the capture's catalog order.
- **Edit:** I appended `{ "id": "rayleigh-taylor", "fingerprint": "ac59254e2231967c" }` and updated `provenance.git_commit` to `e495c30f55f5e9bd667a5aeb6cef561c69e5399e`, `scene_spot_stamp` to `target/dam-break-perf/2026-10-19T03-11-46Z` and `captured_at` to `2026-10-19T03-11-46Z`. The file keeps 2-space indentation and its trailing newline. No tool reformatted it, as in 48-05.
- **Checks:** `.scenes | length == 29` and the last id is `rayleigh-taylor`. The first 28 entries are byte-identical to the previous commit's `scenes` (`diff` is empty), and `fake-ferrofluid` is still `9f1e59fce8af89e7`. All other top-level and provenance fields are unchanged. `cargo test -p xtask --bin xtask fingerprint` passes (34 tests), since the gate's code is the only reader of the file.
- **Commit:** `9e7e61cfc`. `git show --name-only --format= HEAD` prints only `docs/benchmarks/scene-fingerprints.json` (1 file changed, 7 insertions, 3 deletions).

## Task 2: Gate rerun (D-20)

`just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json`, logged to `target/phase49/fingerprint-gate-29.log`:

```
Scene fingerprint gate: PASS
- protected: 29/29 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at e495c30f55f5e9bd667a5aeb6cef561c69e5399e, 2026-10-19T03-11-46Z)
- observed: target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json (git e495c30f55f5e9bd667a5aeb6cef561c69e5399e, full_catalog true)
```

`git diff --quiet HEAD -- . ':(exclude).planning'` passes, so no tracked code or docs changes are left. `git log --oneline -3` shows `9e7e61cfc` as the most recent commit. Later work now protects 29 ids, including `rayleigh-taylor`.

## Task Commits

1. **Task 1: baseline growth:** `9e7e61cfc` (chore)
1. **Task 2: gate rerun:** no commit. Evidence only, in gitignored `target/phase49/fingerprint-gate-29.log`.

## Deviations from Plan

None. The plan ran as written.

## Issues Encountered

None.

## Known Stubs

None.

## Next Phase Readiness

Phase 49 is ready for verification. The next scene phase starts with 29 protected ids. Its gate must report 29/29 unchanged before it grows the baseline to 30 in its own commit.

## Self-Check: PASSED

- FOUND: docs/benchmarks/scene-fingerprints.json (29 scenes)
- FOUND: commit 9e7e61cfc
- FOUND: target/phase49/fingerprint-gate-29.log (PASS, 29/29, added none)

---
phase: 48-fake-ferrofluid
plan: "05"
subsystem: scene-fingerprint-baseline
tags: [fingerprint-gate, baseline-growth, fake-ferrofluid, prep-01]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: "2026-10-15T07:09:47.000Z"
requires:
  - "48-04: full-catalog capture target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json at f798bff91, gate PASS 27/27 with fake-ferrofluid e73ef71f0615b4eb added"
provides:
  - "docs/benchmarks/scene-fingerprints.json protects 28 ids, fake-ferrofluid e73ef71f0615b4eb last"
  - "Gate rerun PASS 28/28 unchanged, added none"
affects: [49]
tech-stack:
  added: []
  patterns:
    - "Reused capture only after git diff --quiet <capture-commit> HEAD -- crates tools web"
key-files:
  created: []
  modified:
    - docs/benchmarks/scene-fingerprints.json
key-decisions:
  - "48-05: baseline grown in single-file commit 2db76fda3 from reused 48-04 capture 2026-10-15T07-06-56Z at f798bff91 (crates/tools/web identical to HEAD); gate rerun PASS 28/28 unchanged, added none; Phase 49 protects fake-ferrofluid"
requirements-completed: []
requirements-advanced: [FERRO-01]
duration: 2min
completed: 2026-10-15
---

# Phase 48 Plan 05: Protect the Fake Ferrofluid Fingerprint Summary

**The protected baseline now has 28 ids, with `fake-ferrofluid` `e73ef71f0615b4eb` added last. Commit `2db76fda3` changes only that file. Rerunning the gate against the same capture reports 28/28 unchanged and no additions.**

## Performance

- **Duration:** about 2 min
- **Started:** 2026-10-15T07:09:27Z
- **Completed:** 2026-10-15T07:09:47Z
- **Tasks:** 2 (Task 1 has a commit; Task 2 changed no tracked file)
- **Files modified:** 1

## Task 1: Grow the baseline (D-25)

- **Capture reuse:** `git diff --quiet f798bff915e346a0b901454bf15eeb590d03fd77 HEAD -- crates tools web` exited 0, so I reused the 48-04 full-catalog capture `target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json` instead of taking a new one. The capture reports `git_head` f798bff91, `full_catalog true`, macos/aarch64, Apple M4 Max and rustc 1.97.0. Those match the existing provenance, so `os`, `arch`, `cpu_brand` and `compiler` did not change.
- **Pre-check:** the gate reported `protected: 27/27 unchanged` and `added (accepted): fake-ferrofluid e73ef71f0615b4eb`.
- **Fingerprint:** the jq read returns `e73ef71f0615b4eb`, which matches `^[0-9a-f]{16}$`.
- **Edit:** I appended `{ "id": "fake-ferrofluid", "fingerprint": "e73ef71f0615b4eb" }` and updated `provenance.git_commit` to `f798bff915e346a0b901454bf15eeb590d03fd77`, `scene_spot_stamp` to `target/dam-break-perf/2026-10-15T07-06-56Z` and `captured_at` to `2026-10-15T07-06-56Z`. The file keeps 2-space indentation and its trailing newline.
- **Checks:** `.scenes | length == 28` and the last id is `fake-ferrofluid`. The first 27 entries are byte-identical to the previous commit's `scenes` (`diff` is empty), and `archimedes-screw` is still `86a28d366447bbeb`.
- **Commit:** `2db76fda3`. `git show --name-only --format= HEAD` prints only `docs/benchmarks/scene-fingerprints.json`.

## Task 2: Gate rerun (D-25)

`just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json`, logged to `target/phase48/fingerprint-gate-28.log`:

```
Scene fingerprint gate: PASS
- protected: 28/28 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at f798bff915e346a0b901454bf15eeb590d03fd77, 2026-10-15T07-06-56Z)
- observed: target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json (git f798bff915e346a0b901454bf15eeb590d03fd77, full_catalog true)
```

`git diff --quiet HEAD -- . ':(exclude).planning'` passes, so no tracked code or docs changes are left. `git log --oneline -3` shows `2db76fda3` as the most recent commit. Phase 49 now protects 28 ids, including `fake-ferrofluid`.

## Task Commits

1. **Task 1: baseline growth:** `2db76fda3` (chore)
1. **Task 2: gate rerun:** no commit. Evidence only, in gitignored `target/phase48/fingerprint-gate-28.log`.

## Deviations from Plan

None. The plan ran as written.

## Issues Encountered

None.

## Known Stubs

None.

## Next Phase Readiness

Phase 49 (Rayleigh–Taylor) starts with 28 protected ids. Its gate must report 28/28 unchanged before it grows the baseline to 29 in its own commit.

## Self-Check: PASSED

- FOUND: docs/benchmarks/scene-fingerprints.json (28 scenes)
- FOUND: commit 2db76fda3
- FOUND: target/phase48/fingerprint-gate-28.log (PASS, 28/28, added none)

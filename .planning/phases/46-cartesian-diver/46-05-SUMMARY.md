---
phase: 46-cartesian-diver
plan: "05"
subsystem: scene-fingerprint-baseline
tags: [scene, cartesian-diver, fingerprint-gate, baseline-growth]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: "2026-10-14T19:33:00Z"
requires:
  - "46-04: full-catalog capture target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json at 89983a79b, gate PASS 25/25 with cartesian-diver added"
provides:
  - "docs/benchmarks/scene-fingerprints.json protects 26 ids, cartesian-diver last (fa849f40b8c2a791)"
  - "Gate evidence target/phase46/fingerprint-gate-26.log: PASS 26/26 unchanged, no additions"
affects: [47]
tech-stack:
  added: []
  patterns:
    - "Reuse a saved full-catalog capture for baseline growth only when `git diff --quiet <capture> HEAD -- crates tools web` holds"
key-files:
  created: []
  modified:
    - docs/benchmarks/scene-fingerprints.json
key-decisions:
  - "46-05: baseline grown in single-file commit 18c8a5e92 from the reused 46-04 capture 2026-10-14T19-29-46Z at 89983a79b (crates/tools/web identical to HEAD 730d789eb); host fields unchanged"
  - "46-05: gate rerun against the same capture reports PASS 26/26 unchanged, added none; Phase 47 protects 26 ids including cartesian-diver"
requirements-completed: [SCENE-01]
duration: about 3 min
completed: 2026-10-14
---

# Phase 46 Plan 05: Protect the Cartesian Diver Fingerprint Summary

**`docs/benchmarks/scene-fingerprints.json` now protects 26 scene ids, with `cartesian-diver` (`fa849f40b8c2a791`) appended last in its own single-file commit (18c8a5e92). Rerunning the id-join gate against the same capture reports PASS with 26/26 protected ids unchanged and no additions, so Phase 47 starts with the diver protected.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-14T19:31:49Z
- **Completed:** 2026-10-14T19:33Z
- **Tasks:** 2 of 2
- **Files:** 1 modified

## Accomplishments

### Task 1 (18c8a5e92): baseline growth in its own commit

- **Capture choice.** The plan allows reuse when the capture commit's `crates`, `tools` and `web` trees match HEAD. `git diff --quiet 89983a79b546153cb61b8bbd5a5dadafe024fec1 HEAD -- crates tools web` exited 0 at HEAD 730d789eb, and only `.planning/**` changed after the capture. So the 46-04 full-catalog capture `target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json` was reused. It has 26 scenes, `full_catalog` true and `git_head` 89983a79b.
- **Pre-check.** Running `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json` reported PASS, `protected: 25/25 unchanged` and `added (accepted): cartesian-diver fa849f40b8c2a791`. The log is `target/phase46/fingerprint-gate-05-pre.log`.
- **Diver fingerprint.** The plan's `jq` command read `fa849f40b8c2a791`, which matches `^[0-9a-f]{16}$`.
- **Edit.** The plan appended `{ "id": "cartesian-diver", "fingerprint": "fa849f40b8c2a791" }` after `tesla-valve`. It also set provenance `git_commit` to `89983a79b546153cb61b8bbd5a5dadafe024fec1`, `scene_spot_stamp` to `target/dam-break-perf/2026-10-14T19-29-46Z` and `captured_at` to `2026-10-14T19-29-46Z`. The capture's `os`, `arch`, `cpu_brand`, `compiler`, warmup, measured steps and runs equal the existing provenance, so those fields were left alone. The 2-space formatting and trailing newline are kept.
- **Validation.**
  - `jq -e '.scenes | length == 26 and .[-1].id == "cartesian-diver" ...'` returned true.
  - The first 25 entries diff clean against `HEAD~1`.
  - `git show --name-only --format= HEAD` prints only `docs/benchmarks/scene-fingerprints.json`.

### Task 2: gate rerun, 26/26 unchanged (no tracked changes, no commit)

Command: `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json 2>&1 | tee target/phase46/fingerprint-gate-26.log` exited 0.

```
Scene fingerprint gate: PASS
- protected: 26/26 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 89983a79b546153cb61b8bbd5a5dadafe024fec1, 2026-10-14T19-29-46Z)
- observed: target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json (git 89983a79b546153cb61b8bbd5a5dadafe024fec1, full_catalog true)
```

- `git diff --quiet HEAD -- . ':(exclude).planning'` passes.
- `git log --oneline -3` shows the baseline commit 18c8a5e92 as the most recent commit, before this SUMMARY's separate docs commit.
- Phase 47 now protects 26 ids, including `cartesian-diver`.

## Decisions Made

- The 46-04 capture was reused instead of taking a fresh one, as the plan allows when the code trees are identical. The provenance names that capture's commit and stamp.

## Deviations from Plan

None. The plan was executed as written.

## Known Stubs

None.

## Next Phase Readiness

Phase 47 (Archimedes screw) runs the id-join gate against a baseline of 26 protected ids. Every protected id, `cartesian-diver` included, must stay unchanged, and the new screw id should appear under "added (accepted)".

## Self-Check: PASSED

- FOUND: commit 18c8a5e92, which changes only `docs/benchmarks/scene-fingerprints.json`
- FOUND: target/phase46/fingerprint-gate-26.log with PASS, 26/26 unchanged and added none
- FOUND: target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json

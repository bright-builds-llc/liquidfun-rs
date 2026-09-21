---
phase: 23-baseline-pair-and-named-audit
plan: "08"
subsystem: observability-evidence
tags: [dam-break, named-audit, samply, dhat, unreviewed-sample]

requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: live unprofiled Dam Break pair stamp at MEASURED_HEAD
  - phase: 23-baseline-pair-and-named-audit
    provides: live samply rust.json.gz plus rust.json.syms.json at the same HEAD
  - phase: 23-baseline-pair-and-named-audit
    provides: copy-only audit-bundle stamp naming both source stamps
provides:
  - committed docs/native-performance-audit.md with live named functions
  - private dhat dump under gitignored heap stamp after allocator/Vec needles
  - refreshed unreviewed SHA-bound Dam Break timing sample
affects:
  - 23-09 independent review
  - Phase 24 shared hot-path edits from named shares

tech-stack:
  added: []
  patterns:
    - sidecar-first names plus gecko sample weights for approximate shares
    - pair.json rust_over_cpp_ratio is the only Dam Break number
    - dhat dump is not_timing_authority and stays under target/

key-files:
  created:
    - docs/native-performance-audit.md
  modified:
    - docs/playground-dam-break-timing.md

key-decisions:
  - "Rank dominating frames from live rust.json.syms.json plus gecko stackTable weights; do not paste the hunt list as causes."
  - "particle_rows is ~93.6% self (ParticleId.position scans); classify as algorithm/shape versus FindContacts indexA/indexB."
  - "Copy rust_over_cpp_ratio 327.53395024734476 only from bundle pair.json at MEASURED_HEAD 6d98531ac799987c209d3fd1e572e482fcab5da6."
  - "Run private dhat after allocator/Vec needles; cite heap stamp 2026-09-21T04-47-09Z; never git add dhat-heap.json."
  - "Independent AI review remains 23-09; this plan does not self-approve."

patterns-established:
  - "Approximate shares are leaf-sample fractions mapped through the live sidecar, not samply milliseconds."
  - "Heap --stamp is required after a docs commit moves HEAD off the capture SHA."
  - "Never git add *.json.gz, *.syms.json, *.trace, dhat-heap.json, or target/."

requirements-completed: [PERF-AUDIT, PERF-HEAP]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:49:44Z

duration: 9min
completed: 2026-09-21
---

# Phase 23 Plan 08: Named Function Audit Summary

**SHA-bound named-function audit from live sidecar ranking: `particle_rows` ~94% self (ParticleId scans vs C++ indices), private dhat dump after allocator needles, refreshed unreviewed timing sample.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-09-21T04:41:06Z
- **Completed:** 2026-09-21T04:49:44Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Wrote `docs/native-performance-audit.md` from durable bundle `2026-09-21T04-34-32Z` at MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`.
- Ranked sidecar-resolved `dam-break-bench` leaf samples (72817); classified each listed frame into the four cause buckets; explicit **Not found** (SIMD/Rayon/PGO/unsafe indexing/WASM-vs-C++/Phase 12 matrix).
- Ran gated `dhat-heap` after needle match; dump at `target/dam-break-perf/2026-09-21T04-47-09Z/dhat-heap.json` (`not_timing_authority`).
- Refreshed `docs/playground-dam-break-timing.md` as the current SHA-bound unreviewed sample, with a historical pointer to `1e5cbcc124becd363049c4d62c60b715bc0d7897`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Write docs/native-performance-audit.md from the live bundle** - `8d9c6f2` (docs)
1. **Task 2: Run or skip heap, then refresh the timing doc** - `0c78589` (docs)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Capture identity used

- **MEASURED_HEAD:** `6d98531ac799987c209d3fd1e572e482fcab5da6`
- **PAIR_STAMP:** `2026-09-21T04-32-19Z`
- **PROFILE_STAMP:** `2026-09-21T04-32-56Z`
- **BUNDLE_STAMP:** `2026-09-21T04-34-32Z`
- **HEAP_STAMP:** `2026-09-21T04-47-09Z`
- **DURABLE_EVIDENCE:** primary-tree `target/dam-break-perf/` (same-tree; no hydrate copy)
- **rust.wall_ms:** `71001.949958`
- **cpp.wall_ms:** `216.777375`
- **rust_over_cpp_ratio:** `327.53395024734476` (only 3× number)
- **Sidecar:** `rust.json.syms.json`

## Named ranking (committed)

Approximate leaf shares of 72817 `dam-break-bench` samples:

1. `liquidfun::particle::contact::particle_rows` ~93.6% self — algorithm/shape vs `FindContacts_Reference` `Proxy.index`
1. `ParticleGroupId` `SliceContains` under `check_invariants` ~2.2% self — checks that survive `--release`
1. `replace_solver_candidate` ~1.6% inclusive — checks that survive `--release`
1. `ParticleNeighborhood::from_view` ~0.4% self — algorithm/shape vs `UpdateProxies`/`SortProxies`
1. `RawVecInner::finish_grow` / `Vec<ParticleContact>` collect — per-step allocation
1. `recompute_weights`, contact/body `listener_effects`, `pressure::damping` — extra per-particle work

**Not ranked / Not found:** `backup_step_limit_state` (~0.02% inclusive); `velocities().to_vec` as a named leaf; SIMD-first; default Rayon; PGO; unsafe indexing; WASM-vs-C++; Phase 12 sealed matrix.

## Heap

Needles matched (`alloc::`, `__rdl_alloc`, `alloc::vec::Vec`, `RawVec`, `to_vec`, `GlobalAlloc`, `core::alloc::`, `core::clone::`). dhat ran once at 60+600. Dump stays gitignored. Heap identity `git_head` is `8d9c6f2` (Task 1 docs commit). No repo-root `dhat-heap.json`.

## Files Created/Modified

- `docs/native-performance-audit.md` — named functions, cause classes, pair.json ratio, Not found, heap stamp
- `docs/playground-dam-break-timing.md` — current SHA-bound unreviewed sample
- Gitignored heap stamp under `target/dam-break-perf/2026-09-21T04-47-09Z/` (not committed)

## Decisions Made

- Rank from live sidecar + gecko sample weights; hunt-list items without share go to Not found / Not ranked.
- Dominating extra work is `particle_rows` ID-to-row scans, not SIMD.
- Ratio digits come only from bundle `pair.json`.
- Run dhat after needles; pass `--stamp 2026-09-21T04-34-32Z` after HEAD drifted.
- Do not self-approve; 23-09 is independent review.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Targeted the 23-07 bundle with `--stamp`**
- **Found during:** Task 2 (default heap stamp pick requires current HEAD)
- **Issue:** Task 1 docs commit moved HEAD to `8d9c6f2`, while pair/profile/bundle `git_head` remains MEASURED_HEAD `6d98531`. `just playground-dam-break-heap` would fail closed with no matching stamp.
- **Fix:** Ran `cargo xtask playground dam-break-heap --stamp 2026-09-21T04-34-32Z` once (defaults 60/600). Recorded that argv in the audit.
- **Files modified:** gitignored `target/dam-break-perf/2026-09-21T04-47-09Z/`; `docs/native-performance-audit.md`
- **Verification:** nonempty `dhat-heap.json`; `heap-identity.json` `matched_needles` populated; no repo-root dump
- **Committed in:** `0c78589`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to run dhat against the live 23-07 profile after the Task 1 docs SHA moved. No physics edits, no recipe shrink, no `git add` of blobs.

## Issues Encountered

None beyond the expected HEAD drift after the Task 1 docs commit.

## Authentication Gates

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-09 independent AI review of the complete diff and evidence. The implementing agent must not approve this work.
- Phase 24 should start from `particle_rows` (and the smaller ranked frames), not SIMD/Rayon/`unsafe` indexing.
- Do not quote samply or dhat duration as the 3× number.
- Do not `git add` `*.json.gz` or `dhat-heap.json`.

## Self-Check: PASSED

- FOUND: `docs/native-performance-audit.md`
- FOUND: `docs/playground-dam-break-timing.md`
- FOUND: `.planning/phases/23-baseline-pair-and-named-audit/23-08-SUMMARY.md`
- FOUND: `8d9c6f2` Task 1, `0c78589` Task 2
- FOUND: `git ls-files '*.json.gz'` empty

---
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*

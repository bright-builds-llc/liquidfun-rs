---
phase: 24-shared-hot-path-waves-through-3
plan: "03"
subsystem: performance-admission
tags: [leftover, check_invariants, slice_contains, PERF-ADMIT, PERF-SHARED]

requires:
  - phase: 24-shared-hot-path-waves-through-3
    provides: exclusive unprofiled pair stamp 2026-09-21T15-50-46Z with rust_over_cpp_ratio 21.00845179052245
  - phase: 24-shared-hot-path-waves-through-3
    provides: sibling samply stamp 2026-09-21T15-52-27Z not_timing_authority leftover retarget
provides:
  - first leftover ParticleStorage::check_invariants / slice_contains gated on replace_solver_candidate only
  - exclusive unprofiled pair stamp 2026-09-21T16-07-02Z with rust_over_cpp_ratio 15.08956518243927
  - sibling samply stamp 2026-09-21T16-07-39Z not_timing_authority leftover retarget
affects:
  - 24-04 leftover ranking from post-check_invariants samply
  - PERF-GATE leftover scalar waves until pair.json <= 3

tech-stack:
  added: []
  patterns:
    - admit one D-07 leftover from live sidecar leaf shares, never Phase 23 hunt-list paste
    - gate release check_invariants only on the solver-candidate path; keep create/mutate fail-closed
    - mint a new exclusive pair stamp; samply sibling stays not_timing_authority

key-files:
  created: []
  modified:
    - crates/liquidfun/src/particle/storage/runtime.rs
    - docs/playground-dam-break-timing.md
    - docs/native-performance-audit.md

key-decisions:
  - "Admit leftover check_invariants / slice_contains from 24-02 sidecar 2026-09-21T15-52-27Z (~32.85% self, first D-07 named share)."
  - "Gate candidate.check_invariants only on replace_solver_candidate behind debug_assertions; creation.rs and mutation.rs stay fail-closed."
  - "Admit from exclusive unprofiled pair 2026-09-21T16-07-02Z with rust_over_cpp_ratio 15.08956518243927, strictly below Wave 1 21.00845179052245. Do not claim PERF-GATE."
  - "Sibling samply stamp 2026-09-21T16-07-39Z is leftover-ranking input only for 24-04."

patterns-established:
  - "Walk D-07 on the live leftover sidecar; do not paste the Phase 23 named table as current causes."
  - "Keep Unreviewed local sample banners; leave reviewed_reports empty."

requirements-completed: [PERF-ADMIT, PERF-SHARED]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T16:10:05Z

duration: 11min
completed: 2026-09-21
---

# Phase 24 Plan 03: First Leftover Admit-or-Skip Summary

**First leftover `check_invariants` / `slice_contains` admitted on the solver-candidate path: exclusive unprofiled Dam Break Medium pair stamp `2026-09-21T16-07-02Z` has `rust_over_cpp_ratio` `15.08956518243927`, down from Wave 1 `21.00845179052245`; PERF-GATE is not claimed.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-09-21T15:58:18Z
- **Completed:** 2026-09-21T16:10:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Ranked leftover frames from live sidecar `target/dam-break-perf/2026-09-21T15-52-27Z/rust.json.syms.json` plus gecko `stackTable` leaf weights on the `dam-break-bench` thread (Phase 23 outer-symbol method, 4630 samples). Did not paste the Phase 23 named table as current causes.
- Wave 1 pair ratio `21.00845179052245` was still greater than 3, so D-06 skip did not apply.
- Admitted exactly one leftover: `ParticleStorage::check_invariants` / `ParticleGroupId` `slice_contains` (~32.85% self). Gated `candidate.check_invariants()` behind `#[cfg(debug_assertions)]` in `replace_solver_candidate` only.
- User-facing `check_invariants()?` on create/mutate stays fail-closed. Shared `liquidfun` only; public `ParticleId` unchanged; no SIMD / `unsafe` / `HashMap` identity cache.
- Re-ran locked `just playground-dam-break-bench` into a **new** exclusive stamp. Ratio `15.08956518243927` is finite and strictly below `21.00845179052245`. Prior stamps remain on disk.
- Captured sibling samply stamp `2026-09-21T16-07-39Z` (`kind samply_cpu`, `not_timing_authority: true`, no `pair.json`) for 24-04 leftover ranking.
- Refreshed unreviewed timing and audit notes from the new `pair.json` only. Did not claim PERF-GATE (ratio still > 3). `reference/performance/manifest.toml` still has `reviewed_reports = []`.

## Ranked leftover table (24-02 sidecar)

Approximate outer-leaf shares of 4630 `dam-break-bench` samples from stamp `2026-09-21T15-52-27Z`:

| Symbol (D-07 order) | Approximate leaf share | Decision |
| --- | --- | --- |
| `check_invariants` / `slice_contains` | ~32.85% self (~22.68% under `replace_solver_candidate`, ~10.60% under `prepare_permutation`) | **admitted** (first D-07) |
| `replace_solver_candidate` | ~0.15% self / ~23.95% inclusive | not this wave |
| `ParticleNeighborhood::from_view` | ~7.60% self | not this wave |
| `RawVecInner::finish_grow` / `Vec<ParticleContact>` collect | ~6.61% / ~3.80% self | not this wave |
| `listener_effects` (contact + body-contact) | ~14.28% self | not this wave |

Chosen concern: `ParticleStorage::check_invariants` / `slice_contains`. Files actually edited: `crates/liquidfun/src/particle/storage/runtime.rs` only. Skip-because-≤3: no.

## Task Commits

Each task was committed atomically:

1. **Task 1: Rank 24-02 samply and admit-or-skip one leftover** - `907bdf3` (perf)
2. **Task 2: Exclusive unprofiled re-pair after admit-or-skip** - `f517e0d` (docs)

**Plan metadata:** pending `docs(24-03): complete first leftover admit-or-skip plan`

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/runtime.rs` — `#[cfg(debug_assertions)]` on solver-candidate `check_invariants`
- `docs/playground-dam-break-timing.md` — current unreviewed sample from stamp `2026-09-21T16-07-02Z`; historical MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6` / `327.53395024734476` and Wave 1 `21.00845179052245`
- `docs/native-performance-audit.md` — first leftover admission subsection; leftover ranking table from `2026-09-21T15-52-27Z`; **PERF-GATE is not claimed**

Gitignored evidence (not committed):

- Pair: `target/dam-break-perf/2026-09-21T16-07-02Z/pair.json`
- Profile: `target/dam-break-perf/2026-09-21T16-07-39Z/profile-identity.json` and `rust.json.gz`

## Recorded pair (3× authority)

Copied from `target/dam-break-perf/2026-09-21T16-07-02Z/pair.json` only.

- git HEAD: `907bdf360c423184df1dc9210cce60a65a073c8f`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- particles: `1920`
- warmup_steps: `60`
- measured_steps: `600`
- native Rust wall_ms: `3247.300834` (`rustc 1.97.0 (2d8144b78 2026-07-07)`)
- pinned C++ wall_ms: `215.20175` (`AppleClang 21.0.0.21000334`)
- `rust_over_cpp_ratio`: `15.08956518243927`

Improved versus Wave 1 sample `21.00845179052245`. Still greater than 3, so leftover waves remain.

## Recorded profile (not timing authority)

- stamp: `target/dam-break-perf/2026-09-21T16-07-39Z/`
- `kind`: `samply_cpu`
- `not_timing_authority`: `true`
- git HEAD: `907bdf360c423184df1dc9210cce60a65a073c8f`
- no `pair.json` in that stamp

## Decisions Made

- Admit leftover `check_invariants` / `slice_contains` from 24-02 sidecar `2026-09-21T15-52-27Z` (~32.85% self, first D-07 named share).
- Gate `candidate.check_invariants` only on `replace_solver_candidate` behind `debug_assertions`; `creation.rs` and `mutation.rs` stay fail-closed.
- Admit from exclusive unprofiled pair `2026-09-21T16-07-02Z` with `rust_over_cpp_ratio` `15.08956518243927`, strictly below Wave 1 `21.00845179052245`. Do not claim PERF-GATE.
- Sibling samply stamp `2026-09-21T16-07-39Z` is leftover-ranking input only for 24-04.

## Deviations from Plan

None - plan executed exactly as written.

## Authentication Gates

None.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 24-04 leftover ranking from post-leftover samply stamp `2026-09-21T16-07-39Z` (D-07 remainder: `replace_solver_candidate` clone, then `from_view`, then contact `Vec`/`RawVec`, then `listener_effects`, plus `prepare_permutation` `check_invariants` if still named). Do not gold-plate if a later pair is already ≤ 3. Independent AI review remains a later phase step — this implementing agent does not self-approve.

---
*Phase: 24-shared-hot-path-waves-through-3*
*Completed: 2026-09-21*

## Self-Check: PASSED

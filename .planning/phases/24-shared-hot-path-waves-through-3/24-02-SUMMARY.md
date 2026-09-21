---
phase: 24-shared-hot-path-waves-through-3
plan: "02"
subsystem: performance-admission
tags: [unprofiled-pair, samply, PERF-ADMIT, particle_rows, leftover-retarget]

requires:
  - phase: 24-shared-hot-path-waves-through-3
    provides: neighborhood Proxy.row ParticleIndex plus private aligned pair_rows
provides:
  - exclusive unprofiled pair stamp 2026-09-21T15-50-46Z with rust_over_cpp_ratio 21.00845179052245
  - sibling samply stamp 2026-09-21T15-52-27Z not_timing_authority leftover retarget
affects:
  - 24-03 leftover admit-or-skip from post-wave-1 samply
  - PERF-GATE leftover scalar waves until pair.json <= 3

tech-stack:
  added: []
  patterns:
    - admit one named wave only when a new exclusive unprofiled pair.json improves on this host
    - samply sibling stamps retarget leftovers and stay not_timing_authority
    - keep Unreviewed local sample banners; leave reviewed_reports empty

key-files:
  created: []
  modified:
    - docs/playground-dam-break-timing.md
    - docs/native-performance-audit.md

key-decisions:
  - "Admit wave 1 from exclusive unprofiled pair 2026-09-21T15-50-46Z with rust_over_cpp_ratio 21.00845179052245, strictly below Phase 23 327.53395024734476."
  - "Do not claim PERF-GATE: 21.01 remains greater than 3; leftover D-07/D-09 waves stay required and check_invariants was not gated in wave 1."
  - "Sibling samply stamp 2026-09-21T15-52-27Z is leftover-ranking input only (kind samply_cpu, not_timing_authority true, no pair.json)."

patterns-established:
  - "Mint a new exclusive target/dam-break-perf stamp; never overwrite 2026-09-21T04-32-19Z or 2026-09-21T04-34-32Z."
  - "Copy the 3x number only from pair.json; keep a historical MEASURED_HEAD 6d98531 pointer."

requirements-completed: [PERF-ADMIT]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T15:54:16Z

duration: 5min
completed: 2026-09-21
---

# Phase 24 Plan 02: Wave 1 Unprofiled Pair Admission Summary

**Wave 1 `particle_rows` admitted on this host: exclusive unprofiled Dam Break Medium pair stamp `2026-09-21T15-50-46Z` has `rust_over_cpp_ratio` `21.00845179052245`, down from Phase 23 `327.53395024734476`; PERF-GATE is not claimed.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-21T15:49:33Z
- **Completed:** 2026-09-21T15:54:16Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Re-ran locked `just playground-dam-break-bench` (1920 particles, 60 warmup + 600 measured) into a **new** exclusive stamp. `pair.json` is `kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`.
- Admitted wave 1: ratio `21.00845179052245` is finite and strictly below `327.53395024734476`. Phase 23 stamps `2026-09-21T04-32-19Z` and `2026-09-21T04-34-32Z` remain on disk.
- Captured sibling samply stamp `2026-09-21T15-52-27Z` (`kind: samply_cpu`, `not_timing_authority: true`, `rust.json.gz` present, no `pair.json`) for leftover ranking.
- Refreshed unreviewed timing and audit notes from the new `pair.json` only. Did not claim PERF-GATE (ratio still > 3). `reference/performance/manifest.toml` still has `reviewed_reports = []`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Unprofiled re-pair into a new exclusive stamp** - `d55a363` (docs)
2. **Task 2: Sibling samply retarget and unreviewed audit note** - `d2afc7c` (docs)

**Plan metadata:** pending `docs(24-02): complete wave-1 unprofiled pair admission plan`

## Files Created/Modified

- `docs/playground-dam-break-timing.md` — current unreviewed sample from stamp `2026-09-21T15-50-46Z`; historical MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6` / `327.53395024734476`
- `docs/native-performance-audit.md` — Wave 1 admission subsection; leftover retarget stamp; **PERF-GATE is not claimed**

Gitignored evidence (not committed):

- Pair: `target/dam-break-perf/2026-09-21T15-50-46Z/pair.json`
- Profile: `target/dam-break-perf/2026-09-21T15-52-27Z/profile-identity.json` and `rust.json.gz`

## Recorded pair (3× authority)

Copied from `target/dam-break-perf/2026-09-21T15-50-46Z/pair.json` only.

- git HEAD: `27dc3191da7cc4e8a79571c608673e5a563d3c3c`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- particles: `1920`
- warmup_steps: `60`
- measured_steps: `600`
- native Rust wall_ms: `4610.920125` (`rustc 1.97.0 (2d8144b78 2026-07-07)`)
- pinned C++ wall_ms: `219.479292` (`AppleClang 21.0.0.21000334`)
- `rust_over_cpp_ratio`: `21.00845179052245`

Improved versus Phase 23 sample `327.53395024734476` at MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`. Still greater than 3, so leftover waves remain.

## Recorded profile (not timing authority)

- stamp: `target/dam-break-perf/2026-09-21T15-52-27Z/`
- `kind`: `samply_cpu`
- `not_timing_authority`: `true`
- git HEAD: `d55a36308fdb79fdfffa7157295be0bbf585f95d`
- no `pair.json` in that stamp

## Decisions Made

- Admit wave 1 from exclusive unprofiled pair `2026-09-21T15-50-46Z` with `rust_over_cpp_ratio` `21.00845179052245`, strictly below Phase 23 `327.53395024734476`.
- Do not claim PERF-GATE: 21.01 remains greater than 3; leftover D-07/D-09 waves stay required and `check_invariants` was not gated in wave 1.
- Sibling samply stamp `2026-09-21T15-52-27Z` is leftover-ranking input only (`kind samply_cpu`, `not_timing_authority` true, no `pair.json`).

## Deviations from Plan

None - plan executed exactly as written.

## Authentication Gates

None.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 24-03 leftover admit-or-skip from the post-wave-1 samply stamp `2026-09-21T15-52-27Z` (D-07 order: `check_invariants` / `slice_contains`, then `replace_solver_candidate`, then `from_view`, then contact `Vec`/`RawVec`, then `listener_effects`). Do not gold-plate if a later pair is already ≤ 3. Independent AI review remains a later phase step — this implementing agent does not self-approve.

---
*Phase: 24-shared-hot-path-waves-through-3*
*Completed: 2026-09-21*

## Self-Check: PASSED

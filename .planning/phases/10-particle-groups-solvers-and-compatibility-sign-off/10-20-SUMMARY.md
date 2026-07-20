---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "20"
subsystem: particle-constraint-solvers
tags: [rust, particles, elastic, spring, velocity-limit, source-parity]

requires:
  - phase: 10-02
    provides: Checked exact elastic and spring coefficients
  - phase: 10-03
    provides: Complete pair and triad rest-state records
  - phase: 10-05
    provides: Storage-owned stable particle identities and aligned lanes
  - phase: 10-12
    provides: Provenance-bound zero-length-pair and degenerate-triad decisions
provides:
  - Exact distinct S18 elastic, S19 spring, and S20 velocity-limit kernels
  - Complete topology preflight with stable-identity validation before one velocity commit
  - Probe-backed degeneracy, source-order, retargeting, and threshold-edge witnesses
affects: [10-21, 10-22, 10-23, particle-solvers, particle-world-coupling, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Pure source-ordered velocity candidates committed once through ParticleStorage
    - Complete topology admission before numerical effects
    - Exact probe classification for finite degeneracy without broad epsilon suppression

key-files:
  created:
    - crates/liquidfun/src/particle/solver/constraints.rs
    - crates/liquidfun/src/particle/solver/constraints/tests.rs
  modified:
    - crates/liquidfun/src/particle/solver.rs

key-decisions:
  - "Resolve and retain the complete stable particle-ID order before solving, validate every topology record and lane first, then commit one finite velocity candidate through ParticleStorage."
  - "Use pa/pb/pc in the pinned S18 numerical formula while requiring ka/kb/kc/s to pass complete stored-record validation; never reconstruct rest geometry from current positions."
  - "Apply the Plan 10-12 typed error only to exact zero-length active springs while preserving barrier-only and degenerate-triad source behavior without a global epsilon."

patterns-established:
  - "Constraint pass transaction: validate aligned lanes, every endpoint, every rest field, exact degeneracy decisions, and stable identities before replacing velocities once."
  - "Topology-order evidence: sequential pair and triad records observe prior candidate velocity updates in incumbent storage order."

requirements-completed: [PART-11, PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T16:48:00Z

duration: 1h
completed: 2026-07-20
---

# Phase 10 Plan 20: Exact Topology Constraint Kernels Summary

**Crate-private elastic, spring, and velocity-limit passes now consume validated stored topology in source order and commit one finite velocity candidate with exact LiquidFun strengths and thresholds.**

## Performance

- **Duration:** 1h
- **Started:** 2026-07-20T15:48:00Z
- **Completed:** 2026-07-20T16:48:00Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments

- Implemented S18 elastic with predicted positions, centroid-relative stored offsets, the pinned approximate inverse square root, exact rotation grouping, and default `0.25` strength.
- Implemented S19 spring with stored rest distances, exact sequential pair interaction, default `0.25` strength, and a named typed error for exact zero-length active springs.
- Implemented S20 velocity limiting with the exact diameter-times-inverse-timestep critical speed and a strict-above threshold branch.
- Preflighted lane alignment, stable particle identity order, every topology endpoint, and every pair/triad rest field before building a complete finite velocity candidate.
- Added twelve focused witnesses for manifest admission, empty and inactive controls, default activation, multi-constraint order, exact threshold edges, transformed rest state, retargeted join/split records, and both mandatory oracle classifications.

## Task Commits

Each task was committed atomically:

1. **Task 1: Solve pair and triad constraints exactly** - `8b908ea` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/constraints.rs` - Distinct safe S18-S20 kernels, complete topology preflight, stable-ID validation, and atomic velocity candidates.
- `crates/liquidfun/src/particle/solver/constraints/tests.rs` - Co-located control, activation, order, threshold, transform, retargeting, and probe-backed degeneracy witnesses.
- `crates/liquidfun/src/particle/solver.rs` - Private constraint-module registration.

## Decisions Made

- Kept all three passes independent. Their gates, coefficients, traversal semantics, and numerical formulas remain directly comparable to the pinned source and independently dispatchable by the later solver shell.
- Treated the complete triad as an admission contract: `pa`, `pb`, and `pc` drive the exact S18 formula, while `ka`, `kb`, `kc`, and `s` must remain finite and preserved even though the pinned `SolveElastic` formula does not numerically reference them.
- Snapshotted stable particle IDs before calculation and compared them again before the one storage-owned velocity replacement. Dense indices remain private and cannot silently retarget a solve.
- Reused the Plan 10-12 artifact verbatim: exact zero-length active springs fail with `ZeroLengthPairDistance`, barrier-only pairs are inactive no-ops in S19, and the all-zero degenerate triad remains finite and deterministic.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the new private constraint module**

- **Found during:** Task 1 implementation
- **Issue:** The planned `constraints.rs` file would not compile or be available to the later solver dispatcher without registration in the private solver parent.
- **Fix:** Added only `mod constraints;` to `particle/solver.rs`; no public visibility changed.
- **Files modified:** `crates/liquidfun/src/particle/solver.rs`
- **Verification:** The focused module filter, all-target clippy, all-target build, and complete test suite passed.
- **Committed in:** `8b908ea`

**2. [Rule 3 - Code Shape] Split the co-located witness module from production kernels**

- **Found during:** Task 1 simplification review
- **Issue:** Production and the required control/degeneracy matrix together exceeded the repository's approximate 628-line refactor trigger.
- **Fix:** Kept the private `#[cfg(test)] mod tests;` co-located under `constraints` and moved its body to `constraints/tests.rs`.
- **Files modified:** `crates/liquidfun/src/particle/solver/constraints.rs`, `crates/liquidfun/src/particle/solver/constraints/tests.rs`
- **Verification:** Production is 306 lines, tests are 405 lines, the longest production function is 38 lines, and all 12 focused witnesses pass.
- **Committed in:** `8b908ea`

**Total deviations:** 2 auto-fixed (2 blocking/code-shape).
**Impact on plan:** Both changes were narrow private seams required for compilation and repository source-shape guidance; no public API, dependency, unsafe/FFI code, external I/O, or solver-order change was introduced.

## Issues Encountered

- A Cursor-owned workspace check repeatedly held Cargo's shared build lock during focused development. Equivalent focused tests and clippy ran in an isolated target directory, then the mandatory default-target gate ran uninterrupted after the lock cleared.
- macOS executable-policy scanning dominated the all-target build and integration-test wall time. The exact ordered gate was preserved and completed without narrowing or skipped binaries.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-22 can dispatch S18-S20 directly in the closed manifest order without widening storage or topology visibility.
- Plan 10-23 can reuse the twelve focused witnesses for native solver-coverage closure.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all three implementation paths and this summary exist.
- Confirmed task commit `8b908ea` exists.
- Confirmed the mandatory Plan 10-12 artifact/provenance identity and both relevant per-case decisions are asserted by focused tests.
- Confirmed no topology reconstruction, broad epsilon, unsafe/FFI code, dependency, public API, or new external surface was introduced.
- Confirmed format, warning-denied all-target clippy, all-target build, 377 unit tests, every integration binary, and 19 doctests pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

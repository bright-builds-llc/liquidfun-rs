---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "06"
subsystem: particle-storage
tags: [rust, particle-solvers, scratch-lanes, transactionality, permutation]

requires:
  - phase: 10-02
    provides: Checked particle-system solver coefficients and particle flags
  - phase: 10-03
    provides: Complete topology records and authoritative permutation transaction
  - phase: 10-05
    provides: Source-ordered storage-owned group records with public and internal flags
provides:
  - Candidate-owned optional static-pressure, tensile-accumulation, and depth lanes
  - Sole dirty/current particle and group aggregate-flag authority
  - Storage-owned pending system-force marker
  - Exhaustive solver-state allocation, permutation, compaction, and buffer-ownership inventory
affects: [10-07, 10-08, 10-09, 10-10, 10-11, 10-12, 10-13, particle-solver]

tech-stack:
  added: []
  patterns:
    - Candidate-first bounded solver scratch allocation
    - Stable aggregate scans under one SolverState authority
    - Derived solver state excluded from consumer buffer transfer

key-files:
  created:
    - crates/liquidfun/src/particle/storage/solver_state.rs
    - crates/liquidfun/src/particle/storage/solver_state/tests.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/lane_inventory.rs
    - crates/liquidfun/src/particle/storage/lanes.rs
    - crates/liquidfun/src/particle/storage/permutation.rs

key-decisions:
  - "Keep aggregate group flags exclusively in SolverState and refresh them with a stable scan of authoritative GroupRecord values."
  - "Reserve optional scratch against declared capacity and construct complete aligned candidates before replacing live state."
  - "Carry solver state through creation and permutation candidates while keeping derived scratch out of consumer-owned buffer bundles."

patterns-established:
  - "Solver scratch gate: exact aggregate flags authorize bounded lazy allocation with deterministic zero backfill."
  - "Permutation authority: every aligned solver lane remaps through the existing candidate transaction; aggregates recompute from candidate rows."

requirements-completed: [PART-10, PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T16:43:30Z

duration: 1h 48m
completed: 2026-07-19
---

# Phase 10 Plan 06: Candidate-Owned Particle Solver State Summary

**Bounded candidate-first solver scratch, sole stable-scan aggregate authority, and exhaustive permutation ownership now protect every aligned particle solver lane.**

## Performance

- **Duration:** 1h 48m
- **Started:** 2026-07-19T14:55:18Z
- **Completed:** 2026-07-19T16:43:30Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added lazily gated static-pressure, tensile-accumulation, and depth vectors with deterministic zero backfill at particle counts N and N+1.
- Added dirty/current aggregate particle and group flags plus the pending system-force marker under one private `SolverState`.
- Preserved optional scratch by stable identity through creation, rotation, destruction compaction, and arbitrary valid permutation candidates.
- Extended invariant checks to reject misaligned or non-finite solver candidates before live replacement.
- Made solver allocation, clearing, permutation, and consumer-buffer ownership exhaustive in the lane inventory.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add deterministic solver scratch and lane inventory coverage** - `53e85d4` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/solver_state.rs` - Private aggregate, force-marker, optional scratch, validation, allocation, and permutation authority.
- `crates/liquidfun/src/particle/storage/solver_state/tests.rs` - Focused gate, rollback, alignment, permutation, aggregate-authority, and teardown tests.
- `crates/liquidfun/src/particle/storage.rs` - Candidate lifecycle and exact source-point gate integration.
- `crates/liquidfun/src/particle/storage/lane_inventory.rs` - Exhaustive lifecycle and ownership entries for every new solver state value.
- `crates/liquidfun/src/particle/storage/lanes.rs` - Documents that consumer lane bundles exclude storage-derived solver scratch.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Carries prepared solver state through the existing atomic permutation commit.

## Decisions Made

- Used one combined aggregate group value containing public and private internal flags, avoiding a second writable cache in the group table.
- Refresh aggregate scans at access and pinned structural commits while retaining explicit dirty state for flag mutation points and invariant validation.
- Kept optional scratch storage-derived rather than extending `OwnedLaneBundle`, so external teardown cannot return solver-only state.
- Preserved pending force across row permutations and clear it only through the explicit solver-consumption boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired solver state into storage and permutation candidates beyond the incomplete task file list**

- **Found during:** Task 1 (Add deterministic solver scratch and lane inventory coverage)
- **Issue:** The task action required candidate ownership, N/N+1 append alignment, destruction compaction, and permutation preservation, but its declared file list omitted the existing creation and permutation authorities.
- **Fix:** Added the smallest required integration in `storage.rs` and `permutation.rs`, plus a split focused test module to keep production code cohesive.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/permutation.rs`, `crates/liquidfun/src/particle/storage/solver_state/tests.rs`
- **Verification:** Seven focused solver-state tests, five lane-inventory tests, and the complete ordered Rust gate passed.
- **Committed in:** `53e85d4`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The narrow wiring was necessary to satisfy the plan's explicit candidate-ownership and permutation guarantees; no unrelated API or subsystem changed.

## Issues Encountered

- macOS provenance checks delayed first launch of freshly linked test executables in `_dyld_start`. The exact full test process was preserved while read-only `--list` warm-ups were attempted; all library, integration, and doctests ultimately passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Exact particle solver kernels can request each scratch lane through deterministic source-point gates and replace only complete finite candidates.
- Later group and force mutations have one aggregate/marker authority to update at pinned mutation points.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all six created/modified source files exist.
- Confirmed task commit `53e85d4` exists.
- Confirmed no known stub patterns or unplanned trust-boundary surfaces were introduced.
- Confirmed focused solver-state/inventory tests and the exact ordered Rust gate pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

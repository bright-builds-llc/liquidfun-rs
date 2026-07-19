---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "05"
subsystem: particle-storage
tags: [rust, particle-groups, invariants, transactionality, cache]

requires:
  - phase: 10-01
    provides: Public particle-group flags and borrow-scoped view contract
  - phase: 10-03
    provides: Complete particle topology records and storage validation
provides:
  - Storage-owned source-ordered particle-group records
  - Separate private internal group lifecycle and depth flags
  - Timestamped statistics cache with explicit invalidation and exact empty values
  - Linear ownership, range, membership, and cache invariant validation
affects: [10-06, 10-07, 10-10, 10-13, 10-15, particle-solver]

tech-stack:
  added: []
  patterns:
    - Source-ordered group authority under ParticleStorage
    - Typed public/internal flag separation
    - Candidate-first group-record reconstruction

key-files:
  created:
    - crates/liquidfun/src/particle/storage/group.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/validation.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage/permutation/tests.rs

key-decisions:
  - "Keep public flags, internal flags, range, strength, transform, association, and statistics together in one source-ordered ParticleStorage table."
  - "Represent statistics invalidation as an absent source timestamp while preserving finite stale values until recomputation; retained-empty records reset every aggregate to exact positive zero."
  - "Rebuild group records in particle membership order during transactions while retaining record metadata and appending retained/deferred empty records afterward."

patterns-established:
  - "Group authority: membership and GroupRecord candidates validate completely before one storage replacement."
  - "Empty lifecycle: zero-length records require CAN_BE_EMPTY or WILL_BE_DESTROYED and exact-zero statistics."

requirements-completed: [PART-09, PART-10, PART-11, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T14:51:15Z

duration: 1h 13m
completed: 2026-07-19
---

# Phase 10 Plan 05: Storage-Owned Group Authority Summary

**One source-ordered `ParticleStorage` table now owns complete per-group metadata, internal lifecycle/depth state, and timestamped finite statistics while validating exact membership agreement.**

## Performance

- **Duration:** 1h 13m
- **Started:** 2026-07-19T13:37:56Z
- **Completed:** 2026-07-19T14:51:15Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Replaced the range-only mutable authority with `GroupRecord` values carrying the owning system, public/internal flags, range, strength, transform, user association key, and statistics cache.
- Added linear source-order validation for same-system ownership, unique records, contiguous membership, exact record/lane agreement, finite metadata/cache values, and bounded counts.
- Added explicit timestamp invalidation and exact-positive-zero empty cache semantics for retained and deferred-destroy groups.
- Preserved group metadata transactionally through particle creation, compaction, and arbitrary valid permutations.
- Added focused tests for ownership, overlap/gap disagreement, empty retention, stable flag scans, cache invalidation, invalid internal state, and permutation remapping.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add storage-owned group records and cache invariants** - `35cdf5f` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/group.rs` - Private group records, internal flags, statistics cache, validation, and focused tests.
- `crates/liquidfun/src/particle/storage.rs` - Authoritative `group_records` table, flag scan, cache invalidation, and group lifecycle integration.
- `crates/liquidfun/src/particle/storage/validation.rs` - Linear record reconstruction and complete group/membership validation.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Carries authoritative group records through the existing candidate transaction.
- `crates/liquidfun/src/particle/storage/permutation/tests.rs` - Verifies record ranges and source order after whole-group permutation.

## Decisions Made

- Kept aggregate group flags out of records and caches so Plan 10-06 can remain their sole writable authority.
- Kept actual aligned particle depth values for Plan 10-06 solver state; group records own the private `NEEDS_UPDATE_DEPTH` scheduling state without duplicating the depth lane.
- Ordered non-empty records by current particle membership and retained/deferred empty records afterward, making validation and aggregate scans deterministic.
- Used the existing typed `InvalidGroupRange` error at the storage boundary to preserve exhaustive world-layer error mapping.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Migrated the existing permutation transaction**

- **Found during:** Task 1 (Add storage-owned group records and cache invariants)
- **Issue:** `storage/permutation.rs` directly constructed and committed the removed `group_ranges` field, so the planned single authority could not compile or remain transactional without migrating that path.
- **Fix:** Replaced the candidate range vector with rebuilt `GroupRecord` values and updated its focused regression assertion.
- **Files modified:** `crates/liquidfun/src/particle/storage/permutation.rs`, `crates/liquidfun/src/particle/storage/permutation/tests.rs`
- **Verification:** Nine focused permutation tests and the complete ordered Rust gate passed.
- **Committed in:** `35cdf5f`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The narrow migration was required to remove the second mutable authority and preserve the existing atomic permutation contract; no unrelated behavior changed.

## Issues Encountered

- The first clippy pass rejected a direct floating-point equality assertion in a cache test. The assertion now compares exact `f32` bits, which expresses the intended preservation contract and passes warning denial.
- macOS provenance checks delayed first launch of newly linked test executables. Cargo-reported test binaries were warmed with read-only `--list` launches while the exact full test run remained uninterrupted; all tests passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-06 can scan individual record flags into its sole aggregate group-flag authority and add aligned solver depth/scratch lanes.
- Later mutation plans can clone and validate complete group metadata with the existing `ParticleStorage` transaction.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all five created/modified source files exist.
- Confirmed task commit `35cdf5f` exists.
- Confirmed no writable `group_ranges` or aggregate group-flag field remains in particle storage.
- Confirmed the exact ordered Rust gate and focused group/validation/permutation tests pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

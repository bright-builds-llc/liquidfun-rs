---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "14"
subsystem: particle-group-mutation
tags: [rust, particle-groups, connectivity, stable-identity, topology]

requires:
  - phase: 10-12
    provides: Pinned split-created metadata and exact full pair/triad records
  - phase: 10-13
    provides: Atomic identity-preserving group mutation pattern
provides:
  - Exact contact-order split connectivity with longer-list union and A-tie retention
  - Stable-ID split transaction with first-longest survivor and source-ordered new groups
  - Historical pair/triad retargeting without rest-state regeneration
affects: [10-16, particle-groups, particle-topology, group-lifecycle]

tech-stack:
  added: []
  patterns:
    - Pure source-ordered connectivity planner beneath ParticleStorage
    - Group reassignment applied inside one prepared full-lane permutation
    - Probe-bound metadata defaults checked against immutable artifacts

key-files:
  created:
    - crates/liquidfun/src/particle/topology/connectivity.rs
    - crates/liquidfun/src/particle/storage/mutation/split.rs
    - crates/liquidfun/src/particle/storage/mutation/split/tests.rs
    - crates/liquidfun/src/particle/storage/permutation/group_reassignment.rs
  modified:
    - crates/liquidfun/src/particle/topology.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/mutation.rs
    - crates/liquidfun/src/particle/storage/permutation.rs

key-decisions:
  - "Model the upstream clone-and-zombie split as one equivalent stable-ID row permutation: survivor and unrelated rows retain source order, while later components move to the end in component-head and linked-list order."
  - "Apply new group membership while preparing the permutation so interleaved component members never create an invalid intermediate group layout."
  - "Copy probed public flags and user association to created groups, but use exact source defaults for strength, transform, statistics, and SOLID depth scheduling."

patterns-established:
  - "Split connectivity: scan contacts in order, union the longer list with A retaining ties, select the first longest head, then merge isolated zombies into that survivor."
  - "Split rollback: validate identities, connectivity, membership, group records, all lane remaps, and historical topology before replacing live ParticleStorage once."

requirements-completed: [PART-10, PART-11, TEST-01, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T08:06:25Z

duration: 1h 44m
completed: 2026-07-20
---

# Phase 10 Plan 14: Exact Connectivity-Based Group Split Summary

**Contact-order group splitting now preserves stable particle identity and historical constraint bytes while assigning the original group to the pinned first-longest component.**

## Performance

- **Duration:** 1h 44m
- **Started:** 2026-07-20T06:22:09Z
- **Completed:** 2026-07-20T08:06:25Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Implemented the pinned linked-list connectivity algorithm with contact-order scanning, longer-list union, A-tie retention, first-longest survival, and isolated-zombie merging.
- Planned later components by source head order and preserved each component's upstream linked-list member order.
- Replaced upstream cloning with an equivalent stable-ID permutation that preserves every `ParticleId`, keeps unrelated groups in source order, and appends new components after existing rows.
- Retargeted historical pair and triad dense endpoints through the permutation without sorting, regenerating, or changing any strength, distance, rest offset, coefficient, orientation, or record order.
- Applied the mandatory `split_created_metadata` witness verbatim: copied public flags and user association, default strength and identity transform, exact-zero invalid statistics, and SOLID depth scheduling.
- Added fixed and 128-case properties for ties, zombies, other-group contacts, source order, metadata, stable identities, topology bits, deterministic partitions, and invalid-candidate rollback.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement pinned connectivity and split transaction** - `8159997` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/topology/connectivity.rs` - Pure bounded source-equivalent linked-list connectivity planner and focused properties.
- `crates/liquidfun/src/particle/storage/mutation/split.rs` - Checked split candidate, identity validation, metadata construction, mapping, and no-fail commit.
- `crates/liquidfun/src/particle/storage/mutation/split/tests.rs` - Stable-ID, metadata, topology-bit, rollback, and bounded property evidence.
- `crates/liquidfun/src/particle/storage/permutation/group_reassignment.rs` - Narrow prepared-permutation seam for simultaneous membership reassignment.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Accepts validated row membership and metadata sources while preparing all aligned lanes.
- `crates/liquidfun/src/particle/storage/mutation.rs` - Registers the private split operation.
- `crates/liquidfun/src/particle/storage.rs` - Exposes private split candidates to the later world mutation shell.
- `crates/liquidfun/src/particle/topology.rs` - Registers the private connectivity module.

## Decisions Made

- Preserved stable IDs directly instead of modeling the upstream temporary clones as new public identities; the resulting permutation matches the source's post-clone ordering while avoiding public identity churn.
- Kept survivor members in their original dense source order because upstream does not clone that list; later components use linked-list traversal because upstream clones those rows in that order.
- Required callers to preflight and provide the exact number of fresh group IDs, leaving world arena, diagnostic, association-table, and lifecycle-journal reservation to the Plan 10-16 public shell.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added membership-aware prepared permutation**

- **Found during:** Task 1 (Implement pinned connectivity and split transaction)
- **Issue:** Assigning interleaved disconnected members to their new groups before permutation would temporarily violate contiguous group-range invariants, while permuting first would temporarily leave one group in multiple ranges.
- **Fix:** Added a narrow private permutation seam that copies reassigned membership into destination rows and rebuilds complete group records inside the same validated candidate.
- **Files modified:** `crates/liquidfun/src/particle/storage/permutation.rs`, `crates/liquidfun/src/particle/storage/permutation/group_reassignment.rs`, `crates/liquidfun/src/particle/storage/mutation/split.rs`
- **Verification:** Fixed intermingling/range assertions, complete-storage rollback comparisons, 128-case properties, and the exact ordered Rust gate passed.
- **Committed in:** `8159997`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The private seam is the minimum required to preserve the plan's no-invalid-intermediate and one-commit guarantees; it adds no public API or dependency.

## Issues Encountered

- The plan's literal focused command launched every unrelated integration binary with zero matching tests; macOS dynamic-loader scanning made that run substantially slower, but it completed uninterrupted and passed.
- The first exact Clippy pass identified a tuple type-complexity warning, a wildcard import, and bounded test conversions. Each was corrected, then the exact gate restarted from formatting and passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-16 can preflight world arena shells and diagnostic IDs, call `split_group_count`, then commit the returned `SplitPlan` with the ordered group identities.
- Reactive topology and solver work can rely on split preserving historical pair/triad order and all rest-state bits.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all eight created or modified task files exist.
- Confirmed task commit `8159997` exists.
- Confirmed no unsafe/FFI code, dependency, public dense-index surface, schema change, file/network access, or authentication surface was introduced.
- Confirmed 4 focused connectivity tests, 4 focused storage split tests, 330 library tests, every integration test, and 19 doctests pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

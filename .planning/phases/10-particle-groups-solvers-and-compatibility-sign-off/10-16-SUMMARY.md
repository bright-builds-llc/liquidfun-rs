---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "16"
subsystem: particle-groups
tags: [rust, particles, stable-handles, transactions, property-testing]

requires:
  - phase: 10-10
    provides: Deferred group destruction and retained-empty lifecycle
  - phase: 10-14
    provides: Exact source-ordered connectivity split candidates
  - phase: 10-15
    provides: Reactive topology, depth, flags, and storage-owned group caches
provides:
  - Public stable-handle group join, split, flag replacement, and explicit empty-shell destruction
  - Owned lifecycle evidence for join invalidation and application association cleanup
  - Association-capable split with preflighted clones and side-table capacity
  - Exact rollback evidence across curated failures and 128 generated public operation sequences
affects: [10-22, 10-23, particle-groups, lifecycle, public-api]

tech-stack:
  added: []
  patterns:
    - Clone-plan-validate-commit transactions across group arena, particle storage, owner list, and diagnostics
    - Owned mutation reports as the public bridge to application association cleanup
    - Exact semantic snapshots for public mutation rollback properties

key-files:
  created:
    - crates/liquidfun/src/world/particle_object/group_mutation.rs
    - crates/liquidfun/tests/particle_group_mutation.rs
  modified:
    - crates/liquidfun/src/association.rs
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/group.rs
    - crates/liquidfun/src/particle/storage/mutation.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/particle_object.rs

key-decisions:
  - "Keep the first group identity through join and return it as the MutationReport value while recording the second identity's exact destruction lifecycle."
  - "Keep split identity allocation and storage connectivity planning private, preflight every new shell and diagnostic identity, and return the original group first followed by source-ordered components."
  - "Expose only closed public group flags and explicit empty-shell destruction; populated groups cannot be silently ungrouped by the new lifecycle API."
  - "Clone a source association for every new split component before publishing any world mutation, then insert only into pre-reserved side-table capacity."

patterns-established:
  - "Public group mutation shell: validate poison, lock, handle ownership, and operation shape; construct complete arena/storage/owner/report candidates; publish by no-fail replacement."
  - "Rollback oracle: compare stable particle identities, memberships, exact float bits, group cache state, topology views, and counts before and after every rejected generated operation."

requirements-completed: [PART-09, PART-10, PART-11, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T21:16:00Z

duration: 2h 13m
completed: 2026-07-20
---

# Phase 10 Plan 16: Public Stable-Handle Group Mutation Summary

**Consumers can now join, split, update, and explicitly retire particle groups through stable identities with atomic arena/storage/report publication and exact rollback evidence.**

## Performance

- **Duration:** 2h 13m
- **Started:** 2026-07-20T19:03:00Z
- **Completed:** 2026-07-20T21:16:00Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- Added documented `World` methods for stable-handle group join, connectivity split, public flag replacement, and explicit retained-empty shell destruction.
- Preserved the first join identity, invalidated the second only at commit, and returned an owned `MutationReport` carrying the exact group destruction record needed by `AssociationMap` cleanup.
- Preserved the original split identity first, allocated later component identities in source order, and cloned the source application association to every new component through a preflighted association path.
- Made group arena, particle-system owner list, particle storage, diagnostic identity, and lifecycle evidence part of complete operation-specific candidates before any live authority changes.
- Added named public errors for lock state, handle ownership, arena exhaustion, association capacity, topology, same-group joins, and nonempty shell destruction while retaining nested error causes.
- Added curated black-box workflows and a generated public-operation property whose exact semantic snapshot covers particles, memberships, flags, group transforms/statistics/depths, contacts, pairs, triads, and group counts.

## Task Commits

Each task was committed atomically:

1. **Task 1: Expose stable-handle group mutations** - `2579ef6` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_object/group_mutation.rs` - Typed public errors and atomic join, split, flag, association, and empty-shell mutation shells.
- `crates/liquidfun/tests/particle_group_mutation.rs` - Curated stable-identity workflows and exact generated rollback oracle.
- `crates/liquidfun/src/world/particle_object.rs` - Documented public `World` mutation surface and child-module registration.
- `crates/liquidfun/src/world/object.rs` - Cloneable private group shells for arena transaction candidates.
- `crates/liquidfun/src/particle/storage/mutation.rs` - Minimal owned join-plan adapter over the exact private join implementation.
- `crates/liquidfun/src/particle/storage/group.rs` - Crate-private checked group-flag candidate seam.
- `crates/liquidfun/src/particle/storage.rs` - Curated internal mutation-plan exports.
- `crates/liquidfun/src/association.rs` - Generalized private association-capacity preflight.
- `crates/liquidfun/src/particle.rs` - Curated particle-module error export.
- `crates/liquidfun/src/lib.rs` - Curated crate-root error export.

## Decisions Made

- Used the existing `MutationReport` lifecycle vocabulary for join so the world publishes identity invalidation and application cleanup evidence together without owning application data.
- Kept exact connectivity, range permutation, pair/triad remapping, and internal group flags inside `ParticleStorage`; the public shell only accepts stable IDs and invariant-bearing public flags.
- Preallocated split shells in a cloned arena and pre-reserved the cloned particle system's owner list before asking storage for the final connectivity candidate.
- Limited explicit shell destruction to groups with zero members. The existing deferred particle lifecycle remains authoritative for populated groups.
- Kept the 596-line black-box test file cohesive because roughly one-third is a reusable exact semantic snapshot oracle and the remainder is one public workflow contract; the production child module remains a cohesive 354 lines.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added minimal private cross-module transaction seams**

- **Found during:** Task 1 implementation
- **Issue:** The public world shell could not reuse exact join/flag candidates or preflight multiple association entries through the existing narrow visibility surfaces.
- **Fix:** Added a private owned join-plan adapter, widened the checked flag mutator only to crate scope, generalized private association reserve, and registered a cohesive world child module.
- **Files modified:** `crates/liquidfun/src/association.rs`, `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/group.rs`, `crates/liquidfun/src/particle/storage/mutation.rs`, `crates/liquidfun/src/world/particle_object.rs`
- **Verification:** Dedicated 128-case mutation test and the exact four-command Rust gate passed.
- **Committed in:** `2579ef6`

**2. [Rule 1 - Bug] Preserved invalid-topology cause classification**

- **Found during:** Final post-gate diff review
- **Issue:** The first mapping grouped `InvalidGroupRange` with wrong-system ownership, which would reclassify a storage topology invariant as a handle error.
- **Fix:** Mapped `InvalidGroupRange` to the named `InvalidTopology` public error and reran the dedicated 128-case test plus a fresh complete gate.
- **Files modified:** `crates/liquidfun/src/world/particle_object/group_mutation.rs`
- **Verification:** `PROPTEST_CASES=128 cargo test -p liquidfun --all-features --test particle_group_mutation` passed 6/6; the fresh exact Rust gate passed.
- **Committed in:** `2579ef6`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug).
**Impact on plan:** The private seams reuse existing storage authority, and the error correction preserves the plan's named-cause requirement. No dependency, unsafe/FFI code, schema, external I/O, or public dense/topology surface was added.

## Issues Encountered

- macOS executable-policy scanning added long pauses before integration binaries. Every dedicated and full-suite command was allowed to finish uninterrupted.
- Warning-denied Clippy identified an overly complex tuple in the exact test snapshot; replacing it with a named `ParticleState` kept the oracle readable without weakening comparison coverage.
- The final diff review found the invalid-range cause mapping described above. After the one-line correction, both the 128-sequence command and a completely fresh ordered gate passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-22 can call the public mutation shell and storage-owned group authority without exposing rows, ranges, caches, or topology.
- Plan 10-23 can count the curated and 128-sequence rollback witnesses toward public particle-group coverage closure.
- No blockers remain.

## Self-Check: PASSED

- Confirmed implementation commit `2579ef6` exists.
- Confirmed both created files and all eight modified API/storage files exist.
- Confirmed join returns group A through the report, split returns the original identity first, and association cleanup/cloning use only curated public APIs.
- Confirmed the final-source dedicated command passed all six tests with `PROPTEST_CASES=128`.
- Confirmed fresh `cargo fmt --all`, warning-denied all-target/all-feature Clippy, all-target/all-feature build, 400 unit tests, every integration test, and 19 doctests passed before the implementation commit.
- Confirmed no public dense row, raw topology, internal flags, unsafe/FFI boundary, dependency, schema, file/network access, or authentication surface was introduced.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

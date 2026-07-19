---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "07"
subsystem: particle-storage
tags: [rust, particle-groups, topology, transactionality, proptest]

requires:
  - phase: 10-05
    provides: Source-ordered storage-owned group records and checked group ranges
  - phase: 10-06
    provides: Candidate-owned solver scratch, aggregate flags, and permutation-safe solver state
provides:
  - Seven closed operation-specific particle mutation candidate kinds
  - Explicit preserve-history and append-sort-deduplicate topology remap policies
  - No-fail commit after complete owned candidate preparation
  - Fixed-seed bounded group and topology mutation state-machine properties
affects: [10-08, 10-09, 10-10, 10-11, 10-12, 10-13, particle-groups, particle-topology]

tech-stack:
  added: []
  patterns:
    - Operation-specific prepare-then-commit mutation payloads
    - Explicit topology policy selected at each source operation
    - Independent stable-ID semantic property model with exact rollback snapshots

key-files:
  created:
    - crates/liquidfun/src/particle/storage/mutation.rs
    - crates/liquidfun/src/particle/storage/properties/group_model.rs
  modified:
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/lifetime.rs
    - crates/liquidfun/src/particle/storage/permutation/tests.rs
    - crates/liquidfun/src/particle/storage/properties.rs
    - crates/liquidfun/src/particle/storage/properties/permutation_model.rs

key-decisions:
  - "Represent create, join, split, zombie compaction, reactive regeneration, group-flag change, and ordinary rotation as a closed candidate enum with operation-specific payload newtypes."
  - "Append and stable-sort topology only for create, join, and reactive candidates; split, compaction, flag changes, and ordinary rotation preserve historical record order and rest data."
  - "Use fixed-seed bounded property sequences with a stable-ID semantic topology model rather than mirroring dense implementation state."

patterns-established:
  - "Mutation gate: prepare every owned lane, group record, solver scratch value, topology record, invalidation, and lifecycle projection before the no-fail commit."
  - "Topology policy gate: source operations must explicitly choose preserve-history or append-sort-first-duplicate behavior."
  - "Rollback witness: rejected handles, ranges, capacity, and non-finite topology compare the complete storage value before and after."

requirements-completed: [PART-10, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T22:05:53Z

duration: 1h 24m
completed: 2026-07-19
---

# Phase 10 Plan 07: Transactional Group Mutation Substrate Summary

**Seven operation-specific mutation candidates now own complete particle permutations and explicit topology policies, backed by a fixed-seed semantic rollback model.**

## Performance

- **Duration:** 1h 24m
- **Started:** 2026-07-19T20:41:36Z
- **Completed:** 2026-07-19T22:05:53Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added closed candidates for create-group, join, split, zombie compaction, reactive regeneration, group-flag changes, and ordinary rotations.
- Split permutation preparation from its no-fail commit and made historical preservation versus append-sort-first-duplicate behavior explicit at every caller.
- Preserved pair and triad order and every rest-value bit through ordinary rotations and split retargeting.
- Routed production row rotation and single-particle zombie compaction through the named mutation authority.
- Added fixed-seed, 128-case bounded command sequences that independently compare stable IDs, complete particle inputs, aligned lanes, contiguous group ranges, topology endpoints, topology order, and rest bits after every command.
- Proved exact full-storage rollback for invalid handles, invalid ranges, capacity exhaustion, non-finite topology, and a panic after preparation but before commit.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define operation-specific mutation candidates** - `002774b` (feat)
2. **Task 2: Extend the reproducible property model** - `f573c93` (test)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/mutation.rs` - Closed candidate kinds, operation-specific preparation, lifecycle projections, invalidation metadata, and no-fail commit.
- `crates/liquidfun/src/particle/storage/properties/group_model.rs` - Fixed-seed group/topology state machine, independent semantic model, and rollback fault injection.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Owned prepared permutations and explicit topology remap policies.
- `crates/liquidfun/src/particle/storage.rs` - Named candidate routing for ordinary rotation and zombie compaction.
- `crates/liquidfun/src/particle/lifetime.rs` - Explicit historical-order permutation selection for lifetime compaction.
- `crates/liquidfun/src/particle/storage/permutation/tests.rs` - Updated direct entrypoint tests and single-authority assertion.
- `crates/liquidfun/src/particle/storage/properties.rs` - Group mutation model registration.
- `crates/liquidfun/src/particle/storage/properties/permutation_model.rs` - Explicit historical-order selection in the existing permutation model.

## Decisions Made

- Kept every operation visible in a closed enum while using small payload newtypes around one complete owned mutation payload, so later public workflows cannot bypass operation classification.
- Carried group records and every particle and solver lane inside `PreparedPermutation`; mutation payloads add operation topology mode, cache/depth invalidation intent, and bounded lifecycle effects.
- Used stable semantic particle IDs in the property oracle and translated production dense topology endpoints only at comparison time, avoiding an implementation-shaped reference model.
- Guaranteed all seven candidate kinds execute in every property case through a fixed regression prefix before bounded generated commands.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Migrated existing permutation callers to explicit topology policy entrypoints**

- **Found during:** Task 1 (Define operation-specific mutation candidates)
- **Issue:** Removing the generic permutation entrypoint required the existing lifetime compaction, permutation tests, and permutation property model to choose a source-specific policy, while production rotation and zombie compaction needed named candidate routing to establish one authority.
- **Fix:** Added the narrow explicit preserve-history calls, updated the authority assertion, and routed `rotate_rows` and `compact_particle` through operation-specific candidates.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/lifetime.rs`, `crates/liquidfun/src/particle/storage/permutation/tests.rs`, `crates/liquidfun/src/particle/storage/properties/permutation_model.rs`
- **Verification:** Focused mutation and permutation tests plus the complete ordered Rust gate passed before commit.
- **Committed in:** `002774b`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The migration was the minimum integration needed to remove the ambiguous generic path and keep every existing caller compiling under an explicit source policy; no public API or unrelated subsystem changed.

## Issues Encountered

- The test-first module declaration produced the intended compile failure before the property model was added.
- Warning-denied clippy rejected uneven hexadecimal seed grouping; the literals were regrouped and the complete ordered gate was restarted from formatting.
- macOS provenance checks delayed first launch of freshly linked test binaries; all test processes were preserved and completed successfully.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Public group creation, join, split, reactive, and flag-change workflows can now build complete operation-specific candidates without inventing another mutation authority.
- Later topology planners have explicit source-operation append, ordering, duplicate, and historical-rest contracts.
- TEST-04 has a reproducible storage-level mutation oracle ready to extend with public workflows.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all eight created or modified source files exist.
- Confirmed task commits `002774b` and `f573c93` exist.
- Confirmed no known stub patterns or unplanned trust-boundary surfaces were introduced.
- Confirmed focused mutation, permutation, and 128-case group-model tests plus both exact ordered Rust gates pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

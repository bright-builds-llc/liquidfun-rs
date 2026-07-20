---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "10"
subsystem: particle-groups
tags: [rust, particles, groups, lifecycle, zombies]

requires:
  - phase: 10-09
    provides: Public group creation, append, and inspection
  - phase: 09
    provides: Authoritative particle zombie marking and deferred compaction
provides:
  - Deferred group-member destruction through the established particle zombie authority
  - Source-ordered particle callbacks followed by final ordinary-group destruction
  - Exact retained-empty CAN_BE_EMPTY statistics, depth, identity, and append behavior
affects: [10-16, 10-17, 10-18, 10-20, 10-21, particle-lifecycle]

tech-stack:
  added: []
  patterns:
    - Candidate-owned group marking with one authoritative storage commit
    - Clone-only direct-compaction preflight with in-place authoritative permutation
    - Group-shell invalidation driven by storage-owned WILL_BE_DESTROYED state

key-files:
  created:
    - crates/liquidfun/src/world/particle_object/group_lifecycle.rs
    - crates/liquidfun/src/world/particle_object/group_lifecycle_tests.rs
    - crates/liquidfun/tests/particle_group_lifecycle.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/group.rs
    - crates/liquidfun/src/particle/storage/mutation.rs
    - crates/liquidfun/src/world/particle_lifecycle.rs
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/src/world/step.rs

key-decisions:
  - "Repeat group destruction requests upgrade the pending listener bit but never duplicate a lifecycle occurrence."
  - "Particle compaction decides empty-group teardown; world shells disappear only after storage marks a non-retained group WILL_BE_DESTROYED."
  - "Direct compaction previews group teardown on a clone, then applies the existing in-place permutation so consumer-owned lane allocations remain stable."

patterns-established:
  - "Group lifecycle ordering: ascending particle destruction events precede the final group destruction event."
  - "Retained empty lifecycle: CAN_BE_EMPTY preserves a live identity, exact zero aggregates, and an allocated empty SOLID depth slice."

requirements-completed: [PART-10, TEST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T13:43:24Z

duration: 2h 48m
completed: 2026-07-20
---

# Phase 10 Plan 10: Deferred Particle Group Lifecycle Summary

**Particle-group destruction now reuses the existing zombie authority, retains empty groups exactly when requested, and emits one source-ordered lifecycle with atomic rollback.**

## Performance

- **Duration:** 2h 48m
- **Started:** 2026-07-20T10:55:00Z
- **Completed:** 2026-07-20T13:43:24Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a public group-particle destruction operation that validates world state and scoped identity, marks every current member through the existing pending-delete transition, and commits the complete candidate once.
- Made positive lifecycle steps and explicit compaction remove ordinary empty groups only after particle compaction, while retaining CAN_BE_EMPTY groups with exact zero aggregates and empty SOLID depth storage.
- Added ascending particle listener occurrences followed by one group destruction record so external association maps clean up only after final compaction.
- Preserved exactly-once behavior across repeated requests and capacity-failure retries.
- Added locked, poisoned, wrong-world, retained-empty append, rollback, association-cleanup, callback-order, and consumer-buffer allocation regressions.

## Task Commits

Each task was committed atomically:

1. **Task 1: Route group destruction through zombie compaction and empty retention** - `068f471` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_object/group_lifecycle.rs` - Public destruction workflow, direct-compaction preflight, lifecycle report assembly, and final shell cleanup.
- `crates/liquidfun/src/world/particle_object/group_lifecycle_tests.rs` - Locked and poisoned no-effect coverage using private step-state controls.
- `crates/liquidfun/tests/particle_group_lifecycle.rs` - Public deferred destruction, callback ordering, retained-empty append, wrong-world, and rollback evidence.
- `crates/liquidfun/src/particle/storage.rs` - Idempotent group-owned pending-delete transition with listener-bit upgrade.
- `crates/liquidfun/src/particle/storage/group.rs` - Storage-owned WILL_BE_DESTROYED query for final world-shell cleanup.
- `crates/liquidfun/src/particle/storage/mutation.rs` - Correct SOLID flag scheduling and initial depth computation.
- `crates/liquidfun/src/world/particle_lifecycle.rs` - Candidate empty-group destruction and total lifecycle-capacity preflight.
- `crates/liquidfun/src/world/particle_object.rs` - Cohesive child-module registration and compaction extraction.
- `crates/liquidfun/src/world/step.rs` - Ordinary destruction-event recording at the shared lifecycle journal seam.

## Decisions Made

- Kept the existing public per-particle repeated-mark contract unchanged and added a narrow group-owned transition that can upgrade a pending snapshot's listener request without duplicating the destruction.
- Used storage-owned WILL_BE_DESTROYED as the only ordinary-empty teardown signal; CAN_BE_EMPTY never fabricates a group callback.
- Recorded all requested particle callbacks before group callbacks, matching ascending old-row zombie compaction followed by group removal.
- Previewed explicit compaction on a clone solely for teardown allocation and validation, then retained the existing in-place authoritative permutation to preserve adopted buffer pointers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SOLID group creation bypassed depth scheduling**

- **Found during:** Task 1 (retained-empty SOLID regression)
- **Issue:** New-group planning assigned public flags directly and never scheduled or computed the aligned depth lane, so a retained empty SOLID group could not expose the required empty depth slice.
- **Fix:** Routed creation through the existing public-flag transition and computed depth on the validated candidate.
- **Files modified:** `crates/liquidfun/src/particle/storage/mutation.rs`
- **Verification:** Retained-empty integration test, 345 library tests, every integration target, and 19 doctests passed.
- **Committed in:** `068f471`

**2. [Rule 2 - Missing Critical] Existing lifecycle seams needed narrow group teardown support**

- **Found during:** Task 1 (group compaction integration)
- **Issue:** The literal plan file list did not include the storage zombie transition, step journal, or lifecycle candidate seams required to preserve one authority and atomic event capacity.
- **Fix:** Added narrow private helpers and ordinary destruction-event recording without broadening the public storage surface.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/group.rs`, `crates/liquidfun/src/world/particle_lifecycle.rs`, `crates/liquidfun/src/world/step.rs`
- **Verification:** Group lifecycle, Phase 9 zombie/lifecycle, full Rust gate, and association cleanup tests passed.
- **Committed in:** `068f471`

**3. [Rule 1 - Bug] Clone replacement changed consumer-owned lane allocation pointers**

- **Found during:** Task 1 full-gate verification
- **Issue:** The first direct-compaction candidate implementation replaced the complete particle system, which preserved values but changed adopted lane allocations.
- **Fix:** Kept the authoritative permutation in place and used a clone only to preflight empty-group records before any live mutation.
- **Files modified:** `crates/liquidfun/src/world/particle_object/group_lifecycle.rs`
- **Verification:** The focused consumer-buffer pointer regression passed, followed by the exact full Rust gate.
- **Committed in:** `068f471`

**Total deviations:** 3 auto-fixed (2 bugs, 1 missing critical)
**Impact on plan:** All changes are narrow lifecycle or correctness fixes required by the stated atomicity, retained-empty, and buffer-ownership contracts.

## Issues Encountered

- The first full gate exposed the adopted-buffer pointer regression after all 345 library tests had passed; the corrected in-place commit path passed the focused regression and a fresh exact gate.
- Diff review found group-record allocation was still ordered after the direct in-place permutation. A clone-only preview moved all fallible teardown preflight before effects while retaining consumer allocations.
- macOS provenance validation imposed long pauses between freshly rebuilt test executables. Every required gate was allowed to finish uninterrupted.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-16 can expose remaining safe group mutation workflows over the completed lifecycle and storage authority.
- Solver plans can rely on ordinary empty-group invalidation and retained-empty SOLID depth/state remaining coherent through compaction.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all nine created or modified task files exist.
- Confirmed task commit `068f471` exists.
- Confirmed no unsafe/FFI code, dependency, schema, file/network access, or authentication surface was introduced.
- Confirmed 4 public group lifecycle tests, 2 private guard tests, 3 Phase 9 zombie tests, 10 Phase 9 lifecycle tests, the focused consumer-buffer pointer regression, 345 library tests, every integration target, and 19 doctests pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

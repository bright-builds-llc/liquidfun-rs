---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "01"
subsystem: rigid-joints
tags: [rust, joints, handles, lifecycle, snapshots]
requires:
  - phase: 03-rust-object-model-and-storage-architecture
    provides: opaque world-scoped identities, generational arenas, and ordered destruction
  - phase: 04-math-settings-and-numerical-policy
    provides: checked finite numerical policy and native math vocabulary
provides:
  - closed tagged definitions for all eleven pinned joint kinds
  - owned common joint snapshots and explicit inverse-timestep reaction queries
  - atomic World-owned joint creation, inspection, and destruction authority
affects: [08-02, 08-03, 08-04, 08-05, 08-06]
tech-stack:
  added: []
  patterns: [closed enum dispatch, candidate-first world mutation, newest-first joint adjacency]
key-files:
  created:
    - crates/liquidfun/src/joint.rs
    - crates/liquidfun/src/joint/definition.rs
    - crates/liquidfun/src/joint/snapshot.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/tests/joint_contract.rs
  modified:
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Keep definitions and snapshots owned and tagged while World alone owns joint storage and effects."
  - "Use explicit inverse timestep inputs for reaction queries; per-kind solvers populate nonzero reactions in later plans."
patterns-established:
  - "Joint lifecycle: validate world state and all endpoints before arena or adjacency mutation."
  - "Kind dispatch: exhaustive enums preserve a closed pinned joint set without trait objects."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T22:51:19Z
duration: 33 min
completed: 2026-07-13
---

# Phase 8 Plan 01: Shared Joint Contract and Lifecycle Summary

**Closed eleven-kind joint vocabulary with opaque world-scoped lifecycle, owned snapshots, typed no-effect errors, and deterministic adjacency.**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-13T22:18:00Z
- **Completed:** 2026-07-13T22:51:19Z
- **Tasks:** 1
- **Files modified:** 11

## Accomplishments

- Added checked definitions and exhaustive tags for revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope, and motor joints.
- Replaced the placeholder two-body seam with typed `World` creation, owned inspection, reaction-query, and destruction authority.
- Preserved generational identities, newest-first body adjacency, last-suppressing-joint refilter behavior, and no-effect stale, foreign, wrong-kind, locked, and poisoned errors.

## Task Commits

Each task was committed atomically:

1. **Implement the closed public joint vocabulary and shared World lifecycle** - `1a05f50` (feat)

## Files Created/Modified

- `crates/liquidfun/src/joint.rs` - Curated public joint vocabulary.
- `crates/liquidfun/src/joint/definition.rs` - Checked per-kind definitions and closed `JointDef`.
- `crates/liquidfun/src/joint/snapshot.rs` - Owned common snapshot and limit state.
- `crates/liquidfun/src/world/joint.rs` - Private record plus checked `World` lifecycle and query authority.
- `crates/liquidfun/tests/joint_contract.rs` - Eleven-kind, identity, ordering, and no-effect contract tests.
- `crates/liquidfun/src/world/object.rs` - Integrates the richer record into existing cascades.

## Decisions Made

- Definitions and snapshots remain owned data contracts; storage, adjacency, refilter effects, and invalidation remain exclusively under `World` authority.
- Common reaction queries are present now with explicit inverse-timestep validation; the family solver plans replace zero runtime impulses with pinned per-kind calculations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Migrated placeholder joint callers to checked definitions**

- **Found during:** Task 08-01-01
- **Issue:** Existing object-model and association tests called the removed two-argument placeholder API.
- **Fix:** Constructed checked `RevoluteJointDef` values at those legacy joint-only test seams.
- **Files modified:** `crates/liquidfun/src/association.rs`, `crates/liquidfun/src/world/object.rs`, `crates/liquidfun/tests/object_model.rs`
- **Verification:** Full all-feature test suite and doctests pass.
- **Committed in:** `1a05f50`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Required API migration only; no scope expansion.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 08-02 through 08-05 can add type-specific configuration, runtime caches, setters, and solver reactions behind the established closed dispatch and lifecycle.
- Gear dependency topology and source-joint cascades remain explicitly assigned to Plan 08-05.

## Self-Check: PASSED

- All key created files exist.
- Task commit `1a05f50` is present.
- Focused joint tests and the complete ordered Rust gate pass.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-13*

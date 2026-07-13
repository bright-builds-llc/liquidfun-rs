---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "04"
subsystem: rigid-joints
tags: [rust, wheel, weld, friction, rope-joint, motor, solver]
requires:
  - phase: 08-01
    provides: checked joint identity, lifecycle, tagged definitions, and snapshots
provides:
  - checked wheel, weld, friction, rope-joint, and motor definitions, mutations, snapshots, anchors, and reactions
  - source-ordered spring, rigid/soft weld, capped friction/motor, and unilateral rope solver scratch
  - explicit RopeJointDef separation from the later standalone rope model
affects: [08-06, 08-10, 08-12]
tech-stack:
  added: []
  patterns: [candidate-first joint mutation, source-specific wake branches, transactional solver scratch]
key-files:
  created:
    - crates/liquidfun/src/world/joint/wheel.rs
    - crates/liquidfun/src/world/joint/weld.rs
    - crates/liquidfun/src/world/joint/friction.rs
    - crates/liquidfun/src/world/joint/rope.rs
    - crates/liquidfun/src/world/joint/motor.rs
  modified:
    - crates/liquidfun/src/joint/definition.rs
    - crates/liquidfun/src/joint/snapshot.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/joint.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Preserve each pinned setter's wake asymmetry: wheel motor operations wake unconditionally, motor offsets wake only on exact change, and softness/cap/rope-length tuning does not wake."
  - "Require RopeJointDef maximum length to remain strictly positive while keeping RopeJoint world-owned and distinct from standalone rope."
patterns-established:
  - "Capped constraints: accumulate privately, clamp in source order against timestep-scaled caps, and reject non-finite candidates before cache replacement."
  - "Unilateral constraints: classify exact inactive/upper state and retain only nonpositive accumulated rope impulse."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T23:42:04Z
duration: 17 min
completed: 2026-07-13
---

# Phase 8 Plan 04: Wheel, Weld, Friction, Rope-Joint, and Motor Joints Summary

**Five isolated non-gear joint families now preserve pinned spring, rigid/soft, capped, unilateral, correction, wake, snapshot, and reaction semantics.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-07-13T23:24:50Z
- **Completed:** 2026-07-13T23:42:04Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Replaced five placeholder definitions and pending snapshots with complete checked family contracts, world-authority setters, semantic anchors, runtime state, and reactions.
- Added private source-ordered solver scratch for wheel point-to-line/spring/motor constraints, rigid and soft weld branches, friction and motor caps, and predictive unilateral rope behavior.
- Added focused public and private tests for invalid inputs, exact setter wake branches, cold/warm caches, spring gamma/bias, rigid/soft weld state, force/torque caps, rope limit state, correction terms, and type separation.

## Task Commits

Each task was committed atomically:

1. **Implement wheel and weld joints** - `6cc0091` (feat)
1. **Implement friction, rope-joint, and motor joints** - `7be7c62` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/wheel.rs` - Wheel configuration, queries, mutations, spring/motor/line solver scratch, and reactions.
- `crates/liquidfun/src/world/joint/weld.rs` - Weld configuration, rigid/soft mass branches, queries, mutations, and reactions.
- `crates/liquidfun/src/world/joint/friction.rs` - Friction caps, no-wake setters, warm caches, and capped linear/angular impulses.
- `crates/liquidfun/src/world/joint/rope.rs` - World-owned RopeJoint state, predictive unilateral constraint, position correction, and maximum-length mutation.
- `crates/liquidfun/src/world/joint/motor.rs` - Relative offsets, changed-only waking, correction errors, and capped linear/angular impulses.
- `crates/liquidfun/src/joint/definition.rs` - Complete checked definitions for all five families.
- `crates/liquidfun/src/joint/snapshot.rs` - Owned tagged family runtime snapshots.
- `crates/liquidfun/src/world/joint.rs` - Exhaustive creation, snapshot, and reaction dispatch.
- `crates/liquidfun/tests/joint_*.rs` - Focused consumer contract tests for the five families.

## Decisions Made

- Exact source equality remains intentional for changed-only motor-offset setters; no epsilon was introduced.
- Wheel motor setters wake both bodies unconditionally, while wheel/weld softness, friction caps, RopeJoint length, and motor caps/correction preserve source no-wake behavior.
- The safe checked RopeJoint contract requires a positive maximum length and stays visibly distinct from the standalone `rope::Rope` scope owned by Plan 08-07.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Deepened the shared definition and snapshot contract**

- **Found during:** Tasks 08-04-01 and 08-04-02
- **Issue:** Plan 08-01 intentionally left these five family definitions and snapshots as placeholders, so complete type-specific APIs could not be implemented only inside the new family modules.
- **Fix:** Extended the existing closed definition, snapshot, curated-export, and runtime-dispatch files while preserving handle lifecycle, adjacency, and later island integration ownership.
- **Files modified:** `crates/liquidfun/src/joint/definition.rs`, `crates/liquidfun/src/joint/snapshot.rs`, `crates/liquidfun/src/world/joint.rs`, curated exports.
- **Verification:** All five focused targets, 173 private library tests, and the complete ordered all-feature Rust gate pass.
- **Committed in:** `6cc0091`, `7be7c62`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Necessary shared-contract completion only; no gear, standalone rope, island integration, origin mutation, callback, diagnostic, protocol, unsafe, or unordered-traversal scope was added.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-06 can dispatch the five private runtime types over copied island body lanes and commit caches in source order.
- Plan 08-10 can consume the complete definitions and snapshots for semantic reconstruction.
- Standalone rope remains isolated for Plan 08-07 and cannot be confused with the world-owned `RopeJointDef` contract.

## Self-Check: PASSED

- All ten created source/test files exist.
- Task commits `6cc0091` and `7be7c62` are present.
- All five focused targets, formatting, warning-denied Clippy, all-target build, all-feature tests, and doctests pass with the clean temporary Cargo target directory.
- `git diff --check` passes and `.codex/tasks/todo.md` remains outside implementation commits.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-13*

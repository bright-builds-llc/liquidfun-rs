---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "05"
subsystem: rigid-joints
tags: [rust, gear, dependencies, cascades, four-body-solver]
requires:
  - phase: 08-01
    provides: checked joint identity, lifecycle, snapshots, and destruction evidence
  - phase: 08-02
    provides: revolute and prismatic coordinates, frames, and solver conventions
provides:
  - checked dependency-owned gear definitions, snapshots, ratio mutation, anchors, and reactions
  - deterministic reverse dependency edges and newest-first source/body cascades
  - pinned RR/RP/PR/PP four-body Jacobian, warm-start, velocity, and position solver core
affects: [08-06, 08-09, 08-10, 08-12, 08-13]
tech-stack:
  added: []
  patterns: [dependency-first validation, reverse ownership edges, deduplicated ordered cascades]
key-files:
  created:
    - crates/liquidfun/src/world/joint/gear.rs
    - crates/liquidfun/src/world/joint/gear/solver.rs
    - crates/liquidfun/tests/joint_gear.rs
  modified:
    - crates/liquidfun/src/joint/definition.rs
    - crates/liquidfun/src/joint/snapshot.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/world/object.rs
key-decisions:
  - "Gear definitions own two source JointIds; World derives and stores their two moving endpoints only after complete live-kind validation."
  - "Safe Rust strengthens the upstream delete-gear-first precondition with newest-first reverse-edge cascades and explicit owned causes."
patterns-established:
  - "Gear lifetime: remove every dependent gear before invalidating a source joint, detaching both reverse edges during gear removal."
  - "Gear solving: preserve pinned A/B/C/D lanes and RR/RP/PR/PP scalar grouping without hash traversal or unsafe code."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T00:22:33Z
duration: 45 min
completed: 2026-07-14
---

# Phase 8 Plan 05: Gear Dependencies and Four-Body Solving Summary

**Dependency-owned gear joints with deterministic safe-Rust cascades and a pinned four-body Jacobian core for every revolute/prismatic combination.**

## Performance

- **Duration:** 45 min
- **Started:** 2026-07-13T23:37:00Z
- **Completed:** 2026-07-14T00:22:33Z
- **Tasks:** 1
- **Files modified:** 20

## Accomplishments

- Replaced the placeholder two-body gear definition with checked source-joint identities, finite signed/zero ratios, derived endpoints, owned dependency/body snapshots, and source-compatible ratio mutation.
- Added forward and reverse dependency ownership so explicit source destruction and body cascades remove dependent gears newest-first, deduplicate shared-source cascades, and never leave dangling reverse edges.
- Added source-grouped RR/RP/PR/PP Jacobian initialization, warm-start application, velocity solving, position correction, and reaction calculations over explicit A/B/C/D lanes.
- Added focused no-effect, cascade-order, ratio-sign, inspection, reaction, and solver tests while retaining the complete Phase 6/7 regression suite.

## Task Commits

1. **Implement gear lifecycle, mutations, snapshots, reactions, and solver** - `57d39b1` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/gear.rs` - Gear source geometry, runtime state, queries, mutation, snapshots, and solver authority.
- `crates/liquidfun/src/world/joint/gear/solver.rs` - Pinned four-body Jacobian and impulse application core.
- `crates/liquidfun/tests/joint_gear.rs` - Public dependency, ratio, no-effect, and cascade coverage.
- `crates/liquidfun/src/joint/definition.rs` - Dependency-owned checked `GearJointDef` and optional pre-resolution body contract.
- `crates/liquidfun/src/joint/snapshot.rs` - Owned gear source/body/coordinate snapshot.
- `crates/liquidfun/src/world/object.rs` - Gear dependency causes, complete destruction evidence, reverse-edge detachment, and body-cascade collection.

## Decisions Made

- Gear definitions expose source `JointId` values instead of asking callers to repeat derived body endpoints; `World` remains the only authority that can resolve and validate the complete topology.
- The creation constant is retained when the ratio changes, matching the pinned setter; the setter validates first and neither wakes bodies nor mutates cached impulse state.
- The large owned gear snapshot remains `Copy` with a documented lint exception because its size comes from six opaque semantic identities, not storage payload leakage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Deepened the shared definition and snapshot constructor contract**

- **Found during:** Task 08-05-01
- **Issue:** The placeholder `JointDef::bodies()` assumed every definition owned two body IDs, but gear bodies can only be derived after resolving live source joints.
- **Fix:** Made pre-resolution bodies optional and passed each record's already-validated bodies into shared snapshot construction; all ten non-gear families retain identical behavior.
- **Files modified:** `crates/liquidfun/src/joint/definition.rs`, `crates/liquidfun/src/joint/snapshot.rs`, and existing family snapshot call sites.
- **Verification:** Warning-denied Clippy, all-target build, all-feature tests, and all doctests pass.
- **Committed in:** `57d39b1`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Necessary contract deepening only; no new dependency, unsafe code, unordered traversal, or public storage coordinate was introduced.

## Issues Encountered

- The first post-extraction Clippy run found one private associated constant; visibility was narrowed correctly for the parent gear module and the complete ordered gate then passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-06 can consume the explicit `GearSolverBody` A/B/C/D contract and gear runtime methods from its exhaustive island dispatcher.
- Dependency snapshots and destruction causes are ready for the lifecycle timeline, diagnostics, and differential protocol plans.
- No blockers remain.

## Self-Check: PASSED

- All three key created files exist and task commit `57d39b1` is present.
- Focused gear tests pass 6/6; private four-body solver tests pass 2/2.
- The mandatory ordered Rust gate and `git diff --check` pass with a clean temporary Cargo target directory.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

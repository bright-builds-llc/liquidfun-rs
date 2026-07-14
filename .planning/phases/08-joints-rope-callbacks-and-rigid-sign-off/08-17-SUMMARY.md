---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "17"
subsystem: joint-live-solvers
tags: [rust, joints, wheel, weld, friction, rope, motor, transactional-staging]
requires:
  - phase: 08-14
    provides: typed activation-aware joint staging with copied runtimes and exact solver lanes
  - phase: 08-16
    provides: live scalar family migration pattern and transactional runtime commit
provides:
  - pinned wheel line, spring, motor, and point-to-line solving
  - pinned rigid and soft weld solving with complete Vec3 warm caches
  - capped friction and motor solving plus unilateral rope prediction and correction
affects: [08-18, 08-19, joint-parity, rigid-sign-off]
tech-stack:
  added: []
  patterns: [family-specific live constraints, source-ordered staged impulses, typed runtime rollback]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/src/world/joint/wheel.rs
    - crates/liquidfun/src/world/joint/weld.rs
    - crates/liquidfun/src/world/joint/friction.rs
    - crates/liquidfun/src/world/joint/rope.rs
    - crates/liquidfun/src/world/joint/motor.rs
    - crates/liquidfun/tests/joint_wheel.rs
    - crates/liquidfun/tests/joint_weld.rs
    - crates/liquidfun/tests/joint_friction.rs
    - crates/liquidfun/tests/joint_rope.rs
    - crates/liquidfun/tests/joint_motor.rs
    - crates/liquidfun/tests/joint_island_solver.rs
key-decisions:
  - "Wheel, weld, friction, rope, and motor use dedicated activated constraint structs and cannot reach LegacyUnmigrated."
  - "Soft weld applies its angular impulse before deriving the linear point velocity, preserving the pinned sequential branch."
  - "All five complete runtimes remain copied candidates until every island candidate validates."
patterns-established:
  - "Remaining ordinary live migration: preserve each family's distinct staged geometry, cache, branch, and solve order behind one transactional dispatch."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T04:18:23Z
duration: 14min
completed: 2026-07-14
---

# Phase 8 Plan 17: Wheel, Weld, Friction, Rope-Joint, and Motor Live Solvers Summary

**The five remaining ordinary joint families now execute their distinct pinned spring, soft, unilateral, capped, correction, warm-cache, and position equations through `World::step`.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-14T04:04:00Z
- **Completed:** 2026-07-14T04:18:23Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Activated wheel solving with exact line and spring lever arms, gamma/bias, motor cap, spring-before-motor-before-line velocity ordering, and point-to-line position correction.
- Activated rigid and soft weld branches, including full anchor mass construction, complete Vec3 warming, angular-before-linear soft solving, and pinned position behavior.
- Activated friction angular/linear caps, unilateral rope prediction and upper correction, and motor offset correction with inverse-timestep terms and force/torque caps.
- Added consumer-visible reactions, branch and cap witnesses, off-center behavior, and complete five-family rollback after a late island failure.

## Task Commits

1. **Task 08-17-01: Wire five distinct non-gear family solvers into live islands** - `a9f1558` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Stages and executes five dedicated source-ordered constraint types through transactional island solving.
- `crates/liquidfun/src/world/joint/wheel.rs` - Exposes the narrow complete warm-cache observation required by live solving.
- `crates/liquidfun/src/world/joint/weld.rs` - Exposes complete cache state and separate soft angular/linear source-order operations.
- `crates/liquidfun/src/world/joint/friction.rs` - Exposes capped linear and angular warm caches.
- `crates/liquidfun/src/world/joint/rope.rs` - Exposes unilateral direction and impulse caches.
- `crates/liquidfun/src/world/joint/motor.rs` - Exposes capped correction warm caches.
- `crates/liquidfun/tests/joint_{wheel,weld,friction,rope,motor}.rs` - Exercises live world-step reactions and family-specific branches.
- `crates/liquidfun/tests/joint_island_solver.rs` - Proves all five typed runtimes and bodies roll back after a late island failure.

## Decisions Made

- Kept five distinct constraint structs so no shared capped or point solver can merge source-specific semantics.
- Applied the soft weld angular impulse before computing its linear point velocity, matching the pinned sequential mutation order.
- Retained gear as the sole explicit legacy family for Plan 08-18; this plan made no gear or compatibility-removal changes.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The RED wheel `World::step` witness observed a zero typed reaction while the family still used `LegacyUnmigrated`; it passed after dedicated staging and finalization.
- F32 normalization can make a capped force vector's computed length a few ulps above its scalar cap. Focused cap assertions use a four-epsilon diagnostic tolerance while the cached vector remains source-clamped.
- The source-order audit caught the soft-weld angular-before-linear dependency before commit; the dedicated runtime operations now preserve it explicitly.

## Verification

- RED: the live wheel reaction witness failed against the compatibility route before implementation.
- Focused suites: wheel 4, weld 5, friction 3, rope 4, motor 4, and joint-island solver 6 tests passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 184 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-18 can activate the four-body gear solver using live ordinary source coordinates and remove the final compatibility route.
- Every non-gear family now commits only a complete typed runtime through the existing all-or-nothing world transaction.

## Self-Check: PASSED

- All twelve modified files and this summary exist.
- Commit `a9f1558` records Task 08-17-01.
- Only gear reaches `stage_legacy_unmigrated`; all five Plan 08-17 families finalize typed runtimes.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

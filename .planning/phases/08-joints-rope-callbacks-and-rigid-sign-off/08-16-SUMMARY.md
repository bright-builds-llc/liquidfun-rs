---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "16"
subsystem: joint-live-solvers
tags: [rust, joints, distance, pulley, mouse, solver, transactional-staging]
requires:
  - phase: 08-14
    provides: typed activation-aware joint staging with copied runtimes and exact solver lanes
  - phase: 08-15
    provides: established live-family migration pattern and transactional family runtime commit
provides:
  - pinned rigid and soft distance solving with off-center point velocities and warm reactions
  - pinned pulley ratio solving with rotating anchors and fresh position effective mass
  - pinned body-B-only mouse solving with angular damping, force capping, and target bias
affects: [08-18, 08-19, joint-parity, rigid-sign-off]
tech-stack:
  added: []
  patterns: [family-specific live constraints, source-ordered point velocities, typed runtime rollback]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/src/world/joint/distance.rs
    - crates/liquidfun/src/world/joint/pulley.rs
    - crates/liquidfun/src/world/joint/mouse.rs
    - crates/liquidfun/tests/joint_distance.rs
    - crates/liquidfun/tests/joint_pulley.rs
    - crates/liquidfun/tests/joint_mouse.rs
    - crates/liquidfun/tests/joint_island_solver.rs
key-decisions:
  - "Distance, pulley, and mouse use dedicated activated constraint structs and cannot reach LegacyUnmigrated."
  - "Mouse damping multiplies body B at source-ordered warm application time so earlier joint warm impulses are preserved."
  - "Pulley position solving recomputes directions, angular levers, and effective mass from the current candidate pose on every iteration."
patterns-established:
  - "Scalar live migration: retain exact anchor radii and point-velocity angular terms while mutating only copied typed runtimes."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T04:04:00Z
duration: 10min
completed: 2026-07-14
---

# Phase 8 Plan 16: Distance, Pulley, and Mouse Live Solvers Summary

**Distance, pulley, and mouse joints now execute their pinned off-center, soft, ratio, target, force-cap, and warm-cache equations through ordinary `World::step`.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-14T03:54:00Z
- **Completed:** 2026-07-14T04:04:00Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Activated rigid and soft distance constraints with rotated anchor radii, complete point velocities, exact gamma/bias grouping, warm impulses, and rigid-only position correction.
- Activated non-unit pulley constraints with source signs, angular levers, ratio-weighted impulses, and current-pose position mass recomputation.
- Activated body-B-only mouse constraints with local-anchor mass matrix, target bias, 0.98 angular damping, capped accumulated impulse, and no position correction.
- Added consumer-visible cold/warm reactions, off-center and rotating-anchor coverage, mouse body-A invariance, and complete three-family late-island rollback evidence.

## Task Commits

1. **Task 08-16-01: Wire distance, pulley, and mouse equations into live islands** - `1ea8c5b` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Owns the three activated family constraints, exact source scratch, dispatch, impulse application, and transactional finalization.
- `crates/liquidfun/src/world/joint/distance.rs` - Exposes narrow private direction and impulse cache observations to the shared solver.
- `crates/liquidfun/src/world/joint/pulley.rs` - Exposes narrow private pulley direction and impulse cache observations.
- `crates/liquidfun/src/world/joint/mouse.rs` - Exposes the private local anchor and accumulated impulse required by body-B-only solving.
- `crates/liquidfun/tests/joint_distance.rs` - Covers rigid/soft off-center cold and warm reactions.
- `crates/liquidfun/tests/joint_pulley.rs` - Covers non-unit ratio, rotating anchors, and warm reactions.
- `crates/liquidfun/tests/joint_mouse.rs` - Covers target bias, force cap, warm reaction, and body-A invariance.
- `crates/liquidfun/tests/joint_island_solver.rs` - Covers typed distance caches and three-family rollback after a late island failure.

## Decisions Made

- Applied mouse angular damping when its source-ordered warm-start stage reaches body B, rather than restoring an angular velocity captured during bulk constraint construction. This preserves preceding joint warm impulses.
- Kept the three typed runtimes as the only reaction-cache authority. Activated families clear the temporary legacy cache through the existing finalizer.
- Recomputed pulley position geometry and effective mass on each iteration, matching the pinned source rather than reusing velocity-stage scratch.

## Deviations from Plan

None - the plan executed exactly as written. No other joint family was migrated.

## Issues Encountered

- The RED distance witness observed the expected zero typed reaction while the family still used `LegacyUnmigrated`; all focused witnesses passed after activation.
- Clippy's similarity and test-length lints required clearer angular-lever names and extraction of joint-fixture setup into a focused helper.
- The source-order review caught mouse damping placement before commit; the implementation now multiplies the current staged body-B angular velocity after earlier warm-start applications.

## Verification

- RED: the rigid distance `World::step` witness failed because the unmigrated typed runtime retained a zero reaction.
- `cargo test -p liquidfun --test joint_distance --all-features`: 5 passed.
- `cargo test -p liquidfun --test joint_pulley --all-features`: 3 passed.
- `cargo test -p liquidfun --test joint_mouse --all-features`: 5 passed.
- `cargo test -p liquidfun --test joint_island_solver --all-features`: 5 passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 184 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-17 can activate wheel, weld, friction, rope, and motor behind the same staged runtime boundary.
- Plan 08-18 can activate gear after every ordinary source family needed for coordinate/Jacobian construction is live.

## Self-Check: PASSED

- All eight modified files and this summary exist.
- Commit `1ea8c5b` records Task 08-16-01.
- Distance, pulley, and mouse have no `LegacyUnmigrated` dispatch route.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

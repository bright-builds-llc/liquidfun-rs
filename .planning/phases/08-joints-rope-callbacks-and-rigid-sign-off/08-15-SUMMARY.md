---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "15"
subsystem: joint-live-solvers
tags: [rust, joints, revolute, prismatic, solver, transactional-staging]
requires:
  - phase: 08-14
    provides: typed activation-aware joint staging with copied runtimes and exact solver lanes
provides:
  - source-ordered live revolute initialization, warm start, motor, limit, velocity, and position solving
  - source-ordered live prismatic axis, block, motor, limit, velocity, and position solving
  - typed reaction runtimes committed only through the complete validated world transaction
affects: [08-18, joint-parity, gear-coordinate-sources, rigid-sign-off]
tech-stack:
  added: []
  patterns: [family-specific live constraints, copied runtime mutation, late-failure rollback]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/src/world/joint/revolute.rs
    - crates/liquidfun/src/world/joint/prismatic.rs
    - crates/liquidfun/tests/joint_revolute.rs
    - crates/liquidfun/tests/joint_prismatic.rs
    - crates/liquidfun/tests/joint_island_solver.rs
key-decisions:
  - "Revolute and prismatic use dedicated activated constraint structs; the remaining nine families retain the explicit temporary LegacyUnmigrated state."
  - "Complete family runtimes remain copied candidates until every island candidate validates, preserving existing all-or-nothing world-step commit behavior."
patterns-established:
  - "Live family migration: derive source Jacobians once during staging, mutate only the copied runtime during iterations, and recompute source-required position Jacobians from candidate body state."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T03:52:00Z
duration: 14min
completed: 2026-07-14
---

# Phase 8 Plan 15: Revolute and Prismatic Live Solvers Summary

**Revolute and prismatic joints now execute their pinned non-center-anchor, motor, limit, warm-start, velocity, and position equations through ordinary `World::step`.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-14T03:38:00Z
- **Completed:** 2026-07-14T03:52:00Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Activated revolute staging with rotated anchor radii, the full symmetric 3-by-3 effective mass, fixed-rotation handling, exact limit classification, scaled warm caches, source-ordered motor and point/limit solves, and freshly recomputed position correction.
- Activated prismatic staging with world axis/perpendicular lanes, source lever arms, full block mass, axial motor mass, translation classification, complete warm impulses, motor-before-block velocity solving, and limit-aware position correction.
- Finalized complete typed runtimes so public reaction force, reaction torque, motor torque, and motor force observations reflect live solved caches rather than the removed center-lock compatibility path.
- Added consumer-visible cold/warm `World::step` coverage for free, lower, upper, equal, enabled-motor, disabled-motor, non-center-anchor, reaction, and late-island atomicity behavior.

## Task Commits

1. **Task 08-15-01: Wire pinned revolute and prismatic equations into live islands** - `e4949b9` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Owns the two activated family constraints, source-derived solver scratch, exhaustive dispatch, and copied-runtime finalization.
- `crates/liquidfun/src/world/joint/revolute.rs` - Exposes narrow private solver cache observations and preserves exact source limit-cache transitions.
- `crates/liquidfun/src/world/joint/prismatic.rs` - Exposes narrow private solver cache observations and preserves exact equal-limit cache behavior.
- `crates/liquidfun/tests/joint_revolute.rs` - Exercises non-center anchors, all limit states, motors, reactions, and cold/warm live steps.
- `crates/liquidfun/tests/joint_prismatic.rs` - Exercises non-center axes/anchors, all limit states, motors, reactions, and cold/warm live steps.
- `crates/liquidfun/tests/joint_island_solver.rs` - Proves complete live revolute/prismatic body and runtime rollback after a late island failure.

## Decisions Made

- Kept dedicated live constraint state for the two migrated families so neither can reach the shared compatibility solver.
- Preserved pinned expression grouping and branch sequence even where source cache behavior differs between the two families.
- Reused the existing candidate world transaction; no live joint runtime is written before every island, body, contact, proxy, and joint candidate validates.

## Deviations from Plan

None - the plan executed exactly as written. No other family solver was migrated.

## Issues Encountered

- The RED `World::step` witnesses initially observed zero reactions because both families still finalized the legacy shared cache. They passed after the typed family solvers became live.
- The source audit found a subtle distinction: disabled/fixed revolute initialization retains its axial cache while an inactive enabled limit clears it, and entering an equal prismatic limit retains its axial cache. Focused unit regressions now protect both branches.

## Verification

- RED: both new non-center live-reaction witnesses failed against the unmigrated compatibility path.
- `cargo test -p liquidfun --test joint_revolute --all-features`: 7 passed.
- `cargo test -p liquidfun --test joint_prismatic --all-features`: 7 passed.
- `cargo test -p liquidfun --test joint_island_solver --all-features`: 4 passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 184 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 08-16 and 08-17 can migrate the remaining ordinary family variants without reopening the revolute/prismatic solver paths.
- Plan 08-18 can consume the live revolute angle and prismatic translation coordinates for every four-body gear combination.

## Self-Check: PASSED

- All six key modified files and this summary exist.
- Commit `e4949b9` records Task 08-15-01.
- No revolute or prismatic variant can reach `LegacyUnmigrated`.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

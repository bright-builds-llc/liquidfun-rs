---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "18"
subsystem: joint-live-solvers
tags: [rust, joints, gear, four-body-solver, transactional-staging]
requires:
  - phase: 08-17
    provides: all ten ordinary joint families executing through typed live constraints
provides:
  - pinned RR, RP, PR, and PP four-body gear solving through semantic A/B/C/D lanes
  - alias-safe role-delta merging for repeated source-body solver lanes
  - exhaustive eleven-family dispatch with no generic compatibility cache or placeholder runtime
affects: [08-19, joint-parity, rigid-sign-off, phase8-evidence]
tech-stack:
  added: []
  patterns: [four-body candidate solving, alias-safe delta merge, exhaustive typed runtime commit]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/world/island.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/world/joint/gear.rs
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/tests/joint_gear.rs
    - crates/liquidfun/tests/joint_island_solver.rs
key-decisions:
  - "Gear solver roles remain semantic A/B/C/D lanes and repeated physical lanes receive the sum of their role deltas instead of last-write-wins replacement."
  - "Every joint solution now carries one complete typed runtime; the temporary shared impulse cache and pending runtime state are removed."
  - "Gear preserves the pinned no-dt-ratio warm cache behavior while all runtime and body changes remain staged until complete world validation."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T04:31:48Z
duration: 12min
completed: 2026-07-14
---

# Phase 8 Plan 18: Four-Body Gear and Complete Island Integration Summary

**All eleven joint families now execute source-ordered typed constraints through `World::step`, including four-body RR/RP/PR/PP gear solving with alias-safe staged runtime commit.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-14T04:19:00Z
- **Completed:** 2026-07-14T04:31:48Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Replaced gear's generic two-body compatibility path with typed A/B/C/D staging that calls the pinned gear initialization, warm-start, velocity, and position routines.
- Preserved all four RR/RP/PR/PP Jacobian combinations and positive, negative, and zero ratio behavior through consumer-visible four-distinct-body world steps.
- Added alias-safe candidate scatter so shared base or source bodies receive every semantic role contribution without a lost update.
- Removed `CommonConstraint`, `LegacyUnmigrated`, the pending joint runtime, and the separate shared linear/angular impulse fields from the live world model.
- Kept source encounter order and the pinned joint-before-contact velocity/contact-before-joint position loops, while retaining contact-only TOI solving.
- Extended the all-eleven-family island witness with a live contact and protected complete gear runtime/body rollback after an injected late-island failure.

## Task Commits

1. **Task 08-18-01: Integrate four-body gear and close the live solver call graph** - `38d5951` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Owns typed gear staging, four-lane gather/solve, alias-safe candidate merging, and exhaustive runtime finalization.
- `crates/liquidfun/src/world/joint/gear.rs` - Separates Jacobian/cache initialization from source-faithful warm impulse application.
- `crates/liquidfun/src/world/island.rs` - Supplies only semantic typed runtime inputs without compatibility cache lanes.
- `crates/liquidfun/src/world/joint.rs` - Removes the pending runtime and generic solver-cache storage.
- `crates/liquidfun/src/world/object.rs` - Commits one complete typed runtime per joint solution.
- `crates/liquidfun/tests/joint_gear.rs` - Exercises all four source combinations with four actual bodies and signed/zero ratios.
- `crates/liquidfun/tests/joint_island_solver.rs` - Exercises all eleven families with a contact and proves final-island gear rollback.

## Decisions Made

- Summed role-local deltas when gear lanes alias. The gear core still evaluates the source-semantic A/B/C/D values, but one physical solver body receives every matching role contribution atomically.
- Removed the compatibility representation instead of retaining unreachable generic code. Exhaustive enum dispatch now makes every live family produce a complete `JointRuntime` candidate.
- Preserved the pinned gear warm-start asymmetry: unlike several ordinary joint families, gear does not scale its accumulated impulse by the timestep ratio.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The RED world-step witness correctly observed zero gear reaction while gear still finalized the compatibility cache.
- The first repository-wide lint pass requested an immutable initialization argument and a shorter rollback test. Extracting the fixture helper resolved both without changing behavior.

## Verification

- RED: the four-body RR world-step witness failed because the live gear runtime retained a zero reaction cache.
- Focused suites: `joint_gear` 7, `joint_island_solver` 7, and `joint_collision_origin` 3 tests passed.
- Unit alias regression: repeated C/D lane deltas combine into one physical solver body.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 185 library tests, every integration target, and 13 doctests passed.
- Legacy-path audit: no `PendingFamily`, `CommonConstraint`, `solver_linear_impulse`, or `solver_angular_impulse` remains under `crates/liquidfun/src/world`.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-19 can add behavioral corpus evidence over a complete live eleven-family solver call graph.
- The implementation blocker `CR-08-14-01` is closed in production code and protected by consumer-visible live-step regressions.

## Self-Check: PASSED

- All seven modified code/test files and this summary exist.
- Commit `38d5951` records Task 08-18-01.
- No generic or pending joint solver path remains.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

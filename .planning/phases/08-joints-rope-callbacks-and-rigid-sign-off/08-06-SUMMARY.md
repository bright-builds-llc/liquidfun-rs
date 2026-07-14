---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "06"
subsystem: rigid-joints
tags: [rust, joints, islands, collision-filtering, origin-shift]
requires:
  - phase: 08-01
    provides: checked joint lifecycle and ordered body adjacency
  - phase: 08-02
    provides: revolute and prismatic runtime state and solver conventions
  - phase: 08-03
    provides: distance, rope, and pulley runtime state and solver cores
  - phase: 08-04
    provides: wheel, weld, friction, motor, and mouse runtime state and solver cores
  - phase: 08-05
    provides: gear dependency ownership and four-body solver core
provides:
  - source-ordered joint island traversal and exhaustive eleven-kind discrete solver dispatch
  - transactional joint warm-cache commit alongside body, contact, and proxy candidates
  - collide-connected pair suppression and selective pulley/mouse origin translation
affects: [08-08, 08-10, 08-12, 08-13, 08-14]
tech-stack:
  added: []
  patterns: [closed enum dispatch, bounded visitation lanes, prepare-then-commit]
key-files:
  created:
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/tests/joint_island_solver.rs
    - crates/liquidfun/tests/joint_collision_origin.rs
  modified:
    - crates/liquidfun/src/world/island.rs
    - crates/liquidfun/src/world/contact_solver.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/origin.rs
key-decisions:
  - "Use an ephemeral bounded joint visitation lane so candidate island construction stays read-only and transactional."
  - "Commit joint warm caches only after every island, body, contact, and proxy candidate validates."
patterns-established:
  - "Discrete island ordering: visit newest-first joint adjacency after contacts, solve joints before contacts in velocity iterations, then contacts before joints in position iterations."
  - "Joint side effects: collision suppression and origin translation validate complete candidates before mutating live world state."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T00:42:36Z
duration: 20 min
completed: 2026-07-14
---

# Phase 8 Plan 06: Joint Island, Collision, and Origin Integration Summary

**All eleven joint families now participate in source-ordered discrete islands with transactional warm caches, connected-body collision policy, and source-compatible origin shifts.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-14T00:22:33Z
- **Completed:** 2026-07-14T00:42:36Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Extended bounded island discovery with source-compatible newest-first joint adjacency, inactive-counterpart handling, explicit joint capacity, and stable encounter-order staging.
- Added a closed eleven-variant joint constraint dispatcher that shares body lanes with contact solving, preserves warm caches, and follows the pinned velocity and position ordering without entering the TOI path.
- Kept stepping atomic by staging every joint impulse and committing it only after all island, body, contact, and proxy candidates succeed.
- Enforced `collide_connected` for existing and future pairs, including multiple suppressing joints and refilter-driven restoration after the final suppressor is removed.
- Shifted only world-space pulley ground anchors and mouse targets while rejecting non-finite joint candidates without partial body, proxy, tree, or joint mutation.

## Task Commits

1. **Integrate joint traversal, solving, collision policy, and origin shifting** - `b205acc` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Exhaustive joint constraint construction, warm start, velocity solving, position solving, and staged impulse output.
- `crates/liquidfun/src/world/island.rs` - Bounded source-order joint graph traversal and island joint staging.
- `crates/liquidfun/src/world/contact_solver.rs` - Shared body lanes and pinned joint/contact iteration ordering.
- `crates/liquidfun/src/world/contact_manager.rs` - Live connected-joint collision eligibility checks.
- `crates/liquidfun/src/world/joint.rs` - Joint warm caches, refilter proxy touches, and selectively shifted definitions.
- `crates/liquidfun/src/world/object.rs` - Transactional joint impulse aggregation and commit.
- `crates/liquidfun/src/world/origin.rs` - Atomic body, broad-phase, and joint origin-shift candidates.
- `crates/liquidfun/tests/joint_island_solver.rs` - Source order, exhaustive dispatch, warm-cache, and late-failure atomicity coverage.
- `crates/liquidfun/tests/joint_collision_origin.rs` - Existing/future suppression and selective/atomic origin-shift coverage.

## Decisions Made

- Joint visitation uses an island-build-local bounded lane instead of persistent record flags, preserving the read-only candidate phase and eliminating cleanup-sensitive state.
- Joint impulses use the same prepare-then-commit boundary as bodies, contacts, and proxies, so a late island error cannot expose partial warm-cache progress.
- The TOI solver remains contact-only; discrete joint dispatch is reachable only through the ordinary island solve path.

## Deviations from Plan

None - the plan was implemented as specified without scope expansion.

## Issues Encountered

- The initial red island-order test exposed that joint adjacency was not yet connecting body components; the completed traversal reduced the scenario from three islands to the expected single source-ordered island.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The complete rigid joint graph now participates in ordinary discrete stepping and is ready for callback, timeline, diagnostics, and differential evidence plans.
- Collision restoration and origin shifting have regression coverage for multi-joint and transaction-failure cases.
- No blockers remain.

## Self-Check: PASSED

- All three created files exist and task commit `b205acc` is present.
- Focused joint island, collision/origin, and rigid CCD suites pass.
- The mandatory ordered Rust gate passes with a clean temporary Cargo target directory: format, warning-denied Clippy, all-target build, all-feature tests, and doctests.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "08"
subsystem: rigid-contact-callbacks
tags: [rust, contacts, callbacks, filtering, pre-solve, ccd]
requires:
  - phase: 03-rust-object-model-and-storage-architecture
    provides: borrow-scoped contact views, deferred commands, lock restoration, and poison semantics
  - phase: 08-06
    provides: joint-aware collision eligibility and transactional discrete solving
provides:
  - source-timed fixture-pair filtering at admission and flagged refilter
  - borrow-scoped pre-solve views with current and previous semantic manifolds
  - validated enabled, friction, restitution, and tangent-speed controls for discrete and CCD updates
affects: [08-09, 08-10, 08-12, 08-13, contact-timeline, rigid-evidence]
tech-stack:
  added: []
  patterns: [borrowed decision hook, closed validated directives, source-point event capture]
key-files:
  created:
    - crates/liquidfun/tests/contact_hook_timing.rs
  modified:
    - crates/liquidfun/src/world/contact.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/world/contact_solver.rs
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/src/world/continuous.rs
    - crates/liquidfun/src/world/continuous/event.rs
    - crates/liquidfun/src/world/step/continuous.rs
key-decisions:
  - "Use CollisionDecisionHook as the source-timed authority while adapting legacy StepHook observers without persistent registration."
  - "Carry validated material controls in a closed directive and reset only enabled state per update, matching the pinned contact contract."
patterns-established:
  - "Pair decisions run before allocation at admission and before removal at flagged refilter."
  - "Every touching solid discrete or CCD refresh emits its transition before pre-solve and applies directives before constraint construction."
requirements-completed: [JOIN-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T01:02:00Z
duration: 18 min
completed: 2026-07-14
---

# Phase 8 Plan 08: Filter and Pre-Solve Timing Summary

**Borrowed fixture filtering and validated pre-solve controls now execute at pinned contact-manager points for discrete and continuous occurrences.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-14T00:44:00Z
- **Completed:** 2026-07-14T01:02:00Z
- **Tasks:** 1
- **Files modified:** 11

## Accomplishments

- Replaced grouped post-update filtering with `CollisionDecisionHook::should_collide` at broad-phase admission and flagged refilter before contact allocation or removal.
- Added `FixturePairView` without contact identity and `PreSolveView` with current/previous semantic manifolds, body/fixture/child identity, and no mutable world or manifold authority.
- Added finite validated friction, restitution, and tangent-speed controls plus per-update enable reset, with solver-visible tangent-speed behavior.
- Routed the same borrowed hook through repeated continuous candidate and TOI-island refresh points while preserving panic poison, command discard, and lock restoration.
- Added eight focused black-box tests plus compile-fail evidence that no public `ContactId` exists.

## TDD Evidence

- **RED:** The focused target failed because `CollisionDecisionHook`, `FixturePairView`, `PreSolveView`, `NoDecisionHook`, and validated controls did not exist.
- **GREEN:** Admission/refilter, manifold, sensor, disable reset, material, CCD, and panic behaviors pass through the new source-timed manager path.
- **REFACTOR:** Shared contact-hook application was extracted, grouped hook collection was removed, and discrete/continuous paths share one directive runner.
- The failing RED state was not committed because repository Rust policy requires the complete passing pre-commit gate before every commit.

## Task Commits

1. **Install source-timed fixture filtering and eligible pre-solve controls** - `5af7a6c` (feat)

## Files Created/Modified

- `crates/liquidfun/tests/contact_hook_timing.rs` - Admission, refilter, manifold, sensor, reset, control, CCD, and panic evidence.
- `crates/liquidfun/src/world/contact.rs` - Body-aware snapshots, tangent speed, and closed control application.
- `crates/liquidfun/src/world/contact_manager.rs` - Pinned admission/refilter/update insertion points.
- `crates/liquidfun/src/world/step.rs` - Borrow-scoped public views, traits, directives, limits, and panic boundary.
- `crates/liquidfun/src/world/contact_solver.rs` - Source-compatible tangent-speed constraint term.
- `crates/liquidfun/src/world/continuous.rs` and `crates/liquidfun/src/world/continuous/event.rs` - Decision propagation through CCD selection and TOI islands.
- `crates/liquidfun/src/world/step/continuous.rs` - Continuous-stage hook threading and typed limit propagation.
- `crates/liquidfun/src/world/object.rs`, `crates/liquidfun/src/world.rs`, and `crates/liquidfun/src/lib.rs` - World integration and curated exports.

## Decisions Made

- Kept `StepHook` as an observation/deferred-command compatibility adapter while making `CollisionDecisionHook` the only new filtering authority; hooks remain borrowed per `World::step` call and `NoDecisionHook` explicitly replaces prior behavior.
- Kept friction, restitution, and tangent speed persistent as in the pinned setters, while enabled state resets at each contact update.
- Kept pair identity semantic and borrow-scoped; rejected pairs never acquire a public or private reusable identity exposed to the hook.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Threaded the hook through continuous refresh modules**

- **Found during:** Task 08-08-01 (continuous occurrence coverage)
- **Issue:** The plan named the discrete contact files, but repeated CCD eligibility could not use the same borrowed hook without routing it through the existing continuous stage, candidate validation, and TOI-island refresh functions.
- **Fix:** Added the minimum generic hook-run parameter and typed error propagation through `world/continuous.rs`, `world/continuous/event.rs`, and `world/step/continuous.rs`; no new authority or persistent registration was introduced.
- **Files modified:** `crates/liquidfun/src/world/continuous.rs`, `crates/liquidfun/src/world/continuous/event.rs`, `crates/liquidfun/src/world/step/continuous.rs`, and the existing `world/object.rs` pair-discovery seam.
- **Verification:** The focused swept-bullet test observes every eligible occurrence, and all prior rigid CCD selection, atomicity, substep, and budget tests pass in the all-feature suite.
- **Committed in:** `5af7a6c`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Minimal propagation required for the plan's explicit repeated-CCD acceptance criterion; no scope or hook authority expansion.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-09 can append filter, transition, pre-solve, post-solve, command, and destruction occurrences to one authoritative owned timeline at the source points established here.
- Diagnostic and differential plans can observe semantic pair/manifold/material state without contact handles or mutable-world callbacks.
- No blockers remain.

## Self-Check: PASSED

- The focused contact-hook target passes 8/8 and legacy hook-contract target passes 8/8.
- Formatting, warning-denied Clippy, all-target/all-feature build, all-feature tests, and doctests pass using the clean temporary Cargo target directory.
- Task commit `5af7a6c` exists and the lifecycle ID matches every Phase 8 artifact.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "05"
subsystem: rigid-sleeping-and-wake-propagation
tags: [rust, sleeping, wake-propagation, island-solver, transactional-commit]
requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "01"
    provides: Checked body sleep controls, wake policies, and sleep-clearing primitive
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "03"
    provides: Source-ordered island construction and candidate-only DFS waking
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "04"
    provides: Transactional all-island solver and body-state commit boundary
provides:
  - Pinned per-island sleep thresholds, timers, convergence gate, and whole-island transition
  - Transactional sleep candidates committed with solved motion and contact impulses
  - Source-ordered pending mutation wake resolution before island seeding
  - Solid contact creation, touching transition, and destruction wakes for both endpoints
affects: [07-07, 07-09, rigid-island-solver, ccd, rigid-differential]
tech-stack:
  added: []
  patterns: [candidate sleep evaluation, source-ordered wake markers, manager-ordered contact waking]
key-files:
  created:
    - crates/liquidfun/tests/rigid_sleeping.rs
  modified:
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/contact_solver.rs
    - crates/liquidfun/src/world/island.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/tests/rigid_island_order.rs
key-decisions:
  - "Return the final position-convergence result from the contact solver so sleep evaluation uses the exact last iteration outcome rather than re-deriving geometry."
  - "Resolve mutation wake markers in explicit newest-first body order before contact discovery and island construction."
  - "Wake both contact endpoints in stored contact orientation for solid creation, touching changes, and touching destruction while sensor contacts do not invent waking."
patterns-established:
  - "Island sleep is staged after solved motion and before WorldStepCandidate commit; failure discards timers and awake-state changes with every other solver lane."
  - "Wake sources use the existing body candidate primitive, preserving one place for awake-flag and sleep-time semantics."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T05:15:00Z
duration: 21 min
completed: 2026-07-13
---

# Phase 7 Plan 05: Sleeping and Wake Propagation Summary

**Rigid islands now sleep transactionally at pinned velocity, duration, and convergence boundaries, while every selected mutation and contact wake source reaches island seeding without activation or sensor contacts inventing wake-ups.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-13T04:54:00Z
- **Completed:** 2026-07-13T05:15:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added exact squared linear and angular sleep-threshold checks, source-ordered timer accumulation, static-body exclusion from the minimum, and the inclusive 0.5-second transition boundary.
- Carried the contact solver's final position-convergence result into per-island evaluation so unconverged contacts cannot sleep an otherwise still island.
- Transitioned every body in a qualifying island through the existing sleep primitive, clearing velocity, force, torque, and sleep time together.
- Kept sleep timers and awake states inside the existing all-island candidate so late-island and proxy failures cannot leak partial sleeping effects.
- Resolved type and sensor mutation wake markers in newest-first body order before contact discovery and island seeding.
- Matched solid-contact wake behavior at creation, touching-state change, and touching destruction while preserving sensor-contact, activation, damping, gravity-scale, bullet, world-gravity, zero-velocity, and preserve-sleep no-wake branches.

## Task Commits

Each task was committed atomically after the exact ordered Rust gate passed:

1. **Task 1: Evaluate and commit per-island sleep state** - `fe8da4a` (`feat`)
2. **Task 2: Complete source-specific wake propagation** - `a04634c` (`feat`)

## Files Created/Modified

- `crates/liquidfun/tests/rigid_sleeping.rs` - Threshold, timer, convergence, whole-island, rollback, mutation wake, contact wake, and no-wake regressions.
- `crates/liquidfun/src/world/body.rs` - Narrow candidate sleep-time accessors used by island staging.
- `crates/liquidfun/src/world/contact_solver.rs` - Final contact-position convergence returned with solved motion and impulses.
- `crates/liquidfun/src/world/island.rs` - Source-mapped sleep timer evaluation and whole-island sleep candidate transition.
- `crates/liquidfun/src/world/contact_manager.rs` - Manager-oriented solid contact creation, transition, and destruction waking.
- `crates/liquidfun/src/world/object.rs` - Newest-first pending mutation wake resolution before contact discovery.
- `crates/liquidfun/tests/rigid_island_order.rs` - Candidate-only DFS witness explicitly re-sleeps after the newly source-faithful contact-creation wake.

## Decisions Made

- Exposed one private `position_solved` result from the existing numerical kernel rather than rerunning manifold calculations or weakening the sleep convergence requirement.
- Retained upstream strictness: squared velocity values greater than their squared tolerances reset time, while equality remains sleep-eligible and `minimum_sleep_time >= TIME_TO_SLEEP` transitions the island.
- Used the established `candidate_set_awake` primitive for both sleep and wake transitions, keeping sleep-time reset and motion/accumulator clearing semantics centralized.
- Consumed mutation wake markers before pair discovery in explicit body-list order. Contact wakes then follow manager occurrence and stored endpoint order.
- Kept activation and passive controls wake-neutral. Accepted TOI waking remains reserved for Plan 07-09 as planned.

## Test Evidence

- Task 1 RED: `cargo test -p liquidfun --test rigid_sleeping thresholds --all-features` exited 101 with six intended missing-sleep failures; the unconverged guard passed before implementation because no sleep transition existed.
- Task 1 GREEN: the same target passed 7/7 tests covering below/equal/above velocity thresholds, before/equal/after duration, unconverged positions, mixed allowed sleep, whole-island transition, sleep clearing, and rollback.
- Task 2 RED: `cargo test -p liquidfun --test rigid_sleeping wake_sources --all-features` exited 101 with the three intended pending/contact wake gaps while the sensor-contact and passive-control no-wake witnesses passed.
- Task 2 GREEN passed 5/5 filtered wake-source tests and all 6 existing `rigid_body_controls` tests.
- The exact ordered Rust gate passed before each task commit:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- The final Task 2 gate completed with exit code 0 after 143 library unit tests, every integration target including 12/12 sleeping and 7/7 island-order tests, and 12 doctests.

## Simplification Review

- One compact island helper owns all sleep eligibility, timer, minimum, and transition logic in the same order as the pinned source.
- One contact wake helper applies the same candidate transition to endpoints in stored order across creation, update, and destruction.
- One pending-marker pass over the existing explicit body-order lane avoids a second wake queue, sorting, hashing, or persistent traversal state.
- The solver returns one boolean it already computes; no geometry duplication or public sleep-timer surface was introduced.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Carried final position convergence out of the contact solver**

- **Found during:** Task 1
- **Issue:** The planned island sleep gate required the final contact-position convergence result, but the existing solver discarded that computed boolean.
- **Fix:** Added a private `position_solved` field to the island constraint solution and consumed it during sleep staging.
- **Files modified:** `crates/liquidfun/src/world/contact_solver.rs`, `crates/liquidfun/src/world/island.rs`
- **Verification:** Unconverged deep-overlap regression remains awake; the complete ordered gate passes.
- **Committed in:** `fe8da4a`

**2. [Rule 3 - Blocking expectation] Updated the older DFS candidate-wake setup for contact-creation waking**

- **Found during:** Task 2 full verification
- **Issue:** The existing DFS test expected a newly created solid contact not to wake an asleep endpoint, conflicting with the pinned contact-manager behavior implemented by this plan.
- **Fix:** Kept the test's original candidate-only purpose by explicitly putting the connected body back to sleep after contact discovery and before diagnostic island construction.
- **Files modified:** `crates/liquidfun/tests/rigid_island_order.rs`
- **Verification:** The focused test and all 7 island-order tests pass alongside the new contact-wake regressions.
- **Committed in:** `a04634c`

### Process adjustment: RED evidence was not committed

- The repository requires the complete ordered Rust gate before every commit, so intentionally failing RED states were run but not committed. Each task produced one verified GREEN commit after preserving the RED failure evidence.

***

**Total deviations:** 2 implementation/test compatibility fixes and 1 commit-process adjustment. No public API expansion, dependency, unsafe code, or scope change was introduced.

## Issues Encountered

- The first Task 1 Clippy gate found `solve_islands` four lines over the configured limit. Extracting its existing lane validation into a focused helper simplified the function; the full ordered gate was restarted and passed.
- The first Task 2 full test gate surfaced the outdated contact-creation expectation described above. After the scoped fixture update, the complete ordered gate was restarted from formatting and exited 0.
- Long full-suite commands yielded through the command transport; retained Cargo session exit codes were used as completion authority before either commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Discrete islands now enter and leave sleep with pinned semantics, giving CCD Plan 07-09 a stable awake baseline and an explicit reserved TOI wake point.
- Differential evidence can compare whole-island awake states and source-specific transitions without exposing internal sleep timers.
- No blockers or residual production stubs remain for later Phase 7 plans.

## Self-Check: PASSED

- Task commits `fe8da4a` and `a04634c` exist, and the declared `rigid_sleeping.rs` artifact exists.
- All seven implementation/test files in the two-commit plan diff are represented above; focused and full verification passes.
- Stub and threat scans found no placeholder implementation, unsafe code, unordered wake collection, public sleep-timer exposure, network endpoint, authentication path, or new filesystem boundary.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*

---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "18"
subsystem: particle-lifecycle
tags: [rust, particles, world-step, zombie, continuous-resume]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "07"
    provides: transactional particle lifetime and zombie maintenance in World::step
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "09"
    provides: source-timed particle contact prefix and rigid reaction
provides:
  - one fresh-positive-dt predicate shared by every Phase 9 particle stage and the discrete rigid solve
  - storage-authoritative bidirectional synchronization between ZOMBIE flags and pending-delete identity state
  - public zero-dt, continuous-resume, and zombie-authority regression coverage
affects: [09-19, 09-20, 09-22, phase-10]
tech-stack:
  added: []
  patterns: [fresh-positive-dt stage guard, atomic flag-and-identity transition, dense-order zombie synchronization]
key-files:
  created:
    - crates/liquidfun/tests/particle_step_guards.rs
    - crates/liquidfun/tests/particle_zombie_authority.rs
  modified:
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/src/world/particle_lifecycle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/identity.rs
    - crates/liquidfun/src/particle/storage/properties.rs
    - crates/liquidfun/tests/particle_lifecycle.rs
key-decisions:
  - "Use one runs_particle_stages predicate for lifecycle, contacts, rigid reaction, preflight, and discrete solve so zero-dt and continuous resumes cannot repeat particle work."
  - "Capture ZOMBIE and any listener bit in the pending snapshot before changing identity state, then synchronize flag-originated zombies in ascending dense order before lifetime solving."
patterns-established:
  - "Particle stage eligibility is computed once from ContinuousStepKind::Fresh and a strictly positive time step."
  - "Every route into PendingDelete passes through one storage transition that first establishes authoritative public flags."
requirements-completed: [PART-01, PART-02, PART-07, PART-08, PART-14, PART-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-16T01:52:00Z
duration: 33 min
completed: 2026-07-16
---

# Phase 9 Plan 18: Step Guard and Zombie Authority Summary

**Particle lifecycle, contact, and rigid-reaction work now executes exactly once per fresh positive-duration step, while one storage transition keeps ZOMBIE flags, pending identity, snapshots, callbacks, and compaction coherent.**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-16T01:19:00Z
- **Completed:** 2026-07-16T01:52:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Moved lifetime advancement, zombie compaction, particle contacts, timestamps, and rigid reaction behind the same fresh-positive-dt predicate as the discrete rigid solve.
- Proved zero-duration calls preserve finite lifetime, pending identity, particle contacts, weights, and body state, and proved continuous resumes do not repeat particle stages.
- Centralized destruction marking so ZOMBIE and listener flags are captured before PendingDelete, then promoted already-flagged live rows in ascending dense order before the existing single compaction transaction.
- Added public exactly-once listener-order and created-with-ZOMBIE regressions while retaining paused positive-duration maintenance.

## Task Commits

Each task was committed atomically:

1. **Task 1: Gate every particle step stage on one fresh positive-dt predicate** - `9af86b2` (fix)
1. **Task 2: Synchronize ZOMBIE flags and pending-delete identity state** - `d66a873` (fix)

## Files Created/Modified

- `crates/liquidfun/src/world/step.rs` - Computes and applies the one fresh-positive-dt particle/discrete-stage predicate.
- `crates/liquidfun/src/world/particle_lifecycle.rs` - Synchronizes zombie flags before lifetime solving and compaction.
- `crates/liquidfun/src/particle/storage.rs` - Owns the atomic flag, snapshot, and PendingDelete transition.
- `crates/liquidfun/src/particle/storage/identity.rs` - Locks ZOMBIE-bearing owned pending snapshots.
- `crates/liquidfun/src/particle/storage/properties.rs` - Keeps the independent storage state machine aligned with the authoritative transition.
- `crates/liquidfun/tests/particle_step_guards.rs` - Covers zero dt, continuous continuation, and fresh positive controls.
- `crates/liquidfun/tests/particle_zombie_authority.rs` - Covers both synchronization directions, listener preservation, ordering, and exactly-once compaction.
- `crates/liquidfun/tests/particle_lifecycle.rs` - Retains paused maintenance on an eligible positive-duration step.

## Decisions Made

- Kept rigid pair discovery and contact updating outside the particle predicate; only the pinned discrete particle/rigid solve slice is gated.
- Reused the existing candidate particle arena and single survivor permutation; zombie synchronization introduces no new lifecycle owner or compaction path.
- Applied the repository-local Rust, architecture, code-shape, verification, and testing standards to keep domain state transitions in storage and World orchestration thin.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated internal snapshot and property-model expectations for authoritative ZOMBIE flags**

- **Found during:** Task 2 full all-feature verification
- **Issue:** Two internal regression guards still modeled a pending snapshot with its pre-mark flags, contradicting the newly required post-flag snapshot contract.
- **Fix:** Updated the focused identity assertions and independent state-machine model to insert ZOMBIE before comparing the returned pending snapshot.
- **Files modified:** `crates/liquidfun/src/particle/storage/identity.rs`, `crates/liquidfun/src/particle/storage/properties.rs`
- **Verification:** Both focused internal tests and the restarted four-command repository gate pass.
- **Committed in:** `d66a873`

***

**Total deviations:** 1 auto-fixed (1 Rule 1 bug).
**Impact on plan:** The change updates regression oracles to the planned public contract; runtime scope remains exactly the two named gap fixes.

## Issues Encountered

- The first Task 2 all-feature run correctly exposed stale internal expected snapshots. The gate was restarted from formatting after updating those independent expectations.
- Shared-worktree Cargo build locking lengthened gate time but did not cause a test hang or skipped verification.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- G09-STEP-GUARD/WR-01 and G09-ZOMBIE-AUTHORITY/WR-02 now have public regression closure.
- Plans 09-19 and 09-20 can close capacity/permutation and protocol gaps without changing the particle step boundary.
- No Phase 10 particle-group or solver behavior was introduced.

## Self-Check: PASSED

- Both created regression files exist.
- Commits `9af86b2` and `d66a873` contain plan ID `09-18`.
- Plan-specific step, body-contact, lifecycle, and zombie tests pass.
- Mandatory format, clippy, all-target build, all-feature tests, and doctests pass.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-16*

---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "09"
subsystem: particles
tags: [rust, particle-groups, transactions, topology, stable-identities, borrow-scoped-views]

requires:
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    plan: "05"
    provides: "Storage-owned group records, ranges, flags, transforms, strengths, and statistics state"
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    plan: "08"
    provides: "Pure bounded source-ordered particle-group sampling"
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    plans: ["12", "13"]
    provides: "Exact pair/triad generation and atomic target-preserving group join planning"
provides:
  - "Atomic World APIs for new particle groups and pinned temporary-create-plus-join append semantics"
  - "Application-owned group association installation reserved before world commit"
  - "Borrow-scoped storage-backed group views with stable member IDs, aligned depths, and aggregate statistics"
  - "Black-box rollback evidence for handle, capacity, sampling, topology, identity, and lifecycle state"
affects: [10-10-group-mutation, 10-14-particle-solvers, particle-group-consumers, compatibility-evidence]

tech-stack:
  added: []
  patterns:
    - "World-facing mutations build one complete owned ParticleStorage candidate before a no-fail shell/diagnostic commit"
    - "Append uses a hidden prospective group identity inside the candidate and publishes only the validated target identity"
    - "Physics metadata remains storage-owned while the world arena retains identity and owner only"

key-files:
  created:
    - crates/liquidfun/tests/particle_groups.rs
  modified:
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/mutation.rs

key-decisions:
  - "Preflight the full diagnostic range without advancing the world counter, then publish the group shell, storage candidate, and next diagnostic value only after every fallible stage succeeds."
  - "Discard an AppendTo recipe association with the hidden temporary group, matching pinned join semantics; a New recipe installs its association in the application-owned table after reservation."
  - "Migrate placeholder-era tests to recipe-created initial members instead of retaining a second public empty-shell creation API."

patterns-established:
  - "Group transaction key link: ParticleStorage::plan_group returns an owned GroupPlan whose commit_group replacement is infallible."
  - "Group inspection validates the arena shell and owning system before borrowing the authoritative storage range."

requirements-completed: [PART-09, PART-10, TEST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T05:30:52Z

duration: 2h 2m
completed: 2026-07-20
---

# Phase 10 Plan 09: Atomic Particle-Group World API Summary

**Owned storage candidates now preflight complete group sampling, lifecycle, contacts, topology, identities, append joins, and associations before publishing stable group state once**

## Performance

- **Duration:** 2h 2m
- **Started:** 2026-07-20T03:28:14Z
- **Completed:** 2026-07-20T05:30:52Z
- **Tasks:** 1
- **Files modified:** 13

## Accomplishments

- Added `World::create_particle_group` and its association-capable equivalent with locked/poisoned world checks, same-system append validation, pure bounded sampling, complete diagnostic preflight, candidate-only lifecycle creation, source-timed contact refresh, exact topology planning, and one no-fail commit.
- Implemented pinned append behavior by building a hidden temporary group inside cloned storage, joining it into the validated target with the existing exact rotation/cross-topology plan, returning the target ID, and never allocating a temporary public shell.
- Added complete borrow-scoped group inspection backed by authoritative storage ranges, including flags, transform, center, velocity, angular velocity, mass, inertia, stable source-ordered member IDs, and aligned optional depths.
- Added seven public group tests and migrated placeholder-era callers to the recipe workflow; focused buffer, lifecycle, lifetime, full workspace, and doctest evidence all pass.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement atomic create and append workflows** - `82f1acc` (feat)

## Files Created/Modified

- `crates/liquidfun/tests/particle_groups.rs` - Fill, stroke, positions, complete view, append identity, association, and rollback evidence.
- `crates/liquidfun/src/world/particle_object.rs` - Public group creation, association, view, planning, and no-fail commit workflow.
- `crates/liquidfun/src/world/object.rs` - Identity/owner-only group shell, group-specific creation errors, and non-mutating diagnostic-range preflight.
- `crates/liquidfun/src/particle/storage/mutation.rs` - `plan_group`/`commit_group` candidate link over metadata, exact topology, and optional append join.
- `crates/liquidfun/src/particle/storage.rs` - Storage-backed semantic group views and crate-private candidate capacity seams.
- `crates/liquidfun/src/association.rs`, `crates/liquidfun/src/particle.rs`, `crates/liquidfun/src/particle/group.rs`, `crates/liquidfun/src/particle/topology.rs` - Narrow crate-private reservation, consuming association, sampler, and bounded-topology integration seams.
- `crates/liquidfun/tests/object_model.rs`, `crates/liquidfun/tests/particle_identity.rs`, `crates/liquidfun/tests/particle_objects.rs`, `crates/liquidfun/tests/particle_views.rs` - Existing behavior migrated from the removed placeholder shell API to complete recipe-created groups.

## Decisions Made

- Diagnostic IDs are reserved arithmetically from the immutable world counter and assigned deterministically: a New group consumes its shell identity before source-ordered particle diagnostics, while Append consumes particle diagnostics only.
- Contact refresh and pair/triad generation run against candidate storage, so topology errors cannot leak rows, contacts, caches, lifecycle state, group identities, or diagnostic advancement.
- The public view computes pinned float32 group statistics from current storage on borrow rather than duplicating mutable statistics authority in the world shell.
- A recipe association is installed only for `New`; pinned `AppendTo` semantics discard the hidden temporary group's association while retaining the target group and its application-owned association.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added narrow private transaction integration seams**

- **Found during:** Task 1 (Implement atomic create and append workflows)
- **Issue:** The planned world transaction could not access the private sampler, bounded topology limits, storage capacity, association reservation, or consuming recipe association without widening carefully selected crate-internal seams.
- **Fix:** Added crate-private re-exports/accessors and the storage-owned `GroupPlan` candidate; no storage lanes, dense rows, topology indices, or mutable metadata became public.
- **Files modified:** `crates/liquidfun/src/association.rs`, `crates/liquidfun/src/particle.rs`, `crates/liquidfun/src/particle/group.rs`, `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/mutation.rs`, `crates/liquidfun/src/particle/topology.rs`
- **Verification:** Strict Clippy, all-target build, focused group/lifecycle/buffer suites, and the full all-feature suite pass.
- **Committed in:** `82f1acc`

**2. [Rule 3 - Blocking] Migrated callers of the removed placeholder group API**

- **Found during:** Task 1 all-target compile
- **Issue:** Existing unit and integration tests created an empty identity shell with the old one-argument placeholder API, which no longer exists once group creation requires a complete recipe.
- **Fix:** Migrated each caller to a recipe-created initial member and preserved the original identity, destruction-order, ownership, view, and teardown assertions without adding a competing empty-shell API.
- **Files modified:** `crates/liquidfun/src/world/object.rs`, `crates/liquidfun/tests/object_model.rs`, `crates/liquidfun/tests/particle_identity.rs`, `crates/liquidfun/tests/particle_objects.rs`, `crates/liquidfun/tests/particle_views.rs`
- **Verification:** `cargo check -p liquidfun --all-targets --all-features` and the complete all-feature test suite pass.
- **Committed in:** `82f1acc`

**Total deviations:** 2 auto-fixed (2 Rule 3 blocking issues).
**Impact on plan:** Both changes were required to connect the planned private foundations and keep the workspace compiling; no alternative public mutation path or unrelated behavior was added.

## Issues Encountered

- The first focused compile exposed intentionally private module boundaries between world, sampler, storage, and topology; the final solution widened only crate-private types and methods needed by the transaction.
- Strict Clippy required an explicit source-parity justification for the pinned integer-count-to-`f32` statistics conversion and a cast-free topology-failure test fixture.
- Shared-host Cargo locks and executable startup contention made the exact full gate slow. The required commands were not changed or interrupted and all returned successfully.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later group mutation and solver plans can consume stable world group IDs and complete storage-backed views without duplicating physics authority.
- New and append creation now provide atomic public entry points for differential and compatibility evidence.
- No unsafe code, runtime C++, network endpoint, file-access trust boundary, dependency, or schema change was introduced.
- No blockers remain.

## Self-Check: PASSED

- Created integration evidence and all modified implementation files exist.
- Task commit `82f1acc` exists on the current branch.
- The required `plan_group`/`commit_group` key link exists from the world workflow to the storage mutation candidate.
- Stub scan found no goal-blocking TODO, FIXME, placeholder, coming-soon, or unavailable paths.
- Threat-surface scan found no unplanned trust-boundary changes beyond the plan's public group creation/view boundary.
- Focused group, buffer, lifecycle, and lifetime suites pass, and the ordered Rust gate passes through all 19 doctests.

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "05"
subsystem: particle-inspection-mutation
tags: [rust, particles, borrow-scoped-views, stable-identity, transactional-editing]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "03"
    provides: system-owned particle storage and stable public particle identity
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "04"
    provides: authoritative owned lanes and allocation-preserving permutations
provides:
  - one borrow-scoped aggregate view over every Phase 9 semantic particle lane
  - stable-ID derived contact, body-contact, pair, triad, and expiration-order inspection
  - validate-edit-repair particle transactions with panic-safe copied candidates
affects: [09-06, 09-07, 09-08, 09-09, phase-10]
tech-stack:
  added: []
  patterns: [semantic view over private rows, copied edit candidate, synchronous derived-state repair]
key-files:
  created:
    - crates/liquidfun/src/particle/view.rs
    - crates/liquidfun/src/particle/editor.rs
    - crates/liquidfun/src/particle/storage/editor_tests.rs
    - crates/liquidfun/tests/particle_views.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/lanes.rs
    - crates/liquidfun/src/world/particle_object.rs
key-decisions:
  - "Resolve application-owned particle user data through stable ParticleId keys instead of duplicating or type-erasing values in World storage."
  - "Prepare position and velocity edits in a closure-scoped copied candidate, then commit only after successful closure return."
  - "A changed position rebuilds the proxy lane and clears weights, stuck state, contacts, body contacts, pairs, and triads before returning."
patterns-established:
  - "Public aggregate views align row-owned slices with stable IDs and translate every derived row reference before exposure."
  - "No mutable lane escapes: typed direct setters delegate to the same validate-edit-repair transaction as closure-scoped edits."
requirements-completed: [API-09, PART-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T06:29:31Z
duration: 17 min
completed: 2026-07-15
---

# Phase 9 Plan 05: Safe Particle Views and Scoped Editing Summary

**Added stable-identity aggregate particle inspection and copied-candidate mutation transactions that synchronously repair spatially derived state.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-07-15T06:12:58Z
- **Completed:** 2026-07-15T06:29:31Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Exposed positions, velocities, optional colors, weights, flags, stable group identities, application-owned associations, contacts, body contacts, pairs, triads, and optional expiration order through one immutable `ParticleSystemView` borrow.
- Translated every private derived row reference to stable particle, body, fixture, group, and system identities without exposing dense indices, capacities, scratch storage, or mutable slices.
- Added checked direct setters and a closure-scoped `ParticleEditor`; rejected candidates and panicking closures perform no storage mutation.
- Made position edits synchronously rebuild proxies and clear every currently inventoried contact- or spatially-derived lane, with focused internal evidence for non-empty derived state.

## TDD Evidence

### Task 1: Build the aggregate semantic view

- **RED:** `cargo test -p liquidfun --test particle_views aggregate_view` failed with E0599 at both aggregate tests because `World::particle_system_view` did not exist.
- **GREEN:** Both focused aggregate tests passed after adding the borrow-scoped view, stable-ID record wrappers, application association resolution, optional-lane coverage, and compaction-stability evidence.

### Task 2: Add scoped mutation with derived-state repair

- **RED:** `cargo test -p liquidfun --test particle_views` failed with unresolved `ParticleEditError` and three E0599 errors because `World::edit_particle` did not exist.
- **GREEN:** All five integration tests, the focused non-empty derived-lane repair unit test, and all compile-fail doctests passed after adding copied-candidate editing and synchronous repair.

## Task Commits

Each task was committed atomically:

1. **Build the aggregate semantic view** - `e55c089` (feat)
1. **Add scoped mutation with derived-state repair** - `5fe0585` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/view.rs` - Borrow-scoped semantic lanes and stable-ID derived-record wrappers.
- `crates/liquidfun/src/particle/editor.rs` - Checked, lifetime-scoped copied edit candidates and typed errors.
- `crates/liquidfun/src/particle/storage.rs` - Narrow immutable lane adapters and the proxy/contact repair transaction.
- `crates/liquidfun/src/particle/storage/lanes.rs` - Particle-module-only visibility for derived records translated by the view boundary.
- `crates/liquidfun/src/particle/storage/editor_tests.rs` - Non-empty proxy, contact, topology, weight, and stuck-state repair regression.
- `crates/liquidfun/src/world/particle_object.rs` - Public view, editor, and typed setter entrypoints over the authoritative storage owner.
- `crates/liquidfun/src/particle.rs` - Public child-module declarations and curated re-exports.
- `crates/liquidfun/src/lib.rs` - Crate-root re-exports for view/editor consumer reachability.
- `crates/liquidfun/tests/particle_views.rs` - Black-box aggregate, compaction, validation, panic, and coherence coverage.

## Decisions Made

- Application associations remain in `AssociationMap<ParticleId, T>`. The view resolves them in stable particle order rather than publishing the unused internal architecture-spike key lane or introducing type erasure.
- Editors operate on copied position and velocity values. The stable handle and current state validate before invoking the closure, and the storage commit occurs only after a successful return.
- Position changes invalidate spatial observations conservatively: proxies are rebuilt for every current row, while weights, stuck counters/candidates, particle contacts, body contacts, pairs, and triads are cleared for later source-timed regeneration.

## Deviations from Plan

### Auto-fixed Supporting Files

**1. [Rule 3 - Blocking] Added narrow storage inspection adapters**

- **Found during:** Task 1
- **Issue:** The public view could not translate private row references or borrow authoritative lanes from a sibling module without a narrow internal boundary.
- **Fix:** Added particle-module-only accessors and visibility for the exact derived record types consumed by `view.rs`.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/lanes.rs`
- **Verification:** Aggregate compaction tests, warning-denied Clippy, all-target build, and all-feature tests pass.
- **Committed in:** `e55c089`

**2. [Rule 3 - Blocking] Connected view and editor entrypoints to World ownership**

- **Found during:** Tasks 1 and 2
- **Issue:** Only the world object module can validate a live system/particle and borrow or mutate its authoritative storage owner.
- **Fix:** Added view, editor, and typed setter methods at the established particle-object boundary.
- **Files modified:** `crates/liquidfun/src/world/particle_object.rs`
- **Verification:** Wrong/rejected candidates remain effect-free, views hold an immutable world borrow, and panic recovery tests pass.
- **Committed in:** `e55c089`, `5fe0585`

**3. [Rule 2 - Missing Critical] Added direct non-empty derived-state repair evidence**

- **Found during:** Task 2 verification
- **Issue:** Public Phase 9 paths do not generate contacts or pair/triad topology yet, so black-box empty-lane assertions could not prove repair of populated internal lanes.
- **Fix:** Added a focused storage-child unit test that seeds every affected derived category and proves position editing rebuilds or clears it.
- **File:** `crates/liquidfun/src/particle/storage/editor_tests.rs`
- **Verification:** `position_edit_rebuilds_or_clears_every_spatially_derived_lane` passes under the all-feature suite.
- **Committed in:** `5fe0585`

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical). **Impact:** The extra files are narrow adapters and verification at the existing storage/world boundaries; no raw mutable lane, unsafe code, contact generation, group topology, solver pass, or other Phase 10 behavior was added.

## Issues Encountered

- The first Task 1 warning-denied gate requested `#[must_use]` on iterator-returning view methods; the annotations were added before rerunning the full ordered gate.
- The first Task 2 warning-denied gate requested an elided implementation lifetime and `# Errors` rustdoc on editor setters; both were corrected before the ordered gate passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 09-06 and 09-07 can populate lifetime/contact lanes while retaining the same public aggregate view and stable-ID translation boundary.
- Later source-timed regeneration can replace the conservative clear/rebuild operation without changing the editor API or permitting stale state to escape.
- Phase 10 group construction, pair/triad generation, and particle solver behavior remain explicitly unimplemented.

## Self-Check: PASSED

- Both task commits are present and all 9 source/test files exist.
- `cargo check -p liquidfun --all-features` passes through the warning-denied Clippy and all-target build gates.
- The focused aggregate tests, complete particle-view/editor suite, non-empty derived-state repair test, and compile-time rejection doctests pass.
- The exact ordered `cargo fmt --all`, warning-denied Clippy, all-target/all-feature build, and all-feature test gates pass before each task commit.
- Base-to-head review finds no unsafe code, raw mutable lane, dense-index export, contact/group generation, or particle solver behavior.
- `.planning/STATE.md` and `.planning/ROADMAP.md` are unchanged.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*

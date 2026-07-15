---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "09"
subsystem: particle-rigid-coupling
tags: [rust, particles, fixture-contacts, pressure, damping, transactional-step]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "07"
    provides: transactional particle storage, lifecycle maintenance, and stable identities
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "08"
    provides: source-ordered particle contacts and listener effects
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "15"
    provides: independent strict-contact tie witness
provides:
  - stable-ID fixture-particle contacts with source fields, strict pruning, stuck tracking, and ordered effects
  - borrowed flag-gated particle-pair and fixture-particle decisions in the shared step hook
  - transactional source-timed particle contact prefix with body-only pressure and damping reaction
affects: [09-13, phase-10, particle-oracle]
tech-stack:
  added: []
  patterns: [candidate-arena transaction, borrowed contact decision view, source-grouped f32 kernel]
key-files:
  created:
    - crates/liquidfun/src/particle/body_contact.rs
    - crates/liquidfun/src/world/particle_coupling.rs
    - crates/liquidfun/tests/particle_body_contacts.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/world/config.rs
    - crates/liquidfun/src/world/step.rs
key-decisions:
  - "Default particle sub-iterations to one and expose a checked additive builder so existing step configuration remains source-compatible."
  - "Clone body and particle-system arenas for the prefix, then commit both only after every system and hook effect succeeds."
  - "Preserve fixture newest-first adjacency and stable equal-weight ordering before strict pruning to match the independent witness."
patterns-established:
  - "Particle contact effects enter the existing lifecycle journal at their source update point rather than being reconstructed from final sets."
  - "Phase 9 coupling reproduces only the body-contact pressure and damping branches; all particle-particle and later solver passes remain deferred."
requirements-completed: [PART-07, PART-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T08:07:15Z
duration: 28 min
completed: 2026-07-15
---

# Phase 9 Plan 9: Fixture Contacts and Rigid Coupling Summary

**Stable fixture-particle contacts now run in the source-timed step prefix with strict pruning, shared hook effects, and transactional body-only pressure and damping reaction.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-07-15T07:39:34Z
- **Completed:** 2026-07-15T08:07:15Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- Generated stable body, fixture, and particle contact records with pinned weight, normal, effective mass, source order, strict pruning, stuck candidates, and flag-gated listener/filter behavior.
- Extended the single synchronous step hook with borrow-scoped particle-pair and fixture-particle views and appended their begin/end occurrences directly to the shared lifecycle journal.
- Executed newest-first non-paused systems for the configured particle sub-iterations, applying source-grouped body-contact pressure and damping to particle velocity and body linear/angular velocity without integrating particle positions or entering Phase 10 solvers.
- Preserved rollback and panic poisoning by preparing body and particle-system candidate arenas before committing either authoritative state.

## Task Commits

1. **Task 1: Generate and prune fixture/body contacts** - `00944b4` (feat)
2. **Task 2: Wire the single hook, shared journal, and scoped rigid reaction** - `64b052c` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/body_contact.rs` - Fixture query records, stable contacts, strict pruning, and source-ordered listener effects.
- `crates/liquidfun/src/world/particle_coupling.rs` - Transactional particle contact prefix plus body-only pressure and damping kernels.
- `crates/liquidfun/tests/particle_body_contacts.rs` - Contact fields, witness pruning, hooks, journal, stuck, ordering, pressure, damping, rollback, and panic regressions.
- `crates/liquidfun/src/particle/storage.rs` - Authoritative semantic particle/body contact replacement, weights, velocity mutation, and stuck lanes.
- `crates/liquidfun/src/particle/definition.rs` - Checked strict-contact and stuck-threshold system controls.
- `crates/liquidfun/src/particle/view.rs` - Public body contacts and stuck-candidate projection.
- `crates/liquidfun/src/world/config.rs` - Checked particle sub-iteration builder and default.
- `crates/liquidfun/src/world/step.rs` - Borrowed particle contact hook views, lifecycle variants, typed errors, and pinned prefix seam.
- `crates/liquidfun/src/particle.rs`, `crates/liquidfun/src/world.rs`, and `crates/liquidfun/src/lib.rs` - Child module and curated public export wiring.
- `crates/liquidfun/src/world/object.rs` and `crates/liquidfun/src/world/particle_object.rs` - Particle timestamp and source-order state exposed to the coupling shell.
- `crates/liquidfun/src/particle/contact.rs` - Internal stable-contact reconstruction for authoritative storage.

## Decisions Made

- A particle iteration count of one is the compatibility-preserving default; the checked builder uses the existing reviewed solver maximum, which is exactly representable as `f32` for substep scaling.
- Complete candidate arenas are simpler and safer than piecemeal rollback for cross-body, cross-system reaction and hook failure.
- Pressure and damping preserve the upstream statement and multiplication grouping, including reciprocal-of-rounded-timestep behavior and the explicit quadratic damping coefficient.
- Strict pruning retains four contacts because the pinned removal predicate increments after testing whether the prior count exceeds three.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Persisted body contacts, weights, and stuck state in authoritative particle storage**

- **Found during:** Task 1 (Generate and prune fixture/body contacts)
- **Issue:** The listed kernel and test files could generate semantic contacts but could not preserve them across steps or expose stuck candidates without storage, definition, view, and object wiring.
- **Fix:** Added transactional replacement and derived-weight recomputation, checked system controls, stable public projections, timestamps, and curated exports.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/definition.rs`, `crates/liquidfun/src/particle/view.rs`, `crates/liquidfun/src/world/object.rs`, `crates/liquidfun/src/world/particle_object.rs`, and export modules.
- **Verification:** Five focused contact RED/GREEN regressions and the complete all-feature suite pass.
- **Committed in:** `00944b4`

**2. [Rule 3 - Blocking] Added checked particle sub-iterations to step configuration**

- **Found during:** Task 2 (Wire the single hook, shared journal, and scoped rigid reaction)
- **Issue:** Existing step configuration had no particle iteration input, so the required multi-iteration source prefix could not be represented.
- **Fix:** Added a checked additive builder/getter with a default of one and reused the reviewed solver iteration bound.
- **Files modified:** `crates/liquidfun/src/world/config.rs`
- **Verification:** The newest-first, paused-system, three-sub-iteration regression and full configuration consumers pass.
- **Committed in:** `64b052c`

***

**Total deviations:** 2 auto-fixed (2 blocking).
**Impact on plan:** Both changes supplied state and configuration required by the planned behavior; no Phase 10 solver path or dependency was added.

## Issues Encountered

- The exact static pressure expectation initially assumed mathematical `60.0` inverse time. The pinned step uses the reciprocal of the rounded `f32` timestep; correcting the independent expectation resolved the two-ULP difference.
- The first full clippy pass identified bounded precision casts and unused receiver parameters. Explicit bounds rationale and associated helper functions resolved all warnings before the mandatory gate sequence was restarted.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 9 now has stable particle and fixture contacts, source-timed hook effects, and the scoped rigid reaction needed by remaining particle validation and orchestration work.
- Particle-particle pressure/damping, position integration, forces, material solvers, pair/triad generation, and all other Phase 10 passes remain explicitly excluded.

## Self-Check: PASSED

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*

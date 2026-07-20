---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "22"
subsystem: particle-solver-world-integration
tags: [rust, particles, solver-order, transactions, world-step, rollback, source-parity]

requires:
  - phase: 10-04
    provides: Closed O01-O05 and S01-S26 manifest with validated gates and order
  - phase: 10-10
    provides: Deferred group lifecycle and source-ordered compaction effects
  - phase: 10-15
    provides: Reactive topology and storage-owned group cache transactions
  - phase: 10-17
    provides: Preparation, contact, force, gravity, and weight solver passes
  - phase: 10-18
    provides: Material solver passes and complete velocity/color candidates
  - phase: 10-19
    provides: Pressure, damping, and body-coupling solver passes
  - phase: 10-20
    provides: Elastic, spring, velocity-limit, and topology constraint passes
  - phase: 10-21
    provides: Rigid damping/projection and staged boundary-tail candidates
provides:
  - One manifest-driven production dispatcher for O01-O05 and admitted S01-S26 passes
  - Atomic particle, group, body, contact-journal, and lifecycle rollback at the world-step boundary
  - Source-ordered filtered fixture collision hits and candidate body impulse application
  - Private exact pass-trace evidence plus public pause, empty, iteration, typed-error, and rollback witnesses
affects: [10-23, 10-24, 10-25, 10-26, particle-solvers, particle-world-coupling, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Closed manifest as the sole production pass-order authority
    - Functional numerical candidates adapted by one transactional world shell
    - Staged boundary tail committed only after S26 integration

key-files:
  created:
    - crates/liquidfun/src/particle/solver/order_tests.rs
    - crates/liquidfun/src/world/particle_coupling/body_coupling.rs
    - crates/liquidfun/src/world/particle_coupling/executor.rs
    - crates/liquidfun/src/world/particle_coupling/executor/boundary_runtime.rs
    - crates/liquidfun/tests/particle_solver_order.rs
  modified:
    - crates/liquidfun/src/particle/solver.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/world/particle_coupling.rs
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/tests/particle_body_contacts.rs

key-decisions:
  - "Execute the validated manifest directly through a crate-private ParticlePassExecutor so production and private trace tests share one order authority."
  - "Back up bodies, particle systems, and group arena before lifecycle work, run numerical passes on candidates, and restore every subsystem on any solver or journal error."
  - "Keep fixture collision admission in source particle/body/fixture/child order and invoke the existing fixture-particle filter only for particles carrying the filter flag."
  - "Split world integration, manifest dispatch, boundary staging, and candidate body coupling into cohesive modules while leaving numerical behavior in the existing deep solver modules."

patterns-established:
  - "Solver transaction: run outer lifecycle once, honor the pause terminator, repeat only admitted S01-S26 passes for each checked particle iteration, and publish candidates after every system succeeds."
  - "Boundary transaction: carry one BoundaryCandidate from barrier through collision, rigid projection, wall, and integration, then replace all aligned solver lanes together."

requirements-completed: [PART-09, PART-10, PART-11, PART-12, PART-13, TEST-01, TEST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T22:55:01Z

duration: 3h 50m
completed: 2026-07-20
---

# Phase 10 Plan 22: Complete Transactional Particle Solver Summary

**The world now executes the closed O01-O05/S01-S26 manifest as its only particle-step path, with exact sub-iteration order, source-timed callbacks, filtered collision inputs, candidate body reactions, and full rollback.**

## Performance

- **Duration:** 3h 50m
- **Started:** 2026-07-20T19:05:00Z
- **Completed:** 2026-07-20T22:55:01Z
- **Tasks:** 1
- **Files modified:** 24

## Accomplishments

- Replaced the Phase 9 contact prefix with one crate-private, manifest-driven solver that evaluates admitted O01-O05 once, returns after the outer phase for paused systems, and repeats only admitted S01-S26 passes for each configured particle iteration.
- Integrated every preparation, material, pressure, constraint, rigid, barrier, collision, wall, and integration kernel from Plans 10-17 through 10-21 through candidate-owned storage and body state.
- Preserved one atomic world outcome across lifecycle compaction, group arena state, particle storage, body impulses, source-timed contact journals, and solver failures.
- Added source-ordered real fixture ray hits with existing filter-hook admission and the exact iteration-zero previous-transform reconstruction.
- Added six private exact-order/multiplicity witnesses and five public-only semantic witnesses for empty, pause, one/two iterations, typed zero-iteration rejection, and rollback.
- Removed every production prefix symbol and confirmed no production zero-iteration O-only branch remains.

## Task Commits

Each task was committed atomically:

1. **Task 1: Assemble the exact full-solver transaction** - `9e329cc` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver.rs` - Manifest-driven executor contract and sole ordered solver loop.
- `crates/liquidfun/src/particle/solver/order_tests.rs` - Exact private pass order, cardinality, admission, pause, empty, and refresh witnesses.
- `crates/liquidfun/src/particle/storage.rs` - Validated complete solver-candidate replacement seam.
- `crates/liquidfun/src/world/particle_coupling.rs` - World transaction, contact preparation, and filtered collision-hit collection.
- `crates/liquidfun/src/world/particle_coupling/executor.rs` - Closed mapping from all manifest pass IDs to numerical kernels.
- `crates/liquidfun/src/world/particle_coupling/executor/boundary_runtime.rs` - Staged S22-S26 boundary candidate execution.
- `crates/liquidfun/src/world/particle_coupling/body_coupling.rs` - Candidate-body adapter for material and pressure passes.
- `crates/liquidfun/src/world/step.rs` - Single full particle-solver invocation in the existing panic/poison shell.
- `crates/liquidfun/tests/particle_solver_order.rs` - Public semantic iteration, pause, empty, error, and rollback evidence.
- `crates/liquidfun/tests/particle_body_contacts.rs` - Updated Phase 9 premise to assert final S26 integration.

## Decisions Made

- Used `ParticlePassExecutor` as the only bridge from the validated manifest to production behavior. The same function drives the private trace fake, preventing a second ordering implementation.
- Kept O01/O02 lifecycle work inside the world transaction before numerical candidate cloning, with complete bodies/systems/groups backups restoring every mutation when any later pass or journal reservation fails.
- Committed aligned particle positions, velocities, forces, group records, and pending-force state through one validated `ParticleStorage` replacement rather than exposing independent lane writes.
- Kept the S22-S26 candidate alive across conditional omissions, advancing only the stage required by the next admitted pass and committing once after integration.
- Preserved existing fixture/filter authority: collision hits follow stable particle/body/fixture/child traversal, skip sensors and inactive bodies, and borrow the established hook only under `FIXTURE_CONTACT_FILTER`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added minimal crate-private storage and kernel seams**

- **Found during:** Full graph assembly
- **Issue:** The world dispatcher could not construct and atomically publish complete candidates while storage lanes, group records, solver kernels, and boundary stages were visible only inside narrower particle submodules.
- **Fix:** Widened only the required internal items to `pub(crate)` and added one validated complete candidate replacement. No consumer API changed.
- **Files modified:** `crates/liquidfun/src/particle/storage*`, `crates/liquidfun/src/particle/solver/**`
- **Verification:** Warning-denied clippy, all-target build, 406 library tests, every integration suite, and 19 doctests passed.
- **Committed in:** `9e329cc`

**2. [Rule 3 - Code Shape] Split the world adapter into cohesive modules**

- **Found during:** Explicit simplification review
- **Issue:** The initial integrated `particle_coupling.rs` grew to 891 lines and combined transaction ownership, pass dispatch, boundary staging, and body coupling.
- **Fix:** Split it into a 257-line world shell, 431-line manifest executor, 162-line boundary runtime, and 79-line body adapter. Numerical kernels remain in their established deep modules.
- **Files modified:** `crates/liquidfun/src/world/particle_coupling.rs`, `crates/liquidfun/src/world/particle_coupling/**`
- **Verification:** Exact focused order tests and the complete mandatory Rust gate passed after the split.
- **Committed in:** `9e329cc`

**3. [Rule 1 - Behavioral] Updated the obsolete prefix-era integration assertion**

- **Found during:** Full solver integration verification
- **Issue:** A Phase 9 test asserted that pressure reacted without integrating particle position, which was correct only while the temporary prefix intentionally stopped before S26.
- **Fix:** Renamed the test and asserted positive x integration plus exact zero y while retaining the pressure/body-reaction checks.
- **Files modified:** `crates/liquidfun/tests/particle_body_contacts.rs`
- **Verification:** All 14 particle/body-contact witnesses and the full integration suite passed.
- **Committed in:** `9e329cc`

**Total deviations:** 3 auto-fixed (1 blocking, 1 code-shape, 1 behavioral premise).
**Impact on plan:** The changes were necessary private integration seams and test maintenance; scope and public API remained unchanged.

## Issues Encountered

- macOS code-signature scanning delayed first execution of many default-target test binaries. Processes were left uninterrupted; all exact gates completed successfully with no test or diagnostic failures.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-23 can close native compatibility coverage against one production solver path and reuse the exact trace/public rollback evidence.
- Plans 10-24 through 10-26 can extend protocol, native, and oracle scenarios without compensating for the removed Phase 9 prefix.
- No blockers remain.

## Self-Check: PASSED

- Confirmed implementation commit `9e329cc` exists and contains all 24 scoped implementation paths.
- Confirmed the exact plan verification passed: six `private_pass_trace_` unit tests and five public `particle_solver_order` integration tests.
- Confirmed `run_particle_contact_prefix`, `run_system_contact_prefix`, and `contact_prefix` have no production or test references.
- Confirmed no production zero-iteration O-only branch exists; `with_particle_iterations(0)` retains its typed rejection.
- Confirmed the implementation gate passed in exact order: format, warning-denied all-target/all-feature clippy, all-target/all-feature build, full all-feature tests, and 19 doctests.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

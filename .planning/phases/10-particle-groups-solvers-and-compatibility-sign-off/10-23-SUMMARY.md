---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "23"
subsystem: particle-compatibility-testing
tags: [rust, particles, solver-witnesses, property-testing, deterministic-replay, regression]

requires:
  - phase: 10-16
    provides: Complete safe public particle-group creation and mutation workflows
  - phase: 10-22
    provides: Manifest-driven transactional full particle solver and public semantic witnesses
provides:
  - Closed private witness registry for all 31 particle passes and all 23 supported public/internal flags
  - Public-only flag activation, interaction, and inherited rigid-world non-regression evidence
  - Versioned bounded public group operation sequences with every-operation invariants and exact replay
affects: [10-24, 10-25, 10-26, particle-compatibility, property-testing, differential-evidence]

tech-stack:
  added: []
  patterns:
    - Private closed coverage inventory bound to named executable witnesses
    - Versioned deterministic public-API state machines with explicit resource caps
    - Normalized semantic snapshots for exact replay and effect-free error checks

key-files:
  created:
    - crates/liquidfun/src/particle/solver/manifest/witness_registry.rs
    - crates/liquidfun/tests/particle_solver_flags.rs
    - crates/liquidfun/tests/particle_solver_baseline.rs
    - crates/liquidfun/tests/particle_group_properties.rs
    - crates/liquidfun/tests/particle_group_properties/model.rs
    - crates/liquidfun/tests/particle_group_properties/snapshot.rs
  modified:
    - crates/liquidfun/src/particle/solver/manifest.rs

key-decisions:
  - "Keep pass and flag completeness in private closed tables bound to named executable witnesses without exporting PassId or private traces."
  - "Drive adversarial group coverage through a versioned deterministic public-API model with explicit operation, group, and particle caps."
  - "Compare normalized semantic snapshots after every operation and require exact no-diff outcomes for typed rejected mutations."

patterns-established:
  - "Coverage closure: cardinality, uniqueness, known-key, and nonempty witness checks turn manifest coverage claims into executable inventory assertions."
  - "Bounded replay: generator version plus seed reproduces the complete operation sequence, outcomes, lifecycle reports, and semantic snapshots."

requirements-completed: [PART-09, PART-10, PART-11, PART-12, PART-13, TEST-01, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T00:09:00Z

duration: 1h 8m
completed: 2026-07-20
---

# Phase 10 Plan 23: Close Native Particle Compatibility Coverage Summary

**All 31 particle passes and 23 supported flags now have closed executable witnesses, while bounded public group state machines prove deterministic replay, invariant preservation, effect-free typed failures, and unchanged rigid-world behavior.**

## Performance

- **Duration:** 1h 8m
- **Started:** 2026-07-20T23:01:25Z
- **Completed:** 2026-07-21T00:09:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added a crate-private closed witness registry covering every O01-O05/S01-S26 pass and all 18 public particle, three public group, and two internal group flags, with exact cardinality, known-key, duplicate, and nonempty-witness assertions.
- Added public-only semantic flag tests and an inherited rigid-world comparison covering bodies, joints, callbacks, and step reports with Phase 10 behavior inactive.
- Added versioned, explicitly bounded public group operation sequences covering all required creation, mutation, lifecycle, reactive, stepping, compaction, and rejected cross-system operations.
- Asserted stable identity, lane alignment, finite state, group contiguity and association, topology integrity, lifecycle caps, exact same-seed replay, and typed-error no-diff after every operation.
- Checked in an ordinary minimized regression fixture that exercises the complete operation vocabulary independently of transient proptest shrink output.

## Task Commits

Each task was committed atomically:

1. **Task 1: Close the pass and flag witness matrix** - `8afd6a6` (test)
2. **Task 2: Exercise seeded public group workflows** - `15d63f7` (test)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/manifest.rs` - Registers the private witness-registry test module beside the closed manifest.
- `crates/liquidfun/src/particle/solver/manifest/witness_registry.rs` - Closed 31-pass and 23-flag coverage tables plus structural validation tests.
- `crates/liquidfun/tests/particle_solver_flags.rs` - Public semantic activation, interaction, and control witnesses for supported flags.
- `crates/liquidfun/tests/particle_solver_baseline.rs` - Exact rigid-world semantic comparison with an inactive far-away water particle.
- `crates/liquidfun/tests/particle_group_properties.rs` - Proptest entry points, exact replay assertion, and persisted complete-vocabulary fixture.
- `crates/liquidfun/tests/particle_group_properties/model.rs` - Versioned bounded generator and public-API operation executor.
- `crates/liquidfun/tests/particle_group_properties/snapshot.rs` - Normalized semantic snapshot construction and every-operation invariant checks.

## Decisions Made

- Kept the pass enum and witness registry crate-private. Integration tests validate consumer-visible flag semantics without gaining access to implementation traces or `PassId`.
- Required the generated prefix to contain every supported operation kind, then used seeded control values for the remaining bounded suffix. Every case therefore exercises the complete mutation vocabulary while retaining reproducible variation.
- Captured only public semantic state and stable identities. Dense indices, addresses, and other storage implementation details do not enter replay or regression evidence.
- Preflighted split component bounds and treated pending-delete particles through aligned public system views so the model checks valid public semantics without inventing internal exceptions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Code Shape] Split the property state machine into cohesive modules**

- **Found during:** Task 2 simplification review
- **Issue:** The initial single integration-test file reached 710 lines and crossed the managed code-shape review trigger while combining generation, execution, snapshots, and invariants.
- **Fix:** Split the suite into a small test entry point, a generator/executor model, and a semantic snapshot/invariant module without changing behavior or public scope.
- **Files modified:** `crates/liquidfun/tests/particle_group_properties.rs`, `crates/liquidfun/tests/particle_group_properties/model.rs`, `crates/liquidfun/tests/particle_group_properties/snapshot.rs`
- **Verification:** The exact 128-case property command and the mandatory warning-denied Rust gate passed after the split.
- **Committed in:** `15d63f7`

**Total deviations:** 1 auto-fixed (1 code-shape).
**Impact on plan:** The split reduced review surface and preserved the planned tests, bounds, and public-only behavior without scope growth.

## Issues Encountered

- Early generated sequences exposed that pending-delete particles must be snapshotted through aligned public system lanes and that split preflight must reserve the maximum possible connected components. Both model defects were corrected before the task commit, and the resulting minimized semantic sequence was retained as an ordinary regression fixture.
- Warning-denied clippy identified one needless owned recipe parameter after the module split. Borrowing the recipe removed the copy and the exact property suite was rerun before the fresh task gate.
- macOS code-signature scanning delayed initial test-binary launches. Processes were left uninterrupted; all focused and full gates completed successfully.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-24 can extend the strict rigid-world protocol against a closed native pass/flag inventory and protected Phase 9 baseline.
- Plans 10-25 and 10-26 can reuse deterministic public semantic scenarios without depending on private solver identities.
- No blockers remain.

## Self-Check: PASSED

- Confirmed task commits `8afd6a6` and `15d63f7` exist and contain only their scoped implementation/test paths.
- Confirmed the closed registry contains exactly 31 pass entries and 23 supported flag entries, with named executable control, activation, and interaction witnesses.
- Confirmed the exact pass/flag/baseline verification and exact 128-case property verification both pass.
- Confirmed both task commits were preceded by the mandatory Rust gate in exact order: format, warning-denied all-target/all-feature clippy, all-target/all-feature build, and full all-feature tests including 19 doctests.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

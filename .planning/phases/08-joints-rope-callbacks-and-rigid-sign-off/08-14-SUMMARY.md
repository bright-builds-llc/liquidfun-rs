---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "14"
subsystem: joint-solver-contract
tags: [rust, joints, solver, transactional-staging, gear]
requires:
  - phase: 08-13
    provides: complete Phase 8 native and pinned-oracle execution surfaces for remediation review
provides:
  - eleven typed activation-aware joint-family staging variants
  - source-semantic ordinary A/B and gear A/B/C/D solver lanes with checked optional indices
  - complete JointRuntime candidates committed only through the existing validated world transaction
affects: [08-15, 08-16, 08-17, 08-18, joint-parity, rigid-sign-off]
tech-stack:
  added: []
  patterns: [typed family activation, candidate-first runtime staging, explicit legacy compatibility]
key-files:
  created: []
  modified:
    - crates/liquidfun/src/world/joint/solver.rs
    - crates/liquidfun/src/world/island.rs
    - crates/liquidfun/src/world/contact_solver.rs
    - crates/liquidfun/src/world/object.rs
key-decisions:
  - "Only LegacyUnmigrated owns CommonConstraint, so an Activated candidate cannot reach the compatibility solver through the family activation type."
  - "Island staging resolves semantic body identities and optional solver indices before family construction, including source-derived gear A/B/C/D lanes in encounter order."
  - "Activated families finalize a complete JointRuntime after position solving; world commit installs it only after every body, contact, proxy, island, and joint candidate has validated."
patterns-established:
  - "Incremental solver migration: later plans activate one closed family variant without reopening island orchestration or world commit semantics."
  - "Compatibility isolation: unmigrated families preserve the current generic behavior behind the explicitly named legacy_unmigrated path."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T03:38:04Z
duration: 25min
completed: 2026-07-14
---

# Phase 8 Plan 14: Shared Live-Solver Staging and Dispatch Contract Summary

**All eleven joint families now enter one source-ordered transactional staging boundary with typed runtimes, exact body lanes, and an explicit compatibility state ready for incremental pinned-solver activation.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-14T03:13:00Z
- **Completed:** 2026-07-14T03:38:04Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Replaced the borrowed generic joint input with copied typed definitions, complete runtime candidates, semantic body identities, and checked optional island indices.
- Added closed variants for revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope, and motor families, each carrying an explicit `Activated` or `LegacyUnmigrated` state.
- Represented ordinary A/B and gear A/B/C/D lanes directly, deriving gear lanes from both live source joints while preserving island encounter order.
- Added exhaustive initialize, warm-start, velocity, position, and finalize dispatch seams without activating or migrating any family solver in this plan.
- Threaded optional complete runtime candidates through island and world solutions, finalized them after position solving, and installed them only inside the existing no-fail post-validation commit.
- Preserved generic cache behavior for all unmigrated families and cleared those compatibility caches when a complete activated runtime is installed.
- Added six focused Arrange/Act/Assert regressions for lane identity, missing-lane rejection, explicit compatibility routing, activated runtime finalization, source order, and gear A/B/C/D order.

## Task Commits

Each task was committed atomically:

1. **Task 08-14-01: Build the exhaustive candidate-runtime solver contract** - `a2f5d5b` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/solver.rs` - Owns typed family candidates, activation states, exact solver lanes, exhaustive dispatch, compatibility behavior, and contract regressions.
- `crates/liquidfun/src/world/island.rs` - Builds source-ordered copied ordinary and four-body gear inputs with semantic IDs and optional solver indices.
- `crates/liquidfun/src/world/contact_solver.rs` - Runs mutable position dispatch and finalizes complete runtime candidates only after position solving and numeric validation.
- `crates/liquidfun/src/world/object.rs` - Installs a complete staged runtime or the temporary legacy cache in the existing post-validation world commit.

## Decisions Made

- Kept `legacy_unmigrated` as a structural activation variant rather than a boolean branch. The compatibility solver exists only inside that variant, while activated candidates contain no `CommonConstraint`.
- Kept family candidates owned and copied. This makes a candidate independent of live arena storage and allows late failures to discard every runtime change with the rest of the world-step candidate.
- Kept body identity alongside optional solver indices. Family plans can use semantic A/B/C/D roles without inferring identity from scratch positions, while missing or out-of-range lanes fail before dispatch.
- Finalized joint runtime candidates after position iterations. Later family position solvers can update their complete runtime before it enters the world candidate.

## Deviations from Plan

None - the plan executed exactly as written. No family solver was migrated or activated.

## Issues Encountered

- The RED tests were executed and observed failing before implementation, but were not committed because repository policy requires the complete ordered Rust gate, including passing tests, before every commit. The GREEN implementation and tests were committed together after the gate passed.
- The closed contract makes `world/joint/solver.rs` larger than the general file-size refactor trigger. A new file would have exceeded this remediation plan's explicit allowlist, so the cohesive contract remains in the existing module for Plans 08-15 through 08-18 to consume and then simplify once the temporary compatibility path is removed.

## Verification

- RED: three initial contract tests failed on the absent lane, activation, and copied-input APIs before implementation.
- Focused solver contract suite: 6 passed.
- `cargo test -p liquidfun --test joint_island_solver --all-features`: 3 passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 182 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-15 can activate revolute and prismatic candidates behind the closed dispatch without changing island input or world commit semantics.
- Plans 08-16 and 08-17 can activate scalar, soft, capped, and four-body gear families using the already-resolved typed lanes.
- Plan 08-18 can remove `LegacyUnmigrated` only after all eleven variants are activated and the exhaustive family suite proves no compatibility route remains.

## Self-Check: PASSED

- All key modified files and this summary exist.
- Commit `a2f5d5b` records Task 08-14-01.
- Phase lifecycle validation passes with required plans under `8-2026-07-13T21-26-30`.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

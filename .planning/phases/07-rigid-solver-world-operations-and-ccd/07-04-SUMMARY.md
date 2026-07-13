---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "04"
subsystem: rigid-island-solver
tags: [rust, rigid-body, island-solver, warm-starting, transactional-commit]
requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "02"
    provides: Checked step timing, warm-start ratio, and successful-step finalization
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "03"
    provides: Source-ordered bounded island scratch and immutable graph preflight
provides:
  - Source-ordered contact constraints over general contact-only islands
  - Exact warm-start scaling, cold-start zeroing, force integration, damping, velocity solving, and position solving
  - Transactional all-island body, impulse, proxy, contact-discovery, and timing commit
  - Bounded failure evidence proving late island and proxy preparation rejection have no solver effects
affects: [07-05, sleeping, world-operations, ccd, rigid-differential]
tech-stack:
  added: []
  patterns: [island-indexed solver lanes, immutable world-step candidate, prevalidate-then-commit]
key-files:
  created:
    - crates/liquidfun/tests/rigid_island_solver.rs
  modified:
    - crates/liquidfun/src/world/contact_solver.rs
    - crates/liquidfun/src/world/island.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/contact.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/src/rigid_differential.rs
    - crates/liquidfun/tests/rigid_contact_solver.rs
key-decisions:
  - "Index every contact constraint into copied island body lanes and retain island contact, manager occurrence, and manifold point order throughout solving and impulse storage."
  - "Apply cached impulses scaled by the exact time-step ratio only when warm starting is enabled; cold starts zero current impulses but still persist newly solved values."
  - "Prepare every solved body/sweep, contact impulse/report, proxy synchronization, and timing update before one no-fail world commit, then discover pairs after newest-first non-static proxy synchronization."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T03:03:38Z
duration: 40 min
completed: 2026-07-12
---

# Phase 7 Plan 04: Transactional Discrete Island Solver Summary

**General contact-only islands now execute the pinned scalar constraint sequence and commit all solved motion, impulses, proxy updates, contact discovery, and timing as one coherent world transition.**

## Performance

- **Duration:** 40 min
- **Started:** 2026-07-13T02:23:40Z
- **Completed:** 2026-07-13T03:03:38Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Replaced the fixed two-body witness with island-indexed position and velocity constraints that preserve island contact order, contact-manager occurrence order, and manifold point order.
- Implemented source-grouped dynamic force/gravity integration and Padé damping before constraint initialization while retaining user-driven kinematic velocity.
- Preserved tangent-before-normal velocity solving, the four-case two-point block solve, solved impulse storage, clamped position integration, and position iterations.
- Implemented exact warm-start behavior: enabled solves scale cached lanes by the current time-step ratio; disabled solves begin at zero and still store newly solved impulses.
- Added one immutable `WorldStepCandidate` that prepares all solved bodies and sweep endpoints, staged reports and impulses, non-static proxy synchronizations, and inverse-timestep state before mutation.
- Committed body state and impulses only after all islands and proxy bounds validate, synchronized fixtures in newest-first body/fixture order, and discovered newly overlapping contacts afterward.
- Proved late-island and proxy-preparation failures preserve every body, impulse lane, and existing contact topology.

## Task Commits

Each task was committed atomically after its exact ordered Rust gate passed:

1. **Task 1: Generalize contact constraints over island scratch** - `fcf8d71` (`feat`)
2. **Task 2: Stage all islands and commit atomically** - `f7c8504` (`feat`)

## Files Created/Modified

- `crates/liquidfun/tests/rigid_island_solver.rs` - General island, ordering, material, warm-start, integration, transactional success, and failure regression coverage.
- `crates/liquidfun/src/world/contact_solver.rs` - Island-indexed constraint construction, integration, warm start, velocity/block solving, impulse storage, and position solving.
- `crates/liquidfun/src/world/island.rs` - Per-island input assembly, solved body candidates, staged impulses, and bounded late-island rejection.
- `crates/liquidfun/src/world/object.rs` - Immutable `WorldStepCandidate` preparation and one ordered commit boundary.
- `crates/liquidfun/src/world/body.rs` - Checked solved-state candidate construction that preserves prior/current sweep endpoints.
- `crates/liquidfun/src/world/contact.rs` - Non-mutating staged contact snapshots with candidate impulses.
- `crates/liquidfun/src/world/contact_manager.rs` - Staged solve-report preview and final impulse installation.
- `crates/liquidfun/src/world/step.rs` - Full timing handoff, typed proxy-bound failure, and feature-gated transactional failure evidence.
- `crates/liquidfun/src/rigid_differential.rs` - Owned bounded late-island and fixture-proxy failure injection contract.
- `crates/liquidfun/tests/rigid_contact_solver.rs` - Phase 6 expectations updated to the generalized supported topology.

## Decisions Made

- Kept constraint scratch indexed into copied island lanes instead of borrowing mutable world storage, so every numeric lane validates before persistent state can change.
- Kept the pinned source sequence explicit inside one cohesive solver kernel. Splitting the tangent, normal block solve, impulse storage, and position passes would make cross-pass ordering harder to audit without reducing the algorithm.
- Used linear solution-to-body lookup in explicit newest-first body order rather than a hash map or sorting step; reviewed island limits bound the work and deterministic source order stays visible.
- Kept joints out of constraint construction. The reserved Phase 8 joint lane remains empty and cannot affect Phase 7 contact-only results.
- Committed prepared timing inside the same candidate transition; the existing successful-step finalizer repeats the positive inverse-timestep assignment idempotently and remains authoritative for zero-duration and future CCD completions.

## Test Evidence

- Task 1 RED exposed the fixed topology and missing force/gravity/damping behavior before implementation.
- Task 2 RED failed to compile on the intentionally absent bounded failure-injection and typed proxy-bound contracts.
- Focused plan verification passed all 11 `rigid_island_solver` tests and all 8 `rigid_contact_solver` tests.
- Strict package Clippy passed with all targets, all features, and warning denial.
- `git diff --check HEAD~2..HEAD` passed for both implementation commits.
- The exact ordered Rust gate passed before each task commit:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- The final Task 2 gate passed 143 library unit tests, every integration target, and 12 doctests with zero failures.

## Simplification Review

- One parameter value now carries per-step island settings into the solver entrypoint, keeping the interface below the warning-denied argument limit without hiding source-order operations.
- One world candidate owns all fallible preparation; one commit method contains only prevalidated lookups and infallible installations.
- Existing deterministic vectors provide all ordering. No hash collection, sorting pass, dependency, unsafe block, or persistent solver scratch was added.
- The source-mapped contact solver remains a large cohesive numerical kernel because its pass order and shared fixed-capacity scratch are the compatibility boundary; splitting it in this plan would increase indirection without reducing behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Added a bounded feature-gated failure-injection seam**

- **Found during:** Task 2
- **Issue:** Black-box integration tests could not deterministically force rejection after one staged island or during one fixture proxy preparation, so the required no-partial-mutation evidence was not reproducible.
- **Fix:** Added owned semantic failure requests under the existing non-default `differential-internals` boundary and mapped them to ordinary typed step failures before world mutation.
- **Files modified:** `crates/liquidfun/src/rigid_differential.rs`, `crates/liquidfun/src/world/island.rs`, `crates/liquidfun/src/world/object.rs`, `crates/liquidfun/src/world/step.rs`
- **Verification:** Both injected failures preserve body and contact diagnostics; the full ordered Rust gate passes.
- **Committed in:** `f7c8504`

### Process adjustment: RED evidence was not committed

- The repository requires the complete ordered Rust gate before every commit, so deliberately failing RED states were run but not committed. Each task produced one verified GREEN commit after preserving the required RED failure evidence.

***

**Total deviations:** 1 implementation auto-fix and 1 commit-process adjustment.
**Impact on plan:** The evidence seam is confined to the existing unpublished feature and does not widen ordinary production APIs or solver authority.

## Issues Encountered

- The first Task 2 Clippy gate identified an eight-argument `solve_islands` entrypoint. A cohesive `IslandSolveParameters` value resolved the warning before the gate was restarted and passed.
- The first full Task 1 test run was slow under shared process load but completed with all targets green; no duplicate test process was used for the final Task 2 gate.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-05 can compute sleeping over the same candidate body states before the established atomic commit boundary.
- Later world-operation and CCD plans can reuse checked sweep endpoints and prepared proxy synchronization without exposing partial motion.
- Phase 8 can add joint constraints to the reserved island joint lane while retaining contact manager/manifold order and the all-island commit contract.

## Self-Check: PASSED

- Task commits `fcf8d71` and `f7c8504` exist, and the declared `rigid_island_solver.rs` file exists.
- All ten implementation/test files in the two-commit plan diff are represented above; focused and full verification passes.
- Stub and threat scans found no placeholder implementation, unsafe code, unordered solver traversal, hidden error suppression, or partial world mutation after a fallible preparation step.
- The pre-existing `.planning/config.json` change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-12*

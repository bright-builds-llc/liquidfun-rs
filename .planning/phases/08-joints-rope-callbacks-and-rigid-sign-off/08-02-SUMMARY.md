---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "02"
subsystem: rigid-joints
tags: [rust, revolute, prismatic, limits, motors, solver]
requires:
  - phase: 08-01
    provides: checked joint identity, lifecycle, tagged definitions, and snapshots
provides:
  - checked revolute and prismatic frames, limits, motors, coordinates, anchors, and reactions
  - source-ordered private warm-start, motor-cap, and 2x2/3x3 constraint scratch
  - owned type-specific runtime snapshots for future gear and island integration
affects: [08-05, 08-06, 08-10, 08-12]
tech-stack:
  added: []
  patterns: [candidate-first joint mutation, exact changed-only wake branches, closed runtime dispatch]
key-files:
  created:
    - crates/liquidfun/src/world/joint/revolute.rs
    - crates/liquidfun/src/world/joint/prismatic.rs
    - crates/liquidfun/tests/joint_revolute.rs
    - crates/liquidfun/tests/joint_prismatic.rs
  modified:
    - crates/liquidfun/src/joint/definition.rs
    - crates/liquidfun/src/joint/snapshot.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/joint.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Preserve exact source equality for changed-only setters rather than introducing an epsilon policy."
  - "Keep solver impulses private while exposing owned semantic coordinates, branch state, anchors, and explicit-inverse-timestep reactions."
patterns-established:
  - "Family mutation: validate complete candidate, apply exact wake branch, then replace definition/cache state once."
  - "Solver scratch: restore the complete prior cache when derived arithmetic becomes non-finite."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T23:07:11Z
duration: 31 min
completed: 2026-07-13
---

# Phase 8 Plan 02: Revolute and Prismatic Joints Summary

**Checked revolute and prismatic contracts now preserve pinned frames, coordinates, limit branches, motor caps, wake behavior, reaction semantics, and transactional solver caches.**

## Performance

- **Duration:** 31 min
- **Started:** 2026-07-13T22:36:00Z
- **Completed:** 2026-07-13T23:07:11Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Replaced placeholder revolute and prismatic definitions with checked local frames, normalized prismatic axes, ordered limits, and non-negative motor caps.
- Added owned type-specific snapshots and checked World queries for angles, translations, speeds, anchors, reactions, motor torque, and motor force.
- Preserved pinned setter asymmetry: limit changes wake only on exact value changes and clear limit caches, while motor setters wake unconditionally after full validation.
- Added private cold/warm initialization, motor clamping, limit classification, and transactional 2x2/3x3 velocity-block scratch for later mixed-island integration.
- Added ten focused integration tests plus six private solver tests covering invalid definitions, four limit states, coordinate/speed semantics, wake branches, cold caches, warm scaling, motor caps, block clamps, and derived-overflow restoration.

## Task Commits

The two tightly coupled family tasks share one atomic commit because they deepen the same closed definition, snapshot, and runtime-dispatch contract:

1. **Implement revolute definitions, mutations, inspection, reactions, and solver scratch** - `c07abda` (feat)
1. **Implement prismatic definitions, mutations, inspection, reactions, and solver scratch** - `c07abda` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/revolute.rs` - Revolute runtime, queries, mutations, source limit classification, and solver scratch.
- `crates/liquidfun/src/world/joint/prismatic.rs` - Prismatic runtime, normalized-axis coordinates, queries, mutations, and solver scratch.
- `crates/liquidfun/src/joint/definition.rs` - Complete checked configuration for the two families.
- `crates/liquidfun/src/joint/snapshot.rs` - Owned tagged family runtime snapshots.
- `crates/liquidfun/src/world/joint.rs` - Closed runtime dispatch, family reactions, and candidate-first mutation helpers.
- `crates/liquidfun/tests/joint_revolute.rs` - Focused public revolute contract tests.
- `crates/liquidfun/tests/joint_prismatic.rs` - Focused public prismatic contract tests.

## Decisions Made

- Exact `f32` equality is intentional for source setter change detection; no joint-wide epsilon was introduced.
- The prismatic axis is normalized once at checked definition construction and rejects non-finite or zero-length inputs.
- Solver cache state remains private and transactional. Public observations are semantic owned values, including an explicit limit state and inverse-timestep reaction queries.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Deepened the shared definition and snapshot contract**

- **Found during:** Tasks 08-02-01 and 08-02-02
- **Issue:** Plan 08-01 deliberately established only minimal two-body definitions and pending snapshots, so the family modules could not expose complete checked configuration or semantic runtime state in isolation.
- **Fix:** Extended the existing closed shared contracts without changing handle identity, arena authority, adjacency, or later-family variants.
- **Files modified:** `crates/liquidfun/src/joint/definition.rs`, `crates/liquidfun/src/joint/snapshot.rs`, `crates/liquidfun/src/world/joint.rs`, curated exports.
- **Verification:** Both focused targets and the full ordered Rust gate pass.
- **Committed in:** `c07abda`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Necessary shared-contract completion only; no new dependency, unsafe code, unordered traversal, or public storage detail was introduced.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Gear planning can consume stable revolute angle and prismatic translation coordinates.
- The mixed-island plan can connect the private family scratch to copied island body lanes and commit solved caches in source order.
- Other joint-family plans can follow the checked definition, owned snapshot, exact setter, and transactional runtime pattern.

## Self-Check: PASSED

- All four created files exist.
- Task commit `c07abda` is present.
- Focused revolute and prismatic targets pass.
- `cargo fmt --all`, warning-denied Clippy, all-target build, all-feature tests, and doctests pass with a clean temporary Cargo target directory.
- `git diff --check` passes and `.codex/tasks/todo.md` remains outside the implementation commit.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-13*

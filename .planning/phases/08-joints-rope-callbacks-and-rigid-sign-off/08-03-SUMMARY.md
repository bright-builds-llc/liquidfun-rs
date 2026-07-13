---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "03"
subsystem: rigid-joints
tags: [rust, distance, pulley, mouse, softness, solver]
requires:
  - phase: 08-01
    provides: checked joint identity, lifecycle, tagged definitions, and snapshots
provides:
  - checked distance, pulley, and mouse definitions, mutations, snapshots, anchors, and reactions
  - pinned soft-distance gamma/bias, pulley ratio/constant, and mouse target/force-cap solver scratch
  - checked pulley-ground-anchor and mouse-target origin-shift candidates for later central dispatch
affects: [08-06, 08-10, 08-12]
tech-stack:
  added: []
  patterns: [candidate-first joint mutation, source-specific wake branches, transactional solver scratch]
key-files:
  created:
    - crates/liquidfun/src/world/joint/distance.rs
    - crates/liquidfun/src/world/joint/pulley.rs
    - crates/liquidfun/src/world/joint/mouse.rs
    - crates/liquidfun/tests/joint_distance.rs
    - crates/liquidfun/tests/joint_pulley.rs
    - crates/liquidfun/tests/joint_mouse.rs
  modified:
    - crates/liquidfun/src/joint/definition.rs
    - crates/liquidfun/src/joint/snapshot.rs
    - crates/liquidfun/src/world/joint.rs
    - crates/liquidfun/src/joint.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Preserve pinned setter asymmetry: distance tuning and mouse force/frequency/damping do not wake, while mouse target wakes only body B."
  - "Keep family solver state private and transactional while exposing owned semantic anchors, lengths, configuration, and explicit-inverse-timestep reactions."
patterns-established:
  - "Soft constraints: compute source-grouped gamma and bias in private initialization and reject non-finite candidates before cache replacement."
  - "World-space joint state: retain checked shift candidates only for pulley ground anchors and mouse targets until central origin dispatch."
requirements-completed: [JOIN-01, JOIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T23:23:30Z
duration: 16 min
completed: 2026-07-13
---

# Phase 8 Plan 03: Distance, Pulley, and Mouse Joints Summary

**Checked distance, pulley, and mouse contracts now preserve pinned geometry, softness, target wakes, force caps, warm caches, reactions, and transactional solver candidates.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-13T23:07:30Z
- **Completed:** 2026-07-13T23:23:30Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Replaced the three placeholder definitions with complete checked anchors, lengths, ratios, targets, force caps, frequency, and damping configuration.
- Added owned family snapshots and World authority for current geometry, source-specific setters, and explicit-inverse-timestep reactions.
- Added private rigid/soft distance, pulley, and mouse solver scratch covering cold/warm initialization, degenerate branches, gamma/bias, capped impulses, position correction, and transactional overflow.
- Prepared the only family-owned world-space origin state as checked pulley-ground-anchor and mouse-target candidates without entering Plan 08-06 central origin/island scope.

## Task Commits

The three tightly related family tasks share one atomic implementation commit because they extend the same closed definition, snapshot, and runtime dispatcher:

1. **Implement the distance joint** - `9adaa18` (feat)
1. **Implement the pulley joint** - `9adaa18` (feat)
1. **Implement the mouse joint** - `9adaa18` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/joint/distance.rs` - Fixed/soft scalar constraint state, queries, mutations, and solver scratch.
- `crates/liquidfun/src/world/joint/pulley.rs` - Ratio/constant geometry, slop-normalized solver state, and ground-anchor shift candidates.
- `crates/liquidfun/src/world/joint/mouse.rs` - Target-derived local anchor, source wake rules, gamma/beta mass solve, force cap, and target shift candidate.
- `crates/liquidfun/src/joint/definition.rs` - Complete checked definitions for all three families.
- `crates/liquidfun/src/joint/snapshot.rs` - Owned semantic family runtime snapshots.
- `crates/liquidfun/src/world/joint.rs` - Closed runtime creation, snapshot, and reaction dispatch.
- `crates/liquidfun/tests/joint_distance.rs` - Public distance invariants, geometry, reactions, setters, and atomic rejection.
- `crates/liquidfun/tests/joint_pulley.rs` - Public pulley invariants, constant, geometry, and reaction coverage.
- `crates/liquidfun/tests/joint_mouse.rs` - Public mouse configuration, target wake, no-wake tuning, reactions, and atomic rejection.

## Decisions Made

- Distance setters and mouse max-force/frequency/damping setters preserve the pinned no-wake behavior; mouse target mutation wakes only body B.
- Solver caches remain private and are replaced only after complete derived-state validation. Public snapshots expose owned semantic state rather than mutable solver storage.
- Mouse diagnostic reconstruction remains intentionally unsupported; the explicit typed diagnostic status is owned by Plan 08-10 rather than a fabricated dump or persistence surface here.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Deepened the shared definition and snapshot contract**

- **Found during:** Tasks 08-03-01 through 08-03-03
- **Issue:** Plan 08-01 intentionally left these three family definitions and snapshots as placeholders, so the new modules could not expose complete checked public configuration or runtime observations alone.
- **Fix:** Extended the existing closed definition, snapshot, curated-export, and runtime-dispatch files while preserving handle lifecycle, adjacency, and later island/origin ownership.
- **Files modified:** `crates/liquidfun/src/joint/definition.rs`, `crates/liquidfun/src/joint/snapshot.rs`, `crates/liquidfun/src/world/joint.rs`, curated exports.
- **Verification:** All three focused targets, 166 private library tests, and the complete ordered all-feature Rust gate pass.
- **Committed in:** `9adaa18`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Necessary shared-contract completion only; no island integration, central origin mutation, diagnostic reconstruction, dependency, unsafe code, or unordered traversal was added.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-06 can dispatch the private family scratch over copied island body lanes and commit caches in source order.
- Plan 08-06 can apply the prepared pulley-ground-anchor and mouse-target origin candidates atomically with other world state.
- Plan 08-10 can consume complete definitions/snapshots and emit the required explicit `Unsupported(MouseJoint)` reconstruction status.

## Self-Check: PASSED

- All six created files exist.
- Task commit `9adaa18` is present.
- Focused family tests, private solver tests, formatting, warning-denied Clippy, all-target build, all-feature tests, and doctests pass with the clean temporary Cargo target directory.
- `git diff --check` passes and `.codex/tasks/todo.md` remains outside the implementation commit.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-13*

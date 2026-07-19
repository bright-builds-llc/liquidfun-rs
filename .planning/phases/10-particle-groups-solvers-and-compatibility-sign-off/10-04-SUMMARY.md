---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "04"
subsystem: particle-solver
tags: [rust, liquidfun, particle-solver, pass-manifest, determinism]

requires:
  - phase: 10-01
    provides: Checked public particle and group flag contracts with pinned upstream bit values
provides:
  - Closed private phase10-pass-graph-v1 authority for all 31 particle solver passes
  - Exact outer and per-particle-iteration scope, gate, multiplicity, and order declarations
  - Fail-closed manifest validation and private bounded pass-trace instrumentation
affects: [10-17, 10-18, 10-19, 10-20, 10-21, 10-22, 10-23]

tech-stack:
  added: []
  patterns:
    - "One typed static manifest owns particle solver admission and source order"
    - "Invalid test declarations model unknown IDs outside the closed production PassId enum"

key-files:
  created:
    - crates/liquidfun/src/particle/solver.rs
    - crates/liquidfun/src/particle/solver/manifest.rs
  modified:
    - crates/liquidfun/src/particle.rs

key-decisions:
  - "Keep PassId closed while validating unknown declarations through a separate private invalid-input wrapper used by mutation tests."
  - "Represent trace iteration as Option<u32>: outer passes carry None and checked particle sub-iterations carry their exact ordinal."

patterns-established:
  - "PASS_GRAPH is the sole ordered authority for O01-O05 and S01-S26."
  - "Manifest validation rejects unknown and duplicate IDs before checking completeness, then checks order and complete descriptor metadata."

requirements-completed: [PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T13:32:32Z

duration: 47m
completed: 2026-07-19
---

# Phase 10 Plan 04: Private Particle Solver Pass Graph Summary

**A closed 31-entry Rust manifest now fixes every Phase 10 particle solver pass, gate, scope, and multiplicity to the pinned LiquidFun order**

## Performance

- **Duration:** 47m
- **Started:** 2026-07-19T12:45:49Z
- **Completed:** 2026-07-19T13:32:32Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments

- Transcribed all five outer passes and 26 per-particle-iteration passes into one private `phase10-pass-graph-v1` static authority.
- Encoded expiration, aggregate particle and group flags, dirty aggregates, pause termination, depth, force, unconditional, and sole static-pressure extra-damping gates explicitly.
- Added fail-closed validation with stable unknown, missing, duplicate, reordered, and descriptor-mismatch errors.
- Added crate-private, test/unpublished-feature-gated `(PassId, Option<u32>)` tracing whose iteration count comes from checked `StepConfiguration`.
- Added ten focused co-located tests covering exact IDs, counts, gates, multiplicity, trace order, and every manifest mutation family.

## Task Commits

Each task was committed atomically:

1. **Task 1: Transcribe and validate phase10-pass-graph-v1** - `5da47e2` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle.rs` - Declares the solver module privately without changing the public particle surface.
- `crates/liquidfun/src/particle/solver.rs` - Defines private pass IDs, scopes, gates, multiplicities, descriptors, and bounded trace entries.
- `crates/liquidfun/src/particle/solver/manifest.rs` - Owns the exact static graph, fail-closed validator, and focused mutation/order tests.

## Decisions Made

- Production descriptors always carry the closed `PassId` enum. A separate private declaration wrapper represents deliberately unknown IDs only for fail-closed validation tests, so invalid inputs do not weaken the production type.
- Outer trace entries use `None` for the particle iteration, while per-iteration entries use `Some(ordinal)`. This distinguishes scopes without inventing a sentinel iteration.
- The trace helper accepts a checked `StepConfiguration`, preserving the existing 1,024-iteration maximum instead of adding an unbounded scheduler input.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first focused compile used `ParticleGroupFlags` through the crate root, but that type is re-exported only through `particle`. The private solver imports it from its owning module instead; the focused suite and full gate then passed.
- The repository-wide all-feature suite is intentionally large and serialized across many integration binaries. Its complete exit status was preserved in a redirected log and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later kernel plans can add cohesive implementations without inventing their own pass identities or admission ordering.
- Plan 10-22 can replace the Phase 9 partial prefix by walking this one validated graph and recording only admitted private trace entries.
- Plan 10-23 can key its native witness registry to the same closed private IDs.
- No blockers remain; ready for Plan 10-05.

## Self-Check: PASSED

- All three implementation files and this summary exist.
- Task commit `5da47e2` exists on the current branch.
- Required `phase10-pass-graph-v1` and `PassId` markers exist in their planned artifacts.
- Focused manifest tests, public rustdoc privacy, and the exact ordered Rust gate passed.

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

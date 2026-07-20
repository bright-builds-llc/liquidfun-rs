---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "17"
subsystem: particle-solver
tags: [rust, particles, contacts, topology, force, gravity]

requires:
  - phase: 10-04
    provides: Closed phase10-pass-graph-v1 manifest and exact admission gates
  - phase: 10-05
    provides: Storage-owned group, flag, and solver-state authority
  - phase: 10-06
    provides: Candidate-owned solver scratch lanes and finite replacement contracts
  - phase: 10-12
    provides: Exact pair and triad topology constraints
  - phase: 10-15
    provides: Transactional solid-depth and reactive-topology algorithms
provides:
  - Safe crate-private S01-S06 and S13 preparation kernels
  - Ordered contact commits and explicit body-then-particle weight refresh
  - One-shot force consumption and exact substep gravity application
affects: [10-18, 10-20, 10-21, 10-28, particle-solver]

tech-stack:
  added: []
  patterns:
    - Thin solver sequencing over existing storage-owned contact and topology authorities
    - Candidate velocity calculation before one complete storage replacement

key-files:
  created:
    - crates/liquidfun/src/particle/solver/preparation.rs
  modified:
    - crates/liquidfun/src/particle/solver.rs
    - crates/liquidfun/src/particle/storage.rs

key-decisions:
  - "Keep Phase 9 as the sole broad-phase/contact generation authority; preparation commits its already-prepared semantic contacts without duplicating queries."
  - "Delegate solid depth and reactive topology to the Plan 15 storage transactions so failure preserves every live lane and flag."
  - "Treat the pending-force marker as the one-shot consumption authority while retaining the upstream force-buffer bytes."

patterns-established:
  - "Preparation pass shell: validate or compute a complete candidate, then call one storage-owned replacement or transaction."
  - "Inactive pass behavior: return before allocation or replacement and protect exact no-diff semantics with bit-level witnesses."

requirements-completed: [PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T14:42:19Z

duration: 55m
completed: 2026-07-20
---

# Phase 10 Plan 17: Solver Preparation and External Forces Summary

**The particle solver now has deterministic private kernels for contact refresh, weights, solid depth, reactive topology, one-shot forces, and substep gravity at the seven locked manifest positions.**

## Performance

- **Duration:** 55m
- **Started:** 2026-07-20T13:47:12Z
- **Completed:** 2026-07-20T14:42:19Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments

- Added S01/S02 commit kernels over the existing source-ordered Phase 9 particle and body contact outputs, including stuck-state refresh without a second broad-phase path.
- Added S03/S04/S05 sequencing over the single storage authority for exact body-then-particle weights, scheduled solid depth, and commit-bound reactive topology clearing.
- Added S06/S13 velocity candidates with checked finite arithmetic, one-time force-marker consumption, exact source mass and gravity conventions, and zero-step controls.
- Added 13 co-located private witnesses for manifest multiplicity, control, activation, interaction, empty systems, mixed flags, zero steps, retained-empty groups, relative identity slots, deterministic order, and rollback.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement exact preparation kernels** - `7d5a972` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/preparation.rs` - S01-S06/S13 kernels and their private unit witnesses.
- `crates/liquidfun/src/particle/solver.rs` - Registers the preparation child module beside the manifest and pressure families.
- `crates/liquidfun/src/particle/storage.rs` - Exposes the existing weight recomputation authority to sibling particle solver code.

## Decisions Made

- Contact generation stays in the Phase 9 world/contact modules; preparation accepts semantic contact candidates and performs only authoritative storage replacement.
- Depth and reactive passes call the exact Plan 15 transactions instead of reimplementing group or Voronoi logic.
- S06 clears only the pending-force marker after successful velocity replacement, matching upstream one-shot consumption while leaving force-buffer bytes intact.
- The 555-line preparation file remains cohesive because production ends at line 124 and the remaining lines are the plan-required co-located tests; no production function approaches the repository size trigger.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the new module and exposed the existing weight refresh authority**

- **Found during:** Task 1 (Implement exact preparation kernels)
- **Issue:** The plan named only the new preparation file, but an unregistered module cannot compile and sibling solver code could not call storage's private weight refresh.
- **Fix:** Added one module declaration and a four-line crate-particle-private forwarding method over the existing recomputation implementation.
- **Files modified:** `crates/liquidfun/src/particle/solver.rs`, `crates/liquidfun/src/particle/storage.rs`
- **Verification:** The exact focused command passed 13 preparation tests and every zero-match integration launch; the full ordered gate passed 358 library tests, every integration test, and 19 doctests.
- **Committed in:** `7d5a972`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The minimal seams make the planned module reachable without widening public API or duplicating physics behavior.

## Issues Encountered

- macOS executable admission delayed the broad focused filter even though the 13 matching tests completed in 0.01 seconds. Both prescribed focused runs and the final full gate completed uninterrupted and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 10-18, 10-20, and 10-21 can compose their material and structural kernels around prepared contact, weight, topology, force, and gravity state.
- Plan 10-28 can wire the complete pass graph without introducing another contact or topology authority.
- No blockers remain.

## Self-Check: PASSED

- Confirmed the three task files and implementation commit `7d5a972` exist.
- Confirmed all seven preparation IDs occur exactly once in the closed manifest.
- Confirmed 13 focused preparation tests, 358 library tests, every integration test, and 19 doctests pass.
- Confirmed no unsafe/FFI code, dependency, public API, network/file effect, or duplicate broad-phase path was introduced.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

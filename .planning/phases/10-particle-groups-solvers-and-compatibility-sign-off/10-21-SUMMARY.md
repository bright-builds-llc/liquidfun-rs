---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "21"
subsystem: particle-rigid-and-boundary-solvers
tags: [rust, particles, rigid-groups, barriers, collision, integration, source-parity]

requires:
  - phase: 10-02
    provides: Checked particle damping and solver coefficients
  - phase: 10-04
    provides: Closed S01-S26 solver manifest and exact pass order
  - phase: 10-05
    provides: Storage-owned group metadata and timestamped statistics
  - phase: 10-06
    provides: Candidate-owned aligned solver scratch and aggregate gates
  - phase: 10-12
    provides: Provenance-bound retained-empty and one-particle rigid outcomes
  - phase: 10-15
    provides: Storage-owned group lifecycle, statistics, and stable membership
provides:
  - Exact distinct S21 rigid damping and S24 rigid projection candidates
  - Exact staged S22 barrier, S23 previous-transform collision, S25 wall, and S26 integration candidates
  - Bounded body-impulse and semantic-effect journals with typed rollback errors
  - Control, activation, interaction, ownership, resource, and determinism witnesses for every solver-tail pass
affects: [10-22, 10-23, particle-solvers, particle-world-coupling, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Pure source-ordered rigid and boundary candidates committed later by the world shell
    - Explicit pass-stage state machine for collision, rigid projection, wall, and once-only integration
    - Filtered world-query input as the only fixture-collision admission surface

key-files:
  created:
    - crates/liquidfun/src/particle/solver/rigid.rs
    - crates/liquidfun/src/particle/solver/rigid/damping.rs
    - crates/liquidfun/src/particle/solver/rigid/projection.rs
    - crates/liquidfun/src/particle/solver/rigid/support.rs
    - crates/liquidfun/src/particle/solver/rigid/tests.rs
    - crates/liquidfun/src/particle/solver/boundary.rs
    - crates/liquidfun/src/particle/solver/boundary/barrier.rs
    - crates/liquidfun/src/particle/solver/boundary/collision.rs
    - crates/liquidfun/src/particle/solver/boundary/support.rs
    - crates/liquidfun/src/particle/solver/boundary/tests.rs
  modified:
    - crates/liquidfun/src/particle/solver.rs

key-decisions:
  - "Keep S21 body reactions as ordered impulse candidates and never mutate body state inside the rigid numerical kernel."
  - "Accept S23 hits only after the existing world query and filter authority admits them, then reconstruct the previous-transform start only on particle iteration zero."
  - "Represent the S22-S26 tail as an explicit private stage machine so rigid projection, wall enforcement, and final integration cannot be reordered or repeated."
  - "Preserve Plan 10-12 retained-empty and one-particle rigid outcomes exactly without introducing a broad epsilon or special non-finite path."

patterns-established:
  - "Rigid transaction: validate owner generations, contiguous membership, aligned stable IDs, group statistics, contacts, and capacity before returning velocities, transforms, and body impulses together."
  - "Boundary transaction: clone a complete candidate, enforce the expected pass stage, append curated bounded effects, validate finiteness, and leave the source bit-identical on failure."

requirements-completed: [PART-09, PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T19:00:33Z

duration: 2h 10m
completed: 2026-07-20
---

# Phase 10 Plan 21: Exact Rigid and Boundary Solver Tail Summary

**Rigid-group damping and projection now compose with source-ordered barrier, previous-transform collision, wall, and exactly-once integration candidates while preserving stable ownership and transactional body/effect boundaries.**

## Performance

- **Duration:** 2h 10m
- **Started:** 2026-07-20T16:50:00Z
- **Completed:** 2026-07-20T19:00:33Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Implemented S21 rigid damping for rigid-to-body and inter-group contacts using exact mass, inertia, center, point velocity, damping impulse, and source-order accumulation.
- Implemented S24 rigid projection with exact delta rotation/translation composition and member velocity writes while retaining particle IDs, group IDs, ranges, associations, and member order.
- Applied the mandatory Plan 10-12 retained-empty and one-particle rigid classifications verbatim and bound the tests to the reviewed witness/provenance identity.
- Implemented S22 barrier crossing, including barrier-wall endpoint zeroing, rigid-group force distribution, pending-force preservation, collapsed-pair finite no-op behavior, and bounded scan/effect resources.
- Implemented S23 collision over already-filtered query hits with the pinned iteration-zero previous-transform reconstruction, circle local-center correction, stable hit order, exact slop, velocity, and force formulas.
- Implemented S25 wall and S26 final integration behind a private stage machine that rejects early, reordered, or repeated integration.
- Added twenty-three focused witnesses spanning controls, activations, two-group and body interactions, transforms, stable ownership, resource rollback, pass order, and 128 exact deterministic repeats.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement rigid-group solver passes** - `f741006` (feat)
2. **Task 2: Implement barrier, collision, wall, and integration** - `7749226` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/rigid.rs` - Private S21/S24 types and closed module surface.
- `crates/liquidfun/src/particle/solver/rigid/damping.rs` - Exact group statistics, rigid/body damping, inter-group damping, and body-impulse candidates.
- `crates/liquidfun/src/particle/solver/rigid/projection.rs` - Exact group-transform advancement and member-velocity projection.
- `crates/liquidfun/src/particle/solver/rigid/support.rs` - Owner, membership, capacity, and finite-candidate validation.
- `crates/liquidfun/src/particle/solver/rigid/tests.rs` - Oracle, control, interaction, ownership, transform, and rollback witnesses.
- `crates/liquidfun/src/particle/solver/boundary.rs` - Solver-tail stage, effect, filtered-hit, and complete candidate contracts.
- `crates/liquidfun/src/particle/solver/boundary/barrier.rs` - Exact S22 crossing and follow-up force candidate.
- `crates/liquidfun/src/particle/solver/boundary/collision.rs` - Exact S23/S25/S26 previous-transform collision, wall, and integration candidates.
- `crates/liquidfun/src/particle/solver/boundary/support.rs` - Aligned source-lane, ownership, capacity, and finite-candidate validation.
- `crates/liquidfun/src/particle/solver/boundary/tests.rs` - Pass-order, transform, activation, interaction, resource, and determinism witnesses.
- `crates/liquidfun/src/particle/solver.rs` - Private rigid and boundary module registration.

## Decisions Made

- Kept body mutation in the existing future world commit shell. S21 emits ordered `BodyImpulseCandidate` records after all owner, member, numeric, and capacity validation.
- Recomputed rigid statistics in exact source member order, while the timestamp cache retains source semantics across S21 and S24. Empty retained groups remain finite exact-zero controls; one-particle groups retain zero inertia/angular velocity and finite translation.
- Modeled fixture hits as `FilteredCollisionHit`: this private record is produced only after the existing world query/filter path and carries semantic values rather than storage references or callbacks.
- Used a closed `BoundaryStage` sequence for S22, S23, S24 handoff, S25, and S26. The type rejects integration before wall and rejects a second integration without relying on caller convention.
- Preserved the source barrier follow-up-force behavior by updating the candidate force lane and pending-force marker after the immediate velocity/group response.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered both private solver modules**

- **Found during:** Task 1 and Task 2 implementation
- **Issue:** The planned files could not compile or be consumed by Plan 10-22 without private parent-module registration.
- **Fix:** Added only `mod rigid;` and `mod boundary;` to `particle/solver.rs`; no public visibility changed.
- **Files modified:** `crates/liquidfun/src/particle/solver.rs`
- **Verification:** Both exact broad focused filters, warning-denied all-target clippy, all-target build, and complete test suite passed.
- **Committed in:** `f741006`, `7749226`

**2. [Rule 3 - Code Shape] Split production kernels into cohesive child modules**

- **Found during:** Task 1 and Task 2 simplification reviews
- **Issue:** Keeping all numerical kernels, validation, types, and witnesses in the two planned root files would exceed repository code-shape guidance and obscure the distinct pass boundaries.
- **Fix:** Kept `rigid.rs` and `boundary.rs` as closed private API roots, then split damping/projection/support and barrier/collision/support into named child modules. The largest production file is 355 lines.
- **Files modified:** `crates/liquidfun/src/particle/solver/rigid/**`, `crates/liquidfun/src/particle/solver/boundary/**`
- **Verification:** All focused witnesses and exact ordered gates passed; no function approaches the 161-line review trigger.
- **Committed in:** `f741006`, `7749226`

**Total deviations:** 2 auto-fixed (2 blocking/code-shape).
**Impact on plan:** Both changes are private compilation and readability seams. They add no public API, dependency, unsafe/FFI code, duplicate world authority, external I/O, or pass-order change.

## Issues Encountered

- macOS executable-policy scanning dominated broad-filter and all-target wall time. Both required filter commands ran without `--lib` through every zero-match integration binary, and every mandatory gate ran uninterrupted.
- The first Task 1 gate found three exact float assertions that Clippy required to compare as bit patterns. The assertions were corrected, the final-source broad filter was repeated, and a fresh fail-fast four-command gate passed before commit.
- Initial barrier test geometry placed the third particle outside the pinned endpoint AABB. The witnesses were corrected to exercise an admitted crossing at time zero, after which activation, rigid interaction, and journal-overflow behavior passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-22 can adapt storage groups, filtered fixture hits, candidate bodies, and bounded world journals into the closed S21-S26 kernel surfaces without changing their numerical order.
- Plan 10-23 can reuse twenty-three focused witnesses for native solver-tail coverage closure.
- No blockers remain.

## Self-Check: PASSED

- Confirmed both task commits `f741006` and `7749226` exist.
- Confirmed all eleven implementation paths and this summary exist.
- Confirmed the mandatory rigid oracle cases and provenance identity are asserted.
- Confirmed both exact broad filters ran on final source and included every zero-match integration binary.
- Confirmed fresh Task 1 and Task 2 format, warning-denied all-target clippy, all-target build, complete integration suite, and 19 doctests passed before their commits.
- Confirmed stable particle/group identity, association, membership, order, body-candidate ownership, previous-transform collision, bounded rollback, and once-only integration have focused evidence.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

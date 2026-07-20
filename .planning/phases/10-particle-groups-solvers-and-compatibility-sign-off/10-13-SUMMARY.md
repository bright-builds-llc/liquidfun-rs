---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "13"
subsystem: particle-group-mutation
tags: [rust, particle-groups, atomic-mutation, topology, rollback]

requires:
  - phase: 10-07
    provides: Complete permutation-safe particle storage and mutation candidates
  - phase: 10-12
    provides: Exact bounded pair and triad generation with explicit record policies
provides:
  - Atomic source-ordered particle-group join below ParticleStorage
  - Exact preservation of group A, every ParticleId, and historical topology rest records
  - Cross-threshold-only connection generation with bounded failure rollback
affects: [10-14, 10-20, particle-groups, particle-topology, group-lifecycle]

tech-stack:
  added: []
  patterns:
    - Owned storage candidate with single no-fail replacement commit
    - Checked two-rotation mapping over stable particle identities
    - Operation-specific append policy preserving historical topology order

key-files:
  created:
    - crates/liquidfun/src/particle/storage/mutation/join.rs
    - crates/liquidfun/src/particle/storage/mutation/join/tests.rs
  modified:
    - crates/liquidfun/src/particle/storage/mutation.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/topology.rs

key-decisions:
  - "Prepare joins against an owned ParticleStorage candidate and expose only a no-fail whole-candidate commit, so every validation, topology, and capacity failure is effect-free."
  - "Keep the generic mutation candidate's established whole-topology append policy unchanged and add a dedicated private exact-join preparation seam."
  - "Sort and deduplicate generated cross-threshold constraints before append while preserving every historical record's order and rest bytes."

patterns-established:
  - "Join order: validate both live handles, move B to the end, move A immediately before B, generate cross constraints, merge membership and flags, then remove B's record last."
  - "Join rollback: all fallible work targets an owned storage candidate; the live storage changes only through one complete replacement."

requirements-completed: [PART-10, PART-11, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T01:48:11Z

duration: 1h 53m
completed: 2026-07-19
---

# Phase 10 Plan 13: Exact Particle Group Join Summary

**Atomic source-ordered group joining now preserves group A and every particle identity while appending only bounded cross-threshold topology without rewriting historical rest state.**

## Performance

- **Duration:** 1h 53m
- **Started:** 2026-07-19T23:54:49Z
- **Completed:** 2026-07-20T01:48:11Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Added a private `plan_join` transaction that validates distinct live same-world/same-system groups before preparing any effect.
- Implemented the pinned two-rotation order: group B moves to the system end and group A moves immediately before B, including retained-empty A/B behavior.
- Preserved group A's complete record identity, every stable `ParticleId`, and exact historical pair/triad ordering and rest-field bits.
- Generated connections only when endpoints cross the old A/B threshold, using the existing checked constraint generator after contact indices have been remapped.
- Unioned public and required internal group flags, scheduled depth invalidation when SOLID is newly inherited, invalidated A's statistics, reassigned B's members, and removed B's record last.
- Bounded contacts and candidate topology by checked combination limits and propagated bounded Voronoi failures without changing live storage.
- Added six focused tests covering interleaved groups, retained-empty A/B, historical A/A/A and B/B/B topology, flag/cache union, exact stable-ID order, cross-only append, stale/aliased/wrong-system handles, capacity failure, and complete snapshot rollback.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement source-ordered join with cross-only topology** - `1054683` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/mutation/join.rs` - Pure join planning, checked rotations, cross filter, bounds, group merge, and no-fail commit.
- `crates/liquidfun/src/particle/storage/mutation/join/tests.rs` - Focused identity, ordering, topology, flags, empty-group, handle, capacity, and rollback evidence.
- `crates/liquidfun/src/particle/storage/mutation.rs` - Registered the split join module and its dedicated exact-join candidate preparation seam.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Added a private append policy that leaves historical topology untouched and appends only new unique records.
- `crates/liquidfun/src/particle/topology.rs` - Made checked Voronoi limit construction visible within the private particle subsystem.

## Decisions Made

- Used an owned full-storage candidate because it makes rotation, topology generation, group-range changes, flags, solver aggregate state, and B invalidation one atomic transaction without rollback code.
- Kept the pre-existing generic join candidate policy unchanged after the semantic property model proved other mutation tests depend on its established whole-topology sort behavior.
- Treated remapped current contacts as the refreshed post-rotation contact state because rotation changes dense indices but not particle geometry; constraint generation consumes only the remapped candidate.
- Classified absent same-world group records as stale/destroyed and explicitly rejected records owned by a different particle system before storage invariant validation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added a private historical-preserving append policy**

- **Found during:** Task 1 (Implement source-ordered join with cross-only topology)
- **Issue:** The only existing append policy stable-sorted the complete combined topology, which would rewrite historical A/A and B/B record order contrary to the plan.
- **Fix:** Added a private permutation policy that validates/remaps historical records unchanged and appends only newly generated unique records.
- **Files modified:** `crates/liquidfun/src/particle/storage/permutation.rs`, `crates/liquidfun/src/particle/storage/mutation.rs`
- **Verification:** Exact rest-bit/order tests, focused mutation/permutation lanes, and the complete ordered Rust gate passed.
- **Committed in:** `1054683`

**2. [Rule 3 - Blocking] Exposed bounded Voronoi limit construction inside the particle subsystem**

- **Found during:** Task 1 (Implement source-ordered join with cross-only topology)
- **Issue:** Join planning requires explicit bounded topology limits, but the checked constructor was visible only inside the topology module.
- **Fix:** Widened only `VoronoiLimits::new` to `pub(in crate::particle)`; the type and fields remain private to the particle subsystem.
- **Files modified:** `crates/liquidfun/src/particle/topology.rs`
- **Verification:** The injected generator-limit failure test proved exact no-diff rollback, and the complete ordered Rust gate passed.
- **Committed in:** `1054683`

**3. [Rule 1 - Bug] Isolated exact join semantics from the generic mutation candidate**

- **Found during:** Full `cargo test --all-features`
- **Issue:** Initially changing the existing generic join candidate to preserve history broke its independent semantic property model, which intentionally expects the prior whole-topology append sort.
- **Fix:** Restored that established contract and introduced a dedicated private `prepare_exact_join_groups` path used only by `plan_join`.
- **Files modified:** `crates/liquidfun/src/particle/storage/mutation.rs`, `crates/liquidfun/src/particle/storage/mutation/join.rs`
- **Verification:** The minimized failing property passed, followed by all 311 library tests, every integration test, and 19 doctests.
- **Committed in:** `1054683`

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug)
**Impact on plan:** All changes are private, minimal seams required for the specified exact join semantics; no public API, dependency, or runtime trust boundary changed.

## Issues Encountered

- Fresh macOS test executables spent substantial time in the dynamic loader. Already-built binaries were warmed without modifying source or tracked artifacts, after which the exact ordered test command completed successfully.
- The full suite exposed a transient regression in the generic mutation candidate; its generated proptest seed was removed after the root cause was fixed and the minimized property passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Subsequent group lifecycle integration can commit one validated storage join candidate and invalidate the world-owned group B shell from the returned success path.
- Split and reactive topology work can reuse the now-explicit distinction between historical-preserving and whole-topology append policies.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all five created or modified task files exist.
- Confirmed task commit `1054683` exists.
- Confirmed no stub patterns, unsafe/FFI code, public API widening, new network endpoints, file access, schema changes, or auth surfaces were introduced.
- Confirmed focused join, mutation, permutation, topology, minimized property, and exact ordered Rust gates all pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

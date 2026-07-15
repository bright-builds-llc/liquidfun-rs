---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "10"
subsystem: particle-forces-statistics
tags: [rust, particles, forces, impulses, semantic-statistics, transactional-validation]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "09"
    provides: source-timed particle/body contacts, stable stuck state, and rigid coupling
provides:
  - checked singleton and stable contiguous-range particle force and impulse operations
  - source-scaled force accumulation and source-mass impulse velocity updates
  - owned per-system and aggregate world particle statistics over semantic state
affects: [09-13, phase-10, particle-oracle]
tech-stack:
  added: []
  patterns: [validate-all then commit, stable-ID contiguous range, owned semantic statistics]
key-files:
  created:
    - crates/liquidfun/src/particle/force.rs
    - crates/liquidfun/src/particle/statistics.rs
    - crates/liquidfun/tests/particle_forces_statistics.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/src/particle/view.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Represent contiguous ranges as caller-borrowed stable-ID slices and require every ID to match current source order before mutation."
  - "Reject force and impulse operations for wall particles and validate complete accumulated-force or resulting-velocity candidates before commit."
  - "Report only explicit capacity contracts and stable semantic snapshots; allocator capacity, timestamps, dense rows, and scratch counters remain private."
requirements-completed: [PART-16]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T08:28:48Z
duration: 20 min
completed: 2026-07-15
---

# Phase 9 Plan 10: Particle Forces and Statistics Summary

**Particle forces and impulses now validate complete stable-ID ranges before source-scaled application, while owned statistics expose collision, contact, stuck, lifecycle, pause, and capacity state without leaking private coordinates.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-15T08:09:00Z
- **Completed:** 2026-07-15T08:28:48Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added checked singleton and contiguous-range force/linear-impulse APIs with complete world, system, liveness, wall, finite-input, mass, scale, and final-candidate validation before mutation.
- Preserved the pinned arithmetic: range force divides one total force by source count, while impulse divides by `count * density * stride * stride` and applies equal velocity deltas in source order.
- Added owned per-system statistics for stable particle IDs, collision energy, contact/body-contact counts, stable stuck candidates, paused/pending state, group counts, and explicit declared/effective/configured capacities.
- Added aggregate world statistics in newest-first system traversal order without exposing allocator capacity, timestamps, internal rows, or scratch counters.

## TDD Evidence

### Task 1: Checked particle/range forces and impulses

- **RED:** `cargo test -p liquidfun --test particle_forces_statistics forces` failed with six E0599 errors because the force/range APIs and immutable semantic force observation did not exist.
- **GREEN:** Eight focused tests pass for singleton/range equivalence, pinned force and impulse scaling, empty/noncontiguous ranges, pending/stale/cross-system/cross-world IDs, wall particles, non-finite inputs, invalid derived mass, and full no-effect rejection.

### Task 2: Semantic Phase 9 statistics

- **RED:** `cargo test -p liquidfun --test particle_forces_statistics statistics` failed with six E0599 errors because per-system and aggregate-world statistics entrypoints did not exist.
- **GREEN:** Three focused tests pass for contact refresh and source-grouped collision energy, explicit capacity reporting, pause/pending/compaction/teardown transitions, aggregate counts, and non-empty stable-ID stuck candidates.

## Task Commits

1. **Implement checked particle/range forces and impulses** - `92634f0` (feat)
1. **Expose semantic Phase 9 statistics** - `67118b4` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/force.rs` - Pure validate/prepare candidates, typed failures, pinned distribution scales, and source-ordered commit payloads.
- `crates/liquidfun/src/particle/statistics.rs` - Owned per-system and world aggregate semantic statistics plus pinned collision-energy computation.
- `crates/liquidfun/tests/particle_forces_statistics.rs` - Black-box reachability, transactionality, scaling, lifecycle, contact, capacity, and stuck regressions.
- `crates/liquidfun/src/particle/storage.rs` - Narrow particle-module adapters for range resolution, final candidate replacement, explicit declared capacity, and semantic lane reads.
- `crates/liquidfun/src/world/particle_object.rs` - Public ownership-shell entrypoints over the authoritative system storage.
- `crates/liquidfun/src/particle/view.rs` - Immutable accumulated-force inspection aligned with stable identities.
- `crates/liquidfun/src/particle.rs` and `crates/liquidfun/src/lib.rs` - Curated child-module and crate-root exports.

## Decisions Made

- Caller-borrowed ID slices are the safe range coordinate: the full slice resolves first, every adjacent ID must map to the next current source row, and no dense coordinate becomes public.
- Wall particles reject both force and impulse controls under the locked checked API contract, even though the pinned assertion is force-specific, because D-22 requires wall restrictions across the combined operation surface.
- Prepared force and impulse candidates include each final selected row value. This catches overflow against already accumulated force or velocity before any authoritative lane changes.
- Statistics own stable identity vectors and counts. Aggregate energy follows system traversal order, while per-contact energy follows the pinned contact-buffer statement grouping.

## Deviations from Plan

### Auto-fixed Supporting Files

**1. [Rule 3 - Blocking] Added narrow authoritative-storage and world ownership adapters**

- **Found during:** Tasks 1 and 2
- **Issue:** The planned particle child modules could validate arithmetic but could not resolve live system-owned stable IDs, commit authoritative lanes, or inspect current contact/lifecycle state from a sibling module.
- **Fix:** Added narrow storage adapters and public `World` shell methods at the existing particle-object ownership boundary; immutable force observation was added to the established borrow-scoped view.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/world/particle_object.rs`, and `crates/liquidfun/src/particle/view.rs`
- **Verification:** Focused force/statistics suites, warning-denied Clippy, all-target build, all-feature tests, and compile-fail doctests pass.
- **Committed in:** `92634f0`, `67118b4`

***

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** The supporting changes stay at the existing storage/world/view boundaries and expose no mutable lane, raw capacity, dense row, unsafe code, solver pass, or Phase 10 topology behavior.

## Issues Encountered

- The first Task 1 warning-denied gate rejected the intentional `usize`-to-`f32` source count conversion. A narrow documented allowance now records the int32 count bound and pinned float32 cast.
- The first Task 2 warning-denied gates requested panic documentation for the internal order invariant and separation of an oversized integration test. The docs and focused test split were completed before restarting and passing the mandatory gate sequence.
- The collision-energy expectation initially regrouped multiplications and differed by one ULP; the independent expectation now preserves the pinned `0.5 * mass * sum_v2` statement grouping.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 9 now has checked force/impulse inputs and the semantic statistics required by PART-16 for protocol and evidence work.
- Phase 10 can consume the authoritative accumulated-force lane during solver execution without changing the public transaction boundary.

## Self-Check: PASSED

- Both task commits are present and all three created files exist.
- Focused force and statistics suites pass with 8 and 3 tests respectively.
- `cargo check -p liquidfun --all-features` passes.
- The exact ordered `cargo fmt --all`, warning-denied Clippy, all-target/all-feature build, and all-feature test gates pass before each task commit.
- Base-to-head review finds no unsafe code, raw mutable lane, allocator-capacity export, dense-index export, internal timestamp/scratch-counter export, or Phase 10 solver behavior.
- `.planning/STATE.md` and `.planning/ROADMAP.md` are unchanged.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*

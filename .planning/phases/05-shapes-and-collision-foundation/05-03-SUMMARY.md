---
phase: 05-shapes-and-collision-foundation
plan: "03"
subsystem: collision-distance
tags: [rust, gjk, simplex, distance-cache, overlap, property-tests]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "02"
    provides: immutable validated shape-child topology and source-ordered math queries
  - phase: 04-math-settings-and-numerical-policy
    provides: exact f32 vector, transform, epsilon, and operand-order foundations
provides:
  - validated private distance proxies for every supported shape child
  - topology-bound initialized reusable GJK cache with semantic snapshots
  - source-ordered simplex solve2/solve3 and bounded GJK distance
  - radii-adjusted witnesses and strict pinned overlap predicate
affects: [05-04-narrow-phase, 05-06-toi, 05-07-differential-evidence, phase-6-rigid-world]
tech-stack:
  added: []
  patterns: [bounded private proxy, topology-bound cache, static simplex kernel, semantic diagnostic surface]
key-files:
  created:
    - crates/liquidfun/src/collision/distance/proxy.rs
    - crates/liquidfun/src/collision/distance/simplex.rs
    - crates/liquidfun/tests/collision_distance.rs
  modified:
    - crates/liquidfun/src/collision/distance.rs
key-decisions:
  - "Reject incompatible cache topology before cached support indices are read rather than silently resetting cross-shape state."
  - "Bind cache compatibility to exact private shape-child geometry while allowing transforms to change between warm calls."
  - "Expose only semantic cache pairs, metric, witnesses, distance, and iteration count; keep topology identity and branch diagnostics private."
patterns-established:
  - "Distance proxy pattern: borrow bounded polygon vertices and inline one/two-point shape children without public raw storage."
  - "GJK pattern: preserve source branch and expression order in one fixed-cap path, then apply radii as a separate final step."
requirements-completed:
  - COLL-03
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T16:44:11Z
duration: 14 min
completed: 2026-07-11
---

# Phase 5 Plan 03: GJK Distance, Cache, and Overlap Summary

**One bounded source-ordered GJK kernel now computes topology-safe warm-cached witnesses, radii-adjusted distance, and strict overlap for every validated shape child.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-11T16:30:26Z
- **Completed:** 2026-07-11T16:44:11Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added private bounded distance proxies for circle, edge, polygon, and checked chain children, with exact radii and strict first-on-tie support selection.
- Added initialized `DistanceCache::empty()`, topology-safe reuse, exact metric flush thresholds, and owned semantic snapshots of ordered support pairs and metric.
- Translated source-ordered one-, two-, and three-point simplex behavior, barycentric solve branches, search direction, witnesses, metric, duplicate detection, and the fixed 20-support-call loop.
- Added public distance results with witnesses, scalar distance, exact iteration count, and updated cache for both core and radii-enabled modes.
- Added strict `distance < 10.0 * EPSILON` overlap behavior with below/equal/above regression witnesses.
- Proved the kernel with 10 public integration/property tests and 16 focused proxy/cache/simplex/GJK unit tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build validated shape-child proxies and initialized cache** - `9707243` (`feat`)
1. **Task 2: Implement source-ordered simplex solve, GJK output, and overlap** - `f80703b` (`feat`)

## Files Created/Modified

- `crates/liquidfun/src/collision/distance.rs` - Curates public cache/result APIs, topology binding, the bounded GJK loop, radii behavior, overlap, and focused branch tests.
- `crates/liquidfun/src/collision/distance/proxy.rs` - Maps every validated shape child to bounded vertices, exact radius, private identity, and strict support selection.
- `crates/liquidfun/src/collision/distance/simplex.rs` - Implements cache reconstruction, simplex solve2/solve3, witnesses, metric, and support bookkeeping.
- `crates/liquidfun/tests/collision_distance.rs` - Exercises public cold/warm cache behavior, witnesses, radii, overlap boundaries, symmetry, and private-debug redaction.

## Decisions Made

- Cache reuse returns `CollisionError::IncompatibleDistanceCache` when ordered shape-child geometry differs. This fail-closed policy prevents any cross-topology index access and makes invalid reuse explicit evidence.
- Cache identity uses private exact vertex/radius bits and semantic child kind/coordinate, not pointers, allocation addresses, or public storage coordinates. Transform changes remain compatible, matching ordinary warm-cache use.
- Polygon points are borrowed immutably; circle, edge, and selected chain-child points use private inline two-point storage. The iterative loop performs no vertex allocation growth.
- GJK iteration count remains the number of support calls, matching the pinned source. Triangle, near-zero direction, duplicate support, and iteration-limit causes stay in a bounded private diagnostic trace.
- Radii adjustment remains after core witness/cache production: separated witnesses move to surfaces, while overlap collapses both witnesses to their midpoint and sets exact zero distance.
- The 666-line distance entrypoint was reviewed against the file-size trigger. Proxy and simplex responsibilities already occupy separate planned modules; the remaining entrypoint combines its cohesive public/cache/GJK orchestration with branch-focused unit tests, so another unplanned file would not clarify ownership.

## TDD Evidence

- **RED Task 1:** `cargo test -p liquidfun --test collision_distance cache --all-features` failed on the unresolved public `DistanceCache` import.
- **GREEN Task 1:** Public cache/proxy-surface filters and eight internal proxy/cache tests passed after the bounded proxy and topology binding were added.
- **RED Task 2:** `cargo test -p liquidfun --test collision_distance gjk --all-features` failed on unresolved `distance` and `test_overlap` imports.
- **GREEN Task 2:** Five GJK-filtered public tests, two overlap-filtered tests, and all 16 internal distance tests passed.
- **REFACTOR:** Split proxy and simplex kernels into planned child modules, replaced private derived debug output with semantic-only formatting, and documented the two-shape argument boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Prevented private cache and diagnostic disclosure through `Debug`**

- **Found during:** Task 2 simplification and threat-model review
- **Issue:** Derived `Debug` would have printed private topology identity and bounded branch diagnostics, contradicting T03-4 and the semantic-only public cache contract.
- **Fix:** Added manual semantic-only `Debug` implementations and a public regression test that rejects topology-field names.
- **Files modified:** `crates/liquidfun/src/collision/distance.rs`, `crates/liquidfun/tests/collision_distance.rs`
- **Verification:** Strict Clippy, debug-redaction test, full Rust gate, and rustdoc passed.
- **Committed in:** `f80703b`

**Total deviations:** 1 auto-fixed (1 missing critical). **Impact:** The fix narrows information disclosure without changing distance behavior or plan scope.

## Issues Encountered

- The first witness-consistency property found a one-ULP difference between the pre-adjustment scalar distance and distance recomputed from rounded radii-shifted witness coordinates. Exact A/B symmetry remains required; the coordinate-consistency assertion now uses a named scale-aware four-epsilon test bound. The generated out-of-scope proptest file was removed.
- Strict Clippy required `finish_non_exhaustive()` for the intentionally redacted distance-result debug representation.
- `cargo package` reports the repository's existing warnings that integration tests are excluded by the publish include list. The verified 43-file package contains native Rust and Cargo/documentation inputs only.

## Verification

- `cargo test -p liquidfun --test collision_distance --all-features` passed all 10 public tests.
- `cargo test -p liquidfun collision::distance --all-features` passed all 16 focused unit tests.
- Every task commit was preceded by `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests in the mandated order.
- Final `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps` passed.
- `cargo package -p liquidfun --allow-dirty` packaged and verified successfully.
- Forbidden unsafe/unchecked/raw-cache/FMA scans and `git diff --check` passed.
- Source audit confirmed strict support replacement, cache ratio/epsilon comparisons, near-zero direction, 20-call cap, radii branch, and strict overlap predicate against pinned `b2Distance.cpp`/`b2Collision.cpp`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 05-04 can consume stable distance and overlap semantics while implementing supported manifold pairs.
- Plan 05-06 can reuse the same checked shape-child proxy, cache, simplex, and support ordering for TOI.
- Plan 05-07 can serialize semantic cache pairs, metric, iterations, witnesses, and private diagnostic evidence without changing consumer APIs.

## Self-Check: PASSED

- Task commits `9707243` and `f80703b` exist in history.
- All four planned source/test artifacts exist on disk.
- Full focused, Rust, rustdoc, package, forbidden-pattern, and diff checks passed.
- Only pre-existing orchestrator changes to `.planning/STATE.md` and `.planning/config.json` remain outside the plan commits.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_

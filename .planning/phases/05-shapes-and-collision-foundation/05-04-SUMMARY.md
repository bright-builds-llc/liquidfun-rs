---
phase: 05-shapes-and-collision-foundation
plan: "04"
subsystem: collision-narrow-phase
tags: [rust, clipping, manifolds, contact-features, edge-adjacency, pair-registry]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "01"
    provides: initialized semantic manifold, point-state, feature, and collision-outcome values
  - phase: 05-shapes-and-collision-foundation
    plan: "02"
    provides: immutable validated circle, polygon, edge, chain, and child topology
  - phase: 04-math-settings-and-numerical-policy
    provides: source-ordered vectors, transforms, epsilon, slop, and polygon-radius constants
provides:
  - private source-ordered segment clipping with semantic feature identity
  - ordered semantic point-state transitions and world-manifold conversion
  - circle, polygon, edge, and checked chain-child manifold kernels
  - closed seven-family pair registry with explicit primary/reversed orientation
affects: [05-07-differential-evidence, phase-6-contact-persistence, phase-6-rigid-world]
tech-stack:
  added: []
  patterns: [private bounded clip kernel, semantic feature identity, canonical pair orientation, active-only world manifold]
key-files:
  created:
    - crates/liquidfun/src/collision/narrow/clipping.rs
    - crates/liquidfun/src/collision/narrow/circle.rs
    - crates/liquidfun/src/collision/narrow/polygon.rs
    - crates/liquidfun/src/collision/narrow/edge.rs
    - crates/liquidfun/tests/collision_manifolds.rs
  modified:
    - crates/liquidfun/src/collision/narrow.rs
key-decisions:
  - "Keep clipping private and compare contact persistence through the four semantic feature fields rather than a packed union key."
  - "Represent world manifolds with active points only, preserving source point order and normals oriented from canonical shape A to B."
  - "Evaluate reversed registered inputs in pinned primary order and expose the reversal explicitly instead of silently remapping the registry contract."
patterns-established:
  - "Manifold kernel pattern: validate public transforms, preserve source branch/loop order internally, and return supported separation as None without inactive payload."
  - "Pair registry pattern: validate both child coordinates, dispatch a closed match over seven families, and distinguish Unsupported, Separated, and Touching."
requirements-completed:
  - COLL-03
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T17:17:40Z
duration: 25 min
completed: 2026-07-11
---

# Phase 5 Plan 04: Clipping, Manifolds, and Pair Dispatch Summary

**All seven pinned narrow-phase pair families now produce ordered semantic manifolds through private clipping, adjacency-aware edge classification, explicit canonical reversal, and active-only world conversion.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-11T16:52:47Z
- **Completed:** 2026-07-11T17:17:40Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added the private two-point Sutherland-Hodgman clip kernel with exact retained/crossing order and source feature assignment.
- Added fixed-capacity semantic add/persist/remove transitions without packed contact keys or unordered matching.
- Translated circle-circle, polygon-circle, polygon-polygon, and world-manifold branches with strict tie behavior, reference hysteresis, point order, feature swaps, and A-to-B normals.
- Translated edge-circle endpoint ownership plus the adjacency-aware edge-polygon EPCollider front/back, normal-limit, axis, hysteresis, and clipping behavior.
- Added one public closed dispatcher for exactly circle-circle, polygon-circle, polygon-polygon, edge-circle, edge-polygon, chain-child-circle, and chain-child-polygon, including all registered reversals.
- Proved the surface with 15 public manifold integration tests and 7 private clipping/reference/edge branch tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement ordered clipping and feature-state transitions** - `2ef7bc9` (`feat`)
2. **Task 2: Implement circle and polygon manifold kernels** - `de42a81` (`feat`)
3. **Task 3: Implement adjacency-aware edge kernels and closed pair registry** - `57c00f3` (`feat`)

## Files Created/Modified

- `crates/liquidfun/src/collision/narrow.rs` - Curates public manifold kernels, active world conversion, point transitions, pair orientation, and the closed dispatcher.
- `crates/liquidfun/src/collision/narrow/clipping.rs` - Owns the private bounded clip vertex and exact source-ordered line clipping.
- `crates/liquidfun/src/collision/narrow/circle.rs` - Implements circle-circle and polygon-circle region branches.
- `crates/liquidfun/src/collision/narrow/polygon.rs` - Implements max-separation selection, incident edges, reference hysteresis, clipping, and feature flips.
- `crates/liquidfun/src/collision/narrow/edge.rs` - Implements endpoint ownership and the adjacency-aware edge-polygon collider.
- `crates/liquidfun/tests/collision_manifolds.rs` - Exercises public clipping capacity, persistence, all manifold families, reversals, adjacency, chain delegation, and world values.

## Decisions Made

- Public kernels return `Result<Option<Manifold>, CollisionError>`: invalid transforms remain typed errors, while a valid supported pair with no contact remains explicit separation.
- `WorldManifold` owns only active `WorldManifoldPoint` values. No inactive array slots or solver impulses enter the Phase 5 contract.
- Polygon reference selection preserves strict `separation_b > separation_a + 0.1 * LINEAR_SLOP`; equality stays face A. Edge-polygon selection preserves strict `polygon > 0.98 * edge + 0.001`; equality stays edge A.
- Reversed asymmetric pairs invoke the same primary kernel with operands swapped and return `PairOrientation::Reversed`. Their canonical manifolds and semantic features therefore match the primary invocation exactly.
- Checked chain children materialize the existing owned adjacency-bearing `EdgeShape` and delegate to the exact edge kernel; invalid foreign child coordinates fail before dispatch.
- The 598-line edge module was reviewed during the simplification pass. Its length follows the cohesive pinned EPCollider state machine and nearby classification helpers; splitting that state across files would obscure the source mapping without reducing behavior.

## TDD Evidence

- **RED Task 1:** Integration tests initially lacked point-state and clipping-capacity operations.
- **GREEN Task 1:** Named clipping and point-state filters plus all private clip-order tests passed.
- **RED Task 2:** Circle, polygon, and world-manifold imports were unresolved before the kernels existed.
- **GREEN Task 2:** Named circle, polygon, world-manifold, and exact reference-hysteresis tests passed.
- **RED Task 3:** Edge and registry operations were unresolved before the EPCollider and closed dispatcher existed.
- **GREEN Task 3:** Named edge and pair-registry filters, chain delegation, exact axis hysteresis, front/back, convex/concave adjacency, endpoint ownership, and transition tests passed.
- **REFACTOR:** Kept clipping private, factored exact hysteresis predicates into named helpers, and exposed one canonical dispatcher rather than per-reversal implementations.

## Deviations from Plan

### Cross-Plan Regression Repair

- **Found during:** Task 2 mandatory full-suite gate
- **Issue:** The randomized Plan 05-03 property `gjk_circle_symmetry_preserves_distance` found a deterministic large-coordinate witness where radii-adjusted witness reconstruction exceeded its rounding bound.
- **Disposition:** Plan 05-04 paused without hiding the failure. The root cause was fixed in the separate Plan 05-03 commit `dd7c222`, with deterministic regression coverage and a 30,000-case property run, before Plan 05-04 resumed.
- **Impact:** No narrow-phase scope or files were mixed into the repair; every Plan 05-04 commit remained atomic and passed against the corrected distance foundation.

## Issues Encountered

- The first full Task 2 gate surfaced the cross-plan GJK witness above. The generated proptest sidecar was not retained as an unexplained artifact; its minimized values were folded into stable Plan 05-03 coverage.
- Strict Clippy required public `# Errors` documentation, tolerance-based test assertions for derived floats, and a less ambiguous local name in the edge collider.
- `cargo package` reports the repository's existing warnings that integration tests are excluded by the publish include list. The verified 47-file package contains native Rust and Cargo/documentation inputs only.

## Verification

- `cargo test -p liquidfun --test collision_manifolds --all-features` passed all 15 public manifold tests.
- Every task commit was preceded by `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests in the mandated order.
- The final full gate passed 124 library tests, all integration suites including 15 manifold and 12 distance tests, and 9 rustdoc compile-fail tests.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps` passed.
- `cargo package -p liquidfun --allow-dirty` packaged and verified successfully.
- Forbidden packed-key, union, unordered point handling, impulse, and unsafe scans passed.
- `git diff --check` passed, and `.planning/STATE.md` plus `.planning/config.json` remained outside every commit.

## Requirement Scope

- `COLL-03` now includes ordered clipping, semantic point persistence, circle/polygon/edge/chain-child manifolds, world conversion, and the exact supported pair registry.
- `COLL-07` has named exact-boundary, ordering, identity, adjacency, reversal, invalid-child, and state-transition evidence.
- Persistent contacts, warm-start impulses, listener timing, and world-owned contact lifecycle remain Phase 6 responsibilities.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 05-07 can serialize deterministic manifold kind, points, semantic features, orientation, world values, and point transitions without exposing private clip state.
- Phase 6 can consume canonical pair outcomes and semantic feature transitions for persistent contact identity while adding solver-owned impulses separately.

## Self-Check: PASSED

- Implementation commits `2ef7bc9`, `de42a81`, and `57c00f3` exist in history.
- All six planned source/test artifacts exist on disk.
- Full Rust, manifold, rustdoc, package, forbidden-pattern, and diff checks passed.
- Only pre-existing orchestrator changes to `.planning/STATE.md` and `.planning/config.json` remain outside the plan commits.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_

---
phase: 05-shapes-and-collision-foundation
plan: "02"
subsystem: collision-shapes
tags: [rust, collision, shapes, convex-hull, ray-cast, property-tests]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "01"
    provides: checked AABB, mass, ray, child-index, and collision-error values
  - phase: 04-math-settings-and-numerical-policy
    provides: source-ordered f32 math, transforms, and pinned collision settings
provides:
  - immutable owned circle, edge, polygon, and open/closed chain values
  - exhaustive static Shape dispatch for every unary query
  - source-ordered polygon hull, normals, centroid, mass, and ray behavior
  - canonical chain vertices with checked owned adjacency-bearing child edges
affects: [05-03-distance, 05-04-narrow-phase, 05-06-toi, phase-6-rigid-world]
tech-stack:
  added: []
  patterns: [owned immutable topology, validate-then-commit shapes, static enum dispatch, source-ordered hull construction]
key-files:
  created:
    - crates/liquidfun/src/collision/shape/circle.rs
    - crates/liquidfun/src/collision/shape/edge.rs
    - crates/liquidfun/src/collision/shape/polygon.rs
    - crates/liquidfun/src/collision/shape/chain.rs
    - crates/liquidfun/tests/collision_shapes.rs
  modified:
    - crates/liquidfun/src/collision/shape.rs
key-decisions:
  - "Return a finite zero normal for exact circle-center distance while retaining the pinned signed distance."
  - "Preserve the pinned polygon squared-distance weld comparison against 0.5 * LINEAR_SLOP despite its dimensional quirk."
  - "Store closed-loop vertices once and reconstruct closing-edge adjacency from modular indices."
patterns-established:
  - "Shape boundary pattern: validate finite topology before storing private immutable owned geometry."
  - "Unary query pattern: concrete kernels preserve source operand order and Shape dispatch remains exhaustive and static."
requirements-completed:
  - COLL-02
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T16:06:03Z
duration: 12 min
completed: 2026-07-11
---

# Phase 5 Plan 02: Owned Shapes and Unary Queries Summary

**All four native owned shape types now expose checked construction, deep cloning, source-ordered unary queries, and exhaustive static dispatch without C++ or trait-object runtime dependencies.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-11T15:54:33Z
- **Completed:** 2026-07-11T16:06:03Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added immutable private-representation circle and adjacency-bearing edge values with checked finite geometry, exact mass/AABB/ray behavior, and a documented finite result for the upstream circle-center NaN case.
- Added the pinned polygon weld, rightmost-lowest start, gift-wrapping, farthest-collinear tie, outward normals, centroid, point/ray/AABB, and mass/inertia expression ordering.
- Replaced upstream polygon assertion, excess-input, and substitute-geometry paths with explicit typed rejection, while retaining accepted-input order and arithmetic.
- Added explicit open and closed chain topology, optional ghosts only at open endpoints, canonical loop storage without a repeated closing point, checked children, and owned adjacency-bearing edges.
- Completed exhaustive `Shape` dispatch for radius, child count, point test, distance, ray cast, AABB, mass, and deep clone.
- Added 20 focused public integration/property tests and two module tests covering exact values, boundary behavior, topology, adjacency, rejection, and dispatch equivalence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement circle and adjacency-bearing edge values** - `ebee98a` (`feat`)
1. **Task 2: Implement pinned polygon hull, centroid, normals, and mass** - `e9d31f2` (`feat`)
1. **Task 3: Implement open/closed chains and exhaustive shape dispatch** - `4dfa215` (`feat`)

## Files Created/Modified

- `crates/liquidfun/src/collision/shape.rs` - Defines `PointDistance`, the exhaustive `Shape` enum, validation helpers, and static unary dispatch.
- `crates/liquidfun/src/collision/shape/circle.rs` - Implements checked circle geometry and pinned unary kernels.
- `crates/liquidfun/src/collision/shape/edge.rs` - Implements isolated/adjacent edges, two-sided rays, distance, bounds, and zero mass.
- `crates/liquidfun/src/collision/shape/polygon.rs` - Implements source-ordered hull construction, box constructors, derived topology, validation, and unary kernels.
- `crates/liquidfun/src/collision/shape/chain.rs` - Implements explicit open/closed ownership, ghosts, checked child edges, and delegated queries.
- `crates/liquidfun/tests/collision_shapes.rs` - Exercises public construction, exact query values, safe rejection, properties, adjacency, clone ownership, and enum dispatch.

## Decisions Made

- Exact circle-center distance returns `-radius` and `Vec2::ZERO`; this is finite, named by `PointDistance`, documented, and regression-protected instead of reproducing upstream arithmetic NaN.
- Radius zero remains a valid initialized circle, matching the source's representable geometry and allowing later collision code to make policy decisions explicitly.
- Polygon hull construction accepts no more than `MAX_POLYGON_VERTICES`, performs the source's unusual squared-distance weld comparison unchanged, and rejects any retained hull with fewer than three points.
- Box constructors share the same private derived-state validation while preserving the pinned initial point and normal order directly.
- Closed chains store no ghost fields or duplicate closing point. Modular child reconstruction produces the same previous/next semantic adjacency for every edge.
- Chain AABB, ray, and distance use the exact owned `EdgeShape` child, keeping one authoritative segment-query implementation.

## TDD Evidence

- **RED Task 1:** `cargo test -p liquidfun --test collision_shapes circle --all-features` failed on unresolved `CircleShape`, `EdgeShape`, and `Shape` imports.
- **GREEN Task 1:** Circle/edge focused filters passed after checked immutable representations and kernels were added.
- **RED Task 2:** `cargo test -p liquidfun --test collision_shapes polygon --all-features` failed on the unresolved `PolygonShape` import.
- **GREEN Task 2:** Six focused polygon integration/property tests and the module hull-tie test passed.
- **RED Task 3:** `cargo test -p liquidfun --test collision_shapes chain --all-features` failed on the unresolved `ChainShape` import.
- **GREEN Task 3:** Seven chain-filtered tests plus the shape-dispatch filter passed.
- **REFACTOR:** Narrow source-faithful Clippy allowances document the bounded hull-count cast and exact float tie comparison; all other strict warnings remain denied.

## Deviations from Plan

### Auto-fixed Plan Metadata Omission

- **Found during:** Task 2
- **Issue:** Task 2's `<files>` list omitted `shape.rs`, but its required public `PolygonShape` seam and eventual exhaustive enum dispatch cannot compile without declaring, re-exporting, and dispatching the polygon module there.
- **Fix:** Included the necessary `shape.rs` integration in the Task 2 atomic commit. The file was already inside the plan-level `files_modified` boundary.
- **Impact:** No scope expansion; this was the minimum integration required by the task action and acceptance criteria.

## Issues Encountered

- Strict Clippy initially flagged missing `# Errors` sections, the deliberately exact polygon float tie, the bounded hull-count cast, and one exact float assertion. Public docs, narrow rationale comments, and bitwise test comparison resolved each issue before commit.
- `cargo package` reports the repository's existing warnings that integration tests are excluded by the publish include list. The package still built and verified successfully with 39 Cargo/native-Rust files and no C++ dependency.

## Verification

- `cargo test -p liquidfun --test collision_shapes --all-features` passed all 20 public shape tests.
- `cargo test -p liquidfun shape:: --all-features` passed both shape module tests.
- Each task commit was preceded by `cargo fmt --all`, strict workspace Clippy, all-target/all-feature build, and all-feature tests in the mandated order.
- Final `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps` passed.
- `cargo package -p liquidfun --allow-dirty` packaged and verified successfully.
- Forbidden unsafe/trait-object/borrowed-topology and polygon replacement-algorithm scans passed.
- `git diff --check` and the three task-commit diff audit passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 05-03 can construct validated shape-child distance proxies from immutable concrete topology and checked `ChildIndex` values.
- Plan 05-04 can consume stable polygon normals/order and exact edge/chain adjacency without defending against malformed public state.
- Plan 05-06 can reuse the same exhaustive child geometry and unary-query conventions for TOI inputs.

## Self-Check: PASSED

- Task commits `ebee98a`, `e9d31f2`, and `4dfa215` exist in history.
- All six planned source/test artifacts exist on disk.
- The full Rust, rustdoc, package, forbidden-pattern, and diff checks passed.
- Only pre-existing orchestrator changes to `.planning/STATE.md` and `.planning/config.json` remain outside the plan commits.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_

---
phase: 05-shapes-and-collision-foundation
plan: "05"
subsystem: collision-broad-phase
tags: [rust, dynamic-tree, broad-phase, aabb, filtering, generational-identity]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "01"
    provides: checked AABB and ray-input values plus collision module seams
  - phase: 03-rust-object-model-and-storage-architecture
    provides: scoped opaque identity and generation-retirement architecture
provides:
  - generic source-ordered dynamic AABB tree with opaque scoped proxy identity
  - borrow-scoped AABB and ray visitors with checked closed controls
  - exact broad-phase move buffering, private-slot pair ordering, and deduplication
  - pure filter and refilter/touch reconsideration substrate
affects: [05-07-differential-evidence, phase-6-rigid-world, collision-queries]
tech-stack:
  added: []
  patterns: [opaque generational identity, private ordering coordinate, visitor control enum, sort-then-dedup candidates]
key-files:
  created:
    - crates/liquidfun/src/collision/tree/pool.rs
    - crates/liquidfun/src/collision/tree/traversal.rs
    - crates/liquidfun/tests/collision_broad_phase.rs
  modified:
    - crates/liquidfun/src/collision/tree.rs
    - crates/liquidfun/src/collision/broad_phase.rs
key-decisions:
  - "Keep ProxyId tree-scoped and generational while reserving pool coordinates exclusively for source-faithful topology and pair order."
  - "Expose ordinary query and ray collections as unspecified-order unique sets while retaining child-2-first traversal only as diagnostic evidence."
  - "Keep broad-phase candidate reporting separate from the pure filter decision; refiltering replaces data and touches the proxy for reconsideration."
patterns-established:
  - "Tree boundary pattern: validate full proxy identity before payload, movement, filter, or lifecycle access."
  - "Broad-phase pattern: retain duplicate move occurrences, tombstone all destroyed occurrences, then stable-sort and adjacent-deduplicate candidate pairs."
requirements-completed:
  - COLL-04
  - COLL-05
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T16:28:01Z
duration: 10 min
completed: 2026-07-11
---

# Phase 5 Plan 05: Dynamic Tree, Broad Phase, and Filtering Summary

**A native generic dynamic AABB tree now provides safe opaque proxy lifecycle, checked visitor traversal, exact candidate ordering, and pure refilter reconsideration without crossing into world lifecycle work.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-11T16:18:05Z
- **Completed:** 2026-07-11T16:28:01Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added a 16-node source-faithful pool with doubling growth, LIFO reuse, checked generation advancement, permanent retirement on exhaustion, and no public storage coordinate.
- Implemented insertion costs, child-2 equality descent, child-2 equality rotations, removal/reinsertion, fat movement prediction, origin shift, validation, height, balance, and area-ratio metrics.
- Added borrow-scoped query and ray visitors with closed continue/stop and ignore/terminate/clip controls, interval validation, and unordered unique collection helpers.
- Added duplicate-preserving move/touch buffering, destruction tombstones, exact private-slot pair sorting, adjacent deduplication, and ordered borrow-scoped pair callbacks.
- Added exact filter defaults, signed equal-group override, symmetric masks, filter replacement, and touch-driven reconsideration while leaving world-owned lifecycle behavior to Phase 6.
- Proved the surface with 24 public integration tests and 8 focused tree/pool/traversal unit tests, including seeded create/move/destroy histories.

## Task Commits

Each implementation task was committed atomically:

1. **Task 1: Implement opaque proxy identity and source-faithful node pool/tree** - `a45ab2a` (`feat`)
2. **Task 2: Implement borrow-scoped query and ray visitors** - `d0469f1` (`feat`)
3. **Task 3: Implement move/touch pair generation and pure filtering/refiltering** - `1b0aa76` (`feat`)
4. **Acceptance audit: Add named equality, randomized lifecycle, and metric-forwarding witnesses** - `14cba81` (`test`)

## Files Created/Modified

- `crates/liquidfun/src/collision/tree.rs` - Owns opaque proxy identity, checked errors, topology, balancing, movement, validation, and metrics.
- `crates/liquidfun/src/collision/tree/pool.rs` - Owns private node coordinates, free-list growth/reuse, generations, and retirement.
- `crates/liquidfun/src/collision/tree/traversal.rs` - Implements borrow-scoped AABB and ray visitors plus unordered collection helpers.
- `crates/liquidfun/src/collision/broad_phase.rs` - Implements move buffering, ordered candidate reporting, filtering, refiltering, and metric forwarding.
- `crates/liquidfun/tests/collision_broad_phase.rs` - Exercises public lifecycle, movement, visitors, ordering, filtering, and randomized invariants.

## Decisions Made

- `DynamicTree::new` is fallible so process-unique tree-key exhaustion remains explicit rather than wrapping or panicking.
- Public `ProxyId` equality and hashing cover tree key, private coordinate, and generation, while `Debug` remains opaque and no raw constructor, serialization, or `Ord` promise exists.
- Tree arithmetic rejects non-finite overflow at the safe boundary; valid operations retain the pinned operand order and movement-sign branches.
- Query and ray callbacks borrow payloads only during each call. The implementation preserves child-1-then-child-2 pushes, but consumer documentation explicitly denies result-order guarantees.
- Broad-phase callbacks report potential overlap candidates regardless of filtering. `FilterData::should_collide` is a separate pure decision, matching the upstream separation and keeping later world ownership out of this module.
- The 660-line tree entrypoint was reviewed during the simplification pass. Its topology responsibility remains cohesive, while pool and traversal logic are already split into the two planned child modules; an additional unplanned file would not improve the current boundary.

## TDD Evidence

- **RED Task 1:** The focused integration test failed on unresolved `DynamicTree` and `TreeError` imports before the tree and pool existed.
- **GREEN Task 1:** Seven initial lifecycle/movement/metric tests and three pool tests passed; the final acceptance audit added named equality and seeded lifecycle witnesses.
- **RED Task 2:** Query/ray tests failed on unresolved `query`, `query_ids`, `ray_cast`, and `ray_candidate_ids` methods.
- **GREEN Task 2:** Query, ray, invalid-clip, collection-set, and internal diagnostic traversal tests passed.
- **RED Task 3:** Broad-phase tests failed on unresolved `BroadPhase` and `FilterData` imports.
- **GREEN Task 3:** Pair, duplicate, tombstone, containment, every filter branch, refilter, and metric-forwarding tests passed.
- **REFACTOR:** Replaced exact decimal literals with source-ordered bit witnesses, isolated equality choices in named helpers, and kept internal coordinates out of every public signature.

## Deviations from Plan

### Acceptance Audit Commit

- **Found during:** Plan-level verification
- **Issue:** The first three task commits implemented the required behavior, but the final checklist called for more explicitly named equality, randomized lifecycle, and broad-phase metric-forwarding witnesses.
- **Fix:** Added one focused test-only acceptance audit commit, plus the minimal metric forwarding methods required by the planned Task 3 surface.
- **Impact:** No scope expansion, dependency change, or extra production file; the added evidence stays within all five planned files.

## Issues Encountered

- Strict Clippy required public panic documentation for impossible internal-invariant failures and bitwise comparison for exact `f32` clip assertions. Both were resolved before any affected commit.
- `cargo package` reports the repository's existing warnings that integration tests are excluded from the published include list. The verified 41-file package contains only Cargo metadata, documentation/license files, and native Rust sources.

## Verification

- All plan-focused tree, pool, query, ray, pair, and filter commands passed.
- Every implementation and acceptance-audit commit was preceded by `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests in the mandated order.
- The final full gate passed 101 library tests, 24 broad-phase integration tests, all existing integration suites, and 8 rustdoc compile-fail tests.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps` passed.
- `cargo package -p liquidfun --allow-dirty` packaged and verified successfully.
- Forbidden public-slot/raw-constructor/serialization/unsafe and world-lifecycle scans passed.
- `git diff --check` passed, and `.planning/STATE.md` plus `.planning/config.json` remained outside every commit.

## Requirement Scope

- `COLL-04` is implemented for proxy lifecycle, movement, queries, rays, metrics, and private tie behavior.
- Phase 5's candidate generation, pure filtering, and refilter/touch portion of `COLL-05` is implemented. World-owned creation, persistence, removal, wake state, joint suppression, and listener timing remain explicitly pending Phase 6.
- `COLL-07` now has focused and seeded evidence for identity, topology, ordering, deduplication, movement, controls, filtering, and state preservation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 05-07 can add closed tree and broad-phase differential probes without changing consumer identity or ordering contracts.
- Phase 6 can consume ordered potential pairs and pure filter decisions while owning all persistent world state and timing.

## Self-Check: PASSED

- Implementation commits `a45ab2a`, `d0469f1`, `1b0aa76`, and `14cba81` exist in history.
- All five planned production/test artifacts exist on disk.
- Full Rust, rustdoc, package, forbidden-pattern, and diff checks passed.
- Only pre-existing orchestrator changes to `.planning/STATE.md` and `.planning/config.json` remain outside the plan commits.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_

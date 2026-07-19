---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "11"
subsystem: particles
tags: [rust, voronoi, particle-topology, determinism, property-testing, resource-bounds]

requires:
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    plan: "01"
    provides: "Checked particle-group recipes and private particle storage foundations"
  - phase: 05-shapes-and-narrow-phase
    provides: "Compatibility Vec2 arithmetic and finite geometry validation"
provides:
  - "Pure bounded Voronoi ownership planning with dense generator ordinals"
  - "Pinned FIFO propagation, strict distance tie retention, and row-major oriented node emission"
  - "Typed pre-allocation generator, grid, queue, work, node, arithmetic, and allocation failures"
affects: [10-12-topology-records, elastic-particle-triads, compatibility-evidence]

tech-stack:
  added: []
  patterns:
    - "Necessary generators define bounded grid extents while every dense generator remains eligible to own cells"
    - "A bounded VecDeque preserves left/down/right/up FIFO propagation without hash or sort order"
    - "Preflight limits conservatively bound cells, live queue tasks, total work, and possible nodes before allocation"

key-files:
  created:
    - crates/liquidfun/src/particle/topology.rs
    - crates/liquidfun/src/particle/topology/voronoi.rs
    - crates/liquidfun/src/particle/topology/voronoi/tests.rs
  modified:
    - crates/liquidfun/src/particle.rs

key-decisions:
  - "Use dense generator ordinal as the only tie identity and replace an incumbent only for a strictly smaller squared distance."
  - "Emit source-oriented triples directly in row-major cell order without canonicalization, sorting, or deduplication."
  - "Separate facade limits/errors, the pinned algorithm, and private test evidence so each file remains below the repository split trigger."

patterns-established:
  - "Topology planning is a pure private boundary with no storage, identity, callback, or world mutation."
  - "Finite but extreme geometry is rejected through typed checked bounds before grid or queue allocation."

requirements-completed: [PART-11, TEST-01, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T19:57:57Z

duration: 1h 25m
completed: 2026-07-19
---

# Phase 10 Plan 11: Pinned Bounded Voronoi Topology Summary

**Pure bounded Voronoi planning reproduces LiquidFun seed, queue, tie, and oriented node order while rejecting excessive geometry and work before allocation**

## Performance

- **Duration:** 1h 25m
- **Started:** 2026-07-19T18:33:15Z
- **Completed:** 2026-07-19T19:57:57Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Transcribed the pinned necessary-generator bounds, dense source-order seeding, left/down/right/up FIFO flood, boundary relaxation, and strict squared-distance replacement rules.
- Emitted the exact two source-oriented node candidates per row-major cell and filtered only triples lacking a necessary generator.
- Added checked pre-allocation limits for generators, grid cells, queue capacity, conservative work, possible nodes, arithmetic, and allocation failure.
- Added exact ordered-node, duplicate/tie, neighbor-order, necessary-filter, small-input, and typed-failure fixtures plus two 128-case deterministic property tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement exact bounded Voronoi generation** - `fc7f6ff`

## Files Created/Modified

- `crates/liquidfun/src/particle/topology.rs` - Private topology facade with bounded Voronoi policy limits and typed errors.
- `crates/liquidfun/src/particle/topology/voronoi.rs` - Pure pinned Voronoi grid, queue, relaxation, and oriented-node algorithm.
- `crates/liquidfun/src/particle/topology/voronoi/tests.rs` - Fixed source-order witnesses and bounded deterministic properties.
- `crates/liquidfun/src/particle.rs` - Private topology module seam for compilation and later Phase 10 consumers.

## Decisions Made

- Retain the first incumbent on equal squared distance by replacing ownership only for strict improvement.
- Preserve dense generator ordinal, queue insertion order, row-major cell traversal, and oriented triples without canonicalization.
- Split policy/errors and tests from the algorithm after the explicit simplification pass; the largest source file is 622 lines and every function remains below the 161-line review trigger.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired the private topology facade into the particle module**

- **Found during:** Task 1 (Implement exact bounded Voronoi generation)
- **Issue:** The plan created `particle/topology.rs` but did not list the parent `particle.rs` module declaration, so the new implementation and private tests could not compile or run.
- **Fix:** Added one private `mod topology;` declaration with a scoped dead-code allowance because later Phase 10 plans consume the planner.
- **Files modified:** `crates/liquidfun/src/particle.rs`
- **Verification:** Focused topology tests and the complete ordered Rust gate pass.
- **Committed in:** `fc7f6ff`

**Total deviations:** 1 auto-fixed (1 blocking issue)
**Impact on plan:** The single private integration seam was required to compile and verify the planned module; no public API or scope expansion was introduced.

## Issues Encountered

- The tests-first RED run failed with 32 unresolved topology symbols as expected before the implementation was added.
- The first implementation compile exposed Rust struct-variant constructor syntax, and the first focused run exposed a fixture whose zero margin excluded optional generators from necessary-generator-derived bounds. Both were corrected before the all-green task commit.
- Strict Clippy required field names without a redundant common prefix and removal of lossy comparison casts.
- Shared-workspace Cargo locks and serialized macOS integration binaries made the exact full gate slow. The required commands were kept intact and completed successfully.
- The focused property lane used `--lib` to run the private topology suite directly; both properties completed with `PROPTEST_CASES=128`, and the unfiltered full all-feature suite passed afterward.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-12 can map dense generator ordinals to candidate particle indices and apply pair/triad filtering without changing Voronoi ordering.
- No public API, unsafe code, FFI, network endpoint, authentication path, file-access boundary, schema change, hash ordering, sort, or parallel traversal was introduced.
- No blockers remain.

## Self-Check: PASSED

- All four created or modified source paths exist.
- Task commit `fc7f6ff` exists on the current branch.
- Eleven fixed tests and two 128-case bounded properties pass.
- The ordered Rust gate passes: format, strict Clippy, all-target/all-feature build, 284 library tests, all integration binaries, and 19 doctests.
- Stub and threat-surface scans found no goal-blocking stubs or unplanned trust-boundary changes.

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

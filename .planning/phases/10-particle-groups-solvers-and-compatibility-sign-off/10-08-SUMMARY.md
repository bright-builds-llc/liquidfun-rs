---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "08"
subsystem: particles
tags: [rust, particle-groups, sampling, determinism, property-testing, resource-bounds]

requires:
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    plan: "01"
    provides: "Checked owned particle-group recipes for filled shapes, strokes, and explicit positions"
  - phase: 05-shapes-and-narrow-phase
    provides: "Validated shape point tests, child-edge access, transforms, AABBs, and compatibility math"
provides:
  - "Pure bounded sample plans for filled-shape unions, edge and chain strokes, and explicit positions"
  - "Exact source-ordered traversal, transform timing, and grouped initial-velocity arithmetic"
  - "Typed pre-allocation work, capacity, arithmetic, geometry, position, velocity, and allocation failures"
affects: [10-09-group-world-api, particle-group-creation, compatibility-evidence]

tech-stack:
  added: []
  patterns:
    - "Two-pass sampling validates and counts every output before exact materialization"
    - "Filled union membership short-circuits in source order while grid traversal remains y-outer and x-inner"
    - "Stroke sampling carries one source-compatible distance accumulator across chain children"

key-files:
  created:
    - crates/liquidfun/src/particle/group/sampling.rs
  modified:
    - crates/liquidfun/src/particle/group.rs

key-decisions:
  - "Preflight source-derived work and effective particle capacity before output allocation, then materialize into an exactly reserved owned sample plan."
  - "Keep the fixed/property oracle evidence co-located with the private sampler: the production portion remains below the repository split trigger and one file keeps exact source-order witnesses beside their implementation."

patterns-established:
  - "Sampling is a pure planning boundary with no world, storage, identity, callback, or lifecycle effects."
  - "Derived world positions and initial velocities are checked for finiteness before any sample output is allocated."

requirements-completed: [PART-09, TEST-01, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T18:20:56Z

duration: 1h 32m
completed: 2026-07-19
---

# Phase 10 Plan 08: Source-Ordered Particle-Group Sampling Summary

**Pure two-pass samplers reproduce LiquidFun fill-union, carried-chain stroke, and explicit-position order while rejecting excessive or non-finite work before allocation**

## Performance

- **Duration:** 1h 32m
- **Started:** 2026-07-19T16:49:04Z
- **Completed:** 2026-07-19T18:20:56Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Implemented exact filled-union sampling with `floor(lower / stride) * stride`, strict upper bounds, y-outer/x-inner traversal, source-order union membership, and overlap deduplication.
- Implemented edge and chain stroke sampling with one `positionOnEdge` accumulator carried across child edges instead of resetting at every child.
- Preserved explicit-position input order, applied transforms after local membership or source selection, and retained the pinned `linearVelocity + cross(angularVelocity, worldPosition - group.position)` expression grouping.
- Added two-pass work/capacity/finiteness validation, checked arithmetic, typed allocation failure, ten fixed regression cases, and two bounded property tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement and verify source-ordered sampling** - `32d1765`

## Files Created/Modified

- `crates/liquidfun/src/particle/group/sampling.rs` - Private pure sample planning, typed bounds/errors, fixed witnesses, and bounded property evidence.
- `crates/liquidfun/src/particle/group.rs` - Crate-private sampling module seam for the later transactional world integration.

## Decisions Made

- Count and validate outputs in a first pass, enforce effective capacity, then reserve exactly and repeat the deterministic traversal for materialization.
- Charge filled-union work for both validation and materialization point tests, and charge strokes for both passes plus child traversal.
- Keep the 854-line sampler intact because about 350 lines are required co-located fixed/property oracle evidence; the production implementation remains within the repository's 300-500-line split guidance, and separating evidence would weaken source-order auditability without producing a deeper module.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The initial strict-Clippy pass rejected unchecked float-to-integer conversions. Bounds are now proven before a narrow conversion, while wider axis and product calculations use checked integer conversions.
- Shared-workspace Cargo locks and macOS executable policy checks delayed the focused RED run and many integration binaries. The exact required commands remained unchanged and completed successfully.
- The focused property lane used `--lib` so the private co-located sampler tests ran directly without launching unrelated integration binaries; both properties completed with `PROPTEST_CASES=128`, and the unfiltered full all-feature suite passed afterward.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-09 can consume `SamplePlan` only after same-world/same-system validation and can atomically commit stable particle/group identities after sampling succeeds.
- No world state, storage, identity, callbacks, lifecycle effects, unsafe code, FFI, network surface, file-access boundary, or schema boundary was introduced.
- No blockers remain.

## Self-Check: PASSED

- Created and modified source files exist.
- Task commit `32d1765` exists on the current branch.
- Ten fixed tests and two 128-case bounded properties pass.
- The ordered Rust gate passes: format, strict Clippy, all-target/all-feature build, full all-feature tests, and 19 doctests.
- Stub and threat-surface scans found no goal-blocking stubs or unplanned trust-boundary changes.

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*

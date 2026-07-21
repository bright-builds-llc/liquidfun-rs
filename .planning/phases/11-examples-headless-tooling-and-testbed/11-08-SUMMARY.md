---
phase: 11-examples-headless-tooling-and-testbed
plan: "08"
subsystem: public-observability
tags: [diagnostics, observations, profiling, renderer-neutral, bounded-collections]
requires:
  - phase: 08-joints-rope-callbacks-and-rigid-sign-off
    provides: Exact world counts, tree metrics, owned contact snapshots, and source-ordered step reports
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    provides: Stable particle views, contact translation, and semantic statistics
provides:
  - Default-feature exact world diagnostics and bounded owned semantic observations
  - Stable rigid, particle, fixture, body, system, and child identities without storage coordinates
  - Separate closed-phase wall-clock profiles excluded from equality, hashing, and checkpoints
affects: [phase11-checkpoints, phase11-debug-draw, phase11-headless-runner, phase11-testbed]
tech-stack:
  added: []
  patterns:
    - Preflight every owned observation collection before allocation or translation
    - Keep nondeterministic wall-clock diagnostics separate from deterministic step evidence
key-files:
  created:
    - crates/liquidfun/src/world/observation.rs
    - crates/liquidfun/src/world/observation/profile.rs
    - crates/liquidfun/tests/world_observations.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/world/diagnostics.rs
    - crates/liquidfun/src/world/step.rs
key-decisions:
  - "Expose tight current fixture-child AABBs with stable body/fixture/child identities while keeping fat-tree bounds and proxy identities private."
  - "Make profile phase names structural but keep phase timings and the aggregate profile non-comparable, non-hashable, and outside StepReport."
  - "Preflight rigid contacts, particle systems, particle identities, particle contacts, body contacts, and fixture-child observations against independently reviewed limits."
patterns-established:
  - "Observation boundary: count and validate first, then allocate owned records and translate every private coordinate to a semantic ID."
  - "Profile boundary: opt in through step_profiled; ordinary step uses the same internal transaction with timing disabled."
requirements-completed: [RIGD-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-21T23:18:12Z
duration: 12 min
completed: 2026-07-21
---

# Phase 11 Plan 08: Stable Public World Observations Summary

**Default-feature consumers can now collect bounded renderer-neutral rigid and particle observations through stable semantic identities, while opt-in wall-clock phase timing remains structurally separate from deterministic step evidence.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-21T23:06:21Z
- **Completed:** 2026-07-21T23:18:12Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Made exact body, fixture, joint, contact, manifold-point, proxy, and tree metrics available without the non-default differential feature.
- Added bounded owned rigid contacts, particle-pair contacts, particle-body contacts, fixture-child AABBs, per-system particle statistics, and aggregate particle statistics in documented source-significant order.
- Added `step_profiled` with six closed diagnostic phase names and wall-clock `Duration` values that cannot participate in profile equality, hashing, `StepReport`, or checkpoint conversion.
- Added default-feature public integration coverage for stable identities, source order, inclusive AABB query semantics, exact-limit acceptance, first-excess rejection, and ordinary/profiled report equivalence.

## TDD Evidence

- **RED:** The new default-feature `world_observations` target failed with unresolved observation/profile types and missing `World::world_observation` and `World::step_profiled` methods.
- **GREEN:** The bounded observation model, public diagnostics export, opt-in profiler, and three behavior-focused integration tests were implemented; focused and complete repository gates pass.
- **REFACTOR:** Collection was separated into limit preflight, particle translation, and broad-phase translation helpers so the pure boundary remains reviewable and strict Clippy passes without a size suppression.

## Task Commits

Each task was committed atomically:

1. **Task 1: Make layered observability a documented default-feature API** - `fac192c` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun/src/world/observation.rs` - Owns public observation records, reviewed limits, typed errors, preflight, stable-ID translation, and source-ordered collection.
- `crates/liquidfun/src/world/observation/profile.rs` - Owns closed phase names and bounded-count diagnostic-only wall-clock timings.
- `crates/liquidfun/tests/world_observations.rs` - Proves the complete API through default features and all features.
- `crates/liquidfun/src/world/diagnostics.rs` - Keeps reconstruction feature-private while making curated counts and tree metrics public by default.
- `crates/liquidfun/src/world/step.rs` - Routes ordinary and profiled stepping through one transaction, with timing disabled on the ordinary path.
- `crates/liquidfun/src/world.rs` and `crates/liquidfun/src/lib.rs` - Curate and document the new public surface.

## Decisions Made

- Broad-phase observations expose recomputed tight current fixture-child AABBs and an inclusive `overlaps` helper. Private fat-tree bounds, proxy IDs, nodes, and traversal coordinates remain inaccessible.
- Observation limits are independent per semantic collection and cannot exceed reviewed hard maxima; live counts are preflighted before owned output allocation.
- Profile phase identity may be inspected and compared by its closed enum, but timing records and profiles intentionally omit `PartialEq` and `Hash` and have no checkpoint conversion.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and were not committed or reverted.

## Security Verification

- T-11-08-01: Every private row/contact/proxy seam is translated to stable owned public identities, with collection preflight before output allocation and typed invariant failures.
- T-11-08-02: Wall-clock profiles are separate from `StepReport`, do not implement equality or hashing, and expose no deterministic-checkpoint conversion.
- T-11-08-03: Rigid contacts, particle systems, particles, particle contacts, particle-body contacts, fixture-child observations, and profile phase count all have explicit reviewed bounds.
- T-11-08-04: Errors expose only closed semantic resource categories and limits; no private index, pointer, raw record, stack trace, secret, or unbounded diagnostic content crosses the boundary.
- T-11-08-05: No renderer, C++, protocol, process, network, or new dependency entered the published crate.
- T-11-08-06: RED evidence, focused tests, the complete ordered workspace gate, and atomic commit `fac192c` provide attributable review evidence.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Known Stubs

None.

## Requirements Status

The frontmatter preserves Plan 11-08's `RIGD-10` mapping. The global requirement checkbox remains pending until later Phase 11 plans add and verify the renderer-independent debug-draw primitive surface and final phase integration; this plan does not claim that broader closure early.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Canonical checkpoint and debug-draw plans can consume one bounded stable-ID observation surface instead of reading private world or particle storage.
- Headless and visual adapters can display exact counts, contacts, broad-phase AABBs, particle statistics, and diagnostic timings without coupling simulation to a renderer.
- Plan 11-02 remains the earliest incomplete Phase 11 plan; wave-order execution state must continue to report it as pending.

## Self-Check: PASSED

- Confirmed all three created files and four modified source files exist.
- Confirmed task commit `fac192c` exists and contains only the seven Plan 11-08 artifacts.
- Confirmed the focused default-feature test, all three named all-feature integration targets, and the exact ordered full-workspace format, Clippy, build, test, and doctest gate pass with `/tmp/liquidfun-rs-phase11-11-08`.
- Confirmed no known stub or unplanned threat surface was introduced and all four unrelated shared-tree edits remain unstaged.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*

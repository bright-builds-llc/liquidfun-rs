---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "20"
subsystem: particle-protocol-validation
tags: [rust, cpp, particles, protocol, lifecycle, result-validation]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "12"
    provides: bounded Phase 9 declarations, actions, observations, and native/C++ adapters
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "19"
    provides: authoritative pending deletion, compaction, and creation outcome semantics
provides:
  - declaration-aware system and particle request lifecycle validation before either engine runs
  - one exact action-specific Phase 9 result observation contract with semantic state replay
  - Rust/C++ parity for finite negative lifetimes, mixed identities, statistics capacity, and explicit compaction
affects: [09-21, 09-22, 09-23, 09-24, phase-10]
tech-stack:
  added: []
  patterns: [validated lifecycle replay, declaration-ordered semantic identity, action-specific result matching]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-differential/src/rigid_world/phase9.rs
    - crates/liquidfun-differential/tests/particle_protocol.rs
    - crates/liquidfun-differential/tests/particle_oracle.rs
    - tools/reference/src/rigid_world_phase9_decode.hpp
    - tools/reference/src/rigid_world_phase9_execute.hpp
key-decisions:
  - "Replay declared, live, and pending particle identity at the result boundary; non-statistics/query/ray actions require exact declaration-ordered MixedState identities."
  - "Treat growable statistics capacity as the configured maximum, or i32::MAX when unlimited, rather than allocator or initial-allocation capacity."
patterns-established:
  - "Particle protocol validation applies the same lifecycle/ownership model before execution and while consuming result observations."
  - "Native and C++ adapters normalize semantic identity arrays through timeline declaration order."
requirements-completed: [PART-01, PART-02, PART-03, PART-07, PART-08, PART-15, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-16T06:24:47Z
duration: 1h 3m
completed: 2026-07-16
---

# Phase 9 Plan 20: Lifecycle-Aware Particle Protocol Summary

**Phase 9 requests now fail before execution on invalid ownership or lifecycle order, and every accepted result must satisfy the exact observation variant, identity, ordering, and length contract for its particle action.**

## Performance

- **Duration:** 1h 3m
- **Started:** 2026-07-16T05:21:24Z
- **Completed:** 2026-07-16T06:24:47Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added a closed request lifecycle state machine covering declared/created/live/pending systems and particles, owner validation, duplicate and stale use, cross-system ranges, and terminal teardown.
- Accepted every finite lifetime bit pattern, including negative and signed-zero infinite lifetimes, while continuing to reject NaN and both infinities in Rust and C++.
- Added dedicated result replay that maps every particle action to exactly one `MixedState`, `Statistics`, `Query`, or `RayCast` observation contract and rejects fabricated variants, IDs, owners, order, multiplicity, and parallel arrays.
- Made native statistics/query/ray observations semantic rather than placeholder mixed states, normalized native/C++ mixed identities in declaration order, and aligned C++ explicit compaction and statistics capacity with the validated Rust contract.
- Added Rust and raw-process mutation matrices plus native/C++ parity regressions while preserving retained Phase 8 request bytes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Validate Phase 9 request ownership and lifecycle order** - `f2c8e6d` (fix)
1. **Task 3: Mirror protocol acceptance and mixed identity in the pinned C++ adapter** - `279e243` (fix)
1. **Task 2: Match every Phase 9 action to one exact result observation contract** - `c1494cd` (fix)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Replays system/particle creation, ownership, live/pending state, compaction, and teardown while validating requests.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs` - Implements the dedicated action-specific result state and observation matchers.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Integrates Phase 9 state replay into checkpoint action-window validation without changing Phase 8 matching.
- `crates/liquidfun-differential/src/rigid_world/phase9.rs` - Emits semantic statistics, query, ray, and declaration-ordered mixed observations from native execution.
- `crates/liquidfun-differential/tests/particle_protocol.rs` - Covers request and result lifecycle, identity, owner, ordering, multiplicity, variant, and array-length mutations.
- `crates/liquidfun-differential/tests/particle_oracle.rs` - Covers C++ lifecycle rejection, negative lifetime acceptance, and native/C++ mixed identity equality.
- `tools/reference/src/rigid_world_phase9_decode.hpp` - Mirrors Rust request lifecycle and finite-lifetime validation before C++ allocation.
- `tools/reference/src/rigid_world_phase9_execute.hpp` - Emits declaration-ordered identities and aligned statistics/compaction results.

## Decisions Made

- Unused Phase 9 declarations remain valid; only created identities enter the live/pending state machine and every created identity must be torn down by request completion.
- Marking destruction immediately removes a particle from request-visible live use while retaining it in result-visible pending membership until explicit compaction.
- Query and ray observations require unique live-or-pending semantic IDs owned by the optional requested system; ray fractions additionally require equal parallel lengths and finite normalized values.
- Mixed-state bodies and particles are normalized through timeline declaration order in both engines rather than relying on arena, insertion, or unordered-map traversal.
- Growable statistics report the explicit storage limit: configured maximum when present and `i32::MAX` for the pinned unlimited convention.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced native placeholder observations with action-specific results**

- **Found during:** Task 2 (action-specific result validation)
- **Issue:** Native statistics, query, and ray actions executed the correct APIs but discarded their values and always emitted `MixedState`, so the planned exact validator correctly rejected native results.
- **Fix:** Collected stable IDs, statistics fields, query occurrences, ray hits, and exact fraction bits into their declared nested observation variants.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world/phase9.rs`
- **Verification:** Focused result tests, full particle protocol tests, and the mandatory repository gate pass.
- **Committed in:** `c1494cd`

**2. [Rule 1 - Bug] Aligned C++ capacity reporting and explicit zombie compaction**

- **Found during:** Task 2 cross-boundary regression
- **Issue:** C++ reported growable initial allocation as declared capacity and attempted explicit compaction with `World::Step(0)`, which does not enter upstream zombie solving; both contradicted native semantic results.
- **Fix:** Reported the configured growable maximum/unlimited bound and used the smallest positive `float32` tick to enter upstream zombie compaction below lifetime granularity before discarding invalid handles.
- **Files modified:** `tools/reference/src/rigid_world_phase9_execute.hpp`
- **Verification:** Oracle-debug rebuilt successfully; the full action-family oracle test and complete particle oracle suite pass.
- **Committed in:** `c1494cd`

***

**Total deviations:** 2 auto-fixed (1 blocking native adapter gap, 1 C++ semantic bug).
**Impact on plan:** Both fixes were required to make the planned validated result contract executable and identical across engines; no Phase 10 group or solver behavior was added.

## Issues Encountered

- Concurrent Cargo activity intermittently held the shared target-directory lock, and macOS delayed several test binaries in dynamic-loader startup. Every required gate was allowed to finish; no target was skipped.
- The C++ adapter correctly rejected a stale generated provenance digest after its source changed. Re-running oracle-debug configure regenerated the build identity before the successful rebuild.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- WR-05 through WR-08 and G09-PROTOCOL-VALIDATION now have request, result, native, C++, mutation, and full-suite closure.
- Plan 09-21 can consume validated native/C++ results through a real semantic comparator without arbitrary nested particle payloads crossing the boundary.
- Plans 09-22 through 09-24 can build executable branch evidence on lifecycle-consistent requests and action-specific observations; this plan makes no compatibility promotion claim.

## Self-Check: PASSED

- All three task commits exist and contain plan ID `09-20`.
- Focused request, result, C++ decode, mixed-identity, full particle protocol, and full particle oracle suites pass.
- Oracle-debug configures and builds with every changed adapter input included in the existing manifest.
- Mandatory format, warning-denied Clippy, all-target/all-feature build, all-feature tests, and 16 doctests pass.
- Retained Phase 8 request bytes remain unchanged.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-16*

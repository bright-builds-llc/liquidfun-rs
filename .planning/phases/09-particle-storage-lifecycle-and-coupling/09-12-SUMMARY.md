---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "12"
subsystem: particle-protocol
tags: [rust, particles, jsonl, native-adapter, policy-registry]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "07"
    provides: transactional particle lifetime and zombie maintenance
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "09"
    provides: particle contacts, strict pruning, callbacks, and rigid coupling
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "10"
    provides: checked particle forces, impulses, and semantic statistics
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "11"
    provides: stable-ID particle and mixed-world queries
provides:
  - bounded additive Phase 9 particle declarations, actions, observations, and generated schemas
  - checked native execution for every closed Phase 9 action family
  - phase9-v1 declaration and named numerical-policy registry with fail-closed unknown paths
  - byte-identical retained Phase 8 request serialization and explicit Phase 10 rejection
affects: [09-13, 09-14, phase-10, particle-oracle]
tech-stack:
  added: []
  patterns: [serde-omitted additive child schema, stable-ID native mapping, closed named policy paths]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world/phase9.rs
    - crates/liquidfun-differential/src/rigid_world/phase9.rs
    - crates/liquidfun-differential/tests/particle_protocol.rs
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_world.rs
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
key-decisions:
  - "Keep Phase 9 additive inside the existing rigid-world request and result records through omitted-by-default child fields and tagged variants."
  - "Use stable semantic maps for native particle systems and particles; dense indices and allocator details never enter the protocol."
  - "Close phase9-v1 over an explicit required policy-path list; unknown, wildcard, group, pair/triad, and solver-baseline paths return no policy."
patterns-established:
  - "Each Phase 9 action appends one source-timed closed semantic observation before checkpoint capture."
  - "Earlier rigid serializers remain byte-compatible because empty particle child collections are skipped."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T09:06:03Z
duration: 19 min
completed: 2026-07-15
---

# Phase 9 Plan 12: Closed Particle Protocol and Native Adapter Summary

**The existing rigid-world JSONL protocol now carries bounded Phase 9 particle semantics through stable IDs, checked native execution, source-timed observations, and a closed phase9-v1 policy registry without changing retained Phase 6-8 bytes.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-15T08:47:26Z
- **Completed:** 2026-07-15T09:06:03Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- Added bounded system/particle declarations, buffer modes, actions, lifecycle/contact/statistic/query records, and mixed state to the existing protocol.
- Routed all closed Phase 9 actions through checked public native APIs and captured one semantic observation at each action source point.
- Proved byte-identical Phase 8 request round trips while group topology and every unknown Phase 10 family remain rejected.
- Closed the phase9-v1 policy registry over reviewed exact-discrete, exact-bit, ULP, absolute-relative, and dimensioned-absolute paths with no wildcard fallback.

## Task Commits

1. **Task 1: Add bounded additive Phase 9 schema** - `fbabf9d` (feat)
2. **Task 2: Execute native scenarios and close declarations/policies** - `87df7f4` (feat)
3. **Task 2 regression: Capture source-timed Phase 9 semantic checkpoints** - `75bb838` (test)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs` - Closed Phase 9 wire types and bounds.
- `crates/liquidfun-test-protocol/src/schema/rigid_world/phase9.rs` - Closed generated JSON Schema fragments.
- `crates/liquidfun-differential/src/rigid_world/phase9.rs` - Native action adapter and phase9-v1 policy registry.
- `crates/liquidfun-differential/tests/particle_protocol.rs` - Codec, compatibility, native execution, checkpoint, and fail-closed regressions.
- `protocol/schemas/scenario-v1.schema.json` - Additive request presentation.
- `protocol/schemas/trace-v1.schema.json` - Additive trace presentation.

## Decisions Made

- Additive optional timeline children are omitted when empty so the exact earlier serializer order and bytes remain unchanged.
- Native mappings carry only stable scenario identities and checked public handles; particle dense storage stays private.
- One closed required-path list is the phase9-v1 policy authority, and any missing path is a harness-level absence rather than a tolerance fallback.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended the real protocol authority beyond the abbreviated plan file list**

- **Found during:** Task 1
- **Issue:** The listed schema presentation files could not add an executable wire contract without the scenario types, strict decoder, result validator, and tracked generated presentations.
- **Fix:** Added the Phase 9 scenario child, updated strict request/result validation, preserved exhaustive older comparator matches, and regenerated the tracked scenario/trace schemas.
- **Files modified:** Protocol scenario, schema, earlier comparator exhaustive matches, and tracked schema presentations.
- **Verification:** Protocol tests, byte regression, schema closed-record checks, and all four full Rust gates pass.
- **Committed in:** `fbabf9d`

**2. [Rule 3 - Blocking] Adapted Phase 9 actions to exact checked public API ownership contracts**

- **Found during:** Task 2 RED
- **Issue:** Native particle creation requires explicit optional group ownership, and range force/impulse APIs require the owning particle system.
- **Fix:** Supplied `None` for deferred group topology, resolved and checked one shared system for stable-ID ranges, and used the one-argument destruction marker.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world/phase9.rs`
- **Verification:** Native action and source-timed checkpoint regression plus all four full Rust gates pass.
- **Committed in:** `87df7f4`, `75bb838`

**Total deviations:** 2 auto-fixed (2 blocking). **Impact:** Both changes were necessary to make the planned schema executable and to preserve checked ownership boundaries; no Phase 10 behavior was added.

## Issues Encountered

- The TDD RED test was intentionally not committed while failing because repository-level Rust instructions require every commit to follow a passing fmt, clippy, build, and test sequence. The RED failure and subsequent GREEN were still executed and reported.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 09-13 to assemble representative Phase 9 scenarios and differential evidence over this closed protocol.
- Canonical D1 promotion remains intentionally deferred; these local passes do not self-bless parity evidence.

## Self-Check: PASSED

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*

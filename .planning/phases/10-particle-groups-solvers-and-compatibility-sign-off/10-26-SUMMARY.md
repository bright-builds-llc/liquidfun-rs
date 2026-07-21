---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "26"
subsystem: particle-differential-cpp-oracle
tags: [cpp, liquidfun, particle-groups, differential-testing, jsonl, process-isolation]

requires:
  - phase: 10-24
    provides: Strict Phase 10 request/result contracts, semantic validation, and canonical wire schema
provides:
  - Pinned upstream execution for every Phase 10 particle-group operation
  - Pointer-independent semantic capture for groups, particles, topology, contacts, lifecycle events, and witnesses
  - Per-request foreign exception isolation with stderr-only diagnostics and valid-batch recovery
affects: [10-27, 10-28, phase10-comparison-policy, particle-differential-corpus, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Closed bounded C++ decode precedes every allocation and upstream operation
    - Protocol semantic IDs remain independent of upstream object addresses
    - One long-lived JSONL dispatcher isolates malformed requests without contaminating stdout

key-files:
  created:
    - tools/reference/src/rigid_world_phase10_decode.hpp
    - tools/reference/src/rigid_world_phase10_execute.hpp
    - tools/reference/src/rigid_world_phase10_operations.hpp
    - tools/reference/src/rigid_world_phase10_capture.hpp
    - crates/liquidfun-differential/tests/phase10_oracle.rs
  modified:
    - tools/reference/adapter-inputs.txt
    - tools/reference/src/main.cpp
    - tools/reference/src/rigid_world.cpp
    - tools/reference/src/rigid_world.hpp
    - tools/reference/src/rigid_world_decode.hpp
    - tools/reference/src/rigid_world_phase9_decode.hpp

key-decisions:
  - "Execute Phase 10 inside the existing rigid-world lifetime so inherited Phase 9 bodies, fixtures, systems, particles, and observations share one upstream world."
  - "Bind protocol group and particle IDs through explicit typed maps and use upstream pointers only as private lookup handles."
  - "Capture only portable semantic outcomes and typed witness roles; omit Rust-private solver pass identifiers and traces entirely."
  - "Reject one malformed request to stderr and continue the long-lived request loop so later valid batches remain independently usable."

patterns-established:
  - "Foreign oracle boundary: strict revision, schema, field, flag, count, iteration, float-bit, ownership, and lifecycle validation completes before execution."
  - "Canonical capture: upstream dense rows and pointers are translated back to declared semantic order before JSONL emission."

requirements-completed: [PART-09, PART-10, PART-11, PART-12, PART-13, PART-18, TEST-01, TEST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T03:41:00Z

duration: 1h 38m
completed: 2026-07-20
---

# Phase 10 Plan 26: Extend the Pinned C++ Oracle for Phase 10 Summary

**The isolated pinned LiquidFun process now executes the complete Phase 10 particle-group protocol and returns bounded pointer-independent semantic evidence while recovering cleanly after rejected requests.**

## Performance

- **Duration:** 1h 38m
- **Started:** 2026-07-21T02:03:00Z
- **Completed:** 2026-07-21T03:41:00Z
- **Tasks:** 1
- **Files modified:** 11

## Accomplishments

- Added strict closed decode for explicit, fill, stroke, append, join, split, flag, destruction, step, and inspect operations with reviewed count and iteration bounds, exact finite float-bit validation, pinned-revision provenance, and lifecycle-aware semantic identity checks.
- Executed every operation against the pinned upstream `b2World` and particle APIs while maintaining declared group and particle identities independently of pointers, dense indices, compaction, and split-created upstream objects.
- Captured canonical group membership, particle state, complete pairs and triads, particle and body contacts, lifecycle events, inherited body and joint observations, typed outcomes, and control/activation/interaction witnesses without Rust-private pass data.
- Kept stdout protocol-only, routed rejection diagnostics to stderr, and proved malformed and oversized records do not poison a reused process or its following valid request.
- Verified the published `liquidfun` package builds from its crate archive without discovering the private C++ oracle.

## Task Commits

Each task was committed atomically:

1. **Task 1: Decode and execute Phase 10 against pinned LiquidFun** - `4a842e6` (feat)

## Files Created/Modified

- `tools/reference/src/rigid_world_phase10_decode.hpp` - Strictly decodes and validates the bounded Phase 10 extension before execution.
- `tools/reference/src/rigid_world_phase10_execute.hpp` - Owns the upstream Phase 10 world, semantic identity maps, and operation/capture dispatch.
- `tools/reference/src/rigid_world_phase10_operations.hpp` - Implements creation, append, join, split, flags, destruction, stepping, and inspection against pinned upstream APIs.
- `tools/reference/src/rigid_world_phase10_capture.hpp` - Translates upstream groups, particles, topology, contacts, events, and witnesses into canonical semantic records.
- `crates/liquidfun-differential/tests/phase10_oracle.rs` - Covers every operation family, source ordering, exact replay, private-trace exclusion, bounded rejection, request-loop recovery, and Cargo isolation.
- `tools/reference/src/rigid_world.cpp` - Integrates Phase 10 execution with inherited rigid and Phase 9 observations in the existing dispatcher.
- `tools/reference/src/rigid_world.hpp` - Carries validated Phase 10 timelines through the rigid-world request.
- `tools/reference/src/rigid_world_decode.hpp` - Decodes Phase 10 before stripping its extension for inherited strict decoders.
- `tools/reference/src/rigid_world_phase9_decode.hpp` - Accepts the Phase 10 particle wrapper while retaining Phase 9 validation behavior.
- `tools/reference/src/main.cpp` - Contains per-request exceptions so rejected records cannot terminate a reusable oracle.
- `tools/reference/adapter-inputs.txt` - Includes every new module in the configured adapter digest.

## Decisions Made

- Phase 10 owns execution of an extended timeline and emits inherited Phase 9 lifecycle observations at their original positions, avoiding two executors mutating parallel upstream worlds.
- Stable particle tokens stored in upstream user-data lanes survive row compaction and let semantic IDs be rebound after split and zombie removal without serializing pointer values.
- Group depth remains `null` in C++ captures because the pinned public upstream API exposes no depth view; all other required group aggregates and membership remain captured.
- Group-join invalidation is represented by the join lifecycle event rather than a fabricated standalone destroy event, matching upstream semantics.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split the Phase 10 executor into reviewable operation and capture modules**

- **Found during:** Task 1 implementation
- **Issue:** Keeping decode, execution, upstream mutation, and semantic capture in only the two named plan artifacts would make the executor nearly 1,000 lines and violate repository file-size guidance.
- **Fix:** Kept `rigid_world_phase10_execute.hpp` as the cohesive private facade and extracted operation and capture fragments into `rigid_world_phase10_operations.hpp` and `rigid_world_phase10_capture.hpp`; every module remains below 500 lines and participates in the adapter digest.
- **Files modified:** `tools/reference/src/rigid_world_phase10_execute.hpp`, `tools/reference/src/rigid_world_phase10_operations.hpp`, `tools/reference/src/rigid_world_phase10_capture.hpp`, `tools/reference/adapter-inputs.txt`
- **Verification:** The documented CMake/Ninja build, all five focused oracle tests, import/link audit, and full exact Rust gate passed.
- **Committed in:** `4a842e6`

**Total deviations:** 1 auto-fixed (1 blocking structural seam).
**Impact on plan:** The additional private modules preserve the planned boundary and behavior while keeping each implementation unit reviewable. No production dependency or protocol scope was added.

## Issues Encountered

- The configured local oracle tools are CMake 3.27.9 and Apple Clang 21.0.0 rather than the canonical CMake 4.3.3 and Clang 22.1.8. The documented D2 build emitted explicit warnings, remained bound to upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`, and passed; canonical D1 evidence remains assigned to Plan 10-31.
- macOS provenance scanning delayed first launches of newly linked test executables. The mandatory gate was left running to completion and every suite passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-27 can compare native and upstream results through one strict shared semantic schema without adapting pointer identities or private pass traces.
- Harness failures remain distinct from valid physics results, and reused oracle batches recover after bounded parser rejection.
- No blockers remain.

## Self-Check: PASSED

- Confirmed implementation commit `4a842e6` exists and contains only the 11 scoped oracle and test files.
- Confirmed the documented CMake/Ninja/xtask path rebuilds `liquidfun-reference` at the pinned upstream revision.
- Confirmed `cargo test -p liquidfun-differential --all-features --test phase10_oracle` passes all five focused tests.
- Confirmed `cargo package -p liquidfun --allow-dirty` packages and verifies the published crate without C++ discovery.
- Confirmed the implementation commit was preceded by the exact mandatory Rust gate: format, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests including 409 unit tests and 19 doctests.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*

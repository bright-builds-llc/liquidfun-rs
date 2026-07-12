---
phase: 06-minimal-rigid-world-vertical-slice
plan: "10"
subsystem: cpp-rigid-world-oracle
tags: [cpp, box2d, jsonl, rigid-world, semantic-trace, differential-oracle]
requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Duplicate-aware bounded JSONL framing, exact float-bit transport, process isolation, and reset proof
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "06"
    provides: Closed rigid-world declarations, actions, checkpoints, semantic results, and both required witness families
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "07"
    provides: Closed Phase 6 result schema and exact-first observable policy
provides:
  - Strict typed C++ decoding for the complete bounded rigid-world timeline before any b2World effect
  - One fresh pinned b2World per request executing both required lifecycle witness families
  - Ordered semantic body, fixture, contact, manifold, impulse, event, and destruction traces with exact float bits
  - Protocol-loop dispatch and monotonic terminal reset proof for reusable rigid-world oracle sessions
affects: [06-09-rigid-supervision, 06-11-rigid-build-identity, 06-12-rigid-evidence]
tech-stack:
  added: []
  patterns: [strict SAX before typed domain decode, semantic pointer-to-ID side maps, declaration-order snapshots, listener-backed lifecycle evidence]
key-files:
  created:
    - tools/reference/src/rigid_world.cpp
    - tools/reference/src/rigid_world.hpp
    - tools/reference/src/rigid_world_decode.hpp
    - tools/reference/src/rigid_world_trace.hpp
  modified:
    - tools/reference/src/protocol.cpp
    - tools/reference/src/protocol.hpp
    - tools/reference/src/main.cpp
    - tools/reference/tests/protocol_tests.cpp
    - tools/reference/CMakeLists.txt
key-decisions:
  - "Run the existing duplicate-aware bounded SAX pass before constructing the rigid typed domain, so duplicate, depth, string, collection, framing, and record limits fail before world creation."
  - "Use one b2World for both complete timelines in each request, requiring the first timeline to fully destroy its state before the second and destroying the world before terminal reset proof."
  - "Orient harness-private contact identity by fixture declaration order while preserving pinned contact-manager, manifold-point, callback, and destruction collection order."
  - "Process touched broad-phase proxies before each action-level step through the pinned contact manager, avoiding a second simulation-time advance during filter and activation reconsideration."
patterns-established:
  - "Rigid oracle boundary: strict bytes become closed variants and invariant-bearing values before b2World is constructed."
  - "Semantic trace boundary: upstream pointers remain private map keys and only declaration IDs, source order, active payloads, and exact bits cross JSONL."
requirements-completed: [RIGD-01, RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T05:28:22Z
duration: 47 min
completed: 2026-07-12
---

# Phase 6 Plan 10: Pinned C++ Rigid-World Adapter Summary

**Pinned LiquidFun execution for both closed rigid-world lifecycle families with strict pre-effect decoding, exact semantic traces, ordered teardown evidence, and reusable reset epochs**

## Performance

- **Duration:** 47 min
- **Started:** 2026-07-12T04:41:00Z
- **Completed:** 2026-07-12T05:28:22Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a closed typed C++ request domain whose strict duplicate-aware, bounded decoding and lifecycle validation complete before a `b2World` exists.
- Executed the non-colliding and single-contact timelines through one fresh pinned world per request, including material, filter, mass, sensor, activation, contact persistence, warm-start, and teardown behavior.
- Emitted declaration-ordered body/fixture state, manager-ordered contacts, active manifold features and impulses, callback events, destruction snapshots, and exact `f32` bits without serializing pointers or layout.
- Proved real subprocess reuse emits identical rigid results followed by reset epochs 1 then 2, with protocol-only stdout.

## Task Commits

Each task was committed atomically:

1. **Task 1: Strictly decode and execute rigid timelines in pinned C++** - `1d5616d` (feat)

## Files Created/Modified

- `tools/reference/src/rigid_world.hpp` - Closed invariant-bearing request variants, declarations, checkpoints, and adapter result surface.
- `tools/reference/src/rigid_world_decode.hpp` - Strict field, version, bound, identifier, geometry, lifecycle, witness, and ordering validation.
- `tools/reference/src/rigid_world.cpp` - Pinned world execution, semantic ID maps, contact listener, snapshots, teardown, and reset lifecycle.
- `tools/reference/src/rigid_world_trace.hpp` - Exact-bit semantic JSON encoding and terminal reset record.
- `tools/reference/src/protocol.hpp` - Rigid request kind and shared bounded-record validation surface.
- `tools/reference/src/protocol.cpp` - Existing process-loop request dispatch classification and duplicate-aware validation exposure.
- `tools/reference/src/main.cpp` - Rigid adapter dispatch in the existing long-lived oracle process.
- `tools/reference/tests/protocol_tests.cpp` - Complete witness, strict rejection, ordering, redaction, and reuse-reset regression coverage.
- `tools/reference/CMakeLists.txt` - Minimal source-list binding required to compile and test Plan 06-10; Plan 06-11 owns complete provenance identity coverage.

## Decisions Made

- Reused the established bounded SAX parser as the outer trust boundary, then decoded the already-validated record into closed C++ variants rather than duplicating world-facing validation throughout execution.
- Kept upstream pointers entirely inside request-scoped maps; contact occurrence identity is scenario semantic identity plus a checked ordinal, never an address or packed key.
- Captured bodies and fixtures in declaration order and contacts/events/destructions in source manager/report order without global sorting.
- Kept terminal reset separate from the semantic result record so the supervisor can fail closed on missing, false, repeated, or out-of-order reset evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Bound the new translation unit into the debug build before Plan 06-11 provenance work**

- **Found during:** Task 1 (TDD RED setup)
- **Issue:** Plan 06-10 requires compiling and executing `rigid_world.cpp`, but the explicit CMake source list did not include it and Plan 06-11 is scheduled afterward.
- **Fix:** Added only `rigid_world.cpp` to the existing protocol library. Plan 06-11 still owns adapter-input manifests, complete source/header hashing, and compile-database identity enforcement.
- **Files modified:** `tools/reference/CMakeLists.txt`
- **Verification:** The debug library, protocol tests, and real oracle executable all compile under the existing warning and floating-point flags.
- **Committed in:** `1d5616d`

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** Necessary build plumbing only; no provenance claim was pulled forward from Plan 06-11.

## Issues Encountered

- The plan names a `liquidfun-reference-tests` target and an `oracle-debug` CTest preset that do not exist. Verification used the repository's actual `liquidfun-reference-protocol-tests` target and `ctest --test-dir target/reference/oracle-debug --output-on-failure` after the reviewed xtask configure command.
- Local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8. The configured local run is D2 evidence only, consistent with project policy.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-11 can add `rigid_world.cpp`, `rigid_world.hpp`, `rigid_world_decode.hpp`, and `rigid_world_trace.hpp` to adapter content identity and require the rigid translation unit in compile-command provenance.
- The rigid supervisor can consume one `rigid_world_result` followed by one `rigid_world_end` with a verified monotonic reset epoch.
- No implementation blocker remains; canonical D1 execution remains intentionally deferred to the pinned Linux/Clang lane.

## Self-Check: PASSED

- All created files exist and task commit `1d5616d` is present.
- Rust format, strict Clippy, all-target build, all-feature tests, C++ debug configure/build, CTest, subprocess reuse, forbidden-token scans, and `git diff --check` passed.

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*

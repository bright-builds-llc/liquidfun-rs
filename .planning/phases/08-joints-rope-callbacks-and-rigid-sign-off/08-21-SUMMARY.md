---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "21"
subsystem: rigid-pinned-oracle-evidence
tags: [cpp, liquidfun, joints, callbacks, destruction, differential-evidence]
requires:
  - phase: 08-20
    provides: public-API native execution for the strengthened Phase 8 corpus
provides:
  - pinned C++ execution for every strengthened Phase 8 witness
  - source-timed callback and destruction lifecycle observations
  - bounded fail-closed Phase 8 decode, execution, contact, and reset checks
affects: [08-22, rigid-comparator, rigid-sign-off]
tech-stack:
  added: []
  patterns: [typed-family contact isolation, source-callback lifecycle capture, actual-count checkpoint validation]
key-files:
  created: []
  modified:
    - tools/reference/src/rigid_world_phase8_decode.hpp
    - tools/reference/src/rigid_world_phase8_execute.hpp
    - tools/reference/tests/protocol_tests.cpp
    - crates/liquidfun-differential/tests/rigid_world.rs
    - crates/liquidfun-differential/tests/round_trip.rs
key-decisions:
  - "Phase 8 C++ contact evidence is emitted only from live pinned contacts and listener invocations; solver-only timelines reject incidental fixture pairs."
  - "Destroying a collision-suppressing joint explicitly touches both bodies' fixture proxies so the strengthened mixed-joint witness observes restored collision deterministically."
  - "C++ checkpoint counts are derived from actual pinned state and checked against the declaration before any result is emitted."
requirements-completed: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T05:55:31Z
duration: 20min
completed: 2026-07-14
---

# Phase 8 Plan 21: Pinned C++ Step-Bearing Adapter and Protocol Summary

**The pinned C++ adapter now executes all nineteen accumulated rigid families and emits bounded live joint, rope, contact, callback, destruction, reconstruction, and diagnostic evidence.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-14T05:35:00Z
- **Completed:** 2026-07-14T05:55:31Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Executed real positive pinned-world steps for all eleven joint kinds and all RR/RP/PR/PP four-body gear topologies, preserving defined pre-step reactions and exact post-step getters.
- Mirrored native typed-family contact isolation and emitted the exact nine-event callback and thirteen-event destruction lifecycle arrays from live filter/listener/destruction occurrences.
- Serialized actual contact manifolds and actual checkpoint counts, rejected zero-step, missing post-step observations, duplicate IDs, and N+1 action/joint/rope/rope-vertex inputs, and proved repeated complete reset.
- Opened the strict Rust process boundary: oracle framing, result decode, declaration validation, and reset proof now pass and reach the Phase 8 physics comparator.

## Task Commits

1. **Task 08-21-01: Execute and observe every strengthened witness in the pinned oracle** - `053651a` (feat)

## Files Created/Modified

- `tools/reference/src/rigid_world_phase8_decode.hpp` - Strict Phase 8 action shapes, positive-step rules, post-step observation requirements, and collection bounds.
- `tools/reference/src/rigid_world_phase8_execute.hpp` - Live pinned stepping, all joint/gear/rope observations, callback directives, lifecycle capture, contact snapshots, cascades, and reset-safe teardown.
- `tools/reference/tests/protocol_tests.cpp` - All-kind, gear, callback, destruction, malformed-input, boundary, reaction, and repeated-reset regressions.
- `crates/liquidfun-differential/tests/rigid_world.rs` - Replaces the temporary pre-08-21 rejection contract with validated process success.
- `crates/liquidfun-differential/tests/round_trip.rs` - Requires the C++ protocol self-test to pass after adapter implementation.

## Decisions Made

- Used `b2ContactFilter`, `b2ContactListener`, and `b2DestructionListener` directly on each isolated timeline world so evidence is appended where pinned effects occur.
- Kept semantic IDs in adapter-private maps, ordered contact identities by fixture declaration, and emitted no pointers, addresses, or storage coordinates.
- Preserved safe Rust's dependent-gear-first cascade as an explicit adapter semantic strengthening while all resulting destruction operations execute against real pinned objects.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced Plan 08-20's temporary C++ rejection expectations**

- **Found during:** Focused Rust round-trip and rigid-world verification
- **Issue:** Three tests intentionally required the adapter to fail only until Plan 08-21, so they failed once the implementation became available.
- **Fix:** Updated only those boundary tests to require validated nineteen-family process and CTest success.
- **Files modified:** `crates/liquidfun-differential/tests/rigid_world.rs`, `crates/liquidfun-differential/tests/round_trip.rs`
- **Verification:** Focused suites passed 45/45 and 13/13.
- **Committed in:** `053651a`

**2. [Rule 1 - Bug] Serialized actual Phase 8 contact state instead of declaration-only counts**

- **Found during:** Direct rigid-world comparison
- **Issue:** The initial implementation copied declared contact counts but emitted an empty contact array, causing strict result validation to report a malformed record.
- **Fix:** Captured pinned contact identities, material, enabled/touching state, and manifolds, then checked derived counts before emission.
- **Files modified:** `tools/reference/src/rigid_world_phase8_execute.hpp`
- **Verification:** Direct comparison passes framing, result decode, declaration validation, and reset proof before reaching the numeric comparator.
- **Committed in:** `053651a`

**3. [Rule 2 - Missing Critical] Touched proxies after collision-suppressing joint destruction**

- **Found during:** Actual checkpoint-count validation
- **Issue:** Upstream only flags existing contacts when a suppressing joint is removed; the strengthened witness had no preexisting contact, so collision restoration required deterministic proxy reconsideration.
- **Fix:** Reapplied each affected fixture's current filter after the suppressing joint was destroyed.
- **Files modified:** `tools/reference/src/rigid_world_phase8_execute.hpp`
- **Verification:** The mixed-joint checkpoint contains one live one-point contact and protocol CTest passes.
- **Committed in:** `053651a`

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 missing critical). **Impact:** All fixes were necessary to complete the planned adapter boundary without changing comparator, policy, or xtask ownership.

## Issues Encountered

- The first direct comparison now reaches a genuine Phase 8 physics mismatch at `joint-def-revolute` coordinate (`0x00000000` native versus `0x33a2c494` pinned C++). Plan 08-22 owns strict comparison/remediation; this is no longer an adapter framing or declaration failure.
- Local evidence uses CMake 3.27.9 and Apple Clang 21.0.0 rather than canonical CMake 4.3.3 and Clang 22.1.8, so it remains noncanonical D2 evidence.

## Verification

- Oracle debug configure and build: passed.
- `liquidfun-reference-protocol` CTest: 1/1 passed.
- Focused Rust rigid-world process suite: 45/45 passed.
- Focused Rust round-trip suite: 13/13 passed.
- Ordered Rust gate with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-target`: format, clippy with warning denial, all-target build, and all-feature tests passed (185 library tests, all integration targets, 13 doctests).
- Direct rigid-world compare passes strict process/result validation and reports the first numeric physics mismatch for Plan 08-22.
- `git diff --check`: passed.

## User Setup Required

None.

## Next Phase Readiness

- Plan 08-22 can compare independently produced native and pinned C++ Phase 8 records through the strict typed boundary.
- The first numeric mismatch is recorded above and must be resolved or dispositioned without widening policy silently.

## Self-Check: PASSED

- All five implementation/test files and this summary exist.
- Commit `053651a` records Task 08-21-01.
- Lifecycle ID `8-2026-07-13T21-26-30` matches the plan.
- No comparator production code, tolerance policy, schema authority, fixture, or xtask command changed.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

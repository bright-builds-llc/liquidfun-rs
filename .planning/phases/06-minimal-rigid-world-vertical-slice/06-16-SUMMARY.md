---
phase: 06-minimal-rigid-world-vertical-slice
plan: "16"
subsystem: rigid-protocol-contract
tags: [rust, cpp, protocol, schema, validation, differential]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "15"
    provides: Correct non-dynamic contact admission and complete declaration-first witnesses
provides:
  - Exact fixed Phase 6 step tuple at schema, Rust, native, and C++ boundaries
  - Shared 128-action Rust/schema/C++ contract with maximum and maximum-plus-one evidence
  - Source-ordered centered custom-inertia rejection before either engine executes
  - Checked-in rejected negative-centered-inertia protocol fixture
affects: [06-17-rigid-staging, 06-18-sanitizer-signoff, phase-07-rigid-solver]
tech-stack:
  added: []
  patterns: [named cross-language contract constants, exact fixed-tuple admission, source-ordered boundary validation]
key-files:
  created:
    - protocol/fixtures/rejected/rigid-world-negative-centered-inertia.jsonl
    - .planning/phases/06-minimal-rigid-world-vertical-slice/06-16-SUMMARY.md
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - crates/liquidfun-test-protocol/src/schema/tests.rs
    - crates/liquidfun-test-protocol/tests/fixtures.rs
    - protocol/schemas/scenario-v1.schema.json
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/tests/rigid_world.rs
    - tools/reference/src/rigid_world.hpp
    - tools/reference/src/rigid_world_decode.hpp
    - tools/reference/tests/protocol_tests.cpp
key-decisions:
  - "Keep Phase 6 solver configuration closed by admitting only timestep bits 0x3c888889, eight velocity iterations, and three position iterations at every boundary."
  - "Use one exact action maximum of 128 in Rust bounded decoding, generated schema presentation, and C++ decoding."
  - "Reject custom mass data by reproducing the engine's source-ordered center dot, parallel-axis product, and centered-inertia subtraction before effects."
requirements-completed: [RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T16:59:00Z
duration: 15 min
completed: 2026-07-12
---

# Phase 6 Plan 16: Rigid Protocol Contract Closure Summary

**The rigid protocol now admits one exact Phase 6 solver tuple, one 128-action bound, and only custom mass data whose source-ordered centered inertia is finite and non-negative across Rust and C++.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-12T16:44:00Z
- **Completed:** 2026-07-12T16:59:00Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Replaced permissive step ranges with named exact constants for `0x3c888889`, 8, and 3 in typed Rust validation, generated scenario schema, native defense, and C++ decode.
- Unified the action maximum at 128 and proved exact-maximum acceptance plus 129 rejection on both protocol implementations.
- Added source-ordered centered-inertia validation for negative and non-finite dot, product, and subtraction results before either world can observe the request.
- Added `rigid-world-negative-centered-inertia.jsonl` and an ordinary fixture-suite assertion for the exact `InvalidGeometry` classification.
- Preserved generic empty-world schema behavior and the Phase 7 boundary: no public timestep or iteration configuration was introduced.

## Verifier Gap Closure

| Gap ID | Closure evidence |
| --- | --- |
| `ignored-step-parameters` | Exact tuple constants, three alternate-lane Rust tests, three matching C++ cases, and explicit native-field destructuring before `World::step`. |
| `rigid-action-bound-mismatch` | Rust accepts exactly 128 and rejects 129; C++ protocol CTest exercises the same valid lifecycle request at 128 and rejects 129 before execution. |
| `invalid-centered-inertia-boundary` | Rust unit and checked-in fixture tests classify `(1, (2,0), 1)` as `InvalidGeometry`; C++ rejects the same source-ordered negative result before adapter construction. |

## Task Commits

1. **Task 1: Define one exact Rust/schema contract** - `7cd734d` (`fix`)
2. **Task 2: Mirror the exact contract at the C++ boundary** - `b67de4d` (`fix`)

## Decisions Made

- Fixed Phase 6 input admission instead of exposing early configurable solver semantics reserved for Phase 7.
- Compared step bits directly rather than decoding a float, keeping transport authority exact and avoiding an irrelevant numeric conversion.
- Checked every centered-inertia intermediate explicitly so overflow cannot be reclassified as an engine or physics failure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Reconciled admission-witness schema drift from Plan 06-15**

- **Found during:** Task 1 schema byte-stability verification
- **Issue:** Typed witness authority already included `static_kinematic_overlap_rejected` and `kinematic_kinematic_overlap_rejected`, but the checked-in scenario presentation omitted both, so the required read-only presentation check failed before this plan's schema changes could pass.
- **Fix:** Added the two typed-authority witness values to the tracked generated scenario schema while applying this plan's exact step constants.
- **Files modified:** `protocol/schemas/scenario-v1.schema.json`
- **Verification:** `schema_presentations_are_byte_stable_and_newline_terminated` passes and the generated presentation is byte-identical.
- **Committed in:** `7cd734d`

**Total deviations:** 1 auto-fixed blocking generated-presentation drift. **Impact:** Restored an already-intended Plan 06-15 schema link without expanding Phase 6 behavior or evidence authority.

## Issues Encountered

- Existing debug/release CMake directories correctly rejected stale adapter digests after C++ source changes. Reconfiguration through `cargo xtask upstream configure` refreshed reviewed identity before rebuilding; no stale binary was accepted.
- Local CMake 3.27.9 and Apple Clang 21 remain noncanonical D2 tools by policy; both real compares correctly reported `d2_supported` and no D1/platform evidence was promoted.

## Validation Evidence

- All 18 focused rigid protocol tests pass, including exact action maximum, alternate tuple lanes, negative centered inertia, and non-finite intermediates.
- The ordinary rejected-fixture suite returns exactly `Some(RigidWorldErrorKind::InvalidGeometry)` for the new checked-in fixture.
- The native differential contract test executes both families with the fixed tuple; no `RigidWorldAction::Step { .. }` wildcard remains.
- Debug CTest passes after rebuilding `liquidfun-reference-protocol-tests`, covering 128/129, all three step mutations, and centered-inertia rejection.
- Real debug and release rigid-world comparisons each match both required families under `phase6-v1` with D2 classification.
- Schema presentation byte-stability and `git diff --check` pass.
- Before each task commit, the mandatory sequence passed in order: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-17 can integrate rigid staging and D1 promotion on top of one fail-closed request contract.
- Plan 06-18 still owns sanitizer execution and final Phase 6 signoff; Phase 6 is not complete here.

## Self-Check: PASSED

- Task commits `7cd734d` and `b67de4d` exist in history.
- The rejected fixture, named Rust/C++ constants, exact schema constants, and boundary tests exist on disk.
- Debug/release real comparisons and CTest passed after reviewed native rebuilds.
- No public solver configuration or Phase 7 behavior was introduced.

***

_Phase: 06-minimal-rigid-world-vertical-slice_
_Completed: 2026-07-12_

---
phase: 06-minimal-rigid-world-vertical-slice
plan: "20"
subsystem: rigid-world-custom-mass-validation
tags: [rust, cpp, protocol, mass-data, differential, validation]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "16"
    provides: Source-ordered centered-inertia validation at Rust and C++ boundaries
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "19"
    provides: Candidate-first atomic implicit mass transitions
provides:
  - Branch-faithful custom mass validation preserving zero-origin no-inertia semantics
  - Strict positive centered inertia for every positive-origin custom mass
  - Checked-in equality-boundary malformed JSONL fixture
  - Matched Rust domain, protocol, native, and C++ rejection evidence
affects: [06-22-completion-matrix, phase-6-reverification, phase-7-rigid-solver]
tech-stack:
  added: []
  patterns: [source-ordered parallel-axis validation, typed decode before effects]
key-files:
  created:
    - protocol/fixtures/rejected/rigid-world-zero-centered-inertia.jsonl
    - .planning/phases/06-minimal-rigid-world-vertical-slice/06-20-SUMMARY.md
  modified:
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-differential/src/rigid_world.rs
    - tools/reference/src/rigid_world_decode.hpp
key-decisions:
  - "Treat exact origin inertia zero as the pinned no-inertia branch without evaluating the parallel-axis subtraction."
  - "For positive origin inertia, require every source-ordered intermediate to be finite and centered inertia to be strictly positive before effects."
requirements-completed: [RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T22:08:00Z
duration: 24 min
completed: 2026-07-12
---

# Phase 6 Plan 20: Zero Centered-Inertia Boundary Summary

**Custom mass data now preserves the pinned zero-origin no-inertia branch while rejecting exact-zero centered inertia consistently before either Rust or C++ can mutate a world.**

## Performance

- **Duration:** 24 min
- **Started:** 2026-07-12T21:44:00Z
- **Completed:** 2026-07-12T22:08:00Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Renamed the public error to `BodyMassDataError::NonPositiveCenteredRotationalInertia` and made its documentation and diagnostic match the strict-positive invariant.
- Preserved exact origin inertia zero as a valid no-inertia branch even with a nonzero center, including native execution evidence.
- Reproduced the parallel-axis calculation in explicit source order with finite checks at the public Rust, Rust protocol, native executor, and C++ decode boundaries.
- Added `rigid-world-zero-centered-inertia.jsonl` for `(mass=1, center=(1,0), origin inertia=1)` and proved stable `RigidWorldErrorKind::InvalidGeometry` classification.
- Proved the native typed defense rejects equality without changing world state even when JSON decoding is bypassed.
- Closed verifier gap `zero-centered-inertia-boundary` without adding solver configuration or other Phase 7 behavior.

## TDD Evidence

- **RED intent:** Equality cases were absent and the old `< 0` checks admitted zero centered inertia into the pinned assertion/divide-by-zero path.
- **GREEN:** Public, protocol, fixture, native-defense, and C++ tests pass with `<= 0` rejection limited to the positive-origin branch.
- Separate RED commits were not created because repository instructions require formatting, strict Clippy, all-target build, and the full all-feature test suite to pass before every commit.

## Task Commits

1. **Task 1: Enforce the branch-faithful Rust domain and protocol invariant** - `f4dae15` (`fix`)
2. **Task 2: Reject equality at the C++ boundary and prove real workflows** - `f023bc8` (`fix`)
3. **Focused native zero-origin integration evidence** - `282e7c9` (`test`)

## Files Created/Modified

- `crates/liquidfun/src/world/body.rs` - Strict-positive positive-origin branch and truthful public typed error.
- `crates/liquidfun/tests/rigid_definitions.rs` - Public equality rejection and zero-origin acceptance controls.
- `crates/liquidfun/tests/fixture_dynamics.rs` - Updated effective-unit-mass error contract.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Matching source-ordered branch validation.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs` - Equality rejection and zero-origin acceptance.
- `crates/liquidfun-test-protocol/src/schema/tests.rs` - Explicit test that relational f32 validity remains typed-decoder authority.
- `crates/liquidfun-test-protocol/tests/fixtures.rs` - Checked-in malformed fixture classification.
- `protocol/fixtures/rejected/rigid-world-zero-centered-inertia.jsonl` - Equality-boundary request fixture.
- `crates/liquidfun-differential/src/rigid_world.rs` - Shared native custom-mass reconstruction and no-mutation defense test.
- `crates/liquidfun-differential/tests/rigid_world.rs` - Native zero-origin execution control.
- `tools/reference/src/rigid_world_decode.hpp` - C++ strict-positive boundary before action construction.
- `tools/reference/tests/protocol_tests.cpp` - Equality, zero-origin, and non-finite intermediate controls.

## Decisions Made

- Used the authored origin inertia as the branch condition, matching `b2Body::SetMassData`: exact zero never performs the parallel-axis subtraction.
- Kept schemas structural and byte-identical because the relational IEEE-754 bit invariant cannot be expressed truthfully in the current JSON Schema.
- Kept native `BodyMassData` reconstruction in the executor so typed requests constructed outside JSON decoding still fail before world mutation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the required schema-authority assertion outside the frontmatter file list**

- **Found during:** Task 1 schema byte-stability acceptance work
- **Issue:** The task action explicitly required an assertion/comment at the schema-facing test seam, but `crates/liquidfun-test-protocol/src/schema/tests.rs` was omitted from `files_modified` and the task `<files>` list.
- **Fix:** Added one read-only assertion documenting that cross-field centered-inertia arithmetic remains typed Rust/C++ authority; no schema presentation changed.
- **Files modified:** `crates/liquidfun-test-protocol/src/schema/tests.rs`
- **Verification:** Schema presentations remain byte-stable and newline-terminated.
- **Committed in:** `f4dae15`

**Total deviations:** 1 auto-fixed blocking plan-file omission. **Impact:** Satisfied the plan's explicit schema-trust-boundary requirement without changing schema bytes or widening input acceptance.

## Issues Encountered

- The first strict Clippy pass rejected similar x/y squared binding names. Replacing them with one fixed two-element source-order array resolved the lint without changing arithmetic order.
- Local CMake 3.27.9 and Apple Clang 21 differ from the canonical D1 pins, so real oracle evidence is correctly reported as D2 rather than canonical authority.

## Validation Evidence

- Public centered-inertia tests pass, including equality rejection and zero-origin acceptance.
- All 20 rigid protocol tests pass, including equality, negative, non-finite, fixed-step, and action-bound controls.
- The checked-in equality fixture returns exactly `Some(RigidWorldErrorKind::InvalidGeometry)`.
- Native unit evidence rejects equality before world mutation; native integration evidence executes the zero-origin branch.
- Fresh debug and release C++ protocol CTest each pass 1/1 after rebuilding the adapter.
- Real debug and release rigid comparisons each match both required families under `phase6-v1` at D2.
- Debug rigid replay matches both required families at D2.
- Schema presentation byte-stability, warning-denied workspace rustdoc, and `git diff --check` pass.
- Before every commit, the mandatory sequence passed in order: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-22 can reconcile the completion matrix and perform final Phase 6 re-verification.
- RIGD-02 and RIGD-04 remain pending formal Phase 6 verification; this executor does not mark Phase 6 complete.

## Self-Check: PASSED

- Task commits `f4dae15`, `f023bc8`, and `282e7c9` exist in history.
- The equality fixture and every named Rust/C++ boundary artifact exist on disk.
- The work stays within the custom-mass validation boundary and adds no Phase 7 solver behavior.

***

_Phase: 06-minimal-rigid-world-vertical-slice_
_Completed: 2026-07-12_

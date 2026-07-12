---
phase: 06-minimal-rigid-world-vertical-slice
plan: "01"
subsystem: rigid-world-contracts
tags: [rust, body, fixture, shape-ownership, validation]
requires:
  - phase: 04-math-settings-and-numerical-policy
    provides: Source-ordered f32 math values and finite-domain boundaries
  - phase: 05-shapes-and-collision-foundation
    provides: Immutable validated Shape values and FilterData
provides:
  - Closed BodyType and reusable checked BodyDef contracts
  - Checked custom BodyMassData with centered-inertia proof
  - Immutable owned FixtureDef and semantic fixture snapshots
  - Typed pre-world validation errors and curated root exports
affects: [06-02-world-storage, 06-03-fixture-dynamics, rigid-world-api]
tech-stack:
  added: []
  patterns: [parse-at-boundary domain values, private-field semantic snapshots, owned immutable shapes]
key-files:
  created:
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/fixture.rs
    - crates/liquidfun/tests/rigid_definitions.rs
  modified:
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Retain checked body position and angle as their accepted f32 bits, deriving Transform only on semantic access."
  - "Own Shape directly in FixtureDef and clone it only when producing another owned definition or snapshot."
patterns-established:
  - "Definition boundary: reject invalid primitives once, then expose only checked private-field values."
  - "Semantic snapshot: consumers inspect owned state without receiving mutable topology or world storage authority."
requirements-completed: [RIGD-01, RIGD-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T03:21:20Z
duration: 13 min
completed: 2026-07-12
---

# Phase 6 Plan 01: Checked Body and Fixture Contracts Summary

**Checked reusable body and fixture definitions now preserve accepted physics bits, reject invalid transforms, materials, and centered inertia, and expose only immutable semantic state.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-12T03:08:00Z
- **Completed:** 2026-07-12T03:21:20Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added a closed three-variant body type, checked reusable body definitions, semantic body snapshots, and invariant-bearing custom mass data.
- Added fixture definitions that own immutable shape topology by value together with exact density, friction, restitution, sensor, and filter state.
- Added 13 black-box tests plus two compile-fail rustdoc contracts covering finite boundaries, exact retained bits, centered inertia, deep shape ownership, and storage privacy.

## TDD Evidence

- **Task 1 RED:** `cargo test -p liquidfun --test rigid_definitions body --all-features` failed because the body contracts did not exist.
- **Task 1 GREEN:** The same filter passed 9 tests after implementing and curating the checked body contracts.
- **Task 2 RED:** `cargo test -p liquidfun --test rigid_definitions fixture --all-features` failed because the fixture contracts did not exist.
- **Task 2 GREEN:** The same filter passed 4 tests after implementing owned fixture definitions and snapshots.
- No separate RED commits were created because repository instructions require the full Rust verification sequence to pass before every commit.

## Task Commits

Each task was committed atomically after its complete green verification gate:

1. **Task 1: Define checked body state contracts** - `8a05836` (feat)
1. **Task 2: Define immutable fixture and material contracts** - `7ebc448` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/body.rs` - Closed body type, checked definitions, snapshots, mass data, and typed errors.
- `crates/liquidfun/src/world/fixture.rs` - Owned checked fixture definitions, snapshots, validation errors, and compile-fail privacy contracts.
- `crates/liquidfun/tests/rigid_definitions.rs` - Focused black-box definition and ownership tests.
- `crates/liquidfun/src/world.rs` - Explicit body and fixture contract curation.
- `crates/liquidfun/src/lib.rs` - Explicit crate-root consumer exports.

## Decisions Made

- Body definitions retain the original checked position and angle fields so signed zero and every accepted `f32` bit remain observable without a trigonometric round trip.
- Custom mass construction stores both origin inertia and the source-ordered centered result, proving the later world mutation path cannot receive negative or non-finite centered inertia.
- Fixture definitions accept `Shape` by value and expose only shared immutable shape access; snapshots clone the owned topology and never expose world implementation storage.

## Deviations from Plan

### Process Adjustment

- The required RED failures were observed before implementation, but were not committed separately because the task-specific instruction required `cargo fmt`, Clippy, build, and the full test suite to pass before every commit. Each TDD task therefore landed as one atomic green outcome commit.

**Total implementation deviations:** 0.
**Impact on plan:** Product scope, behavior, tests, and acceptance criteria were unchanged.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Verification

- Both focused `rigid_definitions` filters pass: 9 body tests and 4 fixture tests.
- Warning-denied rustdoc passes all 12 doctests, including both new compile-fail privacy contracts.
- Strict all-target, all-feature Clippy passes.
- The ordered repository gate passes: format, Clippy, all-target build, and all-feature tests.
- Acceptance scans confirm all required contracts and no deferred controls, mutable topology, or raw proxy identities in the curated surface.
- `git diff --check` passes.

## Next Phase Readiness

- Plan 06-02 can deepen world storage using checked `BodyDef` and `FixtureDef` inputs and owned snapshots.
- No blockers or unresolved stubs remain.

## Self-Check: PASSED

- Created contract and test files exist.
- Task commits `8a05836` and `7ebc448` exist in repository history.
- No unplanned threat surface was introduced beyond the plan's consumer-to-domain validation boundary.

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*

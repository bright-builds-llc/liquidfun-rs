---
phase: 11-examples-headless-tooling-and-testbed
plan: "04"
subsystem: test-protocol-catalog
tags: [rust, catalog, rigid-world, joints, rope, deterministic-replay]
requires:
  - phase: 11-03
    provides: bounded catalog model and canonical resolver
provides:
  - ten reviewed native rigid-world scenario definitions
  - one reviewed definition for each of the eleven rigid joint kinds
  - standalone-rope scenario definition with request-materialized stepping
  - typed coverage metadata and fail-closed exact-action replay validation
affects: [11-05, 11-06, 11-07, 11-09, 11-10, 11-18]
tech-stack:
  added: []
  patterns:
    - closed RigidWorldAction schedules with deterministic semantic IDs
    - typed test/evidence mappings and explicit consumer eligibility
    - request-specific materialization of exact world and rope steps
key-files:
  created:
    - crates/liquidfun-test-protocol/src/catalog/model/metadata.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios/rigid.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios/joints.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios/rope.rs
  modified:
    - crates/liquidfun-test-protocol/src/catalog.rs
    - crates/liquidfun-test-protocol/src/catalog/model.rs
    - crates/liquidfun-test-protocol/src/catalog/resolve.rs
    - crates/liquidfun-test-protocol/src/ids.rs
key-decisions:
  - "Reuse the existing closed RigidWorldAction vocabulary for native scenario schedules instead of creating a second backend model."
  - "Keep exact-action construction crate-private and validate the catalog subset fail-closed during canonical replay."
  - "Materialize configured world and rope steps from ResolveRequest settings so custom settings remain replay-consistent."
patterns-established:
  - "Every native definition carries stable tags, exact defaults, typed evidence leaves, public-test IDs, and explicit regression/benchmark/visual eligibility."
  - "Catalog actions reference only declared deterministic semantic entities of the expected kind."
requirements-completed: []
metrics:
  duration: 22m25s
  completed: 2026-07-21
  tasks: 1
  files: 9
---

# Phase 11 Plan 04: Native Rigid, Joint, and Rope Catalog Summary

Twenty-two deterministic native catalog definitions now express the established rigid-world, all-joint, and standalone-rope behavior surface through the existing closed protocol vocabulary and typed evidence mappings.

## Performance

- **Duration:** 22m25s
- **Started:** 2026-07-21T23:47:41Z
- **Completed:** 2026-07-22T00:10:06Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added ten rigid definitions covering non-colliding lifecycle, contacts, stacks, sleep/wake, continuous collision, filtering, queries, callback control, mutation, and destruction.
- Added one definition for every `RigidJointKind`, including a four-body, dependency-ordered gear setup, plus a standalone-rope definition with rope-specific steps.
- Attached non-empty typed test/evidence mappings, exact default settings, stable tags, and explicit regression, benchmark, and visualization eligibility to every native definition.
- Preserved canonical replay under custom settings and rejected undeclared semantic references or unsupported catalog actions fail-closed.

## Task Commits

1. **Task 1: Encode rigid, joint, and rope scenarios as typed definitions** - `e05a1b4` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/catalog/scenarios/rigid.rs` - ten rigid-world definition families and deterministic contract tests.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/joints.rs` - all eleven joint-kind definitions and uniqueness/coverage tests.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/rope.rs` - standalone-rope schedule, custom-setting replay, and tamper rejection test.
- `crates/liquidfun-test-protocol/src/catalog/scenarios.rs` - shared source-oriented definition helpers and module routing.
- `crates/liquidfun-test-protocol/src/catalog/model/metadata.rs` - typed coverage, defaults, tags, and consumer eligibility.
- `crates/liquidfun-test-protocol/src/catalog/model.rs` - exact bounded action programs and metadata attachment.
- `crates/liquidfun-test-protocol/src/catalog/resolve.rs` - multi-setup scheduling, request materialization, and fail-closed validation.
- `crates/liquidfun-test-protocol/src/ids.rs` - standalone-rope semantic entity kind.
- `crates/liquidfun-test-protocol/src/catalog.rs` - native scenario module export.

## Decisions Made

- Reused `RigidWorldAction`, `RigidWorldWitness`, and stable protocol IDs rather than introducing backend, renderer, filesystem, or oracle dependencies.
- Kept exact action authoring crate-private; public persisted bytes are accepted only when their schedule, values, and semantic references satisfy the reviewed catalog subset.
- Left EXMP-01 and EXMP-03 globally pending. This plan authors canonical definitions, while later Phase 11 plans still need to make them runnable through examples, headless tooling, and the testbed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Extended the catalog core for genuine typed action plans**

- **Found during:** Task 1 RED implementation
- **Issue:** Plan 11-03 supported only gravity plus repeated configured steps, with no typed coverage metadata or standalone-rope entity identity; the required rigid/joint/rope definitions could not be represented truthfully.
- **Fix:** Added bounded exact setup/logical actions, typed metadata, request-specific step materialization, and `SemanticEntityKind::Rope` while preserving existing gravity-program bytes and tests.
- **Files modified:** `catalog/model.rs`, `catalog/model/metadata.rs`, `catalog/resolve.rs`, `ids.rs`
- **Commit:** `e05a1b4`

**2. [Rule 2 - Missing Critical Functionality] Closed generalized canonical replay fail-closed**

- **Found during:** Task 1 threat and diff review
- **Issue:** General action schedules would otherwise let persisted bytes bypass the prior gravity/step-only action validator.
- **Fix:** Added a closed catalog-action whitelist, exact numeric checks, declared-entity kind checks, bounded schedule validation, and a tampered-reference regression test.
- **Files modified:** `catalog/resolve.rs`, `catalog/scenarios/rope.rs`
- **Commit:** `e05a1b4`

**3. [Rule 3 - Blocking] Added native scenario module routing and shared helpers**

- **Found during:** Task 1 RED compilation
- **Issue:** The three planned scenario files had no Rust module entrypoint, and duplicating identity/metadata construction would risk drift.
- **Fix:** Added `catalog/scenarios.rs` and exported the reviewed scenario modules through `catalog.rs`.
- **Files modified:** `catalog.rs`, `catalog/scenarios.rs`
- **Commit:** `e05a1b4`

## Verification

- RED: focused scenario tests failed on the absent definitions and metadata API.
- GREEN: `cargo test -p liquidfun-test-protocol catalog::scenarios --all-features` passed 3/3 focused tests.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed across the workspace, integration suites, and doctests.

## Known Stubs

None.

## Issues Encountered

- A final threat review found that widening resolved action schedules required a new fail-closed validator. It was fixed before the task commit and protected by a tampered semantic-reference test.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The catalog now has source-oriented rigid/joint/rope definitions ready for registry composition and backend adapters.
- Global EXMP-01/EXMP-03 completion remains gated on the later executable examples, controller, replay, benchmark, and testbed plans.

## Self-Check: PASSED

- All nine task files exist.
- Task commit `e05a1b4` exists.
- No known stubs or unresolved high-severity ASVS L1 findings remain.

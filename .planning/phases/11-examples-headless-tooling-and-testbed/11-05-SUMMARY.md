---
phase: 11-examples-headless-tooling-and-testbed
plan: "05"
subsystem: test-protocol-catalog
tags: [rust, particles, particle-groups, queries, callbacks, deterministic-replay]
requires:
  - phase: 11-03
    provides: bounded catalog model and canonical resolver
  - phase: 11-04
    provides: exact-action catalog metadata and fail-closed replay seams
  - phase: 09-particle-storage-lifecycle-and-coupling
    provides: sealed Phase 9 action vocabulary and evidence registry
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    provides: sealed Phase 10 group operations and compatibility leaves
provides:
  - twenty-one representative particle, group, query, callback, and mutation definitions
  - typed mappings to the closed Phase 9 and Phase 10 evidence universes
  - fail-closed particle-system, particle, and particle-group ownership validation
  - request-materialized Phase 10 particle solver steps
affects: [11-06, 11-07, 11-09, 11-10, 11-11, 11-18]
tech-stack:
  added: []
  patterns:
    - deterministic Phase 9 and Phase 10 action schedules with semantic IDs
    - closed evidence identities validated against sealed manifests
    - lifecycle-aware canonical decoding for particle and group ownership
key-files:
  created:
    - crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs
    - crates/liquidfun-test-protocol/src/catalog/scenarios/queries_callbacks.rs
  modified:
    - crates/liquidfun-test-protocol/src/catalog/scenarios.rs
    - crates/liquidfun-test-protocol/src/catalog/model/metadata.rs
    - crates/liquidfun-test-protocol/src/catalog/resolve.rs
key-decisions:
  - "Represent particle-system pause only as Phase9ParticleAction::SetPaused; reserve session/controller pause for the later controller layer."
  - "Validate coverage IDs against the sealed Phase 9 branch registry and exact Phase 10/inherited leaf universe rather than accepting descriptive strings."
  - "Track system, particle, and group lifecycle during canonical decoding so kind-correct but stale or cross-owner identities fail closed."
patterns-established:
  - "LiquidFun-specific catalog definitions reuse the Phase 9/10 protocol vocabularies and never expose dense storage indices or private solver pass IDs."
  - "Every particle-group solver step is materialized from ResolveRequest settings before canonical encoding."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T00:31:14Z
metrics:
  duration: 13m38s
  completed: 2026-07-21
  tasks: 1
  files: 6
---

# Phase 11 Plan 05: Particle and Group Scenario Catalog Summary

Twenty-one deterministic definitions now cover representative particle lifecycle, contacts, forces, flags, queries, callbacks, mutations, group topology, and group solver behavior through the sealed Phase 9 and Phase 10 vocabularies.

## Performance

- **Duration:** 13m38s
- **Started:** 2026-07-22T00:17:36Z
- **Completed:** 2026-07-22T00:31:14Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added twelve particle definitions for lifecycle/storage, contacts and body coupling, forces/statistics, typed particle-system pause, and every representative public solver-flag family.
- Added five particle-group definitions for construction/append, join, split/reactive behavior, solid/rigid flags, and destruction.
- Added four query/callback/mutation definitions covering AABB continuation/termination, all four ray directives, lifecycle occurrence ordering, and accepted position/velocity mutations.
- Bound coverage metadata to exact sealed Phase 9 branches and Phase 10 compatibility leaves, with duplicate and unknown mappings rejected.
- Extended canonical decoding with bounded numeric checks and ordered system/particle/group ownership validation; a tampered mutation identity regression proves fail-closed behavior.

## Task Commits

1. **Task 1: Encode particle and group scenario definitions** - `ddba0a2` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs` - particle lifecycle, contact/coupling, force/statistics, pause, and solver-flag families.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs` - exact group creation/append, join, split, flags, stepping, and destruction schedules.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/queries_callbacks.rs` - AABB, ray, occurrence, accepted mutation, and rejected-reference coverage.
- `crates/liquidfun-test-protocol/src/catalog/scenarios.rs` - shared typed-evidence definition constructor and module routing.
- `crates/liquidfun-test-protocol/src/catalog/model/metadata.rs` - closed Phase 9/10 evidence identities and exact-universe validation.
- `crates/liquidfun-test-protocol/src/catalog/resolve.rs` - Phase 10 step materialization plus fail-closed particle/group lifecycle and ownership validation.

## Decisions Made

- Reused `Phase9ParticleAction` and `Phase10Operation` instead of inventing a parallel example or engine action vocabulary.
- Kept particle-system pause as an explicit physics action. No session/controller pause representation was added to the protocol catalog.
- Treated the checked-in Phase 9 and Phase 10 manifests as coverage authority; descriptive or unknown evidence labels are invalid metadata.
- Left EXMP-01 and EXMP-03 globally pending because later Phase 11 plans still must compose the registry and deliver runnable examples, headless execution, and testbed flows.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Extended typed evidence metadata for Phase 9 and Phase 10**

- **Found during:** Task 1 GREEN implementation
- **Issue:** Plan 11-04 metadata accepted only rigid-world witnesses, so particle and group mappings could not truthfully reference their sealed evidence authorities.
- **Fix:** Added typed Phase 9 and Phase 10 evidence identities, exact registry membership checks, and duplicate rejection while preserving rigid mappings.
- **Files modified:** `catalog/model/metadata.rs`, `catalog/scenarios.rs`
- **Verification:** All focused catalog tests and the exact repository gate passed.
- **Committed in:** `ddba0a2`

**2. [Rule 2 - Missing Critical Functionality] Added fail-closed Phase 9/10 canonical validation**

- **Found during:** Task 1 threat-model review
- **Issue:** Plan 11-04 deliberately rejected particle and group actions during persisted replay; merely allowing them would have bypassed ownership, lifecycle, float, and resource checks.
- **Fix:** Added bounded action-shape validation and ordered semantic state for systems, particles, and groups, including stale, duplicate, cross-owner, and teardown checks.
- **Files modified:** `catalog/resolve.rs`, `catalog/scenarios/queries_callbacks.rs`
- **Verification:** The tampered unknown-particle regression returns `InvalidIdentifier`; the complete Rust gate passed.
- **Committed in:** `ddba0a2`

**3. [Rule 3 - Blocking] Registered the three new scenario modules and shared evidence constructor**

- **Found during:** Task 1 RED compilation
- **Issue:** The planned files had no module declarations, and the existing helper accepted rigid witnesses only.
- **Fix:** Registered all three modules and routed rigid and particle/group definitions through one typed-evidence constructor.
- **Files modified:** `catalog/scenarios.rs`
- **Verification:** RED tests failed on absent definitions; all seven new focused tests passed after implementation.
- **Committed in:** `ddba0a2`

**Total deviations:** 3 auto-fixed (2 Rule 2, 1 Rule 3)

**Impact on plan:** The extensions are narrow protocol/catalog seams required for truthful mappings and secure replay. No runtime engine, renderer, dependency, FFI, network, or public storage-identity surface was added.

## Verification

- RED: each focused module test failed specifically because `definitions()` was absent.
- GREEN: 7/7 new focused particle, group, query, callback, and mutation tests passed.
- `cargo fmt --all` passed with the mandated isolated target directory.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed across workspace tests, integration suites, and doctests.
- `git diff --check` passed before the atomic task commit.

## Security Review

- All catalog action/entity/checkpoint limits remain bounded by named reviewed maxima.
- Canonical decode validates entity kind, liveness, uniqueness, ownership, provenance consistency, finite values, query/ray geometry, and request-matched solver settings before replay.
- Errors remain bounded semantic categories and disclose no dense indices, raw records, pointers, secrets, or unbounded diagnostics.
- No unresolved high-severity ASVS L1 or STRIDE finding remains.

## Known Stubs

None.

## Issues Encountered

- Final evidence review found descriptive group labels in the first GREEN draft. They were replaced with exact manifest leaves, and metadata now rejects any future unknown or duplicate mapping.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-06 can compose rigid, joint, rope, particle, group, query, callback, and mutation definitions into the closed registry.
- Later controller work can introduce session pause without conflating it with the established typed particle-system pause action.
- Global EXMP-01 and EXMP-03 remain gated on executable examples, headless tooling, and testbed delivery.

## Self-Check: PASSED

- All six task files exist.
- Task commit `ddba0a2` exists.
- No known stubs, unexpected threat surfaces, or unresolved high-severity ASVS L1 findings remain.

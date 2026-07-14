---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "11"
subsystem: differential-protocol
tags: [rust, jsonl, joints, rope, schemas, tolerances]
requires:
  - phase: 08-07
    provides: complete joint families and standalone rope behavior
  - phase: 08-10
    provides: bounded semantic reconstruction and world diagnostics
provides:
  - closed bounded phase8-v1 requests and observations for eleven joints, rope, callbacks, destruction, reconstruction, and diagnostics
  - strict nineteen-family witness registry retaining every Phase 6 and Phase 7 family
  - explicit thirty-seven-path Phase 8 tolerance policy with no wildcard or fallback selection
affects: [phase-8-native-adapter, differential-evidence, rigid-sign-off]
tech-stack:
  added: []
  patterns: [typed schema authority, bounded preallocation decode, exact-bit configuration, named computed policies]
key-files:
  created:
    - protocol/tolerances/phase8-v1.toml
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
key-decisions:
  - "Phase 8 is a strict protocol superset that requires all nineteen retained and new witness families."
  - "Transported configuration remains exact-bit while computed observations select one explicit named numeric policy."
  - "Joint, rope, directive, and timeline collections are bounded during strict decode before domain allocation."
patterns-established:
  - "Closed joint registry: declarations, mutations, observations, schemas, and tests enumerate all eleven kinds."
  - "Fail-closed policy: every observable path is explicit and unknown, missing, duplicate, wildcard, or widened rules are rejected."
requirements-completed: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T02:03:28Z
duration: 24min
completed: 2026-07-14
---

# Phase 8 Plan 11: Closed Phase 8 Protocol and Policy Summary

**A bounded fail-closed phase8-v1 contract now covers all eleven joint kinds, standalone rope, callback and destruction timing, reconstruction, diagnostics, and every retained rigid witness family.**

## Performance

- **Duration:** 24 min
- **Started:** 2026-07-14T01:39:28Z
- **Completed:** 2026-07-14T02:03:28Z
- **Tasks:** 1
- **Files modified:** 13

## Accomplishments

- Added explicit typed declarations, actions, mutations, result observations, dependency identities, and strict validation for every Phase 8 rigid behavior surface.
- Expanded the accepted request to nineteen exact witness families, including all ten new Phase 8 families and all eleven joint kinds, while retaining Phase 6 and Phase 7 witnesses unchanged.
- Added bounded rope declarations with N/N+1 rejection, strict joint dependency and mutation-kind validation, and fail-closed contact directives.
- Generated closed scenario and trace schemas from typed Rust authority and added a thirty-seven-path `phase8-v1` tolerance profile with exact structural/configuration rules and named computed-float policies.

## Task Commits

Each task was committed atomically:

1. **Task 08-11-01: Define bounded phase8-v1 requests, observations, schemas, policies, and witnesses** - `2dd82cf` (feat)

## Files Created/Modified

- `protocol/tolerances/phase8-v1.toml` - Defines thirty-seven explicit structural, exact-bit, ULP, absolute-relative, and dimensioned-absolute comparison paths.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs` - Adds eleven joint definitions, rope declarations, Phase 8 actions, mutations, and callback directives.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Enforces strict bounded decode, dependency ordering, mutation-kind compatibility, directive validity, and the complete witness registry.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Adds joint, rope, lifecycle, reconstruction, and diagnostic observations with semantic identities.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs` - Registers all ten new Phase 8 witness families and retains the prior nine families.
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs` - Presents the complete closed Phase 8 request and result schema.
- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs` - Parses, hashes, and fail-closes the exact Phase 8 policy registry.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Provides the canonical nineteen-family typed request fixture.
- `protocol/schemas/scenario-v1.schema.json` - Regenerated request schema presentation.
- `protocol/schemas/trace-v1.schema.json` - Regenerated trace schema presentation.

## Decisions Made

- Required the complete nineteen-family `ALL` registry for Phase 8 requests so retained coverage cannot silently disappear.
- Restricted gear dependencies to earlier revolute or prismatic declarations and restricted each mutation to supported joint kinds.
- Kept exact transported `f32` configuration as bit patterns and reserved tolerances for named computed semantic observations.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The initial staged diff exposed one trailing blank line in the new TOML policy; it was removed and the complete ordered Rust gate was rerun before commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The native Rust and C++ adapters can now consume one closed schema and produce typed Phase 8 observations without inventing fields or tolerance selection.
- No known blockers remain; the full ordered workspace Rust gate is green.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

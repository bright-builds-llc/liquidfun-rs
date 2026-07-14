---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "19"
subsystem: rigid-protocol-evidence
tags: [rust, protocol, joints, callbacks, destruction, typed-corpus]
requires:
  - phase: 08-18
    provides: complete live eleven-family joint solver call graph
provides:
  - validator-backed step-bearing Phase 8 witness corpus
  - closed observation contracts for joint, callback, destruction, rope, and reconstruction evidence
  - regenerated typed schemas and exact Phase 8 tolerance identity
affects: [08-20, 08-21, differential-adapters, rigid-sign-off]
tech-stack:
  added: []
  patterns: [ordered behavior witnesses, semantic cascade validation, closed lifecycle contracts]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase8.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation/phase8.rs
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
    - protocol/tolerances/phase8-v1.toml
key-decisions:
  - "Phase 8 claims are explicit behavior witnesses whose validation proves meaningful state, positive stepping, and post-step observation order."
  - "Gear-source and body destruction validation models dependent-gear cascades before accepting later actions or lifecycle evidence."
  - "Typed source remains schema authority; lifecycle ordinals begin at zero to match the native ordered event model."
requirements-completed: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T04:55:20Z
duration: 1h
completed: 2026-07-14
---

# Phase 8 Plan 19: Step-Bearing Typed Corpus, Schemas, and Policy Summary

**Every Phase 8 protocol claim now requires a meaningful positive world or rope step before closed semantic observations, with explicit joint, gear, callback, destruction, and reconstruction behavior witnesses.**

## Performance

- **Duration:** 1 hour
- **Started:** 2026-07-14T03:55:00Z
- **Completed:** 2026-07-14T04:55:20Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Replaced ten `*Covered` placeholders with 53 explicit witnesses spanning all eleven joint kinds, limit/motor modes, four RR/RP/PR/PP gear combinations, signed ratios, callbacks, cascades, rope, and reconstruction diagnostics.
- Added fail-closed request validation for nonzero mutation, positive stepping, post-step inspection, eligible callback geometry, repeated directive timing, touching destruction geometry, and dependent-gear cascade order.
- Added closed result contracts for finite typed observations, callback lifecycle order and multiplicity, destruction lifecycle order, explicit-destruction no-goodbye behavior, and reconstruction dependency ordering.
- Rebuilt the ten Phase 8 fixture timelines while retaining the nine Phase 6/7 timeline order and action counts exactly.
- Registered every new witness against explicit numeric and structural policy paths without wildcards or widened thresholds, producing canonical policy hash `72075452596abf03013832b19cf865315b2621654a3debf7f74f4c5a45146c55`.
- Regenerated the scenario and trace schemas from typed authority and kept presentation byte-stability tests green.

## Task Commits

1. **Task 08-19-01: Define and validate the complete step-bearing Phase 8 witness corpus** - `83dc3bf` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation/phase8.rs` - Enforces ordered family-specific Phase 8 behavior and structural coverage.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase8.rs` - Enforces Phase 8 observation and lifecycle contracts while housing extracted observation matching helpers.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Integrates Phase 8 validation, positive-step checks, and semantic joint/body cascade tracking.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Integrates checkpoint-level Phase 8 result validation.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs` - Defines the closed 53-witness Phase 8 behavior vocabulary and required action kinds.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs` - Protects step ordering, invalid cases, full family coverage, and retained Phase 6/7 action counts.
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs` - Aligns lifecycle ordinal schema rendering with zero-based typed events.
- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs` - Maps every behavior witness to closed numeric and structural paths.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Carries the rebuilt ten-timeline Phase 8 typed request and exact policy identity.
- `protocol/schemas/scenario-v1.schema.json` - Presents the explicit witness enum.
- `protocol/schemas/trace-v1.schema.json` - Presents zero-based lifecycle ordinals.
- `protocol/tolerances/phase8-v1.toml` - Records the updated closed policy profile.

## Decisions Made

- Kept Phase 8 validation family-specific so each claim encodes the exact prerequisite, step, and observation sequence instead of relying on action-kind presence alone.
- Modeled source-joint and body destruction cascades inside validation so dependent gear identity disappears at the same semantic boundary as production behavior.
- Split the large request and result validation additions into dedicated modules during the simplification pass, keeping the primary orchestration files focused.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Corrected lifecycle ordinal schema minima**

- **Found during:** Result/schema contract implementation
- **Issue:** Native lifecycle records are zero-based, but source and checked-in trace schemas required a minimum ordinal of one.
- **Fix:** Changed both typed rendering and tracked trace presentation to minimum zero and retained byte-stability verification.
- **Files modified:** `crates/liquidfun-test-protocol/src/schema/rigid_world.rs`, `protocol/schemas/trace-v1.schema.json`
- **Verification:** Typed schema presentation test and the full protocol suite passed.

**2. [Rule 2 - Missing Critical] Added semantic dependent-gear cascade tracking**

- **Found during:** Destruction corpus validation
- **Issue:** Structural validation removed only explicitly named joints, so a source-joint destruction could not prove the required dependent-gear-first cascade.
- **Fix:** Track joint body endpoints and gear dependencies, removing dependent gears on source-joint or body destruction before validating later actions.
- **Files modified:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs`
- **Verification:** Ordered destruction acceptance and rejection regressions passed.

## Issues Encountered

- The initial RED witness audit listed all eight world-step-dependent Phase 8 families because every legacy Phase 8 timeline lacked a positive step before observation.
- Updating gear topology invalidated two old negative-test IDs; the tests were corrected to target the rebuilt four-body topology while preserving their original rejection concern.

## Verification

- RED: `rigid_world_phase8_step_dependent_families_step_before_observation` reported all eight step-dependent families before corpus replacement.
- Focused Phase 8 protocol suite: 12 tests passed.
- Focused Phase 8 policy suite: 2 tests passed.
- Full protocol crate: 118 unit tests and 11 fixture tests passed.
- Schema renderer byte-stability tests passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 185 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 08-20 and 08-21 can execute one closed, validator-backed request independently through the native and pinned C++ adapters.
- CR-08-14-02 is structurally closed at the protocol boundary; adapter execution and canonical evidence remain the next steps.

## Self-Check: PASSED

- All twelve implementation files and this summary exist.
- Commit `83dc3bf` records Task 08-19-01.
- The accepted request uses policy hash `72075452596abf03013832b19cf865315b2621654a3debf7f74f4c5a45146c55`.
- No adapter execution files changed.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*

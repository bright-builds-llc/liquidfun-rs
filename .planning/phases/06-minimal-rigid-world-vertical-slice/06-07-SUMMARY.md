---
phase: 06-minimal-rigid-world-vertical-slice
plan: "07"
subsystem: rigid-world-policy-schema
tags: [rust, json-schema, toml, rigid-world, exact-bits, tolerance-policy]
requires:
  - phase: 04-math-settings-and-numerical-policy
    provides: Closed FieldPolicy vocabulary, exact float transport, horizons, collection semantics, and D0-D3 authority tiers
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "06"
    provides: Bounded rigid-world request/result records and exact declaration, manager, manifold, event, and destruction ordering
provides:
  - Closed 57-path phase6-v1 policy with exact comparison, phase-local horizons, ordered collections, and D1/D2 field authority
  - Closed protocol and scenario schemas for rigid-world declarations, actions, checkpoints, and both required witness families
  - Closed trace schema for body, fixture, contact, manifold, impulse, event, and destruction results
  - Read-only byte-stability checks for policy and schema presentations
affects: [06-08-native-adapter, 06-09-comparison, 06-10-cpp-oracle, 06-12-evidence-workflow]
tech-stack:
  added: []
  patterns: [closed field registry, fail-closed policy parsing, typed deterministic schema rendering, exact-first numeric policy]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - protocol/tolerances/phase6-v1.toml
  modified:
    - crates/liquidfun-test-protocol/src/tolerance.rs
    - crates/liquidfun-test-protocol/src/schema.rs
    - crates/liquidfun-test-protocol/src/schema/tests.rs
    - protocol/schemas/protocol-v1.schema.json
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
key-decisions:
  - "Classify all 34 structural and ordering paths as exact D1 metadata and all 23 finite float paths as exact-bit D2 metadata until canonical evidence justifies a field-specific widening."
  - "Use phase-local horizons and ordered collection semantics for every Phase 6 field; declaration, manager, manifold-point, event, and destruction order remain explicit semantic paths."
  - "Keep rigid schema construction in a cohesive schema/rigid_world.rs child module while ordinary protocol builds expose no filesystem write path."
patterns-established:
  - "Exact-first registry: a Phase 6 path is accepted only when its comparison kind, zero/non-finite rules, collection semantics, horizon, and evidence tier match the reviewed closed registry."
  - "Presentation authority: tracked schemas and TOML must equal deterministic in-memory renderings and repeated checks cannot mutate them."
requirements-completed: [RIGD-01, RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T05:01:13Z
duration: 9 min
completed: 2026-07-12
---

# Phase 6 Plan 07: Rigid Policies and Schemas Summary

**A closed exact-first `phase6-v1` registry now classifies every rigid-world result field, while deterministic typed renderers present the complete request, scenario, and trace wire contract.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-07-12T04:52:11Z
- **Completed:** 2026-07-12T05:01:13Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a strict 57-path policy profile that rejects missing, duplicate, wildcard, fallback, widened-threshold, horizon, evidence-tier, edge-case, and collection-policy changes.
- Classified every body transform/velocity/mass value, fixture and mixed material value, manifold geometry value, and normal/tangent impulse as exact-bit finite evidence under a fixed local horizon.
- Preserved exact declaration order, contact-manager order, manifold-point order, event/report order, destruction order, counts, identities, features, and lifecycle state through explicit discrete policy paths.
- Extended the protocol, scenario, and trace schemas with closed rigid-world declarations, all 16 Phase 6 actions, expected transitions, semantic contact occurrence identity, and complete result records.
- Proved schema and policy presentations are deterministic, newline-terminated, read-only, and byte-stable across repeated test runs.

## Task Commits

1. **Task 1: Add phase6-v1 field policies and deterministic schemas** - `533c011` (feat)
2. **Task 1 verification correction: Keep the test-only renderer warning-clean** - `7e5d6dc` (fix)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs` - Closed Phase 6 registry, validation, hashing, rendering, and negative tests.
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs` - Cohesive typed schema builders for rigid requests, scenarios, actions, checkpoints, and results.
- `protocol/tolerances/phase6-v1.toml` - Reviewed 57-field exact-first policy presentation.
- `protocol/schemas/protocol-v1.schema.json` - Adds the rigid-world request envelope.
- `protocol/schemas/scenario-v1.schema.json` - Adds the complete bounded rigid-world timeline presentation.
- `protocol/schemas/trace-v1.schema.json` - Adds the complete semantic rigid-world result presentation.

## Decisions Made

- Structural and ordering fields carry exact D1 policy metadata; finite physical floats begin exact under D2 metadata because local supported-toolchain evidence cannot claim canonical promotion.
- Every field uses a phase-local horizon and an ordered collection contract. The semantic paths distinguish declaration, manager, manifold-point, report, and destruction ordering without adding wildcard policies.
- The existing central schema renderer remains the only presentation entrypoint; the rigid-specific construction moved to a child module to keep the large registry navigable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed a non-canonical trailing policy blank line**

- **Found during:** Task 1 commit review
- **Issue:** The first staged policy artifact contained an extra blank line at EOF, which `git diff --check` reported.
- **Fix:** Made the in-memory renderer terminate with exactly one newline and aligned the tracked TOML bytes.
- **Files modified:** `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs`, `protocol/tolerances/phase6-v1.toml`
- **Verification:** Focused policy tests, `git diff --check`, and the complete ordered Rust gate passed after the correction.
- **Committed in:** `533c011` (amended task commit)

**2. [Rule 1 - Bug] Made the test-only renderer match exhaustive**

- **Found during:** Final package-scoped strict Clippy verification
- **Issue:** A wildcard match represented only the remaining `FieldComparison::Float` variant and violated the repository's warning-denied Clippy policy.
- **Fix:** Named the remaining float comparison variant explicitly without changing renderer behavior.
- **Files modified:** `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs`
- **Verification:** Package-scoped strict Clippy and the complete ordered Rust gate passed.
- **Committed in:** `7e5d6dc`

**Total deviations:** 2 auto-fixed bugs. **Impact:** Canonical presentation bytes and warning-denied verification are stricter; scope and runtime behavior are unchanged.

## Issues Encountered

None beyond the two auto-fixed presentation and verification defects.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-08 can serialize native adapter results against the closed schema.
- Plan 06-09 can align every comparison to an exact semantic path and reject unclassified fields before numeric comparison.
- No blockers remain.

## Self-Check: PASSED

- Created policy and rigid schema modules exist on disk.
- Task commits `533c011` and `7e5d6dc` exist and contain the complete implementation plus its warning-clean verification correction.
- Focused policy, schema, rigid-world decode, fixture, wildcard, byte-stability, Clippy, build, test, and diff checks pass.

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*

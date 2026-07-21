---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "27"
subsystem: particle-differential-comparator
tags: [rust, particle-groups, differential-testing, comparator, tolerances, fail-closed]

requires:
  - phase: 10-24
    provides: Strict Phase 10 semantic result schema and protocol validation
  - phase: 10-25
    provides: Native public-API execution and semantic capture
  - phase: 10-26
    provides: Pinned C++ oracle execution and pointer-independent semantic capture
provides:
  - Closed one-binding-per-field Phase 10 semantic comparison registry
  - Distinct D0 canonical byte-identity and D1 named semantic-policy modes
  - Contextual stable first-mismatch diagnostics and fail-closed malformed-result handling
affects: [10-28, 10-29, 10-30, 10-31, phase10-comparison-policy, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Validate complete policy registry and both semantic results before comparison
    - Walk validated records in declared source order and stop at the first contextual mismatch
    - Reserve numeric tolerance for explicitly named finite dynamic fields

key-files:
  created:
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/registry.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/numeric.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/groups.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/topology.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/witness.rs
    - crates/liquidfun-differential/tests/phase10_comparator.rs
  modified:
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_world/phase10.rs

key-decisions:
  - "Treat D0 as canonical JSON semantic-byte identity and D1 as an exhaustive source-ordered field walk through a closed named registry."
  - "Fail closed before comparison when policy bindings or semantic records are unknown, missing, duplicate, unbound, non-finite, or structurally invalid."
  - "Forbid Rust-private pass identifiers, traces, and inventories from receiving comparator policy bindings or influencing portable parity."

patterns-established:
  - "Closed comparator authority: every supported Phase 10 semantic field family has exactly one named policy and wildcard bindings are invalid."
  - "Stable diagnosis: mismatch signatures include scenario, operation, entity, index, field path, policy, expected value, and actual value."

requirements-completed: [PART-09, PART-10, PART-11, PART-12, PART-13, PART-18, TEST-01, TEST-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T06:40:00Z

duration: 2h 59m
completed: 2026-07-21
---

# Phase 10 Plan 27: Exhaustive Exact and Numeric Comparison Policy Summary

**Phase 10 parity now has one closed comparator that enforces canonical D0 byte identity, exhaustive D1 semantic policy, and fail-closed contextual mismatch reporting across every portable particle-group result family.**

## Performance

- **Duration:** 2h 59m
- **Started:** 2026-07-21T03:41:00Z
- **Completed:** 2026-07-21T06:40:00Z
- **Tasks:** 1
- **Files modified:** 17

## Accomplishments

- Added a mechanically validated registry of 50 named field policies covering exact identity, exact float bits, bounded ULPs, absolute/relative error, and dimensioned absolute error without wildcard fallback.
- Implemented distinct D0 and D1 modes: canonical serialized semantic bytes for deterministic replay and an exhaustive source-ordered semantic walk for cross-engine parity.
- Compared group state, particles, topology, contacts, lifecycle events, inherited observations, outcomes, and typed witnesses while preserving exact identity, ownership, connectivity, and record order.
- Rejected malformed or incomplete results, duplicate and reordered records, non-finite numeric values, missing or duplicate policy bindings, unknown paths, and attempted private pass-field policies before parity can be claimed.
- Added mutation witnesses for every record family, tolerance boundaries and one-over values, non-finite classes, duplicate/reorder/drop cases, D0/D1 distinction, and stable contextual first-mismatch signatures.

## Task Commits

Each task was committed atomically:

1. **Task 1: Compare every Phase 10 semantic field under closed policy** - `e54daf1` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs` - Exposes D0/D1 modes, validated comparison entrypoints, outcomes, and contextual mismatch diagnostics.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/registry.rs` - Defines and validates the closed 50-field policy registry.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/numeric.rs` - Applies finite exact-bit, ULP, absolute/relative, and dimensioned policies.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/records.rs` - Routes exhaustive semantic record comparison by family.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/groups.rs` - Compares group, particle, contact, and lifecycle state.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/topology.rs` - Compares pair/triad topology, inherited observations, and operation outcomes.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/witness.rs` - Compares every typed public witness variant exhaustively.
- `crates/liquidfun-differential/tests/phase10_comparator.rs` - Proves registry closure, mutation detection, tolerance boundaries, malformed rejection, and diagnostics.
- `crates/liquidfun-differential/src/rigid_world.rs` and `phase10.rs` - Publish the Phase 10 comparison surface.
- Existing Phase 9/10 native, oracle, protocol, and corpus files - Carry narrow lint rationales and mechanically equivalent cleanup required by the current exact all-target Clippy gate.

## Decisions Made

- D0 compares canonical JSON bytes of already validated Phase 10 semantic observations, while D1 compares the same schema through the explicit named policy registry.
- Numeric policy never repairs or normalizes malformed input: both sides must pass protocol semantic validation, including finite values and unique source-ordered identities, before comparison.
- Collection order is semantic. Duplicate, reorder, and drop mutations fail validation or report the first exact contextual mismatch rather than being canonicalized away.
- Private solver pass identity remains deliberately absent from portable comparison; even a registry path containing `pass_id`, `pass_trace`, or `pass_inventory` is rejected.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split the exhaustive comparator into cohesive reviewable modules**

- **Found during:** Task 1 implementation
- **Issue:** Keeping the registry, numerical policies, and every schema-family walker in the single named comparator artifact would exceed repository file-size guidance and obscure closure review.
- **Fix:** Kept `comparator.rs` as the public facade and extracted private registry, numeric, group, topology, and witness modules; every production comparator file remains below 500 lines.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs` and `crates/liquidfun-differential/src/rigid_world/phase10/comparator/**`
- **Verification:** Focused comparator tests and the complete exact Rust gate passed.
- **Committed in:** `e54daf1`

**2. [Rule 3 - Blocking] Repair current-toolchain Clippy debt exposed by the mandatory all-target gate**

- **Found during:** Task 1 pre-commit verification
- **Issue:** The exact warning-denied all-target/all-feature Clippy command rejected pre-existing Phase 9/10 exhaustive fixtures and captures for `similar_names`, `too_many_lines`, `too_many_arguments`, `needless_pass_by_value`, range-end, redundant-closure, and identical-match-arm findings.
- **Fix:** Added narrow rationale-scoped lint allowances where exhaustive source-shaped records are intentional, replaced closures with method references, named an insertion index, and merged equivalent match arms without changing behavior.
- **Files modified:** `phase10/native/capture.rs`, `phase10/native/evidence.rs`, `phase9/evidence.rs`, `tests/phase10_native.rs`, `tests/phase10_oracle.rs`, `tests/phase10_protocol.rs`, and `tests/phase9_corpus.rs`
- **Verification:** The exact warning-denied Clippy gate, focused comparator tests, all-target build, and full all-feature tests passed.
- **Committed in:** `e54daf1`

**Total deviations:** 2 auto-fixed (1 structural seam, 1 blocking verification repair).
**Impact on plan:** Comparator semantics and protocol scope are unchanged. The extra private modules improve auditability, and lint repairs are rationale-scoped or mechanically behavior-preserving.

## Issues Encountered

- macOS provenance scanning delayed first launches of newly linked test executables. The mandatory test gate was allowed to finish; every suite and all 19 doctests passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-28 can build its closed five-family corpus on one validated comparator with explicit exact and numeric policy authority.
- D0 replay and D1 cross-engine mismatches now produce stable contextual signatures suitable for minimization and evidence validation.
- No private pass details or unregistered numeric tolerances can silently enter compatibility evidence.
- No blockers remain.

## Self-Check: PASSED

- Confirmed implementation commit `e54daf1` exists and contains only the scoped comparator, tests, exports, and documented lint-gate repairs.
- Confirmed `cargo test -p liquidfun-differential --all-features --test phase10_comparator` passes all 7 focused tests.
- Confirmed the implementation commit was preceded by the exact mandatory Rust gate: format, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests including 409 unit tests and 19 doctests.
- Confirmed `.planning/config.json`, `.planning/agent-history.json`, and `.planning/current-agent-id.txt` were not staged or committed.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-21*

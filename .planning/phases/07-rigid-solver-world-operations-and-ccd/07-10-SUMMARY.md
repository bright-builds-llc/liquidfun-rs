---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "10"
subsystem: testing
tags: [rust, json-lines, json-schema, differential-testing, ccd, rigid-world]

requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    provides: Closed Phase 6 rigid-world request/result protocol and evidence policy
provides:
  - Closed bounded Phase 7 rigid-world actions and semantic observations
  - Seven-family Phase 7 witness registry alongside retained Phase 6 compatibility families
  - Explicit per-observable Phase 7 structural, multiset, set, absolute-relative, absolute, and ULP policies
affects: [07-11, rigid-differential-adapters, ccd, world-queries, origin-shift]

tech-stack:
  added: []
  patterns: [semantic-only CCD evidence, closed witness-policy mapping, evidence-only query canonicalization]

key-files:
  created: [protocol/tolerances/phase7-v1.toml]
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
    - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
    - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json

key-decisions:
  - "Retain REQUIRED as the two-family Phase 6 compatibility corpus and expose PHASE7_REQUIRED plus ALL for the new closed registry."
  - "Expose CCD only as Complete, ContinuousPending, or bounded continuous-work exhaustion; no candidate, cache, or TOI counter state crosses the boundary."
  - "Treat query occurrences as a multiplicity-preserving multiset and equal-minimum ray identities as a set only in evidence comparison."

patterns-established:
  - "Protocol evolution keeps generated schema, runtime bounds, tagged enums, and semantic validation synchronized."
  - "Every Phase 7 witness family maps to named policy paths, and unknown observable paths have no fallback."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T01:36:38Z

duration: 23min
completed: 2026-07-12
---

# Phase 7 Plan 10: Protocol and Evidence Policy Summary

**A closed, bounded Phase 7 rigid-world protocol now carries semantic body, step, query, ray, and origin-shift evidence under an explicit witness and tolerance registry.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-07-13T01:13:40Z
- **Completed:** 2026-07-13T01:36:38Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Added checked Phase 7 body controls, world configuration, configured stepping, AABB query, ray-cast, and origin-shift actions with finite-value, identity, directive, and work-budget validation.
- Added semantic-only completion, bounded partial-progress, body-control, query, ray, and shift observations without exposing CCD storage or candidate details.
- Registered bounded witnesses for force policy, multi-contact islands and warm starts, sleeping/waking, CCD and resume, query/ray edge cases, and origin-shift covariance.
- Added a 42-field closed tolerance profile with exact structure, multiset query occurrences, set-valued ray ties, and path-specific numeric rules.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend the closed rigid-world protocol with Phase 7 operations** - `d09937b` (feat)
2. **Task 2: Register Phase 7 witnesses and closed comparison policies** - `e0ec567` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs` - Closed Phase 7 actions, directives, and action-kind registry.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Semantic Phase 7 observations and bounded result transport.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Fail-closed Phase 7 semantic and resource validation.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs` - Seven bounded Phase 7 witness families and their required actions.
- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs` - Strict Phase 7 policy parser, witness-policy linkage, and rejection tests.
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs` - Closed schema generation for all registered families and Phase 7 records.
- `protocol/tolerances/phase7-v1.toml` - Reviewed per-observable comparison manifest.
- `protocol/schemas/scenario-v1.schema.json` - Generated Phase 7 request and witness presentation.
- `protocol/schemas/trace-v1.schema.json` - Generated semantic Phase 7 result presentation.

## Decisions Made

- Kept the checked-in Phase 6 fixture valid by preserving its two-family `REQUIRED` contract while separately naming all Phase 7-required families; schema generation uses the complete registry.
- Raised request/result timeline bounds from two to nine, exactly matching the complete closed family registry rather than opening an arbitrary collection limit.
- Kept all production traversal ordering untouched; multiset/set classification exists only in the evidence policy.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Kept the Phase 6 native adapter exhaustive during protocol extension**
- **Found during:** Task 1
- **Issue:** New closed action variants and checkpoint observations made the existing differential adapter non-exhaustive before Plan 07-11 implements them.
- **Fix:** Added explicit fail-closed unsupported-action branches and empty Phase 6 observation output.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world.rs`
- **Verification:** Workspace clippy, build, and tests passed.
- **Committed in:** `d09937b`

**2. [Rule 2 - Missing Critical] Added protocol and schema boundary regression tests**
- **Found during:** Task 1
- **Issue:** The listed implementation files alone did not protect closed variant decoding, boundary rejection, semantic-only CCD output, or generated-schema currentness.
- **Fix:** Added focused rigid-world and schema tests, including private-state exclusion assertions.
- **Files modified:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs`, `crates/liquidfun-test-protocol/src/schema/tests.rs`
- **Verification:** All 102 protocol unit tests and 11 fixture tests passed.
- **Committed in:** `d09937b`, `e0ec567`

**3. [Rule 3 - Blocking] Expanded timeline bounds and generated enum presentation with the registry**
- **Found during:** Task 2
- **Issue:** The previous two-timeline bounds and schema enum would reject registered Phase 7 families before adapter work could use them.
- **Fix:** Set the exact nine-family bound and generated schemas from `RigidWorldWitnessFamily::ALL` while retaining Phase 6 required-family validation.
- **Files modified:** `result.rs`, `validation.rs`, `schema/rigid_world.rs`, generated scenario and trace schemas
- **Verification:** Schema byte-currentness and full protocol tests passed.
- **Committed in:** `e0ec567`

---

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** All changes close compile, regression, or protocol-reachability gaps required by the planned contract; no production physics scope was added.

## Issues Encountered

- Initial Task 1 compilation exposed the expected exhaustiveness gap while action validation and schema wiring were incomplete; completing the closed mappings resolved it.
- Strict clippy identified protocol-specific independent boolean flags and an inline JSON test helper; narrow reasoned allowances preserve the semantic wire shape and test readability.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-11 can implement the Rust and C++ adapters against one closed action/result/schema contract and seven explicit Phase 7 witness families.
- The adapter must use the computed `phase7-v1` profile identity, preserve semantic multiplicity/tie handling, and continue to fail closed on unimplemented actions.
- No blockers remain.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-12*

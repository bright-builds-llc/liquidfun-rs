---
phase: 11-examples-headless-tooling-and-testbed
plan: "06"
subsystem: testing
tags: [scenario-registry, provenance, deterministic-projection, differential-testing]
requires:
  - phase: 11-04
    provides: Reviewed native rigid-body, joint, and rope scenario families
  - phase: 11-05
    provides: Reviewed particle, group, query, and callback scenario families
provides:
  - Closed 43-scenario registry in stable slug and version order
  - Validated cross-consumer test, evidence, regression, benchmark, visualization, corpus, and compatibility mappings
  - Deterministic checked JSON projection derived from the typed Rust authority
affects: [11-corpus-closure, differential-runners, benchmarks, testbed, release-evidence]
tech-stack:
  added: []
  patterns: [typed-authority-with-read-only-projection, closed-reference-validation, bounded-embedded-authority]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/catalog/mapping.rs
    - crates/liquidfun-test-protocol/src/catalog/mapping/projection.rs
    - crates/liquidfun-test-protocol/tests/catalog_registry.rs
    - reference/scenario-catalog.json
  modified:
    - crates/liquidfun-test-protocol/src/catalog/scenarios.rs
    - crates/liquidfun-test-protocol/src/catalog/model.rs
    - tools/xtask/src/differential.rs
key-decisions:
  - "The typed Rust registry is the sole runtime authority; scenario-catalog.json is an exact, read-only checked projection."
  - "Every mapping resolves against checked test, evidence, upstream-corpus, and compatibility authorities before consumer eligibility is accepted."
  - "Registry order is canonical slug then scenario version, independent of family declaration order."
patterns-established:
  - "Closed registry: compose every reviewed family, sort once, then reject duplicate or missing identities."
  - "Projection integrity: hash compact canonical scenario records and compare exact pretty-printed tracked bytes without rewriting."
requirements-completed: [TEST-03, EXMP-01, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T01:38:03Z
duration: 20min
completed: 2026-07-22
---

# Phase 11 Plan 06: Closed Scenario Registry and Mappings Summary

**A closed 43-scenario typed registry now validates every consumer and provenance join and produces one byte-stable, read-only JSON review projection.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-22T01:18:15Z
- **Completed:** 2026-07-22T01:38:03Z
- **Tasks:** 1
- **Files modified:** 11

## Accomplishments

- Composed all six reviewed family modules into a unique registry sorted by stable slug and scenario version.
- Required every scenario to have one complete mapping to public tests, reviewed oracle-equivalent evidence, regression use, benchmark and visualization eligibility, upstream corpus identities, and compatibility ledger leaves.
- Rejected duplicate, missing, unknown, stale, contradictory, title-derived, and incomplete seeded-generator records with bounded semantic error categories.
- Generated `reference/scenario-catalog.json` from the typed renderer and protected it with exact byte and SHA-256 identity checks that never rewrite tracked content.
- Added ten focused registry tests plus the planned `cargo xtask differential check-protocol` entrypoint.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the closed registry, mappings, and deterministic projection** - `bc48122` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/catalog/mapping.rs` - Closed mapping model, checked authority resolution, and registry validation.
- `crates/liquidfun-test-protocol/src/catalog/mapping/projection.rs` - Deterministic serialization, digest calculation, and exact tracked-byte comparison.
- `crates/liquidfun-test-protocol/tests/catalog_registry.rs` - Positive closure and focused duplicate, stale, contradictory, and drift rejection tests.
- `reference/scenario-catalog.json` - Generated machine-readable projection of all 43 reviewed scenarios.
- `crates/liquidfun-test-protocol/src/catalog/scenarios.rs` - Stable composition of all reviewed scenario families.
- `crates/liquidfun-test-protocol/src/catalog/model.rs` - Public scenario-version accessor and bounded mapping/projection error kinds.
- `crates/liquidfun-test-protocol/src/catalog.rs` - Public catalog mapping surface.
- `crates/liquidfun-differential/src/session.rs` - Compiler-compatible constant completed-step calculation.
- `tools/xtask/src/differential.rs` - Planned `check-protocol` command dispatch.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs` - Clippy-clean direct action accessor.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs` - Documented lint disposition for the ordered lifecycle inventory.

## Decisions Made

- Kept the Rust registry authoritative and made JSON a pure presentation/join artifact, preventing two writable sources of truth.
- Embedded the checked upstream corpus and compatibility ledger under explicit 4 MiB and 4,096-record bounds so every mapping resolves locally and deterministically.
- Used stable semantic identifiers only; display titles are rejected as identity inputs.
- Kept all new behavior inside private protocol/tooling crates, preserving the published `liquidfun` crate's renderer-, oracle-, and foreign-runtime-free dependency boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced an unsupported constant-expression helper**

- **Found during:** Task 1 scoped protocol verification
- **Issue:** A pre-existing Phase 11 session accessor used `unwrap_or_default()` in a `const fn`, which the pinned compiler rejected while building the differential tooling.
- **Fix:** Replaced it with an explicit constant-compatible `match` preserving the same zero and subtraction semantics.
- **Files modified:** `crates/liquidfun-differential/src/session.rs`
- **Verification:** The scoped protocol command and full ordered Rust gate passed.
- **Committed in:** `bc48122`

**2. [Rule 3 - Blocking] Exposed the planned protocol-check command**

- **Found during:** Task 1 scoped protocol verification
- **Issue:** The internal protocol check existed, but `cargo xtask differential check-protocol` was not registered in command dispatch.
- **Fix:** Added the closed command to usage and dispatched it directly to the existing check routine.
- **Files modified:** `tools/xtask/src/differential.rs`
- **Verification:** `cargo xtask differential check-protocol` passed four schema presentation tests and eleven fixture tests.
- **Committed in:** `bc48122`

**3. [Rule 3 - Blocking] Cleared two lint blockers in composed family modules**

- **Found during:** Task 1 affected-package lint verification
- **Issue:** Composing the Phase 11-05 family modules exposed a redundant test closure and an over-length warning for one intentionally ordered four-definition inventory.
- **Fix:** Used the method directly and documented the narrow inventory-level lint disposition.
- **Files modified:** `crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs`, `crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs`
- **Verification:** Targeted protocol Clippy and the full ordered Rust gate passed with warnings denied.
- **Committed in:** `bc48122`

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** The fixes only enabled the planned compiler, command, and lint verification paths; no consumer scope or architecture changed.

## Issues Encountered

- An additional broad private-workspace Clippy probe reached unrelated pre-existing Phase 9/10 warning-denied failures outside this plan. Those files were left untouched; the required exact repository gate and the targeted protocol library/test lint both pass.
- The aggregate `cargo xtask check` command reaches a pre-existing packaged-crate isolation failure where Phase 10 tests reference artifacts not present in the package. The plan's now-supported scoped `differential check-protocol` command passes and is the authoritative verification for this task.

## Security and Threat Review

- Canonical bytes, duplicate identities, stale references, and projection drift are rejected before a registry is accepted.
- Checked authority parsing is bounded by byte and record limits, and errors expose only semantic categories and stable IDs.
- No network endpoint, authentication path, runtime file writer, unsafe block, schema migration, or published-crate dependency was introduced.
- All six planned STRIDE dispositions are covered; no unresolved high-severity ASVS L1 finding remains.

## Known Stubs

None. The modified code and generated projection contain no goal-blocking placeholders or unwired mock data.

## Requirements Status

Plan 11-06 supplies its assigned TEST-03, EXMP-01, and EXMP-03 evidence. Global requirement completion remains deferred to Phase 11 closure so later corpus, runner, benchmark, and testbed plans can finish their portions before the milestone ledger is marked complete.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Corpus closure, runners, regressions, benchmarks, and the testbed can now join through one closed scenario identity and mapping authority.
- No Plan 11-06 blocker remains. The unrelated pre-existing private-workspace lint and packaged-test isolation findings remain outside this plan's scope.

## Self-Check: PASSED

All four created artifacts, the summary, and implementation commit `bc48122` exist; the registry projection test confirms the tracked bytes exactly match the in-memory typed authority.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*

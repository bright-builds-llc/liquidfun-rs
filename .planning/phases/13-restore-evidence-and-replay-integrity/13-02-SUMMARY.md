---
phase: 13-restore-evidence-and-replay-integrity
plan: '02'
subsystem: compatibility-testing
tags: [catalog-replay, semantic-diff, legacy-projection, sha256, regression-evidence]
requires:
  - phase: 11-compatibility-harness-public-observability-and-evidence
    provides: Exact resolved-byte catalog runner, canonical checkpoints, and reviewed regression manifest
provides:
  - Typed three-way replay drift classification with deterministic semantic paths
  - Versioned legacy physics projection preserving reviewed D0 identity
  - Exact-head rigid-stack capture-schema diagnosis without identity promotion
affects: [13-03-staging, 13-04-canonical-regeneration, replay-evidence]
tech-stack:
  added: []
  patterns:
    - Authority-ordered resolved-input, physics-projection, then expanded-capture diagnosis
    - Stable collection-length divergence for expanded diagnostic arrays
key-files:
  created:
    - crates/liquidfun-differential/src/fixtures/replay/diagnosis.rs
  modified:
    - crates/liquidfun-differential/src/fixtures/replay/catalog.rs
    - crates/liquidfun-differential/src/runner/catalog.rs
    - crates/liquidfun-differential/tests/catalog_regressions.rs
key-decisions:
  - "Preserve the reviewed D0 digest through a versioned legacy projection that retains every historical field and represents debug primitives as the historical empty array."
  - "Treat expanded debug primitives as capture-schema evidence only after sealed resolved bytes and the legacy parity projection match."
  - "Report array-length divergence before element contents so repeated diagnoses remain stable without accepting native-emitted values."
patterns-established:
  - "Diagnosis before promotion: exact sealed bytes are compared first, followed by parity-bearing semantics and only then expanded diagnostics."
  - "Legacy identity preservation: historical serialization is reconstructed explicitly rather than changing the reviewed manifest."
requirements-completed: [COMP-08, TEST-07, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-25T20:35:38Z
duration: 25min
completed: 2026-07-25
---

# Phase 13 Plan 02: Catalog Replay Drift Diagnosis Summary

**Authority-ordered replay diagnosis that proves `rigid-stack-v1` changed only through expanded debug capture while preserving its reviewed legacy physics identity**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-25T20:11:01Z
- **Completed:** 2026-07-25T20:35:38Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added typed resolved-scenario, physics, and capture-schema drift classifications with schema identities, sealed-input digests, first semantic paths, and both compared values.
- Reused the exact resolved-byte native runner and replaced the replay-local native assertion helper that could be mistaken for independent evidence.
- Reconstructed the reviewed checkpoint projection with every historical parity field, preserving the accepted D0 identity without editing the manifest.
- Proved `rigid-stack-v1` first diverges at `$.checkpoints[0].debug_primitives.length` under the expanded checkpoint projection.
- Added fail-closed schema tests and exact replay assertions that the regression manifest and sealed fixture remain byte-identical.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add semantic-path replay diagnosis and drift taxonomy** - `4b98ecd` (feat)
2. **Task 2: Prove the exact-head rigid-stack cause without re-baselining** - `d8afada` (test)

## Files Created/Modified

- `crates/liquidfun-differential/src/fixtures/replay/diagnosis.rs` - Typed diagnosis records, schema validation, semantic path ordering, and legacy/current checkpoint projections.
- `crates/liquidfun-differential/src/fixtures/replay/catalog.rs` - Exact sealed-byte replay integration with repeated legacy D0 checks and structured capture-schema diagnosis.
- `crates/liquidfun-differential/src/runner/catalog.rs` - Reusable native execution entrypoint for an already resolved scenario.
- `crates/liquidfun-differential/src/fixtures/replay.rs` - Diagnosis module exposure.
- `crates/liquidfun-differential/src/fixtures.rs` - Public diagnosis API re-exports.
- `crates/liquidfun-differential/tests/catalog_regressions.rs` - Three-way classification, fail-closed schema, and exact rigid-stack regression coverage.

## Decisions Made

- The historical checkpoint identity includes an empty `debug_primitives` field; the legacy projection preserves that exact field order and value rather than omitting it.
- Repeated D0 authority is measured over the legacy physics projection. Expanded debug capture is validated and diagnosed separately, so it cannot silently redefine the reviewed identity.
- Array length is the first semantic divergence when one capture adds diagnostic records. This yields stable compared values without copying a Rust-emitted primitive into an expectation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Stabilized first divergence for expanded diagnostic arrays**

- **Found during:** Task 1 (Add semantic-path replay diagnosis and drift taxonomy)
- **Issue:** Returning the first added primitive object made the diagnosis value inherit non-authoritative native diagnostic coordinates across repeated worlds.
- **Fix:** Compare unequal array lengths first and report the deterministic `.length` semantic path and counts.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/replay/diagnosis.rs`
- **Verification:** Repeated complete catalog replays compare equal and the exact rigid-stack diagnosis test passes.
- **Committed in:** `4b98ecd`

**2. [Rule 3 - Blocking] Exposed the new diagnosis module through existing fixture APIs**

- **Found during:** Task 1 (Add semantic-path replay diagnosis and drift taxonomy)
- **Issue:** The planned new module and focused integration tests could not compile without wiring the module and public re-exports.
- **Fix:** Added the replay submodule and explicit fixture-level exports.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/replay.rs`, `crates/liquidfun-differential/src/fixtures.rs`
- **Verification:** Focused diagnosis tests and targeted differential clippy pass.
- **Committed in:** `4b98ecd`

***

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes were required for deterministic, callable diagnosis behavior; no reviewed identity, fixture, or manifest changed.

## Issues Encountered

- The shared repository Cargo target was occupied by another execution. Verification used an isolated temporary `CARGO_TARGET_DIR`; all required commands completed successfully.
- The current expanded capture is not itself the historical D0 authority. The legacy projection repeats byte-identically, while expanded diagnostics are compared structurally and remain untrusted observed data.

## Verification

- `cargo test -p liquidfun-differential --test catalog_regressions diagnosis_` - 4 passed.
- `cargo test -p liquidfun-differential --test catalog_regressions rigid_stack_v1_diagnosis -- --exact` - passed.
- `cargo clippy -p liquidfun-differential --all-targets --all-features -- -D warnings` - passed.
- `cargo fmt --all` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo build --all-targets --all-features` - passed.
- `cargo test --all-features` - passed.
- Reviewed regression manifest and `rigid-stack-v1` fixture - no diff.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 13-03 can stage evidence using a stable typed replay diagnosis without changing accepted identities.
- Plan 13-04 retains responsibility for canonical independent D1 evidence and any explicitly reviewed promotion.
- No blocker remains from Plan 13-02.

***

*Phase: 13-restore-evidence-and-replay-integrity*
*Completed: 2026-07-25*

## Self-Check: PASSED

- Created diagnosis and summary files exist.
- Task commits `4b98ecd` and `d8afada` exist.
- Reviewed regression manifest and sealed `rigid-stack-v1` fixture remain unchanged.

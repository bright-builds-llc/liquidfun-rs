---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "06"
subsystem: differential-comparison
tags: [rust, semantic-comparison, tolerance-policy, failure-signature, delta-debugging]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Validated scenario/trace domain types, exhaustive tolerance vocabulary, deterministic schemas, and permanent Phase-2 fixtures from Plans 02-03 through 02-05
provides:
  - Compatibility-gated exhaustive typed semantic comparison
  - Narrow ordered/set/multiset canonicalization with stable key and tie-breaker ordering
  - Deterministic first-divergence reports and stable failure signatures
  - Validity-preserving bounded same-signature scenario minimization
affects: [02-07, 02-10, 02-11, differential-runner, regression-fixtures, replay]

tech-stack:
  added: []
  patterns: [validated inputs before semantic traversal, typed first-divergence paths, pure injected-evaluator reduction, protocol-owned candidate revalidation]

key-files:
  created:
    - crates/liquidfun-differential/src/canonical.rs
    - crates/liquidfun-differential/src/comparator.rs
    - crates/liquidfun-differential/src/report.rs
    - crates/liquidfun-differential/src/minimizer.rs
    - crates/liquidfun-differential/tests/comparison.rs
    - crates/liquidfun-differential/tests/minimizer.rs
    - crates/liquidfun-test-protocol/src/scenario/reduction.rs
  modified:
    - crates/liquidfun-differential/src/lib.rs
    - crates/liquidfun-test-protocol/src/failure.rs
    - crates/liquidfun-test-protocol/src/scenario.rs
    - crates/liquidfun-test-protocol/src/trace.rs

key-decisions:
  - "Reject incompatible request, scenario, tolerance, schema, and engine-role identities before examining any semantic observable."
  - "Define failure identity solely by checkpoint, phase, typed semantic path, and mismatch kind so changed values retain a signature while later or different-kind failures do not."
  - "Keep reduction pure and deterministic by injecting typed signatures plus logical elapsed time, while the protocol crate revalidates every in-memory candidate before evaluation."

patterns-established:
  - "Comparison authority: exhaustive matches over closed protocol policies and typed trace fields; no generic JSON path, global epsilon, or global sorting."
  - "Diagnostic authority: one typed report owns machine JSON, human rendering, exact bits, decimal diagnostics, adjacent checkpoints, and reduction signatures."
  - "Reduction authority: stable checkpoint-first then command-group transforms, exact-signature retention, and explicit complete/attempt/deadline outcomes."

requirements-completed:
  - COMP-06
  - COMP-07
  - COMP-08
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T08:29:08Z

duration: 27 min
completed: 2026-07-10
---

# Phase 2 Plan 06: Typed Semantic Comparison and Same-Signature Reduction Summary

**Validated traces now pass through exhaustive field policies into deterministic first-divergence evidence, while a pure bounded reducer preserves the exact failure signature and serialized scenario value.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-07-10T08:02:00Z
- **Completed:** 2026-07-10T08:29:08Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Added compatibility-first comparison for request/scenario/tolerance/schema/engine identities, exact discrete fields, every Phase-2 float policy, and typed ordered/set/multiset semantics.
- Added stable checkpoint/phase/path/kind failure signatures, bounded adjacent-checkpoint context, exact float bits plus decimal diagnostics, and deterministic machine/human rendering from one typed report.
- Added deterministic hierarchical checkpoint/command reduction that rejects invalid candidates before evaluation, retains only the identical signature, stops explicitly on attempt/deadline bounds, and returns canonical scenario bytes plus original named/seeded metadata.
- Added 17 focused comparison and minimizer integration tests, while preserving all 49 existing protocol unit/fixture tests and Cargo-only package isolation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement exhaustive typed comparison and narrow canonicalization** - `133a744` (`feat`)
2. **Task 2: Produce stable first-divergence reports** - `af81ba8` (`feat`)
3. **Task 3: Minimize valid scenarios while preserving the failure signature** - `a76f815` (`feat`)

## Files Created/Modified

- `crates/liquidfun-differential/src/canonical.rs` - Typed ordered, set, and multiset collection semantics with stable key/tie-breaker canonicalization only for explicitly unordered values.
- `crates/liquidfun-differential/src/comparator.rs` - Pure compatibility gate, exhaustive discrete/float traversal, and first-difference selection.
- `crates/liquidfun-differential/src/report.rs` - Typed semantic paths, stable signatures, exact numeric evidence, adjacent checkpoints, and deterministic renderers.
- `crates/liquidfun-differential/src/minimizer.rs` - Pure deterministic hierarchical reducer with injected evaluation and explicit budgets.
- `crates/liquidfun-differential/src/lib.rs` - Exports comparison, report, canonicalization, and minimization surfaces.
- `crates/liquidfun-differential/tests/comparison.rs` - Focused compatibility, numeric boundary, special-float, collection, signature, and rendering coverage.
- `crates/liquidfun-differential/tests/minimizer.rs` - Successful reduction, invalid-reference, changed-signature, transform-order, attempt, deadline, and canonical-value coverage.
- `crates/liquidfun-test-protocol/src/trace.rs` - Read-only validated trace/checkpoint accessors required by typed comparison.
- `crates/liquidfun-test-protocol/src/failure.rs` - Boxed bounded harness evidence to keep the required result error type Clippy-clean.
- `crates/liquidfun-test-protocol/src/scenario.rs` - Registers the focused scenario reduction child module.
- `crates/liquidfun-test-protocol/src/scenario/reduction.rs` - Typed candidate transforms and strict in-memory scenario reparsing.

## Decisions Made

- Kept field-policy authority in exhaustive Rust matches; ordinary trace comparison never accepts raw JSON, string paths, broad approximate equality, or a globally sorted semantic sequence.
- Kept Phase-2 simulation time on exact bits while using synthetic values only to prove absolute, absolute-relative, and ULP threshold behavior.
- Derived one stable comparable request-contract hash from validated trace identity fields instead of widening the wire protocol or accepting an unvalidated request hash.
- Used evaluator-supplied logical elapsed time instead of a real clock, making deadline tests deterministic and keeping the reducer free of filesystem, process, randomness, and clock effects.
- Split protocol reduction into `scenario/reduction.rs` after the simplification pass so `scenario.rs` remains below the repository's file-size refactor trigger.

## Verification Evidence

- TDD RED was observed for every task: comparison imports failed before Task 1, report/signature methods failed before Task 2, and reducer/reparse imports failed before Task 3.
- `cargo test -p liquidfun-differential --test comparison -- --nocapture` passed all 11 policy/report tests.
- `cargo test -p liquidfun-differential --test comparison first_divergence -- --nocapture` passed the deterministic earliest-path test.
- `cargo test -p liquidfun-differential --test minimizer -- --nocapture` passed all 6 reduction tests.
- `cargo clippy -p liquidfun-differential --all-targets --all-features -- -D warnings`, package build, and all-feature package tests passed.
- Full workspace warning-denied Clippy, all-target/all-feature build, and all-feature tests passed, including 41 protocol unit tests, 8 fixture tests, 17 new differential tests, and all xtask integration suites.
- `cargo xtask package verify` passed and preserved the C++-free published consumer archive.
- Static acceptance scans found every typed report surface and collection branch, no comparator `serde_json::Value`, generic epsilon/approximation, or unstable sort, no reducer RNG/property/seed-only/unwrap pattern, and no reducer filesystem/process path.
- `git diff --check 133a744^..HEAD` passed; all changed production files remain below the 628-line refactor trigger.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exposed read-only validated trace fields**

- **Found during:** Task 1 (Implement exhaustive typed comparison and narrow canonicalization)
- **Issue:** `ValidatedTrace` and `CheckpointRecord` exposed only checkpoint slices/reset epoch, so the planned typed comparator could not inspect compatibility, provenance, semantic IDs, phases, exact bits, or world counts without serializing back to generic JSON.
- **Fix:** Added narrow documented accessors for already-validated trace and checkpoint fields; no raw mutation, parser, or storage detail was exposed.
- **Files modified:** `crates/liquidfun-test-protocol/src/trace.rs`
- **Verification:** Comparator tests prove harness-before-physics gating and exhaustive typed traversal; protocol and workspace suites pass.
- **Committed in:** `133a744`

**2. [Rule 3 - Blocking] Reduced the required harness error result size**

- **Found during:** Task 1 warning-denied Clippy verification
- **Issue:** The plan-required `Result<DifferentialOutcome, HarnessFailure>` failed `clippy::result_large_err` because inline bounded evidence made the error variant at least 168 bytes.
- **Fix:** Boxed the internal `HarnessFailureEvidence` payload while preserving the public error type, constructor, evidence accessor, equality, and clone behavior.
- **Files modified:** `crates/liquidfun-test-protocol/src/failure.rs`
- **Verification:** Warning-denied package/workspace Clippy and all protocol failure-evidence tests pass.
- **Committed in:** `133a744`

**3. [Rule 3 - Blocking] Added protocol-owned typed candidate revalidation**

- **Found during:** Task 3 (Minimize valid scenarios while preserving the failure signature)
- **Issue:** `ValidatedScenarioV1` intentionally kept commands/checkpoints private and exposed no construction seam for the planned reducer, so candidate deletion could not remain typed or prove revalidation before evaluation.
- **Fix:** Added focused range-removal methods that serialize in memory and reparse through the ordinary strict scenario validator, then split them into the existing `scenario/` child-module layout.
- **Files modified:** `crates/liquidfun-test-protocol/src/scenario.rs`, `crates/liquidfun-test-protocol/src/scenario/reduction.rs`
- **Verification:** Tests prove invalid references never reach the evaluator, canonical minimized bytes reparse to the same validated value, and all existing scenario/fixture tests pass.
- **Committed in:** `a76f815`

***

**Total deviations:** 3 auto-fixed (3 blocking implementation/verification gaps)
**Impact on plan:** Each change was the narrow protocol seam required to implement the specified typed comparator or reducer. No protocol breadth, physics behavior, process/file I/O, dependency, or artifact-promotion scope was added.

## Issues Encountered

- RED states were not committed because repository policy requires the full Rust gate to pass before every commit. Each failure was observed first, then each completed GREEN task was committed atomically after focused and repository-wide verification.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for `02-07-PLAN.md` and later adapter/supervisor/evidence plans to consume typed match/mismatch outcomes, machine reports, stable signatures, and canonical minimized scenarios.
- Phase-2 schema 1 has no entity definitions, so dependency-closed entity-group transforms correctly remain inapplicable until a later scenario-schema revision introduces typed entities.
- No comparison-policy, report-determinism, reduction-validity, package-isolation, or verification blocker remains.

## Self-Check: PASSED

- All eleven implementation/test files listed in this summary exist.
- Task commits `133a744`, `af81ba8`, and `a76f815` exist in repository history.
- Summary lifecycle metadata and all four requirement IDs match Plan 02-06.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_

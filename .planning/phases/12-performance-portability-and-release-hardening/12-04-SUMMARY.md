---
phase: 12-performance-portability-and-release-hardening
plan: "04"
subsystem: performance-evidence
tags: [performance-policy, workload-matrix, report-identity, json-schema, differential-testing]
requires: []
provides:
  - closed fourteen-workload performance vocabulary with complete cardinality sweeps
  - bounded five-run, 95-percent, three-percent-floor statistical policy
  - immutable catalog, scenario, toolchain, hardware, policy, and matrix report identities
  - byte-stable machine-readable policy schema and benchmark matrix
affects: [phase-12-performance-runner, phase-12-profiling, release-evidence, compatibility-reporting]
tech-stack:
  added: []
  patterns: [typed generated artifacts, validate-on-deserialize, timing-evidence isolation]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/performance.rs
    - crates/liquidfun-test-protocol/src/performance/matrix.rs
    - crates/liquidfun-test-protocol/src/performance/policy.rs
    - crates/liquidfun-test-protocol/src/performance/report.rs
    - crates/liquidfun-test-protocol/tests/performance_contract.rs
    - protocol/schemas/performance-policy-v1.schema.json
    - protocol/benchmarks/phase12-v1.json
  modified:
    - crates/liquidfun-test-protocol/src/lib.rs
key-decisions:
  - "Use the existing reviewed scenario catalog projection and resolved-scenario hashes as the sealed input identity for every performance case."
  - "Treat only interleaved unprofiled wall-clock samples under the five-run, 95-percent, max(3%, noise-floor) policy as regression authority."
  - "Forbid D1 compatibility promotion in the performance status vocabulary and keep durations outside semantic checkpoints."
patterns-established:
  - "Generated evidence artifacts: checked-in schema and matrix bytes are rendered from typed values and compared byte-for-byte."
  - "Immutable evidence identity: deserialization recomputes and verifies hashes instead of trusting reported identity fields."
requirements-completed: [PERF-01, PERF-02, PERF-04, PERF-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T19:05:38Z
duration: 14m
completed: 2026-07-23
---

# Phase 12 Plan 04: Performance Evidence Contract Summary

**A sealed fourteen-workload matrix, bounded paired-sampling policy, and hash-verified report identity now make performance evidence reproducible without allowing timing data to promote physics compatibility.**

## Performance

- **Duration:** 14m
- **Started:** 2026-07-23T18:51:19Z
- **Completed:** 2026-07-23T19:05:38Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Defined all fourteen mandatory workload tokens and exact 128/1024/8192 sweeps for broad phase, particle paths, mixed worlds, AABB queries, and ray casts.
- Bound every case to deterministic catalog and resolved-scenario hashes, fixed solver settings and horizons, scalar optimization, both engine roles, and explicit setup/warm-up/measured/teardown regions.
- Enforced five independent baseline runs, 95% confidence, a 3% practical floor, `max(3%, noise floor)` thresholds, interleaved Rust/C++ execution, and unprofiled wall-clock authority.
- Added bounded raw measurements, confidence intervals, hardware sessions, and hash-verified immutable report identities while rejecting D1 fixture promotion and checkpoint duration fields.
- Generated and locked the policy schema and complete workload matrix from the typed Rust authority.

## TDD Evidence

- **RED:** The new performance contract test failed because the performance API and tracked policy/matrix artifacts did not exist.
- **GREEN:** The typed policy, matrix, report, schema, and generated projection made all ten focused contract tests pass.
- **REFACTOR:** Deserialize boundaries now recompute matrix and report identities, preventing structurally valid but spoofed evidence from being accepted.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Seal the performance matrix and measurement policy** - `f5d5664` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/performance.rs` - Exposes the closed performance contract and stable error categories.
- `crates/liquidfun-test-protocol/src/performance/matrix.rs` - Defines workload, size, engine, settings, measured regions, sealed cases, and exact completeness validation.
- `crates/liquidfun-test-protocol/src/performance/policy.rs` - Validates sampling bounds and renders the deterministic policy schema.
- `crates/liquidfun-test-protocol/src/performance/report.rs` - Owns bounded raw evidence, intervals, hardware sessions, compatibility status, and hash-verified report identity.
- `crates/liquidfun-test-protocol/src/lib.rs` - Exports the performance contract.
- `crates/liquidfun-test-protocol/tests/performance_contract.rs` - Proves exact vocabulary, bounds, completeness, duration isolation, identity integrity, and byte stability.
- `protocol/schemas/performance-policy-v1.schema.json` - Tracks the generated strict policy schema.
- `protocol/benchmarks/phase12-v1.json` - Tracks the generated complete Phase 12 workload matrix.

## Decisions Made

- Reused the reviewed scenario catalog projection and benchmark-eligible resolved identities instead of inventing a parallel performance scenario namespace.
- Represented percentages as bounded integers and basis points so policy equality and generated JSON remain exact.
- Required decode-time identity recomputation for matrices and reports; reported hashes are evidence to verify, not authority to trust.
- Kept timing evidence explicitly outside D1 canonical physics promotion and proved semantic checkpoints reject duration fields.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Repository `target/` artifacts retain a macOS provenance issue from earlier work. All verification used one Cargo job and the clean temporary target `/tmp/liquidfun-phase12.OJRc0w`; no generated build artifact was committed.

## Known Stubs

None.

## Verification

- `cargo test -p liquidfun-test-protocol --test performance_contract` - 10 passed.
- `cargo clippy -p liquidfun-test-protocol --all-targets --all-features -- -D warnings` - passed.
- Exact ordered commit gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `git diff --check` and the plan-owned stub scan passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Downstream runner and profiler plans can consume one complete typed matrix without redefining workload or statistical semantics.
- Performance reporting can retain raw evidence and derived intervals without weakening the separate physics compatibility hierarchy.

## Self-Check: PASSED

- Confirmed the summary and every declared key file exist.
- Confirmed task commit `f5d5664` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

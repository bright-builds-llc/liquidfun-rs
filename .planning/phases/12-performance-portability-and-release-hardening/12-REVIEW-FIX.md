---
phase: 12-performance-portability-and-release-hardening
fixed_at: 2026-07-24T04:29:51Z
review_path: .planning/phases/12-performance-portability-and-release-hardening/12-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-07-24T04:29:51Z
**Source review:** `.planning/phases/12-performance-portability-and-release-hardening/12-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: Workload and size labels do not change the measured benchmark work

**Files modified:** `crates/liquidfun-benchmarks/src/paired.rs`, `crates/liquidfun-benchmarks/tests/performance_matrix.rs`, `crates/liquidfun-differential/src/performance/native.rs`, `crates/liquidfun-test-protocol/src/performance/matrix.rs`, `crates/liquidfun-test-protocol/tests/performance_contract.rs`, `protocol/benchmarks/phase12-v1.json`, `reference/performance/manifest.toml`, `tools/reference/src/benchmark_run.cpp`, `tools/reference/src/protocol.cpp`, `tools/xtask/src/performance.rs`, `tools/xtask/src/performance/analysis.rs`, `tools/xtask/tests/performance_cli.rs`
**Commit:** caf0a46
**Status:** fixed: requires human verification
**Applied fix:** Replaced misleading entity-size labels with executable workload-unit counts, bound each workload to a distinct reviewed scenario, repeated complete workloads inside both native and C++ measured intervals, and sealed every row with a distinct execution hash. Added matrix tests for exact work-unit counts, distinct workload scenarios, distinct execution hashes, and execution-hash tampering.

### WR-02: Differential coverage is copied from the compatibility ledger instead of measured

**Files modified:** `scripts/phase12-coverage.sh`, `tools/xtask/src/safety_evidence.rs`, `tools/xtask/src/safety_evidence/contract.rs`, `tools/xtask/tests/safety_evidence_contract.rs`
**Commit:** da690e6
**Status:** fixed: requires human verification
**Applied fix:** Mapped reviewed differential leaves to their executing integration targets, recorded leaves only after the target passed, computed exercised and missed sets with a typed validator, and failed closed on misses. Added a negative test that omits `subsystem.particle-contacts`.

### WR-03: Release aggregation synthesizes clean claims without validating producer results

**Files modified:** `.github/workflows/performance.yml`, `scripts/phase12-release-evidence.sh`, `tools/xtask/tests/release_cli.rs`
**Commit:** 28c239a
**Status:** fixed: requires human verification
**Applied fix:** Added inner producer-payload validation for platform verification, safety summaries, sanitizer results, coverage artifacts, regression completion, and performance manifests. Release claims now derive from validated package hashes, compiler/tier values, finding counts, regression results, differential misses, and reviewed performance metadata. Added mutation tests for platform hash/tier, sanitizer findings, coverage misses, and failed regressions.

### WR-04: Performance matrix deserialization accepts tampered case bindings

**Files modified:** `crates/liquidfun-test-protocol/src/performance/matrix.rs`, `crates/liquidfun-test-protocol/tests/performance_contract.rs`
**Commit:** 8581e9b
**Status:** fixed: requires human verification
**Applied fix:** Reconstructed every expected matrix row during validation and required exact equality. Added mutations for case ID, scenario ID, resolved hash, horizon, optimization mode, and every solver setting.

### WR-05: Linux benchmark reports contain placeholder hardware identity

**Files modified:** `tools/xtask/src/performance/runner.rs`
**Commit:** c96b29a
**Status:** fixed: requires human verification
**Applied fix:** Added fail-closed Linux, macOS, and Windows hardware collection with Linux parsing for `/proc/cpuinfo` and `/proc/meminfo`. Added tests that accept concrete Linux facts and reject architecture-only CPU and one-byte memory placeholders.

## Verification

- Exact ordered Rust pre-commit gate passed before every fix commit:
  `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `bash -n` and `shellcheck` passed for modified shell producers.
- `actionlint` passed for the modified performance workflow.
- Focused performance matrix, paired adapter, safety-evidence, release mutation,
  and hardware parser tests passed.
- The C++ oracle rebuilt and linked with the changed benchmark driver. Its
  combined protocol test binary reached an unrelated rigid-witness failure in
  the pre-existing dirty workspace.
- `git diff --check` passed.

***

_Fixed: 2026-07-24T04:29:51Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_

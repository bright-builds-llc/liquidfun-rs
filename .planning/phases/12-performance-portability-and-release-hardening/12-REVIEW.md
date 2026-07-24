---
phase: 12-performance-portability-and-release-hardening
reviewed: 2026-07-24T03:50:07Z
depth: standard
files_reviewed: 133
files_reviewed_list:
  - .codex/tasks/todo.md
  - .github/workflows/ci.yml
  - .github/workflows/coverage.yml
  - .github/workflows/fuzz.yml
  - .github/workflows/performance.yml
  - .github/workflows/platform.yml
  - .github/workflows/regressions.yml
  - .github/workflows/release.yml
  - .github/workflows/safety.yml
  - BENCHMARKING.md
  - COMPATIBILITY.md
  - CONTRIBUTING.md
  - Cargo.lock
  - README.md
  - RELEASE.md
  - SAFETY.md
  - UPSTREAM-CORPUS.md
  - crates/liquidfun-benchmarks/benches/catalog.rs
  - crates/liquidfun-benchmarks/src/lib.rs
  - crates/liquidfun-benchmarks/src/paired.rs
  - crates/liquidfun-benchmarks/tests/performance_matrix.rs
  - crates/liquidfun-differential/src/performance.rs
  - crates/liquidfun-differential/src/performance/native.rs
  - crates/liquidfun-differential/src/performance/oracle.rs
  - crates/liquidfun-differential/src/performance/report.rs
  - crates/liquidfun-differential/src/performance/report/runner.rs
  - crates/liquidfun-differential/src/supervisor/catalog.rs
  - crates/liquidfun-differential/src/supervisor/catalog/benchmark.rs
  - crates/liquidfun-differential/tests/fixtures/fake_oracle.rs
  - crates/liquidfun-differential/tests/paired_performance.rs
  - crates/liquidfun-test-protocol/src/lib.rs
  - crates/liquidfun-test-protocol/src/performance.rs
  - crates/liquidfun-test-protocol/src/performance/matrix.rs
  - crates/liquidfun-test-protocol/src/performance/policy.rs
  - crates/liquidfun-test-protocol/src/performance/report.rs
  - crates/liquidfun-test-protocol/src/performance/wire.rs
  - crates/liquidfun-test-protocol/tests/performance_contract.rs
  - crates/liquidfun-test-protocol/tests/performance_wire.rs
  - crates/liquidfun-testbed/CAPABILITY.md
  - crates/liquidfun-testbed/Cargo.toml
  - crates/liquidfun-testbed/src/bin/interactive.rs
  - crates/liquidfun-testbed/src/capability/input.rs
  - crates/liquidfun-testbed/src/capability/render.rs
  - crates/liquidfun-testbed/src/capability/report.rs
  - crates/liquidfun-testbed/src/interactive.rs
  - crates/liquidfun-testbed/src/lib.rs
  - crates/liquidfun-testbed/src/renderer.rs
  - crates/liquidfun-testbed/src/renderer/image.rs
  - crates/liquidfun-testbed/src/ui/protocol_viewport.rs
  - crates/liquidfun-testbed/src/ui/viewport/draw.rs
  - crates/liquidfun-testbed/tests/capability.rs
  - crates/liquidfun-testbed/tests/comparison_lifecycle.rs
  - crates/liquidfun-testbed/tests/controller_ui.rs
  - crates/liquidfun-testbed/tests/interactive.rs
  - crates/liquidfun-testbed/tests/renderer_contract.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/observation.rs
  - crates/liquidfun/src/world/observation/profile.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/phase12_profiles.rs
  - crates/liquidfun/tests/public_api_documentation.rs
  - deny.toml
  - fuzz/Cargo.toml
  - fuzz/corpus/README.md
  - fuzz/fuzz_targets/groups_ownership.rs
  - fuzz/fuzz_targets/particles.rs
  - fuzz/fuzz_targets/protocol.rs
  - fuzz/fuzz_targets/shapes_collision.rs
  - fuzz/fuzz_targets/world_mutation.rs
  - fuzz/src/lib.rs
  - justfile
  - protocol/benchmarks/phase12-v1.json
  - protocol/schemas/performance-policy-v1.schema.json
  - reference/compatibility.json
  - reference/coverage/contract.json
  - reference/performance/manifest.toml
  - reference/performance/policy.json
  - reference/platform/schema.json
  - reference/platform/support.json
  - reference/regressions/manifest.toml
  - reference/release/required-evidence.toml
  - reference/release/schema.json
  - rust-toolchain-nightly.toml
  - scripts/phase12-coverage.sh
  - scripts/phase12-miri.sh
  - scripts/phase12-performance.sh
  - scripts/phase12-platform.sh
  - scripts/phase12-regressions.sh
  - scripts/phase12-release-evidence.sh
  - scripts/phase12-rust-sanitizers.sh
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/benchmark_run.cpp
  - tools/reference/src/benchmark_run.hpp
  - tools/reference/src/catalog_run_session.cpp
  - tools/reference/src/catalog_run_session.hpp
  - tools/reference/src/main.cpp
  - tools/reference/src/protocol.cpp
  - tools/reference/src/protocol.hpp
  - tools/reference/tests/protocol_tests.cpp
  - tools/xtask/src/docs.rs
  - tools/xtask/src/inventory.rs
  - tools/xtask/src/inventory/report.rs
  - tools/xtask/src/inventory/validation.rs
  - tools/xtask/src/main.rs
  - tools/xtask/src/package.rs
  - tools/xtask/src/package/artifact.rs
  - tools/xtask/src/package/metadata.rs
  - tools/xtask/src/performance.rs
  - tools/xtask/src/performance/analysis.rs
  - tools/xtask/src/performance/evidence.rs
  - tools/xtask/src/performance/paths.rs
  - tools/xtask/src/performance/runner.rs
  - tools/xtask/src/release.rs
  - tools/xtask/src/release/attestation.rs
  - tools/xtask/src/release/domain.rs
  - tools/xtask/src/release/report.rs
  - tools/xtask/src/release/validation.rs
  - tools/xtask/src/safety_evidence.rs
  - tools/xtask/src/safety_evidence/contract.rs
  - tools/xtask/tests/docs_contract.rs
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/nightly_toolchain.rs
  - tools/xtask/tests/package_cli.rs
  - tools/xtask/tests/performance_cli.rs
  - tools/xtask/tests/performance_workflow.rs
  - tools/xtask/tests/platform_workflow.rs
  - tools/xtask/tests/regression_workflow.rs
  - tools/xtask/tests/release_attestation.rs
  - tools/xtask/tests/release_cli.rs
  - tools/xtask/tests/safety_evidence_contract.rs
findings:
  critical: 0
  warning: 5
  info: 0
  total: 5
status: issues_found
---

# Phase 12: Code Review Report

**Reviewed:** 2026-07-24T03:50:07Z
**Depth:** standard
**Files Reviewed:** 133
**Status:** issues_found

## Summary

The Phase 12 performance, portability, safety, coverage, and release-hardening
changes were reviewed under the repository's fail-closed evidence rules. The
implementation compiles cleanly and its static checks pass, but five
correctness issues can make performance or release evidence claim more than the
underlying execution proves. No security-critical issue was found.

## Verification

- `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-features` with an isolated target: all unit and integration
  suites completed successfully; the final `liquidfun` doctest process remained
  stuck in the known macOS `dyld` startup condition and was terminated.
- `actionlint`, `shellcheck`, `cargo deny --locked check`, scoped Markdown,
  JSON, and shell syntax checks: passed.
- Phase 12 docs, inventory, safety, release-constructor, Miri, sanitizer,
  coverage, and regression contract checks: passed.

## Warnings

### WR-01: Workload and size labels do not change the measured benchmark work

**File:** `crates/liquidfun-test-protocol/src/performance/matrix.rs:426-509`

**Issue:** `case_for` passes only the workload into `scenario_binding`; the
`size_point` changes `case_id` but not the resolved scenario, settings, or
logical horizon. Consequently, every 128/1024/8192 row for a scalable workload
executes the same resolved bytes. Several different workload kinds also share
one binding (for example, broad phase, narrow phase, contact solve, CCD, world
step, and mixed world all use `rigid-runtime-mutation`). The concrete runner
passes workload and size only as report/request identity while
`PreparedNativeBenchmark` executes the unchanged resolved scenario. The C++
adapter likewise records these fields without using them to construct work.
The matrix therefore reports cardinality scaling and subsystem workloads that
were not actually measured.

**Fix:** Make both dimensions part of the executable input:

```rust
fn scenario_binding(
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
) -> Result<ScenarioBinding, PerformanceError> {
    // Generate or select a scenario whose entity/operation count and measured
    // region are specific to this exact matrix row.
}
```

Seal the resulting resolved hash per row, use a workload-specific driver when a
catalog scenario cannot isolate the requested subsystem, and add tests that
assert the actual entity count and measured operation differ for each size and
workload row.

### WR-02: Differential coverage is copied from the compatibility ledger instead of measured

**File:** `scripts/phase12-coverage.sh:213-230`

**Issue:** After running the differential tests, the producer selects every
already-evidenced entry from `reference/compatibility.json`, writes all of them
to `exercised`, and hard-codes `missed: []`. No test or harness output is read
to establish which semantic leaves executed in this run. A skipped,
misconfigured, or silently narrowed differential suite can therefore still
publish complete differential coverage.

**Fix:** Instrument the differential harness to emit the stable leaf IDs it
actually exercised, then compute the set difference against the expected
ledger:

```bash
jq -n \
  --slurpfile expected expected-leaves.json \
  --slurpfile observed observed-leaves.json \
  '{exercised: $observed[0],
    missed: ($expected[0] - $observed[0]),
    parity_authority: false}'
```

Fail when required leaves are missed, and add a negative contract test that
omits one scenario and verifies that its leaf appears in `missed`.

### WR-03: Release aggregation synthesizes clean claims without validating producer results

**File:** `scripts/phase12-release-evidence.sh:434-557`

**Issue:** Producer validation checks candidate/run identity and, for several
artifacts, only a payload hash. It does not interpret the hashed result before
`append_independent_evidence` emits clean claims. In particular, platform
identities are not checked against the package archive hash, tier, scalar mode,
runner, workflow, or verification payload, yet the aggregator writes
`package_drift:false` and `evidence_tier:"d2_supported"`. It also hard-codes
zero differential gaps, zero safety/fuzz findings, zero regression misses, and
zero coverage misses rather than deriving those values from validated producer
payloads. The typed release validator can only validate the newly synthesized
claims, so it cannot detect that an upstream artifact reported a failure or
different package.

**Fix:** Parse every producer identity and payload against its typed/schema
contract, validate all provenance fields (including the exact package hash),
and derive each release claim from the validated producer value. Reject missing
or contradictory fields before calling `emit_evidence`. Add aggregation tests
that mutate a platform archive hash/tier, a safety finding count, a coverage
miss, and a regression result and verify that aggregation fails.

### WR-04: Performance matrix deserialization accepts tampered case bindings

**File:** `crates/liquidfun-test-protocol/src/performance/matrix.rs:288-408`

**Issue:** `PerformanceMatrix::deserialize` reconstructs the outer identity
through `Self::new`, but `validate_cases` checks only workload/size coverage,
catalog hash, nonzero horizon, complete regions, and engine roles. It does not
validate `case_id`, `scenario_id`, `resolved_sha256`, solver settings, or
`optimization_mode` against the reviewed binding for that workload and size.
A JSON matrix with one of those fields altered can therefore deserialize as a
valid reviewed matrix.

**Fix:** Reconstruct the expected row for every identity and require exact
equality:

```rust
for case in cases {
    let expected = case_for(case.workload, case.size_point, catalog_sha256.clone())?;
    if *case != expected {
        return Err(PerformanceError::new(
            PerformanceErrorKind::InvalidCaseBinding,
        ));
    }
}
```

Add mutation tests for every sealed field, including `case_id`, `scenario_id`,
resolved hash, each settings value, and optimization mode.

### WR-05: Linux benchmark reports contain placeholder hardware identity

**File:** `tools/xtask/src/performance/runner.rs:141-168`

**Issue:** `collect_hardware_session` uses the macOS-only `sysctl` keys
`machdep.cpu.brand_string` and `hw.memsize` on every platform. On the canonical
Linux performance runner both calls fall back, producing a CPU model equal to
the architecture string and `memory_bytes: 1`. These placeholders are accepted
as an immutable `HardwareSession` and included in the report identity, so the
raw report cannot identify or reproduce the hardware on which its timings were
collected.

**Fix:** Collect platform-specific hardware facts (`/proc/cpuinfo` and
`/proc/meminfo` or a reviewed portable API on Linux, `sysctl` on macOS), or
require the workflow to pass a validated controlled-host identity into the
runner. Fail closed instead of substituting plausible-but-false values. Add a
Linux test that rejects an architecture-only CPU model and the one-byte memory
fallback.

***

_Reviewed: 2026-07-24T03:50:07Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

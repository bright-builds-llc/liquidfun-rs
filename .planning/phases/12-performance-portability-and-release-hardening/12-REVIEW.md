---
phase: 12-performance-portability-and-release-hardening
reviewed: 2026-07-24T05:22:38Z
depth: standard
iteration: 3
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
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 12: Code Review Report

**Reviewed:** 2026-07-24T05:22:38Z
**Depth:** standard
**Iteration:** 3
**Files Reviewed:** 133
**Status:** issues_found

## Summary

Iteration 3 independently re-reviewed commits `bb73a4a`, `8878440`, `b76801d`,
`3b12075`, and `fc702ad`. Four iteration-2 warnings are closed. The differential
leaf implementation now records real run-produced markers and its omission
guard is correct, but the clean differential-coverage CI job cannot execute the
only test that emits two required math leaves. Phase 12 therefore still has one
actionable warning. No critical or security-impacting issue was found.

## Iteration-2 Finding Disposition

| Finding | Disposition | Evidence |
| --- | --- | --- |
| WR-01A | Closed | All 32 sealed matrix rows prepare from `PerformanceCase::scenario_id()`; the benchmark matrix target passes 8 tests. |
| WR-01B | Closed | Rust's 128-unit injected-clock proof and the C++ lifecycle observer place all unit construction before timer start. |
| WR-02 | Open | Markers are genuine, but the clean CI job omits the C++ prerequisite for the test that emits two of the 63 required leaves. |
| WR-03 | Closed | Typed canonical and safety payloads are validated; canonical-gap, safety-waiver, and tampered-log mutations fail closed. |
| WR-05R | Closed | The macOS-hosted scoped Clippy gate passes with `-D warnings`. |

## Verification

- `cargo fmt --all --check`: passed.
- Shell syntax and ShellCheck passed for all modified Phase 11/12 scripts.
- `git diff --check bb73a4a^..fc702ad`: passed.
- `cargo test -p liquidfun-benchmarks --test performance_matrix`: passed, 8
  tests.
- `ctest --test-dir target/reference/oracle-debug ...`: passed the C++ protocol
  test, including the 128-unit setup observer.
- `cargo test -p xtask --test release_cli release_constructor_`: passed, 10
  tests, including canonical-gap, safety-waiver, and tampered-log negatives.
- Scoped `cargo clippy` for the four affected Rust packages with all targets,
  all features, and `-D warnings`: passed.
- `scripts/phase12-coverage.sh check`: passed; its deliberate successful-target
  omission case reported the exact missing leaf.
- `scripts/phase12-coverage.sh differential fc702ad...`: passed with 63 observed
  leaves in the warm workspace, where both C++ oracle presets already existed.
  The clean-checkout defect below is masked by those untracked build artifacts
  and follows directly from the workflow and test control flow.

## Warnings

### WR-02R: Clean differential-coverage CI cannot emit the two required math leaves

**Files:** `.github/workflows/coverage.yml:150`,
`crates/liquidfun-differential/tests/round_trip.rs:180`

**Issue:** The differential-coverage job explicitly checks out with
`submodules: false`, installs only Rust, and immediately runs
`phase12-coverage.sh differential`; it never configures or builds either C++
oracle preset. In `cpp_math_probe_matches_operation_contract`,
`run_cpp_math_probe_twice` returns `None` when an oracle executable is absent,
and the test returns successfully at line 185 before emitting
`public-api.liquidfun-box2d-box2d-common-b2math-h` and
`subsystem.common-math-and-settings`. Those two IDs are part of the 63-leaf
expected set. A clean CI target therefore succeeds without these markers, after
which the typed leaf validator correctly reports two misses and prevents the
differential evidence artifact from being produced. A developer workspace with
pre-existing `target/reference/oracle-debug` and `oracle-release` binaries
masks the workflow defect.

**Fix:** Give the differential-coverage job the pinned upstream checkout and
exact C++ toolchain, then configure and build both oracle presets before running
the selected targets. Under
`LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY`, make the math test fail rather than
skip when either required oracle is absent. Add a workflow contract test that
proves every marker-emitting target's external prerequisites are constructed
inside the same clean job.

***

_Reviewed: 2026-07-24T05:22:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
_Iteration: 3_

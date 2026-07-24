---
phase: 12-performance-portability-and-release-hardening
verified: 2026-07-24T05:44:44Z
status: passed
score: 14/14 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T05:44:44Z
lifecycle_validated: true
overrides_applied: 0
residual_risks:
  - "No real run-bound release-candidate attestation is tracked yet. This is intentional: public status remains not release-ready, the release records are absent, and the audit rejects readiness until external producer evidence is supplied."
  - "Hosted Linux ARM64, macOS, Windows, controlled-performance, Miri, sanitizer, fuzz, and coverage lanes were verified through their fail-closed workflow and contract tests rather than re-executed on every external runner during this local verification."
---

# Phase 12: Performance, Portability, and Release Hardening Verification Report

**Phase Goal:** Turn the complete scalar engine into an auditable v1 release candidate with reproducible performance, supported-platform evidence, hardened safety/testing, complete documentation, and zero unexplained compatibility gaps.
**Verified:** 2026-07-24T05:44:44Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

Phase 12 achieved the auditable-release-candidate goal. The repository contains substantive, connected contracts and producers for package isolation, platform fanout, performance comparison, profiles, fuzzing, Miri, sanitizers, coverage, named regressions, documentation, compatibility closure, and release aggregation. The current checkout does **not** claim that a release has passed: the three run-bound release records are intentionally absent, public documents say the project is not release-ready, and the attestation command fails closed when those records are missing.

This distinction is material. The phase delivered and verified the release machinery and truthful public projection; it did not manufacture external platform, controlled-host, or release-run evidence.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The ordinary published artifact is one native-Rust `liquidfun` crate, isolated from renderer, oracle, and repository-only tooling. | ✓ VERIFIED | Package metadata and archive validators enforce one publishable default member, exact features/dependencies, notices/licenses, and forbidden content. `package_cli` passed 22/22 tests. |
| 2 | Development, nightly, and MSRV identities are exact, and the same content-addressed package is fanned out through closed platform tiers. | ✓ VERIFIED | `rust-toolchain.toml` pins 1.97.0, `rust-toolchain-nightly.toml` pins `nightly-2026-07-15`, workspace MSRV is 1.92, and `platform_workflow` passed 6/6 tests. Four durable targets are D2-supported; Intel macOS explicitly downgrades to unsupported without fresh native evidence. |
| 3 | The performance matrix covers every required subsystem with sealed scenario, work, solver, and statistical identities. | ✓ VERIFIED | `PerformanceWorkloadKind::ALL` contains 14 workloads and constructs 32 reviewed cases. `performance_contract` passed 13/13 and `performance_matrix` passed 8/8 tests. |
| 4 | Rust and C++ performance use independently prepared but semantically equivalent scenarios, interleaved samples, raw timing records, and distinct harness/physics/performance outcomes. | ✓ VERIFIED | Native and supervised-oracle adapters are wired through the paired runner; `performance_wire` passed 6/6 and `paired_performance` passed 10/10 tests. |
| 5 | Profiles are diagnostic-only, use a stable phase vocabulary, and cannot replace unprofiled timing or the scalar deterministic compatibility authority. | ✓ VERIFIED | Profile scopes are storage-neutral, duration fields remain outside semantic contracts, and `phase12_profiles` passed 5/5 tests. |
| 6 | Structural optimization admission is fail-closed on profile/bottleneck evidence, complete workload intervals, scalar build mode, and differential/determinism/safety/API hashes. | ✓ VERIFIED | `OptimizationDecision` rejects incomplete identity, disallowed modes, missing evidence, incomplete workload coverage, regressions, and statistically insignificant improvements. Performance validation and workflow tests passed. |
| 7 | Bounded fuzz targets cover protocol, shapes/collision, world mutation, particles, and group ownership, with typed handoff to regression evidence. | ✓ VERIFIED | Five explicit fuzz targets exist under `fuzz/fuzz_targets`; the shared dated nightly and fuzz workflow contracts reject floating identities and unbounded promotion. |
| 8 | Miri, Rust sanitizers, C++ sanitizers, Rust/C++ coverage, and differential leaf coverage are separate evidence authorities and fail closed on omissions. | ✓ VERIFIED | `phase12-miri.sh check`, `phase12-coverage.sh check`, and 14/14 safety-evidence tests passed. Coverage is explicitly `parity_authority=false`. The repaired clean-CI contract passed 4/4 tests and requires both math oracles before differential coverage. |
| 9 | Every corrected differential mismatch must be minimized, named, provenance-bound, executed, and validated before candidate evidence is accepted. | ✓ VERIFIED | The regression manifest has a closed 13-field schema and currently zero reviewed entries because no real mismatch is pending. Check mode passes; result/run contracts reject missing, duplicate, unregistered, mixed-candidate, or misplaced records. Regression workflow tests passed 8/8. |
| 10 | Public Rust APIs are documented and production Rust remains safe. | ✓ VERIFIED | `liquidfun` has `#![forbid(unsafe_code)]`; source scanning and `public_api_documentation` passed 5/5 tests. `RUSTDOCFLAGS='-D warnings' cargo doc -p liquidfun --all-features --no-deps` passed. |
| 11 | Public documents truthfully describe maturity, commands, compatibility, benchmarking, safety, contribution, packaging, and release evidence. | ✓ VERIFIED | `cargo xtask docs check` verified five Phase 12 document contracts; docs tests passed 38/38. README, COMPATIBILITY, BENCHMARKING, SAFETY, CONTRIBUTING, and RELEASE retain the non-ready status until attestation. |
| 12 | Compatibility and semantic-corpus closure have no unexplained or unaccounted items. | ✓ VERIFIED | `inventory check-report` verified 181 ledger rows; `inventory corpus check-closure` verified 388 terminal items with 0 unresolved. |
| 13 | The release audit is a pure, closed, commit-bound aggregation over all required evidence, and future attestation independently re-hashes its source, manifest, report, and tree. | ✓ VERIFIED | The release schema requires 19 typed records and rejects unknown/mixed/unreviewed evidence, weakened safety, compatibility gaps, corpus omissions, and authority promotion. Release CLI tests passed 24/24 and attestation tests passed 6/6. |
| 14 | Current public readiness is derived from accepted run-bound evidence and cannot be self-declared by local success. | ✓ VERIFIED | Only `required-evidence.toml` and `schema.json` are tracked under `reference/release`; source-candidate, candidate-manifest, and audit-report records are absent. A direct `release attestation validate-worktree` attempt correctly failed on the missing records. |

**Score:** 14/14 truths verified

### Roadmap Success Criteria

| # | Roadmap contract | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Publishable surface, toolchain/MSRV, supported platform fanout, and documented variation tiers | ✓ VERIFIED | One exact package artifact, D2-only platform script, four durable native targets, conditional Intel macOS downgrade, exact Rust pins, and fail-closed workflow tests. |
| 2 | Comparable Rust/C++ benchmarks, complete profiles, justified optimizations, scalar baseline | ✓ VERIFIED | Fourteen-workload/32-case matrix, paired native/oracle runner, raw interleaving, diagnostic profiles, controlled-host producer, and optimization admission rules. |
| 3 | Fuzz, Miri, sanitizers, coverage, minimized regressions, and failure isolation | ✓ VERIFIED | Five bounded fuzz targets, exact nightly, isolated safety lanes, five distinct coverage kinds, differential leaf closure, and named-regression producer. |
| 4 | Truthful rustdoc and public documentation | ✓ VERIFIED | Warning-free public rustdoc plus passing docs contracts; all public status files retain the absence of release attestation. |
| 5 | Closed release audit over compatibility, corpus, package, platform, performance, safety, and publication evidence | ✓ VERIFIED | 19-record schema, pure aggregator, independent attestation, 181-row compatibility report, and 388-item corpus closure. Audit readiness remains correctly unavailable without real producer records. |

## Required Artifacts

The plan-level artifact checker verified 48/52 declarations automatically. The four reported misses were stale literal-pattern expectations, not missing or stub artifacts:

- `performance_matrix.rs` contains the required `large_particle_system` workload; the test asserts the closed enum instead of repeating the snake-case token.
- `analysis.rs` implements the planned closed disposition as `OptimizationDecision`, with more explicit rejection variants.
- `COMPATIBILITY.md` uses “Unexplained rows” and “zero unexplained gaps” rather than the exact title-case phrase.
- `RELEASE.md` contains the required `release-candidate` workflow language with punctuation that differs from the literal matcher.

| Artifact group | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-test-protocol/src/performance/` | Closed matrix, policy, wire protocol, report identity | ✓ VERIFIED | Substantive typed implementations; all focused protocol tests pass. |
| `crates/liquidfun-benchmarks/` | Native reviewed-case matrix and measured-region driver | ✓ VERIFIED | All 32 cases are constructed from the sealed catalog and tested. |
| `crates/liquidfun-differential/src/performance/` and `tools/reference/` | Native/C++ execution with supervision and terminal classification | ✓ VERIFIED | Paired runner tests prove real adapters, restart behavior, bounded output, crash, provenance, sanitizer, and semantic failure paths. |
| `tools/xtask/src/performance/` and `scripts/phase12-performance.sh` | Calibration, controlled-host production, validation, optimization admission | ✓ VERIFIED | CLI, workflow, candidate/run identity, and artifact-last constraints are wired and tested. |
| `fuzz/` and `rust-toolchain-nightly.toml` | Bounded target set and reproducible nightly | ✓ VERIFIED | Five target binaries share the exact dated nightly contract. |
| `tools/xtask/src/safety_evidence/` and Phase 12 safety scripts | Closed coverage/regression/sanitizer/Miri evidence | ✓ VERIFIED | Contract checks and focused tests pass, including deliberate missing-leaf rejection. |
| `tools/xtask/src/package/`, `scripts/phase12-platform.sh`, `reference/platform/support.json` | Content-addressed package and platform policy | ✓ VERIFIED | Exact archive hash, feature/content isolation, D2 tier, durable targets, and conditional downgrade are enforced. |
| `tools/xtask/src/release/`, `reference/release/`, `scripts/phase12-release-evidence.sh` | Closed release registry, aggregator, and attestation | ✓ VERIFIED | Pure audit has no process/network producer capability; future attestation independently validates immutable records. |
| Public docs and `crates/liquidfun/src/lib.rs` | Complete rustdoc, safety model, truthful maturity and release guidance | ✓ VERIFIED | Rustdoc and document contract checks pass; unsafe code is forbidden. |
| `.github/workflows/coverage.yml` and `tools/xtask/tests/coverage_workflow.rs` | Clean-CI coverage fix | ✓ VERIFIED | Recursive oracle checkout and exact tool installation precede debug/release oracle builds; all four negative/positive workflow tests pass. |

## Key Link Verification

The plan-level checker verified 28/32 links automatically. The four reported misses were literal matching limitations and were manually traced:

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Performance matrix | Reviewed scenario catalog | `resolve_catalog` with sealed definition/settings | ✓ WIRED | Every reviewed case resolves canonical catalog bytes; tests compare resolved and content identities. |
| Benchmark cases | Native and C++ adapters | `run_paired_benchmark` | ✓ WIRED | Five independent runs alternate engine order and retain raw identities. |
| Native/C++ runner | Analysis and report identity | raw run deltas plus candidate/policy/environment hashes | ✓ WIRED | Validation rejects missing controlled-host or candidate identity and only writes identity last. |
| Platform workflow | Package verifier | `scripts/phase12-platform.sh` → `cargo xtask package verify-artifact` | ✓ WIRED | The workflow calls the script for MSRV, native matrix, and conditional Intel macOS lanes. |
| Performance workflow | Performance producer | `scripts/phase12-performance.sh paired/calibrate/validate` | ✓ WIRED | All three invocations are present; workflow tests enforce order, identity, and upload cardinality. |
| Regression workflow | Regression producer | `scripts/phase12-regressions.sh run` | ✓ WIRED | One candidate-scoped producer invocation precedes one artifact upload; tests reject ordering/cardinality mutations. |
| Coverage workflow | Differential math oracles | CMake debug/release builds before leaf producer | ✓ WIRED | Review fix is present and 4/4 clean-workflow tests pass. |
| Evidence producers | Release constructor | exact candidate-scoped paths and hashes | ✓ WIRED | Constructor is check-first, aggregate-only, and writes identity last. |
| Release constructor | Attestation validator | source/manifest/report/tree re-hashing | ✓ WIRED | Attestation tests reject missing, malformed, non-ready, mixed, and mutated records. |
| Attestation result | Public status | required semantic markers and non-ready fallback | ✓ WIRED | Docs contract rejects stale or unsupported maturity claims. |

## Data-Flow Trace (Level 4)

| Artifact | Data variable | Source | Produces real data | Status |
| --- | --- | --- | --- | --- |
| Performance matrix | Reviewed cases | `reviewed_scenario_catalog` + `resolve_catalog` | Yes — canonical scenario bytes and hashes | ✓ FLOWING |
| Native benchmark | Checkpoints and nanoseconds | Real native driver and injected clock boundary | Yes — setup outside and declared actions inside timing | ✓ FLOWING |
| C++ benchmark | Checkpoints, timing, terminal record | Supervised oracle executable/process protocol | Yes when producer runs; failures are typed and non-promotable | ✓ FLOWING |
| Platform evidence | Archive hash, candidate, target, toolchain, tier | One downloaded package artifact plus native job environment | Yes when hosted lane runs; missing conditional evidence downgrades support | ✓ FLOWING |
| Safety evidence | Rust/C++ coverage and differential leaves | Separate producer outputs | Yes when lanes run; missing/duplicate leaves fail validation | ✓ FLOWING |
| Regression evidence | Named result records | Reviewed manifest execution list | Yes for each registered mismatch; current empty manifest truthfully yields zero required runs | ✓ FLOWING |
| Compatibility report | Ledger rows and corpus closure | Inventory and terminal corpus authorities | Yes — 181 rows and 388 terminal items validated locally | ✓ FLOWING |
| Release report | 19 typed evidence records | Candidate-bound manifest payloads | Yes only after external producers; current absence is rejected rather than replaced by defaults | ✓ FLOWING |
| Public readiness | Attested source/manifest/report | Run-bound release attestation | No current attestation by design | ✓ FAIL-CLOSED |

## Behavioral Spot-Checks

All Cargo commands used `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and `CARGO_BUILD_JOBS=1`.

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Clean-CI coverage review fix | `cargo test -p xtask --test coverage_workflow` | 4 passed | ✓ PASS |
| Release/platform/performance/safety/docs workflow contracts | Focused 10-test-binary xtask command | 135 passed, 0 failed | ✓ PASS |
| Performance protocol and execution | Focused protocol, benchmark, differential, and library test command | 47 passed, 0 failed | ✓ PASS |
| Coverage contract | `scripts/phase12-coverage.sh check` | Passed; deliberate missing-leaf probe was rejected | ✓ PASS |
| Miri contract | `scripts/phase12-miri.sh check` | Exact nightly and seven subsets verified | ✓ PASS |
| Regression contract | `scripts/phase12-regressions.sh check` | Passed with 0 reviewed registrations | ✓ PASS |
| Docs | `cargo xtask docs check` | 5 Phase 12 public document contracts verified | ✓ PASS |
| Compatibility ledger | `cargo xtask inventory check-report` | 181 rows verified | ✓ PASS |
| Corpus closure | `cargo xtask inventory corpus check-closure` | 388 items, 0 unresolved | ✓ PASS |
| Performance policy | `cargo xtask performance validate` | Sealed policy/evidence surfaces passed | ✓ PASS |
| Safety validators | `validate-regressions` and `validate-coverage` | Passed; coverage has no parity authority | ✓ PASS |
| Formatting and lint | `cargo fmt --all --check`; workspace/all-target/all-feature Clippy with `-D warnings` | Passed | ✓ PASS |
| Public rustdoc | `RUSTDOCFLAGS='-D warnings' cargo doc -p liquidfun --all-features --no-deps` | Generated successfully | ✓ PASS |
| Workflow syntax | `actionlint` on six Phase 12 workflows | Passed with no diagnostics | ✓ PASS |
| Dependency policy | `cargo deny --locked check` | Advisories, bans, licenses, and sources passed; duplicate-version diagnostics are warnings | ✓ PASS |
| Absent release attestation | `cargo xtask release attestation validate-worktree ...` | Correctly rejected missing source record | ✓ PASS |

The full ordered repository quality gate had already passed immediately before this independent verification. The verifier reran the Phase 12-focused contracts plus workspace Clippy, formatting, rustdoc, actionlint, and dependency policy rather than duplicating every expensive hosted producer.

## Requirements Coverage

| Requirement | Source plans | Status | Evidence |
| --- | --- | --- | --- |
| FND-06 | 11, 12, 17 | ✓ SATISFIED | Exact dev/nightly/MSRV pins; package and platform contracts. |
| COMP-10 | 14, 15, 16, 21, 22 | ✓ SATISFIED | 181-row ledger, 388-item corpus closure, and audit rejection of gaps. |
| API-11 | 13 | ✓ SATISFIED | Warning-free public rustdoc and API documentation tests. |
| API-12 | 13 | ✓ SATISFIED | Production crate forbids unsafe code; source audit test passes. |
| TEST-05 | 09, 17 | ✓ SATISFIED | Five bounded fuzz targets cover the required surfaces. |
| TEST-06 | 10, 17, 24 | ✓ SATISFIED | Exact nightly plus isolated Miri/Rust/C++ sanitizer contracts. |
| TEST-07 | 09, 10, 25 | ✓ SATISFIED | Closed minimized-regression schema, execution list, result validator, and producer. |
| TEST-08 | 10, 24, review fix | ✓ SATISFIED | Five distinct coverage authorities and clean-CI differential leaf enforcement. |
| PERF-01 | 04, 06, 19 | ✓ SATISFIED | Fourteen required workload kinds and 32 reviewed cases. |
| PERF-02 | 04, 06, 07, 19, 20, 23 | ✓ SATISFIED | Equivalent scenarios, raw interleaving, exact environment/policy identities, controlled-host producer. |
| PERF-03 | 05 | ✓ SATISFIED | Stable public phase profile vocabulary without storage exposure. |
| PERF-04 | 08 | ✓ SATISFIED | Optimization admission requires profile and complete correctness evidence. |
| PERF-05 | 04, 05, 08 | ✓ SATISFIED | Strict scalar mode is the only compatibility baseline; SIMD/parallel/fast-math are rejected. |
| PERF-06 | 14, 23 | ✓ SATISFIED | BENCHMARKING methodology and run-bound claim records are enforced. |
| PLAT-01 | 11, 12 | ✓ SATISFIED | Linux x86_64 durable D2 lane over the exact artifact. |
| PLAT-02 | 11, 12 | ✓ SATISFIED | Linux ARM64 durable D2 lane over the exact artifact. |
| PLAT-03 | 11, 12 | ✓ SATISFIED | macOS ARM64 durable D2 lane over the exact artifact. |
| PLAT-04 | 11, 12 | ✓ SATISFIED | Intel macOS is conditional and explicitly unsupported without fresh native evidence. |
| PLAT-05 | 11, 12 | ✓ SATISFIED | Windows x86_64 durable D2 lane over the exact artifact. |
| PLAT-06 | 11, 12, 15 | ✓ SATISFIED | Tier/compiler/scalar/tolerance identities are closed and reviewed. |
| DOCS-01 | 13, 22 | ✓ SATISFIED | README maturity and command contract passes. |
| DOCS-04 | 14, 22 | ✓ SATISFIED | Compatibility report exposes closure and truthful non-ready status. |
| DOCS-06 | 14, 23 | ✓ SATISFIED | BENCHMARKING covers workloads, environment, profiles, interpretation, and claims. |
| DOCS-07 | 13 | ✓ SATISFIED | SAFETY covers identity, invalidation, callbacks, ownership, and zero unsafe blocks. |
| DOCS-08 | 13, 21 | ✓ SATISFIED | Contribution/release docs cover gates, provenance, SemVer, MSRV, generated evidence, and publication. |
| DOCS-09 | 15, 16, 21, 22 | ✓ SATISFIED | Closed 19-record audit verifies all required domains and cannot report ready without attestation. |

No Phase 12 requirement mapped in `REQUIREMENTS.md` is orphaned from the 25 plans.

## Anti-Patterns Found

| File | Line/pattern | Severity | Impact |
| --- | --- | --- | --- |
| Phase 12 implementation surface | TODO/FIXME/placeholder/empty-handler scan | ℹ️ None | No user-visible or release-critical stub was found. Temporary-path matches are legitimate bounded producer implementation. |
| `reference/performance/manifest.toml` | No reviewed reports | ℹ️ Expected | Real controlled-host results have not been generated; validation does not substitute synthetic claims. |
| `reference/regressions/manifest.toml` | `regressions = []` | ℹ️ Expected | There is no corrected mismatch awaiting registration. Check mode validates the empty reviewed set; run/result mode remains fail-closed. |
| `reference/release/` | Run-bound records absent | ℹ️ Expected | Prevents a release-ready claim. Direct attestation validation rejects the absence. |
| Plan artifact/key-link literal checks | 8 string-pattern misses | ℹ️ Tool limitation | Each was manually traced to substantive code or script indirection and covered by behavioral tests. |

### Disconfirmation Checks

- **Partial implementation check:** The apparent empty manifests were traced through their validators and consumers. They are explicitly non-ready states, not default-success paths.
- **Misleading test check:** Structural `check` modes alone cannot prove a hosted producer ran. Release aggregation therefore requires run ID, full candidate SHA, producer workflow/job identity, payload hashes, and future attestation; local success cannot promote evidence.
- **Error-path check:** Missing oracle executables, differential leaves, candidate identities, regression results, package hashes, evidence records, and release records all have focused negative tests. The review-fix tests specifically prove clean coverage cannot skip either math oracle.

## Human Verification Required

None. This phase is headless infrastructure, contracts, documentation, and evidence aggregation; the observable goal is programmatically verifiable. Actual release readiness remains a future external-run outcome and is correctly represented as unavailable, not as a pending human judgment about this phase.

## Gaps Summary

No goal-blocking gaps were found. Phase 12 provides an auditable, fail-closed release-candidate system and accurately refuses to call the current checkout release-ready without real producer evidence and run-bound attestation.

***

_Verified: 2026-07-24T05:44:44Z_
_Verifier: the agent (gsd-verifier)_

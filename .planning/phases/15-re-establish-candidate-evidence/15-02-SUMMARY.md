---
phase: 15-re-establish-candidate-evidence
plan: "02"
subsystem: testing
tags: [miri, sanitizers, coverage, provenance, ci]
requires:
  - phase: 15-01
    provides: Canonical compiler provisioning and four-preset build proof
provides:
  - Independently validated complete Miri and Rust sanitizer prerequisite evidence
  - Stable Rust coverage, C++ coverage, and complete differential leaf evidence
  - Fail-closed producer controls and instrumented real CLI regression coverage
affects: [15-03, 15-07, 15-08, candidate-requalification]
tech-stack:
  added: []
  patterns: [explicit interpreter modes, identity-last evidence, Cargo-owned binary discovery]
key-files:
  created:
    - crates/liquidfun-test-protocol/tests/fixtures/bytes.rs
    - crates/liquidfun/src/particle/storage/properties/group_model/configuration.rs
    - tools/xtask/tests/safety_evidence_contract/miri.rs
    - tools/xtask/tests/coverage_workflow/rust_identity.rs
  modified:
    - scripts/phase12-miri.sh
    - scripts/phase12-coverage.sh
    - .github/workflows/safety.yml
    - .github/workflows/coverage.yml
    - scripts/phase13-1-validate-gap-evidence.sh
    - scripts/phase13-1-gap-verification-manifest.json
    - tools/xtask/tests/differential_cli.rs
    - reference/coverage/contract.json
    - SAFETY.md
key-decisions:
  - Keep default Miri for 44 math tests and run only the exact endpoint with no-extra-rounding-error.
  - Preserve isolation, UB checks, fixed property seeds, 128 cases, and shrinking; adapt test fixture storage and persistence.
  - Bind Rust coverage to actual Rust 1.97.0 while retaining separate pinned-nightly sanitizer and nested fuzz-build boundaries.
  - Preserve distinct prerequisite source identities; final frozen-candidate acceptance remains separate.
requirements-completed: []
requirements-supported: [PLAT-01, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T19:27:01Z"
duration: approximately 110min
completed: 2026-09-14
---

# Phase 15 Plan 02: Truthful Safety and Coverage Summary

**Actual Linux producers passed 135 Miri tests, 46 sanitizer tests, separate Rust/C++ coverage, and all 63 required differential leaves.**

## Completion and Source Identities

Both plan tasks are complete. This is prerequisite verification, not final frozen-C acceptance or release readiness. Global PLAT-01, PLAT-05, and DOCS-09 statuses remain pending under the parent orchestrator.

| Proof | Exact source | Actual run | Independently checked result |
| --- | --- | --- | --- |
| Safety | `502cbf138a5479d52153f7367734896cf1c2602a` | [34880890457](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34880890457) | 135 Miri tests across eight cases; 46 Rust ASan tests across six cases; zero ignored; 16 summary/log hashes verified |
| Coverage | `228512bfa296dffaf2c8130bf3bcd2a01078ed04` | [34884682346](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34884682346) | All three jobs passed; Rust LCOV 9,585,834 bytes on rust-1.97.0; C++ LCOV 1,580,005 bytes on clang-22.1.8; 63/63 differential leaves, none missed |

Parent validation records: `target/phase15-plan02/safety-attempt-03/independent-validation.json` and `coverage-attempt-03/independent-validation.json`. Source/run/job identities and payload hashes were checked independently. These two source SHAs are deliberately distinct and are not rebound to a later candidate.

Miri case counts were math 44, endpoint 1, arena 9, identity 13, collision 44, codec 11, permutation 9, and group model 4. The actual group run retained both fixed seeds and 128 generated cases; small diagnostic probes were not substituted. Default isolation and UB checks remain enabled. Only the exact endpoint has `-Zmiri-no-extra-rounding-error`. Recorded bounds are 900 seconds normally and 3,600 seconds for the complete group workload, inside the unchanged 120-minute workflow limit.

## Task Commits

1. `194f669` — Separate Miri math modes, provenance/count checks, and fail-closed scanning.
1. `34242ff` — Execute Rust coverage with its declared stable compiler and per-kind identity checks.
1. `17ba98e` — Exercise actual absent-scanner and lost-endpoint failure controls.
1. `7da6c51` — Build and require the differential protocol-test executable.
1. `202119c` — Preserve Miri isolation with embedded codec bytes and in-memory property persistence.
1. `91860a2` — Isolate fake coverage tools from enclosing instrumentation; retain override rejection.
1. `502cbf1` — Record a separate bounded group-model execution budget.
1. `228512b` — Resolve real CLI siblings through Cargo, complete coverage prerequisites, and align current verification inventories.

These changes touched 26 paths including the amended plan. STATE.md, ROADMAP.md, and REQUIREMENTS.md remain parent-owned.

## Verification and Review

- Final actual local cargo-llvm-cov 0.8.7 / Rust 1.97.0 execution passed **534 tests across 30 xtask test executables**, including real CLI acceptance/rejection, every integration target, and the real pinned-nightly fuzz-build smoke. An isolated target with no default-debug fake-oracle binary prevented cache masking. `real-cli-attempt01/full-xtask-final-verified.log` and `xtask-final.lcov` retain the result (LCOV 1,033,323 bytes).
- Focused controls cover absent/erroring scanners, lost endpoint invocation, zero counts, incorrect modes/bounds, compiler mismatch/override, failed coverage commands, missing native prerequisites, and incomplete/tampered/path-substituted canonical inventories.
- Native and pinned-nightly adapter tests passed without assuming every build reports stable Rust. Native codec tests additionally compare embedded input with actual filesystem bytes.
- Ordered format, Clippy, build, and test gates passed, followed by explicit xtask Clippy, Bright Builds with zero findings, Markdown, shell/workflow checks, and diff checks. Logs remain under `target/phase15-plan02/`.
- `independent-code-review.md` records a clean independent review and corrective follow-up with no actionable introduced bugs or evidence weakening. Its 533-test observation preceded the final additional raw-path negative; the final instrumented run above passed 534.

## Deviations and Preserved Failures

1. **Required runtime prerequisites:** Actual execution exposed codec filesystem reads, proptest file persistence, missing differential protocol-test builds, and missing pinned fuzz tools. Test-only adaptations preserve original bytes, native filesystem/persistence checks, sampling, and actual CLI execution. No isolation disablement, test skipping, fixture-bit blessing, or production math change occurred.
1. **Coverage environment and binary paths:** Instrumented tests inherited compiler wrappers and an obsolete default-target assumption. Only controlled fake-tool subprocess environments were isolated; real CLI binaries now resolve beside Cargo's actual xtask executable. Production compiler-override rejection remains intact.
1. **Current canonical closure:** Plan 15-01 expanded the producer to 21 commands/four presets. Current validation now requires that exact sequence and four compile inventories, with omission, tampering, and source-path substitution rejected. The new compiler test is registered among 28 selected integration targets; existing command/log IDs were preserved. Historical manifests and retained evidence were not rewritten.
1. **Measured execution budget:** The earlier local working-source group run timed out at 900 seconds (exit 124). Its reconstructed source and result are explicitly diagnostic in `group-model-attempt01/`; neither that attempt nor reduced probes count as final-source proof. The successful actual safety run supplies the full workload evidence.
1. **Precommit process error:** `7da6c51` was created before the executor noticed the checker had rejected an uncommitted group-model file at 648 lines. The new configuration/test was moved into a cohesive module, all gates were rerun fail-fast, and the issue was repaired before publication. The failed checker record and commit history remain intact; no history rewrite or claim that the failed check passed was made.
1. **RED evidence without failing commits:** Repository precommit requirements took precedence over RED-stage commits. Expected failures and subsequent corrections were retained in attempt logs.

Failed actual safety runs `34876830545` and `34878648568`, failed coverage runs `34877402506` and `34879818035`, and local diagnostic/control failures remain separate from successful attempts. Standing authorization in AGENTS.md dated 2026-09-13 supplied routine correction and publication authority; no new approval was fabricated.

## Remaining Phase Work

No plan stubs remain. Relevant local instructions, Bright Builds standards, active lessons, and Phase 15 decisions governed execution. Simplification retained existing producer boundaries and Cargo-provided paths rather than adding alternate runtimes or broad exceptions.

Final candidate selection, all same-C producer evidence, canonical requalification, aggregation, and frozen-source attestation remain later phase work. Public readiness and global requirements were not advanced by this prerequisite plan.

## Self-Check: PASSED

The eight listed implementation commits resolve, the independently validated success records were read, and the created summary and referenced source files exist. Parent-owned planning state and global requirements were not staged or modified by this executor.

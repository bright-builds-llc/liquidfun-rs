---
phase: 15-re-establish-candidate-evidence
plan: "01"
subsystem: infra
tags: [clang, llvm, cmake, provenance, canonical-validation]
requires:
  - phase: 14-repair-windows-particle-group-invariants
    provides: Native source repair baseline awaiting canonical requalification
provides:
  - Shared immutable LLVM installer with fail-closed subprocess controls
  - Canonical Clang 22.1.8 configure/build proof for four presets
  - Out-of-tree execution of all 13 pinned upstream unit-test executables
affects: [15-02, 15-07, 15-08, canonical-evidence]
tech-stack:
  added: []
  patterns: [immutable installer inputs, identity-last provisioning, out-of-tree upstream tests]
key-files:
  created:
    - scripts/install-canonical-clang.sh
    - tools/reference/upstream_tests.cmake
    - tools/xtask/tests/canonical_toolchain_workflow.rs
    - tools/xtask/tests/canonical_toolchain_workflow/execution.rs
    - tools/xtask/tests/canonical_toolchain_workflow/fake-tool.sh
  modified:
    - .github/workflows/oracle.yml
    - .github/workflows/coverage.yml
    - .github/workflows/phase13-evidence-producer.yml
    - .github/workflows/phase13-acceptance.yml
    - .github/workflows/phase13-1-canonical-native.yml
    - tools/reference/CMakeLists.txt
    - tools/reference/CMakePresets.json
    - tools/reference/adapter-inputs.txt
    - tools/xtask/src/upstream.rs
    - tools/xtask/tests/upstream_cli.rs
    - tools/xtask/tests/phase13_1_canonical_native_workflow.rs
key-decisions:
  - Reuse reviewed immutable upstream llvm.sh and original SHA-256; reject changed compiler/tool identities after authenticated APT installation.
  - Build pinned upstream tests through a repository-owned wrapper to keep binaries outside the source checkout.
  - Keep the legacy GoogleTest address-only warning visible but nonfatal on its third-party target alone.
  - Preserve stale scoped provenance for Plan 15-07 requalification; native build proof is not final release acceptance.
patterns-established:
  - Fresh retained acquisition attempts publish identity only after compiler, tool, target and compile/link/run checks.
  - Five workflows share one acquisition helper; candidate validation retains four preset outcomes.
requirements-completed: [PLAT-01, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T17:38:00Z"
duration: 34min
completed: 2026-09-14
---

# Phase 15 Plan 01: Canonical Compiler Acquisition and Presets Summary

**Five workflows share verified immutable LLVM provisioning; canonical Clang 22.1.8 built all four presets and passed all 13 pinned upstream test executables.**

## Performance

- Approximately 34 minutes from first retained acquisition at 17:04 UTC through result recording; initial context loading preceded this interval.
- Tasks: 2/2 complete, including actual canonical execution and independent evidence checks.
- Plan-owned changed paths before this summary: 19.

## Task Commits

1. Task 1, installer and negative subprocess controls: `7f4d022`.
1. Task 2, five consumers and four-preset native matrix: `992eb66`.
1. Task 2 corrective native-build fix, legacy GoogleTest warning scope: `1739fc5`.

The parent then committed plan-link metadata as `49fe5e3`. The successful tested candidate is **`49fe5e36bc8327533b4035ccc191feec7ae4ea79`**, containing final source repair **`1739fc54d7a375c4b477cc2349dd628652925a62`**. Candidate tree: **`2ac2aa370e8dfbb32ed73d0a0171973d4f8bc428`**. Later summary metadata is not represented as the tested source.

## Actual Canonical Evidence

- [Successful native run 34874735386](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34874735386), attempt 1, job `canonical-native`, Ubuntu 24.04 x86_64, completed 2026-09-14 at 17:31:51 UTC.
- Exact tools: Rust 1.97.0, Clang/LLVM 22.1.8, CMake 4.3.3, Ninja 1.13.2; target `x86_64-pc-linux-gnu`.
- Upstream stayed at `7f20402173fd143a3988c921bc384459c6a858f2`; source verification ran before and after the matrix.
- `oracle-debug`, `oracle-release`, `oracle-asan-ubsan`, and `upstream-tests` each actually configured and built. Debug/release protocol CTests, rigid-world comparison, replay and deterministic repetition passed.
- Upstream CTest executed **13/13** executables successfully in 4.75 seconds.
- Retained attempt: `target/phase15-preflight/canonical-attempt-02/`; artifact: `artifacts/phase13-1-canonical-native-success-34874735386-49fe5e36bc8327533b4035ccc191feec7ae4ea79/`.
- Parent independent validation in `independent-validation.json` checked exact candidate/run, **21** ordered zero-exit commands, four presets, **29** compile/log hashes, upstream execution count and acquisition identity. Its explicit scope is Plan 15-01 compiler/build proof, not final release acceptance.

## Acquisition and Regression Proof

The helper downloads [the upstream installer at immutable commit eeed6742908255f0eeb12bb8e314366eff3c0a21](https://github.com/opencollab/llvm-jenkins.debian.net/blob/eeed6742908255f0eeb12bb8e314366eff3c0a21/llvm.sh), verifies SHA-256 `9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36` before execution, then validates compiler/tool/target identities and compile/link/run probes. It retains authenticated APT acquisition. This pins the installer input and enforces resulting compiler identity; it does not claim an immutable Debian package snapshot.

The mutable-URL reproduction produced `03878e08f47b66cc95bc4b544b0db3c6d9ce8d60e6cf2492ae357984330a9eae`, rejected against the original checksum without execution. Record: `target/phase15-preflight/installer-attempt-01/reproduction.json`.

Executable tests use fake downloader/compiler commands and a temporary helper copy with only fixture payload digest and installation prefix substituted. Production exposes no source/hash override. Real SHA-256 validation rejects substituted bytes. Controls cover download/install/compile failure, exact-version suffix mismatch, wrong LLVM version, wrong target, missing tool, symlinked attempt root and withheld success identity. Workflow contracts check every shared invocation and preserve delegated failure logging.

- Task 1 RED: six expected missing-helper failures in `installer-attempt-01/red.log`; GREEN: six passing tests in `green-final.log`.
- Task 2 RED: missing consumers/preset contract in `consumers-attempt-01/red.log`, and absent upstream target in `preset-red.log`.
- Final focused suite: **37 tests** across `canonical_toolchain_workflow`, `phase13_1_canonical_native_workflow` and `upstream_cli`; retained in `legacy-gtest-attempt-01/gate.log`.
- Each implementation/correction commit passed ordered `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, followed by managed checker and relevant shell/diff checks. Supplemental all-target xtask Clippy and Actionlint on all five workflows passed.
- Local Cargo gates used the supplied cached Linux ARM64 image/mounts; they remain supplemental to the actual canonical x86_64 result.

## Deviations from Plan

1. **[Rule 3 — Missing prerequisite] Implemented the absent upstream-tests preset.** Neither xtask nor CMake exposed it. Upstream CMake hardcodes binaries into the read-only source tree. Added an out-of-tree wrapper for pinned GoogleTest and the same 13 upstream test sources, a preset/default build target and CLI regression. The new CMake input participates in `adapter-inputs.txt`; source notices remain upstream and the wrapper points to the existing notice policy. Commit `992eb66`.
1. **[Rule 2 — Required test conventions] Added language-aware fixtures and extended native execution fixtures.** Parent approved this small scope refinement; Plan 15-01 records the paths. Cargo-only tests need no C++ build. Commits `7f4d022`, `992eb66`.
1. **[Rule 3 — Canonical Clang compatibility] Narrowed a legacy GoogleTest warning after actual failure.** [Run 34873723892](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34873723892) failed only at upstream-test build. In `gtest-death-test.cc:991`, `StackGrowsDown` passes an uninitialized local's address to a function which compares addresses and never reads the value. The sole demotion is `-Wno-error=uninitialized-const-pointer` on `liquidfun-upstream-gtest` for Clang 22 or later. The warning stays visible; other warnings remain errors, including repository-owned targets and upstream test executables. A scope contract protects the exception; source stays unchanged. Commit `1739fc5`.
1. **[Rule 2 — Instruction precedence] Retained RED evidence without failing commits.** Repository instructions require all precommit gates to pass, so RED logs were preserved and tests committed with GREEN implementation.

Initial failed native records remain in `target/phase15-preflight/canonical-attempt-01/`. Supplemental macOS Clang 22.1.0 reproduced the exact warning and passed the narrow demotion; all upstream test source syntax checks passed. These diagnostics did not replace the fresh successful 22.1.8 run.

## Decisions and Authority

AGENTS.md standing authorization dated 2026-09-13, the Bright Builds sidecar, local overrides, active lessons and relevant architecture/code-shape/testing/verification/Rust standards governed execution. The parent checked repository/ref/candidate/workflow identity, made ordinary main pushes and dispatched each attempt once. No invented approval, credential change, release publication, source tolerance change, action-pin change or acceptance waiver occurred.

D-01/D-02: source prerequisites fixed before final freeze; existing contracts reused. D-04/D-05: local gates then actual canonical checks with separate retained failures. D-06: canonical D1 remains distinct from local supplemental proof. D-08: standing authorization supplied effect authority. Later attestation/infrastructure/acceptance plans retain D-03/D-07/D-09 responsibilities.

Simplification retained one shared helper, one small upstream-test wrapper and existing failure capture. No stubs remain. Source reads/output paths stay within the planned tooling trust boundary; the new build input participates in the adapter digest.

## Deferred Issues and Next Plan Readiness

The compiler/four-preset prerequisite is complete. **Full candidate/release acceptance remains incomplete.** Requirement IDs above identify this plan's supporting scope and do not waive remaining phase-level PLAT-01/DOCS-09 evidence.

Push Oracle run **34874733912** correctly failed the Phase 9 scoped-materials guard after adapter/build inputs changed: expected `176:d30a6879fb37058ce92d179268b28983d4919169bf4dcb8f24e2c2a267017c71`, actual `176:83ecb7c6c30bf7aa223130348d6b25e6ca303a1092db89ca49308a37e22801f0`. Logs: `target/phase15-preflight/ci/34874733912/`. Plan 15-07 must obtain fresh reviewed canonical promotion/provenance; no historical hash or receipt was self-blessed here.

All 19 final evidence entries, controlled performance access and frozen-source attestation remain later work. STATE.md/ROADMAP.md updates belong to the parent orchestrator.

## Self-Check: PASSED

Created files exist and recorded commits resolve. Successful identity and parent independent validation were read from retained artifacts before recording the claims above.

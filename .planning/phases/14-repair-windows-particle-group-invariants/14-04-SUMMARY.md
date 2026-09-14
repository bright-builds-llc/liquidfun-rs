---
phase: 14-repair-windows-particle-group-invariants
plan: "04"
subsystem: ci
tags: [rust, windows, linux, macos, provenance, isolation]
requires:
  - phase: 14-03
    provides: Both literal Windows replays and complete public rollback contracts
provides:
  - Actual same-source successful Windows/Linux/macOS fixed and full-package execution
  - Complete successful Linux quality/isolation workflow
  - Retained failed and cancelled candidates with raw metadata and digests
affects: [15]
tech-stack:
  added: []
  patterns: [source-bound CI inspection, controlled provenance fixtures, single headless workspace suite]
key-files:
  created:
    - .planning/phases/14-repair-windows-particle-group-invariants/14-PLATFORM-EVIDENCE.md
    - tools/xtask/tests/phase13_acceptance_contract/history_fixture.rs
    - tools/xtask/tests/phase13_evidence_contract/materials.rs
  modified:
    - .github/workflows/ci.yml
    - tools/xtask/src/phase13_acceptance.rs
    - tools/xtask/tests/phase13_acceptance_contract.rs
    - tools/xtask/tests/phase13_evidence_contract.rs
    - tools/xtask/tests/phase9_witness_provenance.rs
    - tools/xtask/tests/phase13_1_gap_verification.rs
key-decisions:
  - Require actual same-SHA terminal results rather than rebinding earlier platform success.
  - Preserve production acceptance and canonical receipts; test contracts through controlled fixtures.
  - Keep Cargo-only tests independent of omitted native source and prior target artifacts.
  - Run the complete workspace headlessly once while retaining explicit testbed and focused gates.
requirements-completed: []
requirements-addressed: [PART-03, PART-04, PART-09, PART-10, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T07:18:00Z
duration: approximately 2h15m
completed: 2026-09-14
---

# Phase 14 Plan 04: Supported-platform proof Summary

**Both literal Windows regressions, all focused contracts and 957 default-package tests pass on Windows, Linux and macOS at one SHA; the complete Linux quality/isolation lane also passes.**

## Outcome

Two planned tasks complete. Tested source: `417d38dac6951226fb32ea99a97135defe88da7b`. [Cargo CI run 34815192937, attempt 1](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937) is completed/success with four successful jobs. The detailed record is [14-PLATFORM-EVIDENCE.md](14-PLATFORM-EVIDENCE.md).

| Job | Result |
| --- | --- |
| Windows `103884302536` | Inventory six; each exact seed one pass; focused mutation/property/creation 10/6/12; full default package 957 pass. |
| Linux `103884302645` | Same commands and counts, successful at the identical full source SHA. |
| macOS `103884302627` | Same commands and counts, successful at the identical full source SHA. |
| Linux quality `103884302389` | All 26 steps pass in 25m45s, including workspace, fuzz smoke, consumer isolation, corpus, explicit headless testbed, focused tooling, docs and inventory. |

The workspace command contains 160 passing harness blocks and 2,175 passing test executions, zero failures, and one pre-existing explicitly ignored fixture-regeneration authoring tool. No ignored test or skip was added. Platform fixed/focused/full-package results have zero ignored tests. Remote default-package 957 and local all-feature core 1,012 are separate feature selections, both including doctests; the evidence document gives the exact breakdown.

## Commits

1. Task 1 — `ce37c56`: `ci(14-04): expose exact particle regressions on supported platforms`.
1. Task 2 prerequisite correction, coordinated by root — `fc3af47`: `fix(ci): provide fuzz smoke prerequisites and verify structured deferral`.
1. Task 2 controlled fixture correction — `430e0ad`: `test(ci): isolate acceptance history and native material contracts`.
1. Task 2 scheduling correction — `417d38d`: `ci(14-04): run private suites headlessly without duplicate execution`.
1. Task 2 evidence — `17600fc`: `docs(14-04): record same-source platform and quality success`.

The SUMMARY metadata commit follows the tested source and is recorded in the final handoff. These documentation commits do not claim newer HEADs were tested. Coordinator-owned STATE, ROADMAP, requirements, TODO, review and security records are excluded from executor commits.

## Verification and retained attempts

The existing OS matrix now prints checkout/compiler identity, lists the property target, selects both exact named regressions and runs all three focused targets before the unchanged full package command. Pins, permissions, native-submodule isolation, fail-fast:false, cache conventions and mandatory quality gates remain intact.

All four candidates used verified ordinary main pushes and distinct per-source attempt directories under `target/phase14-platform/`. Run IDs, actual job metadata, raw logs and SHA-256 inventories are retained. The final platform inspector validates actual source/run/attempt, all step outcomes, compiler identity and exact counts; independent root/verifier checks reproduce those claims. Every final quality step succeeds; the complete run status is independently checked.

Local source commits passed ordered fmt, Clippy, build and all-feature core tests (1,012), plus actionlint and managed checks. Fixture corrections additionally pass workspace-wide Clippy and acceptance 31, evidence 24, Phase 9 provenance two with native source and existing target artifacts hidden. Later isolated tooling targets pass too. The scheduling correction passes all 22 package_cli tests, including the workflow/headless/isolation contract. Final managed scan: 939 files, zero findings. All touched Rust files remain below 629 lines.

## Deviations and recovery

1. **[Rule 3 — CI prerequisites]** First candidate `ce37c56` passes platforms but quality fails a stale prose assertion and missing cargo-fuzz. Root corrects the assertion to validate the structured sole Phase 15 deferral and installs existing pinned nightly/rust-src/cargo-fuzz prerequisites. The prior verified report is unchanged.
1. **[Rule 3 — test isolation]** Candidate `fc3af47` passes the prerequisites and 66 gap tests, but its live-history test fails on missing shallow history. A local full-history probe correctly reveals real current-HEAD Closure drift. Controlled Git fixtures replace the inappropriate live-HEAD acceptance assumption while retaining strict schema/ledger/ancestry/closure checks and explicit changed-closure rejection. Two further native-source couplings are identified statically and corrected with confined material fixtures, independent known-digest and mutation/missing-file/inventory controls. Production validation and canonical receipts remain unchanged. Ownership expansion was explicitly coordinated before edits and independently reviewed.
1. **[Rule 3 — redundant scheduling]** Candidate `430e0ad` passes full workspace, consumer isolation and corpus checks, then reaches the unchanged 30-minute cap during duplicated private tests. Timeout annotations and interrupted-test context are retained. The full workspace now runs with explicit headless environment; only redundant four-package build/test commands are removed, preserving equivalent feature coverage, all test targets, explicit testbed/focused gates and the timeout. Final candidate completes successfully in 25m45s.
1. **[Rule 3 — local environment]** Extra preflight exposed local missing zip/jq and noexec tmpfs blocking a generated fake-Git executable. Disposable container setup and an executable empty target overlay resolve these; no repository assertion changes address those environment failures. Failed logs remain retained.

Detailed records: 14-CI-PREREQUISITE-REPAIR.md, 14-CI-ACCEPTANCE-FIXTURE-REPAIR.md, 14-CI-MATERIALS-REPAIR.md and 14-CI-SCHEDULING-REPAIR.md. Failed `ce37c56` and `fc3af47` runs and cancelled `430e0ad` are never labeled successful workflows. No blind rerun, acceptance waiver, fresh authorization token or force push occurred; AGENTS.md standing authorization supplied authority throughout.

## Remaining authority boundary

All results here are D2. Actual current-head canonical acceptance, release-candidate aggregation and attestation remain Phase 15 work. No fixture promotion, compatibility-status change or package release occurred. Root owns phase-level verification and requirement/state completion; this summary completes Plan 04 without independently declaring the entire phase complete.

## Self-Check: PASSED

All named source/fixture/evidence files exist and all four source commits resolve. Final run and job identities match the tested full SHA; platform log digests and counts and terminal quality success are checked. Scoped source review finds no production stubs, new runtime trust surfaces, or accidental canonical changes. Subsequent edits are documentation only.

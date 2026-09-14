---
phase: 14-repair-windows-particle-group-invariants
reviewed: 2026-09-14T07:18:58Z
initial_reviewed: 2026-09-14T05:12:50Z
depth: standard
diff_base: 5aadb6c105b98ae09443f74e44c57f8ce7eae19d
candidate: 417d38dac6951226fb32ea99a97135defe88da7b
previous_candidate: 430e0ad775d64322b04ae04504d5a6275a197dc3
files_reviewed: 32
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/liquidfun-differential/tests/round_trip/evidence.rs
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/particle/solver/preparation.rs
  - crates/liquidfun/src/particle/storage.rs
  - crates/liquidfun/src/particle/storage/group.rs
  - crates/liquidfun/src/particle/storage/group/tests.rs
  - crates/liquidfun/src/particle/storage/solver_state.rs
  - crates/liquidfun/src/particle/storage/solver_state/tests.rs
  - crates/liquidfun/src/particle/storage/transaction_test_support.rs
  - crates/liquidfun/src/world/object/tests.rs
  - crates/liquidfun/src/world/object/tests/particle_group_transactions.rs
  - crates/liquidfun/src/world/particle_coupling/executor.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/src/world/step/report.rs
  - crates/liquidfun/tests/particle_group_mutation.rs
  - crates/liquidfun/tests/particle_group_mutation/transactional_rejection.rs
  - crates/liquidfun/tests/particle_group_properties.proptest-regressions
  - crates/liquidfun/tests/particle_group_properties.rs
  - crates/liquidfun/tests/particle_group_properties/model.rs
  - crates/liquidfun/tests/particle_group_properties/snapshot.rs
  - crates/liquidfun/tests/particle_groups.rs
  - crates/liquidfun/tests/particle_groups/transaction_support.rs
  - crates/liquidfun/tests/particle_groups/transactional_rejection.rs
  - tools/xtask/src/phase13_acceptance.rs
  - tools/xtask/tests/performance_cli.rs
  - tools/xtask/tests/phase13_1_gap_verification.rs
  - tools/xtask/tests/phase13_acceptance_contract.rs
  - tools/xtask/tests/phase13_acceptance_contract/history_fixture.rs
  - tools/xtask/tests/phase13_evidence_contract.rs
  - tools/xtask/tests/phase13_evidence_contract/materials.rs
  - tools/xtask/tests/phase9_witness_provenance.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 14: Code Review Report

**Reviewed:** 2026-09-14T06:08:46Z (working-tree incremental update; earlier reviews retained below)
**Depth:** standard
**Files Reviewed:** 32
**Status:** clean

## Summary

The latest incremental review extends the prior 26-file result to the explicit 32-file scope in `target/phase14-review-scope-final.json`. Six newly changed Rust files were read in full and reviewed against their callers and unchanged production validators. The other 26 scoped files are byte-identical to `fc3af47291750af418befe98ad437dc40aa87bb3`. This result currently binds the reviewed working tree; a final candidate commit has not yet been supplied. No Critical, Warning or Info findings were found. This is a static code review, not CI or Phase 15 acceptance.

Initially reviewed every file in the explicit `target/phase14-review-scope.json` scope and the net changes from `5aadb6c105b98ae09443f74e44c57f8ce7eae19d` to `ce37c5632507df2be9baf61305967a57a68567d4`. The incremental review extends that result through `fc3af47291750af418befe98ad437dc40aa87bb3`, rereading both changed source files and adding `tools/xtask/tests/phase13_1_gap_verification.rs` to the exact scope. This explicit scope includes Plan 04 CI wiring and the adjacent private tooling test repairs; it takes precedence over SUMMARY-derived scope.

All reviewed files meet quality standards within this standard-depth scope. No issues found.

## Review Evidence

- **Scratch allocation:** `zeroed_lane` reserves actual rows after the existing declared-capacity and signed-index preflight. Lazy lane creation, zero initialization, append copying and permutation behavior remain intact. The bounded allocation regression distinguishes the sizing repair from error-mapper suppression.
- **Group candidate errors:** The new storage/lifetime mappers are used by unpublished group preparation. Only `InvalidLaneBundle` changes category there; the shared authoritative mapper and capacity/identity/handle categories retain their previous behavior. The existing clone-plan-commit path publishes neither shells nor diagnostic advances on preparation failure.
- **Reactive topology:** Storage and generation causes remain distinguishable through preparation. Only `ConstraintError::ZeroLengthPairDistance` maps to the new `StepError::InvalidParticleGroupTopology`; other causes still map to the existing lifecycle invariant boundary. The existing typed zero-rest rejection is retained, with corruption and other-generation-error controls. This adds no global step-transaction guarantee.
- **Rollback and lifecycle:** Private tests compare complete particle storage, lifetime state, both arenas, system metadata/order and diagnostic allocation. Rare state is seeded only in test support; populated contacts, topology, optional lanes, pending listener state and subsequent allocations make the comparisons meaningful. Public snapshots retain actual identities and topology/rest data; normalized fresh-world replay remains a separate comparison. Focused rejections verify deferred listener identity/cause and successful neighboring operations.
- **Historical replays and property outcomes:** Both literal Windows seeds, controls and expanded operations are present. The original generator/version, case count and bounds are unchanged. Both final appends must apply, retain the target and preserve member order while increasing the particle/member count. Unexpected errors fail explicitly, and accepted pending-delete creation errors require a nonzero pending-row witness.
- **CI and private tooling:** The existing supported-platform matrix adds source/compiler identity, inventories and focused/exact replay commands while retaining full package tests and Cargo-only checkout isolation. Differential evidence assertions survive extraction unchanged. Performance fixtures use fresh confined roots, copied checked-in policies and an injected provider; missing-oracle negative controls preserve production validation for both paired and check modes. No production tooling validation, native reference source or canonical fixture was changed.

## Verification and Limits

At initial review completion, HEAD was `ce37c5632507df2be9baf61305967a57a68567d4` and all 25 working-tree files were byte-identical to their candidate blobs. None of the initial scoped files was gitignored; no `.claudeignore` or project skill directories were present. The initial base-to-candidate `git diff --check` passed.

This reviewer performed static source/diff analysis and focused caller/error-boundary inspection. No Cargo commands were run, as requested to avoid contention with the coordinator's verification gate. Local test and Clippy outcomes recorded by Plans 01–03 and the coordinator were contextual evidence, not independently rerun results. Matching Windows, Linux and macOS CI acceptance remains Plan 04's responsibility; this report does not certify a whole CI run, complete Phase 14, promote compatibility evidence or close Phase 15.

The review was informed by `AGENTS.md` repo-local guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, the standards index and architecture/code-shape/verification/testing/Rust pages, both active lesson files, Phase 14 context and Plans 01–03 summaries. Both active lesson files fit the startup reading budgets and were read in full. No source files were modified and no commit was created.

## Incremental Review — 2026-09-14T05:31:28Z

Reviewed the complete `.github/workflows/ci.yml` and `tools/xtask/tests/phase13_1_gap_verification.rs` files, their exact `ce37c5632507df2be9baf61305967a57a68567d4..fc3af47291750af418befe98ad437dc40aa87bb3` diff, and `14-CI-PREREQUISITE-REPAIR.md`. These are the only two source changes in that interval; the third changed path is the repair record. Result: zero Critical, Warning or Info findings.

The workflow installs `nightly-2026-07-15` with `rust-src` and `cargo-fuzz` 0.13.2 with `--locked` before workspace tests. These match the existing fuzz workflow and the real `fuzz_build_keeps_tracked_lock_and_clean_tree` invocation. The smoke test remains active and continues to require successful compilation, a tracked fuzz lockfile and unchanged Git status. Native reference checkout isolation is retained.

The deferral assertion normalizes CRLF, bounds inspection to the opening frontmatter and its indented `deferred` section, then requires one truth entry, one exact Phase 15 destination and the existing Phase 13 acceptance truth. It removes dependency on obsolete body prose while preserving the current structured contract. The historical `13.1-VERIFICATION.md` is byte-unchanged across the interval. Physics, oracle fixtures and production validation are unchanged.

All 26 scoped working-tree source files were byte-identical to `fc3af47291750af418befe98ad437dc40aa87bb3`; the incremental diff passed `git diff --check`. The initial candidate's three default-platform jobs passed, but its quality job failed in run `34808501136`, according to the retained repair record. That failed candidate is not relabeled as accepted. The coordinator reports a passing targeted deferral test, workspace Clippy, 1,012 core tests, actionlint and managed checks for this repair; this reviewer did not rerun those commands. Fresh candidate CI, including the real fuzz-build smoke result, remains pending.

***

_Reviewed: 2026-09-14T05:31:28Z; initial review retained above_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

## Incremental Review — 2026-09-14T06:08:46Z

The six added scope paths are the acceptance facade, acceptance contract tests and their history fixture, evidence contract tests and their materials fixture, and Phase 9 witness provenance tests. All six files were read in full; the acceptance identity/closure implementation and witness materials hashing implementation were inspected to verify the boundaries exercised by the new tests. No scoped path is gitignored; no `.claudeignore` or project skill directory was present. The working-tree diff passed `git diff --check`.

- **Repository history:** The fixture creates its own temporary Git repository, synthetic producer/base/promotion commits, receipt, ledgers and evidence bytes. Its only production-facade addition is guarded by `cfg(test)`. Four focused tests call the unchanged `validate_repository_identity_at`: valid schema-v2 promotion and a metadata-only descendant must pass, while schema v1 and changed replay inputs must fail with their exact categories. No test accepts a closure error as success. This exercises real Git ancestry, promotion discovery, content identity and closure enforcement without asserting acceptance of an arbitrary live repository HEAD.
- **Scoped materials:** Synthetic materials cover all four byte-bearing kinds, an identity-only compile definition, exact count/digest, unlisted-file exclusion, ordering/deduplication, per-kind byte mutation and missing-file rejection. This reviewer independently recalculated the expected digest with Python `hashlib` and big-endian u64 length prefixes: `9fca1c0db33ce9124311acd1cbff697c4e8f255e2dc5f78eae9435684cc112d4`. The tracked-manifest test checks 176 distinct inventory entries and schema/target/preset without requiring native files in Cargo-only CI. It makes no assertion of canonical native content acceptance.
- **Provenance boundary:** The Phase 9 test removes only its incidental live native-tree lookup and count assertion. Existing target-derived fixture coverage still verifies scoped/unrelated changes, repository binding, undeclared target inputs, compiler dependencies and declared-only materials. Production provenance and evidence validation remain byte-unchanged.
- **Isolation and evidence:** No canonical receipt, artifact, source map, upstream pin, production acceptance validator body or physics implementation changes in this increment. The CI workflow is byte-identical to the prior candidate; the exploratory full-history checkout change is absent. Synthetic data is confined to owned test roots and explicitly labeled as synthetic. No acceptance criteria are waived and Phase 15 remains deferred.

The reviewer ran no Cargo commands, modified no source, and created no commit. Runtime checks and the fresh same-SHA complete CI run remain the executor's responsibility. Prior failed CI records retain their failed status; this review does not promote their evidence. Repo-local guidance, the Bright Builds sidecar and overrides, architecture/code-shape/verification/testing/Rust standards, GSD review workflow and both active lesson files informed the review. Active lessons totaled 11,548 bytes and 3,850 estimated tokens and were read completely.

## Committed Source Binding — 2026-09-14T06:15:52Z

The coordinator verified all 32 reviewed working-tree files byte-for-byte against `430e0ad775d64322b04ae04504d5a6275a197dc3` and checked the full base-to-candidate diff. The six fixture changes reviewed above are the committed changes; no production or canonical-evidence change was added. The static result remains clean, with zero findings. Executor runtime evidence is retained in the two CI repair records; final same-SHA CI acceptance remains pending.

## Scheduling Review and Source Binding — 2026-09-14T06:52:55Z

The coordinator and independent phase verifier reviewed the sole source diff from `430e0ad775d64322b04ae04504d5a6275a197dc3` to `417d38dac6951226fb32ea99a97135defe88da7b`: the full workspace test receives the four empty display variables, and redundant four-package build/test invocations are removed from the later headless step. All four packages are workspace members; the retained workspace build includes all targets and all features, and the retained workspace test includes all features. The only engine feature, `differential-internals`, was also enabled by the removed selection through its differential dependency. Explicit testbed commands and every focused, package, corpus, documentation and inventory gate remain. Timeout, action pins, concurrency, permissions and checkout isolation are unchanged.

The retained cancelled run proves duplicate work: its full workspace suite passed, then the repeated rigid-fixture suite passed 15/15 after 404.47 seconds before the job reached its 30-minute limit during later repeated tests. This correction removes duplicate execution without suppressing any failed assertion or test configuration. Actionlint, the 22-test package contract and ordered 1,012-test core gate passed per `14-CI-SCHEDULING-REPAIR.md`. All 32 scoped files match `417d38dac6951226fb32ea99a97135defe88da7b` byte-for-byte. Incremental result: zero findings; fresh same-SHA CI remains required.

## Final Verification Context — 2026-09-14T07:18:58Z

The coordinator independently checked terminal run `34815192937` at reviewed source `417d38dac6951226fb32ea99a97135defe88da7b`: all three platform jobs and all 26 quality steps succeeded. The phase verifier independently checked the same source, raw logs and test execution counts. The static review remains clean across 32 files. These final runtime results supplement the earlier static reviews without changing their historical scope or relabeling cancelled or failed candidates. Phase 15 canonical acceptance remains deferred.

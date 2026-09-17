---
phase: 15-re-establish-candidate-evidence
plan: "03"
subsystem: testing
tags: [regressions, provenance, particle-groups, ci]
requires:
  - phase: 15-02
    provides: Independently verified safety and coverage prerequisites
provides:
  - Two genuine hash-bound Phase 14 inputs consumed by exact native replay tests
  - Actual successful regression producer with independently verified artifacts
  - Nonzero exact-test enforcement and preserved bounded attempt diagnostics
affects: [15-07, 15-08, candidate-requalification]
tech-stack:
  added: []
  patterns: [strict input parsing, package-safe test mirror, identity-last artifacts]
key-files:
  created:
    - scenarios/regressions/phase14-particle-groups.txt
    - crates/liquidfun/tests/particle_group_properties/phase14-particle-groups.txt
    - crates/liquidfun/tests/particle_group_properties/registered.rs
    - tools/xtask/tests/regression_workflow/cargo.sh
    - tools/xtask/tests/regression_workflow/unix.rs
  modified:
    - reference/regressions/manifest.toml
    - crates/liquidfun/tests/particle_group_properties.rs
    - tools/xtask/src/safety_evidence/contract.rs
    - tools/xtask/tests/safety_evidence_contract.rs
    - scripts/phase12-regressions.sh
    - .github/workflows/regressions.yml
    - tools/xtask/tests/regression_workflow.rs
key-decisions:
  - Classify native invariant panics without inventing oracle or tolerance authority.
  - Preserve existing test names and select the integration target explicitly.
  - Keep strict success artifacts separate from retained diagnostics.
requirements-completed: []
requirements-supported: [PLAT-05, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T19:57:00Z"
duration: approximately 27min
completed: 2026-09-14
---

# Phase 15 Plan 03: Registered Native Regressions Summary

**Two preserved Windows regression inputs now execute exact native tests, with successful GitHub producer evidence and independently checked payload hashes.**

## Actual Completion Evidence

Both tasks are complete at tested source **`1c4e2565b1cad923477c48503d390a39cdd1a1cf`**. [Run 34889506405](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34889506405), attempt 1, job `104128339966` (`regressions`), completed successfully at 2026-09-14 19:53:37 UTC. The Ubuntu x86_64 producer used Rust 1.97.0 and executed both named tests with **one pass, zero failures and zero ignored tests each**.

Parent independent validation is retained at `target/phase15-plan03/remote-attempt-02/independent-validation.json`. It checked the actual run/source/conclusion, both exact integration-target/test commands, completion and identity hashes, registry/input hashes, diagnostic index and **10 diagnostic payload hashes**, and historical failure/fix commit existence and ancestry.

- Registry SHA-256: `8c5776e8d139ed49daec4179b105eaa5690e118a7ef6c4ce3b006bc4d8fc5ef6`.
- Shared input SHA-256: `172eecc6d4522d5cbb449c9147145241f20db995a06cc5435aeceeb734a8a6c0`.
- Success artifact: `phase12-regressions-1c4e2565b1cad923477c48503d390a39cdd1a1cf`.
- Diagnostic artifact: `phase12-regression-diagnostics-34889506405-1-1c4e2565b1cad923477c48503d390a39cdd1a1cf`.

This is Plan 15-03 prerequisite proof. Final frozen-C production, fresh final-candidate Windows evidence, release aggregation and readiness remain pending. No global requirement was completed.

## Commits and Behavior

1. `818a3bc` — Bind the two genuine inputs to native replay tests and add private `InvariantViolation` classification.
1. `34eeae4` — Require one executed exact pass, reject existing destinations, and retain command/log/compiler/run identities and checksums.
1. `a7b4850` — Remove an unobserved operation label from the newer historical failure signature.
1. `1c4e256` — Use runner-baseline `grep` and execute the actual workflow verification step in a no-`rg` test environment.

The registry preserves audited seed `4149329052036581951`, current seed `190752942043209832`, and their exact controls. Original sources are `4b6b15ba562cedcad3f37e379d01be9e9cb2e949` and `5aadb6c105b98ae09443f74e44c57f8ce7eae19d`; the shared scratch-sizing fix is `e58e028a31ffe845c5465e8d72dd5d933c9d80f5`. The audited diagnostic trace proves operation 14's allocation failure becoming `InvalidLaneBundle` and the impossible-state panic. The newer historical log records the panic but not first-operation reachability; its signature states that limit explicitly.

The strict parser rejects malformed, duplicate, missing and out-of-bounds records. Both replay tests consume parsed values while retaining literal seed/control/operation guards and actual append assertions. Shared input paths are permitted with exact byte hashes; IDs and named selections remain unique. Unknown classes and `PhysicsMismatch` without oracle/tolerance identities remain rejected.

## Verification and Preserved Failures

- Task 1 RED rejected the new invariant category; GREEN passed 19 contract tests and eight particle-group tests. Task 2 RED exposed zero-test success and destructive destination reuse. Final workflow suite passed **14 tests**, including exact YAML verification without `rg`, failed/zero/oversized test output, duplicate destinations, malformed provenance and real typed projection into shell replay. The release CLI suite passed 24 tests.
- Each commit passed fail-fast ordered `cargo fmt --all`, Clippy, build and tests, followed by the managed checker and applicable shell/workflow/diff checks. Explicit xtask Clippy also passed. Local gates used the reviewed Linux ARM64 image and remain supplemental.
- `cargo package -p liquidfun --allow-dirty` verified the ordinary package. The package already excludes integration tests. A separate disposable harness copied the exact test sources/mirror into the extracted package and explicitly registered the target; all eight tests passed against packaged source. The initial missing-test-target attempt remains in `task1-attempt01/package.log`; supplemental harness success is not represented as tests shipped inside the package.
- Actual local producers at separate source directories executed both native tests and passed the existing release payload validator. Their identities explicitly say `local`, run `0`, ARM64; they are not GitHub acceptance evidence.
- [Failed run 34888870200](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34888870200) at `34eeae429f934f06a93f127a1e7f4eec312ccfce` completed producer execution but failed workflow verification because `rg` was unavailable. Its logs and diagnostics remain under `remote-attempt-01`. The corrected full workflow passed in the fresh run above. No failed output was relabeled or overwritten.

## Scope Refinements and Remaining Limits

The plan records the package-safe mirror/parser and language-aware subprocess fixtures as bounded additions. Native repository loading fails on missing or changed authority bytes and checks mirror equality; isolated consumer workspaces use embedded inputs with the same literal guards. Existing LF attributes preserve both copies across platforms. No production physics, tolerance or oracle authority changed.

Success artifacts retain the existing strict typed completion inventory. Diagnostics are separately uploaded on every outcome and bound from the producer identity. The current release consumer accepts the success artifact and additional identity fields, but **does not download or validate the separate diagnostic artifact**; the parent independently retained and checked it here. The release workflow's exact named download and 21-artifact set remain unchanged.

AGENTS.md standing authorization dated 2026-09-13, the Bright Builds sidecar/overrides, active lessons and relevant Rust/testing/architecture/verification standards governed execution. RED evidence was retained without failing commits because repository precommit rules take precedence. Simplification moved the subprocess shell fixture and focused Unix tests into cohesive files. No blocking stubs or unplanned security surfaces remain. STATE.md, ROADMAP.md and REQUIREMENTS.md belong to the parent orchestrator.

## Self-Check: PASSED

The four implementation commits resolve, created source files and both retained artifact sets exist, and the successful source/run and parent independent validation were read before recording completion.

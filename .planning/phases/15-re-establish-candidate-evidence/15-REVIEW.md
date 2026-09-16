---
phase: 15-re-establish-candidate-evidence
reviewed: "2026-09-14T22:38:32Z"
depth: standard
reviewed_head: 71c0cb617db2056b9a836d4889c6dfcc06b6a8e0
original_reviewed_head: 00358a585f8629dbd7b4b4a486da6af0cfe94c7c
diff_base: d990dc6
original_scope_count: 115
repair_scope_count: 43
followup_scope_count: 3
followup_commit: 71c0cb617db2056b9a836d4889c6dfcc06b6a8e0
files_reviewed: 135
files_reviewed_list:
  - .github/workflows/ci.yml
  - .github/workflows/coverage.yml
  - .github/workflows/oracle.yml
  - .github/workflows/phase13-1-canonical-native.yml
  - .github/workflows/phase13-acceptance.yml
  - .github/workflows/phase13-evidence-producer.yml
  - .github/workflows/regressions.yml
  - .github/workflows/release.yml
  - .github/workflows/safety.yml
  - RELEASE.md
  - SAFETY.md
  - crates/liquidfun-differential/tests/phase9_corpus/workflow.rs
  - crates/liquidfun-differential/tests/rust_adapter.rs
  - crates/liquidfun-test-protocol/tests/fixtures.rs
  - crates/liquidfun-test-protocol/tests/fixtures/bytes.rs
  - crates/liquidfun/src/particle/storage/properties/group_model.rs
  - crates/liquidfun/src/particle/storage/properties/group_model/configuration.rs
  - crates/liquidfun/tests/particle_group_properties.rs
  - crates/liquidfun/tests/particle_group_properties/phase14-particle-groups.txt
  - crates/liquidfun/tests/particle_group_properties/registered.rs
  - reference/coverage/contract.json
  - reference/regressions/manifest.toml
  - scenarios/regressions/phase14-particle-groups.txt
  - scripts/install-canonical-clang.sh
  - scripts/phase10-evidence.sh
  - scripts/phase12-attempt.sh
  - scripts/phase12-coverage.sh
  - scripts/phase12-miri.sh
  - scripts/phase12-regressions.sh
  - scripts/phase12-release-evidence.sh
  - scripts/phase12-release-evidence/common.sh
  - scripts/phase12-release-evidence/identity_validation.sh
  - scripts/phase12-release-evidence/package_import.sh
  - scripts/phase12-release-evidence/producer_validation.sh
  - scripts/phase12-release-evidence/raw_payloads.sh
  - scripts/phase12-rust-sanitizers.sh
  - scripts/phase13-1-gap-verification-manifest.json
  - scripts/phase13-1-validate-gap-evidence.sh
  - scripts/phase13-evidence-archive.py
  - scripts/phase13-evidence-query.py
  - scripts/phase15-candidate-evidence.sh
  - scripts/phase15-candidate-evidence/common.py
  - scripts/phase15-candidate-evidence/dispatch.py
  - scripts/phase15-candidate-evidence/dispatch.sh
  - scripts/phase15-candidate-evidence/docs_ci.py
  - scripts/phase15-candidate-evidence/main.py
  - scripts/phase15-candidate-evidence/payloads.py
  - scripts/phase15-candidate-evidence/retention.py
  - scripts/phase15-candidate-evidence/retention.sh
  - scripts/phase15-candidate-evidence/validate.sh
  - scripts/phase15-candidate-evidence/windows_capture.py
  - scripts/phase15-candidate-evidence/windows_job.py
  - scripts/phase15-docs-check.sh
  - scripts/phase15-inventory-check.sh
  - scripts/phase9-evidence.sh
  - tools/reference/CMakeLists.txt
  - tools/reference/CMakePresets.json
  - tools/reference/adapter-inputs.txt
  - tools/reference/upstream_tests.cmake
  - tools/xtask/src/docs.rs
  - tools/xtask/src/docs/contracts.rs
  - tools/xtask/src/docs/contracts/readiness.rs
  - tools/xtask/src/docs/contracts/validation.rs
  - tools/xtask/src/inventory.rs
  - tools/xtask/src/inventory/command.rs
  - tools/xtask/src/inventory/projection.rs
  - tools/xtask/src/inventory/report.rs
  - tools/xtask/src/phase13_evidence/promotion.rs
  - tools/xtask/src/phase13_evidence/promotion/operations.rs
  - tools/xtask/src/phase13_evidence/promotion/preparation.rs
  - tools/xtask/src/phase13_evidence/promotion/provider.rs
  - tools/xtask/src/phase13_evidence/promotion/provider_tests.rs
  - tools/xtask/src/phase13_evidence/promotion/rendering.rs
  - tools/xtask/src/phase13_evidence/promotion/review.rs
  - tools/xtask/src/phase13_evidence/promotion/review_tests.rs
  - tools/xtask/src/phase13_evidence/promotion/validation.rs
  - tools/xtask/src/release.rs
  - tools/xtask/src/release/attestation.rs
  - tools/xtask/src/safety_evidence/contract.rs
  - tools/xtask/src/safety_evidence/contract/validation.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/canonical_toolchain_workflow.rs
  - tools/xtask/tests/canonical_toolchain_workflow/execution.rs
  - tools/xtask/tests/canonical_toolchain_workflow/fake-tool.sh
  - tools/xtask/tests/coverage_workflow.rs
  - tools/xtask/tests/coverage_workflow/rust-identity.sh
  - tools/xtask/tests/coverage_workflow/rust_identity.rs
  - tools/xtask/tests/differential_cli.rs
  - tools/xtask/tests/differential_cli/rigid_commands.rs
  - tools/xtask/tests/docs_contract.rs
  - tools/xtask/tests/docs_contract/core_contracts.rs
  - tools/xtask/tests/docs_contract/promotion_and_workflow.rs
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/inventory_cli/attestation.rs
  - tools/xtask/tests/phase13_1_canonical_native_workflow.rs
  - tools/xtask/tests/phase13_1_canonical_native_workflow/execution.rs
  - tools/xtask/tests/phase13_1_canonical_native_workflow/fake-tool.sh
  - tools/xtask/tests/phase13_1_gap_verification.rs
  - tools/xtask/tests/phase13_1_gap_verification/canonical.rs
  - tools/xtask/tests/phase13_1_gap_verification/producer/lifecycle.rs
  - tools/xtask/tests/phase13_promotion_contract.rs
  - tools/xtask/tests/phase13_promotion_contract/archive_controls.py
  - tools/xtask/tests/phase13_promotion_contract/fake_gh.py
  - tools/xtask/tests/phase13_promotion_contract/query_controls.py
  - tools/xtask/tests/phase15_candidate_evidence.rs
  - tools/xtask/tests/phase15_candidate_evidence/checks.py
  - tools/xtask/tests/phase15_candidate_evidence/docs_ci_checks.py
  - tools/xtask/tests/phase15_candidate_evidence/fake-gh.py
  - tools/xtask/tests/phase15_candidate_evidence/process_child.py
  - tools/xtask/tests/phase15_candidate_evidence/process_controls.py
  - tools/xtask/tests/platform_workflow.rs
  - tools/xtask/tests/regression_workflow.rs
  - tools/xtask/tests/regression_workflow/cargo.sh
  - tools/xtask/tests/regression_workflow/unix.rs
  - tools/xtask/tests/release_attestation.rs
  - tools/xtask/tests/release_attestation/claims.rs
  - tools/xtask/tests/release_attestation/fixture.rs
  - tools/xtask/tests/release_attestation/inventory.rs
  - tools/xtask/tests/release_attestation/readiness.rs
  - tools/xtask/tests/release_cli.rs
  - tools/xtask/tests/release_cli/construction_cases.rs
  - tools/xtask/tests/release_cli/package_handoff.rs
  - tools/xtask/tests/release_cli/package_handoff.sh
  - tools/xtask/tests/release_cli/raw_payloads.rs
  - tools/xtask/tests/release_cli/raw_payloads.sh
  - tools/xtask/tests/release_cli/real_package_handoff.sh
  - tools/xtask/tests/safety_evidence_contract.rs
  - tools/xtask/tests/safety_evidence_contract/attempt-tool.sh
  - tools/xtask/tests/safety_evidence_contract/attempts.rs
  - tools/xtask/tests/safety_evidence_contract/attempts.sh
  - tools/xtask/tests/safety_evidence_contract/miri-control.sh
  - tools/xtask/tests/safety_evidence_contract/miri.rs
  - tools/xtask/tests/support/inventory.rs
  - tools/xtask/tests/support/maturity.rs
  - tools/xtask/tests/upstream_cli.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
historical_findings:
  critical: 0
  warning: 4
  info: 0
  total: 4
resolved_findings: 4
fix_commit: 84405cc21fa5f7cf70cedc5ad532d37101ade2c0
status: clean
generated_by: gsd-code-review
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T21:38:34Z"
---

# Phase 15: Code Review Report

**Reviewed:** 2026-09-14T22:38:32Z
**Depth:** standard
**Files reviewed:** 135 distinct paths across the original review and two repair follow-ups
**Status:** clean — zero unresolved findings; four historical warnings resolved
**Final source:** `71c0cb617db2056b9a836d4889c6dfcc06b6a8e0`
**Original review:** `d990dc6..00358a585f8629dbd7b4b4a486da6af0cfe94c7c`

## Summary

Reviewed the explicit prefreeze source scope covering Plans 01–06 and the implemented Plan 07 promotion prerequisite. Per-file analysis covered workflow inputs, compiler acquisition, Miri and coverage modes, genuine regression registration, package reuse, raw payload checks, dispatch attribution, provider retention, promotion, and C/A/D readiness. A delegated reviewer read all 55 test-related files and reported WR-03; the coordinating reviewer covered the remaining source and integration boundaries.

All four original correctness/recovery warnings were repaired in commit `84405cc21fa5f7cf70cedc5ad532d37101ade2c0` and independently re-reviewed. No unresolved source issue or critical finding remains. The original 115-file standard review was followed by reviews of 43 repair files and three later correction files; their union is 135 paths. Unchanged source retained its original review rather than being represented as fully re-reviewed.

AGENTS.md, AGENTS.bright-builds.md, standards-overrides.md, the architecture/code-shape/verification/testing/operability/Rust standards, both active lesson files (11,548 bytes combined), Phase 15 context, Plans 01–07 and Summaries 01–06 informed the review. No scoped file was excluded by Git ignore rules. No repository-specific skill directory or additional ignore policy was found.

## Final resolutions and verification

- WR-01: bounded process supervision now terminates and reaps owned processes on timeout, overflow and capture I/O exceptions. Original errors survive cleanup/recording failures; partial-write and missing-record controls pass.
- WR-02: every JSON, ZIP and log API call pins github.com, and dispatch names the host-qualified repository. Explicit-host negative controls pass.
- WR-03: the regression fixture fixes its synthetic run attempt. All 14 tests pass with an enclosing attempt of two; the real workflow fixture uses configured GNU Bash.
- WR-04: explicit attestation-backed inventory generation/checking and docs checking preserve source-bound inputs and deterministic report bytes. Oracle, Phase 9/10 and coverage native routes restore fresh provider-validated evidence before checking explicit A. Reuse requires complete byte/hash equality and never overwrites retained files.

The follow-up also reviewed Windows owned-job capture, portable path guards and the incidental grep-based Miri scanner repair. No added actionable source issue was confirmed. Detailed source hashes and staged resolution history are retained in `target/phase15-prefreeze-review/resolution.md`.

`target/phase15-plan07/combined-repair-gate/passed.json` records exit 0 for ordered fmt/Clippy/build/test, private xtask all-target Clippy, **237 tests across nine focused targets**, Bright Builds, configured Markdown, actionlint and diff checks. The reviewer recomputed all 43 repair hashes and confirmed they equal both the gate inventory and committed blobs at `84405cc21fa5f7cf70cedc5ad532d37101ade2c0`.

At the `84405cc` review, native Windows runtime proof remained pending. Subsequent limited proof and the latest correction review are recorded below; final same-C evidence and milestone acceptance remain separate.

### Follow-up at 71c0cb

The three-file correction in `71c0cb617db2056b9a836d4889c6dfcc06b6a8e0` was reviewed clean: `tools/xtask/src/phase13_evidence/promotion/provider.rs` and the two direct Windows Python invocations in `.github/workflows/oracle.yml` now pass `-B` before the script, preventing imported bytecode caches from dirtying the checkout. `crates/liquidfun-differential/tests/phase9_corpus/workflow.rs` detects the identity redirection independently of formatting whitespace while preserving validator-before-identity ordering and behavioral failure controls. No new unresolved finding was identified.

`target/phase15-plan07/read-only-and-workflow-repair/passed.json` records exit 0 for the ordered four Rust gates, workspace all-target/all-feature Clippy, full-workspace tests and ancillary checks. Inspection of `05.log` confirmed **2,274 passed, zero failed, one ignored**, with the ignored item explicitly identified as the fixture-regeneration utility.

`target/phase15-native-portability/attempt-01/collection/validation.json` records **nine actual native Windows process controls passed** at source `3f954fe8183b665d4d1fd15dae467cce5e9e4da3`, run `34903184049`, attempt 1, job `104173806565`. The reviewer confirmed the process helper and control-test bytes are unchanged at `71c0cb`. The overall Oracle run and job **failed provenance validation**; this limited successful step is not passing Oracle or candidate acceptance. The new `-B` startup flags were source-reviewed and have not yet been rerun natively.

Independent human promotion review remains required and unwaived. This code review supplies no human acknowledgement or final evidence acceptance.

## Historical warnings — all resolved in 84405cc

The descriptions and line references below preserve the original findings at `00358a585f8629dbd7b4b4a486da6af0cfe94c7c`. Each is resolved by `84405cc21fa5f7cf70cedc5ad532d37101ade2c0`; none contributes to the current unresolved finding count.

### WR-01: Provider acquisition can block indefinitely and write unbounded diagnostics

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/tools/xtask/src/phase13_evidence/promotion/provider.rs:171-198`

**Issue:** The new provider query bounds stdout byte count, but `read_to_end` waits for EOF without a deadline, and the subsequent `child.wait()` also has no deadline. A stalled provider connection, credential helper, or child that closes stdout without exiting can hang autonomous acquisition indefinitely. Stderr goes directly to an uncapped file. Moreover, the overflow branch writes retained bytes before killing the child; a filesystem-write failure can return while the child remains running. This defeats the bounded failed-attempt recovery contract.

**Fix:** Use a deadline-aware subprocess adapter with concurrent bounded stdout/stderr capture. On timeout, overflow, or capture failure, terminate and reap the owned child/process group before returning, and retain terminal classification and bounded available logs. Add fake-provider controls for no-EOF sleep, closed-stdout sleep, stderr overflow, nonzero exit, and successful bounded JSON.

### WR-02: Candidate orchestration leaves the GitHub hostname implicit

**Files:**

- `/Users/peterryszkiewicz/Repos/liquidfun-rs/scripts/phase15-candidate-evidence/common.py:133-134`
- `/Users/peterryszkiewicz/Repos/liquidfun-rs/scripts/phase15-candidate-evidence/dispatch.py:89`
- Raw archive/log API invocations in `retention.py` and `docs_ci.py` use the same implicit-host pattern.

**Issue:** At the reviewed head, API calls omit `--hostname github.com`, and workflow dispatch supplies only the owner/repository slug. GitHub CLI's documented `GH_HOST` selection can target an authenticated Enterprise host instead. If that host contains the same slug and source/ref, the preflight's `full_name` checks can pass and dispatch can affect that other repository. The hardcoded github.com response-URL check runs only after the effect. Read-only restoration can likewise query the wrong host.

**Evidence:** Local `gh help environment` documents `GH_HOST`; `gh workflow run --help` explicitly supports `[HOST/]OWNER/REPO`. No remote effect was performed for this review.

**Fix:** Pin every API call to `--hostname github.com` and dispatch with `--repo github.com/bright-builds-llc/liquidfun-rs`. Cover both JSON and raw ZIP/log paths, and add a non-default `GH_HOST` fixture that verifies explicit host arguments before the dispatch effect.

### WR-03: Regression fixture fails when CI reruns inherit attempt two or later

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/tools/xtask/tests/regression_workflow/unix.rs:158-161,466`

**Issue:** `Fixture::run` sets synthetic workflow, job, and run ID but inherits `GITHUB_RUN_ATTEMPT`. The production regression helper intentionally reads this variable; the test later requires `producer["run_attempt"] == 1`. Consequently a GitHub rerun with attempt 2 or later fails the test even when the producer records the correct environment.

**Fix:** Set the fixture attempt explicitly alongside its other synthetic provider identities:

```rust
.env("GITHUB_RUN_ID", "42")
.env("GITHUB_RUN_ATTEMPT", "1")
```

Verify the affected regression test with an enclosing `GITHUB_RUN_ATTEMPT=2`; do not weaken the production attempt binding or the assertion.

### WR-04: Ready docs validation has no matching generated compatibility projection

**Primary file:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/tools/xtask/src/docs/contracts/readiness.rs:27-32`

**Integration reference:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/tools/xtask/src/inventory/report.rs:85` at the reviewed head.

**Issue:** Explicit accepted C/A docs require `COMPATIBILITY.md` to contain standalone release-ready and source/attestation markers. Its repository-owned generator still unconditionally emits non-ready status and has no accepted-attestation input. The documented later-D path therefore cannot satisfy both generated-report freshness and ready docs validation without hand-editing generated output or changing source after C. The orchestrator independently identified this integration gap during this review.

**Fix:** Add an explicit attestation-aware generation path that calls the existing full validator before rendering ready C/A markers. Keep default generation non-ready, preserve ledger-derived counts, and test actual generation followed by docs validation plus invalid/tampered attestation rejection. Complete this source prerequisite before freezing C.

## Scope and verification limits

- The original primary scope contained 115 paths. The first follow-up reviewed 43 repair paths, including 19 additional paths. The latest three-file follow-up added the Phase 9 workflow test, producing the explicit 135-path union in frontmatter. This is cumulative scope, not a claim that all 135 were re-reviewed from scratch at the final commit.
- Existing assertions and prerequisite evidence were reviewed; this read-only review did not rerun broad Cargo, native, Miri, coverage, or remote suites. It does not certify new producer execution.
- The nine-path attestation allowlist, seven-path promotion transaction, 21 producer artifacts, and 19-entry release registry were not widened in the reviewed changes.
- Missing final-C producers, controlled-performance infrastructure, final C/A attestation, and milestone acceptance are known remaining phase work, not source findings by themselves.
- No source files, state, roadmap, commits, pushes, dispatches, or approval records were changed by this review.

______________________________________________________________________

_Reviewer: gsd-code-reviewer, with delegated test-scope review._

## Reviewer policy update — 2026-09-16

The earlier human-only promotion requirement above describes the policy at review time. The owner's subsequent authorization, recorded in AGENTS.md Independent review and Phase 15 D-10, permits a separate identified AI reviewer. A fresh acknowledgment must bind the current review packet; prior reviews and evidence retain their original identities and scope.

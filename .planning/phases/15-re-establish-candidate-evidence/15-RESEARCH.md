---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: 2026-09-14
---

# Phase 15: Re-establish Candidate Evidence - Research

**Researched:** 2026-09-14
**Domain:** Candidate-bound Rust/C++ release evidence and GitHub Actions orchestration
**Confidence:** HIGH for inspected contracts and observed failures; MEDIUM for unexecuted remediation.

<user-constraints>
## User Constraints (from CONTEXT.md)

The following decisions and deferred scope are copied verbatim. [VERIFIED: 15-CONTEXT.md]

### Locked Decisions and Agent Discretion

### Candidate freeze
- **D-01:** Finish source, tooling, planning and prerequisite fixes before freezing a clean reviewed full 40-hex candidate SHA. All accepted producers bind that SHA; never substitute later HEAD or Phase 14 D2 repair proof.
- **D-02:** Reuse existing release/attestation and producer contracts. Audit their actual end-to-end behavior before expensive production; fix demonstrated orchestration defects with negative tests instead of weakening required evidence, hashes, source confinement, or review authority.
- **D-03:** Preserve the frozen-source/later-attestation separation. Planning, ordinary docs and scripts are outside the current nine-path attestation allowlist. Plan an explicit clean checkout/range strategy so ongoing GSD metadata does not falsify the frozen-source attestation.

### Evidence and recovery
- **D-04:** Run inexpensive source-bound and exact-candidate Cargo/Oracle prerequisites first, then independent producers where resources permit. Collect all 19 required registry entries through the actual reviewed platform, oracle, safety, fuzz, regression, coverage, performance and release workflows.
- **D-05:** Retain complete bounded artifacts, raw measurements, metadata, logs, checksums and run/attempt identities promptly in distinct attempt directories. Reuse only independently valid same-candidate complete outputs; diagnose failed runs and reconcile uncertain dispatches before retrying.
- **D-06:** Preserve one exact package across MSRV/platform verification and release aggregation. Keep canonical D1, portability D2 and diagnostic D3 evidence distinct; never widen tolerances or self-bless expected oracle outputs.

### Infrastructure and acceptance
- **D-07:** Controlled performance requires the reviewed `performance-controlled-linux-x64` runner and matching `PERFORMANCE_CONTROLLED_HOST_IDENTITY`, complete 32 cases/14 workloads, and existing measurement policy. Shared-host scheduled results remain diagnostic. On 2026-09-14 repository API listed zero runners and repository secret list lacked this secret; organization APIs returned 403. An asynchronous question requests existing approved infrastructure details; independent work continues.
- **D-08:** Standing authorization in AGENTS.md dated 2026-09-13 authorizes routine diagnosis, fixes, checks, commits, ordinary main pushes and relevant workflow recovery. Older per-candidate approval or one-attempt limits do not apply. Missing access, material scope expansion, destructive/security changes, package release and acceptance waivers remain outside that standing scope.
- **D-09:** Keep public status non-ready until aggregation and both worktree/committed-range attestation pass. Then project accepted source/attestation identities and repeat the milestone audit for all requirements and critical end-to-end flows. Do not mark missing evidence complete.

### Agent discretion
- Choose bounded orchestration, attempt-directory names, plan granularity and independent scheduling; preserve the exact acceptance contracts. Correct demonstrably broken producer plumbing before choosing the final frozen candidate.

### Deferred Ideas (OUT OF SCOPE)

Package publication, tags authorizing a release, new platform promises and physics acceleration are outside this phase request. No pending todos matched this phase.
</user-constraints>

<phase-requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| PLAT-01 | The complete supported v1 surface builds and passes required verification on Linux x86_64. | Exact package MSRV/native Linux run, canonical acceptance, safety/coverage repairs and candidate freeze sequence. |
| PLAT-05 | The complete supported v1 surface builds and passes required verification on Windows x86_64. | Fresh Windows package/native run at the final SHA; Phase 14 D2 is historical repair evidence only. |
| DOCS-09 | The final v1 release audit verifies complete docs, required notices, packaged crate contents, upstream test/example accounting, supported platforms, benchmarks, safety evidence, and zero unexplained compatibility gaps. | All 19 evidence entries, confined retained bundle, exact package join, two attestation validations, truthful projections, repeated milestone audit. |

Requirement text and current pending status are verified in `.planning/REQUIREMENTS.md`. [VERIFIED: .planning/REQUIREMENTS.md]
</phase-requirements>

## Project Constraints (from AGENTS.md)

- Work through the GSD lifecycle; keep planning and verification artifacts accurate. Do not format `.planning/**` with mdformat. Format changed non-GSD Markdown with mdformat 1.0.0/Python 3.13 and run `just markdown-check`. [VERIFIED: AGENTS.md]
- Apply standing authorization dated 2026-09-13 for ordinary diagnosis, repairs, verification, commits, main pushes and workflow recovery; preserve failed evidence, recheck effect identity, and do not invent approval timestamps. Package publication, acceptance waivers, destructive actions and security/account changes require separate authority. [VERIFIED: AGENTS.md; standards-overrides.md]
- Keep native Rust, renderer independence, private out-of-process pinned C++ oracle, Cargo-only consumers, exact semantic comparisons, deterministic ordering and truthful maturity claims. No new dependencies or architecture changes are needed for evidence collection. [VERIFIED: AGENTS.md; RELEASE.md]
- Run the managed checker before commits; use pure typed validation with thin effectful orchestration, early returns, focused Arrange/Act/Assert tests, and no swallowed failures. Use module files rather than embedded foreign programs. [VERIFIED: AGENTS.bright-builds.md; standards/core/architecture.md; standards/core/testing.md; standards/core/verification.md; standards/languages/rust.md]
- Respect the user-provided ordered Rust precommit sequence: format, Clippy, all-target/all-feature build, all-feature tests; parent owns research-commit gates. This researcher makes no source edits, dispatches or commits. [VERIFIED: user instructions and delegated task]
- Both active lesson files were loaded fully: global 5,230 bytes and repository 6,318 bytes, total 11,548 bytes. Lessons require exact CI boundaries, explicit test identities, all canonical presets, Cargo isolation, independent oracle bits, exact content hashes and standing authorization. No project skill indexes were found under `.agents/skills` or `.claude/skills`. [VERIFIED: filesystem probes]

## Summary

Phase 15 must start with prerequisite repair, not a release dispatch. The retained current-source failures include an LLVM installer checksum mismatch, nightly coverage tier assertion, Miri exact-bit assertion, and empty regression registry. Separately, source inspection proves the release workflow packages twice, regression/safety retries erase fixed output directories, and readiness text contracts are hardcoded to non-ready. These are repairable repository issues. Controlled performance infrastructure is the genuine external dependency currently unconfirmed. [VERIFIED: target/phase15-preflight/ci/*/failed.log; scripts/phase12-release-evidence/common.sh; scripts/phase12-regressions.sh; scripts/phase12-miri.sh; tools/xtask/src/docs/contracts.rs; 15-CONTEXT.md]

Use existing producer/validator boundaries, repair and test them before freezing, regenerate reviewed canonical closure after all relevant source changes, then choose one final full SHA. Collect 21 precisely named producer artifacts which become 19 registry records. Retain original raw outputs in addition to aggregate envelopes. Make the attestation commit directly descend from the frozen candidate with only allowlisted records changed; write later GSD summaries and public projections after that commit. [VERIFIED: reference/release/required-evidence.toml; scripts/phase12-release-evidence/producer_validation.sh; tools/xtask/src/release/attestation.rs]

**Primary recommendation:** Plan bounded prerequisite repairs first, followed by canonical requalification, one-package candidate production, retained aggregation, isolated attestation and milestone audit. Do not gate repair work on the performance-host answer. [VERIFIED: 15-CONTEXT.md D-01 through D-09]

## Standard Stack

Use existing pins, not newest releases. This phase introduces no package; registry freshness is not a reason to upgrade the locked evidence toolchains. Listed versions are repository pins, not a claim about latest upstream releases. [VERIFIED: rust-toolchain.toml; rust-toolchain-nightly.toml; RELEASE.md; workflows]

| Component | Required identity | Purpose and source |
| --- | --- | --- |
| Rust/Cargo | 1.97.0; published MSRV 1.92.0 | Package and ordinary verification. [VERIFIED: RELEASE.md] |
| Rust nightly | nightly-2026-07-15 | Miri, ASan and instrumented coverage. [VERIFIED: scripts/phase12-miri.sh; scripts/phase12-rust-sanitizers.sh; scripts/phase12-coverage.sh] |
| Clang | 22.1.8 | Canonical C++, sanitizer and performance identity. [VERIFIED: required-evidence.toml; performance.yml] |
| CMake/Ninja | 4.3.3 / 1.13.2 in canonical workflows | Exact reference build tooling. [VERIFIED: oracle.yml; coverage.yml] |
| Upstream oracle | 7f20402173fd143a3988c921bc384459c6a858f2 | Read-only reference. [VERIFIED: reference/upstream-lock.toml] |
| Bash, jq, gh, SHA-256 utilities | Bash with mapfile; repository CI utilities | Artifact acquisition/joins. Mac system Bash is 3.2 and cannot satisfy the existing mapfile-based aggregator. Use a verified modern Bash or run aggregation on Linux. [VERIFIED: local bash --version; producer_validation.sh] |
| cargo-deny / cargo-llvm-cov | 0.20.2 / 0.8.7 | Audit and coverage. [VERIFIED: release.yml; phase12-coverage.sh] |

No installation command for new application libraries is required. Repair canonical tool provisioning through a reviewed immutable source and verify installed version, not merely the installer's checksum. [VERIFIED: current mutable llvm.sh URL and downstream compiler identity checks]

## Architecture Patterns

### Evidence producer matrix: all 19 records

Each row is derived from the closed registry; producer artifact names and prerequisites come from the corresponding workflow and aggregator. Every row is a required acceptance condition, not a report that it currently passes. [VERIFIED: reference/release/required-evidence.toml; scripts/phase12-release-evidence/aggregation.sh; producer_validation.sh]

| # | Registry key | Producer / artifact or source | Required preparation |
| --- | --- | --- | --- |
| 1 | package / all | release.yml release-candidate; imported platform package | Create once in platform package job; consume unchanged in release preparation. |
| 2 | msrv / Linux x64 | platform.yml msrv; phase12-platform-msrv-RUN-SHA | Rust 1.92.0 verifies exact imported archive. |
| 3 | platform / Linux x64 | platform.yml native; phase12-platform-x86_64-unknown-linux-gnu-RUN-SHA | Native Cargo-only isolated package verification. |
| 4 | platform / Linux ARM64 | native; phase12-platform-aarch64-unknown-linux-gnu-RUN-SHA | Same bytes on native architecture. |
| 5 | platform / macOS ARM64 | native; phase12-platform-aarch64-apple-darwin-RUN-SHA | Same bytes; D2, not D1. |
| 6 | platform / Windows x64 | native; phase12-platform-x86_64-pc-windows-msvc-RUN-SHA | Fresh final-candidate verification; preserve Phase 14 regressions. |
| 7 | conditional_platform / macOS Intel | conditional-policy; native or downgrade artifact | Current null native evidence requires explicit unsupported downgrade, not a new support promise. |
| 8 | canonical_differential / Linux x64 | oracle.yml phase11-canonical-linux; phase11-canonical-RUN-SHA | Manual evidence_phase=phase11 at exact workflow SHA; strict semantic-result and file inventory. |
| 9 | rust_safety / Linux x64 | safety.yml miri; phase12-miri-RUN-SHA | Both Miri and separate Rust ASan artifacts are required by aggregation. |
| 10 | cpp_sanitizer / Linux x64 | oracle.yml phase11-sanitizer-linux; phase11-sanitizer-RUN-SHA | Fail-fast ASan/UBSan, protocol scope and all comparison scenarios. |
| 11 | fuzz / Linux x64 | fuzz.yml fuzz; five fuzz-TARGET-RUN-SHA artifacts | protocol, shapes_collision, world_mutation, particles, groups_ownership; bounded successful classifications. |
| 12 | regressions / Linux x64 | regressions.yml regressions; phase12-regressions-SHA | Register actual minimized findings and prove every named test executes, then independent validation. |
| 13 | rust_coverage / Linux x64 | coverage.yml rust-coverage; phase12-rust-coverage-RUN-SHA | Repair nightly tier test; retain raw coverage and summary. |
| 14 | cpp_coverage / Linux x64 | coverage.yml cpp-coverage; phase12-cpp-coverage-RUN-SHA | Repair compiler acquisition; also obtain separate differential coverage artifact with zero missed leaves. |
| 15 | performance / Linux x64 | performance.yml performance; phase12-performance-RUN-SHA | Approved controlled host + matching secret, all 32 cases/14 workloads, complete raw reports and calibration. |
| 16 | docs / all | release preparation | Warning-denied rustdoc, doctests and docs contracts at frozen source. |
| 17 | notices / all | release preparation | cargo-deny and provenance/license/source notices pass. |
| 18 | corpus_closure / all | release preparation | inventory corpus check-snapshot and check-closure; no unresolved/nonterminal items. |
| 19 | compatibility_closure / all | release preparation | Fresh authority hash; zero unexplained applicable gaps without D2/coverage promotion. |

The 21-artifact closed set is seven platform/package artifacts, two Phase 11 Oracle artifacts, two Rust safety artifacts, five fuzz artifacts, one regression artifact, three coverage artifacts and one performance artifact. Avoid broad download patterns when an exact name list is available, while keeping registry cardinality 19 unchanged. [VERIFIED: write_expected_artifacts in producer_validation.sh]

### Concrete repair backlog before freeze

1. **Immutable compiler provisioning.** Oracle runs 34818098018 and 34834656349 and coverage 34835817427 fail the fixed checksum for mutable `https://apt.llvm.org/llvm.sh`. Audit all consumers, including Phase 13 producer/acceptance workflows, then pin reviewed immutable installer/build inputs and assert Clang 22.1.8 plus compile/link identities. Never merely remove checksum validation. Validate canonical debug, release and sanitizer presets before expensive evidence. [VERIFIED: retained failed.log files; oracle.yml; coverage.yml; phase13-acceptance.yml]
1. **Nightly identity test.** Coverage run 34835817427 fails `adapter_binds_native_identity_and_request_hashes` at `crates/liquidfun-differential/tests/rust_adapter.rs:116`. Reproduce under the exact nightly/coverage command, distinguish expected evidence-tier downgrade from production behavior, and use explicit known tier identities where the test concerns classification. Do not label nightly as canonical stable. [VERIFIED: retained coverage failed.log; lesson-use-explicit-ci-identities]
1. **Miri numerical test and missing rg.** Safety run 34830274188 fails `math::sweep::tests::transform_endpoints_preserve_exact_expected_bits` at sweep.rs:437; its prerequisite step also reports missing `rg`. The negative scan's `if rg ...` can treat command-not-found as a clean scan. Install/probe required tools or use a checked available equivalent, and make scan execution failure fatal. Diagnose the exact-bit case under Miri against the numerical policy; preserve independent native/oracle numerical evidence and do not copy Miri actual bits into expectations or skip the safety concern. [VERIFIED: retained safety failed.log; safety.yml]
1. **Truthful regression registration.** `reference/regressions/manifest.toml` has zero rows and production intentionally rejects zero, confirmed by run 34844182788. Phase 14 has genuine persisted Windows findings, fixed commits and named replays. Translate those records into the existing registry with immutable minimized input bytes and actual original/fix identities; use a replay entry that consumes the registered input. Preserve true failure classification, rather than inventing a PhysicsMismatch/oracle identity. Check exact test selection returns one executed test; Cargo's successful exit alone is insufficient. [VERIFIED: manifest.toml; phase12-regressions.sh; 14-VERIFICATION.md]
1. **One package.** `prepare_output` currently invokes `package create-artifact` again; platform.yml has already created the archive. Move exact platform package acquisition before prepare; pass/import its verified archive and identity, retain the release registry producer identity, and run verify-artifact without repackaging. Explicitly compare publication dry-run output bytes with imported archive; current code runs dry-run without that comparison. Negative-test wrong candidate/hash/bytes and the actual cross-workflow handoff. [VERIFIED: common.sh; platform.yml; RELEASE.md]
1. **Artifact confinement and retention.** Shell `validate_target_path` checks target prefix and symlinks but does not reject dot/dot-dot path components; payload paths read from identities are joined without a common normalized-component check. The Rust auditor has a stronger confined path boundary. Repair and negatively test lexical traversal, parent-directory symlinks, extra roots/files, duplicates, wrong payload hashes, missing logs, malformed summaries and stale run identities. Verify performance payload-files.sha256 and raw files, not just its manifest-entry envelope. [VERIFIED: common.sh; producer_validation.sh; identity_validation.sh; release/validation/repository.rs; performance.yml]
1. **Preserve attempts.** Safety and regression prepare helpers remove pre-existing candidate directories; regression success deletes temporary per-test logs. Introduce fresh attempt storage or require a new isolated checkout per attempt and retain logs externally before cleanup. Final producer success uploads currently omit failed-run artifacts. Add bounded failure retention with terminal classification and no successful identity on failure. [VERIFIED: phase12-miri.sh; phase12-rust-sanitizers.sh; phase12-regressions.sh; safety.yml; regressions.yml]
1. **Readiness projection.** Docs contracts require literal non-ready sentences in README, COMPATIBILITY and RELEASE. Add a tested conditional projection before freeze that accepts ready text only from valid attestation, rejects absent/tampered records, and leaves frozen source non-ready. Do not edit those contracts after freeze. [VERIFIED: tools/xtask/src/docs/contracts.rs:330; RELEASE.md]

### Supplemental findings for first implementation wave

- **Coverage toolchain identity takes priority over changing the test.** The coverage script invokes pinned nightly while the release registry calls the Rust coverage producer `rust-1.97.0`. Verify whether the existing coverage operation works on stable 1.97.0 and prefer aligning execution with its intended stable identity. Never conceal a nightly run behind a stable envelope or relax the compiler-tier assertion merely to make that mismatch pass. [VERIFIED: scripts/phase12-coverage.sh; reference/release/required-evidence.toml; parent research update]
- **Miri diagnosis has a concrete reproduction.** The separate diagnostic agent reports that two independent trigonometric evaluations differ under default pinned Miri extra-rounding behavior; both the minimal probe and focused endpoint test pass with `-Zmiri-no-extra-rounding-error`. Plan a bounded solution that preserves default-mode semantics/UB checks and separately identifies any deterministic-math mode; do not widen native tolerance or copy observed bits. Confirm the diagnostic agent's Linux math-subset result before locking the exact implementation. [VERIFIED: parent-provided diagnostic agent results; final full-subset validation pending]

### Canonical receipt sequence after repairs

The historical Phase 13 acceptance hashes the replay closure across engine, differential, protocol, Cargo and other sources. Phase 14 changes engine sources included in that closure; an old receipt therefore cannot authorize the changed source merely because the original witness values remain equal. The closure projects seven promoted output paths back to promotion base R, which supports a noncircular P → R → Q → A sequence. [VERIFIED: phase13_acceptance.rs REPLAY_REPOSITORY_PREFIXES/PROMOTED_PATHS; phase13_acceptance/closure.rs; 14-VERIFICATION.md]

After all closure-affecting fixes, run the existing Phase 13 evidence producer at P, retain the full bundle/provider identity, independently check acquisition, review the actual semantic diff, and promote through existing typed promotion transactions at R/Q. Use required commit trailers from the generated receipt, not invented metadata. Then run exact-head canonical acceptance at A, which can be the final candidate or a source-equivalent later commit; any subsequent closure-affecting change requires requalification. Inspect existing promotion APIs to prepare concrete arguments during planning. [VERIFIED: phase13_evidence.rs commands; phase13_evidence/promotion/*; phase13_acceptance/identity.rs; inherited Phase 13 CONTEXT]

Run `oracle.yml` with `evidence_phase=phase11` at final C as well: push Cargo CI and ordinary Oracle jobs do not provide the required Phase 11 pair. Current Oracle dispatch has no candidate input and uses github.sha throughout; either dispatch an exact resolved ref and verify its returned SHA, or add candidate_sha plumbing consistently before freeze. Never send an unsupported input or silently reuse current main. [VERIFIED: oracle.yml triggers/conditions; scripts/phase11-evidence.sh]

### Freeze, artifact paths and attestation

- Complete plans, implementation, prerequisites and allowed prefreeze verification metadata before C. Preserve later execution observations in ignored, append-only attempt records until the attestation commit exists. Do not claim phase evidence is complete before producing it. If lifecycle requirements insist on a final verification document before freeze, clarify the prefreeze source-readiness versus final evidence distinction in RELEASE.md before C. [VERIFIED: 15-CONTEXT.md D-01/D-03; RELEASE.md sequence]
- Prefer an isolated checkout at C for collection and attestation, while the orchestrating checkout tracks progress. Attestation source-tree hash is SHA-256 of raw `git ls-tree -r -z --full-tree C` bytes, not `git rev-parse C^{tree}`. Manifest and report hashes are exact bytes. [VERIFIED: release/attestation.rs]
- Bundle manifest paths are repository-relative `target/...` paths. Download retained contents back to the same repository-relative locations in the attestation checkout; copying only the three tracked JSON files cannot pass audit. Keep the package and all aggregate envelopes available, with separate original producer archives/metadata/raw logs for independent review. [VERIFIED: common.sh emit_evidence; release.yml upload; release/validation.rs]
- Validate proposed worktree; commit only the three release JSON records as direct descendant of C; validate explicit C..A range using full A. Current nine-path allowlist also includes six workflow/Rust/test paths, but there is no need to change them after freeze. It excludes all GSD metadata, scripts and ordinary docs. [VERIFIED: ATTESTATION_PATHS; validate_worktree_paths; validate_committed_paths]
- After A is validated, commit projections and final lifecycle/milestone audit metadata at D. Always name C as source and A as attestation. A future command executed at D must use explicit attestation A and retained artifacts; do not run worktree validation against D and expect ordinary docs to be allowed. [VERIFIED: attestation.rs range semantics; RELEASE.md]

## Don't Hand-Roll

| Problem | Use | Why |
| --- | --- | --- |
| Release joins and authorization | Existing typed release audit plus repaired producer adapters | Closed kinds, identities, hashes and claims are already implemented. [VERIFIED: release/validation.rs] |
| Canonical promotion | Phase 13 producer, acquire-check, typed review/promotion and acceptance | Binds scoped closures and prevents native self-blessing. [VERIFIED: phase13_evidence.rs; phase13_acceptance.rs] |
| Scenario/performance execution | Existing paired runner and policy | Sealed exact scenarios and sample cardinality already exist. [VERIFIED: BENCHMARKING.md; performance/policy.json] |
| Platform proof | Platform archive fan-out | Tests actual consumer package in native lanes. [VERIFIED: platform.yml] |
| Regression inventory | Typed safety-evidence registry/execution projection | Carries minimized content hashes and historical failure/fix provenance. [VERIFIED: phase12-regressions.sh] |

## Runtime State Inventory

This phase repairs/migrates evidence plumbing, so include runtime state explicitly. No project-wide rename is proposed. [VERIFIED: delegated scope; 15-CONTEXT.md]

| Category | Items found / uncertainty | Required action |
| --- | --- | --- |
| Stored data | GitHub run artifacts and local target attempt trees; historical receipts remain tracked. [VERIFIED: retained CI logs; workflows] | Preserve failures; acquire fresh C-bound artifacts, no relabeling or overwrite. |
| Live service config | Required controlled runner currently unconfirmed; org runner APIs inaccessible. [VERIFIED: 15-CONTEXT.md] | Resolve approved runner access before performance dispatch; do not invent availability. |
| OS-registered state | No OS service registration is part of current scripts; remote runner service state was not inspected. [VERIFIED: inspected producer scripts; scope limitation] | Confirm only if approved runner provisioning becomes authorized; no local service edits. |
| Secrets/env vars | PERFORMANCE_CONTROLLED_HOST_IDENTITY is mandatory for controlled manual performance; repository listing lacks it. [VERIFIED: performance.yml; 15-CONTEXT.md] | Use existing approved infrastructure or resolve access; security changes are separate authority. |
| Build artifacts | C++ target/reference and candidate package/evidence outputs are compiler/SHA-bound. [VERIFIED: workflow and package contracts] | Rebuild at repaired pinned toolchain; fresh attempt paths; never reuse stale outputs by name alone. |

## Common Pitfalls

- A green Cargo run is D2/build evidence, not 19-entry closure. Current source ab42204e68049b645165c213e0ca1f9b6dfdaacc has Cargo run 34818097534 success, while independent producers fail. [VERIFIED: parent-retained CI metadata; failed logs]
- Nine allowlisted paths are not nine arbitrary categories: adding .planning or README breaks worktree/range attestation. Use the exact clean-checkout sequence above. [VERIFIED: release/attestation.rs]
- Same candidate archive hashes are necessary but do not fulfill the create-once requirement if two producers independently package. Repair actual ownership. [VERIFIED: RELEASE.md; common.sh; platform.yml]
- Do not call an empty reviewed performance-report list an automatic blocker: release accepts reviewed_report_count >= 0 with no generalized performance claim, but still requires the controlled complete raw matrix. [VERIFIED: aggregation.sh; identity_validation.sh; reference/performance/manifest.toml]
- Capture short-retention evidence promptly: safety, regression and coverage currently retain for 14 days; Oracle/performance/fuzz for 30; platform/release for 90. [VERIFIED: respective workflow retention-days]
- A successful registry filter that executes zero tests, a missing scanner inside an if condition, or a ready aggregate envelope without retained raw producers is insufficient evidence. [VERIFIED: scripts inspected above; acceptance contracts]

## Code Examples

Existing command forms; use only after required checkout/tool preparation. SHA and path values must come from the candidate/attempt record. [VERIFIED: RELEASE.md; phase13_evidence.rs; phase12-release-evidence.sh]

```bash
cargo xtask safety-evidence validate-regressions --emit-execution-list
bash scripts/phase12-release-evidence.sh check
cargo xtask phase13 evidence live-check --tracked --require-reviewed
cargo xtask phase13 acceptance

cargo xtask release audit --manifest reference/release/candidate-manifest.json \
  --candidate "$candidate_sha" --output json
cargo xtask release attestation validate-worktree \
  --source reference/release/source-candidate.json \
  --manifest reference/release/candidate-manifest.json \
  --report reference/release/audit-report.json
cargo xtask release attestation validate \
  --source reference/release/source-candidate.json \
  --manifest reference/release/candidate-manifest.json \
  --report reference/release/audit-report.json \
  --attestation-commit "$attestation_sha"
```

Useful existing negative-test surfaces are `release_cli`, `release_attestation`, `regression_workflow`, `coverage_workflow`, `safety_evidence_contract`, `phase13_promotion_contract` and `phase13_acceptance_contract` under tools/xtask/tests. Add behavior tests at these seams, including real package handoff and fresh-checkout materialization. [VERIFIED: test-file inventory]

## Bounded Plan Recommendations

1. Repair canonical tool acquisition and smoke every canonical preset; shared helper with pinned immutable inputs, not copies of mutable installer recipes. [VERIFIED: observed LLVM failures]
1. Repair exact nightly coverage/Miri tests and fail-closed required-tool checks; replay focused failures under the actual toolchain before full safety production. [VERIFIED: retained failures]
1. Register genuine Phase 14 minimized findings, prove selected tests execute, and preserve regression logs/attempts. [VERIFIED: registry plus Phase 14 proof]
1. Repair one-package handoff, strict artifact/path validation, payload verification, failure retention and tested prefreeze readiness projection. [VERIFIED: inspected producer/consumer defects]
1. Requalify canonical closure with independently reviewed fresh P/R/Q evidence, then acceptance; finish all source changes and freeze C. [VERIFIED: closure/promotion code]
1. Produce and retain platform + exact package, Phase 11 canonical/sanitizer, Rust safety, fuzz, regressions and coverage at C. Resolve controlled-host access concurrently; run performance only when approved host identity is available. [VERIFIED: registry; D-07]
1. Aggregate exactly 19 at C, retain complete evidence, materialize records in isolated checkout, validate worktree, commit A, validate committed range. [VERIFIED: release contract]
1. Project accepted C/A identities, finish final phase verification and repeat complete milestone requirements/integration/flow audit; package publication remains excluded. [VERIFIED: D-09; milestone audit scope]

## Environment Availability

| Dependency | Observed availability | Fallback / requirement |
| --- | --- | --- |
| Local Rust/Cargo | cargo 1.97.0 available | Local focused tests; no substitute for native remote evidence. [VERIFIED: command probe] |
| Local Bash | /bin/bash 3.2.57 | Modern Bash required for mapfile, or canonical Linux aggregation. [VERIFIED: command probe; producer_validation.sh] |
| jq, gh, timeout, SHA-256, node, bun | Commands available | Existing shell and GSD orchestration. [VERIFIED: command probes] |
| Local CMake / Ninja | 3.27.9 / 1.13.2 | Local CMake meets dev floor but differs from canonical 4.3.3; remote pinned environment required. [VERIFIED: command probes; UPSTREAM.md] |
| Canonical Linux compiler installation | Current workflow fails integrity validation | Repair immutable provisioning before evidence. [VERIFIED: retained Oracle/coverage failures] |
| Controlled Linux performance runner | Not demonstrated accessible; repo runners empty, org APIs 403 | Existing approved runner information/access required. Shared hosts cannot satisfy acceptance. [VERIFIED: 15-CONTEXT.md] |
| PERFORMANCE_CONTROLLED_HOST_IDENTITY | Not in repository secret listing; inherited access unknown | Match approved host identity, never fabricate or change account security silently. [VERIFIED: 15-CONTEXT.md; performance.yml] |

**External acceptance blocker:** approved controlled performance host/access. Repository API absence does not prove an organization runner cannot exist. No closure may be claimed without the successful controlled run. **Fixable local prerequisites:** all source/workflow/registry defects above. [VERIFIED: 15-CONTEXT.md; performance workflow checks]

## Security Domain

Security enforcement is enabled by default because config does not set it false. This is a native library/developer workflow, not a web authentication application. ASVS is a web technical-control standard; its current official page identifies 5.0.0 and warns that numbered identifiers change by version. Therefore do not mislabel the legacy template's V2/V3/V4/V5/V6 headings as current 5.0 category IDs. [VERIFIED: .planning/config.json; Cargo.toml] [CITED: https://owasp.org/www-project-application-security-verification-standard/]

| Security concern | Applies | Standard control |
| --- | --- | --- |
| Authentication / session management | CI access only, no new application login/session | Existing GitHub token permissions and approved runner secret. [VERIFIED: workflows] |
| Access control | Yes, source and workflow effects | Recheck repository, run, candidate and artifact identities; standing authorization scope. [VERIFIED: AGENTS.md] |
| Input validation / injection | Yes | Full-SHA parsing, bounded typed JSON, normalized confined paths, quoted arguments; ASVS v5.0.0-1.2.5 supports parameterized OS calls. [VERIFIED: release validators] [CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| Cryptography / integrity | Yes | Existing SHA-256 crates/utilities; no custom cryptography or checksum bypass. [VERIFIED: release validation code] |

| Threat | STRIDE | Mitigation |
| --- | --- | --- |
| Mixed candidate or substituted producer | Spoofing/tampering | Exact identity joins, run/attempt retention, independent validator. [VERIFIED: release contracts] |
| Traversal/symlink payload escape | Tampering/disclosure | Reject components and symlink ancestors, normalize before I/O. [VERIFIED: Rust confined reader; shell repair finding] |
| Missing logs or overwritten failure | Repudiation | Append-only attempt storage, terminal status, original complete artifacts. [VERIFIED: standing authorization] |
| Hung native/fuzz/sanitizer process | Denial of service | Existing bounded runtime/log policies and failure classification. [VERIFIED: producer scripts] |

## State of the Art

This phase advances the repository from implemented evidence contracts to actually produced, retained candidate evidence; it does not upgrade the stack. Prior Phase 14 repair D2 and historical Phase 13 receipts retain their historical authority but cannot be rebadged as final C-bound acceptance. [VERIFIED: 14-VERIFICATION.md; phase13_acceptance closure; 15-CONTEXT.md]

## Assumptions Log

No unverified external facts are locked here. Remediation steps are recommendations based on inspected contracts; they remain unexecuted. Availability of inherited organization runner/secret access and exact cause of Miri numerical drift are explicitly unresolved, not assumed safe. [VERIFIED: research scope; retained failures; 15-CONTEXT.md]

## Open Questions

1. Which existing approved controlled runner and matching identity are available? User question is pending in the parent; independent repairs must continue. [VERIFIED: 15-CONTEXT.md]
1. What precise numerical operation causes the Miri endpoint-bit difference? Reproduce on pinned Miri and compare policy/oracle before choosing a test repair. [VERIFIED: failure observed; root cause not yet established]
1. Does a candidate-ready lifecycle snapshot meet existing RELEASE.md prefreeze wording without falsely completing postfreeze evidence? Resolve the documented split before C; never widen attestation allowlist as a convenience. [VERIFIED: RELEASE.md; D-01/D-03]
1. Are all prefreeze repair workflows fully terminal at final C and are earlier retained artifacts still complete? Parent performs CI inspection; this research authorizes no success claim. [VERIFIED: delegated research scope]

## Sources

Primary code/evidence sources: `15-CONTEXT.md`; `AGENTS.md`; `AGENTS.bright-builds.md`; `standards-overrides.md`; relevant managed architecture/testing/verification/Rust standards; active lessons; `.planning/REQUIREMENTS.md`, `STATE.md`, `ROADMAP.md`, milestone audit and Phase 12/13/14 context; `RELEASE.md`, `BENCHMARKING.md`, `SAFETY.md`, `TESTING.md`, `UPSTREAM.md`; release, performance, regression and upstream authorities; producer workflows/scripts; release validation/attestation, Phase 13 closure/promotion code; retained `target/phase15-preflight/ci/*/{metadata.json,failed.log}`. [VERIFIED: repository inspection]

External primary source: [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/) for version-qualified security mapping; consulted 2026-09-14. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

## Metadata

- Standard stack: HIGH, existing repository pins and local tool probes. [VERIFIED: sources above]
- Architecture: HIGH for existing identities, paths and validators; MEDIUM for proposed unexecuted repair sequence. [VERIFIED: source inspection]
- Pitfalls: HIGH for actual failure logs and static defects; unresolved Miri root cause is explicitly open. [VERIFIED: retained failures]
- Validity: recheck after any source/workflow change or infrastructure update; availability and run/artifact status are time-sensitive. [VERIFIED: candidate-bound contracts]
- Validation Architecture omitted because workflow.nyquist_validation is explicitly false. No commit made: parent owns checks and git finalization. [VERIFIED: .planning/config.json; delegated task]

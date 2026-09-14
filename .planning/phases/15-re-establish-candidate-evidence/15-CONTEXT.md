---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: 2026-09-14T16:43:10.303691+00:00
---

# Phase 15: Re-establish Candidate Evidence - Context

**Gathered:** 2026-09-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Establish a green full-SHA v1.0 candidate, collect all required retained evidence, validate frozen-source attestation, and repeat milestone acceptance audit. Address PLAT-01, PLAT-05 and DOCS-09 without expanding physics, platform, or release-publication scope.
</domain>

<decisions>
## Implementation Decisions

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
</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Authority and requirements
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` — local instructions and standing authorization.
- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/v1.0-MILESTONE-AUDIT.md` — scope and gaps; older audit failures must be reassessed using current evidence.
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md`, `.planning/phases/13-restore-evidence-and-replay-integrity/13-CONTEXT.md`, `.planning/phases/14-repair-windows-particle-group-invariants/14-CONTEXT.md`, `.planning/phases/14-repair-windows-particle-group-invariants/14-VERIFICATION.md` — inherited closed policies and D2 repair limits.

### Release and producers
- `RELEASE.md`, `BENCHMARKING.md`, `TESTING.md`, `SAFETY.md`, `UPSTREAM.md` — package, evidence and publication boundaries.
- `reference/release/required-evidence.toml`, `reference/release/schema.json`, `reference/performance/manifest.toml`, `reference/performance/policy.json`, `reference/upstream-lock.toml` — closed evidence and provenance inputs.
- `.github/workflows/release.yml`, `.github/workflows/performance.yml`, `scripts/phase12-release-evidence.sh`, `scripts/phase12-release-evidence/aggregation.sh` — actual producer handoff.
- `tools/xtask/src/release.rs`, `tools/xtask/src/release/attestation.rs`, `tools/xtask/src/release/validation.rs`, `tools/xtask/src/phase13_acceptance.rs` — acceptance validators.
</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable assets
- Existing release audit has a closed 19-entry producer registry and typed evidence validation.
- Existing worktree and committed-range attestation validates candidate/tree/manifest/report hashes and a strict changed-path allowlist.
- Existing Phase 12 workflows and shell helpers own producer generation and artifact aggregation.

### Established patterns
- Candidate-scoped immutable outputs, identity published last, independently validated terminal success, Cargo-only consumer isolation.
- Phase 14 recorded cross-platform repair success without promoting it to canonical candidate acceptance.

### Integration points
- Producer workflow downloads -> release aggregation -> retained artifact paths -> source candidate record -> worktree/committed attestation -> generated documentation and milestone audit.
</code-context>

<specifics>
## Specific Ideas

Use staged fresh production unless retained exact-candidate outputs independently qualify. Preserve failed evidence. Resolve unavailable controlled infrastructure without fabricating host identities or relaxing the performance requirement.
</specifics>

<deferred>
## Deferred Ideas

Package publication, tags authorizing a release, new platform promises and physics acceleration are outside this phase request. No pending todos matched this phase.
</deferred>

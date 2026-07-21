---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "29"
subsystem: particle-differential-evidence
tags: [rust, xtask, evidence-validation, supply-chain, archive-security]

requires:
  - phase: 10-28
    provides: Closed five-case Phase 10 corpus and typed 80-leaf evidence contract
provides:
  - One shared fail-closed semantic validator for local D2 and exact-ref D1 evidence
  - Immutable same-run canonical and sanitizer authority validation on the locked D1 stack
  - Adversarial content, identity, path, resource, live-metadata, and archive regressions
affects: [10-30, 10-31, 10-32, phase10-evidence-generation, phase10-compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Preflight bounded regular-file topology before parsing semantic content
    - Recompute payload, proof, manifest, identity, archive, and live authority bindings independently
    - Layer exact-ref authority over the same typed semantic validator used by local evidence

key-files:
  created:
    - tools/xtask/src/phase10_evidence.rs
    - tools/xtask/src/phase10_evidence/authority.rs
    - tools/xtask/src/phase10_evidence/content.rs
    - tools/xtask/src/phase10_evidence/paths.rs
    - tools/xtask/tests/phase10_evidence_cli.rs
    - tools/xtask/tests/phase10_evidence_cli/exact.rs
    - tools/xtask/tests/phase10_evidence_cli/support.rs
  modified:
    - tools/xtask/src/main.rs

key-decisions:
  - "Use one exact five-case typed semantic evaluator for both local and exact-ref evidence; exact-ref adds authority requirements but cannot reinterpret content."
  - "Admit D1 only from one successful same-run canonical and fail-fast sanitizer pair whose live API, extracted identity, toolchain, archive, and full-SHA records all agree."

patterns-established:
  - "Proof topology: each case owns ten canonical non-aliased roles with exact replay/debug-release/minimization equality rules."
  - "Authority topology: repeatable denysets and exact live cardinality reject stale or mixed runs, jobs, and artifacts before promotion."

requirements-completed: [PART-18, TEST-01, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T11:57:53Z

duration: 45m
completed: 2026-07-21
---

# Phase 10 Plan 29: Fail-Closed Evidence Validator Summary

**Local D2 and exact-reference D1 evidence now pass through one bounded 80-leaf semantic validator, while D1 additionally requires one immutable same-run canonical and sanitizer authority pair on the locked Linux toolchain.**

## Performance

- **Duration:** 45m
- **Started:** 2026-07-21T11:13:25Z
- **Completed:** 2026-07-21T11:57:53Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added `cargo xtask phase10-evidence validate --mode local|exact-ref` plus identity-last `validate-content`, with normalized target containment, symlink rejection, bounded depth/count/bytes, and exact regular-file topology.
- Validated the exact five-case Phase 10 manifest through the shared typed 80-leaf contract, canonical per-case proof paths, closed roles, recomputed payload/proof/fixture/manifest/file identities, passing logs, and replay/debug-release/minimization equality.
- Locked exact-ref authority to one successful `workflow_dispatch` run and full SHA, distinct canonical and sanitizer jobs/artifacts, Linux x86_64, Rust 1.97.0, Clang 22.1.8, pinned upstream/protocol/generator, live metadata equality, current expiry, and repeatable run/artifact denysets.
- Inspected archives without extraction for unsafe paths, links, duplicate or case-colliding names, unexpected files, oversized uncompressed entries, compressed-bomb totals, and API/archive digest mismatch.
- Added ten Arrange/Act/Assert CLI tests covering valid local and exact modes plus malformed leaves, policies, proof aliases, replay drift, failed outcomes/logs, paths, resources, identities, denylists, stale authority, toolchain/platform mixing, expiry, live metadata, extracted identity, and archive attacks.

## Task Commits

Each task was committed atomically:

1. **Task 1: Validate bounded semantic evidence identically in both modes** - `205710b` (feat)
2. **Task 2: Enforce immutable same-run D1 authority** - `4b55dc4` (feat)

## Files Created/Modified

- `tools/xtask/src/main.rs` - Routes the new Phase 10 evidence command through typed errors.
- `tools/xtask/src/phase10_evidence.rs` - Parses the closed command surface and invokes shared content plus exact-ref authority validation.
- `tools/xtask/src/phase10_evidence/content.rs` - Owns five-case, 80-leaf, proof-role, log, identity-file, replay, and digest truth.
- `tools/xtask/src/phase10_evidence/authority.rs` - Owns immutable run/job/artifact/toolchain/live-metadata and safe-archive authority.
- `tools/xtask/src/phase10_evidence/paths.rs` - Owns canonical target containment and bounded regular-file traversal/read helpers.
- `tools/xtask/tests/phase10_evidence_cli.rs` - Contains the local/content and exact-ref adversarial assertions.
- `tools/xtask/tests/phase10_evidence_cli/support.rs` - Builds deterministic closed local evidence fixtures.
- `tools/xtask/tests/phase10_evidence_cli/exact.rs` - Builds exact same-run archive and live-metadata fixtures.

## Decisions Made

- Local and exact-ref modes share every semantic/content decision. Exact-ref can only narrow admissibility by adding immutable D1 authority checks.
- Evidence directories contain exactly the manifest-declared regular files, and identities enumerate every non-identity file so metadata repair cannot hide extras, omissions, or substitutions.
- Canonical and sanitizer authority names are fixed now for Plan 10-30 workflow generation: `Phase 10 canonical Linux oracle`, `Phase 10 fail-fast sanitizer`, and `phase10-{canonical|sanitizer}-{run}-{sha}`.
- Archive safety is checked from central-directory metadata before any later workflow extracts a downloaded artifact.

## Threat Model Outcomes

- **T-10-29-01 Spoofing:** Mitigated by full-SHA, exact live cardinality, distinct IDs, extracted identity equality, and repeatable denysets.
- **T-10-29-02 Tampering:** Mitigated by independently recomputed payload, proof, manifest, file, API, and archive digests.
- **T-10-29-03 Repudiation:** Mitigated by categorized typed errors and complete identity manifests.
- **T-10-29-04 Denial of service:** Mitigated by path depth, file count, file byte, archive byte, uncompressed-entry, and total-uncompressed bounds.
- **T-10-29-05 Information disclosure:** Mitigated by fixed argument-based subprocesses and bounded diagnostics without environment output.
- **T-10-29-06 Elevation of privilege:** Mitigated by canonical containment and rejection of links, special files, traversal, absolute paths, duplicates, and case collisions.
- **T-10-29-07 Tampering/rollback:** Validator publication is read-only; every error exits before a promotion output exists.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- A concurrent workspace semantic check briefly held Cargo's build lock. The required commands were allowed to complete normally.
- macOS delayed first launch of newly linked test executables. No command was interrupted; all focused and full suites completed successfully.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-30 can generate identity-last local and CI artifacts directly against the closed manifest/identity shape and fixed job/artifact names.
- Plan 10-31 can bind fresh GitHub API snapshots and downloaded archives to the exact-ref contract without adding a second interpretation layer.
- No blockers remain.

## Self-Check: PASSED

- Confirmed Task 1 commit `205710b` and Task 2 commit `4b55dc4` exist and are atomic.
- Confirmed focused content validation passes 5/5, focused exact-ref validation passes 5/5, and the complete adversarial CLI suite passes 10/10.
- Confirmed the plan key-link verifier passes 1/1 for typed Phase 10 evidence evaluation.
- Confirmed each task commit was preceded by its own exact ordered Rust gate: format, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests.
- Confirmed every touched source/test module remains below 500 lines after the simplification split.
- Confirmed `.planning/config.json`, `.planning/agent-history.json`, and `.planning/current-agent-id.txt` were not staged or committed.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-21*

---
phase: 15-re-establish-candidate-evidence
plan: "05"
subsystem: release
tags: [artifact-integrity, safety, coverage, diagnostics]
requires:
  - phase: 15-04
    provides: Exact package import and preserved original archive identity
provides:
  - Confined complete performance raw-file inventory verification
  - Bounded immutable safety and coverage attempt diagnostics
  - Checksum-bound terminal and run-attempt consistency validation
affects: [15-07, 15-08, 15-11, candidate-requalification]
tech-stack:
  added: []
  patterns: [identity-last, bounded command capture, exact file inventories]
key-files:
  created:
    - scripts/phase12-release-evidence/raw_payloads.sh
    - scripts/phase12-attempt.sh
    - tools/xtask/tests/release_cli/raw_payloads.rs
    - tools/xtask/tests/release_cli/raw_payloads.sh
    - tools/xtask/tests/safety_evidence_contract/attempts.rs
    - tools/xtask/tests/safety_evidence_contract/attempts.sh
    - tools/xtask/tests/safety_evidence_contract/attempt-tool.sh
  modified:
    - scripts/phase12-release-evidence/common.sh
    - scripts/phase12-release-evidence/producer_validation.sh
    - scripts/phase12-release-evidence/identity_validation.sh
    - scripts/phase12-miri.sh
    - scripts/phase12-rust-sanitizers.sh
    - scripts/phase12-coverage.sh
    - .github/workflows/safety.yml
    - .github/workflows/coverage.yml
key-decisions:
  - Keep the existing 21 success artifacts and 19 required entries unchanged.
  - Refuse existing producer destinations and use fresh checkouts for another attempt.
  - Require elapsed deadline evidence before classifying exit 124 or 137 as timeout.
  - Preserve coverage LCOV stdout separately from bounded diagnostic stderr.
requirements-completed: []
requirements-supported: [DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T20:41:50Z"
duration: approximately 27min
completed: 2026-09-14
---

# Phase 15 Plan 05: Artifact Boundaries and Attempt Retention Summary

**Release validation now checks every retained performance digest, while safety and coverage producers preserve bounded command failures and publish checksum-bound success identities last.**

## Tasks and Commits

1. `5eaea95` — Validate normalized confined payload paths, unexpected artifact roots, raw performance hashes, exact inventory and size bounds.
1. `15a376f` — Retain bounded safety/coverage attempt diagnostics, verify terminal identity and command logs, and upload diagnostics on every workflow outcome.

Both implementation tasks are complete. This is prerequisite tooling proof; no new remote Miri, sanitizer or coverage run was dispatched, and no frozen-source release acceptance is claimed.

## Behavior and Evidence

All downloaded members receive lexical/component checks before payload consumption. Absolute relative references, empty/dot/dot-dot components, trailing separators, symlink files/ancestors, special files, unexpected root files, oversized files and wrong artifact names fail closed. The actual aggregate entrypoint rejects a traversal download path before constructing an identity.

Performance validation checks the bound `payload-files.sha256`, every listed file digest, duplicate paths, a 256-file index ceiling and the existing 16 MiB per-file limit. It requires exactly 32 matrix-named raw reports, three producer logs and four metadata files, plus the separately bound envelope files. The paired summary must contain the exact matrix case IDs. NUL-safe enumeration rejects malformed actual paths and does not hide nested files named like root envelopes. Controls cover changed bytes, missing raw files/logs, oversized data, duplicate entries, malformed summary, extra files and nested reserved names. The global registry remains 19 entries and the expected success artifact set remains 21 names.

Miri, Rust ASan and all three coverage modes refuse existing destinations. The shared recorder captures argv, underlying command exit, capture exit, elapsed time, timeout budget, retained bytes, digest and classification. Each command log is capped at 16 MiB; a capped log cannot publish success. Timeout commands have a ten-second termination grace. The Miri math modes, 900-second ordinary budget, 3600-second group budget, default isolation and unchanged group cases/seeds remain intact; ASan retains 1200 seconds and coverage retains 1800 seconds. Coverage exports LCOV stdout separately, retaining bounded diagnostics without contaminating the report.

An EXIT recorder preserves terminal status and checksums on failure. Failed identity writes are retained under diagnostics rather than left as success identities. A successful identity binds the diagnostic index and run attempt. Release validation checks every diagnostic digest, command log hash/size, successful command status, terminal source/run/attempt/workflow/job consistency and complete safety case names. Immediate tool exits 124 and 137 are classified as command failures; a real one-second timeout probe demonstrates timeout exit 124 with elapsed deadline evidence.

Separate `always()` uploads retain only the bounded diagnostic directories, with run/attempt artifact names. The original success uploads remain success-gated and retain their names. Successful safety/coverage artifacts also include the diagnostic directory, allowing the release consumer to verify it without increasing the 21-artifact set.

## Verification and Preserved Failures

- Task 1 RED reproduced acceptance of an unexpected root file in `target/phase15-plan05/task1-attempt01/red-tests.log`. Task 2 RED reproduced absent terminal diagnostics in `target/phase15-plan05/task2-attempt01/red.log`. RED stages were not committed because repository precommit rules require passing checks.
- Final focused suites passed: coverage workflow **7**, release attestation **6**, release CLI **27**, safety contract **20**. The five producer variants use controlled test-only tool outcomes and execute their actual checked-in workflow verification blocks. Each variant rejects failed/124/137/oversized commands, preserves failed checksums through a rejected retry, and succeeds in a fresh fixture. Stale run-attempt envelopes fail downstream consistency validation.
- Final retained fixture root: `target/phase15-attempt-tests/11363-1/`. These are explicitly synthetic orchestration controls, not actual Miri/ASan/coverage execution or provider acceptance evidence. The real deadline probe is under `real-timeout/` in that root.
- Final ordered gates and focused suites: `target/phase15-plan05/task2-attempt01/storage-recovery-gates.log`. Before each implementation commit, fmt, all-target/all-feature Clippy, build and tests passed in order. Explicit xtask Clippy also passed. Bright Builds, Markdown check, shfmt, ShellCheck through entrypoints, actionlint and diff checks passed.
- Gates used the reviewed Linux ARM64 Docker image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, existing shared Rust/Cargo volumes and `/workspace` mount. These local checks remain supplemental to canonical/native evidence.
- A later gate run encountered host APFS exhaustion while the real package handoff test wrote its output; `review-final-gates.log` preserves that failure. A subsequent real package attempt also failed compilation with OS error 28. Only disposable `publish-dry-run/debug` compilation caches from eleven Plan 05 test fixtures were removed; archives, identities, sources and logs remain. Fresh gates passed with `CARGO_INCREMENTAL=0`. The first Task 2 commit attempt could not create `.git/index.lock`; after further cache reclamation the same verified source committed successfully. No failed record was relabeled passing.

## Scope, Decisions and Remaining Limits

The plan records cohesive shell/test helpers and small adaptations to existing Miri/coverage subprocess fixtures. The simplification pass reused the existing producer scripts, centralized only command capture/terminal recording, and retained typed release authority. Parent review found and corrected nested envelope-name exclusion and premature timeout classification. No blocking stubs or new security surface outside the planned artifact/file boundaries remain.

The consumer checks run-attempt consistency inside retained evidence. It does not independently compare that attempt with GitHub's latest run attempt: the aggregate interface still receives producer run IDs. Actual provider/attempt acquisition and independent same-C qualification remain later phase work. Complete internally consistent older attempts are not claimed to be rejected merely because a newer attempt exists.

AGENTS.md standing authorization dated 2026-09-13, Bright Builds sidecar/overrides, active lessons and architecture/testing/verification/Rust standards governed execution. No package publication, acceptance waiver or public readiness change occurred. The parent orchestrator owns STATE.md, ROADMAP.md and REQUIREMENTS.md; no global requirement was completed here.

## Self-Check: PASSED

Both implementation commits resolve; all seven created helper/test files exist; RED, final passing gate logs and the final five-variant fixture root exist. The summary carries the required lifecycle and actual generation timestamp. Parent-owned state and requirements were not staged or modified.

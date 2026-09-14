---
phase: 15-re-establish-candidate-evidence
plan: "06"
subsystem: release
tags: [dispatch, retention, attestation, documentation, ci]
requires:
  - phase: 15-05
    provides: Confined producer payload and attempt validation
provides:
  - Authoritative dispatch receipt binding and immutable producer collections
  - Fresh provider-attempt and archive-digest validation with confined restoration
  - Explicit C/A documentation readiness with committed-record byte binding
  - CI restoration of the exact attested release artifact before the full audit
affects: [15-07, 15-08, 15-09, 15-10, 15-11, 15-12]
tech-stack:
  added: []
  patterns: [Python standard-library adapters, authoritative dispatch receipts, immutable attempts, explicit attestation projection]
key-files:
  created:
    - scripts/phase15-candidate-evidence.sh
    - scripts/phase15-candidate-evidence/dispatch.py
    - scripts/phase15-candidate-evidence/retention.py
    - scripts/phase15-candidate-evidence/payloads.py
    - scripts/phase15-candidate-evidence/docs_ci.py
    - scripts/phase15-docs-check.sh
    - tools/xtask/src/docs/contracts/readiness.rs
    - tools/xtask/tests/phase15_candidate_evidence.rs
    - tools/xtask/tests/release_attestation/fixture.rs
    - tools/xtask/tests/support/maturity.rs
  modified:
    - tools/xtask/src/release/attestation.rs
    - tools/xtask/src/docs.rs
    - .github/workflows/ci.yml
    - scripts/phase13-1-gap-verification-manifest.json
    - RELEASE.md
key-decisions:
  - A run can be attributed only through the authoritative URL retained from the dispatch response; temporal uniqueness is insufficient.
  - Keep the existing 21-artifact producer boundary, 19-entry release registry, and nine-path attestation allowlist unchanged.
  - Validate retained producers in a clean C checkout; validate public projection using explicit A from later D.
  - CI restores the exact release artifact before invoking the existing full audit, and does not cache retained evidence.
requirements-completed: []
requirements-supported: [DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-09-14T16-41-28
generated_at: "2026-09-14T21:24:49Z"
duration: approximately 35min
completed: 2026-09-14
---

# Phase 15 Plan 06: Dispatch, Retention and Explicit Readiness Summary

**Candidate tooling binds dispatches to authoritative receipts, retains and restores exact provider archives, and permits ready documentation only after explicit committed C/A evidence passes the existing release audit.**

## Tasks and Commits

1. `72cd27c` — Add bounded dispatch/reconciliation, producer collection, archive inventories, fresh provider checks, confined restoration and behavioral controls.
1. `90d4d49` — Add explicit attestation-backed readiness, record-byte and history validation, release-artifact restoration in CI, isolated maturity fixtures and C/A/D regressions.

Both implementation tasks are complete: 32 unique implementation/support files changed. The parent orchestrator owns the combined GSD metadata commit and global STATE/ROADMAP/requirement updates. This summary and the bounded Plan 06 amendment are handed back unstaged for that commit.

No production candidate was frozen, no workflow was dispatched, and no package was released by this plan. No production ready records were manufactured. Public documentation remains non-ready; DOCS-09 and global phase acceptance remain pending.

## Behavior and Decision Coverage

- D-01/D-03: source and CI readiness prerequisites are implemented before freeze. The nine-path attestation allowlist is byte-identical to its prior definition. Committed validation checks every changed path in C..A history, including reverted changes, and binds the three checked records to their exact blobs at A. Later D cannot substitute for A or C.
- D-02/D-06: the producer validator and 19-entry release auditor remain authoritative. The retention helper restores all 21 producer artifact directories and calls the existing semantic validator. CI restores the release envelope/package artifact and invokes the full existing docs/attestation audit.
- D-04/D-05: dispatch persists intent before the provider effect, validates exact repository/workflow/ref/SHA and the authoritative returned run URL, and refuses missing/ambiguous receipts without redispatch. Collection binds explicit run and current attempt, required jobs, original ZIP bytes, provider digests and raw payload inventories. Existing producer destinations and restoration destinations are never overwritten.
- D-07/D-08: standing authorization in AGENTS.md dated 2026-09-13 is the authority for implementation and verification. Controlled performance infrastructure remains unresolved in the parent phase work; no host identity, approval timestamp or waiver was invented. Docker was not restarted or otherwise changed.
- D-09: default docs checks require non-ready copy. Explicit A requires accepted retained evidence and matching standalone C/A/status markers. CI obtains A from an explicit marker, verifies A/C record bytes and ancestry, fetches only the matching provider-bound release artifact, and runs the full validator. Its Cargo cache excludes restored evidence.

## Verification

Every source commit followed the required fail-fast order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

Final Task 2 sequence returned exit 0, including:

- Additional `cargo clippy -p xtask --all-targets --all-features -- -D warnings`.
- `cargo test -p xtask --test docs_contract --test release_attestation`: **43 docs + 12 attestation tests passed**.
- `cargo test -p xtask --test phase15_candidate_evidence`: **2 Cargo runners passed**, executing **22 dispatch/retention + 16 CI restoration controls**.
- Four affected verification-manifest tests passed after adding the new target with stable ID/log index 074; existing IDs and historical records were preserved.
- The actual `bash scripts/phase15-docs-check.sh` default path passed against current non-ready documents.
- Bright Builds checker: **975 files, zero findings**. Configured Markdown, ShellCheck, shfmt, actionlint and `git diff --check` passed.
- Independent parent review found no remaining source issue after dispatch attribution and CI integration corrections. Record: `target/phase15-plan06/independent-review.md`.

Logs are retained under `target/phase15-plan06/`: Task 1 final evidence uses `task1-final-*`; the final successful Task 2 sequence uses `task2-commit-*`. Earlier failed logs are preserved. The first retained RED command began at 2026-09-14T20:52:49Z.

The macOS native verification profile used a dedicated `target/phase15-native-verification`, `CARGO_INCREMENTAL=0`, and development/test debug information disabled to avoid the unhealthy old cache and reduce disk use. Optimization and debug assertions were unchanged. GNU Bash/find/coreutils supplied the documented shell boundary. This is local verification, not canonical Linux x64 or native Windows acceptance evidence.

### Test Evidence Boundaries

Task 1's exact requested Cargo RED command failed because the entrypoint was absent, before that entrypoint was created. Subsequent behavioral controls exercise fake provider responses through production orchestration and real bounded ZIP handling. Task 2's old CLI rejected the new explicit option, but its initial queued Cargo run compiled implementation edits; strict suite-level RED-before-GREEN ordering was not established and is not claimed. Passing final semantic regressions are retained.

The Rust C/A/D fixtures independently run the real 19-entry release audit with small explicit test evidence, restore missing paths at a fresh clone, and validate from later D. CI restoration fixtures use real temporary Git C/A/D records and the exact release ZIP layout, fake only the provider and final Cargo handoff, and assert the explicit validator arguments. The real final validator is separately covered by the Rust fixtures. These fixtures are test evidence, not accepted production records.

## Deviations and Repairs

1. **[Rule 2 — Missing critical functionality] Dispatch attribution.** Parent review showed a unique same-ref/SHA/time match could belong to another caller. Reconciliation now requires the persisted authoritative response URL and validates that exact run. Missing/duplicate URL and unrelated unique-run controls protect the boundary.
1. **[Rule 2 — Missing critical functionality] Later-D CI integration.** Bare docs checks and fixtures inheriting current maturity would fail after projection. Added explicit artifact restoration in CI, required history/actions read access, a cache key excluding retained payloads, and explicit non-ready fixture input. No readiness criterion was relaxed.
1. **[Rule 3 — Blocking integration] Closed test inventory.** Registered the new xtask test in the current manifest, preserving existing IDs and log paths. An initial broad suite passed 67 controls but failed two registration count/order assertions; both were repaired and the four affected manifest controls passed.
1. **[Rule 1 — Fixture bug] Preserve status prose.** Final tests caught removal of required attestation prose sharing a status line. The normalizer now removes only the status prefix, with a regression for both ready and non-ready prefixes. All 55 final docs/attestation tests pass.
1. **Verification environment.** Cold native process launches caused bounded test timeouts; stable fixture executables and warmed dedicated builds resolved them without changing production acceptance. A GNU-tool PATH selected a plugin-free Homebrew mdformat and reported eleven unchanged documents. The failed run is retained; the configured Python 3.13/mdformat 1.0.0 plus GFM/frontmatter environment passed without rewriting those documents. An intermediate ancillary-success report was premature and corrected before any commit.

Python standard-library modules and small Rust support modules are bounded implementation details beyond the original file list. The amended plan records their scope. No dependencies were added. Stub and threat-surface scans found no production placeholders or additional unmodeled trust boundary; all new provider/archive access is covered by the plan's workflow-input/downloaded-bytes boundary.

## Remaining Phase Work

Finish the parent's remaining pre-freeze promotion prerequisite repair, then obtain actual final-C provider runs, complete retained raw evidence, accepted release aggregation, direct C-to-A attestation, later D projection and milestone acceptance. Controlled performance access remains a parent-owned dependency. Missing/expired artifacts, ambiguous dispatch receipts, invalid identities and source changes after freeze remain failures requiring honest recovery or requalification.

## Self-Check: PASSED

All 32 implementation/support files exist, both task commits resolve, and lifecycle validation passes with the expected yolo lifecycle ID. Summary and plan amendment remain unstaged for the parent-owned combined metadata commit; global requirement completion is intentionally unchanged.

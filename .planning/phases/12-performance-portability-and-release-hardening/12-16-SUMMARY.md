---
phase: 12-performance-portability-and-release-hardening
plan: "16"
subsystem: release-engineering
tags: [github-actions, release-audit, artifact-provenance, supply-chain]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: Strict 19-entry commit-bound release manifest and pure fail-closed auditor from Plan 12-15
provides:
  - Check-first manual release-candidate workflow pinned to one detached full commit SHA
  - Exact 21-artifact cross-run aggregation with name, cardinality, producer, payload, and candidate validation
  - Untracked 19-entry release manifest, machine audit report, and identity-last digest artifact
affects: [release-readiness, publication, phase-12-sign-off]
tech-stack:
  added: []
  patterns: [cheap-evidence-before-download, exact-cross-run-artifact-join, identity-last-readiness]
key-files:
  created:
    - .github/workflows/release.yml
    - scripts/phase12-release-evidence.sh
  modified:
    - reference/release/required-evidence.toml
    - tools/xtask/src/release/validation.rs
    - tools/xtask/tests/release_cli.rs
key-decisions:
  - "Fresh package, docs, notices, corpus, and compatibility records truthfully name release.yml/release-candidate as their producer."
  - "Producer run IDs are positive canonical decimal GitHub Actions run identifiers, not arbitrary labels."
patterns-established:
  - "Release aggregation validates all cheap typed records before downloading expensive evidence."
  - "Readiness upload is impossible until the single pure audit succeeds and its digest identity is written last."
requirements-completed: [COMP-10, API-11, API-12, TEST-05, TEST-06, TEST-07, TEST-08, PERF-04, PERF-05, PERF-06, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06, DOCS-01, DOCS-04, DOCS-06, DOCS-07, DOCS-08, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T03:01:36Z
duration: 15min
completed: 2026-07-23
---

# Phase 12 Plan 16: Release Evidence Aggregation Summary

**A detached full-SHA workflow now joins 21 independently produced artifacts into one validated 19-entry release manifest without rerunning expensive suites or modifying tracked readiness.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-24T02:46:42Z
- **Completed:** 2026-07-24T03:01:36Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Added a non-cancelling manual workflow whose inexpensive package, publication dry-run, policy, rustdoc, documentation, corpus, compatibility, and generation checks all complete before artifact downloads.
- Added a confined Bash constructor that rejects missing, extra, substituted, wrong-run, wrong-candidate, wrong-job, hash-drifted, or finding-bearing producer artifacts before manifest construction.
- Joined the exact package hash across the local package, independent package artifact, MSRV, durable platforms, and conditional platform disposition.
- Kept the final readiness authority pure: the workflow invokes `cargo xtask release audit` exactly once, then writes and uploads only untracked manifest/report/package digests with identity last.
- Hardened release manifest producer run IDs to canonical positive decimal GitHub Actions identifiers and extended exhaustive negative coverage.

## Task Commits

1. **Task 1: Aggregate independent same-commit artifacts and reject every incomplete set** - `ca09e9b` (feat)

## Files Created/Modified

- `.github/workflows/release.yml` - Full-SHA manual aggregation workflow with exact cross-run downloads and post-audit upload.
- `scripts/phase12-release-evidence.sh` - Check-first evidence emitter, artifact validator, manifest constructor, and identity-last publisher.
- `reference/release/required-evidence.toml` - Truthful producer identities for the five inexpensive records created by the release job.
- `tools/xtask/src/release/validation.rs` - Positive canonical decimal producer run-ID validation.
- `tools/xtask/tests/release_cli.rs` - Workflow-order, forbidden-producer, tracked-write, artifact-name, and producer-substitution negatives.

## Decisions Made

- The release job owns only the five evidence records it actually creates: package, docs/rustdoc, notices/licenses, corpus closure, and compatibility closure.
- Platform, oracle, safety, fuzz, regressions, coverage, and performance remain bound to their independent reviewed workflow run IDs.
- GitHub artifact names and run-scoped downloads are validated as a closed 21-directory set before any producer payload is opened.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Corrected inexpensive evidence producer identities**

- **Found during:** Task 1
- **Issue:** The existing registry named `platform.yml/package` and `ci.yml/quality` for records Plan 12-16 requires the release job to create freshly. Reusing those labels would falsely attribute provenance.
- **Fix:** Changed only package, docs, notices, corpus closure, and compatibility closure to `release.yml/release-candidate`; all expensive independent producers remain unchanged.
- **Files modified:** `reference/release/required-evidence.toml`
- **Verification:** All 14 release CLI tests pass, including wrong workflow/job/run substitution negatives.
- **Committed in:** `ca09e9b`

**2. [Rule 2 - Missing Critical] Restricted producer run IDs to real GitHub run identity syntax**

- **Found during:** Task 1
- **Issue:** The Plan 12-15 validator accepted arbitrary identifier strings such as `run-1`, weakening exact run provenance.
- **Fix:** Require positive canonical decimal run IDs of at most 20 digits.
- **Files modified:** `tools/xtask/src/release/validation.rs`, `tools/xtask/tests/release_cli.rs`
- **Verification:** Malformed alphanumeric run IDs fail closed; the exhaustive release suite passes.
- **Committed in:** `ca09e9b`

**Total deviations:** 2 auto-fixed (2 missing critical)

**Impact on plan:** Both corrections make producer attribution and exact run identity truthful without widening the release workflow or rerunning any producer.

## Verification

- `bash -n scripts/phase12-release-evidence.sh`
- `shellcheck scripts/phase12-release-evidence.sh`
- `actionlint .github/workflows/release.yml`
- `bash scripts/phase12-release-evidence.sh check`
- `cargo test -p xtask --test release_cli` — 14 passed
- Forbidden expensive-producer and tracked-readiness-write source scans — clean
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `git diff --check`

## Known Stubs

None.

## Threat Flags

| Flag | File | Description |
| --- | --- | --- |
| threat_flag: cross-run-artifact-download | `.github/workflows/release.yml` | Manual workflow reads artifacts from seven reviewed GitHub Actions runs using read-only `actions: read`; exact run/name/cardinality checks confine the trust boundary. |

## Issues Encountered

- The constructor contract check initially queried the JSON Schema document as an instance and used over-escaped AWK patterns. The checks were corrected and rerun under fail-fast shell semantics before commit.

## User Setup Required

None - no secrets or external service configuration required.

## Next Phase Readiness

- Plan 12-21 can derive or attest tracked release-facing documentation from a completed source-candidate audit without changing the audited candidate.
- Plan 12-22 can run final verification against the machine report and identity-last digests.
- No blockers remain for the next Phase 12 plan.

## Self-Check: PASSED

- All five implementation files and this summary exist.
- Task commit `ca09e9b` is the current committed implementation.
- Summary frontmatter has exactly one opening and one closing delimiter.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*

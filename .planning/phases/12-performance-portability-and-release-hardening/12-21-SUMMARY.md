---
phase: 12-performance-portability-and-release-hardening
plan: "21"
subsystem: release-engineering
tags: [release-attestation, frozen-source, artifact-retention, fail-closed]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: Commit-bound release audit and release-candidate aggregation from Plans 12-15 and 12-16
provides:
  - Complete future release bundle retention for every manifest-referenced artifact and package byte
  - Frozen-source worktree and committed-range attestation validators
  - Explicitly unattested current state until real external producer runs exist
affects: [release-readiness, publication, phase-12-sign-off]
tech-stack:
  added: []
  patterns: [frozen-source-later-attestation, closed-diff-allowlist, independent-git-tree-hash]
key-files:
  created:
    - tools/xtask/src/release/attestation.rs
    - tools/xtask/tests/release_attestation.rs
  modified:
    - .github/workflows/release.yml
    - tools/xtask/src/release.rs
    - tools/xtask/src/main.rs
    - tools/xtask/tests/release_cli.rs
key-decisions:
  - "Readiness remains absent until a real full-SHA release-candidate run produces the complete retained evidence bundle."
  - "The source candidate is supplied only by a strict source record and independently checked against the Git tree, manifest, report, audit, and allowlisted diff."
patterns-established:
  - "Future release artifacts retain all bytes reopened by the pure release audit, not only digest metadata."
  - "Attestation validates both proposed worktree paths and the later committed source-to-attestation range without substituting current HEAD for the frozen source."
requirements-completed: [COMP-10, DOCS-09, PERF-04, PERF-05, PERF-06, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T03:21:28Z
duration: 16min
completed: 2026-07-23
---

# Phase 12 Plan 21: Fail-Closed Release Attestation Summary

**A complete future audit bundle and frozen-source validator now support truthful later attestation while the current branch remains explicitly unattested.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-24T03:05:04Z
- **Completed:** 2026-07-24T03:21:28Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Repaired the release-candidate upload to retain all 19 evidence envelopes and the exact package archive in addition to the manifest, report, and identity records.
- Added closed `validate-worktree` and committed `validate` commands for frozen-source attestation.
- Bound attestation to a strict ready source record, full commit SHA, independently recomputed Git tree hash, manifest/report byte hashes, exact candidate joins, the existing pure audit, and byte-exact regenerated report.
- Rejected non-attestation staged, unstaged, untracked, and committed paths through one closed allowlist and required the frozen source to be an ancestor.
- Added negative coverage for missing and malformed records, `ready: false`, mixed current-HEAD identity, source-tree drift, manifest/report hash drift, missing payloads, payload hash tampering, and non-attestation paths.
- Preserved the honest current state: no source-candidate, candidate-manifest, or audit-report readiness record was created.

## Task Commits

1. **Task 1: Retain the complete future audit bundle and implement fail-closed attestation validation** - `f4fc5fe` (feat)

## Files Created/Modified

- `.github/workflows/release.yml` - Retains every audit-reopened evidence envelope and package byte.
- `tools/xtask/src/release/attestation.rs` - Strict frozen-source input, Git identity, diff, audit, and report validator.
- `tools/xtask/tests/release_attestation.rs` - Fail-closed source, candidate, tree, and record-hash tests.
- `tools/xtask/src/release.rs` - Closed attestation command dispatch and usage.
- `tools/xtask/src/main.rs` - Release command description includes audit and attestation.
- `tools/xtask/tests/release_cli.rs` - Complete-retention, missing-payload, and payload-hash regression tests.
- `12-21-PLAN.md`, `12-22-PLAN.md` - Truthful contracts that defer readiness until external run-bound evidence exists.

## Decisions Made

- A local green build is not release attestation evidence; only a completed full-SHA workflow bundle can supply the future records.
- The later attestation commit may change only the closed implementation, test, workflow, and three readiness-record paths.
- Report validation is byte-exact after independently rerunning the strict manifest audit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Retained the complete auditable release payload**

- **Found during:** Pre-execution evidence inspection
- **Issue:** Plan 12-16 uploaded four metadata files, but the audit reopens all 19 artifact envelopes and the exact package archive. A downloaded bundle could not be independently re-audited.
- **Fix:** Added every evidence envelope and `package/liquidfun.crate` to the validated release artifact.
- **Files modified:** `.github/workflows/release.yml`, `tools/xtask/tests/release_cli.rs`
- **Verification:** Static retention regression, actionlint, focused release tests, and the full ordered Rust gate pass.
- **Committed in:** `f4fc5fe`

**2. [Rule 2 - Missing Critical] Replaced unavailable readiness construction with fail-closed attestation tooling**

- **Found during:** Pre-execution evidence inspection
- **Issue:** The branch was not on the remote, the release workflow had never run, and no real candidate manifest/report/payload set existed. Creating `ready: true` records would manufacture provenance.
- **Fix:** Revised Plans 12-21 and 12-22 to preserve the frozen-source/later-attestation design, implemented the validator, and left all readiness records absent.
- **Files modified:** `12-21-PLAN.md`, `12-22-PLAN.md`, release attestation implementation and tests
- **Verification:** Source-record absence remains explicit; negative tests fail closed across every available boundary.
- **Committed in:** `f4fc5fe`

**Total deviations:** 2 auto-fixed (2 missing critical)

**Impact on plan:** The release path is now executable when real run-bound evidence exists, without falsely claiming that the external lifecycle has already occurred.

## Verification

- `actionlint .github/workflows/release.yml`
- `cargo test -p xtask --test release_attestation` - 6 passed
- `cargo test -p xtask --test release_cli` - 16 passed
- `cargo xtask release --help` - exposes both closed attestation commands
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `git diff --check`

All Cargo commands used `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and `CARGO_BUILD_JOBS=1`.

## Known Stubs

None.

## Issues Encountered

- Real GitHub producer evidence was intentionally unavailable because lifecycle verification must pass before any push or workflow dispatch. The implementation therefore stops at truthful fail-closed tooling.

## User Setup Required

None for the tooling. A future release operator must supply one completed full-SHA release-candidate bundle before creating readiness records.

## Next Phase Readiness

- Plan 12-22 can document the explicit non-ready state and exact missing run-bound inputs.
- A future release lifecycle can download the complete bundle, validate proposed records, commit the attestation, validate the committed range, and only then project public readiness.

## Self-Check: PASSED

- All six implementation files, two revised plans, and this summary exist.
- Task commit `f4fc5fe` exists.
- No tracked readiness record exists.
- Summary frontmatter has exactly one opening and one closing delimiter.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*

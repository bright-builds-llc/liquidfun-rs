---
phase: 12-performance-portability-and-release-hardening
plan: "15"
subsystem: release
tags: [rust, release-audit, evidence, sha256, supply-chain]
requires:
  - phase: 12-23
    provides: package artifact and platform evidence contracts
  - phase: 12-09
    provides: canonical differential evidence
  - phase: 12-24
    provides: safety, regression, fuzz, and coverage evidence
  - phase: 12-11
    provides: performance policy and report authority
  - phase: 12-13
    provides: public documentation and release checklist
  - phase: 12-14
    provides: terminal corpus and compatibility closure
provides:
  - pure commit-bound release manifest audit
  - closed 19-identity evidence registry and JSON schema
  - stable human and JSON readiness reports
  - exhaustive negative release-audit test matrix
affects: [12-16, release-workflow, publication]
tech-stack:
  added: []
  patterns:
    - closed evidence-kind and kind/target identity joins
    - bounded repository-confined artifact validation
    - producer-free aggregation of existing evidence
key-files:
  created:
    - tools/xtask/src/release.rs
    - tools/xtask/src/release/domain.rs
    - tools/xtask/src/release/validation.rs
    - tools/xtask/src/release/report.rs
    - tools/xtask/tests/release_cli.rs
    - reference/release/schema.json
    - reference/release/required-evidence.toml
  modified:
    - tools/xtask/src/main.rs
    - tools/xtask/src/inventory/validation.rs
    - justfile
key-decisions:
  - "Release readiness requires exactly 19 kind/target identities from a closed 16-kind enum, with four separate durable platform records."
  - "The audit reads only repository-confined ordinary files and never invokes producers, subprocesses, workflows, or network clients."
  - "Coverage and performance evidence remain explicitly non-parity authorities, while conditional macOS support must match the tracked supported-or-unsupported disposition."
patterns-established:
  - "Release evidence fan-out: package, MSRV, and every platform record must share one content-addressed package archive identity."
  - "Two-level integrity: every manifest record revalidates artifact bytes and the canonical serialized claims payload."
requirements-completed: [COMP-10, DOCS-09, PERF-04, PERF-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T02:44:07Z
duration: 18min
completed: 2026-07-23
---

# Phase 12 Plan 15: Commit-Bound Release Audit Summary

**A producer-free release gate now accepts only one complete 19-record evidence set whose candidate, workflow/job, toolchain, target, artifact, payload, review, and status identities all agree.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-24T02:26:32Z
- **Completed:** 2026-07-24T02:44:07Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- Added `cargo xtask release audit --manifest PATH --candidate COMMIT --output human|json` with 4 MiB/256-item manifest bounds, confined non-symlink reads, strict schemas, and stable categorized failures.
- Added the closed 19-entry required evidence registry spanning package/MSRV, four durable platforms, conditional macOS disposition, parity, safety, sanitizer, fuzz, regressions, coverage, performance, docs/notices, corpus, and compatibility closure.
- Joined one exact package archive across package/MSRV/platform evidence and rejected stale conditional support, unsafe/advisory weakening, parity-authority misuse, incomplete corpus, compatibility gaps, mixed commits, hash substitution, duplicates, and unreviewed records.
- Added 11 focused tests, including independent removal of every required kind/target identity, and verified that the implementation contains no process, workflow, benchmark, fuzz, sanitizer, or network invocation seam.

## Task Commits

Each task was committed atomically:

1. **Task 1: Parse and validate one complete commit-bound release evidence set** - `aa77444` (feat)

## Files Created/Modified

- `tools/xtask/src/release.rs` - Closed CLI and stable categorized error boundary.
- `tools/xtask/src/release/domain.rs` - Closed evidence kinds, identities, untrusted records, and validated readiness types.
- `tools/xtask/src/release/validation.rs` - Bounded file reads, exact joins, authority checks, and pure readiness decision.
- `tools/xtask/src/release/report.rs` - Deterministic human and JSON ready reports.
- `tools/xtask/tests/release_cli.rs` - Complete-set fixture and exhaustive negative acceptance matrix.
- `reference/release/schema.json` - Closed manifest schema with exact reviewed/passed states and a 256-item maximum.
- `reference/release/required-evidence.toml` - Exact producer/job/toolchain allowlist for all 19 kind/target identities.
- `tools/xtask/src/main.rs` - Registered the release command.
- `justfile` - Added the thin `release-audit` recipe.
- `tools/xtask/src/inventory/validation.rs` - Preserved exact `.toml` tolerance matching through a Clippy-clean path-extension check.

## Decisions Made

- Model repeated durable-platform evidence as four distinct `platform` kind/target identities rather than four more enum variants; completeness remains exact while the domain stays compact.
- Require artifact paths and the nested package archive path to be normalized repository-relative paths so the audit cannot follow absolute paths, traversal, or symlink substitution.
- Accept the tracked conditional macOS unsupported disposition when native evidence is absent; if tracked native evidence is present, require an exact 90-day non-expired timestamp pair.
- Keep performance at `unprofiled_wall_clock` / `workload_only` with `no_generalized_performance_claim`, and require both Rust and C++ coverage to retain `parity_authority=false`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced a case-sensitive tolerance suffix check with an exact path-extension check**

- **Found during:** Task 1 strict Clippy verification
- **Issue:** The existing `reference.ends_with(".toml")` check triggered Rust 1.97's denied `case_sensitive_file_extension_comparisons` lint and blocked the mandated repository-wide gate.
- **Fix:** Preserved the required prefix and exact lowercase `toml` behavior through `Path::extension`.
- **Files modified:** `tools/xtask/src/inventory/validation.rs`
- **Verification:** The exact ordered fmt, Clippy, build, and test gates passed.
- **Committed in:** `aa77444`

**Total deviations:** 1 auto-fixed (1 blocking)

**Impact on plan:** The minimal lint-compatible rewrite preserved inventory behavior and introduced no release-audit scope expansion.

## Issues Encountered

- The optional system Python environment did not provide the `jsonschema` module. The checked-in schema was still parsed and its closed enum, additional-properties rules, reviewed status, and item bound were asserted by the Rust acceptance suite; no dependency or network access was added.

## User Setup Required

None - the audit is local and requires no credentials or external service configuration.

## Next Phase Readiness

- Plan 12-16 can aggregate workflow artifacts into the exact manifest contract and invoke the pure audit without rerunning expensive producers.
- The current conditional macOS x86_64 policy remains an explicit reviewed unsupported disposition until fresh native evidence is tracked.

## Self-Check: PASSED

- All seven created release implementation, test, and contract files exist.
- Task commit `aa77444` exists in repository history.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*

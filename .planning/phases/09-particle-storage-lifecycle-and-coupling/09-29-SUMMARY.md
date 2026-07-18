---
phase: 09-particle-storage-lifecycle-and-coupling
plan: 29
subsystem: testing
tags: [phase9, compatibility, differential-testing, artifact-integrity, asan, ubsan]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    provides: portable exact-ref validation and replacement canonical/sanitizer evidence from plan 09-28
provides:
  - independently revalidated canonical and sanitizer authority for run 29652578231
  - fail-closed compatibility promotion for the four supported Phase 9 rows
  - executable rejection of historical, failed, mixed-run, incomplete, and Phase 10 evidence
  - complete Rust, oracle, evidence, schema, dependency, documentation, and ASVS L1 closure matrix
affects: [phase-09-verification, phase-10-particle-groups-and-solvers, compatibility-ledger]
tech-stack:
  added: []
  patterns:
    - exact live GitHub metadata and archive digests must agree before compatibility promotion
    - inventory validation rejects every historical authority marker and every deferred Phase 10 row
key-files:
  created:
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-29-SUMMARY.md
  modified:
    - TESTING.md
    - reference/compatibility.json
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
key-decisions:
  - "Only run 29652578231 at approved SHA 22b31c0e1be8896df622b1decd58ba2853a60b04 may authorize the four scoped Phase 9 promotion rows."
  - "Runs 29439515367, 29583793056, and 29625083184 and artifact 8423580554 are permanently rejected as Phase 9 promotion authority."
  - "Phase 10 group, topology, pair, triad, source-area, solver, and stable-ID rotation behavior remains not evidenced."
patterns-established:
  - "Promotion references include the run, both archive API digests, both trace and manifest digests, the shared semantic digest, exact commit, recovery summary, and TESTING anchor."
  - "Generated compatibility output is verified by two byte-identical generation passes and is never edited manually."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T17:21:53Z
duration: 23min
completed: 2026-07-18
---

# Phase 09 Plan 29: Replacement Evidence Promotion Summary

**Fresh canonical and sanitizer artifacts now authorize only the supported Phase 9 particle lifecycle and coupling claims through digest-bound, fail-closed compatibility evidence**

## Performance

- **Duration:** 23 min
- **Started:** 2026-07-18T16:58:43Z
- **Completed:** 2026-07-18T17:21:53Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Independently re-queried run, job, and artifact metadata, redownloaded both archives, checked archive safety before extraction, and revalidated 7 cases with exactly 58 semantic bindings.
- Promoted only the two particle public API rows and the particle storage/lifecycle and contact/coupling subsystem rows to replacement run 29652578231.
- Added 21 inventory CLI regressions that reject every prior or failed authority marker, incomplete semantic evidence, mixed artifacts, and deferred Phase 10 claims.
- Passed the complete Rust, debug/release oracle, ASan/UBSan, local/exact-ref evidence, provenance, inventory, dependency, schema, workflow, Markdown, upstream read-only, and diff matrix.

## Task Commits

Each task's durable changes were committed atomically:

1. **Task 1: Independently redownload and revalidate the new approved artifacts** - `9633f52` (docs)
2. **Task 2: Promote only fresh semantic authority and regenerate reports** - `0e9d73a` (feat)
3. **Task 3: Run final Rust, oracle, evidence, schema, security, and diff gates** - verification-only work completed before `0e9d73a`; no separate file delta

## Files Created/Modified

- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-29-SUMMARY.md` - Records the promotion authority, verification matrix, security review, and lifecycle outcome.
- `TESTING.md` - Records the replacement approved run and current Phase 9 maturity boundary.
- `reference/compatibility.json` - Binds only four supported rows to the replacement semantic and platform evidence.
- `tools/xtask/src/inventory/validation.rs` - Enforces the exact replacement authority, complete semantic reference set, denied historical markers, and deferred rows.
- `tools/xtask/tests/inventory_cli.rs` - Proves fail-closed behavior for prior, failed, mixed, incomplete, and Phase 10 evidence.
- `COMPATIBILITY.md` - Generated twice with byte identity and intentionally left without a tracked manual change.

## Closure Matrix

- Exact Rust sequence passed before each commit: formatting, Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused differential suites passed: `phase9_corpus` (25 passed, 1 explicit regeneration test ignored), `particle_oracle` (13 passed), and `particle_protocol` (25 passed).
- Debug, release, and ASan/UBSan oracle builds passed; protocol and sanitizer-scope CTests passed; canonical and sanitizer semantic evidence runs passed.
- Local and exact-ref validators each proved 7 cases and 58 semantic bindings while denylisting runs 29439515367, 29583793056, and 29625083184.
- Provenance, inventory, all 21 inventory CLI tests, `cargo deny`, schema drift, actionlint, Markdown, upstream read-only, generated-report immutability, and diff hygiene passed.
- Two inventory generation passes retained `COMPATIBILITY.md` SHA-256 `6c30bee0036f378348cfdc17d34f49945be26129783bb7d483c509ae45373f3c`.

## Gap Closure

- `G09-DIFFERENTIAL-COMPARISON` is backed by named retained-rigid mutation regressions, including `phase9_comparator_rejects_retained_process_result_through_runner`, and by fresh canonical plus sanitizer evidence.
- `G09-EXECUTABLE-COVERAGE` is backed by the complete 58-binding corpus and named corruption regressions for semantic/payload digests, positive collision energy, nonempty stuck candidates, and exact reviewed branch identity.

## ASVS L1 Review

No high-severity finding was identified across the complete 09-25 through 09-29 diff.

- JSON and JSONL inputs use closed schemas, aggregate limits, bounded identities, and strict semantic validation.
- Evidence files must be bounded regular files; normalized paths stay under `target/`; symlinks and non-regular entries are rejected.
- Archive size and SHA-256 are checked before entry inspection; archive paths must be normalized, and symlink modes are rejected.
- Exact-ref validation requires one successful approved run, one canonical job, one sanitizer job, two distinct artifacts, and matching live snapshots.
- Shell commands use quoted paths or fixed `Command` arguments; no interpolated shell execution or secret output was introduced.
- Comparator and witness-binding work remains bounded, digest-bound, and covered by mutation regressions.

## Decisions Made

- Accepted only the replacement workflow-dispatch run after confirming it was successful, unexpired, at the exact remote head, and distinct from every denylisted run head.
- Required identical complete semantic manifests across canonical and sanitizer artifacts before changing the ledger.
- Kept all five deferred Phase 10 ledger rows at `not_evidenced` in implementation, unit, differential, and platform dimensions.

## Deviations from Plan

None - the plan executed as specified.

## Issues Encountered

- The evidence wrapper's read-only guard initially observed the intentional unstaged ledger update. Staging the scoped task files let the guard correctly verify that the evidence run itself made no repository-owned changes.
- An additional local sanitizer attempt enabled leak detection, which Apple ASan does not support. The plan-exact fail-fast ASan/UBSan settings were rerun and passed, as did the additional sanitizer-scope test.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration is required.

## Next Phase Readiness

Phase 09 is ready for a fresh verifier goal audit. Phase 10 particle groups, topology, pairs, triads, source areas, solver behaviors, and cross-engine stable-ID rotation remain explicitly pending.

## Self-Check: PASSED

- Task commits `9633f52` and `0e9d73a` exist.
- All created and modified files listed above exist.
- No stubs or untracked generated files remain.

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-18*

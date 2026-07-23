---
phase: 12-performance-portability-and-release-hardening
plan: "25"
subsystem: testing
tags: [regressions, replay, provenance, github-actions, evidence]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: typed reviewed regression registry, execution projection, and confined result validator
provides:
  - bounded one-pass execution of every reviewed named regression
  - typed completion and validation identities followed by final producer provenance
  - one scheduled or manual candidate-scoped regression artifact
affects: [release-evidence-aggregation, corrected-mismatch-replay, test-07]
tech-stack:
  added: []
  patterns:
    - typed registry projection before effects
    - completion then confined validation then producer identity
    - fail-closed zero-registration handling
key-files:
  created:
    - scripts/phase12-regressions.sh
    - .github/workflows/regressions.yml
    - tools/xtask/tests/regression_workflow.rs
  modified: []
key-decisions:
  - "Treat the typed execution-list projection as the only registry input; shell never reads or interprets the tracked TOML manifest."
  - "Keep bounded test logs temporary so the predecessor validator receives exactly completion.json, then preserve its identity.json and write producer-identity.json last."
  - "Permit check mode to report the truthful empty registry while run mode rejects zero registrations and cannot publish an empty artifact."
patterns-established:
  - "Named regression execution exports complete minimized-input and mismatch provenance to one exact cargo test invocation with no retries."
  - "Regression CI uploads exactly one phase12-regressions-<candidate> directory only after independent order and digest checks."
requirements-completed: [TEST-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T23:09:57Z
duration: 13min
completed: 2026-07-23
---

# Phase 12 Plan 25: Named Regression Replay QA Summary

**Every reviewed named regression now has one bounded candidate-bound replay path whose complete result set is typed-validated before final producer provenance and a single isolated artifact upload.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-23T22:56:57Z
- **Completed:** 2026-07-23T23:09:57Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added a safe Bash producer that consumes only the deterministic typed execution list, rejects zero/duplicate/malformed registrations, verifies exact minimized bytes, and invokes each named test once.
- Bound execution through environment fields to regression ID, named test, input hash, target, generator/toolchain, original candidate, fix, oracle/tolerance identities, first divergence, and failure class.
- Enforced 300-second per-test, 3,600-second total, 16-MiB log, 120-second registry, and 120-second validator bounds without deterministic retries.
- Published `completion.json`, then invoked the exact confined typed results command, preserved its `identity.json`, and wrote workflow/job/run/cardinality/payload provenance last in `producer-identity.json`.
- Added one scheduled/manual workflow with an exact detached candidate checkout, full-SHA actions, a finite job timeout, and exactly one candidate-scoped regression upload.
- Added eight focused tests covering valid command recording/order, the real confined CLI, malformed or empty registries, incomplete/duplicate/unregistered results, provenance drift, path/order/budget mutations, and workflow candidate/action/upload violations.

## TDD Evidence

- Task 1 RED: all five initial focused tests failed because the producer script did not exist.
- Task 1 GREEN: all five producer tests passed after the bounded script implementation.
- Task 2 RED: two new workflow projection tests failed because the workflow did not exist, while six producer/CLI tests remained green.
- Task 2 GREEN: all eight focused tests passed after the workflow implementation.
- No failing RED state was committed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Execute and validate every registered named regression** - `3f7aa26` (feat)
2. **Task 2: Publish one distinct candidate-bound regression artifact** - `ee85479` (ci)

## Files Created/Modified

- `scripts/phase12-regressions.sh` - Closed check/run producer with finite budgets, exact registry projection, one-pass named tests, typed completion validation, and producer identity last.
- `tools/xtask/tests/regression_workflow.rs` - Arrange/Act/Assert execution fixtures, command recorder, real CLI validation, static order checks, and negative producer/workflow mutations.
- `.github/workflows/regressions.yml` - Scheduled/manual detached-candidate replay and one post-validation artifact upload.

## Decisions Made

- Kept registry parsing in the typed xtask command; Bash validates only the emitted JSON projection and never reads manifest fields.
- Used temporary in-directory logs during execution, then removed them before typed validation because the predecessor validator correctly requires `completion.json` to be the only pre-validation file.
- Preserved the typed validator's `identity.json` and added `producer-identity.json` afterward to bind workflow, job, run ID, manifest hash, named-test cardinality, and the typed identity payload hash.
- Kept current zero-registration behavior truthful: check mode succeeds and reports zero, while run mode fails before any complete identity or artifact can exist.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced non-portable `mapfile` execution-list loading**

- **Found during:** Task 1 GREEN focused tests.
- **Issue:** macOS Bash 3.2 does not provide `mapfile`, so the valid confined producer fixture stopped before executing its named test.
- **Fix:** Replaced `mapfile` with a portable `while IFS= read -r` array loop while retaining exact projection cardinality checks.
- **Files modified:** `scripts/phase12-regressions.sh`
- **Verification:** The valid command-recorder fixture and complete focused suite pass on macOS.
- **Committed in:** `3f7aa26`

**2. [Rule 1 - Bug] Made result cardinality independent of `wc` padding**

- **Found during:** Task 1 GREEN focused tests.
- **Issue:** macOS `wc -l` pads numeric output, causing a string comparison to reject a complete one-result set.
- **Fix:** Counted JSONL result records with typed `jq -s length`.
- **Files modified:** `scripts/phase12-regressions.sh`
- **Verification:** Valid, omitted, duplicate, and unregistered result fixtures now reach their intended typed outcomes.
- **Committed in:** `3f7aa26`

***

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes made the planned producer portable without changing its registry, execution, validation, or artifact contracts.

## Verification

- `bash -n` and `shellcheck` passed for the producer.
- `scripts/phase12-regressions.sh check` passed and reported the tracked zero-entry reviewed registry.
- `cargo test -p xtask --test regression_workflow` passed all eight tests.
- `actionlint .github/workflows/regressions.yml` passed.
- Static assertions confirmed one exact confined validator command, one producer invocation, two full-SHA actions, one finite job timeout, one upload action, one candidate-scoped artifact name/path, and no retry or merged fuzz/sanitizer/coverage paths.
- Before each task commit, the exact ordered gate passed with `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and `CARGO_BUILD_JOBS=1`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

## Issues Encountered

None beyond the auto-fixed portability issues documented above.

## User Setup Required

None - no credentials or repository secrets are required.

## Next Phase Readiness

- Release aggregation can require exactly one `phase12-regressions-<candidate>` artifact and verify its typed and producer identities.
- The registry remains intentionally empty until a real minimized reviewed finding exists; adding the first valid registry entry makes the bounded producer executable without changing its orchestration contract.

## Known Stubs

None. Empty arrays in negative tests and the tracked zero-entry registry are deliberate fail-closed evidence cases, not unwired production data.

## Self-Check: PASSED

- All three implementation files and this summary exist.
- Task commits `3f7aa26` and `ee85479` exist.
- The workflow still passes `actionlint`.

***

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*

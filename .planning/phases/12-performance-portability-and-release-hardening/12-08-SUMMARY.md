---
phase: 12-performance-portability-and-release-hardening
plan: "08"
subsystem: performance
tags: [paired-benchmarks, calibration, confidence-intervals, evidence, xtask]

requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: sealed performance matrix and policy from Plan 12-10
  - phase: 12-performance-portability-and-release-hardening
    provides: identity-complete paired benchmark runner from Plan 12-20
provides:
  - closed performance CLI over all 32 sealed paired benchmark cases
  - five-run Student 95% calibration and fail-closed optimization admission
  - confined raw-report, validation, manifest, script, and just workflow surfaces
affects: [performance-evidence, optimization-review, release-claims]

tech-stack:
  added: []
  patterns:
    - injected provider seam around the concrete paired runner
    - pure calibrated decision core with fixed repository paths
    - validation-before-identity artifact publication

key-files:
  created:
    - reference/performance/policy.json
    - reference/performance/manifest.toml
    - tools/xtask/src/performance.rs
    - tools/xtask/src/performance/analysis.rs
    - tools/xtask/src/performance/evidence.rs
    - tools/xtask/src/performance/paths.rs
    - tools/xtask/src/performance/runner.rs
    - tools/xtask/tests/performance_cli.rs
    - scripts/phase12-performance.sh
  modified:
    - tools/xtask/src/main.rs
    - justfile

key-decisions:
  - "The injected provider abstracts only orchestration tests; production always calls the concrete Plan 12-20 adapters and paired runner."
  - "Optimization admission is scoped to all 32 sealed case identities and uses a Student 95% interval over exactly five independent runs."
  - "The reviewed manifest begins empty rather than representing unreviewed local measurements as performance evidence."

patterns-established:
  - "Performance CLI modes and paths are closed; runtime evidence remains below target/phase12-performance."
  - "Raw measurements precede calibration, validation precedes identity publication, and no workflow emits generalized claims."

requirements-completed: [PERF-04, PERF-05, PERF-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T01:41:14Z

duration: 33min
completed: 2026-07-23
---

# Phase 12 Plan 08: Performance Analysis and Evidence Workflow Summary

**Identity-complete paired benchmark orchestration with five-run Student intervals, calibrated fail-closed optimization admission, and validation-gated local evidence workflows**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-24T01:08:41Z
- **Completed:** 2026-07-24T01:41:14Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Added a closed `cargo xtask performance` surface whose real paired mode resolves all 32 sealed matrix cases and calls the existing Plan 12-20 native/oracle adapters without synthetic timing values.
- Added pure Student 95% interval analysis and optimization admission that applies `max(3%, calibrated noise floor)`, requires profile or typed bottleneck evidence, rejects forbidden build/timing modes, and protects every sealed workload case plus correctness hashes.
- Added confined raw-report validation, truthful reviewed-report manifest handling, and thin Bash/just workflows that publish identity only after validation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the calibrated analysis core and closed performance CLI** - `c59e397` (feat)
2. **Task 2: Add safe thin script and just workflows** - `0037565` (chore)

## Files Created/Modified

- `reference/performance/policy.json` - Reviewed analysis method, thresholds, authority, and claim scope.
- `reference/performance/manifest.toml` - Hash-bound manifest containing only explicitly reviewed reports.
- `tools/xtask/src/performance.rs` - Closed command parsing, sealed case preparation, and provider orchestration.
- `tools/xtask/src/performance/analysis.rs` - Pure calibration interval and optimization-admission rules.
- `tools/xtask/src/performance/evidence.rs` - Raw report calibration, identity validation, and optimization record checks.
- `tools/xtask/src/performance/paths.rs` - Bounded regular-file access and target-confined atomic output.
- `tools/xtask/src/performance/runner.rs` - Concrete D-05 identity collection and Plan 12-20 paired runner bridge.
- `tools/xtask/tests/performance_cli.rs` - Injected-runner and decision-boundary coverage.
- `tools/xtask/src/main.rs` - Performance command registration.
- `scripts/phase12-performance.sh` - Fixed calibrate, paired, and validate workflow modes.
- `justfile` - Discoverable Phase 12 performance recipes.

## Decisions Made

- Kept the test seam outside the timing engine: fake providers can prove orchestration, but production constructs the concrete native/oracle adapters and preserves the exact typed report returned by `run_paired_benchmark`.
- Bound optimization evidence to all 32 case IDs rather than collapsing scalable size points into 14 workload categories.
- Used a two-sided Student 95% interval with four degrees of freedom because the policy requires exactly five independent runs and retains the five raw run deltas.
- Left `reviewed_reports` empty until a human-reviewed immutable report exists; local raw output remains explicitly unreviewed and non-claiming.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Repo Instruction] Split performance orchestration into cohesive modules**

- **Found during:** Task 1
- **Issue:** The initial single `performance.rs` implementation exceeded the repository's 628-line refactor trigger.
- **Fix:** Extracted identity/runner and evidence operations into `performance/runner.rs` and `performance/evidence.rs`, leaving the command module below the threshold.
- **Files modified:** `tools/xtask/src/performance.rs`, `tools/xtask/src/performance/runner.rs`, `tools/xtask/src/performance/evidence.rs`
- **Verification:** Strict xtask Clippy, build, and focused tests passed.
- **Committed in:** `c59e397`

**2. [Rule 3 - Blocking] Annotated a cohesive pre-existing docs validator**

- **Found during:** Task 1 pre-commit verification
- **Issue:** Strict all-target Clippy stopped on `check_document_contracts`, a read-only validator two lines over the function-length threshold.
- **Fix:** Added the approved reasoned `clippy::too_many_lines` allowance to that function only.
- **Files modified:** `tools/xtask/src/docs.rs`
- **Verification:** Exact all-target Clippy passed with `-D warnings`.
- **Committed in:** `c59e397`

**3. [Rule 3 - Blocking] Applied mechanical Clippy fixes to regression workflow tests**

- **Found during:** Task 1 pre-commit verification
- **Issue:** After the docs lint was resolved, strict all-target Clippy exposed two borrowed-argument suggestions and one redundant closure in a pre-existing test helper.
- **Fix:** Borrowed the two JSON values and replaced the closure with the named method, without behavioral changes.
- **Files modified:** `tools/xtask/tests/regression_workflow.rs`
- **Verification:** Exact all-target Clippy and the complete test suite passed.
- **Committed in:** `c59e397`

**Total deviations:** 3 auto-fixed (1 repo-instruction adjustment, 2 blocking verification fixes)
**Impact on plan:** The module split follows repository code-shape rules; the two approved blocking fixes were mechanical and did not change behavior.

## Issues Encountered

- Strict Clippy initially revealed unrelated existing xtask lints only after earlier lint blockers were removed. Each was resolved narrowly with parent approval, and the complete ordered gate then passed.

## Verification

- `cargo test -p xtask --test performance_cli` - 8 focused tests passed.
- `cargo xtask performance paired --check` - resolved 32 sealed cases and the exact `oracle-release` executable without running measurements.
- `cargo xtask performance validate` - tracked policy, matrix, and manifest passed.
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings` - passed.
- `bash -n scripts/phase12-performance.sh` - passed.
- `just --list | rg 'phase12-performance-(paired|calibrate|validate)'` - all three recipes present.
- Non-claiming calibration smoke - the five-run Student interval test passed.
- Before each task commit, the exact ordered `cargo fmt --all`, Clippy, all-target build, and all-feature test gates passed using `/tmp/liquidfun-phase12.OJRc0w` with one build job.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The repository is ready to collect local raw paired measurements and host calibration evidence through the fixed workflow.
- No performance improvement is claimed by this plan; promotion requires an admitted optimization record and explicit review before adding a report to the tracked manifest.

## Known Stubs

None. The empty `reviewed_reports` manifest is intentional: no unreviewed measurement is represented as reviewed evidence.

## Self-Check: PASSED

All claimed implementation files and both task commits were verified.

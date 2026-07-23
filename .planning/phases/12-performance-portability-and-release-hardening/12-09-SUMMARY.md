---
phase: 12-performance-portability-and-release-hardening
plan: "09"
subsystem: testing
tags: [cargo-fuzz, libfuzzer, bounded-inputs, nightly, github-actions]
requires:
  - phase: 12-17
    provides: exact nightly-2026-07-15 toolchain and evidence-tool pinning
provides:
  - five safe bounded fuzz targets for protocol and native physics robustness
  - exact N and N+1 regression coverage for every reviewed resource cap
  - scheduled and release-candidate fuzz campaigns with minimized bounded evidence
affects: [release-hardening, fuzz-regression-corpus, scheduled-ci, compatibility-evidence]
tech-stack:
  added: [cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, arbitrary 1.4.2, sha2 0.10.9]
  patterns: [parse-before-effects mutation programs, exact dated nightly invocation, confined minimized evidence]
key-files:
  created:
    - fuzz/Cargo.toml
    - fuzz/src/lib.rs
    - fuzz/fuzz_targets/protocol.rs
    - fuzz/fuzz_targets/shapes_collision.rs
    - fuzz/fuzz_targets/world_mutation.rs
    - fuzz/fuzz_targets/particles.rs
    - fuzz/fuzz_targets/groups_ownership.rs
    - fuzz/corpus/README.md
    - .github/workflows/fuzz.yml
  modified: []
key-decisions:
  - "Keep the fuzz package outside the root workspace so ordinary Cargo builds and the published liquidfun crate remain independent of nightly-only tooling."
  - "Decode complete typed operation programs and validate all reviewed caps before applying any world or particle effects."
  - "Run pull requests in build-only mode while scheduled and explicit candidate-SHA lanes receive bounded 600-second and 1800-second campaigns."
patterns-established:
  - "Fuzz bounds: reject N+1 before effects and test both the exact accepted cap and first rejected value."
  - "Fuzz evidence: upload only bounded logs, exact identities, closed classifications, and minimized inputs from a target-scoped directory."
requirements-completed: [TEST-05, TEST-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T21:05:09Z
duration: 35m
completed: 2026-07-23
---

# Phase 12 Plan 09: Bounded Fuzzing and Regression Capture Summary

**Five native parser and physics fuzz targets now enforce reviewed resource caps before effects, with exact-nightly CI campaigns that minimize and confine regression evidence.**

## Performance

- **Duration:** 35m
- **Started:** 2026-07-23T20:30:26Z
- **Completed:** 2026-07-23T21:05:09Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added exactly five safe Rust fuzz binaries covering strict protocol decoders, shape collision, rigid-world mutation, particles, and particle-group ownership/invalidation.
- Enforced one-mebibyte protocol input, 256 operations, 128 bodies/fixtures, 4096 particles, and 64 groups with exact N/N+1 regression tests.
- Kept the nested fuzz package outside the normal workspace and preserved the publishable `liquidfun` dependency boundary.
- Added full-SHA-pinned PR build-only, scheduled 600-second, and manual candidate 1800-second GitHub Actions lanes using only `nightly-2026-07-15`.
- Confined CI evidence to target-scoped bounded logs, exact identities, closed classifications, minimization status, and minimized inputs.

## TDD Evidence

- The plan prohibited committing a failing RED state, so boundary tests and their implementation were committed together.
- Six fuzz-package tests pass: one exact classification-vocabulary check plus N/N+1 checks for all five reviewed target bounds.
- All five binaries completed 32-run exact-nightly smoke campaigns without a finding.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add bounded parser and physics fuzz targets** - `4f23188` (feat)
2. **Task 2: Add isolated scheduled and release-candidate fuzz lanes** - `7135f19` (ci)

## Files Created/Modified

- `fuzz/Cargo.toml` - Defines the isolated private fuzz package and exactly five pinned binaries.
- `fuzz/src/lib.rs` - Implements bounded parse-before-effects programs, invariants, closed regression metadata, and N/N+1 tests.
- `fuzz/fuzz_targets/*.rs` - Provides five thin libFuzzer entry points.
- `fuzz/corpus/README.md` - Documents reviewed limits, exact tool identities, seed commands, and regression handoff metadata.
- `.github/workflows/fuzz.yml` - Builds on pull requests and runs bounded scheduled/manual per-target campaigns with confined evidence.

## Decisions Made

- The fuzz package is an isolated nested workspace, preventing nightly-only and fuzz dependencies from entering ordinary workspace or consumer builds.
- Expected typed validation failures are normal rejected inputs; engine panics, non-finite committed state, stale-handle acceptance, and invariant failures remain findings.
- Workflow candidates are exact 40-character commits, actions use full commit SHAs, and checkout credentials and submodules remain disabled.
- Successful campaigns record a null classification; findings use only `Harness`, `PhysicsMismatch`, `Sanitizer`, `Timeout`, or `Schema`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed the missing exact cargo-fuzz release**

- **Found during:** Task 1 (Add bounded parser and physics fuzz targets)
- **Issue:** The required `cargo fuzz` command was not installed locally.
- **Fix:** Installed `cargo-fuzz` 0.13.2 with its locked dependency graph.
- **Files modified:** None.
- **Verification:** `cargo +nightly-2026-07-15 fuzz build` and all five bounded smoke campaigns passed.
- **Committed in:** No repository change.

**2. [Rule 1 - Bug] Removed warning-denied fuzz package diagnostics**

- **Found during:** Task 1 (Add bounded parser and physics fuzz targets)
- **Issue:** The first strict fuzz-package lint found missing panic documentation and precision-loss casts in bounded scalar generation.
- **Fix:** Documented finding-only panic contracts and converted through bounded 16-bit integers before `f32`.
- **Files modified:** `fuzz/src/lib.rs` and the five target entry points.
- **Verification:** Fuzz-package Clippy passed with `-D warnings`; exact-nightly build and smokes also passed.
- **Committed in:** `4f23188`.

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)

**Impact on plan:** Both fixes were necessary to run the planned exact toolchain and satisfy repository warning policy. They did not widen target scope or affect production physics.

## Verification

- `cargo test --manifest-path fuzz/Cargo.toml` - 6 passed, including every exact N/N+1 bound.
- `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --all-features -- -D warnings` - passed.
- `cargo +nightly-2026-07-15 fuzz build` - all five targets built in an isolated nightly target directory.
- Five `cargo +nightly-2026-07-15 fuzz run ... -runs=32` smoke campaigns - passed.
- `actionlint .github/workflows/fuzz.yml` and exact workflow assertions - passed.
- `cargo tree -p liquidfun --edges normal` - only `bitflags` remains in the production dependency tree.
- Root workspace metadata and package-list scans found no fuzz package or private protocol/differential leakage.
- Prohibited-pattern and floating-nightly scans passed.
- The required ordered format, warning-denied Clippy, all-target build, and all-feature tests passed before both task commits.
- `mdformat --check fuzz/corpus/README.md` passed. The repository-wide `just markdown-check` still reports eight unrelated pre-existing tracked Markdown formatting failures; those files were preserved unchanged.
- Task-scoped staged-diff checks found no whitespace errors or unintended files.

## Known Stubs

None.

## Issues Encountered

- Two classifier interruptions paused execution and re-established the same defensive repository-QA scope. Work resumed from the existing local state without expanding behavior or changing the plan.
- The repository-wide Markdown check spent several minutes enumerating a large pre-existing `target/debug/deps` directory before reporting its unrelated baseline failures.

## User Setup Required

None - scheduled execution uses repository GitHub Actions and manual runs require only an exact candidate commit SHA.

## Next Phase Readiness

- The five targets and closed metadata vocabulary are ready to receive minimized regression inputs from scheduled and candidate campaigns.
- The published crate remains independent of fuzz tooling, nightly Rust, C++, and private protocol crates.
- No Plan 12-09 code or workflow blockers remain.

## Self-Check: PASSED

- All nine plan implementation/workflow files and this summary exist.
- Task commits `4f23188` and `7135f19` exist in repository history.
- Stub scan found no incomplete implementation. The workflow's successful-campaign `classification: null` is an intentional closed-schema value, not an unwired placeholder.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*

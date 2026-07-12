---
phase: 06-minimal-rigid-world-vertical-slice
plan: "18"
subsystem: rigid-sanitizer-signoff
tags: [github-actions, asan, ubsan, ctest, rigid-world, documentation]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "14"
    provides: Transactional aggregate mass rejection
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "17"
    provides: Real D1-gated rigid fixture lifecycle
provides:
  - Fail-fast C++ protocol and rigid-world execution under the sanitizer preset
  - Bounded seven-day failure artifact policy with fail-if-missing enforcement
  - Executable documentation contracts covering all seven Phase 6 verifier gaps
  - Local D2 sanitizer, debug, release, replay, and D0 completion evidence
affects: [phase-06-formal-verification, phase-07-rigid-solver, oracle-ci]
tech-stack:
  added: []
  patterns: [execute-before-claim sanitizer evidence, bounded failure upload, gap-ID documentation contract]
key-files:
  created:
    - .planning/phases/06-minimal-rigid-world-vertical-slice/06-18-SUMMARY.md
  modified:
    - .github/workflows/oracle.yml
    - tools/reference/CMakeLists.txt
    - tools/xtask/src/differential.rs
    - tools/xtask/src/docs.rs
    - tools/xtask/tests/differential_cli.rs
    - tools/xtask/tests/docs_contract.rs
    - TESTING.md
    - ARCHITECTURE.md
    - README.md
key-decisions:
  - "The sanitizer lane must build and execute the C++ protocol test target plus one rigid-world comparison before its read-only assertion."
  - "Only rigid-world compare with oracle-asan-ubsan and one-shot is admitted as the Phase 6 sanitizer command shape."
  - "Apple Clang's sanitizer-only overriding-option warning is demoted only for the read-only upstream Box2D target; repository-authored warning denial remains unchanged."
requirements-completed: [RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T17:27:29Z
duration: 13 min
completed: 2026-07-12
---

# Phase 6 Plan 18: Sanitizer Execution and Re-Signoff Summary

**The scheduled sanitizer lane now executes the C++ rigid protocol and full rigid-world path with fail-fast ASan/UBSan, while machine-enforced docs preserve all seven gap closures and the D0/D1/D2 authority boundaries.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-12T17:14:09Z
- **Completed:** 2026-07-12T17:27:29Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added an explicit sanitizer protocol-test build, fail-fast CTest execution, and one-shot `rigid-world` comparison before the existing empty-world one-shot/reuse corpus and read-only assertion.
- Kept failure upload confined to `target/differential/failures`, seven-day retention, and `if-no-files-found: error`, with tests rejecting status suppression or broader artifact paths.
- Made the closed xtask parser admit exactly the reviewed rigid sanitizer comparison and proved it passes canonical child arguments.
- Documented aggregate atomicity, no-dynamic admission, exact fixed step/action/inertia boundaries, real D1-only fixture lifecycle, and executable sanitizer evidence without widening Phase 7 or platform claims.
- Added a fail-closed docs contract naming every original Phase 6 verifier gap.

## Task Commits

1. **Task 1: Execute rigid protocol and comparison under ASan/UBSan** - `ae5d536` (`ci`)
2. **Task 2: Run final gap closure and goal verification matrix** - `bfa3635` (`fix`)

## Decisions Made

- Build `liquidfun-reference-protocol-tests` explicitly because the reviewed xtask upstream build intentionally produces only the oracle executable; clean CI cannot rely on a cached CTest binary.
- Preserve one-shot as the only Phase 6 rigid sanitizer session shape. Reuse remains the empty-world reset corpus, and replay/minimize under ASan/UBSan remain rejected.
- Treat local Apple Clang 21/CMake 3.27 sanitizer results as D2 only. Canonical Clang 22.1.8 Linux CI remains the sole D1/platform authority.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Admit the workflow's rigid sanitizer command through xtask**

- **Found during:** Task 2 local execution of the new workflow command.
- **Issue:** `cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot` exited with `differential/usage` because fixed evidence scenarios admitted only debug/release presets, so the new CI step could never execute.
- **Fix:** Added a scenario/action/preset-specific admission for only the one-shot rigid sanitizer compare and mapped it to `OraclePreset::AsanUbsan`.
- **Files modified:** `tools/xtask/src/differential.rs`, `tools/xtask/tests/differential_cli.rs`.
- **RED signal:** `sanitizer_rigid_compare_passes_only_the_reviewed_one_shot_shape` failed with the exact usage rejection before the fix.
- **Regression proof:** The focused xtask test passes, unreviewed rigid session shapes remain rejected, and the real local sanitizer compare matches both required families.
- **Committed in:** `bfa3635`.

**2. [Rule 3 - Blocking] Keep the sanitizer oracle build executable on Apple Clang**

- **Found during:** Task 2 fresh `oracle-asan-ubsan` rebuild before local execution.
- **Issue:** The pinned upstream Box2D target promotes Apple Clang's `-Woverriding-option` diagnostic to an error when reviewed precise floating flags overlap under the sanitizer preset.
- **Fix:** Demoted only that warning, only when sanitizer flags are present, and only on the read-only upstream Box2D target. Repository-authored targets retain strict warning denial, and canonical D1 identity/flags are unchanged.
- **Files modified:** `tools/reference/CMakeLists.txt`.
- **Failure signal:** Fresh build stopped at Box2D translation units with `clang++: error: overriding '-ffp-model=precise' option with '-ffp-contract=off' [-Werror,-Woverriding-option]`.
- **Regression proof:** A fresh configure/build succeeds; CTest passes 1/1; rigid ASan/UBSan compare matches both required families as D2.
- **Committed in:** `bfa3635`.

**Total deviations:** 2 auto-fixed blocking issues. **Impact:** Both were necessary to make the planned sanitizer evidence executable. Neither widens public solver scope, accepted scenario shapes, repository-authored warning policy, or D1 authority.

## Issues Encountered

- A previously built local sanitizer protocol binary initially passed CTest, but a clean rebuild exposed the Apple Clang option-overlap blocker above. Completion evidence uses the fresh rebuilt target, not the cached binary.
- TDD RED tests were not committed separately because repository instructions require formatting, strict Clippy, all-target build, and all-feature tests to pass before every commit.
- Local tools are CMake 3.27.9 and Apple Clang 21.0.0, so sanitizer/debug/release/replay results remain D2. No local run was represented as canonical D1.
- Formal GSD goal-backward verification and code review are intentionally left to the phase orchestrator after this plan summary; this executor did not hand-edit `06-VERIFICATION.md` or mark Phase 6 complete.

## Validation Evidence

- Gap-focused Rust: aggregate mass 2/2; non-dynamic admission 2/2; rigid protocol 18/18 plus rejected centered-inertia fixture.
- Real fixture lifecycle: 4/4 differential real-binary tests and 2/2 xtask rigid-fixture tests, including D1 acceptance and D2 no-effect rejection.
- Sanitizer contracts: 3/3 xtask sanitizer tests and 4/4 Oracle workflow contract tests.
- Fresh local ASan/UBSan: configured/built `oracle-asan-ubsan`, built the protocol test target, CTest passed 1/1, rigid compare matched two required families as D2, and both preserved empty-world one-shot/reuse commands passed.
- Fixed rigid workflows: debug compare D2, release compare D2, debug replay D2, and exactly two byte-identical native/oracle debug runs D0.
- Repository checks: docs, 177-row inventory, provenance, 58-entry package isolation, warning-denied workspace rustdoc, `mdformat --check` on changed public Markdown, and `git diff --check` passed.
- Mandatory pre-commit sequence passed before both task commits: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The orchestrator can now run code review and formal Phase 6 goal-backward verification against executable sanitizer evidence and all seven gap-specific contracts.
- Phase 7 remains responsible for forces, public velocity/step configuration, general islands, sleeping, CCD, queries, ray casts, origin shifting, and broad world configuration. Phase 8 remains responsible for joints and broad rigid sign-off.

## Self-Check: PASSED

- Task commits `ae5d536` and `bfa3635` exist in history.
- The sanitizer workflow orders build, protocol target build, CTest, rigid comparison, preserved empty-world corpus, and read-only assertion.
- All nine modified implementation/workflow/document files exist and the full matrix passed after the final changes.
- Phase 6 was not marked complete and `06-VERIFICATION.md` was not hand-edited.

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*

---
phase: 12-performance-portability-and-release-hardening
plan: "12"
subsystem: platform-release-evidence
tags: [cargo-package, artifact-fanout, msrv, native-runners, d2-evidence]
requires:
  - phase: 12-11
    provides: content-addressed package creation, exact-byte verification, and closed platform support policy
provides:
  - one-producer artifact-first platform release-candidate workflow
  - exact Rust 1.92 canonical Linux and Rust 1.97 native platform consumers
  - identity-last D2 platform records with exact runner, compiler, archive, and candidate identity
  - distinct fail-closed Intel macOS downgrade evidence
affects: [phase-12-release-audit, platform-support, package-release, ci]
tech-stack:
  added: []
  patterns: [one reviewed artifact fanned across native runners, workflow-bound identity-last evidence, conditional support downgrade]
key-files:
  created:
    - scripts/phase12-platform.sh
    - .github/workflows/platform.yml
    - tools/xtask/tests/platform_workflow.rs
  modified:
    - .github/workflows/ci.yml
key-decisions:
  - "Run broad package/platform evidence only on schedule or explicit release-candidate dispatch while pull-request CI retains one Linux quality job and three default-feature smokes."
  - "Accept exactly a target, D2 tier, archive path, and identity path at the platform script boundary; derive and validate every other authority from the checked-out candidate and tracked support policy."
  - "Run Intel macOS only on macos-15-intel while fresh named native evidence exists; otherwise publish a distinct unsupported downgrade record."
patterns-established:
  - "Artifact-first platform QA: one Rust 1.97 producer uploads one archive and identity, and every consumer verifies those exact bytes without packaging again."
  - "Platform evidence: verification, package isolation, rustdoc, and smoke tests finish before workflow-bound identity.json is atomically published."
requirements-completed: [FND-06, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T21:46:06Z
duration: 18m
completed: 2026-07-23
---

# Phase 12 Plan 12: Artifact-First Platform QA Summary

**One reviewed Cargo archive now fans out to exact MSRV and native platform consumers that publish workflow-bound D2 evidence without submodules, repackaging, or canonical fixture authority.**

## Performance

- **Duration:** 18m
- **Started:** 2026-07-23T21:28:16Z
- **Completed:** 2026-07-23T21:46:06Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Added one full-SHA-pinned scheduled/manual workflow whose Ubuntu 24.04 producer creates the only `.crate`, while every downstream lane downloads and verifies the same artifact name and bytes.
- Added exact Rust 1.92 canonical Linux verification plus Rust 1.97 native verification on Ubuntu x86_64, Ubuntu ARM64, macOS ARM64, Windows x86_64, and conditionally Intel macOS.
- Added a safe four-argument Bash runner that confines inputs, verifies candidate/hash/toolchain/support identities, executes the existing package verifier, runs rustdoc and doc-test smokes from the extracted package, proves canonical records stayed read-only, and writes D2 identity last.
- Added a separate 90-day Intel macOS policy resolver and unsupported downgrade artifact, so missing or expired native evidence cannot preserve a stale support claim.
- Slimmed ordinary CI by moving MSRV breadth into the artifact workflow while retaining Linux quality and three exact mainstream default-feature smoke runners.

## TDD Evidence

- **RED:** Six focused workflow tests failed because the platform workflow and script did not exist and the old CI still carried its independent MSRV job.
- **GREEN:** Six focused tests now prove exact runners, targets, toolchains, action pins, one-producer fan-out, conditional downgrade routing, D2-only script authority, identity-last ordering, and slim submodule-free CI.
- **REFACTOR:** Kept the Bash entrypoint effect-focused, reused the typed package verifier as archive authority, and concentrated conditional policy selection in one workflow job.
- The plan prohibited a failing RED commit, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Fan one reviewed archive across MSRV and native runners** - `b00ff00` (feat)

## Files Created/Modified

- `scripts/phase12-platform.sh` - Confines and validates the platform tuple, consumes the exact package artifact, runs isolated documentation/smoke verification, and publishes workflow-bound D2 identity last.
- `.github/workflows/platform.yml` - Creates one release-candidate archive and fans it across exact MSRV, durable native, conditional native, and downgrade jobs.
- `.github/workflows/ci.yml` - Keeps ordinary CI to Linux quality and three current mainstream default-feature runner smokes.
- `tools/xtask/tests/platform_workflow.rs` - Proves artifact reuse, exact runner/version/target strings, full action pins, support freshness, downgrade separation, D2-only output, and CI isolation.

## Decisions Made

- Used `actions/download-artifact` v8 pinned to full commit `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`, alongside the repository's reviewed checkout v7 and upload v7 pins.
- Kept every checkout submodule-free because package creation, verification, documentation, and smokes operate only on the native Rust crate and tracked policy.
- Recorded GitHub workflow, job, run, runner, timestamp, compiler, target, scalar mode, candidate SHA, and archive SHA in the final D2 identity; local execution uses explicit `local` identities and run `0`.
- Kept `reference/platform/support.json` unchanged with null Intel macOS native evidence, so the current truthful workflow outcome is the distinct unsupported downgrade path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected artifact identity parsing and extracted-package execution**

- **Found during:** Task 1 local producer/consumer smoke.
- **Issue:** An initial jq expression applied `type` to the combined predicate, and documentation commands initially ran from the repository rather than the extracted crate.
- **Fix:** Parenthesized the typed hash predicate and changed rustdoc/doc-test execution to a subshell rooted at the verified extracted package.
- **Files modified:** `scripts/phase12-platform.sh`
- **Verification:** The real local archive round trip completed with rustdoc and all 21 doc tests reporting the temporary extracted package path, then published a valid D2 identity.
- **Committed in:** `b00ff00`

**2. [Rule 3 - Blocking] Preserved the existing CI submodule-isolation contract after removing the standalone MSRV job**

- **Found during:** Task 1 focused package CLI regression verification.
- **Issue:** Slimming CI from three jobs to two made the existing contract's textual checkout-isolation count fail even though the default-feature matrix still expands to three submodule-free runners.
- **Fix:** Added a truthful CI comment documenting that every matrix expansion preserves `submodules: false`.
- **Files modified:** `.github/workflows/ci.yml`
- **Verification:** All 22 package CLI tests pass.
- **Committed in:** `b00ff00`

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking issue)

**Impact on plan:** Both fixes were required to prove actual package isolation and preserve established CI contracts; no feature or authority scope expanded.

## Issues Encountered

- The locally generated package and evidence directories are ignored runtime output under `target/`; they were used only to prove the producer/consumer path and were not committed.
- Native Linux ARM64, Windows x86_64, and Intel macOS execution require their GitHub-hosted runners; their exact command and policy contracts are locally covered by workflow tests and actionlint.

## Known Stubs

None. The null Intel macOS native-evidence record is an intentional fail-closed support-policy state, not a placeholder; it selects the tested downgrade job until reviewed evidence exists.

## Verification

- `bash -n scripts/phase12-platform.sh` - passed.
- `shellcheck scripts/phase12-platform.sh` - passed.
- `shfmt -d scripts/phase12-platform.sh` - passed with no diff.
- `actionlint .github/workflows/platform.yml .github/workflows/ci.yml` - passed.
- `cargo test -p xtask --test platform_workflow` - 6 passed.
- `cargo test -p xtask --test package_cli` - 22 passed.
- Real local producer created one SHA-256-addressed `.crate`; the consumer verified exact bytes, built/tested through the package verifier, generated rustdoc, ran 21 doc tests from the extracted archive, preserved tracked canonical records, and published D2 identity last.
- Negative local checks rejected a non-D2 tier and missing conditional Intel macOS evidence without publishing a conditional identity.
- Exact runner/version/target scan found all five reviewed target triples, Rust 1.92.0/1.97.0, and only `macos-15-intel` for conditional Intel execution.
- Exact ordered commit gate passed after final changes: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `git diff --cached --check` passed and the staged implementation contained exactly the four plan-owned files.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Release aggregation can consume one candidate-bound package identity plus independent MSRV/native D2 result artifacts.
- Intel macOS remains explicitly unsupported until `reference/platform/support.json` contains fresh reviewed `macos-15-intel` evidence within the exact 90-day window.

## Self-Check: PASSED

- Confirmed the summary and all four plan-owned implementation files exist.
- Confirmed task commit `b00ff00` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

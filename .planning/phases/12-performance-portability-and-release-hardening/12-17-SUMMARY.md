---
phase: 12-performance-portability-and-release-hardening
plan: "17"
subsystem: nightly-toolchain-policy
tags: [rustup, nightly, miri, sanitizers, coverage, reproducibility]
requires: []
provides:
  - exact shared nightly-2026-07-15 toolchain predecessor
  - minimal Miri, rust-src, and LLVM tools component contract
  - repository policy rejecting floating unstable-tool selectors
affects: [phase-12-fuzzing, phase-12-safety-evidence, phase-12-coverage]
tech-stack:
  added: [nightly-2026-07-15]
  patterns: [dated toolchain identity, repository selector scan]
key-files:
  created:
    - rust-toolchain-nightly.toml
    - tools/xtask/tests/nightly_toolchain.rs
  modified: []
key-decisions:
  - "Use one exact nightly-2026-07-15 predecessor with a minimal profile and only miri, rust-src, and llvm-tools-preview."
  - "Reject floating Cargo, rustup, YAML toolchain, environment, and TOML channel selectors while accepting exact dated equivalents."
patterns-established:
  - "Unstable-tool consumers depend on one checked-in dated toolchain contract instead of independently resolving nightly."
requirements-completed: [TEST-05, TEST-06, TEST-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T19:52:00Z
duration: 5m
completed: 2026-07-23
---

# Phase 12 Plan 17: Shared Dated Nightly Toolchain Summary

**Fuzz, Miri, Rust sanitizer, and coverage work now share one exact dated nightly with a machine-checked component and no-floating-alias policy.**

## Performance

- **Duration:** 5m
- **Started:** 2026-07-23T19:46:57Z
- **Completed:** 2026-07-23T19:52:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Added `rust-toolchain-nightly.toml` with exact `nightly-2026-07-15`, minimal profile, and exactly `miri`, `rust-src`, and `llvm-tools-preview`.
- Added a focused TOML contract test that verifies the exact channel, profile, and complete component set.
- Added repository policy coverage for workflows, shell scripts, tool declarations, and the command facade, rejecting undated Cargo, rustup, YAML, environment, and TOML selectors.
- Confirmed Plans 12-09 and 12-10 depend on this shared predecessor before creating fuzz or safety/coverage consumers.
- Installed and resolved the declared toolchain locally as `rustc 1.99.0-nightly (da80ed070 2026-07-14)`.

## TDD Evidence

- **RED:** The initial policy run had two failures because `rust-toolchain-nightly.toml` did not exist.
- **GREEN:** The exact TOML made all four focused policy tests pass.
- **REFACTOR:** Tightened selector matching so every exact dated Cargo, rustup, YAML, environment, and TOML form remains accepted while only floating aliases fail.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Pin and enforce the shared Phase 12 nightly** - `8b9914f` (chore)

## Files Created/Modified

- `rust-toolchain-nightly.toml` - Declares the exact shared nightly and required unstable-tool components.
- `tools/xtask/tests/nightly_toolchain.rs` - Parses the contract, scans all current/future consumer surfaces, rejects floating aliases, and checks dependent plan ordering.

## Decisions Made

- Kept the stable `rust-toolchain.toml` unchanged; unstable evidence must opt into the separate dated predecessor explicitly.
- Required exactly the three reviewed components rather than allowing silent component expansion.
- Scanned all repository workflow and shell files plus TOML tool declarations and `justfile`, so future consumers inherit the same rejection policy automatically.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The exact nightly was absent locally, so it was installed using only the plan-declared date, minimal profile, and three components.

## Known Stubs

None.

## Verification

- `cargo test -p xtask --test nightly_toolchain` - 4 passed.
- `rustup run nightly-2026-07-15 rustc --version` - resolved `rustc 1.99.0-nightly (da80ed070 2026-07-14)`.
- Installed-component inspection confirmed Miri, rust-src, and LLVM tools.
- `cargo clippy -p xtask --test nightly_toolchain -- -D warnings` - passed.
- Exact ordered commit gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `git diff --check` passed.

## User Setup Required

None - the exact toolchain was installed during execution.

## Next Phase Readiness

- Plan 12-09 can build cargo-fuzz targets and workflows against the exact shared nightly.
- Plans 12-10 and 12-24 can bind Miri, sanitizer, and coverage evidence to the same resolved compiler/component identity.

## Self-Check: PASSED

- Confirmed the summary and both declared key files exist.
- Confirmed task commit `8b9914f` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

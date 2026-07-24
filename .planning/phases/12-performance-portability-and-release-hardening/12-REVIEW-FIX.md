---
phase: 12-performance-portability-and-release-hardening
fixed_at: 2026-07-24T05:31:43Z
review_path: .planning/phases/12-performance-portability-and-release-hardening/12-REVIEW.md
iteration: 3
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-07-24T05:31:43Z
**Source review:** `.planning/phases/12-performance-portability-and-release-hardening/12-REVIEW.md`
**Iteration:** 3

**Summary:**

- Findings in scope: 1
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-02R: Clean differential coverage can skip the real math comparison

**Files modified:** `.github/workflows/coverage.yml`, `crates/liquidfun-differential/tests/round_trip.rs`, `scripts/phase12-coverage.sh`, `tools/xtask/tests/coverage_workflow.rs`
**Commit:** fc547d07a6290d3017973fa488eb7f1f8fb68d14
**Status:** fixed: requires human verification
**Applied fix:** The differential coverage job now checks out the pinned upstream recursively, installs and asserts the exact Clang 22.1.8, CMake 4.3.3, and Ninja 1.13.2 toolchain, and builds both `oracle-debug` and `oracle-release` before running the producer. The producer fails before executing any differential target unless both exact oracle executables exist. The math comparison also fails closed when coverage marker mode is active, so its two leaves can be emitted only after real debug and release comparisons complete. Static, mutation, and executable clean-lane contract tests prevent checkout, toolchain, build-order, or producer prerequisites from being removed silently.

## Verification

- The exact ordered Rust gate passed with
  `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and
  `CARGO_BUILD_JOBS=1`: `cargo fmt --all`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo build --all-targets --all-features`, and
  `cargo test --all-features`.
- Actionlint passed for `.github/workflows/coverage.yml`.
- Bash syntax validation and ShellCheck passed for
  `scripts/phase12-coverage.sh`.
- Both C++ oracle presets configured and built, and
  `liquidfun-reference-protocol-tests` passed.
- The focused real C++ math probe passed and emitted exactly the two required
  math leaf markers.
- `scripts/phase12-coverage.sh differential <candidate>` ran the real
  differential targets and verified all 63 required leaves.
- `scripts/phase12-coverage.sh check` passed its typed-authority and omission
  guards.
- `cargo test -p xtask --test coverage_workflow` passed all 4 workflow,
  mutation, producer, and missing-oracle contract tests.
- `git diff --check` passed.

***

_Fixed: 2026-07-24T05:31:43Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 3_

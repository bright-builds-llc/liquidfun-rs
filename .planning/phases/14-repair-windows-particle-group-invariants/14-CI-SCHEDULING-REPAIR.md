---
phase: 14-repair-windows-particle-group-invariants
type: ci-scheduling-repair
status: locally-verified
---

# Remove duplicate private suite execution

## Observed timeout

Cargo CI run `34812688006`, attempt 1 at `430e0ad775d64322b04ae04504d5a6275a197dc3`, has three successful platform jobs. Quality job `103876986960` is cancelled after its 30-minute limit. Retained check-run annotations explicitly state `The job has exceeded the maximum execution time of 30m0s` and `The operation was canceled.` This is not a test assertion failure or a passing whole-workflow result.

The full workspace test step passed from `06:22:15Z` to `06:34:48Z`. Consumer isolation and corpus closure also passed. The later headless step reran the same four private packages from `06:36:43Z` until cancellation at `06:45:51Z`. Its repeated `rigid_fixture_workflow` target passed all 15 tests after 404.47 seconds; cancellation interrupted the repeated `supervisor_failures` target afterward. Full raw logs, job/run metadata and timeout annotations are retained under `target/phase14-platform/430e0ad775d64322b04ae04504d5a6275a197dc3/attempt-20260914-01/`.

## Equivalent scheduling

- Existing `cargo test --workspace --all-features` now explicitly receives empty DISPLAY, WAYLAND_DISPLAY, MIR_SOCKET and XDG_RUNTIME_DIR variables, so the complete private suite runs headlessly at its original required source point.
- The later step removes only the redundant four-package build/test commands. The existing workspace all-target/all-feature build and all-feature tests cover those packages and their feature configuration. The selected differential package already enables the sole optional liquidfun feature, differential-internals; no feature-specific coverage is lost.
- The explicit testbed build/test and empty-display environment remain, as do all focused, isolation, corpus, protocol, supervisor, provenance, documentation and inventory gates.
- The quality timeout remains 30 minutes. No test target, assertion, case count, ignore marker, skip flag, tolerance or failure policy changes. The existing ignored fixture-regeneration tool is unchanged; it is not an acceptance test suppressed by this repair.

## Verification

`target/phase14-platform/local-attempt-20260914-ci-scheduling01/` retains passing actionlint and `package_cli` 22/22, including the headless/isolation workflow contract, followed by ordered fmt, Clippy, build and all-feature core tests (1,012 pass). Managed checks report 939 files and zero findings. Root and independent static review verify that the retained workspace commands cover the removed duplicate configuration. Fresh same-SHA platform and terminal quality proof remain required; the cancelled source is not relabeled.

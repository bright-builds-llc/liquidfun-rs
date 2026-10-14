---
phase: 45-curiosity-groundwork-and-feasibility-spikes
fixed_at: 2026-10-14T00:00:00Z
review_path: .planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 45: Code Review Fix Report

**Fixed at:** 2026-10-14
**Source review:** .planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01, IN-01, IN-05, as selected by the owner)
- Fixed: 3
- Skipped: 0
- Deferred out of scope: IN-02, IN-03 and IN-04. They stay recorded in 45-REVIEW.md for later.

## Fixed Issues

### WR-01: The gate ignores the warmup and measured step counts that determine the fingerprints

**Files modified:** `tools/xtask/src/playground/fingerprint_gate.rs`, `tools/xtask/src/playground/fingerprint_gate/tests.rs`
**Commit:** 550a5fc86
**Applied fix:** Added `DETERMINING_FIELDS` (`warmup_steps`, `measured_steps`) and `ensure_same_capture_parameters`. `run` now calls it after the soft host and toolchain warnings and before `compare_fingerprints`. A missing or different observed count is a hard `gate` error that names both values, for example "observed measured_steps 10 differs from baseline measured_steps 120; recapture with the baseline step counts before comparing fingerprints". A baseline whose provenance lacks a count is also rejected. The test fixtures now carry 60/120 step counts. New Arrange/Act/Assert tests check the matching case (`Ok(())`), the exact error for a mismatched `measured_steps`, and the exact error for a missing `warmup_steps`.

### IN-01: The `just` recipe does not quote the forwarded arguments

**Files modified:** `justfile`
**Commit:** 2a8532b9f
**Applied fix:** The justfile has no `set positional-arguments`, and this is its only variadic recipe; the single-value recipes use `quote()`, which cannot split a variadic. The recipe now uses the recipe-scoped `[positional-arguments]` attribute and forwards `"$@"`, so the other recipes are unchanged. Checked with `just --dry-run` and a real run with `--spot` pointing at a path containing a space; the gate read the file and passed.

### IN-05: The failure message for a partial capture does not name the likely cause

**Files modified:** `tools/xtask/src/playground/fingerprint_gate.rs`, `tools/xtask/src/playground/fingerprint_gate/tests.rs`
**Commit:** 6a8d60257
**Applied fix:** Moved the failing-report error into `gate_failure(report, observed)`. When `full_catalog` is `false` and protected ids are missing, the message adds "the observed capture is filtered (full_catalog false), so rerun `cargo xtask playground scene-spot` without --scene". The hint names the xtask command because the `just playground-scene-spot` recipe takes no arguments. Two tests assert the exact error with and without the filter hint.

## Verification

- `cargo fmt --all`: clean, and only the edited files changed.
- `cargo clippy -p xtask --all-targets -- -D warnings`: passed.
- `cargo test -p xtask --bin xtask playground`: 104 passed (32 in `fingerprint_gate`).
- `bun scripts/bright-builds-check.ts all`: 0 findings. `fingerprint_gate.rs` is 420 lines, under the 628-line limit.
- `just scene-fingerprint-gate --stamp 2026-10-14T16-57-32Z`: PASS, 25/25 protected unchanged.
- Negative checks on mutated copies of that capture in the session scratchpad: `measured_steps` set to 10 failed with the WR-01 error, and a two-scene `full_catalog: false` copy failed with "0 changed, 23 missing" plus the IN-05 hint.

---

_Fixed: 2026-10-14_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

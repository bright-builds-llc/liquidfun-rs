---
phase: 46-cartesian-diver
fixed_at: 2026-10-14T19:43:49Z
review_path: .planning/phases/46-cartesian-diver/46-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 46: Code Review Fix Report

**Fixed at:** 2026-10-14T19:43:49Z
**Source review:** .planning/phases/46-cartesian-diver/46-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1 (critical_warning scope; 6 Info findings excluded)
- Fixed: 1
- Skipped: 0

## Fixed Issues

### WR-01: A tap outside the bottle releases a squeeze latched by the Squeeze action

**Files modified:** `crates/liquidfun-wasm/src/scene/cartesian_diver.rs`, `crates/liquidfun-wasm/src/scene/cartesian_diver/tests.rs`
**Commit:** b1f0a5ddc
**Status:** fixed: requires human verification (input-state logic change)
**Applied fix:** Added a `pointer_held` flag to `CartesianDiverHooks`. A `Down` inside the squeeze region sets both `pointer_held` and `squeezed`; `Up`/`Cancel` clear `squeezed` only when `pointer_held` is true; all other pointer events are ignored and still return `Ok`. Added regression test `tap_outside_the_bottle_keeps_a_latched_squeeze` (latch via `apply_action(SQUEEZE_ACTION)`, then `Down`+`Up` at `(2.5, 3.0)`, assert still squeezed). Existing `pointer_up_anywhere_releases` / `pointer_cancel_anywhere_releases` still pass, since those releases follow an inside press.

**Verification:**
- `cargo fmt --all` clean
- `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` clean
- `cargo test -p liquidfun-wasm`: 635 passed, 0 failed, 36 ignored
- `cargo test -p liquidfun-wasm --release cartesian_diver`: 45 passed, 0 failed (includes the release-only motion tests)
- The fingerprint survey drives the scene with `Action("squeeze")`, not pointers, so this pointer-only change cannot alter the cartesian-diver fingerprint; `just scene-fingerprint-gate` was not re-run.

**Behavior note for the human check:** If a squeeze is latched by the action and the user then presses inside the bottle and lets go, the release clears the squeeze. This matches the "latest input wins" model and the hint, but confirm that it is the intended feel.

---

_Fixed: 2026-10-14T19:43:49Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

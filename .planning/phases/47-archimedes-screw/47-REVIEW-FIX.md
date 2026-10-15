---
phase: 47-archimedes-screw
fixed_at: 2026-10-15T03:14:40Z
review_path: .planning/phases/47-archimedes-screw/47-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 47: Code Review Fix Report

**Fixed at:** 2026-10-15T03:14:40Z
**Source review:** .planning/phases/47-archimedes-screw/47-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7
- Fixed: 7
- Skipped: 0

## Fixed Issues

### IN-04: The web-to-Rust sync test matches substrings and misses the minimum, step, and direction tokens

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs`
**Commit:** d54a629d2
**Applied fix:** Replaced `web_speed_constants_match_rust` with a new `ts_const` helper that parses `const NAME = <value>;` from the web speed file. Four tests use it:
- `web_speed_default_and_maximum_match_rust` compares the parsed f32 bits of the default and the maximum with `DEFAULT_SCREW_RPM` and `MAX_SCREW_RPM`.
- `web_speed_minimum_and_step_match_whole_rpm_tokens` checks that MIN is 0 and STEP is 1.
- `web_direction_options_are_forward_then_reverse` reads `scene-records-curiosity.ts` with `include_str!`, takes the `option("<token>"` tokens from the `screw-direction` `runtimePreset` block, and checks that they are exactly `["forward", "reverse"]`.
- `every_web_direction_option_is_a_live_rust_control` sends each web token through `apply_control` and expects `ControlEffect::Live`.

### IN-05: The `VANE_OVERHANG` doc comment describes half the value

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs`
**Commit:** 1da585552
**Applied fix:** The comment now reads "Axial lean of a vane from its bottom tip to its top tip."

### IN-07: `advance_phase` can return exactly `PITCH` because of f32 rounding

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs`
**Commit:** 5e71e5a99
**Applied fix:** When `next < 0`, if `next + PITCH` rounds up to `PITCH`, the function now returns `(0.0, 0.0)`: phase 0 is the same lattice, so no snap is needed. Otherwise it returns `(wrapped, PITCH)` as before. The new test `advance_phase_stays_below_a_pitch_when_the_wrap_rounds_up` first checks that `-1e-8 + PITCH` really rounds to `PITCH`. It then checks that the result is in `[0, PITCH)` with no snap. Normal trajectories never reach this branch, so the scene fingerprint should not change. The orchestrator's fingerprint gate must still confirm this. Status: fixed, but the logic needs human verification.

### IN-01: Snap-housing tests check one phase each, not the full range of snap poses

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw/tests.rs`
**Commit:** e923e7966
**Applied fix:** Replaced the two single-pose tests with range tests built on a `corners_outside` helper, using `max_step = MAX_AXIAL_SPEED * SIM_DT`:
- vane 0 at `[-max_step, 0, max_step]` must lie inside the bottom housing;
- vane `VANE_COUNT - 1` at `[PITCH - max_step, PITCH, PITCH + max_step]` must lie inside the top housing.

### IN-02: Vanes are drawn from the stored phase, so frame checks cannot see body drift

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs`
**Commit:** 3f01c6cb3
**Applied fix:** After every step, `drive` now asserts that the train body origin is within `LATTICE_TOLERANCE` of `train_position(hooks.phase)`. The new test `the_train_body_follows_the_drawn_phase_at_the_maximum_speed_both_ways` drives at `MAX_SCREW_RPM`, forward and reverse, for 150 steps; that is the speed where `clamp_speed` could cause lag. A mutation check confirmed the new test fails: shrinking the clamp to 0.9 times `MAX_AXIAL_SPEED` made it fail. The mutation was reverted and never committed.

### IN-03: The reverse-motion test compares positions only modulo a pitch

**Files modified:** `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs`
**Commit:** ac647648c
**Applied fix:**
- **Reverse test:** checks the train's velocity along `AXIS` against `axial_speed(DEFAULT_SCREW_RPM, true)` within 1e-5. It also checks that the velocity across the axis is zero and that both `hooks.phase` and the body's axial offset equal `PITCH - step`. Nothing is reduced modulo a pitch any more.
- **Forward move test** (`hooks_tests.rs`): the modulo reduction is gone. The test now asserts that the expected travel is below `PITCH` and compares the travel directly.

### IN-06: Duplicated rpm parser and tube-membership test

**Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene/washing_machine.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs`
**Commit:** 29cf2bf15
**Applied fix:**
- **Shared parser:** added `scene::parse_whole_rpm(value, max)`, with the same logic as both old parsers. `parse_drum_rpm` and `parse_screw_rpm` are removed, and both scenes now call `parse_whole_rpm` with their own maximum. The screw control tests keep a local `parse_screw_rpm` wrapper.
- **Tube test:** `in_tube` is no longer test-only, and `wanted_water` now calls it instead of repeating its body.

Both refactors keep the logic exactly the same. The washing machine tests pass (8/8).

## Verification

- Before each commit: `cargo fmt --all`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` (clean), and `cargo test -p liquidfun-wasm --lib archimedes_screw`. The debug run ends at 51 passed, 16 ignored.
- `cargo test -p liquidfun-wasm --lib washing_machine`: 8 passed.
- `cargo test -p liquidfun-wasm --release --lib archimedes_screw -- --include-ignored`: 67 passed, 0 failed.
- `cargo check -p liquidfun-wasm --target wasm32-unknown-unknown --lib`: clean.
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- Not run here (the orchestrator runs them): the web checks and the scene fingerprint gate.

---

_Fixed: 2026-10-15T03:14:40Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

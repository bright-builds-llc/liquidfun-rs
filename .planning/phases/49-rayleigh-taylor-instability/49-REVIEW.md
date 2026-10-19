---
phase: 49-rayleigh-taylor-instability
reviewed: 2026-10-19T03:23:25Z
depth: standard
files_reviewed: 33
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene/particle_force.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/flow_tests.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/force_lane_tests.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/geometry.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/hook_tests.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/sync_tests.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/test_support.rs
  - crates/liquidfun-wasm/src/scene/rayleigh_taylor/tests.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - docs/adding-a-scene.md
  - web/e2e/player-helpers.ts
  - web/scripts/demo-media/model.ts
  - web/scripts/readme-svg/plans.ts
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/preview-kit.tsx
  - web/src/catalog/previews-curiosity.tsx
  - web/src/catalog/previews.tsx
  - web/src/catalog/rayleigh-taylor-ratio.ts
  - web/src/catalog/scene-records-curiosity.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scene-view-bounds.ts
  - web/src/catalog/scenes.ts
  - web/src/components/scene-controls.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/rayleigh-taylor.test.ts
  - web/tests/scene-catalog-controls.test.ts
  - web/tests/scene-catalog-credits.test.ts
  - web/tests/scenes.test.ts
findings:
  critical: 0
  warning: 0
  info: 6
  total: 6
status: issues_found
---

# Phase 49: Code Review Report

**Reviewed:** 2026-10-19T03:23:25Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** issues_found

## Summary

Scope: the Rayleigh–Taylor Instability scene from `fa11f9b00^` to `HEAD`. That covers the new `rayleigh_taylor` Rust module and its geometry, its test suites and test support, the catalog registration hunks in shared Rust and web files, the web catalog record, ratio control, view and portrait bounds, preview, capture and README plans, and the `docs/adding-a-scene.md` R16 row. For the large shared files, only the changed hunks were reviewed.

The production code is sound:

- **Presets:** `ratio_from_presets` fails closed on unknown names and tokens, and the last-wins rule matches the session's replace semantics.
- **Settle and ramp:** `ramp_factor` is monotonic and saturating, and a zero `ramp_steps` is harmless because `inf.min(1.0)` is 1.
- **Gravity:** `on_advance` reads the live gravity each step and skips zero or non-finite gravity through `GravityFrame::from_gravity`.
- **Hooks:** the pointer and action seams behave correctly. `apply_control` only returns `Recreated`, and it never mutates the world.
- **Rust and web agree:** the colors, option ids, default token and view frame match, and `sync_tests.rs` guards them.
- **Repo rules:** every per-scene site in CLAUDE.md and `docs/adding-a-scene.md` is covered: README SVG plan, portrait frame, capture plan, e2e hash path, scene survey, gravity-slider index, and a fingerprint baseline in its own commit (`9e7e61cfc`). Production code has no `unwrap()`, and every error is propagated.

Checks run during review:

- `cargo test -p liquidfun-wasm --lib rayleigh_taylor`: 61 passed, 19 ignored. The ignored tests are release-only and were not run here.
- `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings`: clean.
- `npm run typecheck` in `web/`: clean.
- `npx vitest run` on the five touched test files: 32 passed.

No critical or warning-level issues. The six Info items all concern test-helper robustness, stale comments, or duplication in the test-only code.

## Info

### IN-01: `ripple_depth` returns a `NEG_INFINITY` sentinel that can make S6 pass vacuously

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/test_support.rs:254-267` (consumed at `crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs:300-309`)
**Issue:** When no interior bin holds at least `MIN_FINGER_PARTICLES` heavy particles, `ripple_depth` returns `f32::NEG_INFINITY`. If that happens for `ripple_depth_at_0`, then `depth - report.ripple_depth_at_0` is `+inf`, so `default_ratio_ripple_deepens_within_one_second` passes with no evidence. The current lattice fills every bin at step 0, so the guard is not live today. It would quietly stop guarding anything if the lattice or bins change.
**Fix:** Return an `Option<f32>` and make the S6 test require both samples:

```rust
pub(super) fn ripple_depth(layers: &Layers, frame: GravityFrame, interface: f32) -> Option<f32> {
    // ...same binning...
    bins[1..FINGER_BIN_COUNT - 1]
        .iter_mut()
        .filter(|heights| heights.len() >= MIN_FINGER_PARTICLES)
        .map(|heights| {
            heights.sort_by(f32::total_cmp);
            interface - heights[MIN_FINGER_PARTICLES - 1]
        })
        .reduce(f32::max)
}
```

### IN-02: Stale bin-width comment and hard-coded bin bounds in the finger metric

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/test_support.rs:27-28, 47-50, 104`
**Issue:** The `FINGER_BIN_WIDTH` doc says "two particle diameters at r 0.04", but the locked tuning uses r 0.035, where 0.16 m is about 2.3 diameters. `bin_index` clamps to the literal `16.0`, and the const assert uses the literals `17.0` and `18.0`, instead of deriving them from `FINGER_BIN_COUNT`. Changing the bin count would leave the clamp and the assert out of sync, and `past_counts` would then index out of bounds or never fill the last bin.
**Fix:** Reword the comment, for example "about two particle diameters at the locked radius". Derive the bounds from the count, e.g. `offset.clamp(0.0, f32::from(u16::try_from(FINGER_BIN_COUNT - 1).expect("fits")))` or a `const LAST_BIN: f32 = 16.0;` next to `FINGER_BIN_COUNT` with a const assert that ties the two together.

### IN-03: The spike's S4 stability sampling skips the settle window, unlike its documented rule

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs:21-26, 136, 157-165`
**Issue:** The S4 doc says containment and speed are checked "at every 30th step" over 0..1,200. `run` only starts sampling at `settle_steps + 1` (step 37), so the step-30 sample never happens. The spike's `max_speed_early` and `max_out_of_tank` therefore ignore the settle. `flow_tests` runs through `run_session_stability` and does sample step 30, so the gap only affects the spike lane.
**Fix:** Record stability samples inside `settle` as well, or reword S4 to say that sampling starts after the settle window.

### IN-04: The wall-thickness test name promises more than it checks

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/tests.rs:165-177`
**Issue:** `walls_and_lid_are_at_least_three_diameters_thick` only asserts `thickness >= WALL_THICKNESS`. That is tautological, because the quads are built from `WALL_THICKNESS`. The test also skips the floor (`quads[0]`). The real three-diameter guarantee is the const assert at `rayleigh_taylor.rs:126`.
**Fix:** Assert against `3.0 * 2.0 * TUNING.radius` and iterate every quad, or rename the test to say what it actually checks, such as `tank_quads_use_the_wall_thickness`.

### IN-05: Duplicate release-only 60 s runs across the spike and flow suites

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs:260-268` and `crates/liquidfun-wasm/src/scene/rayleigh_taylor/flow_tests.rs:194-206`
**Issue:** Both suites run a 3,600-step ratio-3 run with the same `assert_bounded` check. The fingers-by-240 and overturn-by-600 assertions at the default ratio are also checked in both suites, once on bare parts and once through the session. The flow lane adds only the session path and the `live == built` check. Each release run of `rayleigh_taylor` pays for the extra 60 s simulation.
**Fix:** Keep one ratio-3 long run, the session-path one in `flow_tests` because it also checks eviction, and remove the spike copy or mark it record-only.

### IN-06: The test particle cap duplicates the production constant

**File:** `crates/liquidfun-wasm/src/scene/rayleigh_taylor/hook_tests.rs:16`
**Issue:** `MAX_PARTICLES = 3_000` restates `MAXIMUM_PARTICLE_COUNT` from `rayleigh_taylor.rs:36` (`tests.rs:23` repeats it as `MAX_LATTICE_POINTS`). If the budget changes, the copies can drift apart.
**Fix:** Import `super::MAXIMUM_PARTICLE_COUNT` in both test modules and drop the local copies.

---

_Reviewed: 2026-10-19T03:23:25Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

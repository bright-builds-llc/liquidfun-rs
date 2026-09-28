---
phase: 33-stacked-drip-fidget
reviewed: 2026-09-28T02:27:16Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene/stacked_drip.rs
  - crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs
  - crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - web/e2e/player-helpers.ts
  - web/e2e/shell.spec.ts
  - web/scripts/demo-media/model.ts
  - web/scripts/readme-svg/plans.ts
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/previews.tsx
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scenes.ts
  - web/src/player/runtime.ts
  - web/tests/demo-media-model.test.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/scene-catalog-controls.test.ts
  - web/tests/scene-catalog-credits.test.ts
  - web/tests/scenes.test.ts
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues
---

# Phase 33: Code Review Report

**Reviewed:** 2026-09-28T02:27:16Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues

## Summary

Reviewed the stacked-drip scene, its shaft and tray geometry, session and gravity-slider registration, and the playground catalog, portrait, preview, README, capture, and test wiring that adds `stacked-drip`.

The three trays are motor-off revolute bodies, the plate schedule stays at speed 0 through the 5 s cascade window, and the catalog lists stay exhaustive for twenty-three scene ids. The return path does not destroy or teleport particles.

The sloped left floor ends on the divider's inner face, leaving a floor slot beside the plate that the lift cannot cover. The return test only requires one original particle to reappear above the top tray, so that slot can strand the rest of the drip without a test failure.

## Warnings

### WR-01: Slope ends short of the plate, so liquid can pool off the lift

**File:** `crates/liquidfun-wasm/src/scene/stacked_drip.rs:43-46` and `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs:57-63`
**Issue:** The 0.02 m side seals are measured from the divider's outer face (`x = 0.98`) and the right wall. The left floor does not reach that seal. `floor_wedge` drops to `y = 0.08` at `DIVIDER_INNER_X` (`x = 0.90`), then the shaft floor continues at `y = 0` from `x = 0.90`. The resting plate starts at `x = 1.00`, `y = 0.02..0.06`. The uncovered slot is about 0.10 m wide (the 0.08 m divider thickness plus the 0.02 m seal) and deep enough for a 0.05 m particle to rest entirely left of the plate, under the divider overhang, where the prismatic lift never reaches it. D-05 asks the cascade to continue for the session. `return_lifts_an_original_particle_above_the_top_tray` only requires one original id to be back above the top tray, so a stranding leak still passes.
**Fix:** Carry the ramp under the divider until its low lip meets the plate, and do not leave a floor at `y = 0` between the divider's inner face and the plate. Keep the 0.02 m seal between the divider's outer face and the plate.

```rust
const SLOPE_END_X: f32 = PLATE_LEFT_X;
const SLOPE_END_Y: f32 = PLATE_REST_TOP + PARTICLE_DIAMETER;

// floor_wedge low lip lands on the plate, not on the divider's inner face.
// Fill or raise the shaft floor so x in (DIVIDER_INNER_X, PLATE_LEFT_X) is not a pocket.
```

Assert in the return test that every original id is either back above the top tray or still on the plate (`x >= PLATE_LEFT_X`), not sitting on the shaft floor at `x < PLATE_LEFT_X`.

## Info

### IN-01: Compile-time assert compares seconds with radians

**File:** `crates/liquidfun-wasm/src/scene/stacked_drip.rs:84`
**Issue:** `assert!(PROOF_SECONDS > ANGLE_FLOOR)` compares 5.0 seconds with 0.05 radians. It stays true and does not lock the dwell or the tray-angle floor.
**Fix:** Drop that assert. `assert!(DWELL > PROOF_SECONDS)` already keeps the plate in its dwell during the cascade window.

---

_Reviewed: 2026-09-28T02:27:16Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

---
phase: 46-cartesian-diver
reviewed: 2026-10-14T19:36:07Z
depth: standard
files_reviewed: 31
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/cartesian_diver.rs
  - crates/liquidfun-wasm/src/scene/cartesian_diver/geometry.rs
  - crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs
  - crates/liquidfun-wasm/src/scene/cartesian_diver/tests.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot/tests.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - docs/adding-a-scene.md
  - docs/benchmarks/scene-fingerprints.json
  - web/e2e/canvas-callout.spec.ts
  - web/e2e/player-helpers.ts
  - web/e2e/player.spec.ts
  - web/scripts/demo-media/model.ts
  - web/scripts/readme-svg/plans.ts
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/previews-curiosity.tsx
  - web/src/catalog/previews.tsx
  - web/src/catalog/scene-records-curiosity.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scene-view-bounds.ts
  - web/src/catalog/scenes.ts
  - web/src/components/PlayerPanel.tsx
  - web/src/styles/player-viewport.css
  - web/tests/canvas-callout.test.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/readme-svg.test.ts
  - web/tests/scene-catalog-controls.test.ts
  - web/tests/scene-catalog-credits.test.ts
  - web/tests/scenes.test.ts
findings:
  critical: 0
  warning: 1
  info: 6
  total: 7
status: issues_found
---

# Phase 46: Code Review Report

**Reviewed:** 2026-10-14T19:36:07Z
**Depth:** standard
**Files Reviewed:** 31
**Status:** issues_found

## Summary

Reviewed the new Cartesian Diver scene (`cartesian_diver.rs`, `geometry.rs`, both test
modules) in full, plus the phase 46 diff (`7b489b4ef..HEAD`) for every shared Rust, web,
docs and test file in scope.

The Rust code follows the project rules. It has no `unwrap()` in production code, errors
propagate through `map_err`, `TAU` is used for the half-sine dent, the files stay under
628 lines, and the tests use Arrange/Act/Assert. The density ramp is exact (integer steps
`0..=15`) and runs once per fixed step, because `SessionCore::advance` calls `on_advance`
inside its per-step loop. The drawn bubble stays inside the diver for any gravity
direction, including zero. The registration sites R2 through R10 and W1 through W6 are
complete and in catalog order. The fingerprint baseline grew in its own commit
(`18c8a5e92` touches only `scene-fingerprints.json`). The caption and hint avoid the
forbidden claims, and both say that the dent and bubble are drawn.

The one behavioral issue is in the pointer input model. A pointer release anywhere clears
a squeeze that the Squeeze action latched, so a tap outside the bottle does change the
scene. That contradicts the hint text. The remaining items are Info-level: test coverage,
duplicated constants, and demo media.

## Warnings

### WR-01: A tap outside the bottle releases a squeeze latched by the Squeeze action

**File:** `crates/liquidfun-wasm/src/scene/cartesian_diver.rs:228-235`
**Issue:** `apply_pointer` ignores `Down` outside the hit region, but `Up | Cancel` always
sets `squeezed = false`. Suppose the user turns on Squeeze with the HUD button or the
keyboard, and the diver sinks. Any tap on the canvas, even far outside the bottle, then
sends `Down` (ignored) followed by `Up` (which releases), and the diver rises. The catalog
hint in `web/src/catalog/scene-records-curiosity.ts:12` says "Presses outside the bottle do
nothing" and "use Squeeze again, and it rises". So the copy no longer matches the
behavior (SCENE-03 caption honesty). `pointer_up_anywhere_releases` only covers an `Up`
that follows an inside `Down`, so it does not catch this case.
**Fix:** Track whether the current pointer gesture began inside the bottle, and release
only that gesture:

```rust
struct CartesianDiverHooks {
    // ...
    squeezed: bool,
    /// True while a press that began inside the bottle is held.
    pointer_held: bool,
    squeeze_steps: u8,
}

match kind {
    PointerKind::Down if geometry::in_squeeze_region(Vec2::new(world_x, world_y)) => {
        self.pointer_held = true;
        self.squeezed = true;
    }
    PointerKind::Up | PointerKind::Cancel if self.pointer_held => {
        self.pointer_held = false;
        self.squeezed = false;
    }
    PointerKind::Down | PointerKind::Up | PointerKind::Cancel | PointerKind::Move => {}
}
```

Add a regression test: latch with `apply_action(SQUEEZE_ACTION)`, send `Down` then `Up` at
`(2.5, 3.0)`, and assert that `squeezed` is still `true`.

## Info

### IN-01: The squeeze hit region extends 0.45 m past the drawn bottle

**File:** `crates/liquidfun-wasm/src/scene/cartesian_diver/geometry.rs:54-56`
**Issue:** The hit region is the physical outer walls plus 0.3 m (x = ±1.9, y = -0.7..6.3).
The physical walls are not drawn; the visible outline sits at x = ±1.45 and
y = -0.25..5.85. A press up to 0.45 m beside the visible bottle therefore still squeezes.
D-02 chose this margin on purpose. Still, "Presses outside the bottle do nothing" is
only approximately true on a phone, where 0.45 m is about 55 CSS px.
**Fix:** Either soften the hint (for example, "Presses away from the bottle do nothing"),
or measure the margin from the drawn outline (`OUTLINE_HALF_WIDTH`, `OUTLINE_BOTTOM_Y`,
`OUTLINE_TOP_Y`) instead of from the undrawn outer walls.

### IN-02: The demo-media plan clicks for a press-and-hold interaction

**File:** `web/scripts/demo-media/model.ts:288-294`
**Issue:** `action: { kind: "click" }` runs `page.mouse.click`, which sends `down` and then
`up` right away. The squeeze ramps for at most a step or two and then reverses, so the
generated demo clip shows no sink. The capture only asserts that `pointerAccepted` went
up, so it still passes.
**Fix:** Add a `hold` action kind (`mouse.down`, wait N steps, `mouse.up`), or use the
labeled `Squeeze` control twice, as the README plan already does.

### IN-03: The release-only motion tests run in no gate or CI job

**File:** `crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs:1-4`
**Issue:** Every sink, rise and containment test is ignored under `debug_assertions`. Neither
the `justfile` nor `.github/workflows` runs `cargo test --release`, so the scene's central
behavior (D-07 and D-07a timing) is checked only by hand. The fingerprint gate catches
trajectory drift, but not whether the diver still sinks and rises. The new R13 row in
`docs/adding-a-scene.md` documents this pattern without a recipe. `level_proofs.rs` is an
existing precedent.
**Fix:** Add a recipe such as
`scene-motion: cargo test -p liquidfun-wasm --release --lib cartesian_diver`, and link it
from the R13 row so that phase gates can cite it.

### IN-04: The README cue schedule is duplicated between Rust and TypeScript with no cross-check

**File:** `crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs:33-38`, `web/scripts/readme-svg/plans.ts:83-89`
**Issue:** `README_SQUEEZE_STEP = 60` and `README_RELEASE_STEP = 330` are hand-derived from
`atSample: 20 / 110` times `STEPS_PER_SVG_SAMPLE = 3`. If the plan cues or the sample rate
change, the Rust test still passes against stale numbers. `readme-svg.test.ts:116-128`
only restates the plan literals.
**Fix:** In `readme-svg.test.ts`, assert the derived step numbers
(`cue.atSample * STEPS_PER_SVG_SAMPLE` equals `[60, 330]`) with a comment that points at
the Rust constants, so that a change on either side breaks a test.

### IN-05: A tautological test checks constants against their own definitions

**File:** `crates/liquidfun-wasm/src/scene/cartesian_diver/tests.rs:517-523`
**Issue:** `thirds_split_the_interior_height` asserts `BOTTOM_THIRD_Y == INTERIOR_HEIGHT / 3.0`
and `TOP_THIRD_Y == INTERIOR_HEIGHT * 2.0 / 3.0`, which is exactly how those constants are
defined. Only the `const { assert!(DIVER_START.y >= TOP_THIRD_Y) }` line checks anything.
**Fix:** Keep only the const assertion, or rename the test to
`diver_starts_in_the_top_third`.

### IN-06: Squeeze is a toggle that does not show its state

**File:** `web/src/catalog/scene-records-curiosity.ts:13`
**Issue:** `action("squeeze", "Squeeze")` renders a plain action button. In Rust it toggles
`squeezed`, but the button has no `aria-pressed` attribute and no label change. Keyboard
and screen-reader users cannot tell whether the diver is latched. This gets more
confusing in combination with WR-01. Soup Stirrer's "Toggle paddle rail" is an earlier
example of the same pattern.
**Fix:** Either note the toggle in the hint (it already says "use Squeeze again"), or, in a
later change, add a toggle control kind that reflects the latched state with
`aria-pressed`.

---

_Reviewed: 2026-10-14T19:36:07Z_
_Reviewer: Claude (gsd-code-reviewer), an independent AI review, not a human approval_
_Depth: standard_

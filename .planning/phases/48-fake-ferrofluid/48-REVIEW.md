---
phase: 48-fake-ferrofluid
reviewed: 2026-10-15T07:14:47Z
depth: standard
files_reviewed: 35
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/control_tests.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/flow_tests.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/spike_tests.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/sync_tests.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/tests.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene/particle_force.rs
  - crates/liquidfun-wasm/src/scene/particle_force/tests.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - docs/adding-a-scene.md
  - web/e2e/player-helpers.ts
  - web/e2e/player.spec.ts
  - web/scripts/demo-media/model.ts
  - web/scripts/readme-svg/plans.ts
  - web/src/catalog/fake-ferrofluid-strength.ts
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/preview-kit.tsx
  - web/src/catalog/previews-curiosity.tsx
  - web/src/catalog/previews.tsx
  - web/src/catalog/scene-records-curiosity.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scene-view-bounds.ts
  - web/src/catalog/scenes.ts
  - web/src/components/range-control.ts
  - web/tests/fake-ferrofluid-controls.test.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/scene-catalog-controls.test.ts
  - web/tests/scene-catalog-credits.test.ts
  - web/tests/scenes.test.ts
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 48: Code Review Report

**Reviewed:** 2026-10-15T07:14:47Z
**Depth:** standard
**Files Reviewed:** 35
**Status:** issues_found

## Summary

I reviewed the Fake Ferrofluid scene, its field, geometry and tests, the new shared
`particle_force` helper, and the phase 48 changes to the shared Rust and web files
(`git diff c5462cb40^..HEAD`). I also checked the locked decisions in 48-CONTEXT.md.

The structure is sound. The pointer, action and control hooks only store intent, and the pull
runs once per fixed step in `on_advance` (D-08). Every field term is built from `GravityFrame`
(D-14). The strength control fails closed. Non-finite pointers are rejected in the session. The
web and Rust constants have `include_str!` sync tests (R14). The catalog, preview, portrait,
README and e2e sites are all filled in. I found no security problems and no crash paths.

The main concerns are in the field model and its test coverage:

1. The sideways term never fades with height.
2. The 60 s hold tests cover only seven fixed magnet positions. The clamp lets a user drag the
   magnet over a much larger area at full strength.
3. The demo-media capture's center click turns the magnet off for most of the capture.

## Warnings

### WR-01: Sideways pull does not fade with height, so a far-away magnet still clumps the puddle

**File:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs:77-85, 143-145`

**Issue:** The sideways term is `scale * |E(h)| * weight_slope`. Past the reach, `E(h)` levels
off at `±8H/15` (`bump_integral` lines 80-82) and does not fall to zero. As a result, a magnet
clamped near the top of the frame, such as `(0, 3.5)`, still pushes the puddle sideways. The
puddle sits about 3.2 m below that magnet, which is twice the 1.6 m reach. Near a column flank
(`t ≈ 0.19`), the uncapped sideways term at full strength is about
`25 * 0.853 * 0.73 * 5.9 ≈ 90 m/s²`. The cap cuts this to 15 m/s² (1.5 g). The lift there is
zero.

So the liquid gathers into heaps at the column centers while the magnet is nowhere near it. This
goes against D-02 ("a smooth falloff to an outer radius") and the D-01 picture of a pull that
weakens with distance. `lift_vanishes_beyond_the_reach` (tests.rs:216-237) checks only `.y`, so
it does not catch this. At its `(0.65, reach + 0.4)` sample, the sideways component is non-zero.

**Fix:** Multiply the sideways term by a smooth fade in `h` that is 1 inside the reach and 0 by a
far radius. For example:

```rust
/// Fades the sideways pull from 1 at the reach to 0 at twice the reach.
fn far_fade(h: f32, reach: f32) -> f32 {
    let x = (h.abs() - reach) / reach;
    if x <= 0.0 { return 1.0; }
    bump(x.min(1.0))
}
let lateral = scale * bump_integral(h, params.reach).abs()
    * far_fade(h, params.reach) * weight_slope;
```

Add a test that, with the magnet more than about 2 times the reach above or below a point,
`magnet_acceleration` returns `Vec2::ZERO`. Then rerun the release flow and spike suites, because
the curl-free test above the magnet is affected only past the reach.

### WR-02: Containment is not tested along drag paths or at clamp extremes at full strength

**File:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs:24-29, 104-115`;
`crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs:224-238`

**Issue:** `clamp_magnet` allows magnet centers up to `y = 3.5` and `|x| = 3.2`, which is
outside the inner wall faces at 3.0. The hold and column-top tests use only fixed positions up to
`y = 1.5`, applied from load. The lift `bump(h/H)` is even in `h`. At 100 % strength the cap is
`1.5 G`, so liquid within about `|h| < 0.97 m` of the magnet's column centers has a net upward
acceleration. That means a blob can float around the magnet itself.

A user who drags slowly upward from `(0, 1.3)` can likely carry the column that the test already
measured at 2.67 m above the 3.0 m wall tops, then sideways to `x = 3.2`, and release it there.
Part of the blob would then fall outside the dish and be evicted. This would break D-09 ("100 …
never detaches liquid") and the D-23 no-eviction rule. It is not verified: no test moves the
magnet during a hold, and none holds it above 1.5 m.

**Fix:** Add a release-only test that holds a press at full strength and moves the magnet in
small steps: from rest, up to `(0, 2.5)`, then across to `(3.2, 2.5)`, then releases. It should
assert `out_of_dish_count == 0` and no evictions for 5 s after release. If it fails, use either
or both of these:

1. Clamp the magnet's height to `WALL_TOP_Y - FIELD.reach` and its x to the inner faces in
   `clamp_magnet`.
2. Make the lift fade out above the magnet. For example, use `bump(h/H)` only for `h <= 0` and a
   faster decay for `h > 0`, so the liquid can't be lifted past the magnet.

### WR-03: The demo-media capture's center click switches the magnet off for the rest of the capture

**File:** `web/scripts/demo-media/model.ts:302-308`

**Issue:** D-21 says this capture entry should be idle. The entry is a canvas
`{ kind: "click", point: { x: 0.5, y: 0.5 } }` at step 180. A click sends `down` and then `up`.
`apply_pointer` turns the magnet on at the click point, then parks it and switches it off
(`fake_ferrofluid.rs:244-248`). So about 5 s of the 8 s capture shows a collapsing, flat puddle
instead of the spikes. The other entries use the same click, but in this scene a click is not
neutral.

**Fix:** Either accept this on purpose and note it in the plan summary, or capture a gesture that
still shows the spikes. One option is a `drag` whose start and end are both under the dish. The
release still parks the magnet, but the capture then shows spikes, a drag, a collapse and a
re-rise if the window is long enough. Another option is a new capture action kind that runs the
`magnet` panel action twice. If the current click stays, update D-21's "idle" wording to match.

## Info

### IN-01: The strength control is parsed with an RPM-named helper

**File:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs:205`

**Issue:** `parse_whole_rpm(value, MAX_STRENGTH_PERCENT)` parses a percentage. The behavior is
correct (canonical whole numbers up to the maximum), but the name is misleading now that it has
non-RPM callers.

**Fix:** Rename the shared helper to `parse_whole_number` (or add a `parse_whole_percent`
wrapper) in `scene.rs`, and update its callers.

### IN-02: The "allocates nothing after the first step" doc claims are inaccurate

**File:** `crates/liquidfun-wasm/src/scene/particle_force.rs:87-88, 125-137`;
`crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs:72`

**Issue:** `ParticleScratch` reuses the lane copies, but `apply_accelerations` calls
`apply_particle_force_range` once per particle. Each of those calls builds a `Vec` in the
engine's `force::prepare_force` (`crates/liquidfun/src/particle/force.rs:84-89`). That is one
heap allocation per affected particle per step, about 1,000 for this scene. The cost itself is
out of scope here. The doc comments overstate the guarantee.

**Fix:** Change both comments to say only the lane copies are reused. If the step budget becomes
tight, consider a per-particle force API that does not allocate.

### IN-03: `apply_group_acceleration` fails above 65,535 members

**File:** `crates/liquidfun-wasm/src/scene/particle_force.rs:171`

**Issue:** `u16::try_from(members.len())` returns `StepFailed` for groups larger than
`u16::MAX`. This shared helper is meant for Phase 49 reuse, and the limit is not documented. The
current scene budgets are far below it, so this only matters for future callers.

**Fix:** Convert through `u32` with a documented precision note, or add the limit to the
function's `# Errors` section.

### IN-04: `apply_accelerations` can leave forces half-applied when it fails

**File:** `crates/liquidfun-wasm/src/scene/particle_force.rs:125-137`

**Issue:** If one particle's force call fails (for example, a wall particle in a future caller's
system), the particles before it have already gathered force for this step. The function then
returns `StepFailed`, which the session treats as fatal, so this does not matter today. It is
still worth documenting for reuse.

**Fix:** Add a line to the `# Errors` doc: "Forces already applied to earlier particles in this
call are not rolled back."

### IN-05: The spike and flow suites repeat the same release-mode holds

**File:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/spike_tests.rs:219-280`;
`crates/liquidfun-wasm/src/scene/fake_ferrofluid/flow_tests.rs:391-439`

**Issue:** The same five 60 s full-strength holds run in `spike_tests`, again through the session
in `flow_tests`, and a third time inside `column_tops_stay_three_tenths_below_the_wall_tops`
(seven positions). `run_appearance` also runs four times for four tests. That adds up to about 17
one-minute simulations plus O(n²) isolation checks. It is not a correctness problem, but it slows
the release suite and repeats one concern across tests.

**Fix:** Have `column_tops_…` reuse the hold reports, or memoize per-position reports with a
`OnceLock`. Alternatively, remove the parts-level hold duplicates now that the session-level
holds exist.

---

_Reviewed: 2026-10-15T07:14:47Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

# Phase 33: Stacked drip fidget - Research

**Researched:** 2026-09-27
**Domain:** Original playground scene. Colored water leaves a top reservoir, tips three motor-off trays from top to bottom, and a slow side-shaft plate returns the same particles to the reservoir.
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Stack
- **D-01:** Add a new scene. Do not replace, hide, or retune `liquid-timer`, `liquid-bubbler`, `water-wheel`, `hydraulic-fountain`, or `liquid-tumbler`. The catalog id is `stacked-drip`, appended after `liquid-bubbler`. The title is Stacked Drip.
- **D-02:** The layout is a top reservoir and three dynamic trays stacked vertically. Liquid leaves the reservoir and lands on the upper tray, then the middle tray, then the lower tray. Do not use Liquid Timer's static shelves, Liquid Bubbler's single waist and wheel, or a column of wheels.

#### Tray reaction
- **D-03:** Each tray is a dynamic body on a revolute joint with the motor off. Liquid that reaches a tray tips that tray and pours onto the next. Do not tip a tray with a timed motor, and do not animate the tip in SolidJS.
- **D-04:** A rest stop or joint limit holds each tray until liquid arrives, then the tray tips one way and pours. Trays do not free-spin before the drip. The upper tray moves before the lower trays, so the cascade reads from top to bottom.

#### Repeat
- **D-05:** The cascade continues for the whole play session. A quiet return lifts liquid back to the top reservoir. The visible spectacle stays the stack of tipping trays. The return is not a piston show and does not add a wheel. Do not destroy or respawn particles to fake the loop. Reset restores the initial reservoir, the resting trays, and the initial particle layout.

#### Liquid color
- **D-06:** Use one plain water particle group with one distinct `ParticleColor`, so the drip reads as colored liquid. Do not add tensile, elastic, rigid, or color-mixing groups. Do not retune Color Mixer or Liquid Timer.
- **D-07:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

#### Catalog, credits, and proof
- **D-08:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/stacked-drip`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-09:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the reservoir, all three trays, and the walls fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `stacked-drip` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-10:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-11:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, after a bounded run, particles that began above the top tray are below the bottom tray, and each tray's angle has changed from its initial pose, with the upper tray moving before the lower trays. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-12:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact reservoir size, tray length, pivot placement, rest angle, joint limits, particle radius, particle count, and the single particle color, as long as three trays tip in top-to-bottom order and the walls stay inside the portrait frame.
- How the return lift is built, as long as particles physically reach each tray before that tray tips, the return does not become the spectacle, and particles are not deleted to fake the loop.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a vertical stack of trays under a colored drip.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Visitor tray, color, or speed controls.
- Replacing or retuning Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, or Liquid Tumbler.
- A stack of wheels, or more than three reacting parts.
- Two-color mixing through the trays.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| — | `phase_req_ids` is null. Do not invent requirement IDs. Locked decisions D-01 through D-12 are the scope. | `.planning/ROADMAP.md` Phase 33 still says "To be planned" and "Requirements: TBD". Use `33-CONTEXT.md`, not that stub. v1.3 rows including `PARITY-01` stay future and unmapped. [VERIFIED: `.planning/REQUIREMENTS.md`; `.planning/ROADMAP.md`] |

This phase is an original playground scene. Its gate is D-11, not a requirements-matrix row.
</phase_requirements>

## Summary

Three trays already fit the public joint API. `RevoluteJointDef::new` leaves `enable_motor` and `enable_limit` false. `with_limits(true, lower, upper)` adds a one-sided angular stop without turning the motor on. A dynamic body receives particle contact, which includes an angular term, so liquid sitting on an off-center deck can rotate the tray. A joint limit only blocks travel past the bound. It does not hold an empty tray whose center of mass already wants to pour. Each tray therefore needs a counterweight fixture on the catch side of the pivot so the empty body presses into the lower limit, and a deck whose loaded mass tips it toward the upper (pour) limit. After the liquid leaves, the counterweight returns the tray. That tray return is not a revolute motor and is not the liquid lift. [VERIFIED: `RevoluteJointDef::new`, `with_limits`, `with_motor` in `crates/liquidfun/src/joint/definition/revolute_prismatic.rs`; angular coupling in `crates/liquidfun/src/world/particle_coupling/body_coupling.rs`]

Copy the liquid return from Liquid Bubbler, not from Hydraulic Fountain. `LiquidBubblerHooks::on_advance` calls `set_prismatic_motor_speed` with `scheduled_plate_speed`: speed `0` for `DWELL` (3 s), then `PLATE_SPEED` (0.15 m/s) up, then down. The plate is a dynamic box on a vertical `PrismaticJointDef` with `with_collide_connected(true)`, `with_limits`, and `with_motor(true, ...)`. The water group uses `with_destruction_by_age(false)` and `ParticleGroupRecipe::with_color`. There is one `create_particle_group` and no `set_particle_position`. The stacked-drip plate uses that schedule and that particle policy. The differences are: three limited revolute trays instead of one unlimited wheel, every revolute motor stays off, the shaft sits beside the tray stack so the plate sweep does not enter any tray's pour envelope, and the cascade proof finishes while plate translation is still about `0`. [VERIFIED: `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`; `liquid_bubbler/tests.rs`]

**Primary recommendation:** Add `stacked-drip` as a watch-first scene: one teal water group in a small top reservoir, three dynamic trays on motor-off revolute joints limited from rest (`0`) to a pour angle, each with a catch-side counterweight, and a side-shaft plate that stays down through a dwell longer than the cascade proof, then creeps upward at about `0.15` m/s. Prove the cascade with `SessionCore::advance(4)` only: every original particle that started above the top tray is below the bottom tray and left of the shaft, each `revolute_joint_angle` has left its rest pose, and the upper tray's first motion sample is earlier than the middle tray's, which is earlier than the lower tray's. Prove the lift in a later plan, the way Phase 32 plan 04 opened the bubbler shaft.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory is present, and this repo has no `.cursor/skills/` or `.agents/skills/` indexes. [VERIFIED: globs of those paths]. These repo rules still bind the planner:

- Physics stays in `liquidfun`. The WASM crate is the scene shell. SolidJS stays controls and rendering. [VERIFIED: `33-CONTEXT.md`; `standards/core/architecture.md`]
- A new catalog id also needs a portrait frame in `web/src/catalog/portrait-bounds.ts` and a README SVG plan in `web/scripts/readme-svg/plans.ts`. Do not mdformat `.planning/**`. [VERIFIED: `AGENTS.md` README scene gallery and playground canvas framing]
- Safe Rust, no `unwrap()` on the production path, `foo.rs` plus `foo/` if the scene module is split, never `foo/mod.rs`. File-length automation fails at 629 physical lines (`floor(100 * tau)` is the mnemonic, about 628). Tests use arrange, act, assert, and one concern each. [VERIFIED: `standards/languages/rust.md`; `standards/core/code-shape.md`; `standards/core/testing.md`]
- The playground shell stays the existing dark Kobalte Dialog. Do not adopt MysticUI for this scene. [VERIFIED: `standards-overrides.md`; `standards/core/frontend-ui.md`]
- Hobby scope: no Linux qualification, no package publication, no sealed parity. [VERIFIED: `PROJECT-SCOPE.md`]
- `just web-player-smoke` is the Chromium proof. Do not add a browser lane. [VERIFIED: `justfile` `web-player-smoke`; `standards/core/verification.md`]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace, Rust 1.97.0 | Revolute limits, prismatic plate, particle group | Scene bodies already use these defs. No new engine API. [VERIFIED: `rustc 1.97.0`; `revolute_prismatic.rs`] |
| `liquidfun-wasm` | workspace | `SceneId`, `build_scene`, `SessionCore::advance` | Every playground scene is built here. [VERIFIED: `scene.rs`] |
| SolidJS catalog | existing `web/` | Record, preview, portrait frame | D-08 keeps `PlaygroundShell`. [VERIFIED: `scene-records.ts`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Playwright Chromium | existing `just web-player-smoke` | Opens, pauses, plays, resets | D-11 only. The recipe is `bun scripts/web-build.ts player-smoke`. |
| Vitest via Bun 1.4.2 | `cd web && bun run test:unit` | Catalog order, credits, portrait, README plan coverage | After the catalog files exist. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Joint-limit rest on a counterweighted tray | A static peg the tray leans on | The peg is a contact stop. Phase 32 had to raise the plate by one polygon skin (`POLYGON_RADIUS` = `2 * LINEAR_SLOP` = 0.01 m) because a flush contact popped translation off 0. A limit does not have that contact. [VERIFIED: `settings.rs`; `STATE.md` Phase 32 plate note] |
| Bubbler side-shaft plate | Hydraulic Fountain piston schedule | Fountain's `scheduled_motor_speed` moves on every half period with no dwell, on a horizontal axis, and the piston is the spectacle. Do not copy it. [VERIFIED: `hydraulic_fountain.rs`] |
| Motor-off revolute | Water Wheel joint plus jet | Water Wheel's `pin_wheel` is motor-off, but `emit_jet`, `with_destruction_by_age(true)`, and `with_lifetime` empty the scene. Do not copy the jet or the wheel. [VERIFIED: `water_wheel.rs`] |
| Plain `ParticleFlags::WATER` group | Liquid Timer tensile shelves | `create_tensile_viscous_slab` sets `TENSILE \| VISCOUS` and `attach_shelf_edges` uses static `EdgeShape` shelves. Do not copy either. [VERIFIED: `liquid_timer.rs`] |

**Installation:** none. Do not add a crate, a npm package, or a Python script.

**Version verification:** `rustc 1.97.0`, `cargo 1.97.0`, `bun 1.4.2`, `just 1.48.0` on this machine on 2026-09-27. [VERIFIED: local `--version`]

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun-wasm/src/scene.rs          # mod stacked_drip; SceneId; parse; build_scene
crates/liquidfun-wasm/src/scene/stacked_drip.rs
crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs
crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs   # only if stacked_drip.rs nears 628 lines
web/src/catalog/scenes.ts
web/src/catalog/scene-records.ts
web/src/catalog/previews.tsx
web/src/catalog/portrait-bounds.ts
web/scripts/readme-svg/plans.ts
web/scripts/demo-media/model.ts
```

`links.ts` `isAllowlistedScenePath` accepts `crates/liquidfun-wasm/src/scene/<file>.rs` only when the remainder has no slash. The credit path must stay `crates/liquidfun-wasm/src/scene/stacked_drip.rs`. Tests and a vessel table may live under `stacked_drip/` because they are `mod tests` / `mod vessel` from that file, not the credit path. Do not add `stacked_drip/mod.rs`. [VERIFIED: `web/src/catalog/links.ts`]

`liquid_bubbler.rs` is 490 lines today. Three tray bodies, three limit defs, a counterweight each, walls, and a plate will pass 500 and can reach the 629-line gate. Split the static wall table into `stacked_drip/vessel.rs` before `stacked_drip.rs` reaches 629 physical lines. Keep `build`, the hooks, and `on_advance` in `stacked_drip.rs`. `previews.tsx` is 498 lines and `scene-records.ts` is 526. A compact preview the size of `LiquidBubblerPreview` (about 16 lines) stays under 629. Do not grow either file with a second component module unless a `file-lengths` run fails. [VERIFIED: `wc -l`; `standards/core/code-shape.md`]

### Pattern 1: Three counterweighted trays on limited revolutes

**What:** Each tray is `BodyDef::new(BodyType::Dynamic, pivot, 0.0, true)` with `with_angular_damping` and `with_sleeping_allowed(false)`, matching `create_wheel`. The body origin is the pivot. One box fixture is the deck, extending mostly to the pour side. A second fixture on the same body, on the catch side, is heavy enough that the empty tray's moment presses into the lower limit. `RevoluteJointDef::new(ground, tray).with_frame(pivot, Vec2::ZERO, 0.0).with_limits(true, 0.0, POUR)` and no `with_motor`. Stagger the three pivots so each pour lip sits above the next tray's deck, not above a gap, and every lip swings away from the side shaft.

**When to use:** D-02, D-03, and D-04.

**Example:**

```rust
// Source: crates/liquidfun/src/joint/definition/revolute_prismatic.rs
//         crates/liquidfun-wasm/src/scene/liquid_bubbler.rs pin_wheel
let joint = RevoluteJointDef::new(ground, tray)?
    .with_frame(pivot, Vec2::ZERO, 0.0)?
    .with_limits(true, 0.0, pour_angle)?;
// Do not call with_motor. new() already sets enable_motor to false.
world.create_joint(JointDef::from(joint))?;
```

Read each angle with `world.revolute_joint_angle(joint)`. Store the three `JointId`s on the hooks. Tests should order them by pivot `y` (highest pivot is the upper tray), not by assuming vector order if a later edit reorders construction.

### Pattern 2: Quiet side-shaft lift, copied from the bubbler with three named differences

**What:** Copy `create_plate`, `create_plate_joint`, and `scheduled_plate_speed` from `liquid_bubbler.rs`. `on_advance` adds `SIM_DT` and calls `set_prismatic_motor_speed`. It must not call `set_revolute_motor_speed`.

**When to use:** D-05. The cascade proof runs entirely inside the dwell.

**Differences from Liquid Bubbler:**

1. The revolute joints are the trays, and each one calls `with_limits`. The bubbler wheel calls only `with_frame` and can free-spin. These trays must not free-spin. Do not copy `pin_wheel` as the whole joint.
2. There is no wheel, no hub, and no paddles. Do not copy `create_wheel` or `PADDLE_POLYGONS`.
3. The divider and the plate stay to the side of every tray's swept pour. The bubbler plate only had to miss one wheel. Here the plate and the rising column must miss three rotating decks. Pour every tray away from the shaft.

**Keep from the bubbler, because they are what makes the loop physical:**

- `with_destruction_by_age(false)`. The particle-system default is `destroy_by_age: true`. [VERIFIED: `system_definition.rs`]
- No `create_particle_with_def`, no `set_particle_position`, no `with_color_mixing_strength`. Default recipe flags are `ParticleFlags::WATER`. Color mixing runs only with `COLOR_MIXING`. [VERIFIED: `recipe.rs` default flags; Phase 32 source note on `material.rs`]
- `with_collide_connected(true)` on the plate so it does not tunnel through the shaft floor.
- `PLATE_FLOOR_CLEARANCE` of `2.0 * 0.01` so polygon skin does not pop the plate off translation 0. [VERIFIED: `liquid_bubbler.rs` comment]
- Plate motor force may be `1.0e6`. Tray revolute motors stay disabled. The plate motor is the lift, and it writes speed `0` during the dwell so it is not the spectacle.
- Advance through `SessionCore::advance(4)`. `advance` calls `on_advance` before `world.step`. A direct `World::step` skips the plate schedule. [VERIFIED: `session.rs` `advance`]

Do not copy `hydraulic_fountain.rs` `scheduled_motor_speed` (horizontal axis, no dwell, piston is the show) or `water_wheel.rs` `emit_jet`.

### Pattern 3: One plain water group, small enough to finish inside the dwell

**What:** `ParticleSystemDef::default().with_radius(...).with_damping(...).with_destruction_by_age(false)`, then one `ParticleGroupRecipe` with `with_color` and `Transform::IDENTITY`. Radius starts at `0.025`, the bubbler value. Diameter is `0.05`. Group spacing follows `PARTICLE_STRIDE` (`0.75`) times diameter, so about `0.0375` m. [VERIFIED: `settings.rs`; `liquid_bubbler.rs` `create_water_group`]

**When to use:** D-06 and D-11. D-11 requires the particles that began above the top tray to be below the bottom tray after a bounded run. That is the whole starting set, not "at least one," because the spectacle is the drain through all three trays. The bubbler test `drip_crosses_the_waist_and_turns_the_wheel` only required one id below the waist, and that reservoir is `0.88` m by `0.38` m. A slab that large will not clear three trays before a short dwell ends. Use a shallow reservoir, on the order of a few tenths of a meter, so the starting set drains while the plate translation is still about `0`. If `0.025` cannot fit the walls, keep the count and add a source comment that starts with `playground adaptation:`. Do not delete particles to pass the test, and do not raise `MAX_ADVANCE_STEPS`.

Use a color other than the bubbler's amber `ParticleColor::new(242, 176, 64, 255)` so the static preview does not clone Liquid Bubbler. A teal `ParticleColor::new(64, 196, 196, 255)` is a starting choice. [ASSUMED]

### Pattern 4: Catalog append, watch-first

**What:** Append after every `liquid-bubbler` entry. The record matches Liquid Bubbler: `ready: true`, `interactionHint: WATCH_FIRST_HINT`, `controls: withGravitySlider([])`, `credits.implementationPath` the single scene file, `credits.inspiration: [SHOWCASE]` only. `SHOWCASE` is the Google LiquidFun showcase href, not a pinned test. [VERIFIED: `scene-records.ts`; `scene-record-shared.ts`]

`DemoNavigation` already renders the caption `Static preview`. Do not put that string inside the SVG. [VERIFIED: `web/src/components/DemoNavigation.tsx`]

`WATCH_FIRST_SCENE_IDS` in `web/e2e/player.spec.ts` is `SCENE_IDS` minus `POINTER_CONTROL`. Do not add `stacked-drip` to `POINTER_CONTROL`. Gravity is not a pointer gesture.

### Pattern 5: SceneId wiring, including the length-22 locks

Touch these together so the catalog cannot advertise a scene the session cannot build:

1. `scene.rs`: `mod stacked_drip;`
2. `SceneId::StackedDrip` after `LiquidBubbler`
3. `parse_scene_id`: `"stacked-drip" => Ok(SceneId::StackedDrip)` after the `liquid-bubbler` arm
4. `build_scene`: `SceneId::StackedDrip => stacked_drip::build(&scene_presets)`
5. `build` returns `SessionError::UnknownControl` when `presets` is non-empty, after gravity has already been stripped by `split_gravity_preset`. [VERIFIED: `build_scene` in `scene.rs`; `liquid_bubbler::build`]
6. `gravity_slider.rs` test: `all_scene_ids() -> [SceneId; 23]`, `seen` becomes `[false; 23]` compared with `[true; 23]`, and `variant_index` adds `SceneId::StackedDrip => 22`. Missing this fails `every_scene_builds_downward_gravity_from_the_slider`. [VERIFIED: current array length 22 ends at `LiquidBubbler`]
7. `session/tests.rs` `parse_scene_id_maps_allowlisted_tokens`: append `("stacked-drip", SceneId::StackedDrip)`
8. `web/src/catalog/scenes.ts` `SCENE_IDS`: append `"stacked-drip"` after `"liquid-bubbler"` (current length 22)
9. `scene-records.ts`: record after the `liquid-bubbler` object. Description is one behavior plus the experimental sentence, in the shape of the bubbler description.
10. `previews.tsx`: `case "stacked-drip"` and a compact SVG of a reservoir, a drip, and three tilted trays. No wheel.
11. `portrait-bounds.ts`: `PORTRAIT_VIEW_BOUNDS["stacked-drip"]` equal to the record `viewBounds`
12. `web/scripts/readme-svg/plans.ts`: `{ id: "stacked-drip", cues: [] }` immediately after the liquid-bubbler plan. Leave `readmeControls` empty for this id. Do not run `just readme-svg`.
13. `web/scripts/demo-media/model.ts`: capture plan after liquid-bubbler, `{ id: "stacked-drip", title: "Stacked Drip", route: "/liquidfun-rs/#/scene/stacked-drip", interactionStep: 180, action: { kind: "click", point: { x: 0.5, y: 0.5 } } }`. `assertSceneCapturePlanCoverage` requires `SCENE_CAPTURE_PLANS` in `SCENE_IDS` order. [VERIFIED: `model.ts`]
14. `web/e2e/player-helpers.ts` `SCENE_HASH_PATHS`: `"stacked-drip": "/liquidfun-rs/#/scene/stacked-drip"`
15. `web/src/player/runtime.ts` `PAGE_SUMMARY`: "All twenty-three demos". The footer prints `PAGE_SUMMARY`. [VERIFIED: `SiteFooter.tsx`]
16. Credits test map: implementation path `crates/liquidfun-wasm/src/scene/stacked_drip.rs`, inspiration `[SHOWCASE]` only. [VERIFIED: `scene-catalog-credits.test.ts` pattern for `liquid-bubbler`]

Length locks that must move from 22 to 23:

- `web/tests/scenes.test.ts`: `toHaveLength(22)`, the ordered id list ending `liquid-bubbler`, the watch-first list, the description map, and the test titles that say "twenty-two"
- `web/tests/demo-media-model.test.ts`: `toHaveLength(22)` and last id `"liquid-bubbler"`
- `web/e2e/shell.spec.ts`: "All twenty-two demos", sidebar `toHaveCount(22)` for captions and `svg[aria-hidden="true"]`, drawer focus math (see Pitfall 5)
- `gravity_slider.rs` as in step 6

`particle_iterations` stays the default arm. Do not give this scene Liquid Tumbler's extra substeps.

### Anti-Patterns to Avoid

- **Timed tray motor:** `with_motor` or `set_revolute_motor_speed` on a tray makes the tip a clock, which D-03 rejects.
- **Copying `pin_wheel` without limits:** the bubbler wheel free-spins. These trays need `with_limits`.
- **Copying Liquid Timer:** static `EdgeShape` shelves and `ParticleFlags::TENSILE | ParticleFlags::VISCOUS`.
- **Copying the fountain piston as the show:** no horizontal squeeze, no immediate motion, no throat.
- **A wheel, a jet, or a fourth reacting body.**
- **`set_particle_position` or destroying and recreating the group to fake the loop.**
- **SolidJS animating tray angles.** The player only renders the WASM frame.
- **A credit path with a slash,** or a pinned `google/liquidfun` test URL in inspiration.
- **Raising `MAX_ADVANCE_STEPS` or `MAX_STEPS_PER_FRAME`.** Both are `4`. [VERIFIED: `session.rs`; `player-helpers.ts`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hold the tray until liquid arrives | A timer that sets angular velocity | `RevoluteJointDef::with_limits` plus a catch-side counterweight | A timer ignores the liquid. A limit blocks one direction. The counterweight supplies the rest torque the limit itself does not. |
| Repeat the drip | Delete particles or write positions | Bubbler prismatic plate in `on_advance` | D-05 counts the same particles. `advance` already calls `on_advance`. |
| One tint | A second group or color mixing | `ParticleGroupRecipe::with_color` on the default water flags | Mixing needs `COLOR_MIXING`. A second group is a deferred idea. |
| Portrait phone fit | A new camera mode | `PORTRAIT_VIEW_BOUNDS` and `worldBoundsForViewport` | `portrait-bounds.test.ts` already contain-fits a 390 by 844 canvas. |
| README coverage | A hand-written SVG in this phase | `{ id: "stacked-drip", cues: [] }` | `assertReadmeSvgPlanCoverage` reads `plans.ts`. Raster export is not the gate. |

**Key insight:** The tip is a moment about a limited pivot. The loop is a slow plate outside the trays. Neither needs a new solver entry point.

## Common Pitfalls

### Pitfall 1: Trays tip before the liquid arrives
**What goes wrong:** All three angles leave rest in the first batches, so the order test fails and the scene reads as three falling planks.
**Why it happens:** `with_limits` does not pull the body back to the lower bound. If the deck's center of mass is on the pour side of the pivot, gravity leaves the lower limit at once. `with_sleeping_allowed(false)` keeps that motion from being slept away. [VERIFIED: `RevoluteJointDef::with_limits` only stores the range]
**How to avoid:** Put a counterweight fixture on the catch side. Start each body at angle `0` with lower limit `0` so the empty moment presses into `AtLower`. Keep the pour limit large enough that a loaded deck can reach it (a starting pour of about `tau/12` to `tau/8`). Tune counterweight size before touching particle radius.
**Warning signs:** The upper tray's first sample past the angle floor is batch 0, or a lower tray moves in the same batch as the upper tray.

### Pitfall 2: The return tunnels through the trays
**What goes wrong:** The plate or the rising column shoves a tray, or particles reappear on a tray without having crossed the reservoir.
**Why it happens:** Phase 32 plan 01 built a plate, and plan 04 still had to open the shaft. A divider that stops above the floor lets particles slide under it into the stack. A plate wider than the shaft hits the divider. A tray that pours toward the shaft sweeps through the lift. [VERIFIED: `32-04-PLAN.md`]
**How to avoid:** Shaft on one side, all three pour lips on the other. Divider from the floor up to a spill lip at the reservoir. Plate gap to the divider wider than one diameter; Phase 32 used `0.10` m, which is two diameters at radius `0.025`. Plate top at full stroke above the spill lip. Proof of the lift is plan 04, not the cascade proof.
**Warning signs:** During the dwell, plate translation is already above `0.01`. An original id returns above the top tray while still on the shaft side of the divider.

### Pitfall 3: The cohort cannot finish before the plate moves
**What goes wrong:** The drain test fails because particles are still on the upper trays, or someone raises `MAX_ADVANCE_STEPS` or shrinks the group by deleting bodies in the test.
**Why it happens:** `PROOF_BATCHES` of `30` calls to `advance(4)` is 2 seconds (`30 * 4 / 60`). Bubbler `DWELL` is `3.0`, which is enough for one waist and not for three pours of a large slab. `advance` rejects `step_count` outside `1..=4`. [VERIFIED: `liquid_bubbler/tests.rs` `advance_proof`; `session.rs`]
**How to avoid:** Keep radius `0.025`. Size the reservoir so the whole starting set is below the bottom tray inside the dwell. Lengthen `DWELL` (a starting value of `6.0` seconds) rather than shortening the proof or speeding the plate. Keep `PLATE_SPEED` at `0.15`. The cascade test samples the first batch each tray crosses an angle floor of `0.05` rad, the bubbler `ANGLE_FLOOR`, and asserts upper batch < middle batch < lower batch, all inside the dwell, with plate translation still under `0.01`.
**Warning signs:** Plate speed is non-zero at the end of the cascade test. Live count changes.

### Pitfall 4: The scene file crosses the length gate
**What goes wrong:** `bun scripts/bright-builds-check.ts file-lengths` fails at 629 lines.
**Why it happens:** `liquid_bubbler.rs` is already 490 lines for one wheel and one plate. Three trays add three bodies and three joints.
**How to avoid:** `stacked_drip.rs` plus `stacked_drip/vessel.rs` and `stacked_drip/tests.rs`. Credit path stays the parent `.rs` file.
**Warning signs:** `wc -l crates/liquidfun-wasm/src/scene/stacked_drip.rs` prints 600 or more before the vessel table has been extracted.

### Pitfall 5: Portrait frame clips walls, or a Playwright locator collides
**What goes wrong:** The phone canvas cuts a wall or a raised plate, or a link named with a short regex activates the wrong scene.
**Why it happens:** A tall phone contain-fits whatever rectangle it is given. The shared 12 m by 9 m frame becomes a short band. `shell.spec.ts` still uses `/Static preview Fountain\b/` because `/Fountain/` also matched Hydraulic Fountain. `/Liquid/` would match Liquid Timer, Liquid Tumbler, and Liquid Bubbler. [VERIFIED: `AGENTS.md` portrait rule; `shell.spec.ts` line for Fountain; `32-03-PLAN.md`]
**How to avoid:** Portrait points include the reservoir, both faces of each tray at rest and at the pour angle, the divider, the floor, the side walls, and the plate corners at translation `0` and at full stroke. Start `MIN_HEIGHT_FRACTION["stacked-drip"]` at `0.40`, the bubbler value for a tall vessel, and include the rest and poured tray corners in `wallEndpoints`. The drawer last link is `/Stacked Drip\b/`. Do not use `/Drip/` or `/Liquid/`. Leave the Fountain locator as `/Static preview Fountain\b/`. New counts: 23 scene links plus the Dismiss button is 24 focusables; the wrap loop uses 25 Tab presses and 25 Shift+Tab presses (`25 ≡ 1 (mod 24)`), matching the current "24 tabs, 23 focusables, last is Liquid Bubbler" arithmetic. Sidebar caption and hidden-svg counts become 23. Footer copy becomes "All twenty-three demos".
**Warning signs:** `portrait-bounds.test.ts` fails a wall point on the 390 by 844 viewport. Smoke clicks Hydraulic Fountain or Liquid Bubbler when the intended link is Stacked Drip.

### Pitfall 6: Age destruction empties the toy
**What goes wrong:** Live count drops, so the return test cannot find an original id.
**Why it happens:** `ParticleSystemDef::default` sets `destroy_by_age: true`. Water Wheel also sets lifetimes. Escape eviction deletes particles only past 48 m or 12 m below the authored ground, so a 2 m vessel does not hit it if the walls hold. [VERIFIED: `system_definition.rs`; `session/escape.rs`]
**How to avoid:** `with_destruction_by_age(false)`. No lifetime. The source guard test rejects `create_particle_with_def` and `set_particle_position`.
**Warning signs:** `live_particle_count` after the proof is less than the starting count.

## Code Examples

### Tray angle the order test can read

```rust
// Source: crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs revolute_angle
world
    .revolute_joint_angle(joint)
    .expect("the tray angle should be readable")
```

Call this from `session.read_particles` during `advance(4)` batches. Record the first batch whose absolute delta from the step-0 angle is at least `0.05`. Assert upper < middle < lower. Assert each final absolute delta is at least `0.05`. Assert step-0 angles are `0` (`to_bits`).

### Cohort below the bottom tray

```rust
// Source: liquid_bubbler/tests.rs drip_crosses_the_waist_and_turns_the_wheel
// Strengthen the bubbler "any" check to "every starting id".
started_above.iter().all(|id| {
    view.particle_ids()
        .iter()
        .zip(view.positions())
        .any(|(live, position)| {
            live == id && position.y < bottom_tray_deck_y && position.x < divider_inner_x
        })
})
```

Also assert `end_count == start_count`. The plate-stay assertion from `plate_stays_down_during_the_proof` belongs in the same proof window: translation abs under `0.01`, motor speed near `0`.

### Recommended starting numbers

These are a geometry hypothesis, not a measured run. [ASSUMED]

| Constant | Start | Role |
|----------|-------|------|
| `PARTICLE_RADIUS` | `0.025` | Same as the bubbler. Shrink only with a `playground adaptation:` comment. |
| `PARTICLE_DAMPING` | `0.2` | Bubbler water |
| `DRIP_COLOR` | `(64, 196, 196, 255)` | Distinct from bubbler amber |
| `GRAVITY` | `(0, -10)` | `build_scene` may overwrite this from the gravity preset |
| Tray deck | about `0.50` m long, `0.02` m thick, density `1` | Deck mass on the pour side of the pivot |
| Counterweight | on the catch side, density high enough to hold the empty deck | Rest torque into the lower limit |
| `POUR` | about `std::f32::consts::TAU / 8.0` | One-way tip. Lower limit `0`. |
| Vertical pitch | about `0.45` m between pivots | Next deck catches the pour |
| `DWELL` | `6.0` | Longer than the cascade proof |
| `PLATE_SPEED` | `0.15` | Bubbler. Do not exceed `0.30` if a later open-shaft plan tunes the lift. |
| Shaft inlet gap | at least `0.10` m | Wider than one diameter (`0.05`) |
| `ANGULAR_DAMPING` | `0.4` | Higher than the wheel's `0.05` so a tipped tray settles instead of oscillating through the next pour. [ASSUMED] |

If the lower trays move before the upper tray, increase counterweight mass or move the pivot toward the pour lip. Do not enable a revolute motor. If the loaded deck never reaches `0.05` rad, lower deck density or lengthen the pour-side deck, keeping the lip at least `0.08` m from the walls, the same clearance Phase 32 used for paddle tips.

### Source guard

Copy `source_lifts_with_a_prismatic_plate` from `liquid_bubbler/tests.rs`. Require `set_prismatic_motor_speed` and `with_destruction_by_age(false)`. Reject `set_revolute_motor_speed`, `create_particle_with_def`, `set_particle_position`, and `with_color_mixing_strength`.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 32 plan stub said "To be planned" | `32-CONTEXT.md` plus four plans, including a return-shaft plan | 2026-09-27 | Phase 33's roadmap stub is the same kind of placeholder. Plan from `33-CONTEXT.md`. |
| Bubbler wheel free-spins on an unlimited revolute | Tray revolute calls `with_limits` | This phase | `pin_wheel` is the motor-off constructor, not the rest stop. |
| Fountain piston moves immediately | Bubbler plate dwells, then creeps | Phase 32 | Stacked drip copies the dwell schedule. |
| `/Fountain/` matched Hydraulic Fountain | `/Static preview Fountain\b/` | Phase 30 smoke | Stacked Drip locators stay `\b`-anchored full titles. |

**Deprecated/outdated:**

- Roadmap Phase 33 "Goal: [To be planned]" and "Requirements: TBD". The context file is the scope.
- Treating Phase 32's "at least one particle below the waist" as the D-11 drain. D-11 here is the starting cohort below the bottom tray, because the scene is the three-tray drain.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | A catch-side counterweight plus `with_limits(true, 0.0, POUR)` holds an empty tray until particles land, without a revolute motor | Pattern 1, Pitfall 1 | If the solver limit does not oppose an empty moment, use a heavier counterweight first. A static peg is the fallback, with `POLYGON_RADIUS` clearance so it does not stick. Do not add a tray motor. |
| A2 | A reservoir a few tenths of a meter across drains through three trays inside a 6 s dwell at radius `0.025` and plate speed `0.15` | Pattern 3, Pitfall 3 | If the cohort is still above the bottom tray, widen pour angles or gaps. Do not cut radius without a `playground adaptation:` comment, and do not raise `MAX_ADVANCE_STEPS`. |
| A3 | Teal `(64, 196, 196, 255)` and `ANGULAR_DAMPING` `0.4` are acceptable discretion choices | Pattern 3, starting numbers | Color and damping can change as long as there is one `ParticleColor` and the trays still tip in order. |
| A4 | Pour angle near `TAU/8` and `0.45` m pitch clear the next deck | Starting numbers | If liquid misses the next tray, stagger the pivots horizontally. Keep three trays and no wheel. |

**If A1 through A4 fail in the cascade test:** tune geometry inside plan 01. Do not retune `liquid_bubbler.rs`, `liquid_timer.rs`, `water_wheel.rs`, `hydraulic_fountain.rs`, or `liquid_tumbler.rs`.

## Open Questions

1. **Will the whole starting set clear three trays before a 6 s dwell ends?**
   - What we know: One waist cleared at least one particle in 2 s with a larger reservoir and radius `0.025`. The plate stayed down for `DWELL` `3.0`. [VERIFIED: bubbler tests and constants]
   - What's unclear: Three sequential pours of a smaller slab. No run was executed in this research.
   - Recommendation: Plan 01 owns the cascade proof and may lengthen `DWELL` or the slab. It must not raise the step cap.

2. **Does the plate need a second geometry pass to actually return a particle?**
   - What we know: Phase 32 plan 01 included the plate and the dwell. Plan 04 still had to shorten the divider, narrow the plate, and set the stroke so the plate top clears the spill lip, then add `return_lifts_an_original_particle_above_the_waist`. [VERIFIED: `32-04-PLAN.md`; `STATE.md` Phase 32]
   - What's unclear: Whether a carefully open shaft in plan 01 would pass the return test on the first try.
   - Recommendation: Keep a fourth plan. Plan 01 proves the cascade while the plate is down. Plan 04 opens the inlet and proves an original id is back above the top tray, left of the divider, with revolute motors still off and the live count unchanged.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Scene tests | ✓ | 1.97.0 | — |
| bun | Catalog unit tests and smoke script | ✓ | 1.4.2 | — |
| just | `just web-player-smoke` | ✓ | 1.48.0 | — |
| Playwright Chromium | D-11 | Installed by the smoke script | — | No second browser |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none. Chromium comes from the existing smoke recipe. Do not add Firefox or Safari.

No new service, database, or C++ oracle build is required. `cargo test -p liquidfun-wasm` does not need the LiquidFun submodule. [VERIFIED: workspace guidance that default members stay Cargo-only]

## Validation Architecture

`workflow.nyquist_validation` is `false` in `.planning/config.json`. This is not a Nyquist matrix. The phase gates are the existing commands:

- Native cascade: `cargo test -p liquidfun-wasm scene::stacked_drip -- --test-threads=1`
- Gravity allowlist: `cargo test -p liquidfun-wasm scene::gravity_slider -- --test-threads=1`
- Parse list: `cargo test -p liquidfun-wasm parse_scene_id_maps_allowlisted_tokens -- --test-threads=1`
- Catalog: `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts tests/scene-catalog-credits.test.ts tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts`
- Browser: `just web-player-smoke`

Leave `ALL_SCENE_TIMEOUT_MS` at `380000` unless this Chromium run times out. Phase 32 left it there after a 38.5 second pass. Leave both step caps at 4. [VERIFIED: `player-helpers.ts`; `.planning/STATE.md`]

## Security Domain

`workflow.security_enforcement` is absent from `.planning/config.json`. The spawn instructions treat that absence as enabled. This scene has no accounts, no secrets, and no network calls.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | Physics stays in `liquidfun`. The scene is a local WASM shell. No new trust boundary. |
| V2 Authentication | no | Local playground scene. No account. |
| V3 Session Management | no | `SessionCore` is a physics session, not a user session. |
| V4 Access Control | no | No privilege boundary. |
| V5 Input Validation | yes | `"stacked-drip"` is one new `parse_scene_id` arm. Other tokens stay `UnknownScene`. Non-empty presets stay `UnknownControl`. Gravity stays `2..=80` via `split_gravity_preset`. Pointer input is a no-op. Credit path stays a single `scene/*.rs` file. [VERIFIED: `scene.rs`; `links.ts`; `liquid_bubbler::build`] |
| V6 Cryptography | no | None. |
| V7 Errors and logging | yes | Construction and step failures stay typed `SessionError` values. Do not log particle positions to a remote sink. |
| V8 Data protection | no | No stored personal data. |
| V9 Communications | no | No fetch and no WebSocket from this scene. |
| V10 Malicious code | yes | Do not add `unsafe`. Do not load remote scripts from the preview SVG. |
| V11 Business logic | yes | Tray motors stay off while the catalog says the drip tips the trays. The order test and the source guard lock that. |
| V12 Files and resources | no | No file upload. |
| V13 API | no | No HTTP API. |
| V14 Configuration | yes | Do not raise `MAX_ADVANCE_STEPS`. Do not embed a secret. |

### Known Threat Patterns for this scene

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown scene id or control string | Tampering | Existing parse rejects unknown tokens. This scene adds one known id and no control names besides the shared gravity slider. |
| Credit URL leaving the host allowlist | Spoofing | `sceneBlobUrl` rejects paths outside `crates/liquidfun-wasm/src/scene/*.rs` with no extra slash. Inspiration stays `[SHOWCASE]`. Do not put a Google test URL in the implementation path. |
| Position write or age destruction used as a hidden loop | Tampering | The cascade test requires every original id that started above the top tray to finish below the bottom tray while the plate is still down and the live count is unchanged. |
| Motor-tipped trays presented as a drip | Spoofing | Source guard rejects `set_revolute_motor_speed`. The order test runs during the dwell, so a plate motor at speed 0 cannot be what tips the trays. |

No high-severity finding is expected if the scene stays inside that allowlist and does not add network, secrets, or `unsafe`.

## Suggested Plan Grain

Four plans, matching Phase 32's split. A fourth plan is needed for the return. Phase 32 plan 01 shipped the plate and the dwell, and the original particle still did not re-enter the reservoir until plan 04 opened the shaft. Three sweeping trays make that inlet stricter, not easier. Do not fold the return proof into the cascade plan. Do not write `PLAN.md` from this note. Do not invent requirement IDs.

### Plan 33-01: Trays, dwell, and cascade proof

Files:

- `crates/liquidfun-wasm/src/scene/stacked_drip.rs` (new)
- `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs` (new)
- `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs` only if the parent file nears 628 lines
- `crates/liquidfun-wasm/src/scene.rs`
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` (the length-22 test arrays and `variant_index`)
- `crates/liquidfun-wasm/src/session/tests.rs` (token list)

Tests, one concern each, arrange / act / assert, using `SessionCore::advance(4)` and never `World::step`:

- Step 0: nonzero count, every particle is above the top tray and left of the divider, every velocity is zero, each revolute angle is `0`, plate translation is `0`
- After a bounded run inside the dwell, live count is unchanged, every original id that started above the top tray is below the bottom tray and left of the divider, and each tray angle has left `0` by at least `0.05` rad
- Sampled order: the first batch the upper tray crosses `0.05` is strictly before the middle tray, which is strictly before the lower tray
- At that same sample, plate translation is still near `0` and plate motor speed is `0`
- Three revolute joints have `is_motor_enabled() == false`. Exactly one prismatic joint uses an upward axis, and its body is beside the trays (`x` past the divider), not overlapping a tray
- Rebuild restores angles `0`, plate translation `0`, and the reservoir layout
- Unknown control, unknown action, and a pointer no-op that does not change the live count
- `parse_scene_id("stacked-drip")` succeeds and gravity `10` still builds downward gravity after the preset is stripped
- `include_str` source guard from the bubbler test

Leave `MAX_ADVANCE_STEPS` at 4. Do not edit `liquid_bubbler.rs`, `liquid_timer.rs`, `water_wheel.rs`, `hydraulic_fountain.rs`, or `liquid_tumbler.rs`.

Quick command: `cargo test -p liquidfun-wasm scene::stacked_drip -- --test-threads=1`

### Plan 33-02: Catalog companions

Files, all appended after the liquid-bubbler entries:

- `web/src/catalog/scenes.ts`
- `web/src/catalog/scene-records.ts`
- `web/src/catalog/previews.tsx`
- `web/src/catalog/portrait-bounds.ts`
- `web/src/player/runtime.ts` (`PAGE_SUMMARY` twenty-three)
- `web/scripts/readme-svg/plans.ts`
- `web/scripts/demo-media/model.ts`
- `web/e2e/player-helpers.ts` (`SCENE_HASH_PATHS` only)
- `web/tests/scenes.test.ts`
- `web/tests/scene-catalog-controls.test.ts` (gravity slider only; no tray, color, or speed control)
- `web/tests/scene-catalog-credits.test.ts`
- `web/tests/portrait-bounds.test.ts` (`MIN_HEIGHT_FRACTION` `0.40` to start, plus wall endpoints for reservoir, three trays at rest and at the pour angle, walls, divider, and plate at rest and at full stroke)
- `web/tests/demo-media-model.test.ts` (length 23, last id `stacked-drip`)

`README_SVG_PLANS` and `SCENE_CAPTURE_PLANS` follow `SCENE_IDS` order. README plan cues stay `[]`. Preview shows a vertical stack of trays under a colored drip and is captioned by `DemoNavigation`, not by text inside the SVG. Do not add a `POINTER_CONTROL` entry. Do not run `just readme-svg`.

Quick command: `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts tests/scene-catalog-credits.test.ts tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts`

### Plan 33-03: Chromium smoke

Update `web/e2e/shell.spec.ts` from twenty-two to twenty-three: footer text, 23 static previews, 23 hidden svgs, 24 focusables, 25 Tab and Shift+Tab presses, last link `/Stacked Drip\b/`. Keep `/Static preview Fountain\b/`. Keep `.catalog-card` at count 0. Do not edit `PlaygroundShell.tsx`. Do not add Firefox or Safari. Leave `ALL_SCENE_TIMEOUT_MS` at `380000` unless the run itself times out. Leave both step caps at 4.

The watch-first loop is derived. It will open `#/scene/stacked-drip`, pause, play, and reset once the id is in `SCENE_IDS` and absent from `POINTER_CONTROL`, and it still opens the scenes already in the catalog.

Command: `just web-player-smoke`. README raster and Pages deploy are outside this gate.

### Plan 33-04: Open the return shaft

Depends on plan 01. Change only return geometry in `stacked_drip.rs` and the matching portrait points: inlet wider than one particle, divider spill lip below the raised plate top, plate still beside the trays so it does not sweep a pour. Add `return_lifts_an_original_particle_above_the_top_tray`: after `DWELL + STROKE / PLATE_SPEED` plus a short settle, an original id that started above the top tray is again above it and left of the divider, live count is unchanged, and revolute motors are still disabled. Re-run the plan 01 cascade tests so the dwell proof still sees the plate down. Do not call `set_particle_position` or `set_revolute_motor_speed`. Do not raise `PLATE_SPEED` above `0.30`. Do not shorten `DWELL` below the cascade proof. Update portrait endpoints if the divider or plate moves. Leave `viewBounds` in place if those points already sit inside it.

Command: `cargo test -p liquidfun-wasm scene::stacked_drip -- --test-threads=1` and `cd web && bun run test:unit -- tests/portrait-bounds.test.ts`.

Independent review is after implementation. The implementing agent records evidence and does not approve its own work. A passing cargo test or a passing smoke command is not a review acknowledgment. [VERIFIED: `33-CONTEXT.md` D-12; `AGENTS.md` independent review]

## Sources

### Primary (HIGH confidence)

- `crates/liquidfun/src/joint/definition/revolute_prismatic.rs` — `RevoluteJointDef::new` leaves motor and limits off; `with_limits`; `with_motor`; prismatic `with_limits` and `with_motor`
- `crates/liquidfun/src/world/particle_coupling/body_coupling.rs` — body velocity relative to a particle includes solver angular velocity
- `crates/liquidfun/src/math/settings.rs` — `LINEAR_SLOP` `0.005`, `POLYGON_RADIUS` `0.01`, `PARTICLE_STRIDE` `0.75`
- `crates/liquidfun/src/particle/group/recipe.rs` — default `ParticleFlags::WATER`, `with_color`
- `crates/liquidfun/src/particle/definition/system_definition.rs` — default `destroy_by_age: true`
- `crates/liquidfun-wasm/src/session.rs` — `on_advance` then step, `MAX_ADVANCE_STEPS = 4`
- `crates/liquidfun-wasm/src/session/escape.rs` — 12 m / 48 m deletion
- `crates/liquidfun-wasm/src/scene.rs` — `SceneId` through `LiquidBubbler`, `parse_scene_id`, `build_scene`
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` — `[SceneId; 22]`, `variant_index` ends at `LiquidBubbler => 21`
- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` — motor-off revolute, dwell plate, amber group, radius `0.025`
- `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` — `advance(4)`, angle floor `0.05`, source guard, dwell translation
- `crates/liquidfun-wasm/src/scene/liquid_timer.rs` — tensile flags and static shelf edges to leave unchanged
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — motor-off pin plus jet and lifetime to leave unchanged
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` — always-on horizontal piston to leave unchanged
- `web/src/catalog/scenes.ts`, `scene-records.ts`, `previews.tsx`, `portrait-bounds.ts`, `links.ts`
- `web/src/components/DemoNavigation.tsx` — caption `Static preview`
- `web/src/player/runtime.ts` — `PAGE_SUMMARY` "All twenty-two demos"
- `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`
- `web/e2e/player.spec.ts`, `web/e2e/player-helpers.ts`, `web/e2e/shell.spec.ts`
- `web/tests/scenes.test.ts`, `portrait-bounds.test.ts`, `demo-media-model.test.ts`, `scene-catalog-credits.test.ts`
- `.planning/phases/33-stacked-drip-fidget/33-CONTEXT.md` — locked scope
- `.planning/phases/32-liquid-motion-bubbler/32-04-PLAN.md` — why the return is its own plan
- `.planning/REQUIREMENTS.md` — no new requirement IDs; `PARITY-01` unmapped
- `.planning/config.json` — `nyquist_validation` false
- `standards/core/code-shape.md` — file-length gate at 629 physical lines
- `AGENTS.md` — README plan coverage and portrait frames

### Secondary (MEDIUM confidence)

- `.planning/STATE.md` Phase 32 — wheel density and plate clearance were tuned after the first proof; smoke finished in 38.5 s with the timeout left at 380000
- `crates/liquidfun/src/particle/solver/material.rs` — cited by Phase 32 research as color mixing only when `COLOR_MIXING` is set. Not re-read line by line in this pass.

### Tertiary (LOW confidence)

- None beyond A1 through A4. Those numbers are a starting geometry, not a measured run.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new libraries. Revolute limits, the bubbler plate schedule, and the catalog seams are in tree.
- Architecture: HIGH — limited revolutes plus a delayed side shaft match D-03 through D-05 and the current solver. MEDIUM on the exact counterweight, because the limit API does not itself supply rest torque.
- Pitfalls: HIGH for a motorized tray, a teleport return, age destruction, catalog length locks, and the Fountain locator. MEDIUM for dwell length and pour angle, which the cascade test has to confirm.

**Research date:** 2026-09-27
**Valid until:** 2026-10-27

## RESEARCH COMPLETE

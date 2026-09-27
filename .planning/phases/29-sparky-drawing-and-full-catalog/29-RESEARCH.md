# Phase 29: Sparky, Drawing, and full catalog - Research

**Researched:** 2026-09-26
**Domain:** Native playground scenes (contact sparks, pointer paint, catalog append)
**Confidence:** HIGH

## Summary

Phases 26–28 already ship ten of the twelve missing testbed scenes plus Liquid Tumbler. Phase 29 finishes PLAY-01 by appending `drawing-particles` and then `sparky` after `liquid-tumbler`, without reordering the existing seventeen ids. Sparky and Drawing Particles are the only new WASM scenes. The original six stay on the same allowlist.

Sparky can spark without a new contact-listener crate, without FFI, and without mid-step group spawn. `World::step` already returns `StepReport::contact_transitions()`, including `ContactTransitionKind::Begin`. The session throws that report away and always passes `NoDecisionHook`. Keep `NoDecisionHook`. After the step unlocks, give the scene those transitions and spawn powder there. `WorldCommand` cannot create particle groups. `ContactPointSnapshot` stores impulses, not a world point; recover the point with the public `world_manifold` after the step.

Drawing paint is destroy-then-create on the existing pointer path. `World::destroy_particles_in_shape` already compacts the pocket. Elastic paint reuses `ParticleFlags::ELASTIC` plus `ParticleGroupFlags::SOLID`. There is no public particle-color writer, so fading sparks need one narrow batch color mutator inside `liquidfun`. Do not fade from JavaScript, and do not turn on destruction-by-age.

**Primary recommendation:** Append the two catalog ids, step Sparky with the existing `NoDecisionHook`, spawn and fade powder only after the step returns, and paint Drawing through `destroy_particles_in_shape` plus `create_particle_group` with a non-recreating water/elastic preset.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Sparky contact sparks
- **D-01:** Sparky is a watch-first scene: six dynamic circles in a tall chamber, particle radius about 0.25, walls from `testSparky.js` / `Sparky.h`. Circles are tagged sparkable. Walls are not. Match that layout closely enough to be recognizable.
- **D-02:** Sparks come from new body contacts, not a timer and not persistent resting contact. Record `ContactTransitionKind::Begin` (or the equivalent hook observation) only when at least one fixture belongs to a sparkable circle. Consume that buffer after the step unlocks and spawn VFX then. Do not create or destroy particle groups mid-step. Do not expand FFI or `WorldCommand` to spawn groups inside the step.
- **D-03:** Other scenes keep stepping with `NoDecisionHook`. Only Sparky installs a scene-scoped contact observer. One spark per begin in that step is the preferred spectacle. Upstream's single overwritten contact point per step is the minimum: a step with at least one new sparkable contact must throw at least one burst.
- **D-04:** Each burst is a powder particle group at the contact point, with outward velocities and a short lifetime (about 0.5–1 s). Fade colors in the Rust color buffer in batch. When the lifetime ends, destroy that group. A bounded ring reuses slots by destroying the previous group before overwrite. Do not enable destruction-by-age on the particle system. Reset drops every live VFX slot.

### Drawing Particles paint
- **D-05:** Drawing Particles starts empty inside the open box from `testDrawingParticles.js` / `DrawingParticles.h` (floor, side walls, ceiling strip; particle radius about 0.05). The vessel is recognizable before any paint.
- **D-06:** Pointer down and move paints. Each sample destroys particles inside a circle of radius 0.2 at the pointer, then creates a particle group there. Pointer up clears `lastGroup`. If the joined group is destroyed, clear `lastGroup` too. Reset disposes the session, clears `lastGroup`, and restores the default material. Reuse `World::destroy_particles_in_shape`. Do not enable destruction-by-age.
- **D-07:** A non-recreating runtime preset `material` has `water` (default, no special particle flags) and `elastic` (elastic particles in a solid group). Elastic strokes must look clumped rather than like flowing water. Consecutive samples join the previous group only while the group flags still match. Keyboard may mirror the labeled preset. Do not ship the freeglut key matrix as the only control.
- **D-08:** Canvas drag and the labeled preset are the interaction. A click with no move may leave a single stamp. Pointer samples outside the vessel still follow the destroy-then-create rule at the unprojected point; they do not need a separate outside-box rejection.

### Catalog claim, credits, and proof
- **D-09:** Append `drawing-particles`, then `sparky`, after the current scene ids. Keep the existing order, including `liquid-tumbler`. Hash routes stay `#/scene/{id}`. Each new entry is `ready: true` with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`.
- **D-10:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` for each new scene so the subject fills a phone canvas, includes the walls, and stays clear of the title and transport. Add each scene id to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-11:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, or filters. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.
- **D-12:** Keep the Phase 18 credit chrome. Implementation links stay host-locked to this repository's scene module. Inspiration cites the pinned JS and C++ tests at commit `7f20402173fd143a3988c921bc384459c6a858f2`: Drawing Particles cites `testDrawingParticles.js` and `DrawingParticles.h`; Sparky cites `testSparky.js` and `Sparky.h`.
- **D-13:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations. Scene docs use the same wording.
- **D-14:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. `MAX_ADVANCE_STEPS` stays 4. If a radius or VFX pool must shrink for playground cost, record that as an explicit playground adaptation. Raising a rigid-body frame cap is allowed only when Sparky's circles and walls cannot be captured otherwise, and that change must be tested.
- **D-15:** Prove locally with the existing Chromium `just web-player-smoke` suite. Both new scenes open, play, pause, and reset. The scenes already in the catalog still open and run. Interactive smoke covers one Drawing paint drag that leaves particles. Sparky smoke may stay watch-first; a native test must show a sparkable contact creates a powder group and that the group is gone after its lifetime. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-16:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber coordinates, circle spawn jitter, VFX radius, speed, and lifetime ranges, as long as collisions throw short-lived fading powder bursts and the chamber stays recognizable.
- Ring length at or below the upstream 50, as long as overwrite destroys the previous slot and particle count does not climb without bound across repeated contacts.
- Whether contact capture is a `CollisionDecisionHook` during the step or a drain of begin transitions after the step, as long as spawn and destroy happen after the world unlocks and persistent contacts do not keep sparking.
- Whether color fade uses an existing color mutator or a narrow batch write, as long as fade is visible in the copied frame and does not loop per particle across the JS boundary.
- Static SVG preview artwork and portrait rectangles, as long as previews are captioned `Static preview` and phone frames include the walls.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.
- Exact preset labels, as long as water versus elastic stays obvious.
- A third Drawing material such as powder, if it reuses existing flags and does not become the full keyboard matrix.

### Deferred Ideas (OUT OF SCOPE)
- Full Drawing Particles keyboard matrix, including barrier, repulsive, zombie, wall, spring, viscous, color mixing, and move mode (DRAW-02).
- Impulse and Liquid Timer particle-type presets (PRESET-01).
- Sealed per-scene differential evidence (PARITY-01) and Box2D-only tests (BOX2D-01).
- Optional Sparky pointer poke. Watch-first collisions are enough.
- Exact upstream `maxVFX = 50` when a smaller bounded ring still destroys expired bursts.
- Phases 30–33 fountain, wave tank, bubbler, and stacked drip.
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PLAY-01 | Visitor can open Drawing Particles, Elastic Particles, Impulse, Liquid Timer, Particles, Rigid Particles, Soup, Soup Stirrer, Sparky, Surface Tension, Theo Jansen, and Wave Machine from the catalog, and the existing six scenes remain available. | Ten of the twelve ids plus the original six and `liquid-tumbler` are already in `SCENE_IDS`. Append `drawing-particles`, then `sparky`. Wire both through `SceneId`, `parse_scene_id`, and `build_scene` so the catalog cannot advertise a session the factory cannot build. |
| FX-01 | Visitor can watch Sparky: colliding circles throw fading particle sparks. | Post-step `ContactTransitionKind::Begin` drain, powder `create_particle_group`, per-member `set_particle_velocity`, batch color write, ring destroy via `destroy_particle_group_particles` plus `compact_pending_particles`. |
| FX-02 | Visitor can paint Drawing Particles into an empty vessel, and at least one non-water material looks different from plain water. | Empty box at radius 0.05, pointer down/move uses `destroy_particles_in_shape` then a circle group. Water is `ParticleFlags::WATER` with empty group flags. Elastic is `ParticleFlags::ELASTIC` with `ParticleGroupFlags::SOLID`, joined only while those group flags still match. |
</phase_requirements>

## Project Constraints (from standards and repo guidance)

`.cursor/rules/` is absent. These directives come from the managed standards and repo-local guidance this phase must follow. [VERIFIED: standards pages and AGENTS.md Repo-Local Guidance]

- Physics stays in `liquidfun`. The WASM crate is the scene shell. SolidJS stays controls and rendering. [VERIFIED: `standards/core/architecture.md`, 29-CONTEXT.md]
- Safe Rust. No `unwrap()` on library paths. Optional internals use `maybe_`. Propagate typed errors. [VERIFIED: `standards/languages/rust.md`]
- Bright Builds file-length check fails at 629 physical lines. `web/src/catalog/scenes.ts` is already 620 lines. A scene module that approaches that limit splits the way Theo Jansen did (`theo_jansen.rs` plus `theo_jansen/tests.rs`). [VERIFIED: `standards/core/code-shape.md`; `wc -l`]
- New catalog ids must gain a portrait frame and a README SVG plan. `assertReadmeSvgPlanCoverage` fails when a `SceneId` is missing from `web/scripts/readme-svg/plans.ts`. [VERIFIED: AGENTS.md Repo-Local Guidance; `plans.ts`]
- Playground chrome stays the existing dark shell. Kobalte Dialog is the recorded override. Do not introduce MysticUI, a card grid, or a new theme. [VERIFIED: `standards/core/frontend-ui.md`; `standards-overrides.md`]
- Local proof before considering the phase done includes the existing Chromium `just web-player-smoke` suite. Nyquist validation is off in `.planning/config.json`, so this document does not add a separate validation-architecture section. [VERIFIED: `config.json` `workflow.nyquist_validation` is false]
- Copy stays an experimental recognizable port. Do not claim sealed C++ parity. [VERIFIED: D-13; REQUIREMENTS.md out of scope]
- Format only non-GSD Markdown with mdformat. This RESEARCH.md is GSD content and must not be mdformat-formatted. [VERIFIED: AGENTS.md Repo-Local Guidance]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace crate, Rust 1.97.0 | Worlds, contacts, particle groups, color lane | Production engine. No second physics crate. [VERIFIED: `cargo 1.97.0`; `crates/liquidfun`] |
| `liquidfun-wasm` | unpublished workspace crate | Scene factory, session step, copied frames | Existing playground shell. [VERIFIED: `crates/liquidfun-wasm/src/session.rs`] |
| SolidJS playground (`web/`) | existing player | Catalog, hash routes, pointer, captions | D-11 keeps `PlaygroundShell`. |
| Pinned LiquidFun tests | commit `7f20402173fd143a3988c921bc384459c6a858f2` | Sparky and Drawing layout | Submodule is at that commit. [VERIFIED: `git submodule status`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | existing `web/tests` | Catalog order, credits, portrait frames, README plan coverage | Every new `SceneId`. |
| Playwright Chromium | existing `just web-player-smoke` | Open, play, pause, reset, one Drawing drag | D-15. `bun` 1.4.2 is installed. [VERIFIED: `bun --version`] |
| `bitflags` particle/group flags | already in `liquidfun` | Water, elastic, powder, solid | Do not invent new flag bits. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Post-step `StepReport` drain | `CollisionDecisionHook::observe` during the step | The hook cannot spawn or destroy groups, and `ContactView` cannot be stored. A hook would only copy points into a side buffer that is consumed after unlock. The report already carries `Begin` transitions after unlock. Use the report. |
| `World::set` color batch | Per-particle WASM calls, or fading only by deleting the group | D-04 requires a visible fade in the copied frame, in batch, on the Rust side. |
| `ParticleGroupDestination::AppendTo` | Create a new group and then `join_particle_groups` | Upstream sets `pd.group = lastGroup`. Append is the same join. |

**Installation:** none. Do not add crates, features, or an FFI surface.

**Version verification:** Cargo 1.97.0 and bun 1.4.2 were probed in this session. Particle APIs below were read from the workspace, not from a registry.

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun/src/world/particle_object/   # narrow batch color write only
crates/liquidfun-wasm/src/session.rs          # keep StepReport; call on_after_step
crates/liquidfun-wasm/src/scene.rs            # SceneId arms for the two new ids
crates/liquidfun-wasm/src/scene/sparky.rs     # chamber, circles, ring; split tests if long
crates/liquidfun-wasm/src/scene/drawing_particles.rs
web/src/catalog/scenes.ts                     # must be split before the records land
web/src/catalog/previews.tsx                  # two captioned static SVGs
web/src/catalog/portrait-bounds.ts
web/scripts/readme-svg/plans.ts
web/scripts/demo-media/model.ts               # capture plan order follows SCENE_IDS
web/e2e/player.spec.ts                        # retarget interactive vs watch-first
web/e2e/player-helpers.ts                     # hash path + timeout
web/e2e/shell.spec.ts                         # nineteen links; .catalog-card stays 0
```

`web/src/catalog/scenes.ts` is 620 lines. Two full records, credit constants, and hints will cross 629. Extract the scene records (or the pinned credit constants) into a sibling module before appending. Do not add a file-length exception for this growth. [VERIFIED: `wc -l`]

`crates/liquidfun-wasm/src/frame.rs` is 486 lines. Do not grow it. Current caps already hold Sparky.

### Pattern 1: Post-step begin drain, then spawn

**What:** `SessionCore::advance` still calls `hooks.on_advance` before the step, still passes `&mut NoDecisionHook`, and still rejects counts outside `1..=4`. It keeps the `Ok(StepReport)` and passes `report.contact_transitions()` to a new defaulted `SceneHooks::on_after_step`. Only Sparky overrides that method.

**When to use:** Sparky bursts, fade ticks, and lifetime destroys. Existing motors stay in `on_advance` because they must run before the step.

**Example:**

```rust
// Source: crates/liquidfun/src/world/step/report.rs and session.rs
let report = self.world.step(
    self.step_configuration,
    &mut NoDecisionHook,
    self.step_limits,
)?;
self.hooks.on_after_step(
    &mut self.world,
    self.particle_system,
    report.contact_transitions(),
)?;
```

Sparky's override keeps `ContactTransitionKind::Begin` where `contact.bodies()` or `contact.fixtures()` hits a sparkable circle id stored on the hooks. Ignore `Persist` and `End`. Ignore begins whose fixtures are both walls. Spawn after this call, never inside `CollisionDecisionHook::command`. [VERIFIED: `WorldCommand` is only `DestroyBody` and `DestroyFixture`]

`advance_profiled` must use the same after-step call so diagnostic profiling does not skip bursts.

### Pattern 2: World point from the manifold, not from `ContactPointSnapshot`

**What:** `ContactPointSnapshot` has `feature_id`, `normal_impulse`, and `tangent_impulse`. It has no world position. [VERIFIED: `crates/liquidfun/src/world/contact.rs`]

**When to use:** Every spark placement.

**Example:**

```rust
// Source: liquidfun::collision::world_manifold, World::body_snapshot, fixture_snapshot
let manifold = transition.contact().maybe_manifold();
let transform_a = world.body_snapshot(bodies[0])?.transform();
let transform_b = world.body_snapshot(bodies[1])?.transform();
let radius_a = world.fixture_snapshot(fixtures[0])?.shape().radius();
let radius_b = world.fixture_snapshot(fixtures[1])?.shape().radius();
let maybe_world = manifold.and_then(|manifold| {
    world_manifold(manifold, transform_a, radius_a, transform_b, radius_b).ok().flatten()
});
let origin = maybe_world
    .and_then(|manifold| manifold.points().first().copied())
    .map(|point| point.point())
    .unwrap_or(circle_center);
```

Transforms are the post-step poses, a short distance from the in-step manifold. That is close enough for a recognizable burst. Do not store `ContactView`; its lifetime ends inside the hook. [VERIFIED: `contact_view.rs` compile-fail doctest]

### Pattern 3: Powder burst, outward velocity, ring destroy

**What:** One `ParticleGroupRecipe` per begin, capped by the ring.

Pinned upstream numbers from `Sparky.h` / `testSparky.js` at the submodule commit:

| Item | Upstream value | Phase use |
|------|----------------|-----------|
| Circles | 6 dynamic, radius 2, density 0.5 | Same count and radius |
| Circle centers | `x = 3 * RandomFloat()`, `y = 7 + 4.5 * i` | Fixed horizontal stagger so the native test is deterministic. Same vertical stack. [ASSUMED: a fixed stagger still produces circle-circle begins before the first wall pile] |
| Walls | floor `y=-10..0`, ceiling `y=40..50`, sides `x=±20..±40` from `y=-1..40` | Same polygons. Inner room is about `x=-20..20`, `y=0..40`. |
| Particle radius | 0.25 | Keep it. |
| VFX | circle radius `RandomFloat(1, 2)`, speed `10..20`, lifetime `0.5..1` s, powder, random RGB | Lifetime 0.5–1 s. Use a fixed palette and a fixed splash radius in that 1–2 m band unless cost forces a smaller radius, which must be written down as a playground adaptation. |
| Pool | `c_maxVFX = 50`, overwrite destroys the previous slot | Ring length 16. Overwrite calls destroy before the slot is reused. 16 is below 50 and is the explicit adaptation. |

Powder fill is small at this radius. A 1.5 m disk at radius 0.25 is on the order of a few dozen particles (`π (size / radius)² / 4`). Sixteen live bursts stay well under `MAX_FRAME_PARTICLES` (16_384). [VERIFIED: formula from disk area and diameter spacing; `MAX_FRAME_PARTICLES` in `session.rs`]

Spawn sequence, all after unlock:

1. If the ring slot is occupied, `destroy_particle_group_particles(group, false)` then `compact_pending_particles`. `destroy_particle_group` drops the shell and leaves the particles. Upstream `DestroyParticles` removes the particles.
2. `create_particle_group` with `ParticleGroupSource::filled_shapes` of one circle, `ParticleFlags::POWDER`, empty group flags, a non-zero `ParticleColor`, lifetime left at the infinite default (`0.0` on the recipe).
3. For each `particle_group_view(group).member_ids()`, `set_particle_velocity(id, (position - origin) * speed)`. `with_linear_velocity` would give the whole cloud one velocity. Outward motion has to be per member, inside Rust.
4. Each later `on_after_step`, age the slot by `1/60` s. Write the faded color in one batch call. When remaining lifetime hits zero, destroy and compact, then clear the slot.
5. Reset rebuilds `SessionCore`, which drops the hooks. That is the VFX reset. Do not also enable `ParticleSystemDef::with_destruction_by_age`.

Preferred spectacle is one group per `Begin` in that step. The ring still bounds the live count. A step with no new `Begin` emits nothing, including a resting pile in `Persist`.

Do not raise `MAX_RIGID_CIRCLES` (48) or `MAX_RIGID_SEGMENTS` (64). Six circles plus four wall polygons fit. [VERIFIED: `crates/liquidfun-wasm/src/frame.rs`] The older research note that those caps were 8 and 16 is stale.

### Pattern 4: Drawing empty vessel and joined strokes

**What:** Build the four wall polygons from `DrawingParticles.h` and create a particle system with radius `0.05` and zero groups. Floor `(-4,-2)-(4,0)`, sides `x=-4..-2` and `x=2..4` up to `y=6`, ceiling `(-4,4)-(4,6)`. Interior opening is about `x=-2..2`, `y=0..4`.

**When to use:** `SceneHooks::apply_pointer`. The session already routes pointer actions there.

**Example:**

```rust
// Source: World::destroy_particles_in_shape; ParticleGroupRecipe
let brush = Shape::from(CircleShape::new(Vec2::new(world_x, world_y), 0.2)?);
world.destroy_particles_in_shape(system, &brush, Transform::IDENTITY)?;
if particle_group_view(last).is_err() || view.member_count() == 0 {
    last = None;
}
let destination = match last {
    Some(group) if view.flags() == current_group_flags => {
        ParticleGroupDestination::AppendTo(group)
    }
    _ => ParticleGroupDestination::New,
};
```

Materials:

| Preset | Particle flags | Group flags | Look |
|--------|----------------|-------------|------|
| `water` (default) | `ParticleFlags::WATER` | empty | Flows. No solid clump. |
| `elastic` | `ParticleFlags::ELASTIC` | `ParticleGroupFlags::SOLID` | Same pairing as the elastic clump in `elastic_particles.rs`. Clumps instead of sloshing. |

Join only while group flags still match. Switching water to elastic starts a new group. Pointer up sets `lastGroup` to none. After `destroy_particles_in_shape`, `compact_pending_particles` removes empty groups that lack `CAN_BE_EMPTY`. The next sample must treat a stale id as cleared. [VERIFIED: `destroy_particles_in_shape` compacts; `retain_empty_after_member_removal` marks non-empty-capable groups `WILL_BE_DESTROYED`; `compact_pending_particles` prepares empty-group destruction]

Do not OR `REACTIVE`, barrier, zombie, wall, spring, viscous, or color-mixing. That is DRAW-02.

A third powder preset is allowed only if it is `ParticleFlags::POWDER` with empty group flags and a label that is obviously not water or elastic. It is not required for FX-02. Ship water and elastic first.

Unpaired pointer-up: the README sampler only sends `pointerAction("up", ...)`. [VERIFIED: `web/scripts/readme-svg.ts`] Stamp once on down, on move, and on an up that has no open stroke, then clear `lastGroup`. An up that ends a stroke only clears. That lets `plans.ts` use the existing `pointer-up` cue inside the vessel without a new cue kind, and it matches D-08's single stamp. Do not run `just readme-svg` as this phase's gate.

Stop creating when the system would pass its maximum count. Use the same 10240 cap as the other playground systems. A failed create is a no-op sample, not a radius shrink and not a higher step cap.

### Pattern 5: Catalog append around Liquid Tumbler

**What:** Current order ends `theo-jansen`, `liquid-tumbler`. New order ends `theo-jansen`, `liquid-tumbler`, `drawing-particles`, `sparky`. Nineteen ids. The twelve PLAY-01 names plus the original six are all present once those two land. Liquid Tumbler stays where it is.

Every place that lists ids in catalog order must grow together:

- `SCENE_IDS` and `SCENES` (after the file split)
- `parse_scene_id` / `SceneId` / `build_scene`
- `web/src/catalog/previews.tsx` switch, each preview still wrapped so the nav caption `Static preview` in `DemoNavigation.tsx` stays the only caption
- `PORTRAIT_VIEW_BOUNDS` and the wall-point table in `web/tests/portrait-bounds.test.ts`
- `README_SVG_PLANS`, in the same order
- `web/scripts/demo-media/model.ts` capture plans (`actualIds.at(-1)` is asserted as `liquid-tumbler` today)
- `web/e2e/player-helpers.ts` `SCENE_HASH_PATHS`
- `web/tests/scenes.test.ts`, `scene-catalog-credits.test.ts`, `scene-catalog-controls.test.ts`, `navigation.test.ts` (dynamic, but length strings that say seventeen are not)
- `crates/liquidfun-wasm/src/session/tests.rs` scene-id pairs
- `PAGE_SUMMARY` in `web/src/player/runtime.ts` ("All seventeen demos") and the matching strings in `web/e2e/shell.spec.ts`

Credits follow the existing pinned-blob shape. Host-locked implementation path `crates/liquidfun-wasm/src/scene/drawing_particles.rs` and `sparky.rs`. Inspiration hrefs:

- `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testDrawingParticles.js`
- `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/DrawingParticles.h`
- `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSparky.js`
- `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Sparky.h`

Keep the shared showcase link only if the neighboring scenes do. Do not claim the preview is a live simulation. Descriptions stay one behavior each, experimental wording, no "bit-exact" and no "matches the C++ test."

Drawing control is `runtimePreset("material", "...", [water, elastic])` with `recreates: false`, plus the existing gravity slider. Gravity recreates; the material preset must not. Sparky is gravity plus no scene-specific control. Interaction hint for Drawing names the drag and the material labels. Sparky uses the watch-first hint.

### Pattern 6: Phone frames and rigid caps

Portrait tests require the contain-fit width fraction above 0.85 on a 390×844 canvas, a per-scene minimum height fraction, and wall endpoints inside the portrait rectangle. [VERIFIED: `web/tests/portrait-bounds.test.ts`]

Starting rectangles, including wall thickness so the stroke is not clipped:

| Scene | Portrait rectangle | Why |
|-------|--------------------|-----|
| Drawing Particles | `minX=-4.2, minY=-2.2, maxX=4.2, maxY=6.2` | Outer faces of the four boxes. Square room, so set the minimum height fraction near 0.4 (a square on that phone is about 0.46 of the height). |
| Sparky | `minX=-22, minY=-1, maxX=22, maxY=42` | Inner chamber plus the inner face of the thick walls (`x=±20`, floor `y=0`, ceiling `y=40`). Do not frame the outer slabs out to `x=±40` and `y=-10..50`. Those extra meters shrink the circles and drop the height fraction. Wall test points are the inner corners. Landscape `viewBounds` uses the same chamber, not the shared 12 m × 9 m frame, or the walls clip. |

`MAX_ADVANCE_STEPS` stays 4. `web/src/physics/clock.ts` `MAX_STEPS_PER_FRAME` stays 4. Do not edit Dam Break's particle recipe.

### Pattern 7: Smoke classifier that ignores the gravity slider

**What:** `withGravitySlider` puts a Gravity range on every scene, including former watch-first scenes. `web/e2e/player.spec.ts` treats `controls.length > 0` as pointer-interactive and throws `missing pointer/control mapping` when `POINTER_CONTROL` has no entry. `POINTER_CONTROL` has no entry for Particles, Liquid Timer, Surface Tension, Elastic Particles, Rigid Particles, Soup, or Liquid Tumbler. [VERIFIED: `player.spec.ts`, `gravity-slider.ts`, `scenes.ts`]

**When to use:** This phase's Chromium proof will fail on the existing catalog until that split is retargeted. Do not "fix" it by adding fake drags for every gravity-only scene.

Classify this way:

- Pointer smoke: scenes in `POINTER_CONTROL`, plus `drawing-particles` with `gesture: "drag"` and the material control's visible label.
- Watch-first play/pause/reset: every other id, including `sparky` and the gravity-only scenes. Assert they have no pointer mapping. Do not assert `controls.length === 0`.
- One Drawing drag must leave `data-particle-count` greater than zero (or the session attribute the canvas already exposes for particle count). If no such attribute exists, assert the accepted pointer and a rising particle count through the existing data attribute the smoke already uses for other scenes. Check `data-pointer-accepted` the way `expectAcceptedPointerGesture` does.
- Raise `ALL_SCENE_TIMEOUT_MS` above 220_000. It was sized for eleven scenes. Nineteen open/reset loops need a higher ceiling. Measure once; do not raise the physics step cap.

Shell focus math in `shell.spec.ts` is "dismiss + seventeen scene links." After this phase it is dismiss + nineteen links. Update the wrap counts. Keep the `.catalog-card` count at 0.

### Anti-Patterns to Avoid

- **Mid-step `create_particle_group` from a hook:** The world is locked, and `WorldCommand` cannot spawn groups.
- **Replacing `NoDecisionHook` for every scene:** Other scenes must keep the no-op hook.
- **Sparking on `Persist`:** A resting pile would spray forever.
- **`with_destruction_by_age(true)`:** That deletes scene content that is supposed to stay. Water Wheel is the emitter exception and is not the pattern here.
- **`destroy_particle_group` for a finished burst:** That leaves the powder in the system.
- **Fading in SolidJS one particle at a time:** The copied frame must already contain the darkened colors.
- **In-place `color *= coeff` each tick:** `Sparky.h` multiplies the live buffer by an absolute coefficient, so later frames compound toward black faster than the piecewise function described just above that loop. Store the original color and write `channel = original * coeff` once per step, leaving alpha at 255. [VERIFIED: `Sparky.h` `ColorCoeff` and `c *= coeff`]
- **Classifying watch-first by `controls.length === 0`:** Gravity makes that set empty.
- **Raising `MAX_ADVANCE_STEPS` or shrinking radii to look smoother.**
- **README raster or a second browser as the phase gate.**

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Begin contact | A listener crate, user-data pointers, or FFI | `StepReport::contact_transitions` after `NoDecisionHook` | The report already has `Begin`, `Persist`, and `End`. |
| Group spawn during the step | A wider `WorldCommand` | `create_particle_group` after unlock | Commands are destroy-only and run before the caller can fill velocities. |
| Brush erase | A JS position list | `World::destroy_particles_in_shape` | It already queries the AABB, tests the circle, and compacts. |
| Stroke join | A second particle system | `ParticleGroupDestination::AppendTo` | Matches `pd.group = lastGroup` and keeps one flag group. |
| Elastic clump | A new material solver | `ELASTIC` plus group `SOLID` | Already solved for Elastic Particles. |
| Outward spark speed | One recipe linear velocity | `World::set_particle_velocity` per member, in Rust | Upstream sets each particle from its offset. |
| Color fade | A per-particle JS loop | One new `liquidfun` batch writer | No public color setter exists today. |
| Burst lifetime | System destruction-by-age or recipe `lifetime > 0` | Scene-owned seconds plus `destroy_particle_group_particles` | Age destruction is global and recipe lifetime is the infinite-default switch, not a VFX clock. |
| Spark point | A new contact-point field unless `world_manifold` cannot see the manifold | `collision::world_manifold` plus fixture radii | `ContactPointSnapshot` has no world point. |
| Catalog chrome | Cards, search, filters | Existing sidebar and Dialog | D-11. |

**Key insight:** The missing pieces are a post-step transition callback on the session, a batch color write, and two scene modules. The contact solver, group factory, brush destroy, and catalog shell are already there.

## Common Pitfalls

### Pitfall 1: Sparks never appear because the report is dropped

**What goes wrong:** Circles bounce and never throw powder.
**Why it happens:** `SessionCore::advance` maps `world.step` to `Result<(), _>` and always uses `NoDecisionHook`.
**How to avoid:** Thread `contact_transitions()` into Sparky only, after a successful step.
**Warning signs:** A Sparky test that only calls `on_advance` and never inspects a `StepReport`.

### Pitfall 2: Resting contacts keep emitting

**What goes wrong:** A pile on the floor sprays every frame.
**Why it happens:** The filter accepts `Persist`, or the scene remembers a contact flag and never clears it.
**How to avoid:** Accept `Begin` only. Clear the consideration when the step has no matching begin.
**Warning signs:** Particle count climbs while every circle's velocity is near zero.

### Pitfall 3: Fade is invisible or blacks out in one frame

**What goes wrong:** Bursts stay neon, or they vanish instantly.
**Why it happens:** Colors are only set at creation, or each frame multiplies the already-scaled byte by a shrinking coefficient.
**How to avoid:** Batch-write the original color scaled by `ColorCoeff` (full until half life, then linear to black), alpha held at 255. Assert the copied `particle_colors` lane changes and is not immediately zero.
**Warning signs:** `maybe_colors()` is `None` after the group is created. A non-zero group color is what allocates the lane. Do not create the burst with `ParticleColor::ZERO`.

### Pitfall 4: `lastGroup` joins a destroyed or empty group

**What goes wrong:** The next stamp errors, or new particles join a stale id.
**Why it happens:** The brush erased the group and the hook kept the id. Upstream clears it in `ParticleGroupDestroyed`.
**How to avoid:** After every destroy, drop `lastGroup` when `particle_group_view` fails or `member_count()` is 0. Pointer up also clears it. Reset rebuilds the session.
**Warning signs:** `HandleError::StaleOrDestroyed` on the second move of a drag that erased its own stroke.

### Pitfall 5: Elastic paint looks like water

**What goes wrong:** Both presets slosh.
**Why it happens:** Only the color changes, or `ELASTIC` is set without group `SOLID`.
**How to avoid:** Assert the elastic group's particle flags contain `ELASTIC` and its group flags contain `SOLID`. Water's group flags stay empty. Use a different color as well so the smoke and the preview can tell them apart.
**Warning signs:** Both presets use `ParticleFlags::WATER`.

### Pitfall 6: Catalog lists a scene the session rejects

**What goes wrong:** The hash opens and the session returns `UnknownScene`.
**Why it happens:** `SCENE_IDS` grew and `parse_scene_id` did not.
**How to avoid:** One Rust test table and one Vitest order table that list the same nineteen ids, ending `liquid-tumbler`, `drawing-particles`, `sparky`.

### Pitfall 7: Smoke treats Gravity as a pointer gesture

**What goes wrong:** `just web-player-smoke` throws `missing pointer/control mapping` before the new scenes are reached.
**Why it happens:** `controls.length > 0` is true for every scene that has the gravity slider.
**How to avoid:** Pointer map versus gravity-only watch loop, as in Pattern 7.
**Warning signs:** The failure names `particles` or `liquid-tumbler`, not the new ids.

### Pitfall 8: File-length failure on `scenes.ts`

**What goes wrong:** `bun scripts/bright-builds-check.ts file-lengths` fails after the records are added.
**Why it happens:** The catalog file is 620 lines and the limit is 629.
**How to avoid:** Split the catalog module in the same change that adds the two ids.
**Warning signs:** A patch that only appends records to `scenes.ts`.

### Pitfall 9: Portrait frame clips the chamber or shrinks it to a band

**What goes wrong:** Phone view clips walls, or Sparky's circles are a speck under the outer 80 m slabs.
**Why it happens:** Default 12×9 bounds, or a portrait rect that includes `x=±40`.
**How to avoid:** The rectangles in Pattern 6, plus wall endpoints in `portrait-bounds.test.ts`.
**Warning signs:** Width fraction at or below 0.85, or a wall point outside `PORTRAIT_VIEW_BOUNDS`.

### Pitfall 10: Claiming parity or raising the step cap

**What goes wrong:** Copy says the scene matches C++, or `MAX_ADVANCE_STEPS` becomes 5 to hide hitches.
**Why it happens:** Sparky's outer walls and Drawing's 0.05 radius are expensive, and the upstream file is sitting nearby.
**How to avoid:** Wording stays "experimental" and "recognizable." The only documented numeric adaptation in this phase is the VFX ring (16 instead of 50) and any splash radius that is actually reduced. Both must be written next to the constant.

## Code Examples

### Begin transitions and contact kinds

```rust
// Source: crates/liquidfun/src/world/contact.rs
pub enum ContactTransitionKind {
    Begin,
    Persist,
    End,
}

// Source: crates/liquidfun/src/world/step/report.rs
pub fn contact_transitions(&self) -> &[ContactTransition]
```

### Hook that cannot spawn

```rust
// Source: crates/liquidfun/src/world/step/report.rs
pub enum WorldCommand {
    DestroyBody(BodyId),
    DestroyFixture(FixtureId),
}
```

### Brush and burst primitives

```rust
// Source: crates/liquidfun/src/world/particle_object/particle.rs
pub fn destroy_particles_in_shape(
    &mut self,
    system: ParticleSystemId,
    shape: &Shape,
    transform: Transform,
) -> Result<usize, ParticleQueryError>

// Source: crates/liquidfun/src/world/particle_object/system.rs
pub fn set_particle_velocity(
    &mut self,
    particle: ParticleId,
    velocity: Vec2,
) -> Result<(), ParticleEditError>

// Source: crates/liquidfun/src/world/particle_object/group_lifecycle.rs
pub fn destroy_particle_group_particles(
    &mut self,
    group: ParticleGroupId,
    call_listener: bool,
) -> Result<(), CreateObjectError>

pub fn compact_pending_particles(
    &mut self,
    system: ParticleSystemId,
) -> Result<DestructionReport, HandleError>
```

### Flags already in the engine

```rust
// Source: crates/liquidfun/src/particle/definition.rs and group/flags.rs
ParticleFlags::WATER      // 0
ParticleFlags::ELASTIC    // 1 << 4
ParticleFlags::POWDER     // 1 << 6
ParticleGroupFlags::SOLID // 0x0001
```

### Frame copy that the fade must reach

```rust
// Source: crates/liquidfun-wasm/src/session.rs
let maybe_colors = view.maybe_colors();
let Some(colors) = maybe_colors else { /* frame capture fails closed */ };
particle_colors.extend_from_slice(&color.components());
```

The batch writer has to mutate the same color lane `ParticleSystemView::maybe_colors` reads. Add it beside `set_particle_velocity`: validate every id, then write. Suggested shape:

```rust
pub fn set_particle_colors(
    &mut self,
    particles: &[ParticleId],
    color: ParticleColor,
) -> Result<(), ParticleEditError>
```

One call per live burst per step. Do not export it across WASM per particle. [The method itself is not in the tree yet; the lane it must write is.]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Research note: rigid caps 16 / 8 | `MAX_RIGID_SEGMENTS = 64`, `MAX_RIGID_CIRCLES = 48` | Already in `frame.rs` | Sparky does not need a cap bump. |
| Research note: maybe add `DestroyParticlesInShape` | `World::destroy_particles_in_shape` exists and compacts | Phase 28 | Drawing reuses it. |
| Unconditional `NoDecisionHook` and discarded `StepReport` | Same code today | Still current | This phase adds a post-step hook. It does not replace the no-op decision hook. |
| `controls.length === 0` means watch-first | Gravity slider is on every scene | Current catalog | Smoke classification must change in this phase or the suite fails. |

**Deprecated/outdated:**

- `.planning/research/STACK.md` line that says the rigid caps are 16 and 8. The current constants supersede it.
- Treating Drawing's full keyboard matrix as required. DRAW-02 is deferred.
- Treating `maxVFX = 50` as required. A smaller ring that destroys on overwrite is explicitly allowed.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | A fixed horizontal stagger on the six circles still produces at least one sparkable `Begin` within a short native step run, so the spark test does not need `rand`. | Pattern 3 | The native test must place two circles in initial overlap, or give one an initial velocity, until a begin happens. Layout can stay recognizable either way. |
| A2 | Post-step body transforms are close enough to the in-step manifold that `world_manifold` places the burst on the contact. | Pattern 2 | If the burst appears a body-radius away, capture the two transforms inside a non-mutating `observe` (copy owned transforms, still spawn after unlock). Do not widen `WorldCommand`. |
| A3 | Ring length 16 with splash radius in the 1–2 m band stays inside the 10240 particle maximum at 60 Hz. | Pattern 3 | Lower the splash radius, record it next to the constant, and keep the ring destroy. Do not raise `MAX_ADVANCE_STEPS`. |
| A4 | `compact_pending_particles` makes a fully erased paint group stale before the next pointer sample returns. | Pattern 4 | If the id stays live with `member_count() == 0`, the clear condition still handles it. If append to a doomed group fails, clear and create a `New` group. |
| A5 | Unpaired `pointer-up` as a single stamp is acceptable for the README cue. | Pattern 4 | Coverage only requires the plan id. An empty-vessel cue is still valid. The Chromium drag is the paint gate, not the SVG. |

## Open Questions

1. **Does the current `just web-player-smoke` already fail on gravity-only scenes?**
   - What we know: The spec throws when an interactive id has no `POINTER_CONTROL` entry, and gravity makes every former watch-first scene interactive.
   - What's unclear: Whether CI has been red on this suite since the gravity slider landed. This session did not run Playwright.
   - Recommendation: Retarget the classifier in the same plan that extends the suite. Do not add pointer mappings for gravity-only scenes.

2. **Exact splash radius and ring length**
   - What we know: Upstream uses a random 1–2 m radius and 50 slots. Discretion allows a smaller ring and a documented smaller radius.
   - What's unclear: Frame cost on a full browser smoke of nineteen scenes.
   - Recommendation: Start at ring 16 and splash radius 1.5 m. Shrink the splash only after a smoke run shows the scene cannot advance, and write the new radius beside the constant.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Cargo / Rust | Native spark and paint tests | ✓ | cargo 1.97.0 | — |
| bun | Vitest and `just web-player-smoke` | ✓ | 1.4.2 | — |
| Node | bun's toolchain | ✓ | v24.13.0 | — |
| Python | mdformat of non-GSD docs only | ✓ | 3.14.6 | Do not mdformat this GSD file. |
| Pinned LiquidFun submodule | Layout reference | ✓ | `7f20402173fd143a3988c921bc384459c6a858f2` | — |
| Playwright Chromium | D-15 smoke | not probed this session | installed by the existing web suite | Smoke is the phase gate; the plan runs `just web-player-smoke` rather than a substitute browser. |

**Missing dependencies with no fallback:**

- None identified for planning. Chromium availability is whatever the existing player-smoke script already requires.

**Missing dependencies with fallback:**

- None.

## Security Domain

This phase is a local playground scene. It does not add accounts, sessions, tokens, or cryptography. `security_enforcement` is not set to false in `.planning/config.json`, so the relevant slice of ASVS still applies to new inputs.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No login surface. |
| V3 Session Management | no | The WASM session is a physics owner, not an HTTP session. |
| V4 Access Control | no | All catalog scenes are public by design. |
| V5 Input Validation | yes | Scene ids stay on the allowlist. Pointer coordinates stay finite (`SessionError::InvalidPointer`). The material preset accepts only `water` and `elastic` (and powder only if that third label is added). Gravity stays on the existing checked slider. Unknown controls return `UnknownControl`. |
| V6 Cryptography | no | No new crypto. |

### Known Threat Patterns for this playground

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unexpected scene id or preset string | Tampering | `parse_scene_id` and the preset match stay exhaustive. Default branch is an error, not a fallback scene. |
| Non-finite pointer or gravity | Tampering | Existing finite checks. Do not skip them for the new scenes. |
| Painting outside the box | Tampering | D-08 allows destroy-then-create at the unprojected point. It is bounded by the particle maximum, not by a secret. |
| Mid-step mutation from a contact hook | Tampering | Spawn only after unlock. Do not add a command that creates groups while locked. |

## Sources

### Primary (HIGH confidence)

- `third_party/liquidfun` at `7f20402173fd143a3988c921bc384459c6a858f2`: `Testbed/Tests/Sparky.h`, `lfjs/testbed/tests/testSparky.js`, `Testbed/Tests/DrawingParticles.h`, `lfjs/testbed/tests/testDrawingParticles.js`
- `crates/liquidfun/src/world/step/report.rs` — `StepReport::contact_transitions`, `WorldCommand`
- `crates/liquidfun/src/world/contact.rs` — `ContactTransitionKind`, `ContactPointSnapshot` fields
- `crates/liquidfun/src/collision/narrow.rs` and `collision.rs` — public `world_manifold`
- `crates/liquidfun/src/world/particle_object/particle.rs` — `destroy_particles_in_shape`
- `crates/liquidfun/src/world/particle_object/system.rs` — `set_particle_velocity`; no color setter
- `crates/liquidfun/src/world/particle_object/group.rs` — `create_particle_group`, `AppendTo`
- `crates/liquidfun/src/world/particle_object/group_lifecycle.rs` — `destroy_particle_group_particles`, `compact_pending_particles`
- `crates/liquidfun-wasm/src/session.rs` — `MAX_ADVANCE_STEPS = 4`, discarded step report, color copy
- `crates/liquidfun-wasm/src/frame.rs` — rigid caps 64 / 48
- `crates/liquidfun-wasm/src/scene.rs` — allowlist ending at `LiquidTumbler`, `SceneHooks`
- `web/src/catalog/scenes.ts` — order ending `liquid-tumbler`, gravity on watch-first scenes
- `web/e2e/player.spec.ts` — `controls.length` split versus `POINTER_CONTROL`
- `web/src/physics/clock.ts` — `MAX_STEPS_PER_FRAME = 4`
- `standards/core/code-shape.md` — 629-line file-length check

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` and `FEATURES.md` — recognition cards. Used for scope, then checked against the pinned tests and current code. The rigid-cap numbers in `STACK.md` were not reused.

### Tertiary (LOW confidence)

- None that the plan should treat as decided. Discretion items are listed in the assumptions log.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — the phase uses crates and scripts already in the repo.
- Architecture: HIGH — step report, brush destroy, and catalog seams were read. The batch color method is specified because the tree has no public writer.
- Pitfalls: HIGH for the dropped step report, gravity smoke split, and `scenes.ts` length. MEDIUM for the exact spark placement using post-step transforms.

**Research date:** 2026-09-26
**Valid until:** 2026-10-26 (stable engine seams; re-check if `session.rs` or `scenes.ts` moves first)

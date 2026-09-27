# Phase 32: Liquid motion bubbler - Research

**Researched:** 2026-09-27
**Domain:** Original playground scene. Colored water drips through one static waist, turns a motor-off paddle wheel, and a slow side-shaft plate lifts the liquid back to the reservoir.
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Vessel
- **D-01:** Add a new scene. Do not replace, hide, or retune `water-wheel`. The catalog id is `liquid-bubbler`, appended after `wave-tank`. The title is Liquid Bubbler.
- **D-02:** The layout is an upper reservoir, one narrow static waist, and a lower chamber. Liquid leaves the reservoir by dripping through that waist. The wheel sits in the lower chamber, under the drip. Do not use Water Wheel's open basin and aimed jet, Liquid Timer's shelves, or a stack of reacting parts.

#### Wheel and flow
- **D-03:** The wheel is a dynamic paddle wheel on a revolute joint with the motor off. Particles that fall through the waist hit the paddles and turn the wheel. Do not spin the wheel with a motor. Do not aim a particle jet at it.
- **D-04:** The drip continues for the whole play session. A quiet return lifts liquid back to the upper reservoir. The visible spectacle stays the waist and the wheel. The return is not a second piston show and does not add a stack of parts that each react when the drip reaches them. Reset restores the initial reservoir, the resting wheel, and the initial particle layout.

#### Liquid color
- **D-05:** Use one plain water particle group with one distinct `ParticleColor`, so the drip reads as colored liquid. Do not add tensile, elastic, rigid, or color-mixing groups. Do not retune Color Mixer.
- **D-06:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

#### Catalog, credits, and proof
- **D-07:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/liquid-bubbler`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-08:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the reservoir, the waist, the wheel, and the chamber walls fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `liquid-bubbler` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-09:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-10:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, after a bounded run, particles that began above the waist are below it, and the wheel angle has changed from its initial pose. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber sizes, waist gap, wheel radius, paddle count, particle radius, particle count, and the single particle color, as long as a drip through the waist visibly turns the wheel and the walls stay inside the portrait frame.
- How the return lift is built, as long as particles physically cross the waist before they turn the wheel, the return does not tunnel through the wheel, and the return does not become the spectacle.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a narrow waist with a wheel beneath a colored drip.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts, and each part reacts when the drip reaches it.
- Visitor waist, color, or wheel controls.
- Replacing or retuning Water Wheel, Color Mixer, Liquid Timer, or Hydraulic Fountain.
- Two-color mixing through the waist.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| — | No phase requirement IDs are assigned. Locked decisions D-01 through D-11 are the scope. | Do not invent a `PLAY-03` credit or a new upstream-test requirement. `PLAY-01`, `PLAY-02`, and `PLAY-03` are already complete. Water Wheel stays the jet-driven wheel. [VERIFIED: `.planning/REQUIREMENTS.md`] |

This phase is an original playground scene. Its gate is D-10, not a requirements-matrix row.
</phase_requirements>

## Summary

The drip and the wheel already have engine support. A static ground body can form an upper reservoir, one waist gap, and a lower chamber. A dynamic paddle body pinned with `RevoluteJointDef::new` leaves the motor off. Particle pressure contacts apply an off-center impulse, and that impulse changes the body's angular velocity, so a falling stream can turn the wheel without `set_revolute_motor_speed`. [VERIFIED: `RevoluteJointDef::new`; `candidate_apply_linear_impulse`]

The return should be one short dynamic plate in a side shaft, on a vertical prismatic joint, written from `on_advance` the way Hydraulic Fountain writes its piston. Keep it quieter than that scene: the shaft sits beside the wheel, the plate does not move during the proof window, and the lift speed stays far below one particle diameter per step. Do not recycle by deleting particles or by writing their positions. Those paths either skip the waist or lose the identities D-10 counts. No new engine API is required. Do not edit `water_wheel.rs`, `color_mixer.rs`, `liquid_timer.rs`, `hydraulic_fountain.rs`, or `MAX_ADVANCE_STEPS`.

**Primary recommendation:** Add `liquid-bubbler` as a watch-first scene: one amber water group in an upper reservoir, a static waist, a motor-off four-paddle wheel in the lower chamber, and a side-shaft plate that stays down for 3 seconds and then creeps upward at about 0.15 m/s. Prove it with a `SessionCore` test that, after 2 seconds of `advance(4)`, original particle ids that started above the waist are below it and `revolute_joint_angle` has left 0, while the plate translation is still about 0.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory is present, and this repo has no `.cursor/skills/` or `.agents/skills/` indexes. [VERIFIED: globs of those paths]. These repo rules still bind the planner:

- Physics stays in `liquidfun`. The WASM crate is the scene shell. SolidJS stays controls and rendering. [VERIFIED: `32-CONTEXT.md`; `standards/core/architecture.md`]
- A new catalog id also needs a portrait frame and a README SVG plan. Do not mdformat `.planning/**`. [VERIFIED: `AGENTS.md` README scene gallery and playground canvas framing]
- Safe Rust, no `unwrap()` on the production path, `foo.rs` plus `foo/` if the scene module is split. File-length automation fails at 629 physical lines. Tests use arrange, act, assert, and one concern each. [VERIFIED: `standards/languages/rust.md` via the phase-31 pattern already used in this crate; `standards/core/code-shape.md` 629-line gate; `standards/core/testing.md` expectation from prior scene plans]
- The shared gravity slider may stay, because every catalog scene includes it. D-04 forbids visitor waist, color, and wheel controls, not that slider. [VERIFIED: `scene-records.ts` `withGravitySlider([])` on `wave-tank`]
- Dark playground chrome already exists. Do not restyle the shell. [VERIFIED: D-07; `DemoNavigation.tsx` still renders `Static preview`]
- Hobby scope: local checks only. No Linux qualification, no package publication, no sealed parity claim. [VERIFIED: `PROJECT-SCOPE.md`]
- The implementing agent does not self-approve. Passing `just web-player-smoke` is not a review acknowledgment. [VERIFIED: `AGENTS.md` Independent review, D-11]
- Before commit, run the repo-native checks for the files this phase touches. `just web-player-smoke` is the browser proof. [VERIFIED: `justfile` `web-player-smoke`]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace crate, Rust 1.97.0 | Static waist, motor-off revolute wheel, vertical prismatic lift, one water group | The joints and the particle-body impulse already exist. [VERIFIED: `cargo 1.97.0`; `revolute.rs`; `body_coupling.rs`] |
| `liquidfun-wasm` | workspace crate | Scene build, `on_advance` lift schedule, `SessionCore` | Hydraulic Fountain already writes a prismatic motor from simulation time. [VERIFIED: `hydraulic_fountain.rs`] |
| SolidJS catalog | existing `web/` | Record, preview, portrait frame, hash route | Append one id after `wave-tank`. Do not add a UI library. [VERIFIED: `web/src/catalog/scenes.ts`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | existing `web` tests | Catalog, credit, portrait, README-plan coverage | Every exhaustive `SceneId` map |
| Playwright Chromium | via `just web-player-smoke` | Open, play, pause, reset | D-10 browser gate only |
| `bun` | 1.4.2 installed | Runs `scripts/web-build.ts player-smoke` | The smoke alias. [VERIFIED: `justfile`, `bun --version`] |
| `std::f32::consts::TAU` | Rust 1.97.0 | Only if a schedule needs a full turn | Do not import Wave Machine's drive. The lift is a dwell plus a signed creep, not a sine. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Side-shaft plate, motor on, speed written in `on_advance` | Hydraulic Fountain's 0.6 m/s throat piston, copied into the center | Rejected. D-04 says the return is not a second piston show. Use the same setter, a vertical axis, a side shaft, a slower speed, and a dwell so the waist and wheel stay the spectacle. |
| Motor-off `RevoluteJointDef::new` | `with_motor(true, ...)` or `set_revolute_motor_speed` | Rejected by D-03. The default constructor already leaves `enable_motor` false. [VERIFIED: `RevoluteJointDef::new`] |
| One `ParticleGroupRecipe` with `with_color` | `create_particle_with_def`, a finite lifetime, or `ParticleFlags::COLOR_MIXING` | Rejected. Spawned particles are the Water Wheel jet. Color mixing is Color Mixer's spectacle. The mixing pass runs only when a particle carries `COLOR_MIXING`. [VERIFIED: `material.rs` mixing gate] |
| `set_particle_position` or `set_particle_velocity` as the return | A physical plate | Rejected. Position writes move liquid without a crossing. A velocity region that overlaps the wheel can shove it or skip the waist. |
| Particle damping `0.2` | `ParticleSystemDef` default damping `1.0` | Use `0.2`. Default damping is the strength applied to approaching contacts. A pinch stream needs to keep falling. Copy the number only. Do not edit Wave Machine. [VERIFIED: `ParticleSystemDef::default` damping `1.0`; `pressure.rs` `damping`] |

**Installation:** none. Do not add crates or npm packages.

**Version verification:** `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustc 1.97.0 (2d8144b78 2026-07-07)`, `just 1.48.0`, `bun 1.4.2`, `node v24.13.0`. [VERIFIED: local probes on 2026-09-27]

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun-wasm/src/scene/liquid_bubbler.rs       # build, walls, wheel, plate, group, on_advance
crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs # waist crossing and wheel angle
crates/liquidfun-wasm/src/scene.rs                      # SceneId, parse, build_scene
web/src/catalog/scenes.ts                               # append id after wave-tank
web/src/catalog/scene-records.ts                        # watch-first record
web/src/catalog/previews.tsx                            # static waist, drip, wheel
web/src/catalog/portrait-bounds.ts                      # phone frame
web/scripts/readme-svg/plans.ts                         # coverage entry, cues: []
web/scripts/demo-media/model.ts                         # center-click stub, catalog order
```

Start as `liquid_bubbler.rs` plus `liquid_bubbler/tests.rs`. The credit path must stay `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`. `sceneBlobUrl` rejects a path with a slash under `scene/`. [VERIFIED: `web/src/catalog/links.ts` `isAllowlistedScenePath`]

Do not edit `water_wheel.rs`, `color_mixer.rs`, `liquid_timer.rs`, `hydraulic_fountain.rs`, `wave_tank.rs`, or `MAX_ADVANCE_STEPS`.

### Pattern 1: Static reservoir, one waist, lower chamber

**What:** One static ground body holds the outer walls, the reservoir floor, and two lips that leave a single gap. The wheel sits under that gap. A vertical wall separates a side shaft from the wheel so the return cannot pass through the paddles.

**When to use:** The whole vessel. This is D-02.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs attach_box
// All of these boxes go on the static ground body. The gap between the two lips is the waist.
attach_box(world, ground, lip_half_width, lip_half_height, left_lip_center, 0.0)?;
attach_box(world, ground, lip_half_width, lip_half_height, right_lip_center, 0.0)?;
```

Place the fill polygon strictly inside the reservoir, above the lips, and clear of every solid. A polygon that overlaps a lip spawns particles inside geometry. The lower chamber starts empty so the crossing is visible. [VERIFIED: Hydraulic Fountain fills a polygon that sits in the chamber and not inside the piston]

The waist gap must be wider than one particle diameter or the stream arches and stops. At radius `0.025` the diameter is `0.05` m. Start the gap near `0.14` m, about three diameters. Widen the gap if the crossing test shows an empty lower chamber. Do not shrink the radius to unjam it. [VERIFIED: diameter is `2 * radius`; gap width is discretion and is logged as an assumption]

### Pattern 2: Motor-off revolute paddle wheel

**What:** A dynamic body with a hub circle and four short polygon paddles, pinned to the ground at the hub. Do not call `with_motor` or `set_revolute_motor_speed`. The default revolute definition has `enable_motor: false`, speed `0`, and torque `0`.

**When to use:** D-03. Reuse Water Wheel's joint and paddle-fixture shape, not its basin, jet, lifetime, or controls.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/water_wheel.rs pin_wheel
// and crates/liquidfun/src/joint/definition/revolute_prismatic.rs RevoluteJointDef::new
let joint = RevoluteJointDef::new(ground, wheel)?
    .with_frame(hub_position, Vec2::ZERO, 0.0)?;
world.create_joint(JointDef::from(joint))?;
```

`BodyDef::new(..., active)` starts the body awake. Chain `.with_sleeping_allowed(false)` and a small angular damping (`0.05`, the Water Wheel number copied locally) so the wheel is still awake when the first drops arrive. Water Wheel's density `0.45` is sized for an 8 m/s jet. Start this wheel lighter, about `0.20`, so a drip can change the angle. [VERIFIED: `BodyDef::new` fourth argument is `active`; `water_wheel.rs` density `0.45`, jet speed `8.0`, angular damping `0.05`]

Leave `collide_connected` at its default `false`. The paddles must not sweep through ground fixtures, because a false flag means they will not collide with the walls that live on the ground body. Keep the paddle circle inside the chamber with clearance. Particles still hit the paddles: particle contacts are not the joint's collide-connected flag. [VERIFIED: `RevoluteJointDef::new` sets `collide_connected: false`]

The pressure solver calls `apply_linear_impulse` at the contact point. For a dynamic body that applies `inverse_inertia * (offset_x * impulse.y - offset_y * impulse.x)` to angular velocity. A paddle hit off the hub changes `revolute_joint_angle`. A static or kinematic wheel would ignore that impulse. [VERIFIED: `pressure.rs` body impulse; `body/control.rs` `candidate_apply_linear_impulse`]

Read the pose with `World::revolute_joint_angle`. The initial angle is `0` when the frame reference angle is `0` and the body is built at angle `0`. [VERIFIED: `revolute_joint_angle`]

### Pattern 3: Quiet side-shaft lift

**What:** One dynamic plate on a vertical prismatic joint, in a shaft to the side of the wheel. The inlet is a floor gap past the wheel's outermost paddle, so liquid hits the paddles before it can enter the shaft. The plate spans the shaft. `with_collide_connected(true)` lets it seal against the ground-owned shaft walls. A lip above the reservoir's resting surface spills the lifted liquid back in, so the reservoir cannot drain down the shaft.

**When to use:** D-04. This is the return. It is not the spectacle.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs create_piston_joint
// Axis (0, 1), a side shaft, and a speed far below that scene's 0.6 m/s.
let definition = PrismaticJointDef::new(ground, plate)?
    .with_collide_connected(true)
    .with_frame(plate_bottom, Vec2::ZERO, Vec2::new(0.0, 1.0), 0.0)?
    .with_limits(true, -LIMIT_MARGIN, stroke + LIMIT_MARGIN)?
    .with_motor(true, 0.0, MAX_MOTOR_FORCE)?;
```

Build the plate at translation `0`. Start the motor at speed `0`. In `on_advance`, add `1/60` and then write speed:

- While `elapsed < DWELL` (start at `3.0` s), speed stays `0`. The plate remains down, the inlet stays open, and the proof window can finish.
- Then a slow positive speed lifts.
- Then the same speed with the opposite sign lowers the empty plate.

Session `advance` calls `on_advance`, then escape eviction, then `World::step`. Writing the motor before the step is the same order Hydraulic Fountain uses. `MAX_ADVANCE_STEPS` is `4`, so integrate `1/60` once per `on_advance`, not once per browser frame. [VERIFIED: `session.rs` `advance`; `hydraulic_fountain.rs` `on_advance`]

Hydraulic Fountain's spectacle is a horizontal plate at `0.6` m/s, stroke `0.35` m, period `2` s, moving from the first step through a throat. Do not copy those numbers, that axis, or that file. A useful quiet start is about `0.15` m/s. At radius `0.025`, one diameter per step is `0.05 * 60 = 3` m/s, so `0.15` m/s is about `0.0025` m per step. [VERIFIED: `hydraulic_fountain.rs` constants; diameter from radius]

`solve_motor` applies no impulse when the limit state is `Equal`. `Equal` is the state for an enabled range shorter than `2 * LINEAR_SLOP` (`0.01` m). The stroke has to be a real range, and the limits should sit about `0.02` m outside it so the plate is not clamped on the first step of the rise. Max motor force `1.0e6` keeps the slow plate moving against the water. Lowering the force is not what makes the return quiet. The low speed and the side placement do that. [VERIFIED: `LINEAR_SLOP` `0.005`; Hydraulic Fountain `MAX_MOTOR_FORCE`]

Do not use `set_particle_position` or `set_particle_velocity` to recycle liquid. Do not call `create_particle_with_def` or `destroy_particles_in_shape` on the return path. Those skip the waist or replace the ids the crossing test follows. [VERIFIED: `World::set_particle_position`; `World::set_particle_velocity`]

Reset already disposes the WASM session and rebuilds the world. Elapsed time on the hooks returns to zero, the plate returns to the bottom, and the group is sampled in the reservoir again. Do not add a SolidJS integrator. [VERIFIED: Wave Tank and Hydraulic Fountain rebuild through `beginScene`]

### Pattern 4: One amber water group

**What:** `ParticleGroupRecipe::new` defaults to `ParticleFlags::WATER`, zero velocity, and lifetime `0` (infinite). Call `with_color` once. Do not call `with_particle_flags`.

**When to use:** D-05.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs create_water_group
// Replace that scene's blue with one amber that this scene owns.
const DRIP_COLOR: ParticleColor = ParticleColor::new(242, 176, 64, 255);
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_color(DRIP_COLOR)
    .with_transform(Transform::IDENTITY)?;
```

Amber `(242, 176, 64, 255)` is distinct from the basin blue `(77, 163, 255, 255)` and from Color Mixer's red `(248, 113, 113, 255)`. The preview SVG should use the same amber. Do not edit `color_mixer.rs`. [VERIFIED: those constants in `hydraulic_fountain.rs`, `water_wheel.rs`, and `color_mixer.rs`]

Color mixing runs only when a particle's flags contain `COLOR_MIXING`. The default recipe does not set that flag. Do not call `with_color_mixing_strength`. The system definition's default strength `0.5` is unused without the flag. [VERIFIED: `material.rs`; `ParticleSystemDef::default`]

Set radius `0.025` and damping `0.2` on `ParticleSystemDef`. The definition default radius is `1.0` and the default damping is `1.0`. Also call `with_destruction_by_age(false)`. The default is `true`, and age destruction evicts oldest particles once a maximum count is set. This scene has no maximum and no finite lifetime, so the default would not delete the group today, but an explicit `false` keeps a later cap from eating the recirculating liquid. Do not copy Water Wheel's `with_destruction_by_age(true)` plus a 3 second lifetime. [VERIFIED: `ParticleSystemDef::default`; `lifetime.rs` `prepare_capacity_for_creation`; `water_wheel.rs` lifetime `3.0`]

Sample stride is `PARTICLE_STRIDE * diameter` with `PARTICLE_STRIDE = 0.75`. At radius `0.025` the stride is `0.0375` m. Fill only the reservoir polygon and keep the resulting count. Do not delete particles to look smoother. Escape deletion starts at 12 m down-gravity or 48 m sideways. A vessel a few meters tall stays inside that keep. [VERIFIED: `world/particle_object/group.rs`; `settings.rs`; `session/escape.rs`]

Gravity `(0, -10)`. [VERIFIED: `hydraulic_fountain.rs` `GRAVITY`]

### Pattern 5: Catalog append, watch-first

**What:** Append `"liquid-bubbler"` after `"wave-tank"` in `SCENE_IDS`. Add the same id everywhere `SceneId` is exhaustive. `web/tests/demo-media-model.test.ts` currently expects the last id to be `wave-tank`. That assertion has to move to `liquid-bubbler`. [VERIFIED: `scenes.ts`; `demo-media-model.test.ts`]

**Watch-first record:** `ready: true`, `interactionHint` equal to `WATCH_FIRST_HINT` (`"This scene is watch-first. Use Play scene, Pause scene, and Reset scene."`), `controls: withGravitySlider([])`. Description is one behavior and says this is an original experimental scene: colored liquid drips through a narrow waist and turns a small wheel. Inspiration is the Phase 18 showcase link only, the same shape as Wave Tank: label `LiquidFun showcase`, href `https://google.github.io/liquidfun/`. Implementation path is `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`. [VERIFIED: `scene-records.ts` `SHOWCASE`, `WATCH_FIRST_HINT`, and the `wave-tank` record]

The caption `Static preview` is already rendered by `DemoNavigation` for every preview. The SVG still has to draw a narrow waist with a wheel beneath a colored drip. [VERIFIED: `web/src/components/DemoNavigation.tsx`]

README plan entry is `{ id: "liquid-bubbler", cues: [] }` appended after `wave-tank`. Watch-first scenes leave cues empty. Do not add a cue. [VERIFIED: `web/scripts/readme-svg/plans.ts`, last entry is `wave-tank` with `cues: []`]

Demo capture still wants a plan for every `SCENE_IDS` entry. Append a center-click stub `{ kind: "click", point: { x: 0.5, y: 0.5 } }` with `interactionStep: 180`, matching the other watch-first originals. Pointer up on this scene is a no-op. Do not run `just readme-svg` as the phase gate. [VERIFIED: `web/scripts/demo-media/model.ts` Wave Tank entry; D-08]

`viewBounds` on the scene record and `PORTRAIT_VIEW_BOUNDS["liquid-bubbler"]` should both be the tight vessel rectangle, including the shaft. Landscape reads `worldBoundsForScene`. The portrait test requires fitted width above `0.85` of a 390 by 844 canvas and height above `MIN_HEIGHT_FRACTION`. A rectangle about as tall as it is wide, or a bit taller, fills that phone. A very wide frame becomes a short band. Include wall endpoints for the reservoir, the waist lips, the chamber, the shaft, the wheel hub, and the plate at rest and at the top of the stroke, so both plate poses stay inside the phone frame. `VIEWPORT_INSET` is 16 CSS pixels. Set `MIN_HEIGHT_FRACTION["liquid-bubbler"]` to a value the finished rectangle clears. Start the expectation near `0.40` and lower only that constant if the honest rectangle cannot reach it. Do not copy Wave Tank's `0.20` unless this frame is actually that short. [VERIFIED: `portrait-bounds.test.ts`; `portrait-bounds.ts` last entry is `wave-tank`]

### Anti-Patterns to Avoid

- **A motor on the wheel:** `with_motor(true, ...)` or `set_revolute_motor_speed` spins the wheel while the liquid only falls nearby. D-03 forbids that.
- **Water Wheel's jet:** `create_particle_with_def`, a finite lifetime, and `emit_jet` are that scene. Copying them aims a particle stream and retires the ids the crossing test needs.
- **Hydraulic Fountain as the spectacle:** `0.6` m/s, a 2 second period, a horizontal throat, and motion from step 1 make the plate the show. Leave that file untouched.
- **Teleport return:** `set_particle_position` moves liquid through the wheel or back above the waist without a crossing. `set_particle_velocity` in a region that overlaps the paddles does the same job less honestly.
- **Kinematic plate:** A kinematic body has no inverse mass, so the prismatic motor does not translate it, and particle impulses do not move it. The plate is dynamic. [VERIFIED: `candidate_apply_linear_impulse` returns early unless the body is dynamic]
- **Proof window after the lift has started:** Particles that already rode the plate back into the reservoir fail "are below the waist." Keep the dwell longer than the native run.
- **Destroy-by-age with a maximum count:** That evicts oldest particles and changes who is below the waist. This scene does not set a maximum and turns age destruction off.
- **PLAY-03 citation:** No pinned test href and no "matches testWaterWheel" copy.
- **Phase 33 stack:** No shelves, gates, or extra joints that each react when the drip arrives.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Turning the wheel | A revolute motor, a scripted angle, or a new torque API | Motor-off revolute joint plus particle-body impulses on the paddles | Off-center impulses already update angular velocity |
| Keeping the drip going | Destroy in the sump and `create_particle_with_def` at the top | One plate in a side shaft, `set_prismatic_motor_speed` | Deletes change particle identity and skip the physical cross |
| The return schedule | A SolidJS timer | `elapsed += 1/60` inside `on_advance` | The session may step 1 to 4 times per frame |
| A one-way shaft | A region that sets particle velocity upward | Shaft walls, a sealing plate (`collide_connected: true`), and a lip above the reservoir surface | The plate cannot overlap the wheel, and the reservoir cannot drain down the shaft |
| The colored drip | A second group or `COLOR_MIXING` | `with_color` on one water recipe | Mixing is gated on the particle flag |
| Reset | Custom rewind of the wheel and particles | Existing session dispose and rebuild | `beginScene` constructs from the scene builder |
| Phone framing | The shared 12 m by 9 m frame | `PORTRAIT_VIEW_BOUNDS["liquid-bubbler"]` | A wide frame becomes a short band on a 390 by 844 canvas |

**Key insight:** The missing piece is a scene, not a solver feature. A static waist, a motor-off revolute wheel, a slow side-shaft plate, and stable particle ids are enough to drip, turn, and keep going.

## Common Pitfalls

### Pitfall 1: The wheel is driven, or it never feels the drip

**What goes wrong:** The wheel spins with no liquid on the paddles, or it stays at angle `0` after the liquid has crossed.

**Why it happens:** A motor was enabled, the body sleeps, the density is Water Wheel's jet density, the paddles miss the drip, or the paddles pass through the walls and the liquid misses them.

**How to avoid:** Do not call `with_motor` on the revolute definition. Disable sleeping. Start density near `0.20`. Put the hub under the waist and keep the paddle sweep inside the chamber. A source test can require `set_prismatic_motor_speed` and reject `set_revolute_motor_speed`.

**Warning signs:** `revolute_joint_angle` changes on a run with an empty lower chamber, or it stays `0` while particles are sitting on the floor under the waist.

### Pitfall 2: The return undoes the crossing, or it hits the wheel

**What goes wrong:** After the bounded run, the original ids are back above the waist, or the plate is inside the paddle circle.

**Why it happens:** The lift starts at step 1, the shaft shares space with the wheel, or the proof window is longer than the dwell.

**How to avoid:** Dwell `3` s at speed `0`. Proof window `2` s. Put a static wall between the shaft and the wheel. The inlet is past the outermost paddle. Assert plate translation is still near `0` in the crossing test.

**Warning signs:** Plate translation is already near the stroke when the angle assertion runs, or particles inside the wheel polygon started in the shaft.

### Pitfall 3: The waist jams or the reservoir drains down the shaft

**What goes wrong:** Nothing crosses, or liquid never uses the waist because the shaft is an open second drain.

**Why it happens:** The gap is about one diameter, the fill overlaps a lip, or the shaft mouth is below the reservoir surface.

**How to avoid:** Start the gap near `0.14` m. Keep the fill inside the reservoir. Spill only over a lip above the resting free surface. If the lower chamber stays empty, widen the gap. Do not lower the radius and do not raise the 4-step cap.

**Warning signs:** Step 0 has particles inside a lip polygon, or the first particles to lose height are in the shaft rather than under the waist.

### Pitfall 4: The plate tunnels or stalls

**What goes wrong:** Liquid stays below the plate, or the plate never leaves the bottom after the dwell.

**Why it happens:** Lift speed is a large fraction of a diameter per step, the limit range is under `0.01` m so `solve_motor` applies nothing, or `collide_connected` stays false and the plate ghosts through the shaft walls.

**How to avoid:** `0.15` m/s at this radius. Limits about `0.02` m outside a stroke of roughly a meter. `with_collide_connected(true)` on the plate only. Max force `1.0e6`.

**Warning signs:** Particles inside the plate polygon, or translation still `0` after the dwell while commanded speed is clearly positive.

### Pitfall 5: Catalog id without its companions

**What goes wrong:** Typecheck or unit tests fail because `SceneId` grew in one list only.

**Why it happens:** Several maps are `Record<SceneId, ...>` and `previews.tsx` switches on every id. README plans and demo capture plans compare order to `SCENE_IDS`. `demo-media-model.test.ts` asserts the last id.

**How to avoid:** Update every file in the touch list in the same wave as `SCENE_IDS`.

**Warning signs:** `assertReadmeSvgPlanCoverage` length mismatch, or `expectedPaths` missing a key, or the last-id assertion still says `wave-tank`.

### Pitfall 6: Age destruction or a finite lifetime empties the toy

**What goes wrong:** The live count falls during the proof, so the crossing ids are gone, and a long play session runs dry.

**Why it happens:** Water Wheel enables destruction by age and a 3 second lifetime because it emits. The default system definition also has `destroy_by_age: true`, which starts evicting once a maximum count exists.

**How to avoid:** Infinite recipe lifetime (the default `0`). `with_destruction_by_age(false)`. No maximum count. Assert `live_particle_count` unchanged.

**Warning signs:** The crossing test cannot find the original ids, or the count at 2 seconds is lower than step 0.

## Code Examples

### Wheel angle the test can read

```rust
// Source: crates/liquidfun/src/world/joint/revolute.rs
pub fn revolute_joint_angle(&self, joint: JointId) -> Result<f32, JointQueryError>
```

At step 0 expect `0`. After the proof window expect the absolute angle to be at least `0.05` rad. That is a change from the initial pose, not a full turn. Exact equality to a scripted angle is the wrong check. [VERIFIED: `revolute_joint_angle`]

### Waist crossing assertion

```rust
// Source: ParticleSystemView::particle_ids and positions
// Stable ids stay aligned with the position slice.
let started_above = view
    .particle_ids()
    .iter()
    .copied()
    .zip(view.positions().iter().copied())
    .filter(|(_id, position)| position.y > waist_top)
    .collect::<Vec<_>>();

// After 2 seconds of advance(4):
// one of those ids has position.y < waist_exit
// and its x is in the lower chamber, not inside the wheel body
// live_particle_count is unchanged
// revolute_joint_angle absolute value is at least 0.05
// prismatic translation is still near 0
```

`read_particles` is `cfg(test)`. Drive the run through `SessionCore::advance` in batches of at most 4. `World::step` alone never runs `on_advance`. [VERIFIED: `session.rs`; Wave Tank tests use `advance(4)`]

### Recommended starting numbers

These are a starting point inside Claude's discretion, not locked geometry. Change them until the crossing test passes. Record any radius or count shrink as a playground adaptation (D-06). Do not retune Water Wheel, Color Mixer, Liquid Timer, or Hydraulic Fountain to borrow a look.

| Parameter | Start | Constraint |
|-----------|-------|------------|
| Radius | `0.025` | Stride `0.0375` m. Default system radius is `1.0` and must be replaced. |
| Damping | `0.2` | So the pinch stream keeps falling. Not a shared constant. |
| Color | `(242, 176, 64, 255)` | One amber. Not basin blue and not Color Mixer's red. |
| Gravity | `(0, -10)` | Same as the other basins. |
| Waist gap | `0.14` m | About three diameters. Widen if the chamber stays empty. |
| Wheel density | `0.20` | Lighter than Water Wheel's `0.45`, which is sized for an 8 m/s jet. |
| Hub / paddle outer | `0.12` m / `0.32` m | Four paddles, under the waist, clear of the walls and the shaft. |
| Angular damping | `0.05` | Water Wheel's local number, copied, not shared. |
| Plate speed | `0.15` m/s after the dwell | About `0.0025` m per step. One-diameter speed is `3` m/s. |
| Dwell | `3.0` s | Longer than the proof window. Speed is `0` until then. |
| Proof window | `2.0` s | 120 steps, 30 calls of `advance(4)`. |
| Stroke | about `1.2` m | Side shaft only. Limits `0.02` m outside that stroke. |
| Max motor force | `1.0e6` | Same order as the hydraulic piston. Quiet comes from speed and placement. |
| Angle floor | `0.05` rad | Changed from `0`. Do not require a full rotation. |
| Destruction by age | `false` | No maximum count. Lifetime stays the recipe default `0`. |

[ASSUMED] These particular meters put original reservoir particles below the waist and move the wheel by at least `0.05` rad within 2 seconds, while the plate is still down. The inequalities (static waist, dynamic motor-off wheel, plate outside the paddle circle, dwell longer than the proof, lift speed under one diameter per step, stable ids, infinite lifetime) are verified from the current solver and session.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Water Wheel turns from an aimed jet on a motor-off revolute | This scene leaves that demo alone and turns a smaller wheel with a waist drip | D-01, D-03 | Do not retune `water-wheel` |
| Hydraulic Fountain's timed throat piston is the spectacle | Same prismatic setter, vertical, slower, in a side shaft, delayed | This phase | Reuse the hook, not the show |
| Color Mixer blends two groups | One `with_color` on one water group | D-05 | Do not retune `color-mixer` |

**Deprecated/outdated:**

- Do not treat the Phase 32 roadmap stub ("To be planned") as scope. `32-CONTEXT.md` is the scope. [VERIFIED: `32-CONTEXT.md` canonical refs]
- Do not add the Phase 33 stacked drip fidget in this phase. [VERIFIED: D-04 and Deferred Ideas]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Waist gap `0.14` m, radius `0.025`, and damping `0.2` let reservoir particles fall through within 2 seconds without arching | Recommended starting numbers | The lower chamber stays empty. Widen the gap. Do not cut the count, shrink the radius, or raise the 4-step cap. |
| A2 | Wheel density `0.20`, four paddles out to `0.32` m, and angular damping `0.05` change the angle by at least `0.05` rad in that window | Pattern 2 | If the angle stays near `0`, lower the density or lengthen the paddles under the drip. Do not enable the revolute motor. |
| A3 | A side shaft whose inlet is past the paddles, plus a plate that does not move for 3 seconds, leaves the proof cohort below the waist | Pattern 3 | If ids are already back above the waist, lengthen the dwell and keep the proof shorter than the dwell. Do not switch to position writes. |
| A4 | Amber `(242, 176, 64, 255)` reads as one colored liquid in the existing particle renderer | Pattern 4 | If the tint is invisible, pick another opaque non-blue color on this scene only. Do not enable `COLOR_MIXING`. |

## Open Questions (RESOLVED)

1. **Will the first drip both cross and turn the wheel before the lift starts?** — RESOLVED
   - What we know: Free-fall time over about a meter at `g = 10` is under a second. The plate is commanded to stay down for 3 seconds. The angle comes from existing particle-body impulses.
   - Resolution: Plan 01 owns the retune. If nothing crosses, widen the waist. If the wheel does not move, lighten it. If the plate moves during the proof, the dwell was not applied. Do not add an engine feature.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Native crossing test and WASM build | ✓ | 1.97.0 | — |
| just | `just web-player-smoke` | ✓ | 1.48.0 | Run `bun scripts/web-build.ts player-smoke` directly |
| bun | Player smoke script | ✓ | 1.4.2 | — |
| node | Tooling beside bun | ✓ | v24.13.0 | — |
| Playwright Chromium | D-10 | Installed by the smoke script | — | No second browser |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none. Chromium comes from the existing smoke recipe. Do not add Firefox or Safari.

Step 2.6 covers those tools because the phase proof is a local Rust test plus the existing Chromium script. No new service is required.

## Validation Architecture

`workflow.nyquist_validation` is `false` in `.planning/config.json`. This is not a Nyquist matrix. The phase gates are:

- Native: `cargo test -p liquidfun-wasm scene::liquid_bubbler`
- Browser: `just web-player-smoke`

The native test is the waist crossing and the wheel-angle change. The smoke suite opens, plays, pauses, and resets the new scene and the scenes already in the catalog. Leave `ALL_SCENE_TIMEOUT_MS` at `380000` and both step caps at 4 unless this Chromium run exceeds that budget. Phase 31 left the timeout there after a 36.1 second pass. [VERIFIED: `player-helpers.ts`; `.planning/STATE.md` Phase 31]

## Security Domain

`workflow.security_enforcement` is not a key in `.planning/config.json`. The spawn instructions treat that absence as enabled, and the phase threat model is ASVS level 1 with a block on high findings. This scene has no accounts, no secrets, and no network calls.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | Physics stays in `liquidfun`. The scene is a local WASM shell. No new trust boundary. |
| V2 Authentication | no | Local playground scene. No account. |
| V3 Session Management | no | WASM `SessionCore` is a physics session, not a user session. |
| V4 Access Control | no | No privilege boundary. |
| V5 Input Validation | yes | Scene id stays on the `parse_scene_id` allowlist. Non-empty presets are rejected as `SessionError::UnknownControl`. Pointer input is a no-op. Credit path stays a single `scene/*.rs` file. [VERIFIED: `scene.rs`; `links.ts`; Wave Tank and Hydraulic Fountain reject presets] |
| V6 Cryptography | no | None. |
| V7 Errors and logging | yes | Construction and step failures stay typed `SessionError` values. Do not log particle positions or credit URLs to a remote sink. |
| V8 Data protection | no | No stored personal data. |
| V9 Communications | no | No fetch and no WebSocket from this scene. |
| V10 Malicious code | yes | Do not add `unsafe`. Do not load remote scripts from the preview SVG. |
| V11 Business logic | yes | The wheel must not be motor-driven while the catalog text says the drip turns it. The crossing test locks that. |
| V12 Files and resources | no | No file upload. |
| V13 API | no | No HTTP API. |
| V14 Configuration | yes | Do not raise `MAX_ADVANCE_STEPS`. Do not embed a secret. |

### Known Threat Patterns for this scene

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown scene id or control string | Tampering | Existing parse rejects unknown tokens. The new scene adds one known token and no new control names besides the shared gravity slider. |
| Credit URL leaving the host allowlist | Spoofing | `sceneBlobUrl` rejects paths outside `crates/liquidfun-wasm/src/scene/*.rs`. Do not put a Google test URL in the implementation path. Inspiration stays the existing showcase href. |
| Position write used as a hidden return | Tampering | The crossing test requires original `ParticleId`s to fall below the waist while the plate translation is still near `0` and the live count stays constant. |
| Motor-driven wheel presented as a drip | Spoofing | Source guard rejects `set_revolute_motor_speed`. The angle test runs with the lift still down, so a motor is not required to pass. |

No high-severity finding is expected if the scene stays inside that allowlist and does not add network, secrets, or `unsafe`.

## Suggested Plan Grain

Three small waves. Do not write `PLAN.md` from this note. Do not invent `PLAY-03`. Do not implement Phase 33.

### Wave 1: Scene and crossing proof

Files:

- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` (new)
- `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` (new)
- `crates/liquidfun-wasm/src/scene.rs` (`mod`, `SceneId::LiquidBubbler`, `parse_scene_id`, `build_scene`)
- `crates/liquidfun-wasm/src/session/tests.rs` (token list)

Tests, one concern each, arrange / act / assert:

- Step 0: nonzero count, every particle is above the waist, every velocity is zero, wheel angle is `0`, plate translation is `0`
- After 2 seconds in `advance(4)` batches, live count is unchanged, at least one original above-waist id is below the waist exit in the lower chamber, and the absolute wheel angle is at least `0.05` rad
- At that same sample, plate translation is still near `0`
- The revolute joint's motor stays disabled. The only prismatic joint uses an upward axis and is the plate, whose body does not overlap the wheel
- Rebuild restores angle `0`, plate translation `0`, and the reservoir layout
- Unknown control, unknown action, and a pointer no-op that does not change the live count
- `parse_scene_id("liquid-bubbler")` succeeds
- `include_str` of `liquid_bubbler.rs`: calls `set_prismatic_motor_speed` and `with_destruction_by_age(false)`, and does not call `set_revolute_motor_speed`, `create_particle_with_def`, `set_particle_position`, or `with_color_mixing_strength`

Leave `MAX_ADVANCE_STEPS` at 4. Do not edit `water_wheel.rs`, `color_mixer.rs`, `liquid_timer.rs`, or `hydraulic_fountain.rs`.

Quick command: `cargo test -p liquidfun-wasm scene::liquid_bubbler`

### Wave 2: Catalog companions

Files, all append-only after the wave-tank entries:

- `web/src/catalog/scenes.ts`
- `web/src/catalog/scene-records.ts`
- `web/src/catalog/previews.tsx`
- `web/src/catalog/portrait-bounds.ts`
- `web/scripts/readme-svg/plans.ts`
- `web/scripts/demo-media/model.ts`
- `web/e2e/player-helpers.ts` (`SCENE_HASH_PATHS` only)
- `web/tests/scenes.test.ts` (description, watch-first hint list)
- `web/tests/scene-catalog-controls.test.ts` (gravity slider only)
- `web/tests/scene-catalog-credits.test.ts` (path map and showcase-only inspiration)
- `web/tests/portrait-bounds.test.ts` (`MIN_HEIGHT_FRACTION` plus wall endpoints, including the plate at rest and at the top of the stroke)
- `web/tests/demo-media-model.test.ts` (the last id is no longer `wave-tank`)

`README_SVG_PLANS` and `SCENE_CAPTURE_PLANS` must follow `SCENE_IDS` order. Portrait walls include the reservoir, the waist, the wheel, the chamber, and the shaft. Landscape keeps `viewBounds` on the scene record. The preview SVG shows a narrow waist with a wheel beneath a colored drip. The caption is already `Static preview`.

Do not add a pointer or labeled-control entry in `player.spec.ts` `POINTER_CONTROL`. `WATCH_FIRST_SCENE_IDS` is derived from `SCENE_IDS` minus that map. Do not add a README cue.

Quick command once the maps exist: `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts tests/scene-catalog-credits.test.ts tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts`

### Wave 3: Browser proof

Run `just web-player-smoke` once the maps exist. The watch-first test is derived: `WATCH_FIRST_SCENE_IDS` is every `SCENE_IDS` entry absent from `POINTER_CONTROL`. It then opens `#/scene/liquid-bubbler`, pauses, plays, and resets, and still walks the scenes that were already in the catalog. Leave `ALL_SCENE_TIMEOUT_MS` at `380000` unless this run exceeds it. Do not rasterize README assets in this wave.

Independent review is after implementation. The implementing agent records evidence and does not approve its own work.

## Sources

### Primary (HIGH confidence)

- `crates/liquidfun/src/joint/definition/revolute_prismatic.rs` — `RevoluteJointDef::new` leaves the motor off; `with_motor`; prismatic `with_motor` and `with_limits`
- `crates/liquidfun/src/world/joint/revolute.rs` — `revolute_joint_angle`, `set_revolute_motor_speed`
- `crates/liquidfun/src/world/body/control.rs` — `candidate_apply_linear_impulse` adds an angular term for dynamic bodies
- `crates/liquidfun/src/particle/solver/pressure.rs` — body contact impulse at the particle position; damping on approaching contacts
- `crates/liquidfun/src/world/particle_coupling/body_coupling.rs` — solver impulse reaches that body method
- `crates/liquidfun/src/math/settings.rs` — `LINEAR_SLOP` `0.005`, `PARTICLE_STRIDE` `0.75`
- `crates/liquidfun/src/particle/group/recipe.rs` — default `ParticleFlags::WATER`, zero velocity, `with_color`
- `crates/liquidfun/src/particle/definition/system_definition.rs` — default radius `1.0`, damping `1.0`, `destroy_by_age: true`, `color_mixing_strength` `0.5`
- `crates/liquidfun/src/particle/solver/material.rs` — color mixing runs only with `ParticleFlags::COLOR_MIXING`
- `crates/liquidfun/src/particle/lifetime.rs` — age eviction requires a maximum count
- `crates/liquidfun-wasm/src/session.rs` — `on_advance` then step, `MAX_ADVANCE_STEPS = 4`
- `crates/liquidfun-wasm/src/session/escape.rs` — 12 m / 48 m deletion
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — motor-off pin, paddle fixtures, jet and lifetime that this scene must not copy
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` — prismatic write, `collide_connected(true)`, constants to stay quieter than
- `crates/liquidfun-wasm/src/scene.rs` — `wave-tank` is the last parsed id
- `web/src/catalog/scenes.ts`, `scene-records.ts`, `portrait-bounds.ts`, `previews.tsx`, `links.ts`
- `web/src/components/DemoNavigation.tsx` — caption `Static preview`
- `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`
- `web/e2e/player.spec.ts`, `web/e2e/player-helpers.ts` — `ALL_SCENE_TIMEOUT_MS` `380000`, derived watch-first list
- `web/tests/portrait-bounds.test.ts`, `web/tests/demo-media-model.test.ts`
- `.planning/phases/32-liquid-motion-bubbler/32-CONTEXT.md` — locked scope
- `.planning/REQUIREMENTS.md` — v1.3 rows already complete
- `.planning/config.json` — `nyquist_validation` false

### Secondary (MEDIUM confidence)

- `crates/liquidfun/src/world/particle_object/group.rs` — sample stride is `PARTICLE_STRIDE * diameter`
- `.planning/STATE.md` Phase 31 — smoke timeout left at 380000 after a 36.1 second pass
- Phase 31 research — file split so the credit path stays a single `scene/*.rs` file; same catalog touch list with `wave-tank` as the previous last id

### Tertiary (LOW confidence)

- None beyond A1 through A4 in the assumptions log. Those numbers are a starting geometry, not a measured run.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new libraries. The revolute pin, the prismatic setter, the particle color, and the catalog seams are in tree.
- Architecture: HIGH — a static waist, a motor-off wheel, and a delayed side-shaft plate match the locked decisions and the current solver.
- Pitfalls: HIGH for a motorized wheel, a teleport return, age destruction, and catalog coverage. MEDIUM for the exact gap, density, and dwell, which the crossing test has to confirm.

**Research date:** 2026-09-27
**Valid until:** 2026-10-27

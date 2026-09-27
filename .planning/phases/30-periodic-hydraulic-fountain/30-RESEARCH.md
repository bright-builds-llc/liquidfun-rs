# Phase 30: Periodic hydraulic fountain - Research

**Researched:** 2026-09-27
**Domain:** Original playground scene. A dynamic piston on a limited prismatic motor pushes one water group through a throat.
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Travel layout
- **D-01:** Add a new scene. Do not replace, hide, or retune `fountain`. The catalog id is `hydraulic-fountain`, appended after `sparky`.
- **D-02:** The layout is two chambers joined by a throat. One chamber holds the liquid and a piston. The other chamber is where that liquid arrives. A cycle is recognizable when liquid that started on the piston side is later on the fountain side.
- **D-03:** The piston advances and then retracts on a repeating schedule. Liquid is not destroyed or spawned to fake the jet. The existing Fountain emitter is not reused.

#### Drive
- **D-04:** Drive the piston with a prismatic joint whose motor speed is rewritten from simulation time, the same kind of live motor write Wave Machine already uses for its revolute joint. Do not teleport a kinematic body through the liquid.
- **D-05:** The period is built in. The scene is watch-first: play, pause, and reset only. Do not add period, stroke, or aim controls. Reset restores the initial piston pose and the initial liquid layout.

#### Liquid
- **D-06:** Use one plain water particle group in the piston chamber. The spectacle is the squeeze and the crossing, not a new material flag. Do not add tensile, elastic, or rigid groups in this phase.
- **D-07:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

#### Catalog, credits, and proof
- **D-08:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/hydraulic-fountain`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-09:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the chambers, throat, and piston fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `hydraulic-fountain` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-10:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-11:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, within one period, particles that began in the piston chamber are present on the fountain side of the throat. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-12:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber sizes, throat width, piston stroke, period, particle radius, and particle count, as long as one cycle visibly moves liquid from the piston chamber through the throat and the walls stay inside the portrait frame.
- Whether liquid returns through a low passage or by falling back when the piston retracts, as long as the motion repeats and particles are not deleted to fake the jet.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a piston, a throat, and two chambers.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Phase 31 sinusoidal wave tank: a still pool whose end platform rises and falls.
- Phase 32 liquid motion bubbler: colored liquid drips through a narrow waist and turns a wheel.
- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts.
- Visitor aim, emission rate, or launch speed. Those controls belong to the existing Fountain.
- A return pipe elaborate enough to be its own machine, if a simple retract-and-repeat cycle already shows travel.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| — | No phase requirement IDs are assigned. | Do not invent a `PLAY-03` credit or any new upstream-test requirement. `PLAY-01`, `PLAY-02`, and `PLAY-03` are already complete for the twelve testbed ports. [VERIFIED: `.planning/REQUIREMENTS.md`] |

This phase is an original playground scene. Its gate is D-11, not a requirements-matrix row.
</phase_requirements>

## Summary

A periodic advance and retract fits the prismatic joint that already ships. Enable the motor and translation limits, then rewrite motor speed from simulation time inside `on_advance`, which the session already calls before `World::step`. The piston has to be a dynamic body. A kinematic body has zero mass, so the motor impulse does not move it, and writing its pose each step is the teleport D-04 forbids. [VERIFIED: `crates/liquidfun/src/world/joint/prismatic.rs`, `crates/liquidfun/src/world/body/mass.rs`, `crates/liquidfun-wasm/src/session.rs`]

No new engine API is required. The scene is a WASM module plus the catalog companions every new id already has. Fountain stays an aimed emitter that creates short-lived particles. This scene fills one water group once, in the piston chamber, and leaves the fountain chamber empty. Return is gravity through a low throat: plain water has no tensile flag, so retracting the piston does not suck the jet back.

**Primary recommendation:** Add `hydraulic-fountain` as a watch-first scene whose dynamic piston is driven by `World::set_prismatic_motor_speed` on a half-period schedule, with limits holding the stroke, a low throat between two open chambers, and a native `SessionCore` test that the original particle ids cross that throat within one period while the live count stays constant.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory is present. [VERIFIED: glob of `.cursor/rules/**`]. These repo rules still bind the planner:

- Physics stays in `liquidfun`. The WASM crate is the scene shell. SolidJS stays controls and rendering. [VERIFIED: `standards/core/architecture.md`, `30-CONTEXT.md`]
- A new catalog id also needs a portrait frame and a README SVG plan. Do not mdformat `.planning/**`. [VERIFIED: `AGENTS.md` repo-local guidance]
- Safe Rust, no `unwrap()` on the production path, `foo.rs` plus `foo/` if the scene module is split. [VERIFIED: `standards/languages/rust.md`]
- Shared gravity slider may stay, because every catalog scene includes it. D-05 forbids period, stroke, and aim controls, not that slider. [VERIFIED: `web/src/catalog/scene-records.ts` `withGravitySlider`]
- Implementing agent does not self-approve. Passing `just web-player-smoke` is not a review acknowledgment. [VERIFIED: `AGENTS.md` Independent review, D-12]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace crate, Rust 1.97.0 | Prismatic joint, particle group, world step | The motor and limits already exist. [VERIFIED: `cargo 1.97.0`; `World::set_prismatic_motor_speed`] |
| `liquidfun-wasm` | workspace crate | Scene build, `on_advance` motor write, `SessionCore` | Wave Machine and Soup Stirrer already live here. [VERIFIED: `crates/liquidfun-wasm/src/scene.rs`] |
| SolidJS catalog | existing `web/` | Record, preview, portrait frame, hash route | Append one id. Do not add a UI library. [VERIFIED: `web/src/catalog/scenes.ts`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | existing `web` tests | Catalog, credit, portrait, README-plan coverage | Every exhaustive `SceneId` map |
| Playwright Chromium | via `just web-player-smoke` | Open, play, pause, reset | D-11 browser gate only |
| `bun` | 1.4.2 installed | Runs `scripts/web-build.ts player-smoke` | The smoke alias. [VERIFIED: `justfile`, `bun --version`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `set_prismatic_motor_speed` plus limits | A new engine oscillator or sine target translation | Rejected. The public setter and limit solver already express advance, hold, and retract. |
| Dynamic piston | Kinematic body plus `set_transform` | Rejected by D-04. Kinematic inverse mass is zero, so the motor cannot drive it. [VERIFIED: `initial_body_mass`] |
| One water `ParticleGroupRecipe` | Fountain `create_particle_with_def` | Rejected by D-03 and D-06. That path emits a stream and turns on destruction by age. |

**Installation:** none. Do not add crates or npm packages.

**Version verification:** `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustc 1.97.0 (2d8144b78 2026-07-07)`, `just 1.48.0`, `bun 1.4.2`. [VERIFIED: local probes on 2026-09-27]

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs   # build, walls, group, on_advance
crates/liquidfun-wasm/src/scene.rs                      # SceneId, parse, build_scene
web/src/catalog/scenes.ts                               # append id after sparky
web/src/catalog/scene-records.ts                        # watch-first record
web/src/catalog/previews.tsx                            # static piston / throat / chambers
web/src/catalog/portrait-bounds.ts                      # phone frame
web/scripts/readme-svg/plans.ts                         # coverage entry, cues: []
web/scripts/demo-media/model.ts                         # center-click stub, catalog order
```

Start as one scene file. Split to `hydraulic_fountain.rs` plus `hydraulic_fountain/` only if the file approaches the 300–500 line preference. The credit path must stay `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`. `sceneBlobUrl` rejects a path with a slash under `scene/`. [VERIFIED: `web/src/catalog/links.ts` `isAllowlistedScenePath`]

Do not edit `crates/liquidfun-wasm/src/scene/fountain.rs` or `MAX_ADVANCE_STEPS`.

### Pattern 1: Dynamic piston, limited prismatic motor

**What:** Static chamber body, dynamic piston, prismatic joint along the stroke, limits at the retracted and advanced translations, motor enabled with a large max force.

**When to use:** The whole drive. This is the engine surface for D-04.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/soup_stirrer.rs create_paddle_rail
// plus crates/liquidfun/src/joint/definition/revolute_prismatic.rs
// PrismaticJointDef::with_limits and with_motor
let definition = PrismaticJointDef::new(ground, piston)?
    .with_collide_connected(true)
    .with_frame(piston_position, Vec2::ZERO, Vec2::new(1.0, 0.0), 0.0)?
    .with_limits(true, 0.0, stroke)?
    .with_motor(true, advance_speed, max_motor_force)?;
world.create_joint(JointDef::from(definition))?;
```

`with_collide_connected(true)` matches Soup Stirrer's rail. The default is `false`. If the walls and the joint anchor share the static body, `false` lets the piston overlap those wall fixtures. Particles still contact both fixtures, which is how a piston can squash water through a wall. [VERIFIED: `PrismaticJointDef::new` sets `collide_connected: false`; `soup_stirrer.rs` sets it true]

The velocity solver applies the motor, then the limit. At the upper limit the axial impulse is clamped so translation does not run past `upper_translation`. At the lower limit it does not run past `lower_translation`. Holding a constant signed speed while the piston sits on a limit is the dwell. The schedule does not have to move the body itself. [VERIFIED: `PrismaticConstraint::solve_velocity` and `PrismaticRuntime::solve_constraint_velocity` in `crates/liquidfun/src/world/joint/prismatic.rs`]

Set limits so the initial translation is the retracted end. `stage_prismatic` measures translation as the axis dot of the anchor separation at the authored pose. A `with_frame` that matches that pose starts at translation `0`. Use `0.0..stroke` only when the build pose is the retracted pose. [VERIFIED: `stage_prismatic` `translation = axis.dot(d)`]

### Pattern 2: Rewrite speed in `on_advance`, before the step

**What:** Session `advance` calls `hooks.on_advance`, then steps the world at `1/60` s. Wave Machine adds `SIM_DT` and writes motor speed there.

**When to use:** The built-in period. SolidJS does not integrate time.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/session.rs advance (on_advance then world.step)
// and crates/liquidfun-wasm/src/scene/wave_machine.rs WaveMachineHooks::on_advance
fn on_advance(&mut self, world: &mut World, _system: ParticleSystemId) -> Result<(), SessionError> {
    self.elapsed += SIM_DT;
    let squeezing = self.elapsed % self.period < self.period * 0.5;
    let speed = if squeezing { self.advance_speed } else { -self.advance_speed };
    world
        .set_prismatic_motor_speed(self.joint, speed)
        .map_err(|_error| SessionError::StepFailed)
}
```

Use a square wave of speed, not a copied cosine. Wave Machine's cosine rocks a revolute joint that has no end stop. A piston needs a signed speed that reaches each limit and stays there until the half-period flips. Choose the period strictly longer than `2 * stroke / speed` so the piston arrives and the limit holds before the sign changes. [VERIFIED: Wave Machine writes `drive.motor_speed(angle)` from sim time; prismatic limits are a separate constraint]

`MAX_ADVANCE_STEPS` is `4`. The browser may call `advance(4)`, and each of those four iterations runs `on_advance` once. Integrating `1/60` per `on_advance` stays aligned with substeps. Do not advance the period once per browser frame. [VERIFIED: `session.rs` loop around `on_advance`]

Reset already disposes the WASM session and calls `beginScene`, which builds a new world. Elapsed time stored on the hooks returns to zero. Do not add a SolidJS period integrator or a custom rewind. [VERIFIED: `web/src/player/scene-lifecycle.ts` `beginScene`]

### Pattern 3: One water group, empty fountain chamber, low throat

**What:** `ParticleGroupRecipe::new` defaults to `ParticleFlags::WATER`. Fill a polygon that sits in the piston chamber and does not overlap the piston solid. Leave the fountain chamber empty at step 0.

**When to use:** D-06. Do not call `with_particle_flags` for tensile, elastic, powder, or rigid. Do not enable `destruction_by_age`.

**Layout to use:** Two open-top chambers, a shared floor, and a dividing wall with a throat at the floor. The piston is a vertical face that travels horizontally toward that throat. Open tops vent the squeeze. The throat is low so, when the piston retracts, liquid that has pooled in the fountain chamber can fall back. Plain water does not stick to a retracting face. A high throat with no low return will leave the fountain chamber full and the piston chamber empty after the first stroke. [VERIFIED: `ParticleGroupRecipe` default flags in `crates/liquidfun/src/particle/group/recipe.rs`; D-06 forbids tensile]

Throat width should be several particle diameters so the jet is a stream. A gap near one diameter becomes a cork. Keep the whole layout inside a few meters when radius is about `0.025`, so escaped-particle eviction does not delete the jet. Escape deletion starts at 12 m down-gravity or 48 m sideways, and also when position in diameters exceeds half of `PROXY_TAG_HALF_EXTENT_DIAMETERS` (`2048`). At radius `0.025` that tag keep is about 51 m. A tumbler-scale radius in a meter-scale box is the opposite problem. [VERIFIED: `crates/liquidfun-wasm/src/session/escape.rs`, `crates/liquidfun/src/particle/proxy.rs`]

### Pattern 4: Catalog append, watch-first

**What:** Append `"hydraulic-fountain"` after `"sparky"` in `SCENE_IDS`. Add the same id everywhere `SceneId` is exhaustive.

**Watch-first record:** `ready: true`, `interactionHint` equal to the existing `WATCH_FIRST_HINT`, `controls: withGravitySlider([])`. Description is one behavior and says this is an original experimental scene. Inspiration is the Phase 18 showcase link only, the same shape as Dam Break, Water Wheel, and Liquid Tumbler: label `LiquidFun showcase`, href `https://google.github.io/liquidfun/`. Do not add a `github.com/google/liquidfun/blob/...` test URL. Implementation path is `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`. [VERIFIED: `scene-records.ts` `SHOWCASE` and `liquid-tumbler` / `water-wheel` credits]

Preview is a `switch` arm in `previews.tsx`. The caption `Static preview` is already rendered by `DemoNavigation` for every preview. The SVG still has to draw a piston, a throat, and two chambers. [VERIFIED: `web/src/components/DemoNavigation.tsx`]

README plan entry is `{ id: "hydraulic-fountain", cues: [] }` appended after `sparky`. Watch-first scenes leave cues empty because the piston moves on its own. [VERIFIED: `web/scripts/readme-svg/plans.ts`]

Demo capture still wants a plan for every `SCENE_IDS` entry. Append a center-click stub `{ kind: "click", point: { x: 0.5, y: 0.5 } }` with `interactionStep: 180`, matching Liquid Tumbler and Sparky. Pointer up on this scene is a no-op. Do not run `just readme-svg` as the phase gate. [VERIFIED: `web/scripts/demo-media/model.ts`; D-09]

### Anti-Patterns to Avoid

- **Kinematic pose writes:** Zero inverse mass means `set_prismatic_motor_speed` does not translate the body. Moving it with a transform skips the liquid. [VERIFIED: `initial_body_mass`]
- **Fountain emission:** `fountain.rs` calls `create_particle_with_def` and `with_destruction_by_age(true)`. That is a spawned stream, not a moved group.
- **Speed above one diameter per step:** Dam Break treats `2 * radius * 60` as the one-diameter-per-step speed. Particles meet fixtures by contacts, not continuous collision. A fast piston face passes through the group. [VERIFIED: `dam_break/tests.rs` `critical = 2.0 * particle_radius * 60.0`]
- **Direct `World::step` in the crossing test:** That skips `on_advance`, so the motor never changes speed. Drive the test through `SessionCore::advance`, in batches of at most 4.
- **PLAY-03 citation:** No pinned test href, no "matches testHydraulic" copy.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Periodic stroke | A new joint type or a position servo in the engine | `set_prismatic_motor_speed` plus `with_limits` | Limit solve already stops the axis at each end |
| Schedule | A SolidJS timer or requestAnimationFrame phase | `elapsed += 1/60` inside `on_advance` | The session may step 1 to 4 times per frame |
| The jet | `create_particle`, lifetime, or `destroy_particle` each step | One `ParticleGroupRecipe` at build | D-03. Spawn/destroy keeps a count while changing identity |
| Return flow | Tensile or elastic flags, or a second machine | A low throat and gravity `(0, -10)` | D-06. Water does not stick to the retracting face |
| Reset | Custom piston rewind | Existing session dispose and rebuild | `beginScene` already constructs from the scene builder |
| Phone framing | The shared 12 m by 9 m frame | `PORTRAIT_VIEW_BOUNDS["hydraulic-fountain"]` | A wide frame becomes a short band on a 390 by 844 canvas |

**Key insight:** The missing piece is a scene, not a solver feature. The public prismatic motor, the pre-step hook, and stable `ParticleId` rows are enough to move and to prove the liquid moved.

## Common Pitfalls

### Pitfall 1: Speed that tunnels the group

**What goes wrong:** The piston face jumps farther than a particle diameter in one `1/60` s step. Water stays behind the face or sticks inside it, so the fountain chamber never receives the original particles.

**Why it happens:** Body-particle contact is a discrete push. Dam Break's regression uses one diameter per step as the critical speed.

**How to avoid:** Keep `|speed|` at or below about one third of `2 * radius * 60`. At radius `0.025` that ceiling is `3` m/s and the recommended speed is at most `1` m/s (`0.01` m per step versus a `0.05` m diameter). Set `max_motor_force` high enough that the face actually tracks that speed against the group. A huge force with a modest speed tracks the target. A huge speed does not become safe by lowering the force, because the face can still move too far in one step once it is moving.

**Warning signs:** Fountain-side count stays zero while piston translation reaches the upper limit, or particles appear inside the piston fixture.

### Pitfall 2: Identity-blind "some particles are on the right"

**What goes wrong:** The test passes because new particles were spawned on the fountain side, or because both chambers started full.

**Why it happens:** `particle_count` stays constant if every destroy is matched by a create. Positions alone do not say which particles moved.

**How to avoid:** Build with every live particle on the piston side of a fixed throat plane, and zero particles on the fountain side. Snapshot `particle_ids()` zipped with `positions()`. After one period, require the live count to be unchanged and at least one of those same ids to have a position on the fountain side. `ParticleSystemView::particle_ids` is documented as stable identities aligned with positions, and the permutation remaps rows rather than minting new ids. [VERIFIED: `crates/liquidfun/src/particle/view.rs`]

**Warning signs:** A test that only checks `count > 0` past the throat, or that steps with raw `World::step`.

### Pitfall 3: Escape eviction faking a drain

**What goes wrong:** Particles that fly through an open top or an unstopped throat are destroyed by `evict_escaped_particles` before the next step. The chamber looks like it sprayed and vanished.

**Why it happens:** Session `advance` destroys particles more than 12 m along gravity, 48 m sideways, or outside the particle tag keep. [VERIFIED: `session/escape.rs`]

**How to avoid:** Close the sides and floor. Keep the free surface inside the open top with walls that are tall enough for one stroke. Keep coordinates small relative to `1024 * diameter`. The crossing test's stable live count fails if eviction runs.

**Warning signs:** `live_particle_count()` drops during the period.

### Pitfall 4: Period shorter than the stroke

**What goes wrong:** The speed sign flips before the face reaches the throat. Liquid sloshes and never crosses.

**Why it happens:** A square wave of amplitude `speed` needs time `stroke / speed` to arrive, then the same time to return.

**How to avoid:** `period > 2 * stroke / speed`. The extra time is the limit dwell. Prove a crossing within that one period, not after several hopeful cycles.

**Warning signs:** Mid-period piston translation is still near zero, or the fountain side stays empty until many periods later.

### Pitfall 5: Catalog id without its companions

**What goes wrong:** Typecheck or unit tests fail because `SceneId` grew in one list only.

**Why it happens:** Several maps are `Record<SceneId, ...>` and `previews.tsx` switches on every id. README plans and demo capture plans compare order to `SCENE_IDS`.

**How to avoid:** Update the files in the touch list below in the same wave as `SCENE_IDS`.

**Warning signs:** `assertReadmeSvgPlanCoverage` length mismatch, or `expectedPaths` missing a key.

### Pitfall 6: Reusing Fountain behavior

**What goes wrong:** The new scene looks like an aimed nozzle. Credits or copy cite Faucet or a pinned test.

**Why it happens:** `fountain` is the nearest name and already has emission, aim, and lifetime.

**How to avoid:** Leave `fountain.rs` and the fountain catalog record untouched. The new description talks about a piston and a throat. Inspiration is the showcase link only.

**Warning signs:** `create_particle_with_def`, `with_destruction_by_age(true)`, or an aim control on the new scene.

## Code Examples

### Motor write already used by Wave Machine

```rust
// Source: crates/liquidfun-wasm/src/scene/wave_machine.rs
fn write_motor_speed(world: &mut World, joint: JointId, speed: f32) -> Result<(), SessionError> {
    world
        .set_revolute_motor_speed(joint, speed)
        .map_err(|_error| SessionError::StepFailed)
}
```

The piston equivalent is `World::set_prismatic_motor_speed`. It wakes both bodies. [VERIFIED: `prismatic.rs` doc comment on `set_prismatic_motor_speed`]

### Water group fill

```rust
// Source: crates/liquidfun-wasm/src/scene/wave_machine.rs create_particle_fill
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_color(GROUP_COLOR)
    .with_transform(Transform::IDENTITY)?;
world.create_particle_group(system, &recipe)?;
```

Leave flags at the recipe default `ParticleFlags::WATER`. Color `(77, 163, 255, 255)` matches the existing water blue.

### Crossing test shape

```rust
// Source pattern: crates/liquidfun-wasm/src/scene/sparky/tests.rs
// SessionCore::advance in batches of at most 4, then read_particles.
// Positions and ids: World::particle_system_view
let started = session.read_particles(|world, system| {
    let view = world.particle_system_view(system).expect("system live");
    view.particle_ids()
        .iter()
        .copied()
        .zip(view.positions())
        .filter(|(_id, position)| position.x < THROAT_X)
        .map(|(id, _position)| id)
        .collect::<Vec<_>>()
});
advance_steps(&mut session, STEPS_PER_PERIOD);
let crossed = session.read_particles(|world, system| {
    let view = world.particle_system_view(system).expect("system live");
    started.iter().any(|id| {
        view.particle_ids()
            .iter()
            .zip(view.positions())
            .any(|(live, position)| live == id && position.x > THROAT_X)
    })
});
```

Also assert the start snapshot contains every live particle, the fountain side starts empty, and `live_particle_count()` after one period equals the start count. Step only through `SessionCore::advance`. `read_particles` is `cfg(test)`. [VERIFIED: `session.rs`]

### Recommended starting numbers

These are a starting point inside Claude's discretion, not locked geometry. Change them until the crossing test passes. Record any radius or count shrink as a playground adaptation (D-07).

| Parameter | Start | Constraint |
|-----------|-------|------------|
| Radius | `0.025` | Same order as Wave Machine. Do not drop it only to look smoother. |
| Advance speed | `0.6` m/s | At or below `1` m/s for this radius. Critical one-diameter speed is `3` m/s. |
| Stroke | `0.35` m | Travel time about `0.58` s. Stop short of sealing the throat with the face. |
| Period | `2` s | `120` steps at `1/60`. Greater than `2 * stroke / speed`. |
| Throat | about `0.12` m | Several diameters, on the floor, dividing wall above it. |
| Gravity | `(0, -10)` | Same as the other basins. |
| Max motor force | `1.0e6` or higher | Enough that the face reaches the limit before the half-period. Wave Machine uses `1.0e7` torque on a revolute motor. |

[ASSUMED] These particular meters pass the crossing test. The inequalities (dynamic body, limits, speed under one diameter per step, period longer than the round trip, low throat, stable ids) are verified from the current solver and session.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Kinematic scripted platforms | Motor speed written from sim time before `World::step` | Wave Machine scene | Reuse that hook for the piston |
| Soup Stirrer prismatic rail with the motor off | Same joint with motor and limits enabled | This phase | Rail pattern already sets `with_collide_connected(true)` |
| Fountain emission | One static water group | D-03, D-06 | Do not share `fountain.rs` |

**Deprecated/outdated:**

- Do not treat the Phase 30 roadmap stub ("To be planned") as scope. `30-CONTEXT.md` is the scope. [VERIFIED: `ROADMAP.md` still says the detail stub is unplanned]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Stroke `0.35` m, speed `0.6` m/s, period `2` s, radius `0.025`, and a `0.12` m floor throat cross within one period | Recommended starting numbers | The native test fails. Adjust those numbers inside discretion. Do not add an engine feature or cut the 4-step cap. |
| A2 | A horizontal floor throat returns liquid on the retract half without a second pipe | Pattern 3 | If the pool does not fall back, widen the throat or lower the fountain floor. Do not add tensile flags or a delete/spawn refill. |

## Open Questions (RESOLVED)

1. **Will the first numbers reach the limit against the water group?** — RESOLVED
   - What we know: Motor impulse is clamped by `timestep * max_motor_force`, and limits hold once translation gets there.
   - Resolution: Plan 01 owns the retune. If the face stalls, raise max force and keep speed. If particles tunnel, lower speed and lengthen the period. Both stay inside discretion. Do not add an engine feature.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Native crossing test and WASM build | ✓ | 1.97.0 | — |
| just | `just web-player-smoke` | ✓ | 1.48.0 | Run `bun scripts/web-build.ts player-smoke` directly |
| bun | Player smoke script | ✓ | 1.4.2 | — |
| node | Tooling beside bun | ✓ | v24.13.0 | — |
| Playwright Chromium | D-11 | Installed by the smoke script | — | No second browser |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none. Chromium comes from the existing smoke recipe. Do not add Firefox or Safari.

Step 2.6 covers those tools because the phase proof is a local Rust test plus the existing Chromium script. No new service is required.

## Validation Architecture

`workflow.nyquist_validation` is `false` in `.planning/config.json`. This section is included because the phase prompt required it.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` for the crossing proof. Vitest for catalog maps. Playwright Chromium for the player. |
| Config file | `crates/liquidfun-wasm` integration via `cargo test -p liquidfun-wasm`. `web` uses its existing Vitest and Playwright config. |
| Quick run command | `cargo test -p liquidfun-wasm scene::hydraulic_fountain` |
| Full suite command | `just web-player-smoke` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-04, D-11 | Original piston-chamber particles are on the fountain side within one period, live count unchanged | native | `cargo test -p liquidfun-wasm piston_chamber_particles_cross_the_throat_within_one_period` | ❌ Wave 0 |
| D-05 | Unknown control and action rejected. Pointer is a no-op and does not change particle count | native | same crate, beside the crossing test | ❌ Wave 0 |
| D-01, D-08 | Id parses, record is ready, hint is watch-first, controls are gravity only, description is one behavior | unit | `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts` | ❌ extend existing files |
| D-10 | Implementation path host-locked. Inspiration is the showcase link only | unit | `cd web && bun run test:unit -- tests/scene-catalog-credits.test.ts` | ❌ extend existing file |
| D-09 | Portrait frame fills the phone and contains the new walls. README plan order matches `SCENE_IDS` | unit | `cd web && bun run test:unit -- tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts` | ❌ extend existing files |
| D-11 | New scene play, pause, reset. Existing scenes still open | e2e | `just web-player-smoke` | ❌ path map only. Watch-first loop is derived |

### Sampling Rate

- **Per task commit:** `cargo test -p liquidfun-wasm scene::hydraulic_fountain` while the scene exists, then the Vitest files above once catalog maps exist.
- **Per wave merge:** those tests for the wave that landed.
- **Phase gate:** `just web-player-smoke` exit 0, and the native crossing test exit 0. README raster export is not the gate.

### Wave 0 Gaps

- [ ] `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` tests — crossing, stable count, unknown controls
- [ ] Extend `session/tests.rs` `parse_scene_id_maps_allowlisted_tokens` with `("hydraulic-fountain", SceneId::HydraulicFountain)`
- [ ] Extend the `Record<SceneId, _>` fixtures listed under suggested plan grain
- [ ] Framework install: none

The watch-first Playwright loop does not need a new spec. `WATCH_FIRST_SCENE_IDS` is every `SCENE_IDS` entry absent from `POINTER_CONTROL`. Leave the new id out of `POINTER_CONTROL` and add it to `SCENE_HASH_PATHS`. [VERIFIED: `web/e2e/player.spec.ts`] `ALL_SCENE_TIMEOUT_MS` is `380_000` for the all-scene loops. If one extra scene blows that budget, raise the timeout. Do not drop the new scene from the loop.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local playground scene. No account. |
| V3 Session Management | no | WASM `SessionCore` is a physics session, not a user session. |
| V4 Access Control | no | No privilege boundary. |
| V5 Input Validation | yes | Scene id stays on the `parse_scene_id` allowlist. New controls are rejected as `SessionError::UnknownControl`. Credit path stays a single `scene/*.rs` file. [VERIFIED: `scene.rs`, `links.ts`] |
| V6 Cryptography | no | None. |

### Known Threat Patterns for this scene

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown scene id or control string | Tampering | Existing parse rejects unknown tokens. New scene adds one known token and no new control names besides the shared gravity slider. |
| Credit URL leaving the host allowlist | Spoofing | `sceneBlobUrl` rejects paths outside `crates/liquidfun-wasm/src/scene/*.rs`. Do not put a Google test URL in the implementation path. |
| Particle destroy used as a hidden jet | Tampering | Crossing test requires the original `ParticleId`s to move and the live count to stay constant. |

## Suggested Plan Grain

Three small waves. Do not write `PLAN.md` from this note. Do not invent `PLAY-03`.

### Wave 1: Scene and crossing proof

Files:

- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` (new)
- `crates/liquidfun-wasm/src/scene.rs` (`mod`, `SceneId::HydraulicFountain`, `parse_scene_id`, `build_scene`)
- `crates/liquidfun-wasm/src/session/tests.rs` (token list)

Tests:

- Piston-chamber particle ids are past the throat after one period, live count unchanged, fountain side empty at step 0
- Unknown control, unknown action, pointer no-op
- `parse_scene_id("hydraulic-fountain")` succeeds
- Optional source check, matching Wave Machine's `include_str` test, that the implementation calls `set_prismatic_motor_speed` and does not call `create_particle_with_def` or `with_destruction_by_age`

Leave `MAX_ADVANCE_STEPS` at 4. Do not edit `fountain.rs`.

### Wave 2: Catalog companions

Files, all append-only beside the existing fountain record:

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
- `web/tests/portrait-bounds.test.ts` (`MIN_HEIGHT_FRACTION` plus wall endpoints for the new chambers)
- `web/tests/demo-media-model.test.ts` (the last id is no longer `sparky`)

`README_SVG_PLANS` and `SCENE_CAPTURE_PLANS` must follow `SCENE_IDS` order. Portrait walls include the chambers, the throat divider, and the piston's retracted and advanced extremes, inside the phone rectangle, clear of the title and transport. Landscape keeps `viewBounds` on the scene record.

Do not add a pointer or labeled-control entry in `player.spec.ts` `POINTER_CONTROL`.

### Wave 3: Browser proof

Run `just web-player-smoke` once the maps exist. The watch-first test then opens `#/scene/hydraulic-fountain`, pauses, plays, and resets, and still walks the scenes that were already in the catalog. Do not rasterize README assets in this wave.

Independent review is after implementation. The implementing agent records evidence and does not approve its own work.

## Sources

### Primary (HIGH confidence)

- `crates/liquidfun/src/world/joint/prismatic.rs` — `set_prismatic_motor_speed`, limit velocity solve, motor impulse clamp
- `crates/liquidfun/src/joint/definition/revolute_prismatic.rs` — `with_limits`, `with_motor`, default `collide_connected: false`
- `crates/liquidfun/src/world/joint/solver/primary.rs` and `staging.rs` — prismatic staging and velocity order
- `crates/liquidfun/src/world/body/mass.rs` — kinematic and static mass start at zero
- `crates/liquidfun-wasm/src/session.rs` — `on_advance` then step, `MAX_ADVANCE_STEPS = 4`, `read_particles`
- `crates/liquidfun-wasm/src/scene/wave_machine.rs` — sim-time motor write
- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs` — prismatic rail
- `crates/liquidfun-wasm/src/scene/fountain.rs` — emitter to leave untouched
- `crates/liquidfun/src/particle/group/recipe.rs` — default `ParticleFlags::WATER`
- `crates/liquidfun/src/particle/view.rs` — stable `particle_ids` aligned with `positions`
- `crates/liquidfun-wasm/src/session/escape.rs` — playground particle deletion
- `web/src/catalog/scenes.ts`, `scene-records.ts`, `portrait-bounds.ts`, `previews.tsx`
- `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`
- `web/e2e/player.spec.ts`, `web/e2e/player-helpers.ts`
- `web/src/catalog/links.ts` — single-file scene path allowlist
- `.planning/phases/30-periodic-hydraulic-fountain/30-CONTEXT.md` — locked scope

### Secondary (MEDIUM confidence)

- `crates/liquidfun-wasm/src/scene/dam_break/tests.rs` — one diameter per step as the speed that a wall contact is expected to survive
- `web/src/player/scene-lifecycle.ts` — reset rebuilds the session

### Tertiary (LOW confidence)

- None beyond A1 and A2 in the assumptions log. Those numbers are a starting geometry, not a measured run.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new libraries. The joint, hook, and catalog seams are in tree.
- Architecture: HIGH — dynamic limited prismatic motor plus a pre-step speed write matches the locked decisions and the current solver.
- Pitfalls: HIGH for tunneling, eviction, emission, and kinematic mass. MEDIUM for the exact chamber numbers, which the crossing test has to confirm.

**Research date:** 2026-09-27
**Valid until:** 2026-10-27

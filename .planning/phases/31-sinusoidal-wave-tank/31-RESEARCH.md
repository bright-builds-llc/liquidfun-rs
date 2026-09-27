# Phase 31: Sinusoidal wave tank - Research

**Researched:** 2026-09-27
**Domain:** Original playground scene. A dynamic end platform on a vertical prismatic motor whose speed is a sine of simulation time sends a wave across one still water group.
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Pool and platform
- **D-01:** Add a new scene. Do not replace, hide, or retune `wave-machine`. The catalog id is `wave-tank`, appended after `hydraulic-fountain`. The title is Wave Tank.
- **D-02:** The layout is one rectangular pool of still water. A platform at one end rises and falls. The rest of the floor and the far wall stay fixed. A cycle is recognizable when that rise and fall sends a wave across the pool toward the far wall.

#### Drive
- **D-03:** Drive the platform with a prismatic joint whose motor speed follows a sinusoid of simulation time, the same kind of live motor write Hydraulic Fountain uses for its prismatic joint and Wave Machine uses for its revolute joint. Do not teleport a kinematic body through the liquid. Do not rock the whole tank.
- **D-04:** The period and stroke are built in. The scene is watch-first: play, pause, and reset only. Do not add period, amplitude, or stroke controls. Reset restores the initial platform pose and the initial still-pool layout.

#### Liquid
- **D-05:** Use one plain water particle group. The spectacle is the traveling wave, not a new material flag. Do not add tensile, elastic, or rigid groups in this phase.
- **D-06:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

#### Catalog, credits, and proof
- **D-07:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/wave-tank`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-08:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the pool, the moving end, and the far wall fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `wave-tank` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-09:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-10:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that the pool starts as a still band and that, after the platform has completed at least a half cycle, particles that began in a sample near the far wall have moved vertically. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact pool length, platform width, stroke, period, particle radius, and particle count, as long as one cycle visibly sends a wave from the moving end toward the far wall and the walls stay inside the portrait frame.
- Whether the moving piece is a short floor slab at one end or a vertical face that translates with that slab, as long as the motion is a rise and fall of the end platform and the rest of the pool floor stays still.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a pool with one end platform displaced from the resting floor.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Phase 32 liquid motion bubbler: colored liquid drips through a narrow waist and turns a wheel.
- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts.
- Visitor amplitude, period, or stroke controls.
- Replacing or retuning Wave Machine.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| — | No phase requirement IDs are assigned (`phase_req_ids` is null). | Do not invent a `PLAY-03` credit or any new upstream-test requirement. `PLAY-01`, `PLAY-03`, and `ACT-04` are already complete. Wave Machine stays the rocking-container scene. [VERIFIED: `.planning/REQUIREMENTS.md`] |

This phase is an original playground scene. Its gate is D-10, not a requirements-matrix row.
</phase_requirements>

## Summary

A rising and falling end platform fits the prismatic motor that already ships. Put the axis on `(0, 1)`, enable the motor, and write speed from simulation time inside `on_advance`, which the session already calls before `World::step`. The speed command is a sine, not Hydraulic Fountain's square wave and not Wave Machine's revolute cosine. A sine that starts at zero speed leaves the pool still, then lifts the platform to the top of the stroke at the half period. [VERIFIED: `World::set_prismatic_motor_speed`, `hydraulic_fountain.rs` `scheduled_motor_speed`, `wave_machine/drive.rs` `feedforward_speed`, `session.rs` `advance`]

The platform has to be a dynamic body. A kinematic body has zero mass, so the motor impulse does not move it, and writing its pose each step is the teleport D-03 forbids. The prismatic joint also locks relative angle, so the tank does not rock. Give the moving body both a short floor slab and a vertical face at the joint with the fixed floor. A bare slab opens a gap there. The rest of the floor and the far wall stay on the static ground body. [VERIFIED: `initial_body_mass`; D-03; discretion on the face]

No new engine API is required. Do not edit `wave_machine.rs`, `MAX_ADVANCE_STEPS`, or the Hydraulic Fountain scene.

**Primary recommendation:** Add `wave-tank` as a watch-first scene whose dynamic end platform is driven by `World::set_prismatic_motor_speed` with `peak_speed * sin(TAU * elapsed / period)`, limits only as a safety margin outside that stroke, one water group, and a native `SessionCore` test that the pool starts as a flat zero-velocity band and that, after a half period of `advance(4)` batches, original particle ids sampled beside the far wall have a higher `y` by at least one diameter.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory is present. [VERIFIED: glob of `.cursor/rules/**`]. These repo rules still bind the planner:

- Physics stays in `liquidfun`. The WASM crate is the scene shell. SolidJS stays controls and rendering. [VERIFIED: `standards/core/architecture.md`, `31-CONTEXT.md`]
- A new catalog id also needs a portrait frame and a README SVG plan. Do not mdformat `.planning/**`. [VERIFIED: `AGENTS.md` README scene gallery and playground canvas framing]
- Safe Rust, no `unwrap()` on the production path, `foo.rs` plus `foo/` if the scene module is split. Tests use arrange, act, assert, and one concern each. [VERIFIED: `standards/languages/rust.md`, `standards/core/testing.md`]
- Shared gravity slider may stay, because every catalog scene includes it. D-04 forbids period, amplitude, and stroke controls, not that slider. [VERIFIED: `web/src/catalog/scene-records.ts` `withGravitySlider` on `hydraulic-fountain`]
- Dark playground chrome already exists. Do not restyle the shell. [VERIFIED: `standards/core/frontend-ui.md`, D-07]
- Implementing agent does not self-approve. Passing `just web-player-smoke` is not a review acknowledgment. [VERIFIED: `AGENTS.md` Independent review, D-11]
- Before commit, run the repo-native checks for the files this phase touches. `just web-player-smoke` is the browser proof. [VERIFIED: `standards/core/verification.md`, `justfile` `web-player-smoke`]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace crate, Rust 1.97.0 | Vertical prismatic joint, one water group, world step | The motor and limits already exist. [VERIFIED: `cargo 1.97.0`; `World::set_prismatic_motor_speed`] |
| `liquidfun-wasm` | workspace crate | Scene build, `on_advance` sine write, `SessionCore` | Hydraulic Fountain already writes this joint from sim time. [VERIFIED: `crates/liquidfun-wasm/src/scene.rs`] |
| SolidJS catalog | existing `web/` | Record, preview, portrait frame, hash route | Append one id. Do not add a UI library. [VERIFIED: `web/src/catalog/scenes.ts`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | existing `web` tests | Catalog, credit, portrait, README-plan coverage | Every exhaustive `SceneId` map |
| Playwright Chromium | via `just web-player-smoke` | Open, play, pause, reset | D-10 browser gate only |
| `bun` | 1.4.2 installed | Runs `scripts/web-build.ts player-smoke` | The smoke alias. [VERIFIED: `justfile`, `bun --version`] |
| `std::f32::consts::TAU` | Rust 1.97.0 | Full-turn constant for the speed sine | Already used by Wave Machine's drive. Do not import that drive. [VERIFIED: `wave_machine/drive.rs`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `set_prismatic_motor_speed` with `peak * sin(TAU * t / period)` | Hydraulic Fountain's square wave of speed, or a new position servo | Rejected. D-03 requires a sinusoid of simulation time. The square wave is the fountain's dwell schedule. |
| Dynamic platform | Kinematic body plus `set_transform` | Rejected by D-03. Kinematic inverse mass is zero, so the motor cannot drive it. [VERIFIED: `initial_body_mass`] |
| Slab plus a vertical face on the same dynamic body | A floor slab alone | Use the face. Discretion allows either. A slab-only joint with the fixed floor leaves a gap particles fall through. |
| One water `ParticleGroupRecipe` | `create_particle_with_def` or extra material flags | Rejected by D-05. Spawned particles and tensile or elastic flags are a different spectacle. |
| Particle damping `0.2` | `ParticleSystemDef` default damping `1.0` | Use `0.2`. Default damping is the strength applied to approaching contacts. Wave Machine sets `0.2` so a free surface keeps moving. The fountain can keep `1.0` because a piston forces the crossing. Do not edit Wave Machine to share the constant. [VERIFIED: `ParticleSystemDef::default` damping `1.0`; `pressure.rs` contact damping; `wave_machine.rs` `PARTICLE_DAMPING`] |

**Installation:** none. Do not add crates or npm packages.

**Version verification:** `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustc 1.97.0 (2d8144b78 2026-07-07)`, `just 1.48.0`, `bun 1.4.2`, `node v24.13.0`. [VERIFIED: local probes on 2026-09-27]

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun-wasm/src/scene/wave_tank.rs          # build, walls, group, on_advance
crates/liquidfun-wasm/src/scene/wave_tank/tests.rs    # still band and far-wall rise
crates/liquidfun-wasm/src/scene.rs                    # SceneId, parse, build_scene
web/src/catalog/scenes.ts                             # append id after hydraulic-fountain
web/src/catalog/scene-records.ts                      # watch-first record
web/src/catalog/previews.tsx                          # static pool with a raised end
web/src/catalog/portrait-bounds.ts                    # phone frame
web/scripts/readme-svg/plans.ts                       # coverage entry, cues: []
web/scripts/demo-media/model.ts                       # center-click stub, catalog order
```

Start as `wave_tank.rs` plus `wave_tank/tests.rs`. Hydraulic Fountain moved its tests out once the implementation plus inline tests would pass 500 physical lines. This scene's proof is the same size. The credit path must stay `crates/liquidfun-wasm/src/scene/wave_tank.rs`. `sceneBlobUrl` rejects a path with a slash under `scene/`. [VERIFIED: `web/src/catalog/links.ts` `isAllowlistedScenePath`; Phase 30 STATE note on the test split]

Do not edit `crates/liquidfun-wasm/src/scene/wave_machine.rs`, `wave_machine/drive.rs`, `hydraulic_fountain.rs`, or `MAX_ADVANCE_STEPS`.

### Pattern 1: Dynamic slab and face, vertical prismatic motor

**What:** Static ground holds the long floor, the near wall, and the far wall. One dynamic body holds a short floor slab at one end and a vertical face at the slab's inner edge. The prismatic axis is world-up. The motor is enabled. Limits sit slightly outside the analytic stroke so they do not clip the sine.

**When to use:** The whole drive. This is the engine surface for D-03.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs create_piston_joint
// Axis (0, 1) replaces that scene's (1, 0). Limits are a safety margin, not the waveform.
let definition = PrismaticJointDef::new(ground, platform)?
    .with_collide_connected(true)
    .with_frame(platform_position, Vec2::ZERO, Vec2::new(0.0, 1.0), 0.0)?
    .with_limits(true, -LIMIT_MARGIN, stroke + LIMIT_MARGIN)?
    .with_motor(true, 0.0, max_motor_force)?;
world.create_joint(JointDef::from(definition))?;
```

Build the platform at translation `0`, flush with the fixed floor. `with_frame` that matches that pose starts at translation `0`, because staging measures translation as the axis dot of the anchor separation. [VERIFIED: Phase 30 research against `stage_prismatic`; Hydraulic Fountain uses the same frame-at-build pattern for a horizontal axis]

`with_collide_connected(true)` matches Hydraulic Fountain and Soup Stirrer. The default is `false`. The vertical face and the fixed floor are on different bodies, so they collide anyway. `true` still matters if any fixture shares the ground body with the joint anchor. [VERIFIED: `PrismaticJointDef::new` sets `collide_connected: false`; `hydraulic_fountain.rs` sets it true]

The velocity solver applies the motor, then the limit. `solve_motor` applies no impulse when `limit_state` is `Equal`. `Equal` is chosen when the enabled limit range is shorter than `2 * LINEAR_SLOP` (`0.01` m). A useful stroke must be a real range, wider than that, and wider than the sine's peak so the half cycle stays `Inactive` instead of sitting on `AtUpper` and flattening the crest. [VERIFIED: `PrismaticRuntime::solve_motor`, `classify_limit`, `LINEAR_SLOP`]

Set `BodyType::Dynamic`, sleeping off, density high enough that the fixture has mass. Max motor force `1.0e6`, the same order Hydraulic Fountain uses, so the face tracks the sine against the water. A huge force with a modest speed tracks the target. A huge speed does not become safe by lowering the force. [VERIFIED: `hydraulic_fountain.rs` `MAX_MOTOR_FORCE`]

### Pattern 2: Sine speed in `on_advance`, before the step

**What:** Session `advance` calls `hooks.on_advance`, then evicts escaped particles, then steps the world at `1/60` s. Add `SIM_DT` first, then write motor speed, matching Hydraulic Fountain's order.

**When to use:** The built-in period. SolidJS does not integrate time.

**Example:**

```rust
// Source: crates/liquidfun-wasm/src/session.rs advance
// and crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs on_advance
fn sinusoidal_motor_speed(elapsed: f32) -> f32 {
    let phase = elapsed * std::f32::consts::TAU / PERIOD;
    PEAK_SPEED * phase.sin()
}

fn on_advance(&mut self, world: &mut World, _system: ParticleSystemId) -> Result<(), SessionError> {
    self.elapsed += SIM_DT;
    world
        .set_prismatic_motor_speed(self.joint, sinusoidal_motor_speed(self.elapsed))
        .map_err(|_error| SessionError::StepFailed)
}
```

Use a sine, not a copied cosine and not the fountain's half-period sign flip. `speed(t) = V * sin(ω t)` with `ω = TAU / PERIOD` integrates to `translation(t) = (V / ω) * (1 - cos(ω t))`. That translation is zero at `t = 0` and at every full period, and it peaks at `2 V / ω` at every half period. Choose `V = stroke * TAU / (2 * PERIOD)` so the peak is the authored stroke. The first command, after adding one `1/60`, is `sin(ω / 60) * V`, nearly zero, so the pool is not kicked on step 1. [VERIFIED: the sine integral; `hydraulic_fountain.rs` increments `elapsed` before the write]

`MAX_ADVANCE_STEPS` is `4`. The browser may call `advance(4)`, and each of those four iterations runs `on_advance` once. Integrating `1/60` per `on_advance` stays aligned with substeps. Do not advance the period once per browser frame. [VERIFIED: `session.rs`]

Reset already disposes the WASM session and calls `beginScene`, which builds a new world. Elapsed time stored on the hooks returns to zero. Do not add a SolidJS period integrator or a custom rewind. [VERIFIED: Hydraulic Fountain rebuild test and `web/src/player/scene-lifecycle.ts` usage in Phase 30]

Half-period step count is `PERIOD * 60`. Keep that count divisible by 4 so the native test uses only `advance(4)`. Period `2` gives 60 steps, which is 15 batches. [VERIFIED: `MAX_ADVANCE_STEPS`]

### Pattern 3: One water group on a still rectangle

**What:** `ParticleGroupRecipe::new` defaults to `ParticleFlags::WATER` and `Vec2::ZERO` velocity. Fill one polygon that sits on the slab and on the fixed floor and does not overlap either solid. Leave flags at that default.

**When to use:** D-05. Do not call `with_particle_flags` for tensile, elastic, powder, or rigid. Do not enable destruction by age as a lifetime. Recipe lifetime `0` is the pinned infinite lifetime. [VERIFIED: `ParticleGroupRecipe::new`; `with_lifetime` docs]

Set particle damping to `0.2` on `ParticleSystemDef`. That is the strength `pressure.rs` applies to approaching contact normals. The default definition uses `1.0`, which is what Hydraulic Fountain inherits and is too strong for a free surface that has to cross the pool. Copy the numeric damping only. Do not call Wave Machine's revolute writer. [VERIFIED: `system_definition.rs` default; `wave_machine.rs` `PARTICLE_DAMPING`]

Sample stride is `PARTICLE_STRIDE * diameter` with `PARTICLE_STRIDE = 0.75`. At radius `0.025` the stride is `0.0375` m. A pool of about `1.5` m by `0.3` m is a few hundred particles. Do not shrink that count to look smoother. Escape deletion starts at 12 m down-gravity or 48 m sideways. A meter-scale pool stays inside that keep. [VERIFIED: `world/particle_object/group.rs` stride formula; `settings.rs` `PARTICLE_STRIDE`; `session/escape.rs`]

Gravity `(0, -10)`, water color `(77, 163, 255, 255)`, matching the other basins. [VERIFIED: `hydraulic_fountain.rs`, `wave_machine.rs`]

### Pattern 4: Catalog append, watch-first

**What:** Append `"wave-tank"` after `"hydraulic-fountain"` in `SCENE_IDS`. Add the same id everywhere `SceneId` is exhaustive.

**Watch-first record:** `ready: true`, `interactionHint` equal to the existing `WATCH_FIRST_HINT` (`"This scene is watch-first. Use Play scene, Pause scene, and Reset scene."`), `controls: withGravitySlider([])`. Description is one behavior and says this is an original experimental scene. Inspiration is the Phase 18 showcase link only, the same shape as Hydraulic Fountain: label `LiquidFun showcase`, href `https://google.github.io/liquidfun/`. Do not add a `github.com/google/liquidfun/blob/...` test URL. Implementation path is `crates/liquidfun-wasm/src/scene/wave_tank.rs`. [VERIFIED: `scene-records.ts` `SHOWCASE`, `WATCH_FIRST_HINT`, and the `hydraulic-fountain` record]

Preview is a `switch` arm in `previews.tsx`. The caption `Static preview` is already rendered by `DemoNavigation` for every preview. The SVG still has to draw a pool and one end platform displaced above the resting floor. [VERIFIED: `web/src/components/DemoNavigation.tsx`]

README plan entry is `{ id: "wave-tank", cues: [] }` appended after `hydraulic-fountain`. Watch-first scenes leave cues empty. `readmeControls` returns `[]` for every id except `wave-machine`. Do not add a cue or a wave-speed control for this plan. [VERIFIED: `web/scripts/readme-svg/plans.ts`]

Demo capture still wants a plan for every `SCENE_IDS` entry. Append a center-click stub `{ kind: "click", point: { x: 0.5, y: 0.5 } }` with `interactionStep: 180`, matching Hydraulic Fountain. Pointer up on this scene is a no-op. Do not run `just readme-svg` as the phase gate. [VERIFIED: `web/scripts/demo-media/model.ts`; D-08]

`viewBounds` on the scene record and `PORTRAIT_VIEW_BOUNDS["wave-tank"]` should both be the tight pool rectangle. Landscape reads `worldBoundsForScene`. Leaving the shared 12 m by 9 m frame makes this pool a short band on a phone. [VERIFIED: `portrait-bounds.ts` header; `worldBoundsForViewport`]

`VIEWPORT_INSET` is 16 CSS pixels inside `createCamera`, and the portrait test measures the fitted fraction after that inset. A wide pool fills the phone width and only part of the height. Set `MIN_HEIGHT_FRACTION["wave-tank"]` to a value the finished rectangle clears. Target `0.20` by padding the pool tightly, including the slab below the floor and the wall tops. Do not copy Hydraulic Fountain's `0.28` unless this frame actually reaches it. Include the fixed walls and the platform corners at rest and at the crest in the `wallEndpoints` map so both poses stay inside the phone frame. [VERIFIED: `web/src/render/camera.ts` `VIEWPORT_INSET`; `web/tests/portrait-bounds.test.ts`]

### Anti-Patterns to Avoid

- **Kinematic pose writes:** Zero inverse mass means `set_prismatic_motor_speed` does not translate the body. Moving it with a transform skips the liquid. [VERIFIED: `initial_body_mass`]
- **Wave Machine reuse:** That scene is a dynamic four-wall tank on a revolute joint. Copying `WaveDrive` rocks the whole container, which D-03 forbids, and editing it retunes the pinned demo, which D-01 forbids.
- **Fountain square wave:** `scheduled_motor_speed` holds a constant signed speed until the half period. Limits then dwell at the stroke. A wave tank needs the speed itself to be the sine, with limits only as a backstop.
- **Speed above one diameter per step:** Body-particle contact is discrete. Hydraulic Fountain's research treated `2 * radius * 60` as the one-diameter-per-step speed. The sine peak must stay well under that. [VERIFIED: `crates/liquidfun/src/particle/solver/pressure.rs` contact impulse; Phase 30 Dam Break critical-speed note]
- **Far sample on top of the platform:** Those particles rise because the slab lifts them. The wave proof is the sample beside the far wall, over the fixed floor.
- **Downward Δy as a pass:** Compaction moves the fill down. The passing delta is upward.
- **Direct `World::step` in the motion test:** That skips `on_advance`, so the sine is never written. Drive the test through `SessionCore::advance`, in batches of at most 4.
- **PLAY-03 citation:** No pinned test href, no "matches testWaveMachine" copy.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Sinusoidal stroke | A new joint type or a position servo in the engine | `set_prismatic_motor_speed` with `V * sin(TAU * t / period)` | The existing motor tracks speed. The integral of that sine is the smooth rise and fall. |
| Schedule | A SolidJS timer or requestAnimationFrame phase | `elapsed += 1/60` inside `on_advance` | The session may step 1 to 4 times per frame |
| The wave | Particle spawn, lifetime, or `destroy_particle` each step | One `ParticleGroupRecipe` at build | D-05. Spawn/destroy can keep a count while changing who moved. |
| The seal at the step | A second machine or a kinematic gate | One dynamic body: slab plus vertical face | The face translates with the slab and closes the joint with the fixed floor. |
| Reset | Custom platform rewind | Existing session dispose and rebuild | `beginScene` already constructs from the scene builder |
| Phone framing | The shared 12 m by 9 m frame | `PORTRAIT_VIEW_BOUNDS["wave-tank"]` | A wide frame becomes a short band on a 390 by 844 canvas |
| Rocking | `set_revolute_motor_speed` or `WaveDrive` | Vertical prismatic axis `(0, 1)` | D-01 and D-03. Wave Machine already owns the rocking tank. |

**Key insight:** The missing piece is a scene, not a solver feature. The public prismatic motor, the pre-step hook, and stable `ParticleId` rows are enough to lift one end and to prove the far-wall water moved up.

## Common Pitfalls

### Pitfall 1: Square wave or cosine copied from the neighbors

**What goes wrong:** The platform slams between two speeds, or it is already at full speed on the first step, so the pool never has a still start. Or the whole tank rocks.

**Why it happens:** Hydraulic Fountain's live write is a half-period sign flip. Wave Machine's live write is `peak * phase.cos()` on a revolute joint.

**How to avoid:** `PEAK_SPEED * (elapsed * TAU / PERIOD).sin()` after adding `1/60`. Axis `(0, 1)`. Leave both neighbor modules untouched. A source test can require `set_prismatic_motor_speed` and reject `set_revolute_motor_speed`.

**Warning signs:** The first sampled motor speed equals a constant `ADVANCE_SPEED`, or a revolute joint exists.

### Pitfall 2: Limits that clip the sine or disable the motor

**What goes wrong:** The crest goes flat, or the platform never moves.

**Why it happens:** `AtUpper` clamps further positive motion while the discrete sine is still slightly positive. `Equal` (limit range under `0.01` m) makes `solve_motor` return zero impulse.

**How to avoid:** Analytic peak is `stroke`. Set limits to about `-0.02 .. stroke + 0.02`. Do not set the limits to `0 .. stroke` as the waveform itself. Do not use a zero-length range.

**Warning signs:** After a half period, translation is stuck at the upper limit while commanded speed is still clearly positive, or translation stays `0` with the motor enabled.

### Pitfall 3: Far-wall motion that is settle, not a wave

**What goes wrong:** The test passes because particles dropped onto the floor, or because the slab lifted the water that was sitting on it.

**Why it happens:** A filled polygon placed inside the slab overlaps a solid and then pops. Gravity then pulls a lattice down. Sampling the platform column measures the slab, not the wave.

**How to avoid:** Place the fill strictly above both floor tops. At step 0, assert every velocity is `Vec2::ZERO` and the free-surface `y` values across the pool, including the far window, agree within one diameter. The far window is the last `0.20` m before the far wall, over the fixed floor. After the half period, require an original id from that window whose `y` increased by at least one diameter (`0.05` m at radius `0.025`). Compaction moves `y` down, so an upward threshold rejects it. Keep `live_particle_count` unchanged.

**Warning signs:** The start surface already slopes, start velocities are nonzero, or the only particles that rose began above the slab.

### Pitfall 4: The wave has not arrived, or it is only a few millimeters

**What goes wrong:** Half a period ends with the platform at the crest and the far sample still flat.

**Why it happens:** A long channel, strong damping, or a tiny stroke. Shallow-water travel time is about `length / sqrt(g * depth)`. A half period of `1` s at depth `0.32` and `g = 10` covers about `1.8` m. A uniform spread of `platform_width * stroke` over a long pool can also raise the far surface by less than one diameter.

**How to avoid:** Start with the channel past the step near `0.90` m, depth near `0.32` m, stroke near `0.16` m, and damping `0.2`, so travel time is about half of the half period and the stroke is several diameters. If the far surface rises less than one diameter, shorten the channel or raise the stroke, and lengthen the period only enough that `period * 60` stays divisible by 4. Do not lower the upward threshold below one diameter, drop the radius, or raise the 4-step cap.

**Warning signs:** Platform translation reaches the stroke while every far-window `Δy` is under one diameter.

### Pitfall 5: Speed that tunnels the group

**What goes wrong:** The slab jumps farther than a particle diameter in one `1/60` s step. Water stays below the top or inside the face, and no wave leaves.

**Why it happens:** The peak of the sine was chosen as a large amplitude instead of the derivative of the stroke.

**How to avoid:** `PEAK_SPEED = stroke * TAU / (2.0 * PERIOD)`. At stroke `0.16` m and period `2` s that peak is about `0.25` m/s, or `0.004` m per step, against a `0.05` m diameter. Keep the peak under about `1` m/s at this radius. Critical one-diameter speed is `2 * radius * 60 = 3` m/s.

**Warning signs:** Particles inside the slab polygon, or the far window empty because particles were evicted.

### Pitfall 6: Catalog id without its companions

**What goes wrong:** Typecheck or unit tests fail because `SceneId` grew in one list only.

**Why it happens:** Several maps are `Record<SceneId, ...>` and `previews.tsx` switches on every id. README plans and demo capture plans compare order to `SCENE_IDS`.

**How to avoid:** Update the files in the touch list below in the same wave as `SCENE_IDS`.

**Warning signs:** `assertReadmeSvgPlanCoverage` length mismatch, or `expectedPaths` missing a key.

## Code Examples

### Speed the motor already accepts

```rust
// Source: crates/liquidfun/src/world/joint/prismatic.rs
pub fn set_prismatic_motor_speed(
    &mut self,
    joint: JointId,
    speed: f32,
) -> Result<(), JointMutationError>
```

The setter wakes both bodies. It fails for a non-finite speed, the wrong joint kind, or a locked world. Call it from `on_advance`, before `World::step`, so the world is unlocked. [VERIFIED: `set_prismatic_motor_speed` doc comment; `session.rs` order]

### Half-cycle translation the test can read

```rust
// Source pattern: crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs
// world_observation joints, then World::prismatic_joint_translation
let translation = world
    .prismatic_joint_translation(joint_id)
    .expect("the platform joint should be readable");
```

After `PERIOD * 30` steps (a half period at `1/60`), expect `translation >= 0.5 * stroke`. Exact equality is the wrong check: the write uses the right-hand sample of the sine. Also expect the joint's `motor_speed()` near zero at that sample, which a square wave would fail. The speed is the definition speed after the last `on_advance`. [VERIFIED: `prismatic_joint_translation`; Hydraulic Fountain `prismatic_motor_speed` helper]

### Far-wall assertion

```rust
// Source: ParticleSystemView::particle_ids, positions, velocities
// Stable ids are aligned with both slices.
let started = view
    .particle_ids()
    .iter()
    .copied()
    .zip(view.positions().iter().copied())
    .filter(|(_id, position)| position.x >= far_wall_inner_x - FAR_SAMPLE_WIDTH)
    .collect::<Vec<_>>();

// After half-period batches of advance(4):
// one of those ids has position.y > start_y + (2.0 * PARTICLE_RADIUS)
// and live_particle_count is unchanged.
```

Step 0, before any `advance`: every `velocities()` entry is `Vec2::ZERO`, the sample is non-empty, and the maximum `y` in the far window is within one diameter of the pool-wide maximum `y`. That is the still band. The passing motion is upward, on an id that began in the far window. `read_particles` is `cfg(test)`. [VERIFIED: `particle/view.rs`; `ParticleGroupRecipe::new` zero velocity]

### Recommended starting numbers

These are a starting point inside Claude's discretion, not locked geometry. Change them until the far-wall test passes. Record any radius or count shrink as a playground adaptation (D-06).

| Parameter | Start | Constraint |
|-----------|-------|------------|
| Radius | `0.025` | Same order as Wave Machine and Hydraulic Fountain. Stride is `0.0375`. |
| Damping | `0.2` | Same strength Wave Machine uses so a free surface persists. Not a shared constant, and not the `1.0` default. |
| Depth | `0.32` m | Several strides of water. `sqrt(10 * 0.32)` is about `1.79` m/s. |
| Platform width | `0.50` m | Short end. The rest of the floor is static. |
| Channel past the step | `0.90` m | Travel time about `0.5` s, inside a `1.0` s half period. |
| Stroke | `0.16` m | Several diameters, under half the depth. |
| Period | `2` s | `60` steps per half period, `15` calls of `advance(4)`. |
| Peak speed | `stroke * TAU / (2 * period)` ≈ `0.251` m/s | Integral peaks at `stroke`. Step size about `0.004` m. |
| Limit margin | `0.02` m | Outside the peak. Range stays above `2 * LINEAR_SLOP`. |
| Max motor force | `1.0e6` | Same order as the hydraulic piston. |
| Gravity | `(0, -10)` | Same as the other basins. |
| Far sample | last `0.20` m before the far wall | Over the fixed floor, not over the slab. |
| Upward floor | one diameter, `0.05` m | Above compaction. Do not weaken this to make a small wave pass. |

[ASSUMED] These particular meters raise the far surface by at least one diameter within one half period. The inequalities (dynamic body, vertical axis, sine of sim time, limits outside the peak, peak speed under one diameter per step, upward id check, stable count) are verified from the current solver and session.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Wave Machine rocks a whole tank with a revolute cosine | This scene leaves that demo alone and lifts one end | D-01, D-03 | Do not retune `wave-machine` |
| Hydraulic Fountain square-wave piston | Same setter, sine of elapsed time, vertical axis | This phase | Reuse the hook, not the schedule |
| Kinematic scripted platforms | Motor speed written before `World::step` | Wave Machine and Hydraulic Fountain | Reuse that hook |

**Deprecated/outdated:**

- Do not treat the Phase 31 roadmap stub ("To be planned") as scope. `31-CONTEXT.md` is the scope. [VERIFIED: `31-CONTEXT.md` canonical refs]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Stroke `0.16` m, peak speed about `0.25` m/s, period `2` s, channel `0.90` m, depth `0.32` m, radius `0.025`, and damping `0.2` raise the far-wall surface by at least one diameter within one half period | Recommended starting numbers | The native test fails. Shorten the channel or raise the stroke. Do not add an engine feature, cut the count, or raise the 4-step cap. |
| A2 | A slab plus a vertical face on one dynamic body seals the step well enough that the pool stays one group | Pattern 1 | If particles fall through the joint, thicken the face overlap. Do not switch to a kinematic pose write. |
| A3 | One diameter of upward motion is above hydrostatic settle for a zero-velocity fill placed above the floor | Far-wall assertion | If a no-motor control run also rises by one diameter, raise the threshold slightly or lower the fill onto a settled lattice at build time. Do not count downward motion as the wave. |

## Open Questions (RESOLVED)

1. **Does the first half cycle reach the far wall?** — RESOLVED
   - What we know: Travel time at the starting depth and channel is about `0.5` s, and the half period is `1` s. The proof is the id test, not the shallow-water estimate.
   - Resolution: Plan 01 owns the retune. If the platform stalls, raise max force and keep the sine amplitude. If the far surface does not rise by one diameter, shorten the channel or raise the stroke and keep the period a multiple of `4/60` s. Do not add an engine feature.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Native far-wall test and WASM build | ✓ | 1.97.0 | — |
| just | `just web-player-smoke` | ✓ | 1.48.0 | Run `bun scripts/web-build.ts player-smoke` directly |
| bun | Player smoke script | ✓ | 1.4.2 | — |
| node | Tooling beside bun | ✓ | v24.13.0 | — |
| Playwright Chromium | D-10 | Installed by the smoke script | — | No second browser |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none. Chromium comes from the existing smoke recipe. Do not add Firefox or Safari.

Step 2.6 covers those tools because the phase proof is a local Rust test plus the existing Chromium script. No new service is required.

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
| Particle destroy used as a hidden wave | Tampering | The far-wall test requires original `ParticleId`s to move up and the live count to stay constant. |

`workflow.nyquist_validation` is `false` in `.planning/config.json`. Test commands live in the plan grain below rather than a separate validation section.

## Suggested Plan Grain

Three small waves. Do not write `PLAN.md` from this note. Do not invent `PLAY-03`.

### Wave 1: Scene and far-wall proof

Files:

- `crates/liquidfun-wasm/src/scene/wave_tank.rs` (new)
- `crates/liquidfun-wasm/src/scene/wave_tank/tests.rs` (new)
- `crates/liquidfun-wasm/src/scene.rs` (`mod`, `SceneId::WaveTank`, `parse_scene_id`, `build_scene`)
- `crates/liquidfun-wasm/src/session/tests.rs` (token list)

Tests, one concern each, arrange / act / assert:

- Step 0 is a still band: nonzero count, every velocity is zero, far-window surface `y` matches the pool surface within one diameter, far window is over the fixed floor
- After `PERIOD * 30` steps in `advance(4)` batches, platform translation is at least half the stroke, live count is unchanged, and at least one original far-window id has `y` greater than its start `y` by one diameter
- Motor speed after one step matches `peak * sin(TAU * dt / period)`, and after the half period it is near zero
- The only dynamic body is the platform. The only prismatic joint uses an upward axis. Rebuild restores translation `0` and the still layout
- Unknown control, unknown action, pointer no-op that does not change the live count
- `parse_scene_id("wave-tank")` succeeds
- Optional `include_str` of `wave_tank.rs`: calls `set_prismatic_motor_speed`, does not call `set_revolute_motor_speed`, `create_particle_with_def`, or `with_destruction_by_age`

Leave `MAX_ADVANCE_STEPS` at 4. Do not edit `wave_machine.rs` or `hydraulic_fountain.rs`.

Quick command: `cargo test -p liquidfun-wasm scene::wave_tank`

### Wave 2: Catalog companions

Files, all append-only after the hydraulic-fountain entries:

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
- `web/tests/portrait-bounds.test.ts` (`MIN_HEIGHT_FRACTION` plus wall endpoints, including platform rest and crest)
- `web/tests/demo-media-model.test.ts` (the last id is no longer `hydraulic-fountain`)

`README_SVG_PLANS` and `SCENE_CAPTURE_PLANS` must follow `SCENE_IDS` order. Portrait walls include the near wall, the fixed floor, the far wall, and the platform at rest and at the crest, inside the phone rectangle. Landscape keeps `viewBounds` on the scene record. The preview SVG shows the end platform above the resting floor. The caption is already `Static preview`.

Do not add a pointer or labeled-control entry in `player.spec.ts` `POINTER_CONTROL`. Do not add a wave-speed README cue.

Quick command once the maps exist: `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts tests/scene-catalog-credits.test.ts tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts`

### Wave 3: Browser proof

Run `just web-player-smoke` once the maps exist. The watch-first test is derived: `WATCH_FIRST_SCENE_IDS` is every `SCENE_IDS` entry absent from `POINTER_CONTROL`. It then opens `#/scene/wave-tank`, pauses, plays, and resets, and still walks the scenes that were already in the catalog. Leave `ALL_SCENE_TIMEOUT_MS` at `380000` unless this run exceeds it. Phase 30 left that timeout after a locator fix, and the passing Chromium run finished in 42.6 seconds. Do not rasterize README assets in this wave.

Independent review is after implementation. The implementing agent records evidence and does not approve its own work.

## Sources

### Primary (HIGH confidence)

- `crates/liquidfun/src/world/joint/prismatic.rs` — `set_prismatic_motor_speed`, `prismatic_joint_translation`, `solve_motor`, `classify_limit`
- `crates/liquidfun/src/joint/definition/revolute_prismatic.rs` — `with_limits`, `with_motor`, default `collide_connected: false`
- `crates/liquidfun/src/world/body/mass.rs` — kinematic and static mass start at zero
- `crates/liquidfun/src/math/settings.rs` — `LINEAR_SLOP` `0.005`, `PARTICLE_STRIDE` `0.75`
- `crates/liquidfun-wasm/src/session.rs` — `on_advance` then step, `MAX_ADVANCE_STEPS = 4`, `read_particles`
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` — sim-time prismatic write, dynamic body, test split
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs` — translation and motor-speed readback through `world_observation`
- `crates/liquidfun-wasm/src/scene/wave_machine.rs` — revolute scene that must stay untouched, damping `0.2`
- `crates/liquidfun-wasm/src/scene/wave_machine/drive.rs` — cosine speed to not copy
- `crates/liquidfun/src/particle/group/recipe.rs` — default `ParticleFlags::WATER` and zero velocity
- `crates/liquidfun/src/particle/view.rs` — stable `particle_ids` aligned with `positions` and `velocities`
- `crates/liquidfun/src/particle/definition/system_definition.rs` — default damping `1.0`
- `crates/liquidfun/src/particle/solver/pressure.rs` — damping applied to approaching contacts
- `crates/liquidfun-wasm/src/session/escape.rs` — playground particle deletion
- `web/src/catalog/scenes.ts`, `scene-records.ts`, `portrait-bounds.ts`, `previews.tsx`, `links.ts`
- `web/src/render/camera.ts` — `VIEWPORT_INSET` 16
- `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`
- `web/e2e/player.spec.ts`, `web/e2e/player-helpers.ts`
- `web/tests/portrait-bounds.test.ts`
- `.planning/phases/31-sinusoidal-wave-tank/31-CONTEXT.md` — locked scope
- `.planning/REQUIREMENTS.md` — `PLAY-03` already complete

### Secondary (MEDIUM confidence)

- `crates/liquidfun/src/world/particle_object/group.rs` — sample stride is `PARTICLE_STRIDE * diameter`
- Phase 30 research and `.planning/STATE.md` — file split, portrait inset lesson, smoke timeout left at 380000
- Shallow-water travel time `length / sqrt(g * depth)` — sizing hint only; the id test is the proof

### Tertiary (LOW confidence)

- None beyond A1, A2, and A3 in the assumptions log. Those numbers are a starting geometry, not a measured run.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new libraries. The joint, hook, and catalog seams are in tree.
- Architecture: HIGH — dynamic vertical prismatic motor plus a pre-step sine write matches the locked decisions and the current solver.
- Pitfalls: HIGH for kinematic mass, limit clipping, far-sample identity, and catalog coverage. MEDIUM for the exact pool numbers, which the far-wall test has to confirm.

**Research date:** 2026-09-27
**Valid until:** 2026-10-27

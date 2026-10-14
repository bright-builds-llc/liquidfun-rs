# Technology Stack: v1.6 Physics Curiosity Scenes

**Project:** liquidfun-rs, milestone v1.6 (six new playground scenes)
**Researched:** 2026-10-14
**Scope:** Only the engine, bridge and web capabilities the six new scenes need. The existing stack (Rust 1.97, wasm-bindgen 0.2.128, SolidJS, renderers, catalog rules) is validated and not re-researched.
**Overall confidence:** HIGH that the APIs exist (read from code). MEDIUM to LOW that each phenomenon will look right; that is tuning risk, rated per scene.

## Bottom Line

1. **No engine change is required for any of the six scenes.** Every mechanism can be built in the private `liquidfun-wasm` bridge from existing public `World` APIs: per-particle and range forces, flags, groups, kinematic and motorized bodies, fixture create/destroy, mass reset, and gravity. This keeps `crates/liquidfun` untouched, the same policy that held all of v1.5, so the 25 scene fingerprints are safe by construction.
2. **One optional additive engine API, only if the hourglass spike needs it:** `World::set_particle_flags(particle, flags)`. It mirrors upstream `b2ParticleSystem::SetParticleFlags` (`third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h:496`) over the existing crate-private `set_particle_flags_internal` (`crates/liquidfun/src/particle/storage/group.rs:350-363`). No existing scene calls it, so it is default-off. Existing APIs already give a fallback (destroy and recreate a particle with `WALL`).
3. **Do not add** per-particle or per-group density/mass, a second particle system per scene, cross-system particle interaction, a gas/air model, or a magnetic-field solver. Reasons are under What NOT to Add.
4. **No new Cargo or npm dependencies.** Deterministic perturbations and fields use plain `f32` maths already used in the bridge (for example `scene/wave_tank.rs:215`, `scene/soup_stirrer.rs:207`).

## Verified Engine Capabilities (file:line)

| Capability | Where | What it means for v1.6 | Confidence |
| --- | --- | --- | --- |
| Per-particle force | `World::apply_particle_force` `crates/liquidfun/src/world/particle_object/system.rs:359-366` | Non-uniform force fields (ferrofluid, siphon fallback, grain friction) | HIGH |
| Range/group force, **divided by count** | `apply_particle_force_range` `system.rs:377-391`; `force/count` at `particle/force.rs:79-80`; contiguous-range check `force.rs:143-145` | One call per step for a whole group (Rayleigh–Taylor heavy layer). Multiply by member count, as Impulse does (`liquidfun-wasm/src/scene/impulse.rs:186-193`) | HIGH |
| Forces reject `WALL` particles | `force.rs:146-148` | Skip frozen grains if the hourglass freezes them as `WALL` | HIGH |
| Force applied once per world step, using the substep dt, then cleared | `particle/solver/preparation.rs:54-87`; substep dt = dt / iterations at `world/particle_coupling/executor.rs:58-71` | **Pitfall:** to add acceleration `a` over a whole step, use `F = m·a·particle_iterations`, with `m = density·(PARTICLE_STRIDE·2r)²` | HIGH |
| Force pass is skipped when nothing is pending | `preparation.rs:59-61` (`has_pending_system_force`) | Scenes that never apply force are bit-identical | HIGH |
| Particle flags (upstream bit values) | `particle/definition.rs:18-55` | `POWDER`, `VISCOUS`, `TENSILE`, `REPULSIVE`, `WALL`, `COLOR_MIXING`, `BARRIER`, `STATIC_PRESSURE` | HIGH |
| Density and gravity scale are **per system only** | `particle/definition/system_definition.rs:16-17`; upstream mass from system density `b2ParticleSystem.h:1358-1369` | No per-particle or per-group mass exists upstream or here | HIGH |
| Systems never interact with each other | Per-system executor loop `world/particle_coupling.rs:50-56`; single-system record `executor.rs:25-34, 73-77`; upstream per-system `FindContacts` `b2ParticleSystem.cpp:1984` | Two systems with different densities would pass through each other; no RT | HIGH |
| Pressure is never negative (no suction) | `particle/solver/pressure.rs:86` (`max(0.0, weight − MIN_PARTICLE_WEIGHT)`), upstream `b2ParticleSystem.cpp:3204-3205` | Plain water cannot pull a siphon column over a crest | HIGH |
| `TENSILE` replaces pressure with a two-sided term | Pressure zeroed for `POWDER`/`TENSILE` `pressure.rs:93-101`; tensile `(combined_weight − 2.0)` term `material.rs:165` | Tensile fluid has capped cohesion (negative pressure), which is the honest siphon mechanism | HIGH (code); MEDIUM (enough for a siphon) |
| `REPULSIVE` acts only between different groups | `material.rs:87` | Optional interface repulsion between the two RT liquids | HIGH |
| `POWDER` only adds repulsion | `material.rs:99-119`; Phase 40 probe: powder drains, spreads and ramps like water, does not pile, and `powder_strength` 1.0 changes nothing (`.planning/phases/40-timed-valve-and-teaching-levels/40-RESEARCH.md:270-283`, `40-08-SUMMARY.md`) | **The hourglass cannot claim an angle of repose from `POWDER` alone** | HIGH |
| Viscous strength above about 2.0 fails the step | `40-RESEARCH.md` Pitfall 3 | Keep `with_viscous_strength` at or below 1.5 | HIGH (measured) |
| Read lanes: ids, positions, velocities, flags, group ids, weights | `particle/view.rs:50-125` | Fields, surface detection (low weight) and "settled" tests all come from one view | HIGH |
| Group membership | `particle_group_view(...).member_ids()` (used at `impulse.rs:180-184`) | Range force for one liquid | HIGH |
| Create group from explicit positions | `ParticleGroupSource::positions` `particle/group/source.rs:85-97, 140`; `create_particle_group` `world/particle_object/group.rs:22-29`; `destroy_particle_group` `world/object/lifecycle.rs:250-257` | Instant hourglass flip by rebuilding the sand at rotated positions | HIGH |
| Kinematic bodies | `set_body_linear_velocity` / `set_body_angular_velocity` `world/object/body_object.rs:123, 139`; `set_body_transform` `:446`; precedent: kinematic plate `scene/hydraulic_fountain.rs:238-246`, body recreate `:485-495` | Archimedes vane track, optional animated hourglass rotation | HIGH |
| Motorized revolute body with liquid inside | `scene/washing_machine.rs:85, 145` | Proven alternative for an animated hourglass flip | HIGH |
| Fixture swap and mass | `create_fixture` (mass only when density > 0) `world/object/fixture_object.rs:17-48`; `destroy_fixture` recomputes mass `lifecycle.rs:146-163`; `set_fixture_density` `fixture_object.rs:271`; `reset_body_mass_data` `fixture_object.rs:237` | Cartesian diver air pocket | HIGH |
| Density-driven buoyancy works | Float or Sink particle density 1.0 vs body 0.3/0.6/2.0 (`scene/float_or_sink.rs:26-28, 147`); v1.5 `float-up` level | Diver floats and sinks by changing displaced volume or density | HIGH (works); MEDIUM (neutral threshold needs tuning) |
| Gravity getter and setter | `world/config.rs:273, 285`; session tilt `liquidfun-wasm/src/session.rs:196-216` | RT extra weight should follow `world.gravity()` so tilt and the gravity slider stay consistent | HIGH |

## Bridge Facts That Constrain the Design

| Fact | Where | Consequence |
| --- | --- | --- |
| One particle system per scene | `BuiltScene.particle_system` `liquidfun-wasm/src/scene.rs:147-152`; frame capture `session.rs:381-395`; fingerprint `scene_spot/fingerprint.rs:50` | Two-system designs would also need frame, fingerprint and renderer changes. One more reason to use one system |
| Every particle needs a color | `session.rs:388-392` (missing color lane fails the frame) | Color every group (also needed for the RT and ferrofluid looks) |
| `on_advance` runs before every fixed step | `session.rs:228-229` | Pointer events only store state in hooks; force fields are applied in `on_advance` |
| Per-scene substeps | `scene.rs:104-115` | Add new match arms for the hourglass neck and siphon tube if they leak. Existing arms stay unchanged |
| Frame caps | `MAX_FRAME_PARTICLES` 16,384 `session.rs:26`; 512 segments and 48 circles `frame.rs:9-10` | Curved tubes and screw decoration must fit in 512 segments |
| Segments are drawn by the scene | e.g. `hydraulic_fountain.rs:418-430` | Decorative helix, bottle dent and bubble outline need no renderer change |
| Survey cues are only Default, Action and PointerUp; the array is fixed at 25 | `scene_spot.rs:27-34, 46` | The array becomes 31. Hold-to-act scenes need a default autopilot or an Action cue |
| README cues are only `action` and `pointer-up` | `web/scripts/readme-svg/plans.ts` | Same as above. Prefer scene-side defaults over a new cue kind |
| No hover: `move` is sent only during a captured press | `web/src/input/pointer.ts:111-124` | Ferrofluid is press-and-drag on desktop and touch, not hover |
| Canvas is near-black `#071018` | `web/src/render/canvas.ts:30` | A "black" ferrofluid would be invisible. Use a dark graphite or violet with highlights |

## Per-Scene Stack Decisions

### 1. Sand hourglass: existing APIs; angle of repose is HIGH risk

- **Pour:** one system, one sand group (`POWDER`, optionally `VISCOUS` at 1.0 or less), a static point-symmetric glass of thick polygon walls, and a 4 to 6 particle-diameter neck. Watch-first. Pour risk is LOW.
- **Flip (recommended): instant point reflection.** The glass is symmetric under a 180° turn, so the geometry stays put. Read positions from the view, `destroy_particle_group`, then `create_particle_group` with `ParticleGroupSource::positions` at `2c − p`. This is one batched rebuild with no per-particle `set_particle_position` calls; each of those rebuilds proxies synchronously (`system.rs:269-283`), which would be O(n²). Expose it as a "Flip" action plus a canvas tap. **Stretch:** an animated turn with a kinematic body or Washing Machine-style motor (MEDIUM risk: sand pressing on moving thin walls).
- **Angle of repose:** the engine has no inter-particle friction, and the measured `POWDER` behaves like water. Spike these in order:
  1. Bridge-only grain friction: for slow, packed powder particles (high weight in the view), apply a force that removes up to `μ·|g|·dt` of velocity each step. No engine change.
  1. Freeze settled grains as `WALL` and unfreeze them all on flip. Use the existing destroy plus `create_particle_with_def(WALL)` (`world/particle_object/particle.rs:24-29`), or add `World::set_particle_flags` (upstream parity). Exclude frozen grains from any force call (`force.rs:146-148`).
  1. If neither looks honest, caption it as "sand pours through the neck" and **never claim piling** (Phase 40 D-25 precedent).
- **Risk:** pour and flip LOW; angle of repose HIGH.

### 2. Archimedes screw: existing APIs, MEDIUM risk

- A 2D side view of a screw is a train of inclined vanes moving up a tilted trough. One **kinematic** vane body spans the trough width and moves along the axis with `set_body_linear_velocity`. After one pitch it snaps back exactly one pitch with `set_body_transform`, so the lattice looks unchanged.
- **Integration rule:** both ends of the vane track must end inside sealed housings that particles cannot reach, so the wrap never moves a vane through water.
- Kinematic vanes overlapping static trough walls is fine (they do not collide with each other). The vanes must close the trough gap to under one particle diameter, or the pockets leak.
- Water or powder both work. Draw the helix as decorative segments. Live speed control through a preset, like Wave Tank.
- **Risk:** MEDIUM (leakage, wrap timing, particles caught between vane and housing).

### 3. Siphon: existing APIs, MEDIUM-HIGH risk

- Plain water cannot pull a column over a crest because its pressure is never negative (`pressure.rs:86`).
- **Honest mechanism:** `TENSILE | VISCOUS` particles. Liquid Timer already uses that pair (`scene/liquid_timer.rs:138`), and tensile pressure has a two-sided `(combined_weight − 2)` term (`material.rs:165`). This mirrors the published "gravity plus cohesion" explanation of siphons, including its height limit.
- **Design:** start primed (tube full), crest only slightly above the upper tank's surface, outlet submerged in the lower tank so flow stops when levels equalize, tube 4 to 6 diameters wide made from static polygon or chain walls, and more substeps if needed.
- **Fallback (bridge-only):** an "atmosphere" push. Apply a downward force to free-surface particles (low weight in the view) inside the two tank regions only.
- **Risk:** MEDIUM-HIGH. Spike before authoring geometry.

### 4. Cartesian diver: existing APIs, MEDIUM risk

- The diver is a dynamic body with an open-bottom hull and an air-pocket fixture of density 0 filling the cup. Pressing shrinks the pocket (Boyle-style, rate-limited per step) by `destroy_fixture` plus `create_fixture`, so water visibly enters the diver and displacement drops. Mass is unchanged because the pocket has density 0 (`fixture_object.rs:23-48`). Releasing grows it back slowly so particles are not ejected violently.
- The engine has no air and the water is nearly incompressible, so the squeeze drives the pocket size directly. The bottle dent is decorative segments only. **Do not** push a kinematic wall into a full bottle: packed-fluid pressure can blow up.
- **Simpler fallback:** `set_fixture_density` plus `reset_body_mass_data` on press.
- Input: pointer down/up, plus a keyboard-accessible "Squeeze" toggle action, which also serves as the README and survey `action` cue.
- **Risk:** MEDIUM (neutral buoyancy in LiquidFun is approximate, and fixture swaps inside fluid need gentle rates).

### 5. Rayleigh–Taylor instability: existing APIs, HIGH risk

- **One system, two groups:** heavy (top, warm color) and light (bottom, blue). The heavy layer gets extra weight through **one** `apply_particle_force_range` call per step on its contiguous group members: `F = m·(ρ_h/ρ_l − 1)·|g|·iterations·count`, directed along `world.gravity()`. Same mass everywhere, more weight on top, so buoyancy-driven overturn emerges from the shared pressure solver.
- Seed fingers with a deterministic sinusoidal interface or a small inline hash (no RNG crate). Optionally add `REPULSIVE` on both groups for an interface (`material.rs:87`) and `VISCOUS` at 1.5 or less for coherent fingers. Avoid `TENSILE` in the bulk, because it zeroes normal pressure.
- **Main visual risk:** heavy particles may percolate one by one ("rain") instead of forming fingers. A spike should sweep density ratio, viscosity, repulsion, perturbation and resolution.
- **Risk:** HIGH. Schedule it last so it can slip.

### 6. Fake ferrofluid: existing APIs, MEDIUM risk

- One `TENSILE | VISCOUS` group in a shallow dish. In `on_advance`, read ids and positions from the view, compute a field toward the magnet (inverse-distance falloff, clamped), and apply it with per-particle `apply_particle_force`. Modulate it by `1 + a·cos(k·(x − x_mag))` so a row of spikes forms instead of one mound.
- Cost: about two small allocations per call. Roughly 1–2k calls per step is small next to the step itself; add a batch API only if profiling says so.
- The magnet follows a press-and-drag (no hover). When no pointer is down, a default magnet with an on/off control keeps it watch-first, which also makes the README and survey work with no new cue kinds.
- Use a dark graphite or violet color, not black.
- **Risk:** MEDIUM (the spike look is tuning; the mechanics are trivial).

## Integration Points (all additive)

| Layer | Additions | Must not change |
| --- | --- | --- |
| `crates/liquidfun` | Nothing. Optionally `World::set_particle_flags` if the hourglass freeze spike needs it | Any solver, pass order or existing API behavior |
| `liquidfun-wasm/src/scene.rs` | 6 `SceneId` variants, `parse_scene_id`, `build_scene`, and `particle_iterations` arms | Existing arms, `DEFAULT_PARTICLE_ITERATIONS`, `attach_basin_fixture`, gravity preset handling |
| `liquidfun-wasm/src/scene/` | 6 new modules. Optionally one small shared helper for "read view, compute per-particle forces, apply", used only by new scenes | Existing scene modules |
| `scene_spot.rs` | `SURVEY_SCENES` 25 → 31 with Default or Action cues | Existing entries and the fingerprint algorithm |
| `web/src/catalog/` | `scenes.ts`, `scene-records.ts`, `portrait-bounds.ts`, `previews.tsx` | Existing records |
| `web/scripts/readme-svg/plans.ts` | 6 plans (Default, or `action` cues for flip and squeeze) | Cue kinds (no new kind needed) |
| Web input and render | Nothing new: existing pointer down/move/up, actions, presets, segments and circles, per-particle colors | Renderer and pointer reducer |

## Alternatives Considered

| Need | Recommended | Alternative | Why not |
| --- | --- | --- | --- |
| RT two densities | One system, extra weight by range force | Two systems with different `density` | Systems never interact (`particle_coupling.rs:50-56`, upstream `FindContacts`); also breaks the single-system frame and fingerprint |
| RT two densities | Bridge force | Engine per-particle mass lane | Not in upstream; would touch every inverse-mass use; large fingerprint and parity risk |
| Ferrofluid field | Per-particle `apply_particle_force` | New engine "force field" API | Bridge maths is enough; the engine stays upstream-shaped |
| Hourglass flip | Point-reflection rebuild | Gravity flip with `set_gravity(0, +g)` | Sand would visibly fall up; the camera cannot rotate (scale legend and framing rules) |
| Hourglass flip | Group rebuild | Per-particle `set_particle_position` | Each call rebuilds proxies synchronously, O(n²) |
| Diver | Pocket fixture swap | Squeezing walls into a full bottle | Packed-fluid pressure explodes; there is no compressible air to model |
| Siphon | `TENSILE` cohesion | Plain water | Pressure is never negative; the column breaks |
| Screw | Kinematic vane train | Rotating helix body | A rotating 2D shape does not convey material along an axis |

## What NOT to Add

| Avoid | Why | Instead |
| --- | --- | --- |
| Engine per-particle or per-group density, mass or gravity scale | Not an upstream concept; changes core solver inputs | Bridge range or per-particle forces |
| Multiple particle systems in one scene | No cross-system contacts; frame, fingerprint and renderer assume one system | Groups plus flags in one system |
| Engine air, gas or magnetic models | Out of LiquidFun's scope; fake behavior belongs in the bridge with honest captions | Fixture swap and bridge force fields |
| Hover pointer events | Touch has none; would change the shared reducer | Press-and-drag plus default autopilot |
| New README or survey cue kinds | Not needed | Default autopilot or `action` cues |
| An RNG crate | No dependency needed | Deterministic sinusoid or inline hash |
| Pure black ferrofluid | Invisible on `#071018` | Dark graphite or violet with highlights |
| Captions claiming piling, suction or compressed air | Phase 40 D-25 honesty precedent | Describe what the engine really does |

## Installation

No new packages. `Cargo.toml` (`liquidfun`: `bitflags`; `liquidfun-wasm`: `liquidfun`, `wasm-bindgen =0.2.128`) and `web/package.json` stay as they are.

## Sources

- Code read at HEAD `b151838be` (file:line above): `crates/liquidfun/src/world/particle_object/system.rs`, `particle/force.rs`, `particle/solver/{preparation,pressure,material}.rs`, `world/particle_coupling.rs`, `world/particle_coupling/executor.rs`, `particle/definition*.rs`, `particle/view.rs`, `particle/storage/group.rs`, `world/object/{body_object,fixture_object,lifecycle}.rs`, `world/config.rs`; `crates/liquidfun-wasm/src/{scene.rs,session.rs,frame.rs,scene_spot.rs,scene_spot/fingerprint.rs}` and scenes `impulse.rs`, `hydraulic_fountain.rs`, `washing_machine.rs`, `float_or_sink.rs`, `liquid_timer.rs`; `web/src/input/{pointer,canvas-pointer}.ts`, `web/src/render/canvas.ts`, `web/scripts/readme-svg/plans.ts`. HIGH.
- Pinned upstream `7f20402173fd143a3988c921bc384459c6a858f2`: `b2ParticleSystem.h:496, 1358-1369`; `b2ParticleSystem.cpp:1984, 3200-3205`. HIGH.
- Phase 40 measurements: `.planning/phases/40-timed-valve-and-teaching-levels/40-RESEARCH.md` (Pitfalls 2–3) and `40-08-SUMMARY.md`. HIGH (measured).
- Siphon cohesion framing: [The height limit of a siphon (Sci Rep 2015)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4667279/) and its [corrigendum](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5411809/). The cohesion interpretation is still debated ([open capillary siphons, JFM](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/open-capillary-siphons/C406E1B88F33E7B281BB8B1425B5454E)), so captions should say "cohesion" carefully. MEDIUM.

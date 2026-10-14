# Pitfalls Research

**Domain:** Adding six "physics curiosity" scenes (Siphon, Archimedes screw, Sand hourglass, Cartesian diver, Rayleigh–Taylor instability, Fake ferrofluid) to the liquidfun-rs WASM/SolidJS playground, which already has 25 scenes with pinned end-state fingerprints
**Researched:** 2026-10-14
**Confidence:** HIGH for solver facts and repo integration points (read from source at HEAD `b151838be`). HIGH for powder behavior (measured in v1.5 Phase 40). MEDIUM for the proposed physics workarounds: they are reasoned from solver code but not yet measured. LOW for phone/wasm step-cost estimates.

Phase labels are topics; numbering starts at 45. Suggested order follows "riskiest last":

- **P-Prep**: catalog-count plumbing, fingerprint baseline, early feasibility spikes for Siphon and Hourglass
- **P-Diver**: Cartesian diver (lowest risk)
- **P-Screw**: Archimedes screw
- **P-Ferro**: fake ferrofluid
- **P-RT**: Rayleigh–Taylor
- **P-Siphon**: siphon (high risk: the engine has no tension)
- **P-Hourglass**: sand hourglass (highest risk: powder does not pile, and rigid grains do not fit the frame protocol)

---

## Engine facts that drive the scene pitfalls

These come from the code. Every pitfall below depends on them.

| Fact | Evidence | Consequence |
| --- | --- | --- |
| Particle pressure is never negative. Particles can push each other but never pull. | `crates/liquidfun/src/particle/solver/pressure.rs:86` (`max(0.0, weight - MIN_PARTICLE_WEIGHT)`), `:63` (static pressure clamped to `[0, max]`) | A real siphon needs the fluid to hold together under tension, so plain water cannot siphon. |
| `POWDER` and `TENSILE` particles contribute **zero** weight pressure. Powder only adds a short-range repulsion. | `pressure.rs:93-98`; `material.rs:99-119` | Powder flows like water and never piles. This was measured in v1.5 (D-25). |
| Particles have no friction with each other. The only shear coupling is `VISCOUS`, which damps the **full** relative velocity, including separation. | `material.rs:28-69` (`relative_velocity = velocities[b] - velocities[a]`, line 61) | There is no angle of repose. Viscosity is the only built-in thing that resists a column pulling apart. |
| Particle mass and density belong to the **system**, not the group. Particle–particle pressure impulses are symmetric and ignore mass. | `pressure.rs:118-135`; `force.rs:101-103` (`definition.density()`) | Groups cannot have different densities without deep solver changes. |
| Each particle system is solved separately. Contacts come from that system's own storage. | `world/particle_coupling.rs:52-62`; `world/particle_coupling/executor.rs:143` | Two systems pass through each other like ghosts. Two systems are not a route to Rayleigh–Taylor. |
| A scene owns exactly one particle system. Frame capture and fingerprints read only that system. | `crates/liquidfun-wasm/src/scene.rs:147-152` (`BuiltScene.particle_system`); `scene_spot/fingerprint.rs:50-60` | A second system would be invisible and unfingerprinted unless the bridge changes. |
| `REPULSIVE` pushes apart contacts whose particles are in **different groups**. | `material.rs:73-97` (line 87) | This is a possible way to keep two fluids apart (RT). It may also leave a gap. Spike it. |
| `set_fixture_density` does **not** recompute mass. You must call `reset_body_mass_data`. | `world/object/fixture_object.rs:238, 265-271` | A diver whose density was "changed" may not change at all. |
| Per-particle force is available (`apply_particle_force`), and so is a contiguous-range force that is **divided** across the range. | `world/particle_object/system.rs:359-391`; `particle/force.rs:72-91` (`force / count`) | Group forces must be scaled by member count. `impulse.rs:186-193` already does this. |
| Changing particle-group flags clones the whole particle system. | `world/particle_object/group_mutation.rs:242` | Changing group flags every step costs O(N) copies. |
| Viscous strength above about 2.0 fails the step with `StepError::ParticleLifecycleInvariant`. | v1.5 `40-RESEARCH.md` Pitfall 3 (measured: 2.5, 3.0 and 4.0 fail; 1.0–2.0 pass) | This limits any tuning that leans on viscosity (siphon, RT, hourglass). |
| `damping` 4 produced runaway particles (the top one reached 11.4 m). | `40-RESEARCH.md` Pitfall 2 | Heavy damping is not a safe way to make sand settle. |
| LiquidFun water compresses with depth. A density-2.0 ball hovers at 3 m depth. | `40-09-SUMMARY.md` static pool table | Bodies near neutral buoyancy settle at a depth instead of sinking (diver). |
| Release native and wasm are bit-identical for static and circle scenes. Motorized linkages diverge (Theo Jansen at step 70). Debug diverges from release at steps 2–91. | `37-01-SUMMARY.md` spike table | Scene tests on rotating bodies must assert outcomes with tolerance and run in release, not debug. |

---

## Critical Pitfalls

### Pitfall 1: The siphon never starts, or it stalls when the source level drops below the crest

**What goes wrong:**
Water rises over the crest only while the source surface is above it. That is ordinary overflow. Once the source level falls below the crest, the column breaks at the top. The down leg falls away as a free jet, the up leg drains back, and the levels never equalize. If the tube is narrow, the flow also chokes or jams.

**Why it happens:**

- A real siphon works because ambient pressure (or cohesion) holds the column together while the crest sits at negative gauge pressure. This engine has no ambient pressure and clamps pressure at zero (`pressure.rs:86`). Any point above both free surfaces would need negative absolute pressure, which the solver cannot represent.
- A tube only 2–3 particle diameters wide carries flow as single-file particles. Pressure transfer becomes noisy, and the column breaks at bends.
- Thin edge or chain walls leak at seams and corners under pressure. v1.5 `float-up` found two such leaks ("window jet-through" and "roof seam", `40-09-SUMMARY.md:80`).

**How to avoid:**

- Treat "does a siphon work at all" as a **feasibility spike in P-Prep**, run long before the siphon phase. Use the D-25 style: a fixed geometry, numbers logged for each candidate, and a written pass rule. Candidates:
  1. Plain water, tube primed full at step 0. This is the control and should fail.
  1. `VISCOUS`, strength 1.0–1.5. Viscous coupling resists separation (`material.rs:61`), so a falling down leg may pull the crest along. Stay at or below 1.5.
  1. `TENSILE`. Its `(combined_weight - 2)` term attracts in sparse regions (`material.rs:150-160`), but it also zeroes weight pressure (`pressure.rs:96`). Measure both effects.
  1. "Atmosphere plates": weighted dynamic lids on both tank surfaces that stand in for air pressure. A lid weight equal to a head greater than (crest height − source level) keeps crest pressure positive. Lids must be thick, and their side gaps must stay under one particle diameter.
  1. A scene-level downward force on surface particles in both tanks (a fake atmosphere), applied in `on_advance`.
- Pass rule (proposed): net transfer continues for at least 2 s after the source surface is 0.3 m below the crest, and the final level difference is under 0.15 m. If no candidate passes, follow the D-25 precedent: either reword the scene honestly (for example "Siphon with air-pressure pistons") or drop it. Never call plain overflow a siphon.
- Make the tube at least 5 particle diameters wide. Build it from thick polygon walls at least 3 diameters thick, with joints that overlap. Avoid edge or chain tubes.
- Start primed (tube full at step 0), so the 10 s README clip shows the siphon running. It should not spend that time filling.

**Warning signs:**
Flow rate falls to zero while the source level is still above the outlet level. A visible gap opens at the crest. The down leg shows separated droplets. Particles appear outside the tube near bends.

**Phase to address:** P-Prep (spike and decision), P-Siphon (build)

---

### Pitfall 2: The hourglass "sand" flows like water and never forms a heap

**What goes wrong:**
The bottom bulb fills level like a liquid. There is no cone and no angle of repose. The neck flow rate falls as the top empties, as water does (Torricelli), instead of staying nearly constant as real sand does (Beverloo). The scene looks like a water clock. Liquid Timer already covers that.

**Why it happens:**
This was measured in v1.5. `POWDER` has no friction and no pressure (`pressure.rs:93-98`). It drains, spreads and runs down ramps within a few percent of water. Settled deposits were 0.28–0.34 m high and 7.1–7.2 m wide for both materials (`40-08-SUMMARY.md`, "No piling anywhere"). `powder_strength` 1.0 changed nothing, `POWDER | VISCOUS` behaved like powder, and `damping` 4 caused runaway particles. SEED-004 assumed "powder particles" would work. That assumption is already disproven.

**How to avoid:**

- Run a hourglass feasibility spike in **P-Prep** with measured pass rules: a deposit slope of at least 20° (from p95 height against base width) that stays within ±5° over 5 s, and a flow that does not jam. Candidates, cheapest first:
  1. **Freeze-to-wall sandpile.** In the scene hook, a slow particle that rests on frozen particles or on the floor is destroyed and re-created as a `WALL` particle at the same position. A heap grows, and its slope comes from the freeze rule. Flip unfreezes everything. This needs `destroy_particle` plus `create_particle` with flags. No public per-particle flag setter was found in `crates/liquidfun/src/world`. Measure the cost of creating particles every step.
  1. **Rigid grains** (small dynamic polygons with friction). These pile for real, but see Pitfall 9: the frame protocol caps rigid circles at 48 and segments at 512. They also jam when the neck is under about 5–6 grain widths.
  1. **Wall drag with `VISCOUS`.** Particle–body viscosity drags particles touching walls (`material.rs:47-57`). This might hold a thin layer on a slope, but not a heap.
- Do not add a friction solver flag to `crates/liquidfun` in this milestone. It would change shared solver paths, put the fingerprints at risk, and leave upstream semantics.
- If no candidate passes, apply D-25: drop the scene, or retitle it honestly. Never write "piles up" or "angle of repose" in a caption unless a measured slope backs it.

**Warning signs:**
The deposit's p95 height after it settles matches water in the same geometry. The top bulb's flow rate follows the head height.

**Phase to address:** P-Prep (spike and drop/reword decision), P-Hourglass (build). This is the riskiest scene, so schedule it last.

---

### Pitfall 3: Rayleigh–Taylor built on two particle systems or per-group density

**What goes wrong:**

- With two particle systems, the heavy fluid falls straight through the light fluid with no interaction. The second system is also not drawn and not fingerprinted.
- Adding per-group density to the engine touches the pressure, viscous, body-coupling and rigid solvers. It reorders floating-point work and changes the 25 fingerprints.

**Why it happens:**
Systems are solved one at a time from their own contacts (`particle_coupling.rs:52-62`, `executor.rs:143`). `BuiltScene` carries one system (`scene.rs:147-152`). Density is read from the system definition (`pressure.rs:118`, `force.rs:101`), and particle–particle impulses ignore mass (`pressure.rs:131-135`).

**How to avoid:**

- Use **one system and two groups**. Each step in `on_advance`, add an extra force along the current gravity direction to the heavy group only. Read the members each step (`particle_group_view(group).member_ids()`) and scale by member count, because range forces are divided across the range (`impulse.rs:186-193` is the pattern). This is the Boussinesq picture: equal inertia, different weight. Say so honestly ("the red layer is given extra weight").
- Read the gravity direction and size from `world.gravity()` each step. Tilt and the gravity slider then keep "heavy" pointing down.
- Make no change under `crates/liquidfun`.

**Warning signs:**
`create_particle_system` called twice in one scene. A diff touching `particle/solver/*`. A `density` field added to `ParticleGroupRecipe`.

**Phase to address:** P-RT. Add a short spike at the start of the phase.

---

### Pitfall 4: Rayleigh–Taylor mixes into noise instead of forming fingers

**What goes wrong:**
The heavy layer "rains" through the light one particle by particle, or turns over as a single blob, instead of growing distinct fingers or mushrooms. The two colors blur together, so the instability is not readable at a glance or in a 10 s README clip.

**Why it happens:**

- At 2–3k particles a finger is only a few particles wide. Without interfacial tension, particle-scale interpenetration dominates.
- A perfect lattice interface has no seeded mode, so the instability grows from lattice-settling noise, which looks random.
- Settling the lattice in the first steps sends a pressure wave that stirs the interface before the instability starts.
- Render-level blending also hurts. `COLOR_MIXING` blends colors in the solver (`material.rs:213`). The shaded-blob and soft-blob modes blend neighboring colors on screen. Density shading darkens packed regions (`web/src/render/density-shade.ts`: floor 0.73), which flattens contrast at the bottom.

**How to avoid:**

- Seed a deterministic interface perturbation: one or two sinusoidal modes in the initial heavy/light boundary. Use no RNG. Fixed constants keep scene-spot runs identical (`scene_spot.rs:323-333` fails if two runs disagree).
- Let the column settle about 0.5–1 s with the extra force off, then ramp it on.
- Spike `VISCOUS` on both groups (at most 1.5) to make fingers coherent, and `REPULSIVE` between groups (`material.rs:87`) for a sharp interface. Measure visible finger count by 3 s and the share of mixed particles.
- Never set `COLOR_MIXING`. Choose two colors that contrast with each other and with the canvas `#071018` (`web/src/render/canvas.ts:30`). Check them in shaded-blob, Water and circle-wireframe (README).
- The overturn finishes and then nothing moves. Provide an obvious restart (a "Flip" action that rebuilds the scene) and make sure the fingers appear in the first 1–4 s for the README clip.

**Warning signs:**
The heavy layer's mean height drops linearly with no structure. No contiguous heavy column wider than 4 particles exists at 2 s.

**Phase to address:** P-RT

---

### Pitfall 5: The ferrofluid force explodes, gains energy or depends on frame rate

**What goes wrong:**

- An attraction toward the pointer that grows without bound (1/r or 1/r²) pulls particles into a point. Pressure saturates at its cap, particles shoot out, they get evicted, and the "liquid" disappears.
- An undamped attraction is a potential well, so particles oscillate around the pointer forever. It looks like energy gain.
- A force applied per **frame** in TypeScript makes the scene stronger on 120 Hz displays than on phones that run 1–4 steps per frame.
- A plain radial pull makes a cone or blob, not spikes. The scene then does not look like ferrofluid.

**Why it happens:**
The step loop runs 1–4 steps per frame (`MAX_ADVANCE_STEPS = 4`, `session.rs:25`), and pointer events arrive between batches (`session.rs:349-366`). The pressure solver caps pressure (`pressure.rs:87`) but not external forces. Force units are newtons, and particle mass depends on radius and density (`force.rs:101-103`), so a force tuned at one radius is wrong at another.

**How to avoid:**

- Follow Color Mixer: store `maybe_pointer` on Down/Move, clear it on Up/Cancel (`color_mixer.rs:326-330`), and apply forces **per step** in `on_advance` (`color_mixer.rs:276-277`).
- Use a bounded, smooth profile: zero force inside a core radius, a smooth falloff to an outer radius, and a capped acceleration (for example at most 2–3× the gravity magnitude, read from `world.gravity()`). Convert acceleration to force with the computed particle mass, so the radius can change safely.
- Add explicit damping for affected particles (a velocity-proportional drag force). This turns the well into a settle-and-hold.
- Get spikes by design, not by luck. For example, aim the force at N fixed "spike tips" spaced on an arc below the pointer, or modulate the force by `cos(k·x)` along the surface. Give the liquid `TENSILE` or `VISCOUS` so spikes look cohesive. Name it honestly: "Fake ferrofluid".
- Test stability natively. Hold a scripted pointer at three positions for 60 s, then assert that max speed stays bounded, kinetic-energy proxies stay bounded, and no particle is evicted.

**Warning signs:**
`escape.rs` evictions while the pointer is held. Max particle speed rises over time while the pointer is still. Different behavior at 60 Hz and 120 Hz in `installControlledRefreshRate` e2e runs.

**Phase to address:** P-Ferro. Run a stability spike first.

---

### Pitfall 6: The Cartesian diver hovers in the middle, never responds, or behaves like a switch with no physics

**What goes wrong:**

- Pressing changes nothing because `set_fixture_density` was called without `reset_body_mass_data` (`fixture_object.rs:265-271`).
- The diver rests at the top, falls asleep, and ignores the density change.
- A "pressed" density around 1.5–2.0 makes the diver hover mid-bottle, because LiquidFun water compresses with depth. A density-2.0 ball hovers at 3 m (`40-09-SUMMARY.md` table).
- Squeezing the bottle by moving its walls compresses the particles. That creates jitter, or leaks at wall seams, and it does not change rigid-body buoyancy at all.
- A small diver (under about 6 particle diameters) gets knocked around by individual particle impacts. Buoyancy looks noisy.

**Why it happens:**
The real diver works because its air bubble compresses, so it becomes denser as it goes deeper. That is an unstable equilibrium. The engine's bodies are rigid, and its water is slightly compressible, which gives the opposite, stable equilibrium.

**How to avoid:**

- Fake the compressibility honestly. On press, raise the diver's density (`set_fixture_density` then `reset_body_mass_data`) or its gravity scale (`set_body_gravity_scale`, `body_object.rs:359`). Spike both. Use wide margins, for example about 0.5 released and 3 or more pressed, and verify full sinks and full rises at the scene's real water depth. A shallower bottle (1–1.5 m of water) reduces hovering.
- Keep the diver awake: disable sleep on its body, or wake it with an impulse after each change, as Float or Sink does (`float_or_sink.rs:29, 213`).
- Use a circle body, or a body whose collision does not depend on rotation (the v1.5 D-12 lesson: circles avoid the native/wasm `sin`/`cos` divergence).
- Make the squeeze visual only (bottle outline and bubble size in the renderer), or keep wall motion under one particle diameter per step (`hydraulic_fountain.rs:36-37`) with thick overlapping walls.
- Caption honestly: "the squeezed air bubble is shown by switching the diver's density".

**Warning signs:**
The diver's y stops between top and bottom for more than 1 s while pressed. `reset_body_mass_data` is missing from the diff. Body sleep is left at its default.

**Phase to address:** P-Diver

---

### Pitfall 7: The Archimedes screw leaks, jams, crushes particles or teleports blades into the fluid

**What goes wrong:**

- A screw is 3D. A 2D side view is blades sweeping up a tilted trough. Wrapping a blade back to the bottom with `set_body_transform` drops it inside the pool and blasts particles away.
- Gaps between blade and trough under one particle diameter crush particles between a kinematic blade (infinitely strong) and a static wall. Particles get ejected or squeezed through the wall. Gaps over 2–3 diameters let most of the water leak back.
- A dynamic screw on a revolute motor stalls under the fluid load if `max_motor_torque` is too low.
- Blades thinner than about 2 diameters, or moving more than one diameter per step, tunnel through particles.
- Lifted water collects at the top and the pool runs dry within 10–20 s. The scene then goes still.

**Why it happens:**
LiquidFun's particle–body collision handles static thin walls well, but rotating or teleported bodies are the weak spot. v1.5 noted that corners and chain gaps still leak. Hydraulic Fountain needed a compile-time speed cap and thick cheeks to stop leaks (`hydraulic_fountain.rs:1-6, 36-37`).

**How to avoid:**

- Draw the screw as kinematic paddles on a closed loop: up through the trough, then back down outside it, below the trough floor. They never teleport through fluid. Set each paddle's velocity every step toward its path point. Paddles that turn at the loop ends use angles, so expect native/wasm divergence and test with tolerance.
- Keep clearance between 1 and 1.5 particle diameters. Paddles should be at least 3 diameters thick, and their speed should satisfy `speed < diameter / dt`, with a `const _: () = assert!(...)` as in `hydraulic_fountain.rs:37`.
- Recirculate: a return chute from the top outlet to the intake pool keeps the scene running forever. Liquid Bubbler and Stacked Drip use a side return shaft. Do not mass-teleport particles.
- If you use a motor, size `max_motor_torque` well above the full-load torque and assert in a test that the angular velocity holds with a full trough.
- Some leak-back is real physics. Do not fight it until the scene stops lifting.

**Warning signs:**
Particles found behind the trough floor. Evictions every step. Motor angle not advancing with a full trough. The pool empties and the scene stops moving.

**Phase to address:** P-Screw

---

## Repo-level pitfalls

### Pitfall 8: An engine or shared-bridge change silently changes the 25 existing fingerprints, or the baseline cannot prove they did not

**What goes wrong:**
New capabilities, such as a per-particle flag setter for the hourglass, a pressure offset for the siphon or a group density for RT, touch shared solver code. Even an equivalent refactor reorders `f32` work and changes bits. The fingerprint baseline is a gitignored per-phase file (`target/phase43/base.txt`). Once `SURVEY_SCENES` grows past 25, a line-by-line diff against an old 25-line file fails or, worse, is "fixed" by regenerating the baseline.

**Why it happens:**
v1.5 never touched `crates/liquidfun`, so it could prove "unchanged" with `git diff --quiet -- crates/` (Phase 44 verification). v1.6 will add scenes to `crates/liquidfun-wasm`, so that shortcut no longer works.

**How to avoid:**

- In P-Prep, before any edit, record and **commit** a pinned `scene fingerprint` list of the 25 scenes from `just playground-scene-spot` (release). Compare by **scene id join**, not by line. Each later phase adds its new scene's fingerprint to the pinned list. From then on, every phase must keep all earlier fingerprints identical, both the 25 and any new ones.
- Implement the new scenes entirely in `crates/liquidfun-wasm/src/scene/<name>.rs` using existing public APIs (per-particle force, group views, fixture density, body gravity scale, particle creation and destruction). Any `crates/liquidfun` change must be additive, off by default and outside existing code paths, and the 25-scene gate must run in the same commit.
- Shared helpers (`attach_basin_fixture`, `escape.rs`, `session.rs::advance`) are shared paths. A new hook point must be a no-op default on `SceneHooks` (`scene.rs:154-211`).

**Warning signs:**
A diff under `crates/liquidfun/src/particle/solver`. A baseline file regenerated in the same commit as a scene change. A diff that compares by line number.

**Phase to address:** P-Prep (baseline and comparison tool), then every phase (gate)

---

### Pitfall 9: The frame protocol cannot carry the scene's rigid geometry

**What goes wrong:**
Rigid grains, many blades or smooth curved bulbs and tubes go past `MAX_RIGID_SEGMENTS = 512` or `MAX_RIGID_CIRCLES = 48` (`crates/liquidfun-wasm/src/frame.rs:8-10`, mirrored in `web/src/physics/frame.ts:6-8`). Capture fails, or the TS parser rejects the frame.

**How to avoid:**
Count segments while designing the scene. Smooth hourglass bulbs and siphon bends need a segment budget (for example at most 120 per scene). Rigid-grain hourglasses cannot fit 48 circles. Raising the caps is a bridge-and-parser change: update both constants and the worker message validation tests (v1.4 P13 keeps strict lane validation). Raising them does not change fingerprints, but it must be its own reviewed change.

**Warning signs:**
`collect_segments` returns more than 400 items. Polygons drawn as many-sided circles.

**Phase to address:** P-Prep (decide during the hourglass spike), P-Siphon, P-Hourglass

---

### Pitfall 10: Hard-coded "25" and per-scene `Record<SceneId, …>` tables break on every new scene

**What goes wrong:**
Each new scene breaks a dozen tests and strings, and each fix repeats per phase (26, 27 … 31). Known locations:

- Rust: `scene_spot.rs:46` (`SURVEY_SCENES: [...; 25]`, checked against `scenes.ts` by `scene_spot/tests.rs:27-35`), `scene/gravity_slider.rs:122, 145, 191` (`[false; 25]`, `[SceneId; 25]`), the `SceneId` enum, `parse_scene_id`, `build_scene` (`scene.rs:45-100, 214-247`), and `particle_iterations` (`scene.rs:108-114`)
- TS tests: `web/tests/scenes.test.ts:140, 174, 239, 433` and its `UI_SPEC_DESCRIPTIONS` and `UI_SPEC_HINTS` records (`:25, :69`); `web/tests/demo-media-model.test.ts:28`; `web/tests/portrait-bounds.test.ts:45` (`MIN_HEIGHT_FRACTION: Record<SceneId, number>`); `web/tests/scene-catalog-credits.test.ts:68`
- e2e: `web/e2e/shell.spec.ts:58-61` ("All twenty-five demos"), `:235-253` (27 focusables, 28 tabs), `:323-335` (25 previews); `web/e2e/player.spec.ts:41-56` (`POINTER_CONTROL`)
- Copy: `web/src/player/runtime.ts:11` ("All twenty-five demos …"), `web/src/components/PlaygroundShell.tsx:73`, README and `PROJECT.md` prose
- Hand-drawn previews: `web/src/catalog/previews.tsx` has a switch arm per scene

**How to avoid:**
In P-Prep, change count assertions to derive from `SCENE_IDS.length` wherever the count itself is not the point. Keep the explicit "approved order" list, since it guards order on purpose, and extend it. Write a per-scene checklist that names every file above. In Rust, consider deriving the gravity-slider and survey arrays from one `ALL_SCENE_IDS` constant. Change the spelled-out "twenty-five" copy once to text that does not hard-code a number, or to a value computed from the catalog.

**Warning signs:**
A scene phase whose diff touches more than about 15 files only to bump counts.

**Phase to address:** P-Prep

---

### Pitfall 11: File-length limits are already nearly full

**What goes wrong:**
`web/src/catalog/scene-records.ts` has 617 lines and `previews.tsx` has 562, against `FILE_LINE_LIMIT = 628` (`scripts/bright-builds-check.ts:8`). One scene record (about 18 lines) or one preview (about 15–25 lines) takes `scene-records.ts` past the limit. Six previews take `previews.tsx` past it. Then `bun scripts/bright-builds-check.ts all` fails.

**How to avoid:**
In P-Prep, create `web/src/catalog/curiosity-scene-records.ts` and a split preview module, then spread them into `SCENES` and the preview switch. Rust scene files should stay under the limit as well (v1.1 kept named scenes under 628 lines). Split large scenes into `scene/<name>.rs` plus `scene/<name>/` modules, as `hydraulic_fountain/` does.

**Phase to address:** P-Prep

---

### Pitfall 12: The README preview or portrait frame is missing, or shows nothing happening

**What goes wrong:**

- `assertReadmeSvgPlanCoverage` fails because a plan is missing or out of order (`web/scripts/readme-svg/plans.ts:132-170`).
- A plan exists, but the 10 s circle-wireframe clip shows a still scene. Pointer-driven scenes (ferrofluid, diver) do nothing without input, and cues support only `action` and a single `pointer-up` (`plans.ts:23-34`; `web/scripts/readme-svg.ts:144-153`). There is no held press or drag.
- Slow phenomena (siphon equalizing, the hourglass emptying, RT finishing) fall outside the 10 s window.
- The README clip uses `DEFAULT_RENDERED_PARTICLE_LIMIT = 4000` (`web/src/render/particle-limit.ts:5`). Scenes above that draw only part of their particles.
- A portrait frame is missing or too small. `PORTRAIT_VIEW_BOUNDS` is keyed by every `SceneId`, and `MIN_HEIGHT_FRACTION` applies. A wide siphon (two tanks side by side) fills little of a 390×844 phone.

**How to avoid:**

- Give every pointer scene an idle demo. The ferrofluid magnet slowly sweeps a fixed path when no pointer is held, and the diver squeezes on a fixed cycle when idle. Alternatively add `pointer-down` and `pointer-move` cue kinds to both `ReadmeSvgCue` and the Rust `SurveyCue` (`scene_spot.rs:24-42`) in one reviewed change. Idle demos also keep `scene-spot` timing realistic (v1.4 P05: an idle default is not the expensive case).
- Tune so the "aha" happens inside 10 s: a primed siphon, sand already flowing at step 0, a seeded RT interface.
- Stack the siphon tanks vertically (upper tank, crest, lower tank) for portrait. The hourglass, RT and diver are naturally tall.
- Run `just readme-svg` in each scene phase and look at the WebP.

**Phase to address:** Each scene phase. Add new cue kinds in P-Prep if chosen.

---

### Pitfall 13: New pointer gestures conflict with the existing poke/drag model

**What goes wrong:**

- Hover does not exist. `reducePointerEvent` ignores moves without a captured pointer (`web/src/input/pointer.ts:112-121`). "Spikes follow the mouse" only works while the button is pressed, and touch has no hover anyway.
- A press that is never released (lost capture, hidden tab, route change) leaves the diver "squeezed" or the magnet on. Cancel is sent on many lifecycle paths (`scene-lifecycle.ts:46, 242, 329`; `scene-surface.ts:147, 295`; `worker-controls.ts:34`), but only scenes that handle `PointerKind::Cancel` benefit.
- When pan is enabled, the canvas pans instead of forwarding pointer events (`attachCanvasPointer` `panEnabled`). A zoomed-in user cannot squeeze.
- The e2e suite only knows `"drag"` and `"click"` gestures (`web/e2e/player.spec.ts:41-56`). A click on the diver is a 1-frame squeeze, so the test asserts nothing useful.
- There is only one pointer (`maybePointerId`). Two-finger gestures are out of scope.

**How to avoid:**
Handle `Up | Cancel` the same way, as Color Mixer does (`color_mixer.rs:329-330`). Add a `"hold"` gesture to the e2e helper and assert a measurable effect (diver y falls while held and rises after release). Provide a labeled keyboard or HUD equivalent. The project rule is that "labeled controls also work from the keyboard". A held key needs down/up semantics, while `action` is one-shot, so use a toggle action or a range control (for example "Squeeze %"). Any on-canvas button must be a `data-hud-obstacle` leaf with a size that does not depend on the camera (`AGENTS.md`, Playground canvas framing).

**Phase to address:** P-Diver (hold gesture), P-Ferro (drag-hold)

---

### Pitfall 14: Shared gravity controls, tilt and eviction break scene assumptions

**What goes wrong:**

- Every scene accepts the gravity slider (2–80 m/s², recreates the world; `gravity_slider.rs:1-21`) and phone tilt (any direction). A ferrofluid force tuned for 10 m/s² cannot lift at 80 and flings everything at 2. A diver's densities stop working at extreme gravity. An "extra weight" RT force pointing straight down is wrong when gravity is tilted.
- Eviction measures from the **world origin**: more than 12 m along gravity, or more than 48 m sideways or upward (`session/escape.rs:8-11, 70-79`). A scene placed far from the origin, or whose parts sit more than 12 m out sideways, loses particles when tilt rotates gravity. An hourglass flipped by inverting gravity evicts anything above y = 12.

**How to avoid:**
Make every scene force proportional to `world.gravity()` (direction and size). Keep the geometry within about 10 m of the origin in every direction. Flip the hourglass by rebuilding or by mirroring particle positions about the neck, not by inverting gravity. Add a test at gravity 2 and at 80 for each scene that depends on gravity.

**Phase to address:** Each scene phase

---

### Pitfall 15: Particle budget and real-world scale make phones fall into slow motion

**What goes wrong:**
The player runs at most 4 steps per frame within a 12 ms frame budget (v1.5 Pitfall 1). Scenes that cost several milliseconds per step on a phone run in visible slow motion. Building at real scale, such as a 30 cm diver bottle or a 20 cm hourglass, needs tiny particles and many particle iterations. Liquid Tumbler, at real scale, needs `PARTICLE_ITERATIONS = 61` and costs 18.5 ms/step natively on an M4 Max, the outlier of the catalog (`liquid_tumbler.rs:41-44`; `docs/benchmarks/scene-survey.md`). The proxy-domain eviction also shrinks with particle size (`escape.rs:66`).

**How to avoid:**
Budget at most 3,000 particles per scene (the v1.5 level cap) and a target of at most about 1.5 ms/step native release. That is the upper band of the current catalog, excluding Tumbler. Measure with `just playground-scene-spot` using a cue that reflects the active state. Use meter-scale "toy" geometry with radius 0.04–0.06 m and the default iterations. The scale legend is mandatory and in meters, so a 3 m hourglass honestly shows 3 m. That is acceptable and better than a scene that does not run. Per-particle force loops (`apply_particle_force` allocates and validates per call; `force.rs:72-91`) are fine at about 3k per step but must be measured. Use range calls for whole groups.

**Warning signs:**
Native release above 2 ms/step. `PARTICLE_ITERATIONS` override added. Particle radius under 0.02 m.

**Phase to address:** Each scene phase (record ms/step in the summary)

---

### Pitfall 16: Native, wasm and debug runs disagree, so scene tests flake

**What goes wrong:**
A Rust test asserts "the siphon equalizes at step 812" or "the diver touches bottom at step 300". It passes in release and fails in debug (divergence by steps 2–91), or it differs from the browser for rotating bodies (Theo Jansen diverged at step 70; `37-01-SUMMARY.md`). The screw, a flipping hourglass and any rotating paddle all use `sin`/`cos`.

**How to avoid:**
Scene behavior tests assert outcomes with margins: "levels within 0.15 m by 20 s", "diver y < 0.5 m within 4 s of press". Keep exact-bit checks for the release-native fingerprint gate only. Put tests in the existing `liquidfun-wasm` lib test binary, because new test binaries trigger roughly 2-minute macOS launch scans (memory: macOS launch stalls). Never use wall clock, `Math.random` or `HashMap` iteration in scene logic. Survey runs compare fingerprints across 3 runs (`scene_spot.rs:323-333`).

**Phase to address:** Each scene phase

---

### Pitfall 17: Dark or low-contrast colors make the subject invisible

**What goes wrong:**
The canvas is `#071018`, nearly black (`web/src/render/canvas.ts:30`; `web/src/styles/player-viewport.css:18`), and so is the hand-drawn preview background (`previews.tsx` `CANVAS`). A "dark" ferrofluid disappears. Density shading (on by default) darkens packed particles to 73% (`density-shade.ts`), which hides the bottom of RT layers and sand heaps. Water mode adds white foam at speed and refraction, which can wash out the RT colors.

**How to avoid:**
Use a charcoal or gunmetal ferrofluid with clear contrast against `#071018`, for example a blue-gray, not black. Check that it reads in shaded-blob (default), Water, circle-wireframe (README) and the static preview. Choose RT and sand colors that survive the 0.73 floor.

**Phase to address:** P-Ferro, P-RT, P-Hourglass

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Add per-group density or a friction flag to `crates/liquidfun` | "Real" physics | Fingerprint risk, departs from upstream semantics, parity debt | Never in this milestone; record it as a seed |
| Apply scene forces in TypeScript per frame | No bridge change | Frame-rate-dependent physics; untestable natively | Never |
| Teleport blades or particles with `set_*_transform` or `set_particle_position` every step | Easy recirculation | Overlap explosions; costly spatial repair per call | Only for a one-off flip or mirror while not stepping |
| Regenerate the fingerprint baseline after a change | Green gate | Hides regressions in the original 25 | Never; extend the pinned list only with new scene ids |
| Bump every "25" by hand each phase | Fast | 6× repeated churn and missed spots | Never; derive from `SCENE_IDS.length` in P-Prep |
| Caption a faked effect as real ("sand piles", "air compresses") | Better story | Breaks the project's Transparency constraint | Never; say "fake" or "stands in for" |
| Leave a pointer scene with no idle behavior | Simpler | Still README clip; survey measures the idle cost | Only with a new README/survey cue kind |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `SceneHooks::on_advance` (`scene.rs:155-159`) | Putting force code in `apply_pointer` (runs per message, not per step) | Store pointer state in `apply_pointer`; apply forces in `on_advance` |
| `evict_escaped_particles` (`session.rs:231-239`) | Caching group member ids across steps | Re-read `particle_group_view(...).member_ids()` every step; eviction shrinks groups |
| `apply_particle_force_range` | Passing the per-particle force | The force is divided by the count; multiply by member count (`impulse.rs:186-193`) |
| `set_fixture_density` | Expecting the mass to update | Call `reset_body_mass_data`, then wake the body |
| `set_fixture_friction` | Changing friction mid-run (hourglass grains) | It affects only future contacts (`fixture_object.rs` doc) |
| Gravity slider (`ControlEffect::Recreated`) | Assuming the scene keeps its state | The world is rebuilt; idle demo state and pointer state reset |
| `ReadmeSvgCue` / `SurveyCue` | Adding a cue kind in TS only | Mirror it in Rust `SurveyCue` and in `readme-svg.ts:144-153` |
| `player.spec.ts` `POINTER_CONTROL` | New interactive scene not listed, so it is tested as watch-first | Add it with a `"hold"` gesture once that exists |
| Portrait frames | Padding for HUD chrome | The safe frame already avoids `data-hud-obstacle` leaves; size to the action only |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Real-scale geometry with tiny particles | Needs high particle iterations; 10× step cost | Toy meter scale, radius 0.04–0.06 | Radius under about 0.02 m |
| Per-particle `create_particle`/`destroy_particle` every step (freeze-to-wall sandpile) | Step cost grows with heap size | Freeze at most K particles per step; measure | Over a few hundred re-creations per step |
| `set_particle_group_flags` per step | O(N) system clone each call (`group_mutation.rs:242`) | Set flags at creation only | Any per-step use |
| Smooth curves with many segments | Frame lane overflow, slow draw | At most about 120 segments per scene | Over 512 total |
| Idle-only survey cue | Survey says "cheap" while the real interaction is costly | Scripted active cue | Ferro and diver scenes |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting pointer world coordinates in force math | NaN or Inf forces poison the world | Already checked at the session boundary (`session.rs:356-358`); also clamp the magnet position to the scene bounds |
| New control or action tokens parsed loosely | Unknown tokens accepted | Follow `parse_gravity_magnitude` strictness; return `SessionError::UnknownControl` |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Desktop users hover and nothing happens (ferrofluid) | "Broken" | Hint: "Press and drag over the liquid"; idle sweep shows the effect |
| Hold gesture with no visible feedback (diver) | Users tap, see nothing | Draw the squeeze on the bottle and the bubble shrink at once |
| Scenes that finish and freeze (RT, hourglass, siphon) | Looks stuck | One clear "Flip" or "Again" action; an auto-restart is optional |
| Fake physics presented as real | Erodes trust | One honest line in the description per faked mechanism |
| Wide two-tank siphon on a phone | Tiny subject | Vertical layout for portrait |

## "Looks Done But Isn't" Checklist

- [ ] **Siphon:** flow continues after the source surface is below the crest, measured, not only overflow. Verify the P-Prep spike numbers and the chosen mechanism are named in the description.
- [ ] **Hourglass:** a measured heap slope of at least 20°. Verify the caption makes no piling claim without that number, and that flip leaves no particles outside the glass.
- [ ] **Diver:** sinks fully and rises fully at default gravity and at gravity 2 and 80. Verify `reset_body_mass_data` and wake.
- [ ] **RT:** fingers visible by 3 s in the README clip. Verify there is no `COLOR_MIXING` and one particle system only.
- [ ] **Ferrofluid:** a 60 s held pointer causes no evictions and bounded speed. Verify the force is per step in Rust, and the idle sweep exists.
- [ ] **Screw:** runs 60 s without the pool emptying. Verify the clearance and speed asserts.
- [ ] **Fingerprints:** the original 25 plus earlier new scenes match the committed pinned list by id.
- [ ] **Catalog plumbing:** `SCENE_IDS`, `SceneId`, `parse_scene_id`, `build_scene`, `SURVEY_SCENES`, gravity-slider ids, README plan, portrait bounds plus `MIN_HEIGHT_FRACTION`, preview arm, credits, UI spec records, `POINTER_CONTROL`, and shell counts all updated.
- [ ] **File lengths:** `bun scripts/bright-builds-check.ts all` passes.
- [ ] **Budget:** ms/step and particle count recorded; at most 3,000 particles.
- [ ] **HUD:** `web/e2e/hud-safe-frame.spec.ts` passes for the new scene; the scale legend shows.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Siphon infeasible | LOW (if found in P-Prep) / HIGH (if found in its phase) | Reword with the honest mechanism (atmosphere plates) or drop, per the D-25 precedent |
| Hourglass infeasible | LOW / HIGH, as above | Drop, or retitle as "Hourglass (liquid)" with a truthful caption; keep the spike numbers |
| Fingerprint drift found late | MEDIUM | Bisect with `just playground-scene-spot`; move the logic from the engine into the scene hook |
| Count churn per phase | LOW | Derive counts once; finish the checklist |
| Ferrofluid instability | LOW | Add core radius, acceleration cap and drag; retune |
| Frame lane overflow | MEDIUM | Simplify geometry, or raise the caps in Rust and TS with validation tests |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Siphon has no tension | P-Prep spike, P-Siphon | Logged candidate numbers; transfer continues with source below crest |
| 2. Powder does not pile | P-Prep spike, P-Hourglass | Measured slope of at least 20°; drop/reword decision recorded |
| 3. Two systems or per-group density | P-RT | One `create_particle_system`; no `crates/liquidfun` solver diff |
| 4. RT noise instead of fingers | P-RT | Finger count by 3 s; README clip review |
| 5. Ferrofluid explosion or frame dependence | P-Ferro | 60 s held-pointer native test; 60/120 Hz e2e parity |
| 6. Diver hover or no response | P-Diver | Full sink and rise test at gravity 2, 10 and 80 |
| 7. Screw leak, jam or teleport | P-Screw | Clearance and speed const asserts; 60 s run without draining |
| 8. Fingerprint drift and baseline | P-Prep, every phase | Committed pinned list; id-joined comparison |
| 9. Frame lane caps | P-Prep, P-Siphon, P-Hourglass | Segment count under budget, or caps raised with tests |
| 10. Hard-coded counts | P-Prep | Counts derived; one scene adds no count edits |
| 11. File-length limit | P-Prep | Bright Builds check passes |
| 12. README and portrait coverage | Each scene phase | `just readme-svg`; portrait test; idle demo or new cue kinds |
| 13. Pointer gesture conflicts | P-Diver, P-Ferro | `"hold"` e2e; Cancel handled; keyboard equivalent |
| 14. Gravity, tilt and eviction | Each scene phase | Gravity 2 and 80 tests; geometry within 10 m of the origin |
| 15. Phone budget and scale | Each scene phase | Scene-spot ms/step recorded under budget |
| 16. Native, wasm and debug drift | Each scene phase | Margin-based assertions; release-only exact gate |
| 17. Color contrast | P-Ferro, P-RT, P-Hourglass | Visual check in every render mode |

## Sources

- Repository code at HEAD `b151838be` (HIGH):
  - `crates/liquidfun/src/particle/solver/pressure.rs`, `material.rs`, `particle/force.rs`
  - `crates/liquidfun/src/world/particle_coupling.rs`, `world/particle_coupling/executor.rs`, `world/particle_object/{system,group,group_mutation}.rs`, `world/object/fixture_object.rs`
  - `crates/liquidfun-wasm/src/{scene,session,frame,scene_spot}.rs`, `session/escape.rs`, `scene_spot/fingerprint.rs`, `scene/{color_mixer,impulse,float_or_sink,hydraulic_fountain,liquid_tumbler,gravity_slider}.rs`
  - `web/src/catalog/{scenes,scene-records,previews}.ts(x)`, `web/scripts/readme-svg/plans.ts`, `web/scripts/readme-svg.ts`, `web/src/input/{pointer,canvas-pointer}.ts`, `web/src/render/{canvas,density-shade,particle-limit}.ts`, `web/src/physics/frame.ts`
  - `web/tests/{scenes,portrait-bounds,demo-media-model,scene-catalog-credits}.test.ts`, `web/e2e/{shell,player}.spec.ts`, `scripts/bright-builds-check.ts`
- Prior project evidence (HIGH, measured locally):
  - `.planning/phases/40-timed-valve-and-teaching-levels/40-RESEARCH.md` (powder, viscous, damping and buoyancy probes), `40-08-SUMMARY.md` (D-25 powder spike numbers), `40-09-SUMMARY.md` (buoyancy at depth, wall-seam leaks)
  - `.planning/phases/37-puzzle-core-in-the-bridge/37-01-SUMMARY.md` (native, wasm and debug divergence)
  - `.planning/research/v1.4-scenario-performance/PITFALLS.md` (P05 idle workloads, P13 lane validation), `.planning/research/v1.5-puzzle-mode/PITFALLS.md` (catalog coupling, thin-wall leaks, determinism)
  - `docs/benchmarks/scene-survey.md` (per-scene ms/step and particle counts)
- Physics background (MEDIUM, textbook knowledge, not re-verified online): siphons need ambient pressure or cohesion at the crest; granular flow through an orifice is roughly independent of head (Beverloo); the Rayleigh–Taylor Boussinesq approximation (a density difference only in the buoyancy term); the Rosensweig normal-field instability behind ferrofluid spikes; the Cartesian diver's unstable equilibrium from the compressible bubble.
- I found no web source that states outright that LiquidFun particle systems do not interact. The claim rests on this repo's solver code (HIGH for this port).

---
*Pitfalls research for: v1.6 Physics Curiosity Scenes on the liquidfun-rs playground*
*Researched: 2026-10-14*

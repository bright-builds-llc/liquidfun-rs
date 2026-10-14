# Project Research Summary

**Project:** liquidfun-rs, milestone v1.6 Physics Curiosity Scenes
**Domain:** Six new watch-first or lightly interactive playground scenes (Siphon, Archimedes screw, Sand hourglass, Cartesian diver, Rayleigh–Taylor instability, Fake ferrofluid), built on the existing pipeline: Rust engine → private `liquidfun-wasm` bridge → SolidJS playground.
**Researched:** 2026-10-14 (repo at `b151838be`)
**Confidence:**

- HIGH that every needed API, seam and integration site exists. These were read from source; the file:line references are in each research file.
- MEDIUM to LOW that each phenomenon will look right in LiquidFun particles. This is tuning and feasibility risk, rated per scene below.

Detailed files: [STACK.md](STACK.md), [FEATURES.md](FEATURES.md), [ARCHITECTURE.md](ARCHITECTURE.md), [PITFALLS.md](PITFALLS.md).

## Executive Summary

v1.6 adds six "physics curiosity" scenes to the 25-scene playground. All four researchers reached the same structural conclusion. The milestone needs none of the following:

- an engine change;
- a new bridge API or worker message;
- a new Cargo or npm dependency;
- a new README or survey cue kind.

Each mechanism is a scene module in `crates/liquidfun-wasm/src/scene/`. It uses existing public `World` APIs (per-particle and range forces, flags, groups, kinematic bodies, fixture create, destroy and density, mass reset, gravity) through the existing `SceneHooks` seams (`apply_pointer`, `apply_action`, `apply_control`, `on_advance`). As in v1.5, `crates/liquidfun` stays byte-identical, so the 25 existing fingerprints are safe by construction. The one optional engine addition is `World::set_particle_flags`. It matches upstream, is off by default, and is added only if the hourglass freeze spike needs it.

The engine sets the main risk, not the integration. Three facts from the code shape the plan:

1. **Pressure is clamped at zero** (`pressure.rs:86`). Plain water has no tension, so it cannot pull a column over a siphon crest.
1. **`POWDER` behaves like water.** This was measured in v1.5 (Phase 40-08, D-25). With no friction between particles, an angle of repose does not come for free.
1. **Density is set per particle system, and systems never interact.** Rayleigh–Taylor must therefore be one system with two groups, and the bridge adds extra weight to the heavy group.

So the siphon and the hourglass pile each need a time-boxed feasibility spike *before* their phases, with a written pass rule and a drop or reword fallback. Rayleigh–Taylor, the ferrofluid and the screw carry visual-tuning risk that a short spike inside their own phase can handle. The diver reuses proven buoyancy and has the lowest risk.

The repository plumbing needs one prep pass before any scene lands:

- `scene-records.ts` (617 lines) and `previews.tsx` (562 lines) are already near the 628-line limit.
- About a dozen places hard-code "25" (Rust arrays, TypeScript counts, e2e copy) without anything catching it, so they would break with every new scene.
- The v1.5 fingerprint gate, a `diff` of a 25-line file, fails as soon as scene 26 exists.

Recommendation:

- **Phase 45 is a prep phase:** a fingerprint baseline with a gate that joins on scene id, removing hard-coded counts through `ALL_SCENE_IDS`, the file splits, and the siphon and hourglass spikes.
- After that, one scene per phase, with the riskiest last: Diver → Screw → Ferrofluid → RT → Siphon → Hourglass.
- Every caption describes what the engine actually does, per the Transparency constraint and the D-25 precedent.

## Key Findings

### Recommended Stack

Nothing new needs installing. Rust 1.97, `wasm-bindgen =0.2.128`, SolidJS, the renderers and the catalog rules stay as they are. All six mechanisms are plain `f32` arithmetic in the bridge, built on engine calls that were checked in the code. Deterministic perturbations use a sinusoid or an inline hash rather than an RNG crate.

**Core capabilities** (all existing; verified file:line in STACK.md):

- **`apply_particle_force` / `apply_particle_force_range`:** used for the ferrofluid field, the RT extra weight, and optional siphon or sand forces. Calibration rules:
  - A range force is divided by the member count, so multiply by the count, as `impulse.rs:186-193` does.
  - The force uses the substep dt and is applied once per world step. To add an acceleration `a` over a full step, use `F = m·a·particle_iterations`, where `m = density·(PARTICLE_STRIDE·2r)²`.
  - Forces are rejected on `WALL` particles.
  - This calibration comes from reading code; measure it in the first force phase.
- **Particle flags:** `TENSILE | VISCOUS` (Liquid Timer precedent), `REPULSIVE` (acts only between groups) and `WALL`. Keep viscous strength at or below 1.5; above about 2.0 the step fails (measured).
- **Group create and destroy from explicit positions** (`ParticleGroupSource::positions`): the hourglass flip as one batched rebuild.
- **Kinematic bodies** (`set_body_linear_velocity`, `set_body_transform`): the screw's vane train.
- **Fixture density, `reset_body_mass_data` and `set_body_gravity_scale`:** the diver. `set_fixture_density` does **not** recompute mass, and changing the gravity scale does **not** wake the body.
- **`world.gravity()`:** every scene force follows it, so the gravity slider (2–80 m/s², which recreates the world) and global phone tilt stay correct.

**Do not add:**

- per-particle or per-group density or mass in the engine;
- a second particle system per scene;
- air, gas or magnetic models in the engine;
- hover pointer events;
- new cue kinds;
- an RNG crate;
- a friction flag in the solver;
- a pure-black ferrofluid (the canvas is `#071018`).

### Expected Features

**Must have (table stakes, per scene):**

- **Siphon:**
  - two tanks at different heights, and an inverted-U tube whose crest is clearly above the source surface;
  - primed and flowing at t = 0;
  - levels converge visibly within the 10 s clip, and flow stops when the levels are equal;
  - no leaks through walls.
- **Archimedes screw:**
  - a trough tilted about 25–35°, with its lower end in a reservoir;
  - moving slanted vanes carrying separate pockets, and water gained in the upper basin;
  - a live crank-speed spinner in the HUD;
  - a return chute so the scene keeps running.
- **Sand hourglass:**
  - two bulbs, a neck about 5 particle diameters wide, and a thin stream;
  - a Flip action, and sand colors.
  - A **peaked pile** is table stakes only if the spike passes; it is what tells sand apart from water.
- **Cartesian diver:**
  - a tall closed bottle with press-and-hold squeeze (pointer down and up);
  - a **Squeeze toggle action** for the keyboard, which also drives the README and survey cues;
  - reliable sinking and rising, with visible squeeze feedback;
  - no callout on an iPhone long press.
- **Rayleigh–Taylor:**
  - high-contrast heavy and light layers, fingers by about 3 s and overturn by about 10 s;
  - a density-ratio control (about 1.2–3×) that recreates the scene, and a way to restart;
  - extra weight that follows gravity.
- **Fake ferrofluid:**
  - a dark but visible puddle with **several separate spikes** (about 5–9, not one blob);
  - a resting magnet by default, so the scene moves on load;
  - press-and-drag to move the magnet, collapse on release or when switched off, and a drawn magnet marker;
  - a strength control clamped below the point where liquid detaches.
- **Every scene:**
  - a catalog record with honest credits, a portrait frame, a README plan, the scale legend in meters, the gravity slider and the HUD safe frame;
  - motion within about 1 s (the diver is the exception and invites a press);
  - a deterministic build with at most about 3,000 particles;
  - readable in every render mode.

**Should have** (add within v1.6 if cheap):

- **Siphon:** an outlet-tank height control (which reverses flow), Lift inlet and Prime actions, and two-tone tracer water (no `COLOR_MIXING`).
- **Diver:** a bubble-shrink overlay tied to the squeeze state.
- **Hourglass:** an animated 180° flip. A phone-tilt flip comes free if the sand rule follows gravity.
- **RT:** perturbation presets, and a Flip-tank action that reuses the hourglass flip transform.
- **Screw:** reverse direction.

**Defer (seed or later):**

- **Screw:** a sand material, a tilt-angle control and a helix overlay.
- **Hourglass:** neck-width jamming and a sand-versus-water flow-rate lesson.
- **Diver:** the diver "no-return depth", and multiple divers.
- **Ferrofluid:** hysteresis.

The **recognizability criteria** (full table in FEATURES.md) become acceptance proxies for requirements. Examples:

- **RT:** at least 3 heavy fingers past 20% of the tank height by 4 s.
- **Ferrofluid:** at least 4 surface maxima with the magnet on.
- **Diver:** reaches the bottom third within 3 s of the squeeze cue.

### Architecture Approach

Each scene is a top-level `scene/<id>.rs` with a `SceneHooks` implementation. It can have `<id>/tests.rs` and an optional `<id>/geometry.rs`; `links.ts` rejects credit paths inside subdirectories.

**Pattern 1: hooks record intent, and `on_advance` does the physics.** Pointer, action and control handlers only store state, or make one edit that does not depend on the step, such as a flip. Continuous forces are applied once per fixed step. The engine clears its force buffer each step, and pointer messages arrive between batches of up to 4 steps.

**Pattern 2: emulate in the bridge.** The RT weight, the magnet and the squeeze are bridge forces or body edits, never engine features. Captions say "the heavy layer is pulled down harder", not "density 1.5×".

Adding a scene touches about 9 Rust sites and 13 web sites. The compiler forces half of them. The other half are counts and lists that nothing checks, and Phase 45 fixes those once.

**Major components:**

1. **`scene/<id>.rs` (×6, new):** world construction, a hooks struct, scene constants, and native outcome tests in the existing lib test binary. A new test binary would cost about 2 minutes of macOS launch scan.
1. **`scene.rs` / `gravity_slider.rs` / `scene_spot.rs` (modified):**
   - `SceneId`, `parse_scene_id` and `build_scene`, plus an optional `particle_iterations` arm;
   - a new `ALL_SCENE_IDS` constant;
   - `SURVEY_SCENES` grown to 31 entries with `Default` or `Action` cues.
1. **Web catalog (modified and split):**
   - a new `curiosity-scene-records.ts` and a split preview module, both spread into `SCENES` and the `ScenePreview` switch;
   - entries in `portrait-bounds.ts`, `plans.ts` and `demo-media/model.ts`;
   - an optional `<id>-controls.ts` for each live control, such as screw speed.
1. **Frame lanes (unchanged):** limits of 16,384 particles, 512 segments and 48 circles. Smooth bulbs and tubes need a budget of about 120 segments per scene. Sand made of rigid circle bodies cannot fit.

### Critical Pitfalls

1. **The siphon has no tension.** Plain water breaks at the crest once the source surface falls below it, and the scene turns into overflow.
   - *Spike pass rule:* water keeps transferring for at least 2 s after the source surface is 0.3 m below the crest, and the final level difference is under 0.15 m.
   - *Geometry:* a tube at least 5 particle diameters wide, made from thick overlapping polygon walls, primed at the start.
   - Never call overflow a siphon.
1. **Hourglass sand flows like water.** No heap forms, and the scene duplicates Liquid Timer.
   - *Spike pass rule:* a deposit slope of at least 20°, stable within ±5° over 5 s, with no jam.
   - Rigid-body grains are ruled out by the 48-circle cap.
   - Never claim "piles" or "angle of repose" without a measured slope.
1. **Fingerprint drift, or a gate that cannot prove there is none.**
   - Pin the 25 fingerprints at `b151838be` before any edit.
   - Gate by **joining on scene id**: every protected id must be present and identical, while new ids may appear.
   - Grow the protected set each phase, and never regenerate the baseline in the same commit as a scene change.
   - Do not reuse or "improve" `basin_family` or `soup_family`; copy what you need.
1. **Forces that are mis-scaled or applied in the wrong place (RT, ferrofluid).**
   - Forces applied in `apply_pointer` or once per TypeScript frame become frame-rate dependent.
   - A range force passed per particle gets divided by the count.
   - An unbounded 1/r pull explodes or gains energy, and a hard-coded −y breaks tilt.
   - *Avoid:* apply forces only in `on_advance`, use the count and iteration calibration, cap and soften the profile with drag, use `world.gravity()` everywhere, and run a native stability test with the pointer held for 60 s.
1. **The diver hovers or ignores the press.**
   - *Causes:* a missing `reset_body_mass_data`, a sleeping body, or water that compresses with depth (a ball of density 2.0 hovers at 3 m).
   - *Avoid:* wide density margins verified at gravity 2, 10 and 80, sleep disabled, a circle body, and a shallow bottle (1–1.5 m of water).
   - Make the bottle squeeze visual only; never push walls into a full bottle.

Every phase also handles these:

- treat pointer `Up` and `Cancel` the same way, so a squeeze or magnet never gets stuck;
- keep geometry within about 10 m of the origin, because particles are evicted based on distance from the origin;
- use a toy meter scale with particle radius 0.04–0.06 m (tiny real-scale particles cost about 10× per step);
- assert outcomes with margins, because debug, release and wasm builds diverge for rotating bodies;
- choose colors that survive the 0.73 floor of density shading.

## Reconciled Disagreements

| Question | Positions | Recommendation |
| --- | --- | --- |
| **Build order** | ARCH: Diver, Screw, RT, Ferro, Siphon, Hourglass. PITFALLS: Diver, Screw, Ferro, RT, Siphon, Hourglass. FEATURES: siphon and screw late. STACK: RT last. | **Diver → Screw → Ferro → RT → Siphon → Hourglass** (the PITFALLS order). The diver and screw are rigid-body mechanisms with direct precedents. Ferro builds the gravity-aware force helper and its calibration, which RT then reuses. Siphon and Hourglass depend on the Phase 45 spikes and go last. No scene depends on another scene's code, so a spike result can reorder them. |
| **Hourglass flip** | STACK: destroy and recreate the sand group at `2c − p`; per-particle `set_particle_position` is O(n²). ARCH: rotate each particle's position and velocity, (p, v) → (2c − p, −v). FEATURES: animated rotation as a differentiator. | **Instant point reflection by rebuilding the group**, with a point-symmetric glass whose walls never move. It is one batched call and nothing tunnels through a wall. The rebuild must gather **all** sand, including frozen grains, and refresh cached group handles. Measure the per-particle cost during the spike before rejecting it outright. **Never flip by inverting gravity.** An animated rotation is a stretch goal (P2). |
| **Diver mechanism** | STACK: shrink an air-pocket fixture of density 0. FEATURES: `set_fixture_density` with margins of 0.85× and 1.2×. ARCH: gravity scale or density. PITFALLS: density or gravity scale, with margins 0.5 / ≥3. | **Density switch** as the core: `set_fixture_density`, then `reset_body_mass_data`, then wake the body. Use wide margins, based on the measured 40-09 depth table. Gravity scale is an acceptable alternative, with sleep disabled. Show the bubble overlay as a drawing. The air-pocket swap is later polish only. |
| **RT risk** | STACK: HIGH. FEATURES and ARCH: MEDIUM. PITFALLS: two critical pitfalls. | **MEDIUM-HIGH overall: the mechanism is LOW risk and the visual result is MEDIUM-HIGH.** Start the phase with a parameter sweep: density ratio, `VISCOUS` ≤ 1.5, `REPULSIVE` between groups, a seeded perturbation, a 0.5–1 s settle before the force ramps up, and resolution. It ships with a qualitative caption if the fingers come out coarse. |
| **Siphon spike order** | Varies between researchers. | Plain water (control) → `VISCOUS` → `TENSILE \| VISCOUS` → an atmosphere push on surface particles → weighted lids → a modeled tube pull as the last resort, gated on the inlet being submerged and credited. Open question: `TENSILE` zeroes weight pressure (`pressure.rs:93-101`), so measure whether the water-height difference that drives the flow still comes through. |
| **Screw wrap and clearance** | STACK and FEATURES: one kinematic body that snaps back exactly one pitch. PITFALLS: closed-loop paddles, clearance of 1–1.5 particle diameters. | **A single kinematic body that snaps back one pitch.** Vane ends are embedded in thick walls, the track ends in sealed housings, the vanes are at least 3 particle diameters thick, and a `speed < diameter/dt` constant assert guards the speed. Fall back to closed-loop paddles only if the snap disturbs the fluid. MEDIUM risk. |
| **Hourglass sand rule** | Varies between researchers. | Try in order: bridge stick-slip friction → freeze settled grains to `WALL` (at most K per step, released on flip and on any gravity change, optionally via `World::set_particle_flags`) → rigid particle clumps. Never freeze grains permanently. |
| **If the hourglass spike fails** | STACK: ship it as "sand pours". FEATURES and PITFALLS: drop it. | **Default: drop the hourglass back to a seed.** The owner decides at the end of Phase 45. The documented alternative is a version that pours, with no claim that it piles. |
| **Fingerprint baseline storage** | ARCH: gitignored. PITFALLS: committed. | **Commit** the pinned list of id → fingerprint with its host and toolchain provenance. Keep raw output in `target/`. Gate by joining on id. |
| **"Twenty-five" copy** | Edit it per scene, or derive it. | **Derive the count, or remove the number from the copy, in Phase 45.** |
| **Idle behavior of pointer scenes** | New cue kinds vs toggles and resting magnet. | **No new cue kinds.** The diver gets a `squeeze` toggle action (README cues, and `SurveyCue::Action` for the survey). The ferrofluid gets a resting magnet by default (`Default` cue). |
| **Scene scale** | Real sizes vs toy meters. | **Toy meter scale**: at most 3,000 particles and at most about 1.5 ms per step in native release. |

## Per-Scene Risk

| Scene | Risk | Main risk | Gate or spike | Fallback |
| --- | --- | --- | --- | --- |
| Cartesian diver | LOW-MEDIUM | It hovers near neutral, or the body sleeps | None; tune margins within the phase | Switch gravity scale instead |
| Archimedes screw | MEDIUM | The wrap disturbs the fluid, leaks, crushed particles, or the pool runs dry | Short wrap and leak spike at the start of the phase | Closed-loop paddles |
| Fake ferrofluid | MEDIUM | One blob instead of spikes; energy gain or eviction | Stability and appearance spike at the start of the phase | Fixed spike-tip targets |
| Rayleigh–Taylor | MEDIUM-HIGH | Rain or one blob instead of fingers | Parameter sweep at the start of the phase | Single-mode seed with a qualitative caption |
| Siphon | HIGH | No tension, so the column breaks at the crest | **Phase 45 spike with a pass rule** | A credited atmosphere mechanism, or drop the scene |
| Sand hourglass | HIGH (the pile; pour and flip are LOW) | A flat deposit like water | **Phase 45 spike with a pass rule** | Drop it back to a seed (default), or ship it as "pours" |

## Caption Honesty Rules

These follow the Transparency constraint and the D-25 precedent. Each faked mechanism gets one plain line.

- **Siphon:**
  - Name the mechanism the spike chose.
  - Never say "suction" or "atmospheric pressure" unless that is what is modeled, and never claim a real 10 m limit.
  - Say "cohesion" carefully.
- **Hourglass:** Say "piles" or "angle of repose" only if the summary records a measured slope of at least 20°. Credit the sand rule.
- **Diver:** "Switching the diver's density stands in for the squeezed air bubble." Never say the air is simulated.
- **Rayleigh–Taylor:**
  - "The orange layer is given extra weight." Never say "density 2×" as if it were simulated.
  - Describe growth qualitatively.
- **Ferrofluid:** Put "fake" in the title or description, and describe "a scripted magnetic pull".
- **Screw:** "Flat vanes stand in for a turning helix (2D side view)."
- **All scenes:** The credits say "original experimental scene", and maturity claims stay within what each phase verified.

## Implications for Roadmap

Each scene phase keeps the per-scene shape used in v1.3:

1. Plan 01: the Rust scene, a native outcome test, and the Rust sites.
1. Plan 02: the catalog record, preview, portrait frame, README and demo-media plans, and unit tests.
1. Plan 03: Chromium smoke tests, `just readme-svg` with a review of the WebP, and the fingerprint gate that joins on id.
1. Plan 04 (optional): polish.

Every phase records its particle count and ms/step.

1. **Phase 45, Curiosity Groundwork and Feasibility Spikes.** It delivers:
   - a pinned, committed list of 25 fingerprints, plus the gate that joins on id;
   - `ALL_SCENE_IDS`, with the hard-coded counts derived from it and the copy made free of numbers;
   - the file splits;
   - a written per-scene checklist;
   - time-boxed siphon and hourglass spikes against written pass rules, each with a recorded decision to go, reshape or drop.
1. **Phase 46, Cartesian Diver.** It delivers:
   - the density switch, a shallow bottle, and a visual squeeze and bubble;
   - a `squeeze` toggle action, suppression of the iPhone long-press callout, and an e2e "hold" gesture;
   - sink and rise tests at gravity 2, 10 and 80.
1. **Phase 47, Archimedes Screw.** It delivers:
   - a kinematic vane train that snaps back one pitch;
   - sealed ends and a return chute;
   - a live speed spinner;
   - a 60 s run with no drain and no evictions.
1. **Phase 48, Fake Ferrofluid.** It delivers:
   - a shared per-step force helper that follows gravity and is mass-calibrated;
   - a capped, modulated magnet field with a resting magnet by default and press-and-drag;
   - a strength control, a marker and a visible color;
   - a 60 s stability test with the pointer held.
1. **Phase 49, Rayleigh–Taylor Instability.** It delivers:
   - one system with two groups, and a heavy-group force along gravity;
   - a settle before the force ramps up, a seeded interface, and a density-ratio control;
   - a restart, and the README showcase.
1. **Phase 50, Siphon.** Its shape is set by the Phase 45 spike. Alternatively it is reworded or dropped, according to the recorded decision.
1. **Phase 51, Sand Hourglass.** Its shape is set by the Phase 45 spike, and it goes last so it can slip or be dropped.

### Research Flags

- **Phase 45:** the spikes are the research.
- **Phases 50 and 51:** run `/gsd-research-phase` after reading the spike summaries.
- **Phase 49:** a parameter sweep inside the phase.
- **Phases 47 and 48:** short spikes inside the phase.
- **Phase 46:** standard patterns, so skip research.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH | APIs read with file:line references, and checked against the pinned upstream. The force calibration is still unmeasured. |
| Features | MEDIUM | How the real phenomena behave is HIGH confidence. How each one emerges from LiquidFun particles is MEDIUM to LOW. |
| Architecture | HIGH | The checklist, the count sites, the line counts and the frame caps were all read from source. |
| Pitfalls | HIGH / MEDIUM | The engine facts and v1.5 measurements are HIGH. The proposed workarounds are reasoned, not measured, so MEDIUM. Phone cost is LOW. |

**Overall:** MEDIUM. Confidence in the plan structure is high. Whether all six scenes ship depends on two spikes whose outcome is uncertain.

### Gaps to Address

These are open questions to settle during the spikes and early phases:

- **Siphon:** does `TENSILE` still pass on the water-height difference that drives the flow?
- **Hourglass freeze:** the cost of freezing settled grains, and how frozen grains interact with the flip.
- **Hourglass flip:** the cost of per-particle `set_particle_position`.
- **Forces:** a numeric check of the force calibration.
- **Diver:** the margins at gravity 2 and 80.
- **Fingerprints:** where the committed fingerprint list lives.
- **Interaction:** pan versus press when the view is zoomed in.
- **Performance:** phone cost.
- **Tests:** Chromium timeout growth.

## Sources

- **Primary (HIGH):**
  - repository code at `b151838be`, as listed in each research file;
  - the pinned upstream at `7f20402173fd143a3988c921bc384459c6a858f2`;
  - measured project evidence in `40-RESEARCH.md`, `40-08-SUMMARY.md`, `40-09-SUMMARY.md`, `37-01-SUMMARY.md` and `docs/benchmarks/scene-survey.md`;
  - template commits `d456577e0`, `c0fe8fdff` and `09ea81605`.
- **Secondary (MEDIUM):**
  - physics references in FEATURES.md;
  - the siphon-cohesion literature: Sci Rep 2015 and its corrigendum, and the JFM paper on open capillary siphons.
- **Tertiary (LOW):** phone cost estimates, and vendor guidance for Archimedes screws.

---

*Research completed: 2026-10-14. Ready for roadmap: yes. Synthesized by a research-synthesizer subagent and saved by the orchestrator, because subagents cannot write summary files in this environment.*

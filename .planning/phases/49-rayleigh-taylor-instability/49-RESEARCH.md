# Phase 49: Rayleigh–Taylor Instability - Research

**Researched:** 2026-10-18
**Domain:** A two-group, one-system particle scene in the private `liquidfun-wasm` bridge, with gravity-aware extra weight on one group (reusing the Phase 48 helper), a recreating preset, and the per-scene catalog checklist (Rust + SolidJS web)
**Confidence:** HIGH for engine facts, helper API, preset/session plumbing and every integration site (all read in this session); MEDIUM-LOW for the starting geometry and tuning numbers (derived, not probed; the Plan 1 spike measures them, as D-06 requires)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Physics model
- **D-01:** One particle system with two groups: the heavy group on top and the light group below. Never two particle systems, because they would pass through each other (PITFALLS 3). Never add per-group density to the engine. Never set `COLOR_MIXING`.
- **D-02:** Once per fixed step in `on_advance`, apply an extra acceleration of `(ratio − 1)·|g|` along the current gravity direction (`-GravityFrame::up`) to the heavy group only, through `particle_force::apply_group_acceleration`. Remove its `expect(dead_code)` attribute now that it has a caller. Build the frame from `world.gravity()` every step. When `GravityFrame::from_gravity` returns `None`, apply nothing for that step. No term hard-codes −y, so the gravity slider (2 to 80) and phone tilt keep "heavy" pointing down. This is the Boussinesq picture: equal inertia, different weight.
- **D-03:** Settle, then ramp. The extra weight is off for a short settle (about 0.3 to 0.6 s) so the lattice pressure wave does not stir the interface, then ramps linearly to full over about 0.2 to 0.4 s. The total delay must leave the 4 s finger target reachable. Exact timings are Claude's discretion and come from the spike.
- **D-04:** The interface starts with a seeded, deterministic perturbation: one to three fixed sinusoidal modes in the initial heavy/light boundary, with an optional small fixed inline hash for noise. Use no RNG crate and no time-based seed, so repeated runs and `scene-spot` stay identical (`scene_spot.rs` fails when two runs disagree).
- **D-05:** Flags are chosen by the spike from these candidates, in this order: plain water on both groups; `VISCOUS` (strength at most 1.5) on both groups; `VISCOUS` plus `REPULSIVE` between groups for a sharper interface. Avoid `TENSILE` in the bulk, because it zeroes normal pressure. Keep the first candidate that passes D-06.

#### Spike first
- **D-06:** Plan 1 starts with a short, time-boxed parameter sweep over flags (D-05), perturbation modes (D-04), particle radius, and the settle and ramp timings (D-03), at ratios 1.2, 2 and 3 and gravity 10. Written pass rules, measured at the default ratio and gravity 10:
  - at least 3 separate heavy fingers reach past 20% of the tank height below the initial interface by about 4 s;
  - overturn by about 10 s, meaning the heavy group's mean height along the up axis is below the light group's mean height;
  - a 60 s run has no eviction, no failed step and a bounded max particle speed;
  - two runs give identical end-state fingerprints.
  At ratio 1.2 the fingers may be slower, but the layers must still visibly overturn within about 20 s, and the measured times are recorded. Record the sweep measurements in the plan summary. If no candidate passes, stop and record a decision checkpoint (an amendment like Phase 46 D-07a) instead of loosening the floor silently. The researched fallback is a single-mode seed with a qualitative caption.
- **D-07:** Fingers are measured from heavy particles binned across the tank along the `across` axis. A finger is a run of adjacent bins whose deepest heavy particle is past the 20% depth line, separated by at least one bin that is not. Use tolerances, not exact trajectories.

#### Weight ratio and restart
- **D-08:** Control `weight-ratio`, labeled "Weight ratio", is a recreating preset (`kind: "preset"`, `recreates: true`, the Dam Break `water-amount` pattern). Values are `1.2` (1.2×), `1.5` (1.5×), `2` (2×, default) and `3` (3×). The Rust builder reads it from the presets, rejects unknown values, and falls back to the default when it is absent. Picking a ratio restarts the run.
- **D-09:** Restart uses the shared "Reset scene" button (`web/src/components/CanvasStage.tsx`) and the recreating preset. Add no new action. A flip-tank action is RT-03 (future).
- **D-10:** The scene has no pointer interaction and no live control. `apply_pointer`, `apply_action` and `apply_control` keep the default hooks, which reject unknown names.

#### Geometry, color and frame
- **D-11:** Use toy meter scale. A closed rectangular tank with thick, overlapping static walls and a lid, roughly as wide as it is tall or a little taller, with the two layers about equal in depth. Keep the scene at or below about 3,000 particles and about 1.5 ms per step in native release. Keep all geometry within about 10 m of the origin. Exact sizes and particle radius are Claude's discretion and come from the spike.
- **D-12:** The heavy layer is a warm orange and the light layer a deep blue, chosen so they contrast with each other and with the `#071018` canvas after the 0.73 density-shade floor. Check Shaded blob (default), Water, circle wireframe (README) and the static preview before the phase closes, and record what was seen.
- **D-13:** The landscape `viewBounds` frames the whole tank. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the tank with its walls, so it fills a phone canvas (about 390 by 844 CSS pixels). A tall tank suits the phone frame.

#### Catalog, caption and cues
- **D-14:** Use id `rayleigh-taylor`, module `rayleigh_taylor`, variant `RayleighTaylor`, title "Rayleigh–Taylor Instability" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the ferrofluid in catalog order. Credits say "original experimental scene", and the source is `sceneSource("rayleigh_taylor.rs")`.
- **D-15:** The caption names the fake plainly. For example: "A heavy orange layer rests on a light blue one. The orange layer is given extra weight, so a small ripple grows into fingers that sink while blue bubbles rise, and the layers turn over." Write a hint, for example "Watch the fingers form. Pick a Weight ratio or press Reset scene to run it again." Never say "density 2×" as if it were simulated, and describe growth qualitatively.
- **D-16:** The README plan is watch-first with no cue (`{ id: "rayleigh-taylor", cues: [] }`), so the 10 s clip shows the flat start, the fingers and the overturn. The survey entry is `SurveyCue::Default`. The demo-media `SCENE_CAPTURE_PLANS` entry follows the watch-first screw precedent. The scene stays in the e2e watch-first loop and gets no `POINTER_CONTROL` entry.
- **D-17:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed; commit only this scene's new WebP and SVG. The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

#### Tests and evidence
- **D-18:** Native release-mode tests live in the existing `liquidfun-wasm` lib test binary, one concern per test:
  - At the default ratio and gravity 10, at least 3 fingers pass the 20% depth line by about 4 s.
  - At the default ratio and gravity 10, the layers overturn by about 10 s.
  - Two runs at the same settings give the same end state, checked at every ratio value.
  - The extra weight follows a tilted gravity vector: with gravity turned sideways, the heavy group's centroid moves along the new gravity direction relative to the light group.
  - No extra weight is applied during the settle window.
  - A 60 s run at ratio 3 has no eviction, no failed step and a bounded max speed.
  - Stability smokes at gravity 2 and 80 have no eviction and no failed step. Fingers are not required at gravity 80, because the pressure cap squashes columns (Phase 46 D-07a lesson).
  - The preset parser accepts each listed ratio and rejects unknown values.
- **D-19:** Before the phase closes, `just scene-fingerprint-gate` must report all 28 protected ids unchanged with `rayleigh-taylor` under "added (accepted)". Record `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene rayleigh-taylor` in the phase summary.
- **D-20:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 29 protected ids unchanged.
- **D-21:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player and hud-safe-frame), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact tank size, particle radius and count, layer depths, settle and ramp timings, perturbation modes and amplitudes, and the flag candidate, within D-03 to D-06 and D-11. The spike evidence picks them.
- Exact color hex values, within D-12.
- Preview drawing, within the curiosity preview pattern.
- Plan breakdown and wave ordering, provided the spike comes first and the baseline growth commit (D-20) is last and separate.
- Whether to split `rayleigh_taylor.rs` into `rayleigh_taylor/` submodules (for example `geometry.rs` and test files, the `foo.rs` + `foo/` layout) to stay under the file-length limit.

### Deferred Ideas (OUT OF SCOPE)
- Perturbation presets (single mode, few modes, noise) and a flip-tank action (RT-03, future requirement).
- An interface tension or viscosity toggle that shows short-wave suppression (research differentiator).
- An Atwood number readout in the hint or label.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RT-01 | Player sees a heavy liquid layer above a light one develop into fingers that sink through the light liquid. The layers overturn within about 10 seconds. | § Engine Facts (group ranges are contiguous by construction, so one range call per step works), § Scene Design (explicit-lattice two-group build, deterministic polynomial seed, settle/ramp), § Finger and Overturn Metrics (rain-robust bin rule), Pitfalls 1–4 |
| RT-02 | Player can choose the heavy-to-light weight ratio and restart the run. The same settings always produce the same pattern. | § Preset and Restart Plumbing (session `apply_control` → `Recreated` is mandatory; `DEFAULT_PRESET_VALUES` entry; Reset clears the ratio), § Determinism, Pitfalls 5–7 |
| SCENE-01..03 (shared) | Catalog record, preview, portrait frame, README preview, legend, gravity slider, HUD safe frame, ≤ ~3,000 particles, honest caption | § Site Map, § Color, Pitfalls 8–11 |
| PREP-01 (shared gate) | Every protected fingerprint unchanged (28), new id added | § Fingerprint Gate |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Follow `docs/adding-a-scene.md` for every Rust and web site; the fingerprint baseline grows only in its own commit. [VERIFIED: AGENTS.md, docs/adding-a-scene.md]
- No change to `crates/liquidfun`. No new dependencies, cue kinds, bridge APIs or worker messages. Continuous forces once per fixed step in `on_advance`, following `world.gravity()`. [VERIFIED: ROADMAP shared rules]
- Rust: no `unwrap()` in production; `?`/`map_err`; `expect()` only in tests; `maybe_` prefix for `Option`; `foo.rs` + `foo/` layout; tau not pi; `let...else`. Workspace clippy is `all` + `pedantic` at warn, run with `-D warnings`, so avoid `as` casts (use `f32::from(u16)`, `u16::try_from`). [VERIFIED: Cargo.toml `[workspace.lints.clippy]`, global CLAUDE.md]
- Tests: one concern per test, Arrange/Act/Assert comments. [VERIFIED: global CLAUDE.md]
- Files ≤ 628 lines (`bun scripts/bright-builds-check.ts all`). [VERIFIED: CONTEXT D-21]
- New Rust tests go in the existing `liquidfun-wasm` lib test binary (a new binary costs ~2 min of macOS scanning). [VERIFIED: memory "macOS launch stalls"]
- `.planning/**` is never mdformatted; run `just markdown-check` after non-GSD Markdown edits (README gallery). [VERIFIED: CLAUDE.md]
- Executors run sequentially on the main tree; SUMMARY files need `generated_by`, `lifecycle_mode`, `phase_lifecycle_id`, `generated_at`. [VERIFIED: user memory]
- `just readme-svg` needs `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf`; commit only this scene's SVG/WebP and `git checkout --` the rest; `~/.local/bin` first on PATH for mdformat. [VERIFIED: memory, file exists]
- High-gravity targets are measured relative to the settled surface (memory "High-gravity water collapse"). [VERIFIED: memory]
- Hobby project: keep plans lightweight. [VERIFIED: PROJECT-SCOPE via CLAUDE.md, memory "Prefer lightweight process"]

## Summary

Everything the scene needs already exists and was read in this session. `particle_force::apply_group_acceleration(world, system, group, scale, acceleration)` reads `particle_group_view(group).member_ids()`, multiplies the per-particle force by the member count, and makes one `apply_particle_force_range` call. The contiguity requirement of that call is **always met for a group**: the storage keeps each group as a `first..last` row range (`particle/storage/group.rs:163`), and `member_ids()` is literally `dense_to_id[range]` (`particle/storage/creation.rs:177, 226`), so the ids are in row order and contiguous by construction. The existing unit test `group_acceleration_reaches_every_member_once` (`particle_force/tests.rs:188`) already proves the calibration to 1e-3. The function carries `#[cfg_attr(not(test), expect(dead_code, ...))]`; once the scene calls it, that `expect` becomes unfulfilled and fails `-D warnings`, so the attribute must be removed **in the same commit** that adds the caller.

The one plumbing fact that corrects a CONTEXT reading: **a recreating preset is not handled by the builder alone.** The web applies a chosen ratio (and replays it after every rebuild) by calling `session.apply_control("weight-ratio", "3")` (`web/src/player/scene-lifecycle.ts:166-177`, `worker-controls.ts:37`). `SessionCore::apply_control` intercepts only `gravity`; everything else goes to `hooks.apply_control`, and only a `ControlEffect::Recreated` return stores the preset and rebuilds through `build_scene(presets)` (`session.rs:317-342`). So `RayleighTaylorHooks::apply_control` must accept `weight-ratio` with a valid token and return `Ok(ControlEffect::Recreated)`, exactly as Dam Break does for `water-amount` (`dam_break.rs:247-268`). It rejects every other name. D-10's "no live control" stays true. Also, `apply_pointer` must return `Ok(())` and ignore input, as the screw does (`archimedes_screw.rs:276-286`); an `Err` would fail the scene in the browser when someone taps the canvas. Second correction: the shared "Reset scene" button calls `recreateScene`, which **clears `constructionValues`** (`scene-lifecycle.ts:249-257`). So Reset restarts at the default 2× and gravity 10, not at the chosen ratio. The preset is a native `<select>` whose `onChange` fires only on a different value, so the hint must not promise that Reset reruns the chosen ratio.

Build the two layers from **one explicit lattice** (`ParticleGroupSource::positions`, the Hydraulic Fountain and Wave Tank pattern) split by a seeded interface height `y_i(x)`. That gives no gap or overlap at the interface and an exact seed. Compute the seed with a **polynomial periodic wave**, not `sin`. Native and wasm libm may differ in the last bit (the v1.5 D-12 and Phase 48 Pitfall 4 lesson), and a one-ulp difference in an initial position at a lattice threshold can flip a particle between groups. The seed runs once at build, so the cost is nil. The fingers then come from the shared pressure solver: both groups have equal mass, and only the heavy group gets extra weight. No probe was run (the orchestrator asked for reads and greps only), so every geometry and tuning number below is a derived starting point for the D-06 spike, tagged as such.

**Primary recommendation:** Plan 1 builds the scene core: a tank with a lid, a two-group lattice split by a polynomial seed, hooks with a step counter for settle and ramp, and `apply_group_acceleration` on the heavy group with the `expect(dead_code)` removed. It also builds the `weight-ratio` parse in `build` and `apply_control`, plus a `test_support` with the rain-robust finger metric and the overturn metric. A release-only sweep test drives a `Tuning` struct through the D-05 flag candidates, and the plan summary records the D-06 table. Plan 2 adds the D-18 release tests. Plan 3 wires the catalog and web. Plan 4 runs README, colors, gates and scene-spot. Plan 5 is the single-file baseline growth.

## Standard Stack

No new dependencies (locked). All in-repo.

| Component | Version | Purpose | Source |
|-----------|---------|---------|--------|
| `liquidfun` (in-repo) | workspace | `ParticleGroupSource::positions`, `ParticleGroupRecipe::with_particle_flags/with_color`, `ParticleSystemDef::with_radius/with_viscous_strength/with_repulsive_strength/with_maximum_count`, `World::particle_system_view` (`positions`, `velocities`, `group_ids`), `particle_group_view(..).member_ids()`, `World::gravity` | [VERIFIED: codebase] |
| `liquidfun-wasm::scene::particle_force` | Phase 48 | `GravityFrame::from_gravity`, `ForceScale::for_system`, `apply_group_acceleration` | [VERIFIED: `particle_force.rs`] |
| Rust | 1.97.0 (`rust-toolchain.toml`) | build/test | [VERIFIED] |
| bun / just / ffmpeg / webpmux / dwebp / mdformat / DejaVu font | installed | web, recipes, README assets, Markdown | [VERIFIED: `command -v`, file checks this session] |

**Installation:** none.

## Engine Facts

| Fact | Evidence | Consequence |
|------|----------|-------------|
| `apply_group_acceleration` takes `(world, system, group, scale, acceleration)`, copies `member_ids()`, returns `Ok(())` for an empty group, multiplies by `u16`-checked count, and maps every error to `SessionError::StepFailed`. It is `#[cfg_attr(not(test), expect(dead_code, reason = "Phase 49 ..."))]`. | `particle_force.rs:142-179` | Call it once per step. Remove the `cfg_attr` in the same commit as the caller, or clippy `-D warnings` fails on the unfulfilled expectation [VERIFIED] |
| `GravityFrame::from_gravity(g)` → `None` for non-finite or `|g| ≤ 1e-4`; `up = −g/|g|`, `across = (up.y, −up.x)`. | `particle_force.rs:30-46` | Heavy acceleration = `frame.up * (-(extra · frame.magnitude · ramp))` [VERIFIED] |
| `ForceScale::for_system(world, system, iterations)` = `density·(PARTICLE_STRIDE·2r)²·iterations` from the live definition; `SceneConstruction` on failure. | `particle_force.rs:57-80` | Compute once at build with `super::particle_iterations(SceneId::RayleighTaylor)` (= 2, no R6 arm) [VERIFIED] |
| A group is stored as one contiguous row range `first..last`; `member_ids()` is `dense_to_id[range]`. | `particle/storage/group.rs:163-171`; `particle/storage/creation.rs:166-229` | Contiguity required by `resolve_contiguous_live_range` (`storage/runtime.rs:478-494`) always holds; no per-particle fallback needed [VERIFIED] |
| Range force rejects empty, non-contiguous, stale, `WALL` and non-finite input; divides the total by the count. | `particle/force.rs:72-91, 136-150` | Plain liquid groups are accepted [VERIFIED] |
| The particle view exposes `group_ids(): &[Option<ParticleGroupId>]` aligned with `positions()` and `velocities()`. | `particle/view.rs:111-116` | Tests split heavy and light particles by group id, not by color [VERIFIED] |
| `create_particle_group(system, &recipe)` returns the `ParticleGroupId`. | `particle_force/tests.rs:47` | Keep the heavy id in the hooks [VERIFIED] |
| `ParticleGroupSource::positions(Vec<Vec2>)` creates one particle per explicit position, in order; rejects empty or non-finite input. | `particle/group/source.rs:72-100`; used by `hydraulic_fountain.rs:312-347`, `wave_tank.rs:354`, `liquid_tumbler.rs:144` | Exact shared lattice for both groups [VERIFIED] |
| `REPULSIVE`: for each contact whose flags intersect `REPULSIVE` and whose particles are in **different groups**, `Δv = repulsive_strength · critical_velocity · weight · normal`. The default strength is **1.0**. `critical_velocity = 2r / substep` (9.6 m/s at r 0.04 and 2 iterations). | `particle/solver/material.rs:72-96, 264-266`; default `system_definition.rs:570` | The default repulsion is very strong. Start the third D-05 candidate at `with_repulsive_strength(0.05..0.2)`. Setting the flag on one group is enough, because contact flags are combined [VERIFIED code; ASSUMED contact flags are the OR of both particles' flags, as upstream] |
| Viscous strength default 0.25; the step fails above ~2.0. | `system_definition.rs:567`; PITFALLS engine table | D-05 caps it at 1.5 [VERIFIED] |
| Pressure is capped at `MAX_PARTICLE_PRESSURE (0.25) × critical_pressure`, with `critical_pressure = density·(2r/substep)²`. | `math/settings.rs:90`; `particle/solver/pressure.rs:35, 237-240` | Tall, heavily loaded columns squash. See Pitfall 4 [VERIFIED] |
| Session order per step: `on_advance` → eviction → `world.step` → `on_after_step`. Recreating controls rebuild through `from_source` with the stored presets. Tilt (`set_gravity`) does not rebuild. | `session.rs:218-268, 317-342, 196-206` | Step counter in hooks resets on any rebuild (ratio, gravity slider), not on tilt [VERIFIED] |
| `build_scene` strips the `gravity` preset before the scene builder sees the presets, and applies it after the build as `(0, −g)`. | `scene.rs:259-299`; `gravity_slider.rs:54-89` | `build(presets)` sees only `weight-ratio` (or nothing) [VERIFIED] |
| Default web draw cap is 4,000 particles. Above it the renderer strides. | `web/src/render/particle-limit.ts:5` | Stay ≤ 3,000 [VERIFIED] |
| Eviction: more than 12 m along gravity or 48 m otherwise from the origin. | `session/escape.rs:9-11, 26-41` | A tank within ±2 m × 0..4 m needs no R12 arm [VERIFIED] |
| The scene-spot fingerprint is taken after 60 warmup and 120 measured steps (3 s), over 3 runs. | `scene_spot.rs:17-21` | It captures the mid-instability state; any nondeterminism fails the survey [VERIFIED] |

## Scene Design (starting points for the spike)

### Geometry [ASSUMED: derived, spike confirms]
| Item | Start value | Why |
|---|---|---|
| Particle radius | **0.04 m** (stride 0.06); sweep 0.045 as the alternate | ~10 particles per 0.6 m wavelength; budget room |
| Tank inner | x ∈ [−1.4, 1.4] (2.8 m), floor top y = 0, lid underside y = 3.2 | "as wide as tall or a little taller" (D-11); fits phone |
| Walls, floor, lid | 0.3 m thick, overlapping at the corners (4 quads via `attach_basin_fixture`, `scene.rs:312-330`) | Thick walls convention; closed tank contains tilt sloshing |
| Fill | lattice from y = r up to y ≈ 3.0 (about 0.2 m headspace under the lid); interface mean y0 = 1.5 | Equal layers; headspace keeps the settle from pushing on the lid |
| Count | (46 columns) × (~50 rows) ≈ **2,300** particles | ≤ 3,000; `with_maximum_count(3_000)` as the cap. Hydraulic Fountain runs 3,200 at 0.76 ms and Liquid Bubbler 3,000 at 1.17 ms (`docs/benchmarks/scene-survey.md`), so expect ≈ 0.7–1.1 ms/step |
| 20% depth line | `y0 − 0.2 × 3.2` = 0.86 m (the "tank height" is the inner height) | D-06/D-07 |
| Frame (`FRAME_*`, mirrored by `RAYLEIGH_TAYLOR_VIEW_BOUNDS`) | x ±2.0, y −0.6 … 3.8 (outer walls + 0.3 m) | Landscape frames the whole tank (D-13) |
| Portrait | the same constant; width-bound on a phone, so the height fraction is ≈ (4.4/4.0)·(390/844) ≈ 0.51 → `MIN_HEIGHT_FRACTION` ≈ 0.48 | Like the diver, the frame is already tall |

### Seed (D-04): deterministic polynomial wave
```rust
/// Parabolic sine stand-in: odd, period 1, peaks ±1 at u = ±0.25. No libm.
fn wave(u: f32) -> f32 {
    let t = u - u.round();          // [-0.5, 0.5]
    8.0 * t * (1.0 - 2.0 * t.abs())
}
/// Interface height across the tank; heavy above, light below.
fn interface_y(x: f32, t: &Tuning) -> f32 {
    let s = (x + TANK_HALF_WIDTH) / (2.0 * TANK_HALF_WIDTH); // 0..1 across
    t.interface_mean
        - t.mode_amplitudes[0] * wave(f32::from(t.mode_counts[0]) * s - 0.25)
        - t.mode_amplitudes[1] * wave(f32::from(t.mode_counts[1]) * s - 0.25)
}
```
- Start at 4 troughs (`mode_counts[0] = 4`, amplitude **0.10 m**, about 1.7 strides) plus a 7-trough mode at 0.03 m. The `− 0.25` phase puts the troughs in the interior and the crests at the walls, so heavy fingers form away from the walls. [ASSUMED]
- Amplitudes below one stride (0.06 m) are quantized into a one-row staircase by the lattice and may read as "no seed". Keep the primary mode ≥ 1.5 strides. [ASSUMED]
- `wave` uses only `−`, `round`, `abs`, `*`, so native, wasm and debug produce bit-identical lattices. [VERIFIED: Phase 48 polynomial precedent]
- Lattice loop: integer row and column counters, `x = x0 + f32::from(col) * STRIDE` (do not accumulate `x += STRIDE`, so the bounds test stays exact). Assign a point to heavy if `y >= interface_y(x)`, else to light. Create the light group first (bottom rows first), then the heavy group. Either order works for contiguity.

### Hooks
```rust
struct RayleighTaylorHooks {
    heavy_group: ParticleGroupId,
    ratio: WeightRatio,          // parsed preset; owns the exact extra factor
    scale: ForceScale,           // from build
    steps_advanced: u16,         // saturating; reset by every rebuild
    tank_segments: Vec<RigidSegment>,
}

fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> Result<(), SessionError> {
    let step = self.steps_advanced;
    self.steps_advanced = step.saturating_add(1);
    let ramp = ramp_factor(step);                 // 0 for step < SETTLE_STEPS
    if ramp == 0.0 { return Ok(()); }            // clippy float_cmp: use `ramp <= 0.0`
    let Some(frame) = GravityFrame::from_gravity(world.gravity()) else { return Ok(()); };
    let extra = self.ratio.extra_weight() * frame.magnitude * ramp;
    particle_force::apply_group_acceleration(world, system, self.heavy_group, self.scale, frame.up * -extra)
}

/// 0 during the settle, then a linear ramp to 1 (D-03).
fn ramp_factor(step: u16) -> f32 {
    let Some(into_ramp) = step.checked_sub(SETTLE_STEPS) else { return 0.0; };
    (f32::from(into_ramp + 1) / f32::from(RAMP_STEPS)).min(1.0)
}
```
- Start with `SETTLE_STEPS = 30` (0.5 s) and `RAMP_STEPS = 18` (0.3 s). The spike sweeps settle 18/30/36 and ramp 12/18/24. [ASSUMED]
- `WeightRatio::{OnePointTwo, OnePointFive, Two, Three}` with `parse("1.2" | "1.5" | "2" | "3")` and `extra_weight()` returning the literals `0.2, 0.5, 1.0, 2.0`. Store the extra directly rather than computing `1.2 − 1.0`, so it is exact and obvious. [ASSUMED: names]
- `apply_control`: `"weight-ratio"` with a valid token → `Ok(ControlEffect::Recreated)`; anything else → `Err(UnknownControl)`. `apply_action` → `Err(UnknownControl)`. `apply_pointer` → `Ok(())` (screw precedent). `collect_circles` → empty.
- `build(presets)`: read `weight-ratio` with the Dam Break `preset_value` (last wins, `dam_break.rs:86-92`); absent → `Two`; unknown token → `Err(UnknownControl)`; any other preset name → `Err(UnknownControl)`.

### Tuning struct for the spike
Put every D-06 sweep knob in one `struct Tuning { radius, flags_candidate, viscous, repulsive, mode_counts, mode_amplitudes, settle_steps, ramp_steps }`, with a production `const TUNING: Tuning`. `build_parts(ratio, &TUNING)` lets the release spike test loop over candidates in one run and print a table (`--nocapture`). Once a candidate wins, commit it as `TUNING` and keep the spike tests as regression guards (Phase 48 spike precedent: `fake_ferrofluid/spike_tests.rs` with a `OnceLock` report). At about 1 ms/step, a 10 s run takes about 0.6 s. A sweep of about 50 runs finishes in about a minute. [ASSUMED: timing estimate]

### Flags (D-05 order)
1. `ParticleFlags::WATER` (0) on both groups, defaults otherwise.
2. `VISCOUS` on both, `with_viscous_strength(0.5..1.5)`.
3. `VISCOUS | REPULSIVE` (REPULSIVE on both is fine), `with_repulsive_strength(0.05..0.2)`. At the default 1.0 it is likely to blow the interface apart (Engine Facts). [VERIFIED code; ASSUMED visual effect]

Never `TENSILE` (zero normal pressure), `COLOR_MIXING`, or `ParticleGroupFlags::SOLID` (upstream solid groups eject other groups' particles, which would block the fingers). [VERIFIED: pressure.rs and the TENSILE rule; ASSUMED: SOLID behavior from upstream semantics]

## Finger and Overturn Metrics (test_support.rs)

Gravity is straight down in the D-06/D-18 finger tests, so `up = +y` and `across = +x`. Write the helpers on `(up, across)` anyway, so the tilt test can reuse the centroid helper.

1. **Split lanes:** `session.read_particles(|world, system| …)` → view `positions()`, `velocities()`, `group_ids()`. A particle is heavy when `group_ids()[i] == Some(heavy_group)` (`SessionCore::create(SceneId::RayleighTaylor)` hides the hooks, so in session tests get the heavy id as the group of a particle known to start at the top, or classify by `up` height at step 0 and keep the ids). Simpler: run spike and flow tests on `build_parts` directly (Phase 48 spike pattern), where the hooks and `heavy_group` are visible, and use `SessionCore` only for the eviction and preset tests. [VERIFIED: APIs; ASSUMED: approach]
2. **Bins:** width 2d = 0.16 m across the inner tank (17 bins); drop the one bin touching each wall, so heavy liquid sliding down a wall does not count as a finger.
3. **Rain filter (important):** D-07 says "a bin whose deepest heavy particle is past the line". A single raining particle would satisfy that literally (Pitfall 1). Count a bin as **penetrating** only when at least `MIN_FINGER_PARTICLES` (start at 4) heavy particles in it lie below the 20% line. That is the deepest heavy particle past the line, made robust. Record it as a stated interpretation of D-07 in the plan.
4. **Fingers:** maximal runs of penetrating bins, separated by at least one non-penetrating bin. **Pass:** the first sampled step (every 6 steps) with ≥ 3 fingers is ≤ 240 (4 s).
5. **Overturn:** `mean(heavy · up) < mean(light · up)`. **Pass:** the first sampled step ≤ 600 (10 s) at ratio 2; record the step at 1.2 (≤ 1,200, 20 s), 1.5 and 3.
6. **Rain diagnostic (spike only, not an assertion):** the share of heavy particles below the line with fewer than 2 heavy neighbors within 3r, at the 4 s sample. A high share with "fingers" passing means it is rain, not fingers. Look at the frame.
7. **Max speed:** reuse the `max_speed` pattern from `fake_ferrofluid/test_support.rs` (it is `pub(super)` there, so copy the 6 lines). Bound it at g 10 to ≤ 4.0 m/s (clamp 9.6). [ASSUMED: threshold; same as Phase 48]
8. **Eviction:** `session.live_particle_count()? == session.particle_count()` after a session run (`session.rs:555-562`).
9. **Determinism:** compare `(x.to_bits(), y.to_bits())` of positions and velocities after 180 steps for two fresh sessions, at each of the 4 ratios. `scene_spot::fingerprint` is `pub(super)` (`scene_spot/fingerprint.rs:51`) and not reachable. The bit-vector comparison is stronger anyway (`particle_force/tests.rs:216-237` pattern).

### D-18 test shapes
| Test | Kind | Shape |
|---|---|---|
| ≥ 3 fingers by ~4 s | release | `build_parts(Two)`, step to 240 sampling every 6 |
| overturn by ~10 s | release | same run to 600 |
| same end state at every ratio | release | 4 × two sessions with `apply_control("weight-ratio", v)`, 180 steps, bit-compare |
| extra weight follows tilted gravity | release (or debug-safe twin) | Debug-safe twin version: two `build_parts` at ratio 2, set `steps_advanced` past the ramp in both, set gravity `(7.071068, −7.071068)` in both, call `on_advance` on one, step both once, and assert the heavy mean Δv minus the twin's is `extra·dt` along `−up` within 1e-3 (the calibration-test shape). Release behavior version: after settling, set gravity to `(−10, 0)` and assert `(c_heavy − c_light)·ĝ` grows by ≥ 0.2 m within 2 s |
| no extra weight during settle | debug-safe | twin worlds, `on_advance` + step for `SETTLE_STEPS` on one and step only on the other → velocities bit-identical |
| 60 s at ratio 3 | release | 3,600 steps, no failed step, `live == built`, max speed bounded |
| g 2 / g 80 smokes | release | `apply_control("gravity", "2"|"80")`, 20 s; also run **g 80 at ratio 3** (heavy feels 240 m/s²), the worst case (Pitfall 4) |
| preset parser | debug | each token accepted (`Recreated`), `"2.0"`, `"4"`, `""`, `"1,2"`, `" 2"` rejected, unknown preset name rejected by `build` |

Gate release-only tests with `#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]` and run them with `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor`.

## Preset and Restart Plumbing (RT-02)

| Layer | Fact | Site |
|---|---|---|
| Web record | Inline object `{ id: "weight-ratio", label: "Weight ratio", kind: "preset", recreates: true, values: [option("1.2","1.2×"), option("1.5","1.5×"), option("2","2×"), option("3","3×")] }`. `runtimePreset()` hard-codes `recreates: false`, so do not use it. Controls: `withGravitySlider([WEIGHT_RATIO_CONTROL])` | `scene-record-shared.ts:149-156`; Dam Break inline at `scene-records-classic.ts:47-57` [VERIFIED] |
| Default | `DEFAULT_PRESET_VALUES` must gain `"weight-ratio": "2"`, because the default is not the first value | `web/src/components/scene-controls.ts:7-20, 84-92` [VERIFIED] |
| Load | The web applies only recreating values that the user chose (`constructionEntriesForScene`); with none chosen, Rust's default must equal `"2"` | `web/src/player/runtime.ts:56-72` [VERIFIED] |
| Change | `owner.applyControl(name, value)` → Rust `hooks.apply_control` → `Recreated` → presets stored, world rebuilt | `worker-controls.ts:25-60`; `session.rs:317-342` [VERIFIED] |
| Reset | `recreateScene` clears `constructionValues` → default ratio 2 and gravity 10 | `scene-lifecycle.ts:249-257` [VERIFIED] |
| Widget | native `<select>`; `onChange` fires only on a new value | `components/SceneControls.tsx:37-50` [VERIFIED] |

Hint consequence: say "Pick a Weight ratio to run it again with that ratio, or press Reset scene to start over." Do not say that Reset keeps the ratio. The hint must also end with `Labeled controls also work from the keyboard.` (Pitfall 9).

Optional R14 sync test (`include_str!` of `scene-records-curiosity.ts` and `scene-controls.ts`): the four option ids equal the Rust allowlist, and the TS default equals the Rust default token. This uses the `fake_ferrofluid/sync_tests.rs` helper pattern.

## Determinism

- No RNG, no time, no `HashMap`, no libm in per-step or seed code. The per-step math is one multiply chain on `f32` (ratio · |g| · ramp). `GravityFrame` uses `length()` (sqrt, correctly rounded). [VERIFIED: code]
- The same binary on the same machine reproduces bits, so the gate passes. Native vs wasm drift can only come from libm, and the polynomial seed removes the only candidate. [VERIFIED: Phase 48 lesson]
- Step counters are integers; `ramp_factor` uses `f32::from(u16)` division, which is exact IEEE. [VERIFIED]

## Color (D-12)

WCAG contrast vs `#071018` (unshaded / at the 0.73 floor), computed this session:

| Candidate | Role | Unshaded | Floor |
|---|---|---|---|
| **`#FB923C` (251, 146, 60)** | heavy orange | 8.46 | 4.70 |
| `#F97316` (249, 115, 22) | orange alt | 6.83 | 3.89 |
| `#F2B040` | (already `DRIP` amber, avoid) | 10.09 | 5.47 |
| **`#3B82F6` (59, 130, 246)** | light blue | 5.21 | 3.10 |
| `#2563EB` | deeper blue | 3.71 | 2.34 (below 3) |
| `#1D4ED8` | deep blue | 2.86 | 1.93 (too dark) |
| `#4DA3FF` | shared water (avoid, reads as "water") | 7.30 | 4.12 |

Start with heavy `#FB923C` and light `#3B82F6`. Their mutual luminance contrast is low (1.63 unshaded, 1.52 at the floor), so the layers separate by hue. Orange and blue are complementary, and the standard color-vision-deficiency-safe pair. Add `RT_HEAVY` and `RT_LIGHT` to `preview-kit.tsx` for the static preview. The README wireframe strokes per-particle colors (`web/src/export/animated-svg.ts:224-237`), so thin blue rings on black are the weakest case: check it. [VERIFIED: computed; ASSUMED: 3:1 bar and hue legibility]

## Site Map (exact sites, current line numbers)

### Rust (`crates/liquidfun-wasm/src`)
| # | Site | Edit |
|---|------|------|
| R1 | `scene/rayleigh_taylor.rs` + `rayleigh_taylor/{geometry,test_support,tests,spike_tests,flow_tests}.rs` | New scene (keep the top-level file the credit target) |
| R2 | `scene.rs:9-40` | `mod rayleigh_taylor;` (alphabetical, after `particles`/`rigid_particles`) |
| R3/R5 | `scene.rs:49-78` `SceneId`; `scene.rs:259-299` `build_scene` | `RayleighTaylor` + `SceneId::RayleighTaylor => rayleigh_taylor::build(&scene_presets)` |
| R4 | `scene.rs:115-149` `parse_scene_id` | `"rayleigh-taylor" => Ok(SceneId::RayleighTaylor)` |
| R6 | `scene.rs:151-161` | None (2 iterations) |
| R7 | `scene.rs:84-113` `ALL_SCENE_IDS` (append last, ships with web `SCENE_IDS`); `scene/gravity_slider.rs:161-191` `variant_index` → `SceneId::RayleighTaylor => 28` | Order test `scene.rs:373-389` |
| R8 | `session/tests.rs:14-44` | Append `("rayleigh-taylor", SceneId::RayleighTaylor)` |
| R9 | `scene_spot.rs:46-120` `SURVEY_SCENES` | `("rayleigh-taylor", SceneId::RayleighTaylor, SurveyCue::Default)` |
| R10 | `scene_spot/tests.rs` | No change (Default cue) |
| R12 | `session/escape.rs` | Not needed |
| R13 | `spike_tests.rs`, `flow_tests.rs` | Release-only |
| R14 | `rayleigh_taylor/tests.rs` or `sync_tests.rs` | `FRAME_*` vs `RAYLEIGH_TAYLOR_VIEW_BOUNDS` f32 bits; ratio tokens and default vs TS |
| R15 | `scene/particle_force.rs:153-159` | Remove the `cfg_attr(not(test), expect(dead_code, ...))` on `apply_group_acceleration` |

Plan 1 registers only R2, R3/R5, R4, R8 and the `variant_index` arm (exhaustive match). `ALL_SCENE_IDS` and `SURVEY_SCENES` must ship in the same commit as web `SCENE_IDS`, because `all_scene_ids_match_the_web_catalog_order` reads `web/src/catalog/scenes.ts` (Phase 48 Plan 1/3 precedent). [VERIFIED]

### Web
| # | Site | Edit |
|---|------|------|
| W1 | `web/src/catalog/scenes.ts:14-43` `SCENE_IDS` | Append `"rayleigh-taylor"`; add `RAYLEIGH_TAYLOR_VIEW_BOUNDS` to the re-export block (`scenes.ts:101-111`) |
| W2 | `scene-records-curiosity.ts` (74 lines) | Record after the ferrofluid: D-15 caption + "This is an original experimental scene.", the hint + keyboard reminder, `withGravitySlider([WEIGHT_RATIO_CONTROL])`, `viewBounds: RAYLEIGH_TAYLOR_VIEW_BOUNDS`, `sceneSource("rayleigh_taylor.rs")`, `[SHOWCASE]` |
| W3 | none (or a tiny `rayleigh-taylor-controls.ts` exporting `WEIGHT_RATIO_CONTROL` for test import) | Recommended: a separate file, so tests and the record share one constant |
| W4 | `web/src/components/scene-controls.ts:7-20` | `"weight-ratio": "2"` |
| W5 | `previews-curiosity.tsx` `RayleighTaylorPreview()`; `previews.tsx:11-15, 100-102` (`never` guard) | Tank outline, orange over blue with 3–4 orange fingers and blue bubbles between |
| W6 | `scene-view-bounds.ts` `RAYLEIGH_TAYLOR_VIEW_BOUNDS` → `scene-records.ts` → `scenes.ts` re-export; `portrait-bounds.ts:84-104` | Landscape = portrait (the same constant) |
| W7 | `range-control.ts` | Not needed |
| S1 | `web/scripts/readme-svg/plans.ts:84-98` | `{ id: "rayleigh-taylor", cues: [] }` |
| S2 | `docs/assets/readme/rayleigh-taylor-10s.{svg,webp}`, README gallery | `just readme-svg`, review frames with `webpmux -get frame N` + `dwebp`, `just markdown-check` |
| S3 | `web/scripts/demo-media/model.ts:296-312` | `{ id: "rayleigh-taylor", title: "Rayleigh–Taylor Instability", route: "/liquidfun-rs/#/scene/rayleigh-taylor", interactionStep: 180, action: { kind: "click", point: { x: 0.5, y: 0.5 } } }` (screw precedent; the click is ignored by `apply_pointer`) |
| T1 | `web/tests/scenes.test.ts` (587 lines): `UI_SPEC_DESCRIPTIONS` (~l.71), `UI_SPEC_HINTS` (~l.120), id list (~l.220), title list (~l.262) | +6 lines. Put the honesty guard and the control-shape test in a **new** `web/tests/rayleigh-taylor.test.ts` (Pitfall 11) |
| T2 | `web/tests/scene-catalog-controls.test.ts:28-63` | Append `"weight-ratio", "gravity"` |
| T3 | `web/tests/scene-catalog-credits.test.ts:99, 219` | Path and `[SHOWCASE_HREF]` |
| T4 | `web/tests/portrait-bounds.test.ts:45-77` `MIN_HEIGHT_FRACTION["rayleigh-taylor"]` (≈ 0.48 for the nominal frame); wall points (~l.420): inner and outer corners of floor and lid | |
| T6 | `controls.test.ts` | Not needed (no HUD control) |
| E1 | `web/e2e/player-helpers.ts:58` | `"rayleigh-taylor": "/liquidfun-rs/#/scene/rayleigh-taylor"` |
| E2 | `web/e2e/player.spec.ts:40-59` | **No entry** (D-16): the scene falls into the watch-first pause, play and reset loop (`player.spec.ts:340-376`) |

**Already derived (no edit):** routing, sidebar, `ScaleLegend`, gravity slider plumbing, `hud-safe-frame.spec.ts`, shell counts, xtask scene list, `PAGE_SUMMARY`. [VERIFIED: docs/adding-a-scene.md]

## Fingerprint Gate

- Baseline `docs/benchmarks/scene-fingerprints.json` protects **28** ids, `fake-ferrofluid` last (`9f1e59fce8af89e7`). [VERIFIED: jq this session]
- The only shared-code change is deleting an attribute on `apply_group_acceleration`, so no protected scene's math changes. Do not touch `particle_force` behavior. [VERIFIED]
- Gate: `just scene-fingerprint-gate` → "protected: 28/28 unchanged", "added (accepted): rayleigh-taylor". Then the D-20 single-file commit, then 29/29.
- `cargo xtask playground scene-spot --scene rayleigh-taylor` → `start_particles` = `end_particles` ≈ 2,300, `median_ms_per_step` ≈ 0.7–1.1 [ASSUMED estimate].

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Group-wide force | A per-particle loop | `particle_force::apply_group_acceleration` | One validated range call; calibrated to 1e-3 by an existing test |
| Gravity direction | `Vec2::new(0, -1)` or trig | `GravityFrame::from_gravity(world.gravity())` | Tilt and slider correct (D-02) |
| Mass/iteration factor | Copied constants | `ForceScale::for_system` | Reads the live definition |
| Preset lookup | Ad-hoc iteration | Dam Break `preset_value` (last wins) | Matches `store_preset` replace semantics |
| Two layers | Two filled boxes plus a wavy polygon | One explicit lattice split by `interface_y` | No interface gap or overlap; exact seed; polygons cap at 8 vertices |
| Eviction check | Hooking `escape.rs` | `live_particle_count()` vs `particle_count()` | Eviction only happens in `SessionCore::advance` |
| Web/Rust constant sync | Manual review | R14 `include_str!` f32-bits test (`fake_ferrofluid/sync_tests.rs`) | Catches drift |
| Sine seed | `f32::sin` | `wave()` polynomial | Native/wasm bit parity |

## Common Pitfalls

### Pitfall 1: Rain passes the finger metric
**What goes wrong:** Single heavy particles percolate through the light lattice. A literal "deepest heavy particle past the line" bin rule counts them as fingers, so the test passes while the scene shows noise (PITFALLS 4). **Avoid:** require ≥ 4 heavy particles below the line per bin, drop the wall bins, and record the rain diagnostic in the spike. Look at a mid-run frame before accepting a candidate. **Warning signs:** fingers ≥ 3 at 1–2 s with no visible structure; the heavy mean height falls linearly.

### Pitfall 2: The settle wave or a too-small seed hides the mode
The explicit lattice starts at stride spacing and compresses under gravity. The first ~0.3 s sends a pressure wave through the column. Ramping the extra weight in after that keeps the wave from stirring the interface. A seed below one stride is quantized away. **Avoid:** settle ≥ 0.3 s, primary amplitude ≥ 1.5 strides, and the ramp. **Warning signs:** finger positions that do not match the seeded troughs.

### Pitfall 3: `expect(dead_code)` turns into a build failure
Adding the caller without deleting `#[cfg_attr(not(test), expect(dead_code, ...))]` (`particle_force.rs:153-159`) leaves an unfulfilled `expect`, which `-D warnings` rejects. Delete it in the same commit.

### Pitfall 4: The pressure cap at high load
The pressure is capped at 0.25 × critical pressure, and critical pressure ∝ (2r/substep)². Phase 46 (r 0.05) measured a 5.4 m column at g 80 settling to 1.7–2.9 m, which puts the cap near g·h ≈ 140–230 m²/s². Scaled by r² to r 0.04, that is roughly 90–150 [ASSUMED: scaling]. The light layer carries `g·h_light + ratio·g·h_heavy`: about 54 at ratio 2 and 72 at ratio 3 with the nominal 1.5 m layers at g 10. That is under the estimate, but ratio 3 is not far from it. So the bottom of the tank will pack visibly at ratio 3 (density-shade darkening). At g 80 every ratio squashes (≥ 360). **Avoid:** keep the column ≤ ~3.2 m at r 0.04, record the settled light-layer thickness at ratio 3 in the spike, and assert only stability at g 80, including g 80 × ratio 3. Do not raise particle iterations to fix this (Phase 46: that changes low-gravity behavior and cost).

### Pitfall 5: `apply_control` must return `Recreated`, not reject
D-10's "default hooks reject unknown names" holds for unknown names only. Without a `weight-ratio` → `Recreated` arm, the web cannot apply a ratio at all: the session never stores the preset, so `build` never sees it. **Test:** `session.apply_control("weight-ratio", "3")` returns `Ok(true)`, and the rebuilt hooks report ratio 3.

### Pitfall 6: Reset does not rerun the chosen ratio
Reset clears all construction values (`scene-lifecycle.ts:249-257`). The hint must not imply that it keeps the ratio. The e2e watch-first loop clicks Reset and only checks the step index near zero, which still passes.

### Pitfall 7: Determinism leaks
Watch for `f32::sin`/`cos` in the seed, HashMap ordering, or a hash seeded from time. Also do not reorder group creation between runs (it is constant code). The survey fails on any run-to-run difference.

### Pitfall 8: Captions claiming density
SCENE-03 forbids "simulated density". The control label is "Weight ratio", not "Density ratio". In `rayleigh-taylor.test.ts`, assert that the lowercased description and hint contain none of `["density", "denser", "atwood", "growth rate", "measured", "simulated", "real fluid"]` and that the description contains `"given extra weight"`. Check that none of these words appear in the locked copy. The D-15 example has none. [ASSUMED: word list; UI-SPEC finalizes]

### Pitfall 9: The hint needs the keyboard reminder
The scene has a labeled control and is not in the unit-test `WATCH_FIRST_SCENE_IDS` (the `WATCH_FIRST_HINT` scenes, `scenes.test.ts:125-136`). Every other hint must contain `Labeled controls also work from the keyboard.` (`scenes.test.ts:323-343`). The D-15 example lacks it, so append it (the Archimedes Screw precedent). Do not use `WATCH_FIRST_HINT`.

### Pitfall 10: Fast fingers whiten in Water mode
Water mode adds foam above 3.5 m/s (Phase 48 noted white specks). Fingers at ratio 3 or high gravity may cross it. Record what the four-mode check shows. It is not a blocker.

### Pitfall 11: File-length pressure
`scenes.test.ts` is 587 lines, so keep the edits there to the record entries (+6). `player-helpers.ts` (583) and `session/tests.rs` (583) gain one line each, and `scene-catalog-controls.test.ts` (448) gains two. Put the new controls and honesty tests in `web/tests/rayleigh-taylor.test.ts`. Split `rayleigh_taylor.rs` into `geometry.rs` (tank quads, segments, lattice, `wave`, `interface_y`, `FRAME_*`), `test_support.rs` and the test files. Run `bun scripts/bright-builds-check.ts all`. [VERIFIED: `wc -l`]

### Pitfall 12: Tooling on this Mac
`LIQUIDFUN_README_FONT_FILE` must be set for `just readme-svg`. Restore the other scenes' re-encoded WebPs with `git checkout --`. Use the pipx mdformat. Extract review frames with `webpmux` and `dwebp` (ffmpeg cannot decode animated WebP). [VERIFIED: memory]

## Code Examples

### Build (sketch)
```rust
pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if presets.iter().any(|(name, _)| name != WEIGHT_RATIO_CONTROL) {
        return Err(SessionError::UnknownControl);
    }
    let ratio = preset_value(presets, WEIGHT_RATIO_CONTROL)
        .map(WeightRatio::parse)
        .map_or(Ok(WeightRatio::DEFAULT), |maybe| maybe.ok_or(SessionError::UnknownControl))?;
    let parts = build_parts(ratio, &TUNING).map_err(|_error| SessionError::SceneConstruction)?;
    Ok(BuiltScene { world: parts.world, particle_system: parts.particle_system,
        particle_radius: TUNING.radius, hooks: Box::new(parts.hooks) })
}
```

### Two groups from one lattice
```rust
let (light, heavy): (Vec<Vec2>, Vec<Vec2>) =
    geometry::lattice(tuning).partition(|p| p.y < geometry::interface_y(p.x, tuning));
create_layer(world, system, light, LIGHT_COLOR, tuning.flags())?;
let heavy_group = create_layer(world, system, heavy, HEAVY_COLOR, tuning.flags())?;
// create_layer: ParticleGroupSource::positions(v) -> ParticleGroupRecipe::new(.., New)
//   .with_particle_flags(flags).with_color(color).with_transform(Transform::IDENTITY)
//   -> world.create_particle_group(system, &recipe) -> ParticleGroupId
```

### Web record
```ts
export const WEIGHT_RATIO_CONTROL: SceneControl = {
  id: "weight-ratio", label: "Weight ratio", kind: "preset", recreates: true,
  values: [option("1.2", "1.2×"), option("1.5", "1.5×"), option("2", "2×"), option("3", "3×")],
};
// record.controls: withGravitySlider([WEIGHT_RATIO_CONTROL])
// DEFAULT_PRESET_VALUES["weight-ratio"] = "2"
```

## Recommended Plan Breakdown

| Plan | Content | Risk |
|---|---|---|
| 49-01 | **Scene core + time-boxed spike.** Geometry (tank + lid, `FRAME_*`, segments), lattice + `wave`/`interface_y`, two groups, `Tuning`, hooks (`steps_advanced`, ramp, `apply_group_acceleration`), R15 attribute removal, `WeightRatio` + `build` preset parse + `apply_control` `Recreated`, `apply_pointer` `Ok`, `test_support` (split lanes, bins, rain-robust fingers, overturn, max speed, rain diagnostic). Register R2/R3/R4/R5/R8 + `variant_index` only. Release sweep against the D-06 rules in D-05 order at ratios 1.2/2/3, g 10; 60 s at ratio 3; double-run bits. Record the table and the decision in the SUMMARY; decision checkpoint if nothing passes | MEDIUM–HIGH (visual: rain vs fingers) |
| 49-02 | D-18 tests: fingers, overturn, determinism at all 4 ratios via the session preset, tilt (twin + behavior), settle window, 60 s ratio 3, g 2 / g 80 (+ g 80 × ratio 3) smokes, parser allowlist | LOW |
| 49-03 | Catalog: `ALL_SCENE_IDS`/`SURVEY_SCENES` + `SCENE_IDS`, record, control constant, `DEFAULT_PRESET_VALUES`, preview, view/portrait bounds, README/demo plans, T1–T4, `rayleigh-taylor.test.ts`, E1, R14 sync | LOW |
| 49-04 | `just readme-svg` + clip review (flat start, fingers by ~4 s, overturn by 10 s), four-mode color check, full gates, scene-spot numbers, `just scene-fingerprint-gate` (28/28 + added) | LOW |
| 49-05 | Single-file baseline growth, gate 29/29 | LOW |

Plans 01 and 02 can merge if the spike passes on the first candidate (a hobby-scale choice). Execute sequentially on the main tree. UI-SPEC (`ui_phase: true`) finalizes the copy, colors and preview before Plan 3.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| "Read members each step; maybe non-contiguous" (PITFALLS 3) | Groups are contiguous by storage design; one range call is always valid | Verified this research | No fallback path needed |
| Sinusoidal seed (D-04 wording) | Parabolic periodic polynomial `wave` | Phase 48 polynomial lesson | Native/wasm bit parity of the initial state |
| "Restart via Reset or preset" | Reset clears the ratio; only the preset reruns a chosen ratio | Verified this research | Hint wording |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Starting geometry (r 0.04, 2.8 × 3.2 m inner tank, ~2,300 particles, 1.5 m layers) gives fingers by 4 s | Scene Design | The spike retunes; a decision checkpoint if no candidate passes |
| A2 | Settle 30 / ramp 18 steps; seed 4 troughs at 0.10 m + 7 at 0.03 m | Scene Design | Spike sweeps them |
| A3 | Default `REPULSIVE` strength 1.0 is too strong; start at 0.05–0.2 | Flags | Only candidate 3 is affected |
| A4 | Contact flags are the OR of both particles' flags (upstream semantics) | Engine Facts | Then set REPULSIVE on both groups (recommended anyway) |
| A5 | `ParticleGroupFlags::SOLID` would block fingers | Flags | Not used either way |
| A6 | Pressure-cap estimate ≈ 90–150 m²/s² at r 0.04 (scaled from Phase 46) | Pitfall 4 | Ratio 3 may pack more than expected; the spike records it |
| A7 | ms/step ≈ 0.7–1.1 at ~2,300 particles | Fingerprint Gate | Under the 1.5 ms budget by survey analogy |
| A8 | `MIN_FINGER_PARTICLES = 4` and dropping wall bins are acceptable readings of D-07 | Metrics | Planner records them as interpretations; they make the rule stricter, not looser |
| A9 | 3:1 at the floor plus hue contrast is enough for legibility | Color | The four-mode visual check is the real gate (D-12) |
| A10 | Honesty word list | Pitfall 8 | Copy review only |
| A11 | `MIN_HEIGHT_FRACTION` ≈ 0.48 for the nominal frame | Site Map | Recompute from the final frame |

## Open Questions

1. **Does candidate 1 (plain water) form fingers or rain at r 0.04?**
   - Known: no interfacial tension; ~10 particles per wavelength.
   - Unclear: whether percolation dominates at ratio 3.
   - Recommendation: the D-05 order and the rain diagnostic decide; look at a frame at 3 s for each candidate.
2. **Should the tilt test be the one-step twin or the behavior run?**
   - Recommendation: write the debug-safe twin (exact, fast) and keep a short release behavior check, since D-18 words it as centroid motion.
3. **Exact D-07 interpretation (minimum particles per bin, wall bins).**
   - Recommendation: lock it in the Plan 1 text before the sweep runs, so the floor is never loosened after measuring (the D-06 spirit).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Tests, scene-spot, gate | ✓ | 1.97.0 via `rust-toolchain.toml` | — |
| bun | Web typecheck/unit, scripts | ✓ | installed | — |
| just | Recipes | ✓ | installed | Run the underlying commands |
| ffmpeg, webpmux, dwebp | `just readme-svg`, clip review | ✓ | installed | — |
| mdformat 1.0.0 | `just markdown-check` | ✓ | `~/.local/bin/mdformat` | — |
| DejaVu Sans font | README WebP | ✓ | `/usr/local/texlive/2025/.../DejaVuSans.ttf` | — |
| Playwright Chromium | player + hud-safe-frame e2e | ✓ (Phases 46–48) | cached | — |

No blocking gaps.

## Verification Commands

| Req | Behavior | Command |
|-----|----------|---------|
| Helper | Calibration still exact | `cargo test -p liquidfun-wasm --lib particle_force` |
| RT-01/02 (debug) | Parser, geometry, settle twin, tilt twin, sync | `cargo test -p liquidfun-wasm --lib rayleigh_taylor` |
| RT-01/02 (release) | Spike rules, D-18 outcomes | `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor -- --nocapture` |
| Registration | Order, survey, slider, tokens | `cargo test -p liquidfun-wasm --lib -- scene::tests scene_spot gravity_slider session::tests` |
| SCENE-01..03 | Catalog, controls, credits, portrait, README plan, demo media, legend, copy | `cd web && bun run typecheck && bun run test:unit` |
| e2e | Player loops, HUD safe frame | `just web-build && cd web && CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts --project chromium` |
| README | Assets | `LIQUIDFUN_README_FONT_FILE=<ttf> just readme-svg && just markdown-check` |
| PREP-01 | Gate | `just scene-fingerprint-gate` |
| Repo | Format, lint, length | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && bun scripts/bright-builds-check.ts all` |

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Exact-token allowlist for `weight-ratio` (`1.2`, `1.5`, `2`, `3`) in both `build` and `apply_control`; unknown preset names and controls → `Err(UnknownControl)`; gravity is parsed by the shared slider parser; pointer input is ignored with `Ok` after the session's finiteness check |
| V2/V3/V4/V6 | no | Static client-side playground |

| Threat | STRIDE | Mitigation |
|--------|--------|-----------|
| Malformed ratio (`"2.0"`, `"10"`, `"-1"`, `" 2"`) | Tampering / DoS (huge extra weight) | Exact-match parse; the extra weight is bounded at 2·|g| |
| Unknown preset in the replay list | Tampering | `build` rejects it |

## Sources

### Primary (HIGH, verified in this session)
- `crates/liquidfun-wasm/src/scene/particle_force.rs` and `particle_force/tests.rs`: the helper API, the `expect(dead_code)` attribute, the group calibration test.
- `crates/liquidfun/src/particle/{force.rs, view.rs, group/view.rs, group/source.rs, storage/creation.rs, storage/group.rs, storage/runtime.rs, solver/material.rs, solver/pressure.rs, definition/system_definition.rs}`, `math/settings.rs`: contiguity, `group_ids`, repulsive pass, pressure cap, defaults.
- `crates/liquidfun-wasm/src/{scene.rs, session.rs, session/escape.rs, scene_spot.rs, scene/gravity_slider.rs, scene/dam_break.rs, scene/color_mixer.rs, scene/hydraulic_fountain.rs, scene/archimedes_screw.rs, scene/fake_ferrofluid.rs, scene/fake_ferrofluid/{geometry,test_support,spike_tests,sync_tests}.rs}`.
- `web/src/catalog/*`, `web/src/components/{scene-controls.ts, SceneControls.tsx}`, `web/src/player/{runtime.ts, scene-lifecycle.ts, worker-controls.ts}`, `web/src/render/{particle-limit.ts, density-shade.ts}`, `web/src/export/animated-svg.ts`, `web/scripts/{readme-svg/plans.ts, demo-media/model.ts}`, `web/tests/{scenes,scene-catalog-controls,scene-catalog-credits,portrait-bounds}.test.ts`, `web/e2e/player{.spec,-helpers}.ts`.
- `docs/adding-a-scene.md`, `docs/benchmarks/{scene-fingerprints.json, scene-survey.md}`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/research/v1.6-curiosity-scenes/{STACK,PITFALLS,FEATURES}.md`, `.planning/phases/48-fake-ferrofluid/*`.
- WCAG contrast computed with a scratch script (relative luminance formula).

### Secondary (MEDIUM)
- RT physics (growth ∝ √(A·g·k), fingers and bubbles, SPH damping): `.planning/research/v1.6-curiosity-scenes/FEATURES.md` §5 (cites Wikipedia); used only qualitatively.

## Metadata

**Confidence breakdown:**
- Engine and helper facts, plumbing, sites: HIGH (read directly)
- Scene design numbers: MEDIUM-LOW (derived; no probe this session by instruction; D-06 spike is the measurement)
- Pitfalls: HIGH for 3, 5, 6, 9, 11, 12 (code-verified); MEDIUM for 1, 2, 4 (mechanism-verified, magnitude assumed)

**Research date:** 2026-10-18
**Valid until:** 2026-11-17 (stable in-repo code)

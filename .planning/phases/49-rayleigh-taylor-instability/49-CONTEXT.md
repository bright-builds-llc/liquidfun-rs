---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: 2026-10-19T00:37:59.379Z
---

# Phase 49: Rayleigh–Taylor Instability - Context

**Gathered:** 2026-10-18
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Ship one new watch-first playground scene, `rayleigh-taylor`. A closed tank holds a high-contrast heavy liquid layer resting on a light one. On load, after a short settle, the heavy layer gets extra weight along world gravity, and a seeded interface ripple grows into fingers that sink through the light liquid (at least 3 heavy fingers past 20% of the tank height by about 4 s). The layers overturn within about 10 s. A recreating "Weight ratio" preset picks the heavy-to-light weight ratio in a bounded range (1.2× to 3×), and the run restarts through that preset or the shared "Reset scene" button. The same settings always give the same end-state fingerprint. The scene uses one particle system with two groups, and the extra weight comes from the Phase 48 shared helper `apply_group_acceleration`, with no engine change. The caption says the heavy layer is given extra weight and never claims a simulated density. The scene satisfies SCENE-01..03 through `docs/adding-a-scene.md`, and the PREP-01 id-join fingerprint gate reports all 28 protected ids unchanged.

Out of scope: perturbation presets and a flip-tank action (RT-03, future), an interface-tension toggle, a growth-rate chart or any quantitative √(A·g·k) claim, new cue kinds and any engine change.

</domain>

<decisions>
## Implementation Decisions

### Physics model
- **D-01:** One particle system with two groups: the heavy group on top and the light group below. Never two particle systems, because they would pass through each other (PITFALLS 3). Never add per-group density to the engine. Never set `COLOR_MIXING`.
- **D-02:** Once per fixed step in `on_advance`, apply an extra acceleration of `(ratio − 1)·|g|` along the current gravity direction (`-GravityFrame::up`) to the heavy group only, through `particle_force::apply_group_acceleration`. Remove its `expect(dead_code)` attribute now that it has a caller. Build the frame from `world.gravity()` every step. When `GravityFrame::from_gravity` returns `None`, apply nothing for that step. No term hard-codes −y, so the gravity slider (2 to 80) and phone tilt keep "heavy" pointing down. This is the Boussinesq picture: equal inertia, different weight.
- **D-03:** Settle, then ramp. The extra weight is off for a short settle (about 0.3 to 0.6 s) so the lattice pressure wave does not stir the interface, then ramps linearly to full over about 0.2 to 0.4 s. The total delay must leave the 4 s finger target reachable. Exact timings are Claude's discretion and come from the spike.
- **D-04:** The interface starts with a seeded, deterministic perturbation: one to three fixed sinusoidal modes in the initial heavy/light boundary, with an optional small fixed inline hash for noise. Use no RNG crate and no time-based seed, so repeated runs and `scene-spot` stay identical (`scene_spot.rs` fails when two runs disagree).
- **D-05:** Flags are chosen by the spike from these candidates, in this order: plain water on both groups; `VISCOUS` (strength at most 1.5) on both groups; `VISCOUS` plus `REPULSIVE` between groups for a sharper interface. Avoid `TENSILE` in the bulk, because it zeroes normal pressure. Keep the first candidate that passes D-06.

### Spike first
- **D-06:** Plan 1 starts with a short, time-boxed parameter sweep over flags (D-05), perturbation modes (D-04), particle radius, and the settle and ramp timings (D-03), at ratios 1.2, 2 and 3 and gravity 10. Written pass rules, measured at the default ratio and gravity 10:
  - at least 3 separate heavy fingers reach past 20% of the tank height below the initial interface by about 4 s;
  - overturn by about 10 s, meaning the heavy group's mean height along the up axis is below the light group's mean height;
  - a 60 s run has no eviction, no failed step and a bounded max particle speed;
  - two runs give identical end-state fingerprints.
  At ratio 1.2 the fingers may be slower, but the layers must still visibly overturn within about 20 s, and the measured times are recorded. Record the sweep measurements in the plan summary. If no candidate passes, stop and record a decision checkpoint (an amendment like Phase 46 D-07a) instead of loosening the floor silently. The researched fallback is a single-mode seed with a qualitative caption.
- **D-07:** Fingers are measured from heavy particles binned across the tank along the `across` axis. A finger is a run of adjacent bins whose deepest heavy particle is past the 20% depth line, separated by at least one bin that is not. Use tolerances, not exact trajectories.

### Weight ratio and restart
- **D-08:** Control `weight-ratio`, labeled "Weight ratio", is a recreating preset (`kind: "preset"`, `recreates: true`, the Dam Break `water-amount` pattern). Values are `1.2` (1.2×), `1.5` (1.5×), `2` (2×, default) and `3` (3×). The Rust builder reads it from the presets, rejects unknown values, and falls back to the default when it is absent. Picking a ratio restarts the run.
- **D-09:** Restart uses the shared "Reset scene" button (`web/src/components/CanvasStage.tsx`) and the recreating preset. Add no new action. A flip-tank action is RT-03 (future).
- **D-10:** The scene has no pointer interaction and no live control. `apply_pointer`, `apply_action` and `apply_control` keep the default hooks, which reject unknown names.

### Geometry, color and frame
- **D-11:** Use toy meter scale. A closed rectangular tank with thick, overlapping static walls and a lid, roughly as wide as it is tall or a little taller, with the two layers about equal in depth. Keep the scene at or below about 3,000 particles and about 1.5 ms per step in native release. Keep all geometry within about 10 m of the origin. Exact sizes and particle radius are Claude's discretion and come from the spike.
- **D-12:** The heavy layer is a warm orange and the light layer a deep blue, chosen so they contrast with each other and with the `#071018` canvas after the 0.73 density-shade floor. Check Shaded blob (default), Water, circle wireframe (README) and the static preview before the phase closes, and record what was seen.
- **D-13:** The landscape `viewBounds` frames the whole tank. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the tank with its walls, so it fills a phone canvas (about 390 by 844 CSS pixels). A tall tank suits the phone frame.

### Catalog, caption and cues
- **D-14:** Use id `rayleigh-taylor`, module `rayleigh_taylor`, variant `RayleighTaylor`, title "Rayleigh–Taylor Instability" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the ferrofluid in catalog order. Credits say "original experimental scene", and the source is `sceneSource("rayleigh_taylor.rs")`.
- **D-15:** The caption names the fake plainly. For example: "A heavy orange layer rests on a light blue one. The orange layer is given extra weight, so a small ripple grows into fingers that sink while blue bubbles rise, and the layers turn over." Write a hint, for example "Watch the fingers form. Pick a Weight ratio or press Reset scene to run it again." Never say "density 2×" as if it were simulated, and describe growth qualitatively.
- **D-16:** The README plan is watch-first with no cue (`{ id: "rayleigh-taylor", cues: [] }`), so the 10 s clip shows the flat start, the fingers and the overturn. The survey entry is `SurveyCue::Default`. The demo-media `SCENE_CAPTURE_PLANS` entry follows the watch-first screw precedent. The scene stays in the e2e watch-first loop and gets no `POINTER_CONTROL` entry.
- **D-17:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed; commit only this scene's new WebP and SVG. The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

### Tests and evidence
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Per-scene checklist and shared rules
- `docs/adding-a-scene.md` — every Rust and web site (R1–R12, W1–W6, S1–S3, T1–T6, E1–E5), the gates, and the baseline-growth rule
- `.planning/ROADMAP.md` § "Shared acceptance rules" and § "Phase 49: Rayleigh–Taylor Instability" — success criteria and the shared SCENE/PREP rules
- `.planning/REQUIREMENTS.md` — RT-01, RT-02, SCENE-01..03 (RT-03 is future)
- `AGENTS.md` § "README scene gallery", "Playground canvas framing", "Playground scale legend"

### Rayleigh–Taylor research
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md` § "5. Rayleigh–Taylor Instability" — real-world reference, defining moment, table stakes, anti-features and README expectations
- `.planning/research/v1.6-curiosity-scenes/PITFALLS.md` § "Pitfall 3" (two systems or per-group density) and § "Pitfall 4" (rain or blob instead of fingers), plus the gravity, budget and color-contrast pitfalls
- `.planning/research/v1.6-curiosity-scenes/STACK.md` § "5. Rayleigh–Taylor instability" — one system, extra-weight force formula, flag advice
- `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md` — hooks seam table (Rayleigh–Taylor row)
- `.planning/research/v1.6-curiosity-scenes/SUMMARY.md` — Rayleigh–Taylor deliverables and caption wording

### Prior phase decisions
- `.planning/phases/48-fake-ferrofluid/48-CONTEXT.md` — D-12 to D-14 shared force helper, spike-first plan shape, gate and baseline growth
- `.planning/phases/47-archimedes-screw/47-CONTEXT.md` — watch-first scene through the checklist, README and demo-media precedent
- `.planning/phases/46-cartesian-diver/46-CONTEXT.md` — D-07a high-gravity lesson
- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-CONTEXT.md` — D-02..D-05 baseline file and gate semantics

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/liquidfun-wasm/src/scene/particle_force.rs`: `GravityFrame::from_gravity`, `ForceScale::for_system` and `apply_group_acceleration` (already marked as waiting for Phase 49), with unit tests in `particle_force/tests.rs`.
- `crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs` and `fake_ferrofluid/`: the latest curiosity scene, its `build`/`build_parts` shape, `on_advance` force application and the split test files.
- `crates/liquidfun-wasm/src/scene/archimedes_screw.rs`: a watch-first curiosity scene.
- `web/src/catalog/scene-records-classic.ts` (Dam Break `water-amount`): the recreating preset pattern.
- `web/src/catalog/scene-record-shared.ts`: `runtimePreset`, `option`, `action`, `sceneSource` and `withGravitySlider`.
- `web/src/components/CanvasStage.tsx`: the shared "Reset scene" button.

### Established Patterns
- `SceneHooks` (`crates/liquidfun-wasm/src/scene.rs`): apply forces once per fixed step in `on_advance`.
- Release-mode native scene tests with tolerance-based outcome assertions, in the existing `liquidfun-wasm` lib test binary. A new test binary triggers about 2 minutes of macOS scanning.
- Files stay under 628 lines. Use the `foo.rs` + `foo/` module layout.
- Scene presets are parsed in the Rust builder and rejected when unknown.

### Integration Points
- Rust: `scene.rs` (`mod`, `SceneId`, `build_scene`, `parse_scene_id`, `ALL_SCENE_IDS`), `gravity_slider.rs` `variant_index`, `scene_spot.rs` `SURVEY_SCENES`, `scene_spot/tests.rs`, `session/tests.rs`, and `particle_force.rs`.
- Web: `scenes.ts` `SCENE_IDS`, `scene-records-curiosity.ts`, `previews-curiosity.tsx`, `portrait-bounds.ts`, `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`, and the tests listed in T1–T4.

</code_context>

<specifics>
## Specific Ideas

- The defining moment: a flat orange-over-blue interface dimples, grows a row of orange fingers that sink with rounded, mushroom-like tips while blue bubbles rise between them, and ends with the orange at the bottom.
- A higher weight ratio gives faster, narrower fingers. A ratio near 1.2 gives a slow, gentle overturn. The gravity slider speeds up or slows down the whole story.
- Expect blobbier fingers than in a real tank; SPH damps fine structure, and the caption stays qualitative.

</specifics>

<deferred>
## Deferred Ideas

- Perturbation presets (single mode, few modes, noise) and a flip-tank action (RT-03, future requirement).
- An interface tension or viscosity toggle that shows short-wave suppression (research differentiator).
- An Atwood number readout in the hint or label.

</deferred>

---

*Phase: 49-rayleigh-taylor-instability*
*Context gathered: 2026-10-18*

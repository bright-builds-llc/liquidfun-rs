---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: 2026-10-15T04:21:14.073Z
---

# Phase 48: Fake Ferrofluid - Context

**Gathered:** 2026-10-15
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Ship one new playground scene, `fake-ferrofluid`. A dark but visible liquid puddle sits in a shallow dish. A magnet rests under the dish on load, and the puddle rises into a row of separate spikes (at least 4 surface peaks) above it. Pressing and dragging moves the magnet and the spikes follow it. Releasing or cancelling the pointer, or switching the magnet off, collapses the spikes back into a puddle. A live strength control is clamped below the point where liquid detaches. The magnet pull is a scripted, gravity-aware, per-fixed-step bridge force, built as a shared helper that Phase 49 reuses. The scene satisfies SCENE-01..03 through `docs/adding-a-scene.md`, and the PREP-01 id-join fingerprint gate reports all 27 protected ids unchanged, including `archimedes-screw`.

Out of scope: hysteresis (FERRO-04, future), real magnetostatics or a field solver, hover-driven magnet, a new render mode, new cue kinds, and any engine change.

</domain>

<decisions>
## Implementation Decisions

### Field model and spikes
- **D-01:** The pull is a scripted fake field, never a pure point attraction toward the magnet (research anti-feature: it makes a ball or flings droplets). Its three parts are: (a) a **lift along the field axis**, pointing out of the puddle opposite to `world.gravity()`, whose strength falls off smoothly with distance from the magnet; (b) a **spatial modulation** that concentrates the lift into separate spike columns, either `1 + a·cos(k·(s − s_mag))` along the surface or N fixed "spike tip" targets spaced across the dish and centered on the magnet; and (c) a weak lateral pull toward the magnet along the surface, so the puddle creeps toward an off-center magnet and spike tips lean toward it. Here `s` is the coordinate across the dish, perpendicular to gravity.
- **D-02:** The profile is bounded and smooth. It has a softened core (no singularity), a smooth falloff to an outer radius, and an acceleration cap of at most about 2 to 3 times the gravity magnitude at maximum strength. Affected particles get explicit velocity-proportional drag, so the field settles and holds instead of oscillating (PITFALLS 5).
- **D-03:** The liquid is one particle group with `TENSILE | VISCOUS` flags (the Liquid Timer precedent) so spikes stay cohesive. Viscous strength stays at or below 1.5, because the step fails above about 2.0.
- **D-04:** Target 5 to 9 spikes across the dish at default strength. The roadmap floor is at least 4 separate surface peaks. Spikes appear within about 1 s of the magnet energizing at gravity 10 and collapse within about 1 to 1.5 s after it switches off.
- **D-05:** Plan 1 starts with a short, time-boxed stability and appearance spike. It measures the cos-modulation candidate and the fixed spike-tip candidate against written pass rules: at least 4 separate peaks within about 1.5 s at default strength and gravity 10; a collapse to a flat surface within about 1.5 s after off; and, during a 60 s hold at maximum strength, no eviction, no failed step and a max particle speed that does not grow over time. Keep the first candidate that passes and record the measurements in the plan summary. If neither passes, stop and record a decision checkpoint (an amendment like Phase 46 D-07a) instead of loosening the floor silently.

### Magnet state and pointer
- **D-06:** The hooks hold `magnet_on: bool` and `magnet_position: Vec2`. On load the magnet is on and rests under the center of the dish floor.
- **D-07:** Pointer `Down` anywhere on the canvas turns the magnet on and moves it to the pointer. `Move` while pressed updates the position. The position is clamped to the scene bounds, and non-finite input is rejected. `Up` and `Cancel` are handled identically: the magnet switches off and parks at its rest position under the dish, so the spikes collapse. The next press, or the Magnet action, turns it back on. There is no hover.
- **D-08:** No physics runs in `apply_pointer`, `apply_action` or `apply_control`. They store intent only. `on_advance` applies the field once per fixed step (Color Mixer pattern and ARCHITECTURE rule), so behavior does not depend on frame rate.

### Strength and on/off controls
- **D-09:** Control `magnet-strength`, labeled "Magnet strength", is a live HUD `range` spinner in percent (`kind: "range"`, `recreates: false`, `surface: "hud"`, `widget: "spinner"`). The Rust hook returns `ControlEffect::Live`. It lives in a new `web/src/catalog/fake-ferrofluid-strength.ts`, copied from `archimedes-screw-speed.ts`. The range is 0 to 100. 0 means no pull, and 100 is the tuned safe maximum, which never detaches liquid. The default is tuned so the load shows 5 to 9 spikes, at roughly 60 to 70.
- **D-10:** Action `magnet`, labeled "Magnet", is a panel action that toggles `magnet_on` at the current position. It is the keyboard path for on and off and the hook for README cues (Diver `squeeze` precedent). Turning it on after a release energizes the magnet at its rest position.
- **D-11:** Strength and on/off are independent. A strength of 0 with the magnet on shows a flat puddle.

### Shared gravity-aware force helper
- **D-12:** Add a small shared bridge module, for example `crates/liquidfun-wasm/src/scene/particle_force.rs`, that converts a per-particle acceleration into the force for one fixed step. It uses the computed particle mass (`density·(stride·2r)²`) and the particle-iteration factor, applies with `apply_particle_force` or `apply_particle_force_range` (range totals are multiplied by member count, as in `impulse.rs`), and exposes gravity-direction helpers built from `world.gravity()`. Phase 49 reuses it for the Rayleigh–Taylor extra weight. Do not reuse or "improve" `basin_family` or `soup_family`.
- **D-13:** Measure the calibration in the Plan 1 spike, because the research derived it from reading code, not from a run. A unit test checks that a known acceleration request produces that acceleration within tolerance, and a second checks that the lift direction follows a tilted gravity vector.
- **D-14:** Every field term scales with the gravity magnitude and points relative to the gravity direction. No term hard-codes −y, so the gravity slider (2 to 80) and phone tilt stay correct.

### Geometry, color and marker
- **D-15:** Use toy meter scale. The particle radius is about 0.04 to 0.06 m, the dish is a shallow, wide static dish with thick overlapping walls, and the puddle depth is a few particle layers, enough to form tall spikes. Keep all geometry within about 10 m of the origin (eviction rule), and keep the scene at or below about 3,000 particles and about 1.5 ms/step native release. Exact sizes are Claude's discretion.
- **D-16:** The liquid color is a dark blue-gray or graphite violet, not black. It must stay readable on the `#071018` canvas after the 0.73 density-shade floor in Shaded blob (default), Water, circle wireframe (README) and the static preview. Check all four before the phase closes.
- **D-17:** The magnet marker is world content drawn through `collect_segments` and/or `collect_circles`, for example a small bar or horseshoe at `magnet_position`. When off, the marker shows a parked, inactive look, such as an outline only. It stays within the frame caps (512 segments, 48 circles). It is not HUD chrome and does not carry `data-hud-obstacle`.
- **D-18:** The scene is wide. The landscape `viewBounds` frames the dish plus enough space above it for spikes and a dragged magnet. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the dish, the spikes and the rest magnet, walls included.

### Catalog, caption and cues
- **D-19:** Use id `fake-ferrofluid`, module `fake_ferrofluid`, variant `FakeFerrofluid`, title "Fake Ferrofluid" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the screw in catalog order. Credits say "original experimental scene", and the source is `sceneSource("fake_ferrofluid.rs")`.
- **D-20:** The caption calls the pull scripted. For example: "A dark liquid puddle sits over a magnet and rises into a row of spikes. The pull is a scripted magnetic pull that lifts the liquid into spike columns, not a simulated magnetic field." Write a hint instead of `WATCH_FIRST_HINT`, for example "Press and drag to move the magnet; let go to switch it off. Use Magnet to toggle it and Magnet strength to change the pull." Do not claim real magnetism, a field solve or measured physics.
- **D-21:** The README plan is watch-first at default settings. It cues the `magnet` action at about 6 s (off) and about 8 s (on), so the 10 s clip shows spikes, a collapse and a re-rise. Check the sample indices against the generated clip and adjust them if needed. The survey entry is `SurveyCue::Default`, because the resting magnet is the active, expensive state. The demo-media `SCENE_CAPTURE_PLANS` entry uses the shared center click. Its release parks the magnet and switches it off at step 180, so the 8 s capture shows the spikes rise for 3 s and then collapse. This is accepted, because every canvas gesture ends in a release (48-REVIEW WR-03). E2E `POINTER_CONTROL` gets an entry so the pointer-driven scene is not dropped into the watch-first loop.
- **D-22:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed (commit only this scene's new WebP and SVG). The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

### Tests and evidence
- **D-23:** Native release-mode tests live in the existing `liquidfun-wasm` lib test binary, one concern per test:
  - On load at gravity 10 and default strength, at least 4 separate surface peaks form within about 1.5 s.
  - Dragging the magnet off-center moves the peak centroid toward it.
  - Pointer `Up` collapses the spikes to a flat surface within about 1.5 s.
  - Pointer `Cancel` collapses them the same way.
  - The `magnet` action off collapses them the same way.
  - Strength 0 with the magnet on keeps the surface flat.
  - A 60 s hold at strength 100, with the pointer held at three positions (under the dish, above the puddle and off-center), has no eviction, no failed step, a bounded max speed and a kinetic-energy proxy that does not grow from the first half to the second.
  - Stability smokes at gravity 2 and 80 have no eviction and no failed step. Spikes are not required at gravity 80, because the pressure cap squashes columns (Phase 46 D-07a lesson).
  - The control allowlist rejects bad values.
  - The helper unit tests from D-13 pass.
  Peaks are measured from a binned surface-height profile across the dish, with tolerances, not exact trajectories.
- **D-24:** Before the phase closes, `just scene-fingerprint-gate` must report all 27 protected ids unchanged with `fake-ferrofluid` under "added (accepted)". Record `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene fake-ferrofluid` in the phase summary.
- **D-25:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 28 protected ids unchanged.
- **D-26:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player and hud-safe-frame), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact field formula, falloff radii, modulation wavelength, drag, acceleration cap and default strength, within D-01 to D-04 and D-09. The Plan 1 spike evidence picks the spike candidate (D-05).
- Dish shape and size, particle radius and count, and puddle depth, within D-15.
- Exact color hex values, within D-16.
- Marker drawing style, within D-17.
- Helper module name and API shape, within D-12 to D-14.
- Plan breakdown and wave ordering, provided the spike comes first and the baseline growth commit (D-25) is last and separate.
- Whether to split `fake_ferrofluid.rs` into `fake_ferrofluid/field.rs` and `fake_ferrofluid/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Per-scene checklist and shared rules
- `docs/adding-a-scene.md` — every Rust and web site (R1–R12, W1–W6, S1–S3, T1–T6, E1–E5), the gates, and the baseline-growth rule
- `.planning/ROADMAP.md` § "Shared acceptance rules" and § "Phase 48: Fake Ferrofluid" — success criteria and the shared SCENE/PREP rules
- `.planning/REQUIREMENTS.md` — FERRO-01..03, SCENE-01..03 (FERRO-04 hysteresis is future)
- `AGENTS.md` § "README scene gallery", "Playground canvas framing", "Playground scale legend"

### Ferrofluid research
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md` § "6. Fake Ferrofluid" — the real-world reference, representation, table stakes, anti-features and README expectations
- `.planning/research/v1.6-curiosity-scenes/PITFALLS.md` § "Pitfall 5" (force explosion, energy gain, frame rate), § "Pitfall 14" (gravity and tilt), § "Pitfall 15" (budget), and the color contrast pitfall
- `.planning/research/v1.6-curiosity-scenes/STACK.md` § "6. Fake ferrofluid" — per-particle force, `TENSILE | VISCOUS`, no hover
- `.planning/research/v1.6-curiosity-scenes/SUMMARY.md` — force calibration rules, the ferrofluid deliverables and the "fake" caption wording
- `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md` — hooks seam table (ferrofluid row) and "Data flow: pointer-held force"

### Prior phase decisions
- `.planning/phases/47-archimedes-screw/47-CONTEXT.md` — the latest scene through the checklist; live HUD spinner file; spike-first plan shape; gate and baseline growth
- `.planning/phases/46-cartesian-diver/46-CONTEXT.md` — the pointer and action toggle pattern (D-01..D-03), the D-07a high-gravity lesson, POINTER_CONTROL e2e entry
- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-CONTEXT.md` — D-02..D-05 baseline file and gate semantics, D-11 file splits

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/liquidfun-wasm/src/scene/color_mixer.rs`: `maybe_pointer` stored on Down/Move and cleared on Up/Cancel, with the force applied in `on_advance`.
- `crates/liquidfun-wasm/src/scene/impulse.rs`: `apply_particle_force_range` with the member-count multiplication.
- `crates/liquidfun-wasm/src/scene/liquid_timer.rs`: a `TENSILE | VISCOUS` particle group.
- `crates/liquidfun-wasm/src/scene/cartesian_diver.rs` and `cartesian_diver/`: a pointer plus panel action toggle, drawn overlays and the curiosity scene test layout.
- `crates/liquidfun-wasm/src/scene/archimedes_screw.rs` and `web/src/catalog/archimedes-screw-speed.ts`: the latest live HUD spinner (`ControlEffect::Live`).
- `web/src/catalog/scene-record-shared.ts`: `runtimePreset`, `option`, `action`, `sceneSource` and `withGravitySlider`.

### Established Patterns
- `SceneHooks` (`crates/liquidfun-wasm/src/scene.rs`): store intent in the hooks and apply motion and force once per fixed step in `on_advance`.
- Release-mode native scene tests with tolerance-based outcome assertions.
- New tests go into the existing `liquidfun-wasm` lib test binary. A new test binary triggers about 2 minutes of macOS scanning.
- Files stay under 628 lines. Use the `foo.rs` + `foo/` module layout.
- Pointer world coordinates are checked at the session boundary (`session.rs`). Also clamp the magnet to the scene bounds.

### Integration Points
- Rust: `scene.rs` (`mod`, `SceneId`, `build_scene`, `parse_scene_id`, `ALL_SCENE_IDS`), `gravity_slider.rs` `variant_index`, `scene_spot.rs` `SURVEY_SCENES`, `scene_spot/tests.rs`, `session/tests.rs`, and the new shared force helper module.
- Web: `scenes.ts` `SCENE_IDS`, `scene-records-curiosity.ts`, `previews-curiosity.tsx`, the new `fake-ferrofluid-strength.ts`, `portrait-bounds.ts`, `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`, the tests listed in T1–T4, and e2e `player-helpers.ts` (`POINTER_CONTROL`) and `player.spec.ts`.

</code_context>

<specifics>
## Specific Ideas

- The defining moment: a flat, dark puddle erupts into a crown of sharp spikes above the magnet, and the spikes slump back into a glossy puddle when the magnet lets go.
- Dragging the magnet sideways makes the puddle creep after it and the spikes lean toward it.
- The 2D crown is a row of spikes, not a hexagonal array, and the spike spacing is not tied to a real capillary length. Both are acceptable simplifications, and the caption says the pull is scripted.

</specifics>

<deferred>
## Deferred Ideas

- Hysteresis: spikes that persist slightly below onset as strength ramps down (FERRO-04, future requirement).
- A magnet position preset (below the dish versus follow the pointer) and a GPU Water tint tuned for ferrofluid gloss (research differentiators).
- Held-pointer README cue kinds (not allowed by the shared rules this milestone).

</deferred>

---

*Phase: 48-fake-ferrofluid*
*Context gathered: 2026-10-15*

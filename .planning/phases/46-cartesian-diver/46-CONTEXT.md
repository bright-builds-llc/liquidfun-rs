---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: 2026-10-14T17:39:28.785Z
---

# Phase 46: Cartesian Diver - Context

**Gathered:** 2026-10-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Ship one new playground scene, `cartesian-diver`: a closed bottle of particle water with a diver floating near the top. Pressing and holding on the bottle, or toggling Squeeze from the keyboard, makes the diver sink. Releasing it, including a cancelled pointer, makes the diver rise again. While squeezed, the bottle visibly dents and the drawn air bubble inside the diver shrinks. This is the first scene to go through `docs/adding-a-scene.md` end to end, so it also satisfies SCENE-01..03, and it must pass the PREP-01 id-join fingerprint gate with all 25 protected ids unchanged.

Out of scope: depth-dependent "no return" behavior and multiple divers (DIVER-04, future), a squeeze-strength slider, and any engine change.

</domain>

<decisions>
## Implementation Decisions

### Squeeze input model
- **D-01:** One `squeezed: bool` intent lives in the hooks struct, and the last input wins. Pointer `Down` inside the bottle hit region sets it true. Pointer `Up` and `Cancel` set it false wherever they happen. The panel action `squeeze` (label "Squeeze") flips it. No physics runs in `apply_pointer` or `apply_action`; `on_advance` applies the state once per fixed step (ARCHITECTURE rule).
- **D-02:** The hit region is the bottle's outer rectangle plus a small margin, about 0.3 m. A `Down` outside it does nothing, so the hint can say "press and hold the bottle". Pointer `Move` is ignored.
- **D-03:** The keyboard path is the existing panel `action("squeeze", "Squeeze")` control. It adds no HUD control, new control kind or "hold" infrastructure. The README and survey cue this same action.

### Sink and rise mechanism
- **D-04:** The diver is a single dynamic **circle** body (the v1.5 D-12 lesson that circles avoid native/wasm `sin`/`cos` divergence). Its radius is at least about 6 particle diameters, so single particle hits do not knock it around. Sleep is disabled on the body, so a density change is never ignored.
- **D-05:** Squeeze switches the diver fixture's density through `set_fixture_density` and then `reset_body_mass_data`. The released density is clearly below water and the squeezed density is clearly above it. The research starting point is about 0.5 released and 3 or more squeezed (PITFALLS 6), because water compresses with depth and densities near 1.5 to 2.0 hover mid-bottle. Tune the final values in native tests. Gravity scale is not used, because the caption names density switching.
- **D-06:** Ramp the effective density and the visual squeeze value over a short time, about 0.2 to 0.3 s, so the sink starts smoothly. Add moderate linear damping for a calm sink and rise. Exact values are Claude's discretion, within the timing rules in D-07.
- **D-07:** Timing is judged at gravity 10. While squeezed, the diver's center reaches the bottom third of the bottle within about 3 s. After release it returns to the top third within about 4 s. Native release-mode tests repeat the sink and rise at gravity 2, 10 and 80. Gravity 10 uses the 3 s and 4 s windows. Gravity 2 and 80 may use windows scaled by about √(10/g) with a sane cap, but must still show a full sink and a full rise. Tests assert outcomes with tolerance on the body y position, not exact trajectories.
- **D-08:** Every force comes from the engine's gravity and buoyancy, so the gravity slider and phone tilt keep working. The scene adds no scripted force on the diver and never pushes the water.

### Squeeze visuals
- **D-09:** The bottle dent is **draw-only**. While squeezed, `collect_segments` returns side walls bowed inward by a few particle diameters at mid-height, following the same ramped squeeze value. The physical walls never move, so the water is never pushed and the scene stays stable at every gravity. The caption does not claim that the dent pushes the water.
- **D-10:** The air bubble is a drawn circle inside the diver, returned by `collect_circles` together with the diver circle. Its radius shrinks from about 0.6 to about 0.3 of the diver radius with the ramped squeeze value. The bubble has no fixture and no physics.

### Bottle geometry and scale
- **D-11:** Use toy meter scale: particle radius about 0.04 to 0.06 m, thick overlapping static polygon walls, and a closed cap on top so water cannot leave. A tall bottle of roughly 2 to 2.5 m interior width and 5 to 6 m interior height is filled with particle water to just below the cap, leaving a small headspace. The diver floats at the surface near the top on load. Keep the scene at or below about 3,000 particles. The exact geometry and particle radius are Claude's discretion within this budget.
- **D-12:** The portrait frame in `web/src/catalog/portrait-bounds.ts` fits the bottle, walls included, with a small margin. A tall bottle is naturally portrait. The landscape `viewBounds` frames the whole bottle.
- **D-13:** Add `particle_iterations` (R6) or `EscapeLimits` (R12) arms only if tests show they are needed.

### Phone long-press callout
- **D-14:** Suppress iOS and Android long-press callouts on the scene canvas for every scene. Add `-webkit-touch-callout: none` and `user-select: none` (with the `-webkit-` prefix) to `canvas.scene-canvas`, and call `preventDefault` on `contextmenu` for the scene canvas. This is a shared web change with no effect on simulation fingerprints. An e2e check under the mobile viewport asserts the computed styles and that a dispatched `contextmenu` event on the canvas is default-prevented. A real-iPhone long press is an optional manual check, not a gate.

### Catalog, copy and cues
- **D-15:** Use id `cartesian-diver`, module `cartesian_diver`, variant `CartesianDiver`, title "Cartesian Diver" and catalog family `curiosity`. This is the family's first scene, so create `web/src/catalog/scene-records-curiosity.ts` and `web/src/catalog/previews-curiosity.tsx` and append them to the `SCENES` spread and the `ScenePreview` switch in catalog order. Credits say "original experimental scene", and the source is `sceneSource("cartesian_diver.rs")`.
- **D-16:** Use a written hint instead of `WATCH_FIRST_HINT`, for example "Press and hold the bottle (or use Squeeze) to sink the diver; let go and it rises." The caption must say plainly that switching the diver's density stands in for the squeezed air bubble, for example: "A diver floats in a closed bottle of particle water. Squeezing switches the diver to a heavier density, standing in for its squeezed air bubble, so it sinks; letting go switches it back and it rises. The dent and the bubble are drawn." Claims of compressed air, real pressure or simulated gas are not allowed (SCENE-03).
- **D-17:** The README plan cues the `squeeze` action at about sample 20 (about 1 s) and again at about sample 110 (about 5.5 s), so the 10 s clip shows a full sink and a full rise. Check the timing against the actual clip and adjust the sample indices if needed. The survey entry is `SurveyCue::Action("squeeze")`, and `only_idle_scenes_are_scripted` is updated (R10). Demo-media `SCENE_CAPTURE_PLANS` uses the same action. E2E `POINTER_CONTROL` gets an entry so the interactive scene is not dropped into the watch-first loop.
- **D-18:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed. The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

### Gates and evidence
- **D-19:** Before the phase closes, `just scene-fingerprint-gate` must report all 25 protected ids unchanged with `cartesian-diver` under "added (accepted)". Record the diver's `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene cartesian-diver` in the phase summary.
- **D-20:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 26 protected ids unchanged. Phase 47 then protects the diver.
- **D-21:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player, hud-safe-frame and the new long-press check), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact densities, ramp time, damping, diver radius, bottle dimensions, particle radius and count, within D-05 to D-11.
- Bottle-dent shape (how many segments, bow profile), and the preview drawing style.
- Plan breakdown and wave ordering, provided the baseline growth commit (D-20) is last and separate.
- Whether to split `cartesian_diver.rs` into `cartesian_diver/geometry.rs` and `cartesian_diver/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Per-scene checklist and shared rules
- `docs/adding-a-scene.md` — every Rust and web site (R1–R12, W1–W6, S1–S3, T1–T6, E1–E5), the gates and the baseline-growth rule
- `.planning/ROADMAP.md` § "Shared acceptance rules" and § "Phase 46: Cartesian Diver" — success criteria and shared SCENE/PREP rules
- `.planning/REQUIREMENTS.md` — DIVER-01..03, SCENE-01..03 (DIVER-04 is future)
- `AGENTS.md` § "README scene gallery", "Playground canvas framing", "Playground scale legend"

### Diver research
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md` § "4. Cartesian Diver" — behavior, timing, table stakes, anti-features, README cue plan
- `.planning/research/v1.6-curiosity-scenes/PITFALLS.md` § "Pitfall 6" and the engine facts table — `set_fixture_density` needs `reset_body_mass_data`, depth compression, sleep, circle body, honest caption
- `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md` — the hooks seam table (diver row), the pointer data flow, and "Avoid new cue kinds"

### Prior phase decisions
- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-CONTEXT.md` — D-02..D-05 (baseline file and gate semantics), D-07..D-09 (derived counts), D-11 (file splits)
- `.planning/phases/40-timed-valve-and-teaching-levels/40-09-SUMMARY.md` — static pool buoyancy table (density 2.0 hovers at 3 m)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs`: buoyancy of dynamic bodies in particle water, body wake-up after changes, and `collect_circles` for drawn circles. This is the closest precedent.
- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs`: a toggle panel action (`toggle-paddle-rail`) handled in `apply_action`, with tests.
- `web/src/catalog/scene-record-shared.ts`: `action(id, label)`, `sceneSource(...)` and `withGravitySlider(...)` helpers.
- `web/src/catalog/preview-kit.tsx`: shared preview drawing pieces.

### Established Patterns
- `SceneHooks` (`crates/liquidfun-wasm/src/scene.rs`): `on_advance`, `on_after_step`, `apply_action`, `apply_pointer`, `collect_segments`, `collect_circles`. Store intent in hooks and apply it in `on_advance`.
- Release-mode native scene tests with tolerance-based outcome assertions. Debug builds diverge from release.
- New tests go into the existing `liquidfun-wasm` lib test binary. A new test binary triggers about 2 minutes of macOS scanning.
- Files stay under 628 lines. Use the `foo.rs` + `foo/` module layout.

### Integration Points
- Rust: `scene.rs` (`mod`, `SceneId`, `build_scene`, `parse_scene_id`, `ALL_SCENE_IDS`), `gravity_slider.rs` `variant_index`, `scene_spot.rs` `SURVEY_SCENES`, `scene_spot/tests.rs`, `session/tests.rs`.
- Web: `scenes.ts` `SCENE_IDS`, the new `scene-records-curiosity.ts` and `previews-curiosity.tsx`, `portrait-bounds.ts`, `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`, the tests listed in T1–T4, and e2e `player-helpers.ts` and `player.spec.ts`.
- `web/src/styles/player-viewport.css` `canvas` rules (already `touch-action: none`; no callout or user-select suppression yet), and the canvas pointer wiring in `web/src/input/`.

</code_context>

<specifics>
## Specific Ideas

- The defining moment: the diver floats, a squeeze sinks it, and a release brings it back up. The bottle bows while squeezed and the bubble shrinks.
- "Making the squeeze move the water directly" is an anti-feature. Change the diver density only.

</specifics>

<deferred>
## Deferred Ideas

- Depth-dependent "no return" behavior and multiple divers (DIVER-04, future requirement).
- A squeeze-strength slider for hovering the diver mid-bottle (research differentiator, not in DIVER-01..03).

</deferred>

---

*Phase: 46-cartesian-diver*
*Context gathered: 2026-10-14*

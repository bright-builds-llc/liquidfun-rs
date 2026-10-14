---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: 2026-10-14T20:03:17.402Z
---

# Phase 47: Archimedes Screw - Context

**Gathered:** 2026-10-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Ship one new playground scene, `archimedes-screw`. It shows a tilted tube whose lower end sits in a low reservoir and whose upper end opens into a higher basin. Moving slanted vanes inside the tube carry separate pockets of water up into the basin. The basin overflows down a return chute into the reservoir, so the scene keeps running. A live HUD speed spinner sets the screw speed. A Direction control runs the screw in reverse, and the vanes then carry water back down. The scene satisfies SCENE-01..03 through `docs/adding-a-scene.md`, and the PREP-01 id-join fingerprint gate reports all 26 protected ids unchanged, including `cartesian-diver`.

Out of scope: a sand or powder material, a tilt-angle control (SCREW-03, future), a helix overlay, pointer-dragging the screw, a motor torque or load model, and any engine change.

</domain>

<decisions>
## Implementation Decisions

### Vane mechanism and wrap
- **D-01:** The vanes are **kinematic** bodies driven by velocity every fixed step in `on_advance`. They are never dynamic bodies on a revolute motor. One revolution advances the vanes by exactly one pitch, so axial speed = pitch × rpm / 60.
- **D-02:** The research-recommended first candidate is one kinematic vane-train body that moves along the tube axis. After one pitch it snaps back exactly one pitch with `set_body_transform`, so the vane lattice is geometrically unchanged at the snap. The fallback is closed-loop paddles that run up through the tube and return outside it, below the tube floor. Each paddle's velocity is set every step toward its path point. Plan 1 starts with a short, time-boxed wrap and leak spike that measures both candidates against written pass rules: no particles behind walls, no evictions, no burst of particle speed at the wrap, and pockets that visibly hold water. Keep the first candidate that passes and record the measurements in the plan summary.
- **D-03:** A vane never appears, disappears or teleports inside water. Any wrap or snap point lies in a sealed housing or dry region that particles cannot reach. Vane ends are embedded in the thick static tube walls (kinematic and static bodies do not collide), so the vane closes the tube gap.
- **D-04:** Clearance and tunneling guards follow PITFALLS 7. Vanes are at least about 3 particle diameters thick. Any vane-to-wall clearance stays between about 1 and 1.5 particle diameters, or the vane overlaps the wall. A `const _: () = assert!(MAX_AXIAL_SPEED < PARTICLE_DIAMETER / SIM_DT)` with margin guards the maximum speed, as in `hydraulic_fountain.rs`.
- **D-05:** Vanes are slanted, not perpendicular to the axis, so in side view they read as the flight lines of a helix. Exact slant is Claude's discretion. Some leak-back past the vanes is real physics and is acceptable as long as the basin gains water.

### Water loop and reverse
- **D-06:** The tube is closed (floor and roof). Its lower end is submerged in the reservoir, and its upper end opens into the upper basin below the basin's overflow lip, so the screw is symmetric: forward lifts water and reverse scoops it back down.
- **D-07:** The basin has an overflow weir. Water above the lip spills into a return chute made of static walls that drops it back into the reservoir. No particles are teleported, emitted or destroyed. The chute follows the Liquid Bubbler and Stacked Drip side-return precedent.
- **D-08:** Size the total water so the reservoir still covers the tube inlet when the basin is at its lip and every pocket in the tube is full. The reservoir never runs dry at default or maximum speed.

### Speed and direction controls
- **D-09:** Control `screw-speed`, labeled "Screw speed", is a live HUD `range` spinner in rpm (`kind: "range"`, `recreates: false`, `surface: "hud"`, `widget: "spinner"`). The Rust hook returns `ControlEffect::Live`. It lives in a new `web/src/catalog/archimedes-screw-speed.ts`, copied from `washing-machine-speed.ts`. The minimum is 0 (stopped), and the default and maximum are tuned in native tests. The default must deliver the first pocket into the basin within about 5 s at gravity 10. The maximum stays under the D-04 tunneling assert with margin and is about twice the default. The research starting point is a default of about 20 to 30 rpm, with no default above 100 rpm.
- **D-10:** Control `screw-direction`, labeled "Direction", is a live runtime preset with options `forward` ("Forward", the default) and `reverse` ("Reverse"), following the Theo Jansen `motor-direction` precedent. Reverse negates the axial velocity at the current speed. Speed and direction are independent, and the hooks state applies both in `on_advance`.
- **D-11:** No scene-specific pointer behavior. `apply_pointer` keeps the default, and dragging the screw is an anti-feature.

### Geometry and scale
- **D-12:** Use toy meter scale like the diver: particle radius about 0.04 to 0.06 m, thick overlapping static polygon walls, a tube tilted about 30° (allowed 25 to 35°), and an interior tube gap of about 6 to 8 particle diameters. The pitch must be long enough that each pocket holds a visible clump of water, about half full at default speed. Keep the scene at or below about 3,000 particles. Exact dimensions, pitch, particle radius and count are Claude's discretion within this budget.
- **D-13:** The scene is wide. The landscape `viewBounds` frames the reservoir, tube, basin and chute. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the action: the tube plus both water bodies, walls included. Some vertical margin is acceptable for a wide scene.
- **D-14:** Optional decoration (Claude's discretion): a drawn shaft line along the tube axis in `collect_segments`. The total must stay within the frame caps (512 segments, 48 circles). No helix overlay.

### Catalog, copy and cues
- **D-15:** Use id `archimedes-screw`, module `archimedes_screw`, variant `ArchimedesScrew`, title "Archimedes Screw" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the diver in catalog order. Credits say "original experimental scene", and the source is `sceneSource("archimedes_screw.rs")`.
- **D-16:** The caption says plainly that flat vanes stand in for a turning helix in a 2D side view. For example: "Slanted vanes slide up a tilted tube and carry pockets of water from the low reservoir into the upper basin, which overflows back down a chute. Flat vanes stand in for a turning helix in this 2D side view." Write a hint instead of `WATCH_FIRST_HINT`, for example "Use Screw speed to speed up or stop the screw, and Direction to run it in reverse. Labeled controls also work from the keyboard." Do not claim rotation, torque or a 3D helix.
- **D-17:** The README preview and survey are watch-first at default settings, with no cues. The 10 s clip must show at least one pocket climbing the full tube and spilling into the basin. Check this against the generated clip and adjust the default speed, not a cue, if it fails. Add the README plan in `web/scripts/readme-svg/plans.ts`, the survey entry, and the demo-media `SCENE_CAPTURE_PLANS` entry as idle.
- **D-18:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed (commit only the new WebP and SVG for this scene). The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

### Tests and evidence
- **D-19:** Native release-mode tests in the existing `liquidfun-wasm` lib test binary cover each of these:
  - A 60 s run at gravity 10 and default speed gains water in the basin, has no particle behind a wall, has no evictions, and keeps the reservoir covering the inlet.
  - The same 60 s run at maximum speed meets the same checks.
  - A reverse run that starts after the basin has gained water shows the basin count falling and the reservoir count rising.
  - Speed 0 holds the vanes still.
  - The control allowlist rejects bad values.
  Assertions use counts and regions with tolerance, not exact trajectories or step indices.
- **D-20:** A shorter stability smoke at gravity 2 and 80 asserts no leaks and no evictions only. Lifting is not required there (the g80 pressure cap squashes columns; see the Phase 46 D-07a lesson).
- **D-21:** Before the phase closes, `just scene-fingerprint-gate` must report all 26 protected ids unchanged with `archimedes-screw` under "added (accepted)". Record `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene archimedes-screw` in the phase summary.
- **D-22:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 27 protected ids unchanged.
- **D-23:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player and hud-safe-frame), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact pitch, vane slant and thickness, tube dimensions, tilt within 25 to 35°, particle radius and count, default and maximum rpm, and basin and chute shapes, within D-04, D-09 and D-12.
- Which wrap candidate ships, decided by the D-02 spike evidence.
- Preview drawing style and whether to draw a shaft line.
- Plan breakdown and wave ordering, provided the spike comes first and the baseline growth commit (D-22) is last and separate.
- Whether to split `archimedes_screw.rs` into `archimedes_screw/geometry.rs` and `archimedes_screw/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Per-scene checklist and shared rules
- `docs/adding-a-scene.md` — every Rust and web site (R1–R12, W1–W6, S1–S3, T1–T6, E1–E5), the gates, and the baseline-growth rule
- `.planning/ROADMAP.md` § "Shared acceptance rules" and § "Phase 47: Archimedes Screw" — success criteria and shared SCENE/PREP rules
- `.planning/REQUIREMENTS.md` — SCREW-01, SCREW-02, SCENE-01..03 (SCREW-03 is future)
- `AGENTS.md` § "README scene gallery", "Playground canvas framing", "Playground scale legend"

### Screw research
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md` § "2. Archimedes Screw" — real-world reference, table stakes, anti-features, README expectations
- `.planning/research/v1.6-curiosity-scenes/PITFALLS.md` § "Pitfall 7" — wrap, clearance, tunneling, motor stall, pool drain
- `.planning/research/v1.6-curiosity-scenes/STACK.md` § "2. Archimedes screw" — kinematic vane train, sealed housings, gap rule
- `.planning/research/v1.6-curiosity-scenes/SUMMARY.md` — "Screw wrap and clearance" resolution and screw caption wording
- `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md` — hooks seam table (screw row) and "Pattern 3: One live HUD control file per scene"

### Prior phase decisions
- `.planning/phases/46-cartesian-diver/46-CONTEXT.md` — the first scene through the checklist; D-07a high-gravity lesson; D-19..D-21 gate and baseline growth
- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-CONTEXT.md` — D-02..D-05 baseline file and gate semantics, D-11 file splits

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`: kinematic plates driven per step, thick cheeks against leaks, and the compile-time speed assert.
- `crates/liquidfun-wasm/src/scene/washing_machine.rs` and `web/src/catalog/washing-machine-speed.ts`: the live `drum-speed` rpm spinner (`ControlEffect::Live`).
- `crates/liquidfun-wasm/src/scene/theo_jansen.rs`: the live `motor-direction` forward and reverse preset, with tests in `theo_jansen/tests.rs`.
- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` and `stacked_drip.rs`: side return shafts that recirculate water without teleporting it.
- `crates/liquidfun-wasm/src/scene/cartesian_diver.rs` and `cartesian_diver/`: the latest curiosity scene, its tests and module layout.
- `web/src/catalog/scene-record-shared.ts`: `runtimePreset`, `option`, `sceneSource` and `withGravitySlider`.

### Established Patterns
- `SceneHooks` (`crates/liquidfun-wasm/src/scene.rs`): store intent in hooks (`apply_control`) and apply motion in `on_advance` once per fixed step.
- Release-mode native scene tests with tolerance-based outcome assertions. Debug builds diverge from release, and rotating bodies diverge between native and wasm.
- New tests go into the existing `liquidfun-wasm` lib test binary. A new test binary triggers about 2 minutes of macOS scanning.
- Files stay under 628 lines. Use the `foo.rs` + `foo/` module layout.

### Integration Points
- Rust: `scene.rs` (`mod`, `SceneId`, `build_scene`, `parse_scene_id`, `ALL_SCENE_IDS`), `gravity_slider.rs` `variant_index`, `scene_spot.rs` `SURVEY_SCENES`, `scene_spot/tests.rs`, `session/tests.rs`.
- Web: `scenes.ts` `SCENE_IDS`, `scene-records-curiosity.ts`, `previews-curiosity.tsx`, the new `archimedes-screw-speed.ts`, `portrait-bounds.ts`, `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts`, the tests listed in T1–T4, and e2e `player-helpers.ts` and `player.spec.ts`.

</code_context>

<specifics>
## Specific Ideas

- The defining moment: separate pockets ride up the tube between slanted vanes and pour into the basin, and the basin visibly gains water and then spills back down the chute.
- Reverse is the fun twist: the same vanes scoop water out of the basin and carry it back down.
- A direct "lift force" on particles is an anti-feature. Only the vanes move the water.

</specifics>

<deferred>
## Deferred Ideas

- A sand or powder material and a tilt-angle control with spill-back above about 40° (SCREW-03, future requirement).
- A rotating helix overlay drawn along the shaft (research P3 differentiator).

</deferred>

---

*Phase: 47-archimedes-screw*
*Context gathered: 2026-10-14*

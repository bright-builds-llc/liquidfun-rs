---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T18:57:28.122Z
---

# Phase 32: Liquid motion bubbler - Context

**Gathered:** 2026-09-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch a new playground scene: colored liquid drips through a narrow waist and turns a small wheel. Water Wheel stays the jet-driven paddle-wheel port. This scene is an original liquid fidget, not a port of a pinned LiquidFun test.

Motion stays in native `liquidfun` and `liquidfun-wasm`. SolidJS stays controls and rendering. This phase does not add the stacked drip fidget, sealed C++ parity, visitor controls, or a change to Water Wheel, Color Mixer, Liquid Timer, or Hydraulic Fountain.

</domain>

<decisions>
## Implementation Decisions

### Vessel
- **D-01:** Add a new scene. Do not replace, hide, or retune `water-wheel`. The catalog id is `liquid-bubbler`, appended after `wave-tank`. The title is Liquid Bubbler.
- **D-02:** The layout is an upper reservoir, one narrow static waist, and a lower chamber. Liquid leaves the reservoir by dripping through that waist. The wheel sits in the lower chamber, under the drip. Do not use Water Wheel's open basin and aimed jet, Liquid Timer's shelves, or a stack of reacting parts.

### Wheel and flow
- **D-03:** The wheel is a dynamic paddle wheel on a revolute joint with the motor off. Particles that fall through the waist hit the paddles and turn the wheel. Do not spin the wheel with a motor. Do not aim a particle jet at it.
- **D-04:** The drip continues for the whole play session. A quiet return lifts liquid back to the upper reservoir. The visible spectacle stays the waist and the wheel. The return is not a second piston show and does not add a stack of parts that each react when the drip reaches them. Reset restores the initial reservoir, the resting wheel, and the initial particle layout.

### Liquid color
- **D-05:** Use one plain water particle group with one distinct `ParticleColor`, so the drip reads as colored liquid. Do not add tensile, elastic, rigid, or color-mixing groups. Do not retune Color Mixer.
- **D-06:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

### Catalog, credits, and proof
- **D-07:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/liquid-bubbler`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-08:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the reservoir, the waist, the wheel, and the chamber walls fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `liquid-bubbler` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-09:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-10:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, after a bounded run, particles that began above the waist are below it, and the wheel angle has changed from its initial pose. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber sizes, waist gap, wheel radius, paddle count, particle radius, particle count, and the single particle color, as long as a drip through the waist visibly turns the wheel and the walls stay inside the portrait frame.
- How the return lift is built, as long as particles physically cross the waist before they turn the wheel, the return does not tunnel through the wheel, and the return does not become the spectacle.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a narrow waist with a wheel beneath a colored drip.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 32 one-line boundary: colored liquid drips through a narrow waist and turns a small wheel. The phase-detail stub still says "To be planned"; use this context as the scope, not that stub.
- `.planning/REQUIREMENTS.md` — v1.3 testbed-port requirements are complete. This original fidget is not a Water Wheel replacement. Out of scope: sealed parity, cutting particle counts, raising the 4-step cap.
- `.planning/PROJECT.md` — Missing engine behavior is in scope only when this scene cannot run without it. Honest experimental limits stay.
- `.planning/STATE.md` — Phase 31 is complete. Phase 32 is the next pending phase.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; hobby scope and independent AI review.
- `AGENTS.md` — README SVG plan coverage and portrait frame rules for every new catalog scene.

### Prior scene decisions
- `.planning/phases/31-sinusoidal-wave-tank/31-CONTEXT.md` — Original watch-first scene, one water group, static preview, portrait frame, README plan, Chromium smoke, no fake upstream credit.
- `.planning/phases/30-periodic-hydraulic-fountain/30-CONTEXT.md` — Timed piston spectacle. The bubbler return must not become a second fountain.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome. Water Wheel is the jet-driven wheel.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — `Static preview` caption and Reset honesty.

### Existing player code
- `web/src/catalog/scenes.ts` — `SCENE_IDS`. Current last id is `wave-tank`.
- `web/src/catalog/scene-records.ts` — Existing `water-wheel` record to leave intact.
- `web/src/catalog/portrait-bounds.ts` — Portrait frames required for every `SceneId`.
- `web/scripts/readme-svg/plans.ts` — README SVG plans required for every catalog scene id.
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — Motor-off revolute paddle wheel turned by a jet. Do not copy the jet or retune this scene.
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` — Timed piston. The return lift must stay quieter than this spectacle.
- `standards/languages/rust.md` — Safe Rust, error handling, and `foo.rs` plus `foo/` module shape.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome already in the player.
- `standards/core/architecture.md` — Physics stays in `liquidfun`; the WASM crate is the scene shell.
- `standards/core/testing.md` — Focused tests for the waist crossing, wheel rotation, and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids. Append `liquid-bubbler` after `wave-tank`.
- `web/src/catalog/scene-records.ts`: the existing `water-wheel` record is a jet-driven paddle wheel. Leave it alone.
- `web/src/catalog/previews.tsx`: static SVG previews. Add one captioned preview.
- `web/src/catalog/portrait-bounds.ts` and `web/scripts/readme-svg/plans.ts`: required companions for a new catalog id.
- `crates/liquidfun-wasm/src/scene/water_wheel.rs`: a dynamic wheel pinned with a motor-off revolute joint. Reuse that joint pattern, not the jet.
- `ParticleColor` on particle-group creation: one tint is enough for the colored drip.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only.
- Static catalog previews captioned `Static preview`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `liquid-bubbler` together so the catalog cannot advertise a scene the session cannot build.
- The waist, wheel, and return live in the WASM scene. SolidJS does not integrate the drip.
- `just web-player-smoke` is the Chromium proof. Extend it for the new scene and the existing catalog regression.
- Engine changes belong in `liquidfun` only if a motor-off revolute wheel and a static waist cannot express the drip.

</code_context>

<specifics>
## Specific Ideas

- The scene should read as a desk-toy bubbler: colored liquid gathers above a pinch, drips through it, and kicks a small wheel underneath.
- It should not look like Water Wheel, which is an open basin with an aimed jet and visitor jet controls.
- A motor that spins the wheel while liquid merely falls nearby, or a kinematic wheel that ignores the drip, is the wrong scene.

</specifics>

<deferred>
## Deferred Ideas

- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts, and each part reacts when the drip reaches it.
- Visitor waist, color, or wheel controls.
- Replacing or retuning Water Wheel, Color Mixer, Liquid Timer, or Hydraulic Fountain.
- Two-color mixing through the waist.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.

</deferred>

---

*Phase: 32-liquid-motion-bubbler*
*Context gathered: 2026-09-27*

---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-27T23:41:57.653Z
---

# Phase 33: Stacked drip fidget - Context

**Gathered:** 2026-09-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch a new playground scene: liquid drains through a vertical stack of moving trays, and each tray tips when the drip reaches it. Liquid Timer stays the tensile shelf drain. Liquid Bubbler stays the waist and the wheel. This scene is an original liquid fidget, not a port of a pinned LiquidFun test.

Motion stays in native `liquidfun` and `liquidfun-wasm`. SolidJS stays controls and rendering. This phase does not add visitor tray controls, sealed C++ parity, or a change to Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, or Liquid Tumbler.

</domain>

<decisions>
## Implementation Decisions

### Stack
- **D-01:** Add a new scene. Do not replace, hide, or retune `liquid-timer`, `liquid-bubbler`, `water-wheel`, `hydraulic-fountain`, or `liquid-tumbler`. The catalog id is `stacked-drip`, appended after `liquid-bubbler`. The title is Stacked Drip.
- **D-02:** The layout is a top reservoir and three dynamic trays stacked vertically. Liquid leaves the reservoir and lands on the upper tray, then the middle tray, then the lower tray. Do not use Liquid Timer's static shelves, Liquid Bubbler's single waist and wheel, or a column of wheels.

### Tray reaction
- **D-03:** Each tray is a dynamic body on a revolute joint with the motor off. Liquid that reaches a tray tips that tray and pours onto the next. Do not tip a tray with a timed motor, and do not animate the tip in SolidJS.
- **D-04:** A rest stop or joint limit holds each tray until liquid arrives, then the tray tips one way and pours. Trays do not free-spin before the drip. The upper tray moves before the lower trays, so the cascade reads from top to bottom.

### Repeat
- **D-05:** The cascade continues for the whole play session. A quiet return lifts liquid back to the top reservoir. The visible spectacle stays the stack of tipping trays. The return is not a piston show and does not add a wheel. Do not destroy or respawn particles to fake the loop. Reset restores the initial reservoir, the resting trays, and the initial particle layout.

### Liquid color
- **D-06:** Use one plain water particle group with one distinct `ParticleColor`, so the drip reads as colored liquid. Do not add tensile, elastic, rigid, or color-mixing groups. Do not retune Color Mixer or Liquid Timer.
- **D-07:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

### Catalog, credits, and proof
- **D-08:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/stacked-drip`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-09:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the reservoir, all three trays, and the walls fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `stacked-drip` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-10:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-11:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, after a bounded run, particles that began above the top tray are below the bottom tray, and each tray's angle has changed from its initial pose, with the upper tray moving before the lower trays. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-12:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact reservoir size, tray length, pivot placement, rest angle, joint limits, particle radius, particle count, and the single particle color, as long as three trays tip in top-to-bottom order and the walls stay inside the portrait frame.
- How the return lift is built, as long as particles physically reach each tray before that tray tips, the return does not become the spectacle, and particles are not deleted to fake the loop.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a vertical stack of trays under a colored drip.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 33 one-line boundary: liquid drains through a stack of moving parts, and each part reacts when the drip reaches it. The phase-detail stub still says "To be planned"; use this context as the scope, not that stub.
- `.planning/REQUIREMENTS.md` — v1.3 testbed-port requirements are complete. This original fidget is not a Liquid Timer or Liquid Bubbler replacement. Out of scope: sealed parity, cutting particle counts, raising the 4-step cap.
- `.planning/PROJECT.md` — Missing engine behavior is in scope only when this scene cannot run without it. Honest experimental limits stay.
- `.planning/STATE.md` — Phase 32 is complete. Phase 33 is the next pending phase.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; hobby scope and independent AI review.
- `AGENTS.md` — README SVG plan coverage and portrait frame rules for every new catalog scene.

### Prior scene decisions
- `.planning/phases/32-liquid-motion-bubbler/32-CONTEXT.md` — Original watch-first fidget: one waist, one motor-off wheel, quiet return, static preview, portrait frame, README plan, Chromium smoke, no fake upstream credit. Do not turn this stack into another wheel.
- `.planning/phases/30-periodic-hydraulic-fountain/30-CONTEXT.md` — Timed piston spectacle. The stacked-drip return must not become a second fountain.
- `.planning/phases/26-catalog-shell-and-basin-scenes/26-CONTEXT.md` — Liquid Timer is the static-shelf tensile drain. Leave it intact.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — `Static preview` caption and Reset honesty.

### Existing player code
- `web/src/catalog/scenes.ts` — `SCENE_IDS`. Current last id is `liquid-bubbler`.
- `web/src/catalog/scene-records.ts` — Existing `liquid-timer` and `liquid-bubbler` records to leave intact. `liquid-tumbler` is a real-size glass with a gravity slider.
- `web/src/catalog/portrait-bounds.ts` — Portrait frames required for every `SceneId`.
- `web/scripts/readme-svg/plans.ts` — README SVG plans required for every catalog scene id.
- `crates/liquidfun-wasm/src/scene/liquid_timer.rs` — Static shelves and a tensile drain. Do not copy that material or those shelves.
- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` — Motor-off revolute wheel and a quiet return. Reuse the joint and return idea, not the wheel.
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — Motor-off revolute paddle wheel. Do not add a wheel to this scene.
- `standards/languages/rust.md` — Safe Rust, error handling, and `foo.rs` plus `foo/` module shape.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome already in the player.
- `standards/core/architecture.md` — Physics stays in `liquidfun`; the WASM crate is the scene shell.
- `standards/core/testing.md` — Focused tests for the tray cascade and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids. Append `stacked-drip` after `liquid-bubbler`.
- `web/src/catalog/scene-records.ts`: `liquid-timer` is a shelf drain and `liquid-bubbler` is a waist-and-wheel fidget. Leave both alone.
- `web/src/catalog/previews.tsx`: static SVG previews. Add one captioned preview.
- `web/src/catalog/portrait-bounds.ts` and `web/scripts/readme-svg/plans.ts`: required companions for a new catalog id.
- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`: a motor-off revolute joint and a quiet return. Reuse those patterns for trays and the lift, not the wheel.
- `ParticleColor` on particle-group creation: one tint is enough for the colored drip.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only.
- Static catalog previews captioned `Static preview`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `stacked-drip` together so the catalog cannot advertise a scene the session cannot build.
- The trays and the return live in the WASM scene. SolidJS does not integrate the drip.
- `just web-player-smoke` is the Chromium proof. Extend it for the new scene and the existing catalog regression.
- Engine changes belong in `liquidfun` only if a motor-off revolute tray and a static reservoir cannot express the cascade.

</code_context>

<specifics>
## Specific Ideas

- The scene should read as a desk-toy cascade: colored liquid leaves a top reservoir, tips one tray, then the next, then the next.
- It should not look like Liquid Timer, which is a tensile drain through fixed shelves, or Liquid Bubbler, which is one pinch and one wheel.
- A motor that tips the trays on a clock, or a kinematic animation that ignores the liquid, is the wrong scene.

</specifics>

<deferred>
## Deferred Ideas

- Visitor tray, color, or speed controls.
- Replacing or retuning Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, or Liquid Tumbler.
- A stack of wheels, or more than three reacting parts.
- Two-color mixing through the trays.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.

</deferred>

---

*Phase: 33-stacked-drip-fidget*
*Context gathered: 2026-09-27*

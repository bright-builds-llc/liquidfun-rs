---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-09-27T14-01-51
generated_at: 2026-09-27T14:02:18.013Z
---

# Phase 30: Periodic hydraulic fountain - Context

**Gathered:** 2026-09-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch a new playground scene in which a timed piston squeezes a liquid reservoir and that liquid travels into a different chamber. The existing Fountain scene stays an aimed particle emitter. This scene is original hydraulics, not a port of a pinned LiquidFun test.

Motion stays in native `liquidfun` and `liquidfun-wasm`. SolidJS stays controls and rendering. This phase does not add the sinusoidal wave tank, the liquid-motion bubbler, the stacked drip fidget, sealed C++ parity, or a change to the existing Fountain controls.

</domain>

<decisions>
## Implementation Decisions

### Travel layout
- **D-01:** Add a new scene. Do not replace, hide, or retune `fountain`. The catalog id is `hydraulic-fountain`, appended after `sparky`.
- **D-02:** The layout is two chambers joined by a throat. One chamber holds the liquid and a piston. The other chamber is where that liquid arrives. A cycle is recognizable when liquid that started on the piston side is later on the fountain side.
- **D-03:** The piston advances and then retracts on a repeating schedule. Liquid is not destroyed or spawned to fake the jet. The existing Fountain emitter is not reused.

### Drive
- **D-04:** Drive the piston with a prismatic joint whose motor speed is rewritten from simulation time, the same kind of live motor write Wave Machine already uses for its revolute joint. Do not teleport a kinematic body through the liquid.
- **D-05:** The period is built in. The scene is watch-first: play, pause, and reset only. Do not add period, stroke, or aim controls. Reset restores the initial piston pose and the initial liquid layout.

### Liquid
- **D-06:** Use one plain water particle group in the piston chamber. The spectacle is the squeeze and the crossing, not a new material flag. Do not add tensile, elastic, or rigid groups in this phase.
- **D-07:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

### Catalog, credits, and proof
- **D-08:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/hydraulic-fountain`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-09:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the chambers, throat, and piston fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `hydraulic-fountain` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-10:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-11:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that, within one period, particles that began in the piston chamber are present on the fountain side of the throat. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-12:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber sizes, throat width, piston stroke, period, particle radius, and particle count, as long as one cycle visibly moves liquid from the piston chamber through the throat and the walls stay inside the portrait frame.
- Whether liquid returns through a low passage or by falling back when the piston retracts, as long as the motion repeats and particles are not deleted to fake the jet.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a piston, a throat, and two chambers.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 30 one-line boundary: a timed piston squeezes liquid so it travels elsewhere. The phase-detail stub still says "To be planned"; use this context as the scope, not that stub.
- `.planning/REQUIREMENTS.md` — v1.3 requirements for the twelve testbed ports are complete. This original scene is not PLAY-03. Out of scope: replacing Fountain, sealed parity, cutting particle counts, raising the 4-step cap.
- `.planning/PROJECT.md` — Existing Fountain stays an original aimed scene. Missing engine behavior is in scope only when this scene cannot run without it.
- `.planning/STATE.md` — Phases 26–29 are complete. Phase 30 is the next pending phase.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; hobby scope and independent AI review.
- `AGENTS.md` — README SVG plan coverage and portrait frame rules for every new catalog scene.

### Prior scene decisions
- `.planning/phases/29-sparky-drawing-and-full-catalog/29-CONTEXT.md` — Append-only catalog, watch-first default, static previews, 4-step cap, Chromium smoke, no fake upstream credit.
- `.planning/phases/28-interaction-seams/28-CONTEXT.md` — Live motor writes from simulation time (Wave Machine) and prismatic rails (Soup Stirrer).
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome. Fountain is the aimed emitter this phase must not replace.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — `Static preview` caption and Reset honesty.

### Existing player code
- `web/src/catalog/scenes.ts` — `SCENE_IDS`. Current last id is `sparky`.
- `web/src/catalog/scene-records.ts` — Existing `fountain` record to leave intact.
- `web/src/catalog/portrait-bounds.ts` — Portrait frames required for every `SceneId`.
- `web/scripts/readme-svg/plans.ts` — README SVG plans required for every catalog scene id.
- `crates/liquidfun-wasm/src/scene/wave_machine.rs` — Live motor speed written from simulation time.
- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs` — Prismatic rail pattern.
- `crates/liquidfun/src/world/joint/prismatic.rs` — `set_prismatic_motor_speed` and limit setters.
- `standards/languages/rust.md` — Safe Rust, error handling, and module shape.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome already in the player.
- `standards/core/architecture.md` — Physics stays in `liquidfun`; the WASM crate is the scene shell.
- `standards/core/testing.md` — Focused tests for piston travel and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids. Append `hydraulic-fountain` after `sparky`.
- `web/src/catalog/scene-records.ts`: the existing `fountain` record is an aimed stream with emission, speed, and aim controls. Leave it alone.
- `web/src/catalog/previews.tsx`: static SVG previews. Add one captioned preview.
- `web/src/catalog/portrait-bounds.ts` and `web/scripts/readme-svg/plans.ts`: required companions for a new catalog id.
- `crates/liquidfun-wasm/src/scene/wave_machine.rs`: rewrites a joint motor from simulation time inside the scene step.
- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs`: prismatic joint on a ground rail.
- `World::set_prismatic_motor_speed`: public motor write for the piston.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only.
- Static catalog previews captioned `Static preview`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `hydraulic-fountain` together so the catalog cannot advertise a scene the session cannot build.
- The piston schedule lives in the WASM scene step, beside the Wave Machine motor write. SolidJS does not integrate the period.
- `just web-player-smoke` is the Chromium proof. Extend it for the new scene and the existing catalog regression.
- Engine changes belong in `liquidfun` only if a prismatic motor cannot express the periodic stroke.

</code_context>

<specifics>
## Specific Ideas

- The scene should read as a pump: the piston closes, liquid is forced through a narrow throat, and liquid shows up in the other chamber. Then the piston opens and the cycle repeats.
- It should not look like the existing Fountain, which is a visitor-aimed particle stream into a bowl.
- A resting piston with a particle emitter hidden inside it is the wrong scene.

</specifics>

<deferred>
## Deferred Ideas

- Phase 31 sinusoidal wave tank: a still pool whose end platform rises and falls.
- Phase 32 liquid motion bubbler: colored liquid drips through a narrow waist and turns a wheel.
- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts.
- Visitor aim, emission rate, or launch speed. Those controls belong to the existing Fountain.
- A return pipe elaborate enough to be its own machine, if a simple retract-and-repeat cycle already shows travel.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.

</deferred>

---

*Phase: 30-periodic-hydraulic-fountain*
*Context gathered: 2026-09-27*

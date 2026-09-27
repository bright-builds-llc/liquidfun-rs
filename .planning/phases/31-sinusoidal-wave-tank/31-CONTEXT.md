---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-09-27T17-22-43
generated_at: 2026-09-27T17:24:09.479Z
---

# Phase 31: Sinusoidal wave tank - Context

**Gathered:** 2026-09-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch a new playground scene: a still pool whose end platform rises and falls and sends waves toward the far wall. Wave Machine stays the rocking-container port. This scene is original playground hydraulics, not a port of a pinned LiquidFun test.

Motion stays in native `liquidfun` and `liquidfun-wasm`. SolidJS stays controls and rendering. This phase does not add the liquid-motion bubbler, the stacked drip fidget, sealed C++ parity, visitor amplitude controls, or a change to Wave Machine.

</domain>

<decisions>
## Implementation Decisions

### Pool and platform
- **D-01:** Add a new scene. Do not replace, hide, or retune `wave-machine`. The catalog id is `wave-tank`, appended after `hydraulic-fountain`. The title is Wave Tank.
- **D-02:** The layout is one rectangular pool of still water. A platform at one end rises and falls. The rest of the floor and the far wall stay fixed. A cycle is recognizable when that rise and fall sends a wave across the pool toward the far wall.

### Drive
- **D-03:** Drive the platform with a prismatic joint whose motor speed follows a sinusoid of simulation time, the same kind of live motor write Hydraulic Fountain uses for its prismatic joint and Wave Machine uses for its revolute joint. Do not teleport a kinematic body through the liquid. Do not rock the whole tank.
- **D-04:** The period and stroke are built in. The scene is watch-first: play, pause, and reset only. Do not add period, amplitude, or stroke controls. Reset restores the initial platform pose and the initial still-pool layout.

### Liquid
- **D-05:** Use one plain water particle group. The spectacle is the traveling wave, not a new material flag. Do not add tensile, elastic, or rigid groups in this phase.
- **D-06:** Keep `MAX_ADVANCE_STEPS` at 4. Do not cut particle count to look smoother. If the radius or count must shrink so the scene fits the playground, record that as an explicit playground adaptation.

### Catalog, credits, and proof
- **D-07:** The catalog entry is `ready: true` with a title, a one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Hash route is `#/scene/wave-tank`. Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`.
- **D-08:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` so the pool, the moving end, and the far wall fill a phone canvas, the walls are inside the frame, and the subject stays clear of the title and transport. Add `wave-tank` to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-09:** Credits use the Phase 18 chrome. The implementation link stays host-locked to this scene module. Inspiration says this is an original playground scene. Do not cite a pinned LiquidFun test. Copy stays an experimental native scene with recognizable behavior, not sealed parity.
- **D-10:** Prove locally with the existing Chromium `just web-player-smoke` suite. The new scene opens, plays, pauses, and resets. Scenes already in the catalog still open and run. A native test shows that the pool starts as a still band and that, after the platform has completed at least a half cycle, particles that began in a sample near the far wall have moved vertically. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact pool length, platform width, stroke, period, particle radius, and particle count, as long as one cycle visibly sends a wave from the moving end toward the far wall and the walls stay inside the portrait frame.
- Whether the moving piece is a short floor slab at one end or a vertical face that translates with that slab, as long as the motion is a rise and fall of the end platform and the rest of the pool floor stays still.
- Static SVG preview artwork, as long as the preview is captioned `Static preview` and shows a pool with one end platform displaced from the resting floor.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 31 one-line boundary: a still pool whose end platform rises and falls and sends waves. The phase-detail stub still says "To be planned"; use this context as the scope, not that stub.
- `.planning/REQUIREMENTS.md` — v1.3 testbed-port requirements are complete. This original scene is not PLAY-03 and must not replace Wave Machine. Out of scope: sealed parity, cutting particle counts, raising the 4-step cap.
- `.planning/PROJECT.md` — Missing engine behavior is in scope only when this scene cannot run without it. Honest experimental limits stay.
- `.planning/STATE.md` — Phases 26–30 are complete. Phase 31 is the next pending phase.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; hobby scope and independent AI review.
- `AGENTS.md` — README SVG plan coverage and portrait frame rules for every new catalog scene.

### Prior scene decisions
- `.planning/phases/30-periodic-hydraulic-fountain/30-CONTEXT.md` — Original scene, watch-first, prismatic motor from simulation time, one water group, static preview, portrait frame, README plan, Chromium smoke, no fake upstream credit.
- `.planning/phases/28-interaction-seams/28-CONTEXT.md` — Wave Machine is the rocking tank. Live motor writes stay in the scene step.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — `Static preview` caption and Reset honesty.

### Existing player code
- `web/src/catalog/scenes.ts` — `SCENE_IDS`. Current last id is `hydraulic-fountain`.
- `web/src/catalog/scene-records.ts` — Existing `wave-machine` record to leave intact.
- `web/src/catalog/portrait-bounds.ts` — Portrait frames required for every `SceneId`.
- `web/scripts/readme-svg/plans.ts` — README SVG plans required for every catalog scene id.
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` — Prismatic motor speed written from simulation time.
- `crates/liquidfun-wasm/src/scene/wave_machine.rs` — Revolute motor write for the rocking tank this phase must not copy or retune.
- `crates/liquidfun/src/world/joint/prismatic.rs` — `set_prismatic_motor_speed` and limit setters.
- `standards/languages/rust.md` — Safe Rust, error handling, and `foo.rs` plus `foo/` module shape.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome already in the player.
- `standards/core/architecture.md` — Physics stays in `liquidfun`; the WASM crate is the scene shell.
- `standards/core/testing.md` — Focused tests for wave travel and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids. Append `wave-tank` after `hydraulic-fountain`.
- `web/src/catalog/scene-records.ts`: the existing `wave-machine` record is a rocking tank. Leave it alone.
- `web/src/catalog/previews.tsx`: static SVG previews. Add one captioned preview.
- `web/src/catalog/portrait-bounds.ts` and `web/scripts/readme-svg/plans.ts`: required companions for a new catalog id.
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`: rewrites a prismatic motor from simulation time inside the scene step.
- `World::set_prismatic_motor_speed`: public motor write for the end platform.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only.
- Static catalog previews captioned `Static preview`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `wave-tank` together so the catalog cannot advertise a scene the session cannot build.
- The sinusoid lives in the WASM scene step, beside the Hydraulic Fountain motor write. SolidJS does not integrate the period.
- `just web-player-smoke` is the Chromium proof. Extend it for the new scene and the existing catalog regression.
- Engine changes belong in `liquidfun` only if a prismatic motor cannot express the sinusoidal stroke.

</code_context>

<specifics>
## Specific Ideas

- The scene should read as a wave tank: the water starts flat, one end lifts and drops on a smooth repeating curve, and a wave travels to the other end.
- It should not look like Wave Machine, which rocks the whole container.
- A kinematic slab that jumps through the particles, or a particle emitter that fakes ripples, is the wrong scene.

</specifics>

<deferred>
## Deferred Ideas

- Phase 32 liquid motion bubbler: colored liquid drips through a narrow waist and turns a wheel.
- Phase 33 stacked drip fidget: liquid drains through a stack of moving parts.
- Visitor amplitude, period, or stroke controls.
- Replacing or retuning Wave Machine.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.

</deferred>

---

*Phase: 31-sinusoidal-wave-tank*
*Context gathered: 2026-09-27*

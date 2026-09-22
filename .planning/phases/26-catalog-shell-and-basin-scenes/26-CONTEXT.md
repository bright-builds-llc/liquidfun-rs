---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:02:54.967Z
---

# Phase 26: Catalog shell and basin scenes - Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can use the existing shared catalog and player with the first watch-first testbed ports — Particles and Liquid Timer — while Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel stay available and runnable.

This phase covers PLAY-02, PLAY-03, BASIN-01, and BASIN-02 for those two scenes only. It does not add the other ten testbed scenes, particle-type presets, sealed C++ differential evidence, a second player, or a crate/tag release.

</domain>

<decisions>
## Implementation Decisions

### Catalog shell
- **D-01:** Append Particles (`particles`) and Liquid Timer (`liquid-timer`) to the existing `DemoNavigation` sidebar and mobile drawer. Keep the current six scene ids and their current order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

### Watch-first controls
- **D-04:** Particles and Liquid Timer expose play, pause, and Reset only. No water-amount, gravity, or particle-type presets in this phase. PRESET-01 stays future work.
- **D-05:** Interaction copy says these scenes are watch-first. Do not add pointer drag, click impulse, or keyboard material modes.
- **D-06:** Reset disposes the session and recreates the documented initial layout. Follow the Phase 20 remount rule if any construction selects exist later: visible preset labels return to `DEFAULT_PRESET_VALUES`.

### Pinned-test credits
- **D-07:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-08:** Inspiration for Particles cites pinned `testParticles.js` and `Particles.h` at commit `7f20402173fd143a3988c921bc384459c6a858f2`. Inspiration for Liquid Timer cites pinned `testLiquidTimer.js` and `LiquidTimer.h` at that same commit. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-09:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.

### Recognizable basin layouts
- **D-10:** Particles must show an open basin (floor plus slanted side walls), a falling water group, and one dynamic ball that drops into the water. Match `testParticles.js` / `Particles.h` closely enough to be recognizable: water circle and a rigid ball above it. Damping tweaks and the C++ particle-type picker are chrome, not this phase.
- **D-11:** Liquid Timer must show tensile and viscous liquid draining through a vertical gap and zig-zag shelves into four bottom columns. Match `testLiquidTimer.js` / `LiquidTimer.h` closely enough to be recognizable. Alternate particle-parameter sets are chrome (PRESET-01).
- **D-12:** Author both scenes in `liquidfun-wasm` on the public `liquidfun` API. Add engine behavior only when a listed scene cannot run without it. Do not cut particle counts or raise the 4-step catch-up cap to look smoother. Do not import the desktop testbed or replay diagnostic protocol recipes as gallery demos.

### Shared session and proof
- **D-13:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. `MAX_ADVANCE_STEPS` stays 4.
- **D-14:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so Particles and Liquid Timer open, play, pause, and reset, and so the original six scenes still open and run. Do not add Firefox/Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-15:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- Whether tensile and viscous strengths use existing `ParticleSystemDef` fields or a narrowly justified engine addition when the scene cannot drain recognizably without it.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.

### Folded Todos
None.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 26 goal, PLAY-02, PLAY-03, BASIN-01, BASIN-02, and the five success criteria. Later phases own materials, interaction seams, Sparky, and Drawing.
- `.planning/REQUIREMENTS.md` — PLAY-02, PLAY-03, BASIN-01, BASIN-02. Out of scope: replacing the six scenes, copying lfjs chrome, sealed parity, cutting particle counts, raising the 4-step cap, a second engine, publication. Future: DRAW-02, PRESET-01, PARITY-01, BOX2D-01.
- `.planning/PROJECT.md` — v1.3 recognizable ports in the existing playground; six current scenes stay; missing engine behavior only where a listed scene cannot run without it.
- `.planning/STATE.md` — Current focus is Phase 26; v1.2 performance claims stay archived.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; `skipLibCheck` for Kobalte declarations; hobby scope and independent AI review.

### Scene recognition research
- `.planning/research/FEATURES.md` — Particles and Liquid Timer recognition cards (setup, must-see, chrome). Catalog and credit expectations for new scenes.
- `.planning/research/ARCHITECTURE.md` — One module per scene under `crates/liquidfun-wasm/src/scene/`; `web/src/catalog/scenes.ts` grows; engine stays renderer-free.
- `.planning/research/STACK.md` — Liquid Timer default `tensile | viscous` on existing `ParticleSystemDef`; pinned JS and C++ paths.
- `.planning/research/PITFALLS.md` — Pinned JS testbed paths and the risk of treating JS chrome as required behavior.

### Pinned upstream tests
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testParticles.js` — JS Particles setup at pin `7f20402173fd143a3988c921bc384459c6a858f2`.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/Particles.h` — C++ cross-check for Particles.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testLiquidTimer.js` — JS Liquid Timer setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/LiquidTimer.h` — C++ cross-check for Liquid Timer.

### Existing player decisions and code
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-CONTEXT.md` — Hash routes, one session, hidden-tab catch-up cap of four steps, site chrome.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome, scene factory.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — Sidebar/drawer previews, `Static preview` caption, Reset label honesty, no `.catalog-card`.
- `web/src/catalog/scenes.ts` — `SCENE_IDS`, scene records, controls, and credits to extend.
- `web/src/components/scene-credits.ts` — Host-locked implementation links.
- `standards/languages/rust.md` — Safe Rust, error handling, and module shape for scene and engine code.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome, source disclosure, and provenance already established in the player.
- `standards/core/architecture.md` — Scene construction stays in the WASM shell; physics stays in `liquidfun`.
- `standards/core/testing.md` — Focused tests for new scene construction and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof for this phase.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids, titles, descriptions, controls, and credits. Append two records; do not replace the six.
- `web/src/components/scene-credits.ts`: implementation, inspiration, and notices labels. Reuse for pinned-test inspiration links.
- `crates/liquidfun-wasm/src/scene/`: one module per existing demo, built through `SessionCore::create(SceneId::...)`.
- Kobalte Dialog drawer and `DemoNavigation` list items from Phase 20, including static SVG previews.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Construction changes recreate the world; these two scenes have no construction presets.
- Static catalog previews captioned `Static preview`. Dark scoped CSS. No MysticUI or Tailwind.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `particles` and `liquid-timer` together so the catalog cannot advertise a scene the session cannot build.
- `just web-player-smoke` is the Chromium proof. Extend it for the two new scenes and keep the six-scene regression.
- Engine changes belong in `liquidfun` only if tensile/viscous drain or the falling ball cannot be expressed with the current public API.

</code_context>

<specifics>
## Specific Ideas

Particles is the classic splash: water falls in an open basin and a ball drops into it. Liquid Timer is the hourglass: tensile, viscous liquid drains through shelves into bottom columns. Both are watch-first. The existing six scenes stay in the same player.

</specifics>

<deferred>
## Deferred Ideas

- Surface Tension, Elastic Particles, and Rigid Particles — Phase 27.
- Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen — Phase 28.
- Sparky, Drawing Particles, and the full twelve-scene catalog claim (PLAY-01) — Phase 29.
- Liquid Timer extra particle-type presets (PRESET-01) and the Drawing Particles keyboard matrix (DRAW-02).
- Sealed per-scene differential evidence (PARITY-01) and commented-out Box2D-only tests (BOX2D-01).
- Optional pointer nudge on watch-first scenes. Not required for recognizability.
- Cross-links between related scenes (Particles as the baseline for later material scenes).

</deferred>

---

*Phase: 26-catalog-shell-and-basin-scenes*
*Context gathered: 2026-09-21*

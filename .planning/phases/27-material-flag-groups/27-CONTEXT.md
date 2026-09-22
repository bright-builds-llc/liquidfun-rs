---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:15:07.461Z
---

# Phase 27: Material flag groups - Context

**Gathered:** 2026-09-22
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch Surface Tension, Elastic Particles, and Rigid Particles as recognizable flag-group ports in the existing playground, beside the six original scenes plus Particles and Liquid Timer. This phase delivers catalog entries, credits, and native scenes for MAT-01, MAT-02, and MAT-03. It does not add Soup, Impulse, Wave Machine, Theo Jansen, Sparky, Drawing Particles, material pickers, or a sealed parity claim.

</domain>

<decisions>
## Implementation Decisions

### Catalog shell
- **D-01:** Append Surface Tension (`surface-tension`), Elastic Particles (`elastic-particles`), and Rigid Particles (`rigid-particles`) after the current eight scene ids. Keep the existing order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

### Watch-first controls
- **D-04:** Surface Tension, Elastic Particles, and Rigid Particles expose play, pause, and Reset only. No particle-type, stiffness, or color pickers in this phase. PRESET-01 stays future work.
- **D-05:** Interaction copy says these scenes are watch-first. Do not add pointer drag, click impulse, or keyboard material modes.
- **D-06:** Reset disposes the session and recreates the documented initial layout.

### Recognizable flag-group layouts
- **D-07:** Surface Tension must show a basin, three tensile and color-mixing groups (red circle, green circle, blue box), and one falling dynamic ball. The groups bead, and color bleeds when the ball hits them. Match `testSurfaceTension.js` / `ParticlesSurfaceTension.h` closely enough to be recognizable. Extra particle types are chrome.
- **D-08:** Elastic Particles must show a basin, a red spring-and-solid circle, a green elastic-and-solid circle, a blue elastic-and-solid box with angle and spin, and one falling dynamic circle. The three soft clumps deform when the ball hits them. Match `testElasticParticles.js` / `ElasticParticles.h` closely enough to be recognizable. Exact recovery stiffness and upstream JS bugs are chrome; recognizable soft motion beats bit-exact recovery.
- **D-09:** Rigid Particles must use the same basin family as Elastic Particles, with three colored rigid-and-solid groups (circles plus a spinning box) and one falling ball. The clumps stay solid and do not stretch like jelly. Match `testRigidParticles.js` / `RigidParticles.h` closely enough to be recognizable. Bit-perfect rigid-solver parity is chrome.

### Pinned-test credits
- **D-10:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-11:** Inspiration for Surface Tension cites pinned `testSurfaceTension.js` and `ParticlesSurfaceTension.h` at commit `7f20402173fd143a3988c921bc384459c6a858f2`. Elastic Particles cites `testElasticParticles.js` and `ElasticParticles.h`. Rigid Particles cites `testRigidParticles.js` and `RigidParticles.h`. Same pin. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-12:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.

### Engine additions
- **D-13:** Author all three scenes in `liquidfun-wasm` on the public `liquidfun` API. Prefer existing `TENSILE`, `COLOR_MIXING`, `ELASTIC`, `SPRING`, `SOLID`, and `RIGID` paths. Color Mixer already proves color mixing; Jelly Drop already proves elastic groups. Add engine behavior only when a listed scene cannot show the required contrast without it.
- **D-14:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. Do not import the desktop testbed or replay diagnostic protocol recipes as gallery demos.

### Shared session and proof
- **D-15:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. `MAX_ADVANCE_STEPS` stays 4.
- **D-16:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so the three new scenes open, play, pause, and reset, and so the current eight scenes still open and run. Do not add Firefox/Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-17:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, group sizes, box angle, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- A private shared basin helper inside `liquidfun-wasm` when it prevents the three layouts from drifting, without a new public engine API.
- Whether a missing tensile, color-bleed, elastic, or rigid behavior needs a narrowly justified engine addition when the scene cannot show the required contrast without it.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.

### Folded Todos
None.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 27 goal, MAT-01, MAT-02, MAT-03, and the four success criteria. Phase 28 owns interaction seams. Phase 29 owns Sparky, Drawing, and the full twelve-scene catalog claim.
- `.planning/REQUIREMENTS.md` — MAT-01, MAT-02, MAT-03. Out of scope for this phase: ACT-*, FX-*, PLAY-01's full twelve-scene claim, PRESET-01, DRAW-02, PARITY-01, BOX2D-01.
- `.planning/PROJECT.md` — v1.3 recognizable ports in the existing playground; current scenes stay; missing engine behavior only where a listed scene cannot run without it.
- `.planning/STATE.md` — Phase 26 is complete; current focus moves to Phase 27.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; `skipLibCheck` for Kobalte declarations; hobby scope and independent AI review.

### Scene recognition research
- `.planning/research/FEATURES.md` — Surface Tension, Elastic Particles, and Rigid Particles recognition cards (setup, must-see, chrome).
- `.planning/research/ARCHITECTURE.md` — Flag and group mapping for the three scenes; one module per scene under `crates/liquidfun-wasm/src/scene/`.
- `.planning/research/STACK.md` — Existing `TENSILE`, `COLOR_MIXING`, `ELASTIC`, `SPRING`, and `RIGID | SOLID` coverage; pinned JS and C++ paths.
- `.planning/research/PITFALLS.md` — Pinned JS testbed paths and the risk of treating JS chrome as required behavior.

### Pinned upstream tests
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testSurfaceTension.js` — JS Surface Tension setup at pin `7f20402173fd143a3988c921bc384459c6a858f2`.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/ParticlesSurfaceTension.h` — C++ cross-check for Surface Tension.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testElasticParticles.js` — JS Elastic Particles setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/ElasticParticles.h` — C++ cross-check for Elastic Particles.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testRigidParticles.js` — JS Rigid Particles setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/RigidParticles.h` — C++ cross-check for Rigid Particles.

### Existing player decisions and code
- `.planning/phases/26-catalog-shell-and-basin-scenes/26-CONTEXT.md` — Append-only catalog, watch-first controls, pinned-test credits, 4-step cap, Chromium smoke.
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
- `web/src/catalog/scenes.ts`: scene ids, titles, descriptions, controls, and credits. Append three records; do not replace the current eight.
- `web/src/catalog/previews.tsx`: static SVG previews, including Particles and Liquid Timer.
- `web/src/components/scene-credits.ts`: implementation, inspiration, and notices labels.
- `crates/liquidfun-wasm/src/scene/`: one module per demo, including `jelly_drop.rs` (elastic/spring) and color-mixing scenes. Built through the checked scene-id factory.
- `ParticleFlags::TENSILE`, `COLOR_MIXING`, `ELASTIC`, `SPRING`, and `ParticleGroupFlags::SOLID` / `RIGID` already exist in `liquidfun`.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only.
- Static catalog previews captioned `Static preview`. Dark scoped CSS. No MysticUI or Tailwind.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `surface-tension`, `elastic-particles`, and `rigid-particles` together so the catalog cannot advertise a scene the session cannot build.
- `just web-player-smoke` is the Chromium proof. Extend it for the three new scenes and keep the eight-scene regression.
- Engine changes belong in `liquidfun` only if tensile beading, color bleed, elastic deformation, or rigid solidity cannot be expressed with the current public API.

</code_context>

<specifics>
## Specific Ideas

Surface Tension is three colored tensile blobs that bead and bleed color when a ball hits them. Elastic Particles is three soft clumps that deform under a falling ball. Rigid Particles is the same basin family with clumps that stay solid. All three are watch-first. The current eight scenes stay in the same player.

</specifics>

<deferred>
## Deferred Ideas

- Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen — Phase 28.
- Sparky, Drawing Particles, and the full twelve-scene catalog claim (PLAY-01) — Phase 29.
- Material and particle-type pickers (PRESET-01) and the Drawing Particles keyboard matrix (DRAW-02).
- Sealed per-scene differential evidence (PARITY-01) and commented-out Box2D-only tests (BOX2D-01).
- Optional pointer nudge on watch-first scenes. Not required for recognizability.
- Cross-links between related scenes (Surface Tension and Color Mixer, Elastic Particles and Jelly Drop).

</deferred>

---

*Phase: 27-material-flag-groups*
*Context gathered: 2026-09-22*

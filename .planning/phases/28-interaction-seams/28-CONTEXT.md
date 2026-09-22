---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T12:57:25.195Z
---

# Phase 28: Interaction seams - Context

**Gathered:** 2026-09-22
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can run Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen in the existing playground. Soup shows liquid holding floating solid bits. Soup Stirrer keeps stirring that soup and lets the visitor free the paddle from its rail or put it back. Impulse lets the visitor click or tap to shove the whole particle blob. Wave Machine rocks on its own and sloshes the water inside. Theo Jansen walks under a particle load and the visitor can reverse its motor.

Motion comes from native destroy-in-shape, group force or impulse, and live revolute motors. SolidJS stays controls and rendering. Sparky, Drawing Particles, the full twelve-scene catalog claim, and extra particle-type presets stay in later phases.

</domain>

<decisions>
## Implementation Decisions

### Catalog shell
- **D-01:** Append Soup (`soup`), Soup Stirrer (`soup-stirrer`), Impulse (`impulse`), Wave Machine (`wave-machine`), and Theo Jansen (`theo-jansen`) after the current eleven scene ids. Keep the existing order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

### Per-scene controls
- **D-04:** Soup and Wave Machine expose play, pause, and Reset only. They are watch-first.
- **D-05:** Soup Stirrer adds one non-recreating action labeled `Toggle paddle rail`. The interaction hint says the action frees the paddle from its rail, and the next press puts it back. A click or tap on the canvas sends that same command. Reset disposes the session and restores the paddle on the rail.
- **D-06:** Impulse adds one non-recreating runtime preset `push-mode`, labeled `Push`, with values `force` (default) and `impulse`. A click or tap inside the box shoves the whole blob from its center toward the pointer using that mode. A click or tap outside the box does nothing. Reset restores `force`.
- **D-07:** Theo Jansen adds one non-recreating runtime preset `motor-direction`, labeled `Motor direction`, with values `forward` (default) and `reverse`. The walker starts moving forward under the particle load. Changing the preset flips the live revolute motor speed sign and does not recreate the world. Reset restores `forward`.
- **D-08:** Do not use freeglut key legends as the only controls. Keyboard access may mirror the labeled controls. Do not add pointer drag, material pickers, or particle-type presets. PRESET-01 stays future work.

### Recognizable layouts
- **D-09:** Soup must show a basin of liquid and floating solid bits: a circle, two squares, and edge noodles, with particles carved out from under those fixtures. Match `testSoup.js` / `Soup.h` closely enough to be recognizable. Broth with bobbing solids is the must-see. Particle-parameter chrome is out.
- **D-10:** Soup Stirrer must reuse that soup, then add a dynamic circle paddle, particles carved out under the paddle, a prismatic rail, and a per-step stirring force while the rail is attached. Match `testSoupStirrer.js` / `SoupStirrer.h` closely enough to be recognizable. Freeing the rail lets the paddle leave the rail. Putting it back constrains the paddle again.
- **D-11:** Impulse must show one particle group in a box. A pointer shove moves that group as one blob. Match `testImpulse.js` / `Impulse.h` closely enough to be recognizable. Default shove is group force. The labeled switch uses group linear impulse. Particle-type presets are chrome.
- **D-12:** Wave Machine must show a motorized revolute tank of four thin walls with water inside. Each advance sets motor speed from simulated time, `0.05 * cos(t) * π`, using step count times `dt`. Pause freezes `t`. Reset returns `t` to zero. Match `testWaveMachine.js` / `WaveMachine.h` closely enough to be recognizable. No pointer control.
- **D-13:** Theo Jansen must show a ground, end walls, a chassis and legs built from revolute and soft distance joints, collision filtering so the walker does not self-collide, a motorized revolute, and a particle load on top. The machine walks. Reverse changes walk direction. Match `testTheoJansen.js` / `TheoJansen.h` closely enough to be recognizable. The limit-toggle key is chrome. Do not weld the legs into rigid polygons.

### Native seams
- **D-14:** Author all five scenes in `liquidfun-wasm` on the public `liquidfun` API. Add engine behavior only when a listed scene cannot show the required behavior without it. Do not approximate carving, group shove, or motor speed in SolidJS.
- **D-15:** Soup and Soup Stirrer carving uses native destroy-in-shape before the first presented frame. Prefer a narrow public helper when the scene cannot express a particle-free pocket with existing queries. A focused test must show particles removed under a placed fixture. Do not enable destruction-by-age on these scenes.
- **D-16:** Impulse shove uses native particle-group force and linear impulse. A focused test must show the group's momentum change. Do not shove one particle, tween positions, or shake the camera.
- **D-17:** Wave Machine and Theo Jansen use the existing revolute motor mutators (`set_revolute_motor_enabled`, `set_revolute_motor_speed`) from scene `on_advance` and control handlers inside the session step. Do not set motor speed from a JavaScript animation frame outside that step. Water Wheel's motor-off revolute is the wrong pattern to copy.
- **D-18:** Soup Stirrer creates and destroys the prismatic joint through typed joint APIs. Stirring force stays in `on_advance` with the in-soup and max-speed guards while the rail is attached. Toggle twice and Reset must both be tested. Do not leak joints or leave an unconstrained paddle integrating unbounded force.
- **D-19:** Share one private soup builder inside `liquidfun-wasm` for Soup and Soup Stirrer. Stirrer composes that builder plus paddle, carve, prismatic joint, and stirring. Do not require the visitor to open Soup first, and do not fork a second soup layout.

### Counts, credits, and proof
- **D-20:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. `MAX_ADVANCE_STEPS` stays 4. If a radius must change for playground cost, record that as an explicit playground adaptation in the scene notes. Do not silently bump Wave Machine or Impulse radii.
- **D-21:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-22:** Inspiration cites the pinned JS and C++ tests at commit `7f20402173fd143a3988c921bc384459c6a858f2`: Soup cites `testSoup.js` and `Soup.h`; Soup Stirrer cites `testSoupStirrer.js` and `SoupStirrer.h`; Impulse cites `testImpulse.js` and `Impulse.h`; Wave Machine cites `testWaveMachine.js` and `WaveMachine.h`; Theo Jansen cites `testTheoJansen.js` and `TheoJansen.h`. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-23:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.
- **D-24:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. Pointer hits reuse the shared CSS-bound unproject path.
- **D-25:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so the five new scenes open, play, pause, and reset, and so the current eleven scenes still open and run. Interactive smoke covers one Soup Stirrer rail toggle, one Impulse click inside the box, and one Theo Jansen direction change. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-26:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, group sizes, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- Whether destroy-in-shape is a public `World` method or a scene-local helper over existing query and `mark_particle_for_destruction`, as long as the carve is native, runs before the first presented frame, and has a focused test.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.
- Exact button and preset wording, as long as free-or-restore, force-or-impulse, and forward-or-reverse stay obvious in the visible controls.
- Whether Theo Jansen also exposes a speed magnitude beside direction, as long as reverse works and the walker moves under the particle load.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 28 goal, ACT-01 through ACT-05, and the five success criteria. Phase 29 owns Sparky, Drawing, and the full twelve-scene catalog claim.
- `.planning/REQUIREMENTS.md` — ACT-01, ACT-02, ACT-03, ACT-04, ACT-05. Out of scope: FX-01, FX-02, PLAY-01's full twelve-scene claim, PRESET-01.
- `.planning/PROJECT.md` — v1.3 recognizable ports in the existing playground; current scenes stay; missing engine behavior only where a listed scene cannot run without it.
- `.planning/STATE.md` — Phase 27 is complete; current focus moves to Phase 28.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; `skipLibCheck` for Kobalte declarations; hobby scope and independent AI review.

### Scene recognition research
- `.planning/research/FEATURES.md` — Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen recognition cards, including must-see behavior and chrome.
- `.planning/research/PITFALLS.md` — Destroy-in-shape, live revolute motors, group impulse, Soup Stirrer joint lifecycle, particle-count, and player-interaction pitfalls.
- `.planning/research/ARCHITECTURE.md` — WASM scene module boundaries and the rule that SolidJS does not own physics.
- `.planning/research/STACK.md` — Pinned JS and C++ test paths and existing joint and particle coverage.

### Pinned upstream tests
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testSoup.js` — JS Soup setup at pin `7f20402173fd143a3988c921bc384459c6a858f2`.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/Soup.h` — C++ cross-check for Soup.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testSoupStirrer.js` — JS Soup Stirrer setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/SoupStirrer.h` — C++ cross-check for Soup Stirrer.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testImpulse.js` — JS Impulse setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/Impulse.h` — C++ cross-check for Impulse.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testWaveMachine.js` — JS Wave Machine setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/WaveMachine.h` — C++ cross-check for Wave Machine.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testTheoJansen.js` — JS Theo Jansen setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/TheoJansen.h` — C++ cross-check for Theo Jansen.

### Existing player decisions and code
- `.planning/phases/27-material-flag-groups/27-CONTEXT.md` — Append-only catalog, watch-first default, pinned-test credits, 4-step cap, Chromium smoke.
- `.planning/phases/26-catalog-shell-and-basin-scenes/26-CONTEXT.md` — Same shell and proof pattern for the first two v1.3 scenes.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome, scene factory.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — Sidebar and drawer previews, `Static preview` caption, Reset label honesty, no `.catalog-card`.
- `web/src/catalog/scenes.ts` — `SCENE_IDS`, scene records, `action()`, and runtime presets to extend.
- `web/src/components/scene-credits.ts` — Host-locked implementation links.
- `standards/languages/rust.md` — Safe Rust, error handling, and module shape for scene and engine code.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome, source disclosure, and provenance already established in the player.
- `standards/core/architecture.md` — Scene construction stays in the WASM shell; physics stays in `liquidfun`.
- `standards/core/testing.md` — Focused tests for new scene construction, native seams, and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof for this phase.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids, titles, descriptions, controls, and credits. Append five records; do not replace the current eleven. `action()` and non-recreating runtime presets already exist.
- `web/src/catalog/previews.tsx`: static SVG previews. Add five captioned previews.
- `web/src/components/scene-credits.ts`: implementation, inspiration, and notices labels.
- `crates/liquidfun-wasm/src/scene/`: one module per demo, including `water_wheel.rs` (revolute without a motor) and `basin_family.rs`. Built through the checked scene-id factory.
- `crates/liquidfun-wasm/src/lib.rs` `pointer_action` and session `on_advance`: the existing gesture and per-step hooks.
- `World::set_revolute_motor_speed` and `mark_particle_for_destruction` already exist. A named destroy-in-shape helper and group-wide force or impulse may still be missing.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only. Interactive scenes add labeled actions or runtime presets that do not recreate the world unless the hint says they do.
- Static catalog previews captioned `Static preview`. Dark scoped CSS. The shell uses the shadcn-solid override (Tailwind CSS v4, Kobalte, Corvu drawer) recorded in `standards-overrides.md`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `soup`, `soup-stirrer`, `impulse`, `wave-machine`, and `theo-jansen` together so the catalog cannot advertise a scene the session cannot build.
- `just web-player-smoke` is the Chromium proof. Extend it for the five new scenes, one gesture on each interactive scene, and keep the eleven-scene regression.
- Engine changes belong in `liquidfun` only if destroy-in-shape, group force or impulse, prismatic create or destroy, soft distance joints, or live motor mutation cannot be expressed with the current public API.

</code_context>

<specifics>
## Specific Ideas

- Soup should read as broth with bobbing solids, which requires carving particles out from under the fixtures rather than letting solids overlap the liquid.
- Soup Stirrer is Soup plus a paddle. Share one soup builder. Click and the labeled action must agree on freeing or restoring the rail.
- Impulse should slosh the blob as a unit toward the click. Force is the default; linear impulse is the labeled alternate.
- Wave Machine should keep rocking from simulated time, including across pause, so the 4-step cap stays honest.
- Theo Jansen should try to walk under particle rain. Reverse is the required control. The limit key is chrome.

</specifics>

<deferred>
## Deferred Ideas

- Impulse and Liquid Timer particle-type presets (PRESET-01) — later work.
- Sparky sparks, Drawing Particles paint, and the full twelve-scene catalog claim (PLAY-01, FX-01, FX-02) — Phase 29.
- Theo Jansen limit toggle and extra speed keys beyond forward and reverse.
- C++ particle-parameter panels on Impulse and Wave Machine.
- Cross-links from Soup to Soup Stirrer as a teaching path.
- Firefox, Safari, Linux qualification, and a live Pages redeploy — not this phase's gate.

</deferred>

---

*Phase: 28-interaction-seams*
*Context gathered: 2026-09-22*

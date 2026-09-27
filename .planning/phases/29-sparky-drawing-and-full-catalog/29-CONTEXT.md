---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T04:15:12.770Z
---

# Phase 29: Sparky, Drawing, and full catalog - Context

**Gathered:** 2026-09-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can watch Sparky sparks and paint Drawing Particles in the existing playground, and can open every scene in the twelve-scene testbed list while the original six stay available. Sparky shows colliding circles throwing fading particle sparks. Drawing Particles starts as an empty vessel and accepts paint, including one non-water material that looks different from plain water.

Motion and paint stay in native `liquidfun` and `liquidfun-wasm`. SolidJS stays controls and rendering. This phase does not add the full Drawing keyboard matrix, extra Impulse or Liquid Timer presets, sealed C++ parity, or the later fountain, wave-tank, bubbler, and drip scenes.

</domain>

<decisions>
## Implementation Decisions

### Sparky contact sparks
- **D-01:** Sparky is a watch-first scene: six dynamic circles in a tall chamber, particle radius about 0.25, walls from `testSparky.js` / `Sparky.h`. Circles are tagged sparkable. Walls are not. Match that layout closely enough to be recognizable.
- **D-02:** Sparks come from new body contacts, not a timer and not persistent resting contact. Record `ContactTransitionKind::Begin` (or the equivalent hook observation) only when at least one fixture belongs to a sparkable circle. Consume that buffer after the step unlocks and spawn VFX then. Do not create or destroy particle groups mid-step. Do not expand FFI or `WorldCommand` to spawn groups inside the step.
- **D-03:** Other scenes keep stepping with `NoDecisionHook`. Only Sparky installs a scene-scoped contact observer. One spark per begin in that step is the preferred spectacle. Upstream's single overwritten contact point per step is the minimum: a step with at least one new sparkable contact must throw at least one burst.
- **D-04:** Each burst is a powder particle group at the contact point, with outward velocities and a short lifetime (about 0.5–1 s). Fade colors in the Rust color buffer in batch. When the lifetime ends, destroy that group. A bounded ring reuses slots by destroying the previous group before overwrite. Do not enable destruction-by-age on the particle system. Reset drops every live VFX slot.

### Drawing Particles paint
- **D-05:** Drawing Particles starts empty inside the open box from `testDrawingParticles.js` / `DrawingParticles.h` (floor, side walls, ceiling strip; particle radius about 0.05). The vessel is recognizable before any paint.
- **D-06:** Pointer down and move paints. Each sample destroys particles inside a circle of radius 0.2 at the pointer, then creates a particle group there. Pointer up clears `lastGroup`. If the joined group is destroyed, clear `lastGroup` too. Reset disposes the session, clears `lastGroup`, and restores the default material. Reuse `World::destroy_particles_in_shape`. Do not enable destruction-by-age.
- **D-07:** A non-recreating runtime preset `material` has `water` (default, no special particle flags) and `elastic` (elastic particles in a solid group). Elastic strokes must look clumped rather than like flowing water. Consecutive samples join the previous group only while the group flags still match. Keyboard may mirror the labeled preset. Do not ship the freeglut key matrix as the only control.
- **D-08:** Canvas drag and the labeled preset are the interaction. A click with no move may leave a single stamp. Pointer samples outside the vessel still follow the destroy-then-create rule at the unprojected point; they do not need a separate outside-box rejection.

### Catalog claim, credits, and proof
- **D-09:** Append `drawing-particles`, then `sparky`, after the current scene ids. Keep the existing order, including `liquid-tumbler`. Hash routes stay `#/scene/{id}`. Each new entry is `ready: true` with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`.
- **D-10:** Add a portrait frame in `web/src/catalog/portrait-bounds.ts` for each new scene so the subject fills a phone canvas, includes the walls, and stays clear of the title and transport. Add each scene id to `web/scripts/readme-svg/plans.ts` so `assertReadmeSvgPlanCoverage` stays green. Do not treat README raster export as this phase's browser gate.
- **D-11:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, or filters. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.
- **D-12:** Keep the Phase 18 credit chrome. Implementation links stay host-locked to this repository's scene module. Inspiration cites the pinned JS and C++ tests at commit `7f20402173fd143a3988c921bc384459c6a858f2`: Drawing Particles cites `testDrawingParticles.js` and `DrawingParticles.h`; Sparky cites `testSparky.js` and `Sparky.h`.
- **D-13:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations. Scene docs use the same wording.
- **D-14:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. `MAX_ADVANCE_STEPS` stays 4. If a radius or VFX pool must shrink for playground cost, record that as an explicit playground adaptation. Raising a rigid-body frame cap is allowed only when Sparky's circles and walls cannot be captured otherwise, and that change must be tested.
- **D-15:** Prove locally with the existing Chromium `just web-player-smoke` suite. Both new scenes open, play, pause, and reset. The scenes already in the catalog still open and run. Interactive smoke covers one Drawing paint drag that leaves particles. Sparky smoke may stay watch-first; a native test must show a sparkable contact creates a powder group and that the group is gone after its lifetime. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-16:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact chamber coordinates, circle spawn jitter, VFX radius, speed, and lifetime ranges, as long as collisions throw short-lived fading powder bursts and the chamber stays recognizable.
- Ring length at or below the upstream 50, as long as overwrite destroys the previous slot and particle count does not climb without bound across repeated contacts.
- Whether contact capture is a `CollisionDecisionHook` during the step or a drain of begin transitions after the step, as long as spawn and destroy happen after the world unlocks and persistent contacts do not keep sparking.
- Whether color fade uses an existing color mutator or a narrow batch write, as long as fade is visible in the copied frame and does not loop per particle across the JS boundary.
- Static SVG preview artwork and portrait rectangles, as long as previews are captioned `Static preview` and phone frames include the walls.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.
- Exact preset labels, as long as water versus elastic stays obvious.
- A third Drawing material such as powder, if it reuses existing flags and does not become the full keyboard matrix.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` — Phase 29 goal, PLAY-01, FX-01, FX-02, and the four success criteria.
- `.planning/REQUIREMENTS.md` — PLAY-01, FX-01, FX-02. Out of scope: DRAW-02, PRESET-01, PARITY-01, cutting particle counts, raising the 4-step cap, sealed parity.
- `.planning/PROJECT.md` — v1.3 recognizable ports in the existing playground; missing engine behavior only where a listed scene cannot run without it.
- `.planning/STATE.md` — Phase 28 is complete. This phase is the next pending discuss target.
- `PROJECT-SCOPE.md` — Hobby scope, honest limitations, experimental API, no package publication from this phase.
- `standards-overrides.md` — Kobalte Dialog for the playground shell; `skipLibCheck` for Kobalte declarations; hobby scope and independent AI review.
- `AGENTS.md` — README SVG plan coverage and portrait frame rules for every new catalog scene.

### Scene recognition research
- `.planning/research/FEATURES.md` — Drawing Particles and Sparky recognition cards, must-see behavior, and chrome (full keyboard matrix, exact VFX pool).
- `.planning/research/PITFALLS.md` — Sparky contact callbacks, VFX lifetime, Drawing `lastGroup`, particle-count, and 4-step cap pitfalls.
- `.planning/research/ARCHITECTURE.md` — Session hook versus post-step contact observation; SolidJS does not own physics.
- `.planning/research/STACK.md` — Pinned JS and C++ test paths, frame caps, and the post-step VFX rule.

### Pinned upstream tests
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testSparky.js` — JS Sparky setup at pin `7f20402173fd143a3988c921bc384459c6a858f2`.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/Sparky.h` — C++ cross-check for Sparky.
- `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/testDrawingParticles.js` — JS Drawing Particles setup at the same pin.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/DrawingParticles.h` — C++ cross-check for Drawing Particles.

### Existing player decisions and code
- `.planning/phases/28-interaction-seams/28-CONTEXT.md` — Append-only catalog, watch-first default, destroy-in-shape, pinned-test credits, 4-step cap, Chromium smoke.
- `.planning/phases/27-material-flag-groups/27-CONTEXT.md` — Elastic and rigid flag groups already shipping.
- `.planning/phases/26-catalog-shell-and-basin-scenes/26-CONTEXT.md` — Shell and proof pattern for v1.3 scenes.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — Catalog metadata, static previews, native-only motion, credit chrome.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — `Static preview` caption and Reset honesty.
- `web/src/catalog/scenes.ts` — `SCENE_IDS` and scene records to extend. Current last id is `liquid-tumbler`.
- `web/src/catalog/portrait-bounds.ts` — Portrait frames required for every `SceneId`.
- `web/scripts/readme-svg/plans.ts` — README SVG plans required for every catalog scene id.
- `crates/liquidfun-wasm/src/session.rs` — Steps with `NoDecisionHook` today.
- `crates/liquidfun/src/world/particle_object/particle.rs` — `destroy_particles_in_shape`.
- `standards/languages/rust.md` — Safe Rust, error handling, and module shape.
- `standards/languages/typescript-javascript.md` — SolidJS catalog and player code.
- `standards/core/frontend-ui.md` — Dark playground chrome and source disclosure already in the player.
- `standards/core/architecture.md` — Physics stays in `liquidfun`; the WASM crate is the scene shell.
- `standards/core/testing.md` — Focused tests for contact sparks, paint, and catalog records.
- `standards/core/verification.md` — Local checks before commit; `just web-player-smoke` is the browser proof.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/catalog/scenes.ts`: scene ids, titles, descriptions, controls, and credits. Append two records after `liquid-tumbler`. Runtime presets already exist for Impulse and Theo Jansen.
- `web/src/catalog/previews.tsx`: static SVG previews. Add two captioned previews.
- `web/src/catalog/portrait-bounds.ts` and `web/scripts/readme-svg/plans.ts`: required companions for a new catalog id.
- `crates/liquidfun-wasm/src/scene/`: one module per demo, built through the checked scene-id factory. Soup already carves with `destroy_particles_in_shape`.
- `crates/liquidfun-wasm/src/session.rs`: one world step path. It always passes `NoDecisionHook`.
- `ContactTransitionKind::Begin` and `CollisionDecisionHook::observe` already exist in `liquidfun`. Color Mixer already shows that copied frames can carry particle colors.

### Established Patterns
- Hash navigation `#/scene/{id}` with unknown-hash fallback.
- One opaque WASM session, copied frame lanes, dispose on reset and scene switch.
- Watch-first scenes expose play, pause, and Reset only. Interactive scenes add labeled presets that do not recreate the world unless the hint says they do.
- Static catalog previews captioned `Static preview`. The shell uses the shadcn-solid override recorded in `standards-overrides.md`.
- Playground copy stays experimental and does not claim sealed parity.

### Integration Points
- `SCENE_IDS` and the scene factory must accept `drawing-particles` and `sparky` together so the catalog cannot advertise a scene the session cannot build.
- Sparky is the first playground scene that needs begin-contact observation. Keep that observer inside the Sparky session path.
- `just web-player-smoke` is the Chromium proof. Extend it for the two new scenes, one Drawing drag, and the existing catalog regression.
- Engine changes belong in `liquidfun` only if post-step contact observation or batch color fade cannot be expressed with the current public API.

</code_context>

<specifics>
## Specific Ideas

- Sparky should read as bouncing circles in a tall room that erupt into short colorful powder bursts, then those bursts fade and disappear.
- A resting pile must not keep spraying. Only a new contact throws sparks.
- Drawing should open as an empty box. A drag leaves a stroke. Elastic paint should clump instead of sloshing like the water stroke.
- The twelve-scene claim is the catalog contents plus the original six, not a new shell. `liquid-tumbler` stays where it is.

</specifics>

<deferred>
## Deferred Ideas

- Full Drawing Particles keyboard matrix, including barrier, repulsive, zombie, wall, spring, viscous, color mixing, and move mode (DRAW-02).
- Impulse and Liquid Timer particle-type presets (PRESET-01).
- Sealed per-scene differential evidence (PARITY-01) and Box2D-only tests (BOX2D-01).
- Optional Sparky pointer poke. Watch-first collisions are enough.
- Exact upstream `maxVFX = 50` when a smaller bounded ring still destroys expired bursts.
- Phases 30–33 fountain, wave tank, bubbler, and stacked drip.
- Firefox, Safari, Linux qualification, and a live Pages redeploy.

</deferred>

---

*Phase: 29-sparky-drawing-and-full-catalog*
*Context gathered: 2026-09-27*

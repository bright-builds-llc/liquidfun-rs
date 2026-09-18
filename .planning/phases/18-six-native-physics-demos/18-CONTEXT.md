---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T03:37:51.969Z
---

# Phase 18: Six Native Physics Demos - Context

**Gathered:** 2026-09-18
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can choose six distinct, persistent physics scenes and experiment with each scene's real Rust behavior through a small, understandable control surface. This phase owns the six-card catalog with names, descriptions, and static previews; per-scene labeled bounded controls with explicit reset-on-change behavior; native Rust scene authorship for Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel; and per-scene implementation, inspiration, and notice links.

Phase 17 already hosts Dam Break in the shared SolidJS player with hash URLs, play/pause/reset, one-session teardown, hidden-tab bounding, site-level source/provenance chrome, and Pages delivery. Phase 19 owns pointer/touch polish, stuck-pointer handling, narrow-width accessibility acceptance, and the complete six-scene browser smoke suite.

</domain>

<decisions>
## Implementation Decisions

### Catalog cards and previews

- **D-01:** Replace the Phase 17 honest name list with a six-card catalog. Every card shows the approved title, a short one-behavior description, a static preview, and one Open action that loads `#/scene/{id}` in the shared player. All six ids remain `dam-break`, `fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, and `water-wheel`.
- **D-02:** Previews are static inline SVG or CSS illustrations authored in the frontend. Do not start a WASM world per card, capture live Canvas thumbnails, or animate fake physics in the catalog. Catalog metadata lives in `web/src/catalog/` and must not own live sessions.
- **D-03:** Mark every approved scene ready once it has a working native implementation. Keep unknown/empty hash fallbacks from Phase 17. Do not keep "not ready yet" chips on shipped scenes, and do not invent extra catalog search, filters, or a scene editor.

### Per-scene control surface

- **D-04:** Each scene exposes two or three labeled, bounded controls in the shared player, plus the existing play/pause/reset chrome. Prefer discrete presets and constrained numeric ranges over unbounded sliders. Changing a construction preset must recreate the world; the UI must say so before the visitor applies it.
- **D-05:** Locked first-pass controls, matching the approved feature table:
  - Dam Break: water-amount preset, gravity preset, obstacle drop/reset.
  - Fountain: emission-rate preset, launch-speed preset, aim-angle preset.
  - Float or Sink: body/density preset, drop-body action.
  - Color Mixer: mix-strength preset, stir-speed preset.
  - Jelly Drop: shape preset, softness preset applied on reset, drop/poke action.
  - Water Wheel: jet-strength preset, emission on/off.
- **D-06:** Runtime-only controls may update the live session without reset when the engine can apply them safely (for example jet strength or emission toggle). If a control cannot be applied live, it resets and the copy says so. Pointer aiming, stirring, poking, and stuck-pointer polish remain Phase 19 unless a control above already covers the documented interaction as a labeled button or preset.

### Scene authorship and physics honesty

- **D-07:** Author all six persistent scenes in `liquidfun-wasm` using the public `liquidfun` API. Diagnostic protocol recipes and the desktop testbed are capability references only; do not import them into the browser package or replay two-particle inspect-and-destroy schedules as gallery demos.
- **D-08:** Keep one opaque session that exclusively owns one world. Expand construction from Dam Break-only `ProofSession::new()` to a checked scene-id factory. Preserve copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch.
- **D-09:** All particle motion, collisions, buoyancy response, color mixing, elastic deformation, and wheel rotation come from this Rust engine. Ban fake buoyancy animation, rendering-only color blending advertised as mixing, and scripted wheel rotation. Emitters, drops, and stir inputs are scene controllers that call engine creation/forces, not JavaScript position animation.
- **D-10:** Prove Float or Sink, Jelly Drop, and Water Wheel early in the phase because they are the least certain compositions. Evolve the existing Dam Break basin into the named demo rather than replacing it with canned motion. Fountain and Color Mixer follow the same shared session/control contract.
- **D-11:** A failed visual experiment requires investigation or an explicit recorded scope decision. Do not silently substitute a different scene, keep a broken name with fake physics, or expand the public engine API solely to preserve a catchy title. Water Wheel may fall back to a simpler pinwheel-and-dropped-balls toy only through that explicit revision.

### Source, credits, and notices

- **D-12:** Each player view includes the current scene's implementation link (this repository's Rust scene module), inspiration link(s), and any applicable notice. Keep the Phase 17 footer for repository, MIT "free and open source" copy, Peter Ryszkiewicz/OpenLinks, and version/commit/build provenance.
- **D-13:** Implementation links point at this project's scene source, not upstream C++ as if it were the running code. Inspiration may cite the LiquidFun showcase, Faucet, particle guide, or other FEATURES.md references. An inspiration link does not replace a source notice. If a scene adapts upstream material, record the exact pinned revision/file and preserve notices via `THIRD_PARTY_NOTICES.md`.
- **D-14:** Copy must identify an experimental native Rust implementation. Do not claim complete LiquidFun parity, physically accurate pigment chemistry, or that catalog previews are live simulations.

### Design system and verification

- **D-15:** Keep semantic HTML plus the existing scoped dark CSS tokens. Do not adopt MysticUI, Tailwind, shadcn, or an icon package in this phase. Six static cards do not justify a design-system migration; the Phase 17 D-09 exception remains in force until the 2026-12-17 review date unless a later phase records a new override.
- **D-16:** Prove each scene locally: it opens from the catalog and hash URL, steps visible native state, reset restores the documented initial state, construction-control changes reset with labeled copy, credits/notices are reachable, and switching scenes disposes the prior world. Do not require the Phase 19 six-scene pointer/accessibility matrix or a new Linux qualification campaign.
- **D-17:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion

- Exact particle counts, basin/bowl/wheel dimensions, colors, camera bounds, and control enumerations within the locked names, bounded-resource, and honesty rules.
- Exact card layout, SVG preview artwork, credit-panel markup, and control labels within the locked dark semantic-HTML contract.
- Exact WASM session factory naming and module split, provided one world remains private and frames stay copied owned arrays.
- Whether Dam Break's existing dynamic circle remains the obstacle or is replaced by a documented equivalent rigid body, as long as reset restores it and the visitor can drop/reset it through a labeled control.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 playground goal, six-scene direction, and milestone boundaries.
- `.planning/REQUIREMENTS.md` — `WEB-01`, `WEB-04`, `WEB-08`, `DEMO-01` through `DEMO-06` owned by this phase; `WEB-05`, `WEB-07`, and `WEBTEST-01` belong to Phase 19.
- `.planning/ROADMAP.md` § Phase 18 — goal, success criteria, uncertain-scene spike note, and no-fake-physics rule.
- `PROJECT-SCOPE.md` — hobby completion standard; playground work does not revive mandatory Linux qualification.

### Browser architecture and feature research

- `.planning/research/v1.1/FEATURES.md` — approved six-scene hooks, suggested controls, inspiration sources, physics-versus-presentation boundary, and anti-features.
- `.planning/research/v1.1/ARCHITECTURE.md` — Rust scene authorship in `liquidfun-wasm`, catalog metadata without live preview worlds, session/control/frame contract.
- `.planning/research/v1.1/STACK.md` — SolidJS/Vite/Bun, copied frames, Pages delivery already established in Phase 17.
- `.planning/research/v1.1/PITFALLS.md` — leaked worlds, unbounded emission, fake physics, wrong claims.
- `.planning/research/v1.1/SUMMARY.md` — deferred pointer polish and six-scene smoke.

### Phase 16 and 17 contracts to preserve

- `.planning/phases/16-rust-wasm-browser-bridge/16-CONTEXT.md` — private session, copied frames, isolation, Canvas 2D.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-CONTEXT.md` — Dam Break player, hash URLs, one-session teardown, hidden-tab cap, site chrome, Pages workflow, D-09 HTML/CSS exception.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-UI-SPEC.md` — dark tokens, typography, catalog/player structure to evolve into six cards.
- `web/src/catalog/scenes.ts`, `web/src/components/CatalogNav.tsx`, `web/src/components/PlayerPanel.tsx`, `web/src/App.tsx`, `web/src/physics/session.ts`, `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/lib.rs` — current honest catalog, Dam Break-only session, and proof-scene constants.

### Engine capabilities and notices

- `crates/liquidfun/src/lib.rs` — public engine curation boundary for scene construction.
- `crates/liquidfun/src/world/particle_object/system.rs` — particle system construction, lifetime/capacity, views.
- `crates/liquidfun/src/world/particle_object/group.rs` — checked group recipes for elastic scenes.
- `crates/liquidfun/src/particle/solver/material.rs` — contact-driven `color_mixing`.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/` — capability references only; not browser runtime.
- `THIRD_PARTY_NOTICES.md` — LiquidFun/Box2D notices and altered-source duties.
- `LICENSE` — MIT; truthful free-and-open-source copy remains allowed.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — hobby scope, standing iteration authorization, independent AI review, Phase 17 HTML/CSS exception.
- `standards/core/architecture.md` — functional-core/imperative-shell around catalog, routing, and session decisions.
- `standards/core/frontend-ui.md` — dark default, public source identity, version/commit/build provenance.
- `standards/core/code-shape.md` — shallow control flow, `maybe_` naming, module sizing.
- `standards/core/testing.md` and `standards/core/verification.md` — focused tests and repo-native verification.
- `standards/languages/rust.md` and `standards/languages/typescript-javascript.md` — Rust module layout and SolidJS/Bun defaults (component-library exception locked above).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/catalog/scenes.ts` already has the six stable ids and titles; extend it with descriptions, preview identifiers, credits, ready flags, and control metadata.
- `web/src/components/CatalogNav.tsx` is a thin honest list; replace it with cards while keeping hash-only navigation.
- `web/src/components/PlayerPanel.tsx` hard-codes Dam Break copy; generalize title, status, canvas name, and a per-scene control slot.
- `web/src/physics/session.ts` and `crates/liquidfun-wasm` already own one copied-frame session; add a scene-id constructor and bounded control application.
- `crates/liquidfun-wasm/src/scene.rs` documents Dam Break reset constants that Phase 18 should evolve, not discard.
- `web/src/components/SiteFooter.tsx` already satisfies site-level WEB-08 identity; add per-scene credits beside the player, not in place of the footer.

### Established Patterns

- `liquidfun` remains the sole default workspace member; browser scenes stay in unpublished `liquidfun-wasm`.
- Copied typed arrays cross the JS boundary; raw WASM memory is not exposed.
- Dark semantic HTML and one scoped stylesheet; no component library.
- Hash routes are `#/scene/{id}`; empty and unknown hashes stay distinct fallbacks.
- Construction failures poison the session; recreate rather than claiming catch-unwind recovery.

### Integration Points

- Expand `SCENES` records and `CatalogNav` into six-card browse UI with static previews.
- Generalize `PlayerPanel` and `App.tsx` so any ready scene id can load, reset, and dispose.
- Add Rust scene modules plus a checked factory on the WASM session.
- Surface per-scene implementation/inspiration/notice links in player chrome.
- Keep Pages workflow and `/liquidfun-rs/` production base unchanged unless a scene asset path requires it.

</code_context>

<specifics>
## Specific Ideas

- Research already recommends title, short description, static preview, and one open action per card; live thumbnails would multiply WASM sessions.
- "A few scene-specific controls" means two or three labeled, constrained values, with copy when a change resets the scene.
- Borrow LiquidFun showcase's one-behavior-per-example presentation and Faucet's bounded-emission idea; do not copy unreviewed C++ or legacy JavaScript.
- Color Mixer must show engine contact-driven channel changes, labeled as particle-color mixing rather than pigment chemistry.
- Architecture places scene definitions in the Rust wrapper and catalog metadata (descriptions, thumbnails, credits) in `web/src/catalog/`.

</specifics>

<deferred>
## Deferred Ideas

- Pointer/touch aiming, stirring, poking, stuck-pointer handling, and ordinary scrolling-outside-player acceptance — Phase 19 (`WEB-05`).
- Dark-default playful catalog accessibility, keyboard/focus/contrast acceptance at narrow widths, and concise text interaction instructions as a milestone gate — Phase 19 (`WEB-07`).
- Focused real-browser six-scene smoke plus production-subpath/live Pages gallery checks — Phase 19 (`WEBTEST-01`).
- MysticUI/Tailwind/shadcn adoption — not justified by six static cards; revisit on 2026-12-17 or in a later UI phase with a new override.
- Seed/control values in share links, scene editor, diagnostic-catalog gallery, WASM workers/threads/zero-copy, WebGPU, npm/crates.io publication — outside v1.1.

</deferred>

---

*Phase: 18-six-native-physics-demos*
*Context gathered: 2026-09-18*

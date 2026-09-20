---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T17:38:10.893Z
---

# Phase 20: Playground catalog previews and Reset honesty - Context

**Gathered:** 2026-09-20
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can browse six in-app demo entries with names, descriptions, and visual previews, then Reset a playing scene to its documented initial physics and matching live-control labels. This phase closes the v1.1 milestone-audit post-gate gaps for WEB-01 (in-app visual catalog previews) and WEB-03 (Reset chrome honesty).

The owner-approved responsive Kobalte / semantic-HTML shell stays. Restoring previews must not revive the Phase 18 card grid that broke narrow widths. README gallery WebPs remain a documentation gallery, not an in-app substitute. Play, pause, and Reset already rebuild the native world; this phase makes preset selects tell the truth after Reset. Phase 21 owns leftover cleanup (`loadProofSession`, dead FallbackPanel branches, oversized scene files). Dam Break headless speed versus C++ is out of this phase.

</domain>

<decisions>
## Implementation Decisions

### Catalog preview medium

- **D-01:** Restore in-app visual previews for Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel. Each DemoNavigation entry keeps its existing title, short description, and hash Open link, and adds a compact static illustration.
- **D-02:** Previews are static inline SVG or CSS illustrations authored in the frontend, as in Phase 18 D-02. Do not start a WASM world per catalog entry, capture live Canvas thumbnails, animate fake physics, or load README `docs/assets/demos/*.webp` as the in-app preview.
- **D-03:** Caption each illustration `Static preview`. Copy must not imply the catalog thumbnail is a live simulation.

### Catalog layout inside the Kobalte shell

- **D-04:** Keep `PlaygroundShell`: sticky header, 280px desktop sidebar, semantic hash links, and Kobalte Dialog drawer on mobile. Do not restore `.catalog-card`, the three-column card grid, CatalogNav cards, or a separate catalog page.
- **D-05:** Place the compact preview inside each existing `DemoNavigation` list item (sidebar and mobile drawer share the same component). Preserve 44px minimum hit targets, `#39D3C7` 2px `:focus-visible` with 4px offset, dark tokens, and catalog scroll outside the canvas.
- **D-06:** Keep `@kobalte/core` 0.13.12 for the drawer only. Do not adopt MysticUI, Tailwind, shadcn, or an icon package. `web/e2e/shell.spec.ts` must continue asserting `.catalog-card` count is 0.

### Reset live-control honesty

- **D-07:** Play, pause, and Reset still rebuild or freeze the native world as today. After Reset (and any `recreateScene` path that clears `constructionValues`), every preset `<select>` shows the documented initial value from `DEFAULT_PRESET_VALUES`, not a stale `pendingValue`.
- **D-08:** Honesty covers live runtime presets (`recreates: false`) and construction presets (`recreates: true`). Representative live labels: Fountain emission-rate / launch-speed / aim-angle, Float or Sink body, Color Mixer stir-speed, Water Wheel jet-strength / emission. Construction selects such as Dam Break water-amount and Color Mixer mix-strength also return to documented initials after Reset.
- **D-09:** Do not change WEB-04 control sets, Apply-setting copy, or native reset physics. This phase only makes chrome match the rebuilt world.

### Focused Chromium smoke

- **D-10:** Extend the existing Chromium `just web-player-smoke` suite. Cover visible static previews for all six catalog entries in the desktop sidebar, one mobile-drawer preview check, and Reset label honesty for representative live presets (Fountain emission-rate, Float or Sink body, Color Mixer stir-speed, Water Wheel jet-strength).
- **D-11:** Do not add Firefox/Safari/WebKit, a live-preset combinatorial matrix, screenshot-hash oracles, Linux qualification, or Dam Break C++ timing. Independent AI review remains eligible; the implementing agent must not approve its own work.

### Claude's Discretion

- Exact SVG/CSS artwork, preview aspect ratio, and list-item packing within the locked 280px sidebar, 44px hit target, and dark-token contract.
- Whether Reset honesty remounts `SceneControls` with a key, resets `pendingValue` from `maybeValues`/`DEFAULT_PRESET_VALUES`, or derives select value from a reactive construction bag — provided labels match the rebuilt world after Reset.
- Exact Playwright file split and helper extraction, provided D-10 is covered without a new browser matrix.

### Folded Todos

None — `todo match-phase 20` returned no pending todos.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 playground goal and milestone boundaries.
- `.planning/REQUIREMENTS.md` — `WEB-01` and `WEB-03` reopened for this phase.
- `.planning/ROADMAP.md` § Phase 20 — goal, success criteria, README-WebP note, and Kobalte-shell constraint.
- `.planning/v1.1-MILESTONE-AUDIT.md` — post-gate catalog-preview and Reset→SceneControls gaps this phase closes.
- `PROJECT-SCOPE.md` — hobby completion; this phase does not revive Linux qualification or package publication.

### Prior locked playground contracts

- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — D-01/D-02/D-14 static SVG/CSS previews; D-04/D-05/D-06 control surface and live-vs-construction split; D-15 semantic HTML.
- `.planning/phases/18-six-native-physics-demos/18-UI-SPEC.md` — token-only preview art and Static preview honesty; do not restore the card grid.
- `.planning/phases/19-interaction-polish-and-browser-verification/19-CONTEXT.md` — D-08 keyboard-native controls; D-09/D-10 dark tokens, 44px, focus; D-12/D-13 focused Chromium smoke.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-CONTEXT.md` — D-08 play/pause/reset; D-09 semantic HTML exception; D-11 one-session ownership.
- `docs/superpowers/plans/2026-09-19-responsive-playground-shell.md` — owner-approved Kobalte shell that replaced `.catalog-card`; keep that shell while restoring previews.
- `standards-overrides.md` — Kobalte Dialog / semantic HTML exception; `skipLibCheck` for Kobalte 0.13.12.

### Live code and tests

- `web/src/components/DemoNavigation.tsx` — current title + description hash links; integration point for compact previews.
- `web/src/components/PlaygroundShell.tsx` — sidebar + Kobalte drawer composition.
- `web/src/components/SceneControls.tsx` and `web/src/components/scene-controls.ts` — `pendingValue` created once; `DEFAULT_PRESET_VALUES` and `initialPresetValue`.
- `web/src/App.tsx` — `recreateScene` / `constructionValues` reset; `SceneControls` `maybeValues`.
- `web/src/catalog/scenes.ts` — six ready ids, descriptions, live vs construction presets.
- `web/src/app.css` — `.demo-sidebar` / `.demo-nav-*` layout; do not revive `.catalog-card`.
- `web/e2e/shell.spec.ts` — asserts `.catalog-card` count is 0; desktop sidebar and mobile drawer.
- `web/e2e/player.spec.ts` — existing play/pause/reset/scene-switch smoke to extend.
- `README.md` and `docs/assets/demos/*.webp` — documentation gallery only; not in-app previews.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — hobby scope, standing iteration authorization, independent AI review.
- `standards/core/architecture.md` — functional-core/imperative-shell around catalog, routing, and control-value decisions.
- `standards/core/frontend-ui.md` — dark default, public source identity, version/commit/build provenance.
- `standards/core/code-shape.md` — shallow control flow, `maybe_` naming, module sizing.
- `standards/core/testing.md` and `standards/core/verification.md` — focused tests and repo-native verification.
- `standards/languages/typescript-javascript.md` — SolidJS/Bun defaults; Kobalte exception is recorded in `standards-overrides.md`.
- `LICENSE` — MIT; truthful free-and-open-source copy is allowed.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `DemoNavigation` already lists six titles, descriptions, and `#/scene/{id}` links in catalog order. Sidebar and mobile drawer both render it.
- `scene-controls.ts` already has `DEFAULT_PRESET_VALUES` and `initialPresetValue`; Reset honesty can reuse those documented initials.
- `PlaygroundShell` already provides the sticky 280px sidebar and Kobalte drawer; previews belong inside that list, not a new layout.
- Token-only Phase 18 `ScenePreview` SVG was deleted with `.catalog-card`; restore the illustration idea, not the card CSS.
- `just web-player-smoke` and `web/e2e/{shell,player}.spec.ts` are the existing Chromium evidence path.

### Established Patterns

- Catalog metadata lives in `web/src/catalog/` and must not own live WASM sessions.
- Construction presets show Apply setting and recreate the world; live presets apply immediately.
- `PresetControl` stores `pendingValue` in a Solid signal initialized once, which is why Reset leaves stale labels.
- Semantic HTML plus one scoped CSS file; Kobalte Dialog is drawer-only.
- `web/e2e/shell.spec.ts` treats `.catalog-card` as forbidden leftover.

### Integration Points

- Add preview markup in `DemoNavigation` / a restored preview helper, then style compact list items in `app.css` without `.catalog-card`.
- Key or resync `SceneControls` when `App.tsx` `recreateScene` / `startScene` clears `constructionValues`.
- Extend Playwright against the existing production-base preview; keep the `.catalog-card` count-0 assertion.

</code_context>

<specifics>
## Specific Ideas

- ROADMAP: keep README gallery WebPs as a documentation gallery, not a substitute for in-app previews.
- Audit named the lying live labels: Fountain rate, Color Mixer mixable (live control is `stir-speed`), Water Wheel speed, Float or Sink density (`body`).
- Owner-approved responsive shell plan explicitly removed obsolete catalog preview/card code because the card grid broke narrow widths.

</specifics>

<deferred>
## Deferred Ideas

- Unused `loadProofSession` and opt-in `rust-wasm-proof.spec.ts` — Phase 21.
- Dead `FallbackPanel` empty/not-ready branches — Phase 21.
- Bright Builds `file-lengths` on `color_mixer.rs`, `dam_break.rs`, and `water_wheel.rs` — Phase 21.
- Playground Dam Break headless speed versus pinned C++ — out of v1.1 definition of done.
- MysticUI/Tailwind/design-system migration — revisit 2026-12-17 per standards-overrides.md.

</deferred>

---

*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Context gathered: 2026-09-20*

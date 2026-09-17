---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T11:25:12.047Z
---

# Phase 17: Shared Player and Early Pages Delivery - Context

**Gathered:** 2026-09-17
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can open, control, and reload one working first scene on the real GitHub Pages project site, and every push to `main` builds that site and WASM package from the same checkout, then deploys the assembled artifact after required checks succeed. This phase owns the shared SolidJS player lifecycle (play/pause/reset, loading, failure/retry, one-session teardown, hidden-tab bounding), stable scene URLs with a useful unknown-id fallback, honest incomplete-catalog labeling, and site-level source/build chrome.

Phase 16 already proved a private WASM session, copied frames, and Canvas 2D rendering. Phase 18 authors the remaining five physics demos plus per-scene controls, previews, and credits. Phase 19 owns pointer/touch polish, accessibility/responsive acceptance, and the complete six-scene browser smoke suite.

</domain>

<decisions>
## Implementation Decisions

### First hosted scene and honest catalog

- **D-01:** Host Dam Break as the first working scene. Evolve the Phase 16 basin proof into that named scene so a visitor can play real Rust particle/rigid motion, not a compile-only or canned animation.
- **D-02:** Show the six approved names in a thin catalog/navigation list. Only Dam Break is playable. Label Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel as not ready yet. Do not invent fake physics or silently substitute scenes.
- **D-03:** Do not implement full demo cards, previews, per-scene bounded controls, or per-scene inspiration notices. Those belong to Phase 18 (`WEB-01`, `WEB-04`, `WEB-08`, `DEMO-01` through `DEMO-06`).

### Stable URLs and unknown-scene fallback

- **D-04:** Use hash-based scene navigation so GitHub Pages needs no server rewrite. Canonical first-scene form: `#/scene/dam-break`. Keep scene identifiers stable and lowercase with hyphens.
- **D-05:** Production Vite `base` is `/liquidfun-rs/`. Local development may keep a root base, but production and preview-of-production builds must load JS/WASM under the project subpath. Expected hosted origin is `https://bright-builds-llc.github.io/liquidfun-rs/` until repository Pages settings prove a different URL; record the actual deployed URL and source revision as evidence.
- **D-06:** Reloading a valid scene URL restores that scene. An unknown or empty scene identifier shows a useful catalog/fallback with an explanation and a working path back to Dam Break. Do not leave a blank canvas or a hard 404-looking empty app.

### Shared player chrome

- **D-07:** The shared SolidJS player is the only runtime surface. It presents the current scene title, Canvas 2D viewport, play/pause, reset to the documented initial state, visible loading, and an actionable failure state with retry/reset. Replace Phase 16's "Dispose session" proof control with these product controls.
- **D-08:** Play starts or resumes stepping. Pause freezes the world without disposing it. Reset disposes the current session and recreates Dam Break from its documented initial state. Failure stops the session; Retry recreates it.
- **D-09:** Keep semantic HTML and scoped CSS evolved from the Phase 16 dark proof. Do not adopt MysticUI/Tailwind in this phase; record that as a deliberate thin-slice exception so Pages delivery is not blocked by a new design-system adoption. Revisit gallery-card library choice in Phase 18 if the catalog surface needs it.
- **D-10:** Establish site-level source/build chrome now: GitHub repository link, truthful "free and open source" copy (MIT), version, short commit, and build provenance in normal product chrome; missing provenance fields show `Unavailable`. Mention Peter Ryszkiewicz with an OpenLinks link. Per-scene implementation/inspiration/notice links wait for Phase 18.

### Session teardown, hidden tabs, and bounded stepping

- **D-11:** Own exactly one WASM world and one animation loop. Resetting, changing selection, leaving the player, and Solid cleanup must cancel the pending frame, disconnect observers/listeners, discard stale async loads with a generation token, and `dispose()` the prior session. Only the current session may step.
- **D-12:** Pause stepping while `document.hidden` is true. On resume, clear accumulated catch-up time so a hidden tab does not dump seconds of simulation. Cap accepted wall-clock delta and allow at most four simulation steps per animation callback; do not unbounded-accumulate emission or steps.
- **D-13:** Preserve Phase 16 ownership rules: copied typed arrays, no raw pointers, no per-particle JS/Rust calls, recreate after a trap rather than claiming native catch-unwind recovery. Do not add WASM workers, threads, or zero-copy views.

### GitHub Pages delivery

- **D-14:** Every push to `main` triggers the WASM package and SolidJS production-site build from that same checkout, with no path filters. PRs may run the same build/smoke without deploy permissions. Do not rebuild WASM in a separate deploy job from different inputs.
- **D-15:** Deploy only after required build checks succeed. Use Actions as the Pages source, `pages: write` plus `id-token: write` on the deploy job, and the `github-pages` environment. Do not add a personal access token. Pin Actions by full commit SHA.
- **D-16:** Concurrent or overlapping `main` pushes must not leave an older completed revision as the final site. Superseded queued revisions may coalesce. Record the deployed URL and source revision. A website delivery runner is not Linux native qualification and must not revive oracle/sanitizer/coverage/performance gates.

### Verification boundary

- **D-17:** Prove the hosted/player slice with: production-base asset loading, Dam Break play/pause/reset, unknown-hash fallback, session disposal on reset/leave, hidden-tab no-catch-up, and a recorded Pages URL/revision after a real `main` deployment. Do not require the Phase 19 six-scene pointer/accessibility smoke matrix.
- **D-18:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion

- Exact hash parser, catalog markup, button labels, loading copy, and failure wording within the locked URL, honesty, and chrome rules.
- Exact Dam Break particle counts, basin dimensions, and colors as long as reset returns to a documented initial state and motion remains visibly native.
- Exact workflow job names, concurrency group, and SHA-pinned official Pages actions, provided latest-main protection and no-PAT permissions remain.
- Whether local preview of `/liquidfun-rs/` uses Vite preview, a tiny static server, or the existing `just web-*` recipes, as long as production-base loading is actually tested.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 playground goal, six-scene direction, and milestone boundaries.
- `.planning/REQUIREMENTS.md` — `WASM-04`, `WEB-02`, `WEB-03`, `WEB-06`, `HOST-01`, `HOST-02`, `HOST-03` owned by this phase; later-phase requirements to leave alone.
- `.planning/ROADMAP.md` § Phase 17 — goal, success criteria, honest-catalog note, and website-delivery-vs-Linux-qualification note.
- `PROJECT-SCOPE.md` — hobby completion standard; Pages work does not revive mandatory Linux qualification.

### Browser architecture and risk research

- `.planning/research/v1.1/ARCHITECTURE.md` — shared player ownership, hash/query navigation, `/liquidfun-rs/` base, hidden-tab catch-up cap, generation-token loads.
- `.planning/research/v1.1/STACK.md` — SolidJS/Vite/Bun, Pages workflow permissions, no-PAT deploy, concurrency, macOS-vs-delivery-runner distinction.
- `.planning/research/v1.1/FEATURES.md` — Dam Break as the first hosted vertical slice; persistent scene and reset expectations.
- `.planning/research/v1.1/PITFALLS.md` — leaked worlds, duplicate loops, catch-up storms, wrong Vite base, Pages permission/order failures.
- `.planning/research/v1.1/SUMMARY.md` — Phase 17 deliverable summary and deferred six-scene work.

### Phase 16 contracts to preserve

- `.planning/phases/16-rust-wasm-browser-bridge/16-CONTEXT.md` — private session, copied frames, isolation, Canvas 2D, deferred player/Pages.
- `.planning/phases/16-rust-wasm-browser-bridge/16-UI-SPEC.md` — dark semantic proof tokens to evolve, not throw away.
- `web/src/App.tsx`, `web/src/physics/session.ts`, `web/src/physics/loader.ts`, `web/src/render/canvas.ts`, `web/vite.config.ts` — current proof shell, one-session owner, root Vite base to replace for production.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — hobby scope, standing iteration authorization, independent AI review.
- `standards/core/architecture.md` — functional-core/imperative-shell around clock, routing, and session decisions.
- `standards/core/frontend-ui.md` — dark default, public source identity, version/commit/build provenance in product chrome.
- `standards/core/code-shape.md` — shallow control flow, `maybe_` naming, module sizing.
- `standards/core/testing.md` and `standards/core/verification.md` — focused tests and repo-native verification.
- `standards/languages/typescript-javascript.md` — SolidJS/Bun defaults and component-library preference (phase exception locked above).
- `LICENSE` — MIT; truthful free-and-open-source copy is allowed.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/physics/session.ts` already owns one generated WASM session with `nextFrame()` and idempotent `dispose()`, poisoning after advance/capture/parse failure.
- `web/src/physics/loader.ts` plus `just web-wasm` / `scripts/web-build.ts` regenerate ignored wasm-pack output before frontend work.
- `web/src/render/canvas.ts` and `web/src/render/camera.ts` already fit world bounds, invert y, and redraw on resize without stepping physics.
- `web/src/App.tsx` already models loading/running/failure/disposed as a tagged union and cancels rAF plus ResizeObserver on cleanup.
- Phase 16 Chromium Playwright smoke and `scripts/phase16/` closure tooling prove visible Rust motion; extend rather than replace the focused browser check.

### Established Patterns

- `liquidfun` remains the sole default workspace member; browser bindings stay in unpublished `liquidfun-wasm`.
- Copied typed arrays cross the JS boundary; raw WASM memory is not exposed.
- Dark semantic HTML and one scoped stylesheet; no component library yet.
- Native Cargo CI stays on macOS; browser/WASM work is additive delivery, not a new native qualification campaign.

### Integration Points

- Replace the proof-only `App.tsx` shell with a catalog/player layout that still consumes `createSceneSession`.
- Set Vite production `base` to `/liquidfun-rs/` and verify built WASM URLs under that prefix.
- Add hash routing and a Dam Break scene id beside the existing proof scene constructor, without importing the desktop testbed or differential runner.
- Add `.github/workflows/pages.yml` (or equivalent) using the repository's SHA-pinned action style from `.github/workflows/ci.yml`.
- Surface version/commit/build fields in the player chrome; `Unavailable` if a field is missing.

</code_context>

<specifics>
## Specific Ideas

- Research and the roadmap both say to deploy Dam Break early rather than waiting for all six scenes.
- Hash routing is the recommended GitHub Pages simplification; query strings are acceptable only if they still work after a hard reload under `/liquidfun-rs/`.
- Architecture's hidden-tab rule is specific: pause while hidden, clear accumulated time on resume, cap steps per frame (start at four).
- Product chrome should feel like a small playground, not the Phase 16 technical proof title ("LiquidFun Rust/WASM browser proof").

</specifics>

<deferred>
## Deferred Ideas

- Full six-card catalog with descriptions, previews, and per-scene controls — Phase 18.
- Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel physics — Phase 18.
- Per-scene source/inspiration/notice pages — Phase 18 (`WEB-08`).
- Pointer/touch interaction, stuck-pointer handling, narrow-width accessibility acceptance, and the all-six browser smoke — Phase 19.
- MysticUI/Tailwind adoption — revisit if Phase 18 catalog cards need a design system.
- WASM workers, threads, zero-copy views, WebGPU, SSR, accounts, npm/crates.io publication — outside v1.1.

</deferred>

---

*Phase: 17-shared-player-and-early-pages-delivery*
*Context gathered: 2026-09-17*

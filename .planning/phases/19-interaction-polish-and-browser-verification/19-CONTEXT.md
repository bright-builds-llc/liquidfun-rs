---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T01:45:03.493Z
---

# Phase 19: Interaction Polish and Browser Verification - Context

**Gathered:** 2026-09-19
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Visitors can comfortably interact with all six hosted demos using ordinary mouse, touch, and keyboard, with focused evidence that the production experience works. This phase owns WEB-05 pointer/touch interaction after resize and at narrow widths, stuck-pointer handling, and ordinary scrolling outside the player; WEB-07 dark-default catalog/control usability at desktop and mobile widths, labeled keyboard-operable controls, visible focus, contrast, and concise text interaction instructions; and WEBTEST-01 focused real-browser smoke of the built Rust WASM artifact plus targeted production-subpath and live Pages checks that record the tested URL/revision.

Phase 18 already ships the six native scenes, labeled bounded controls, static catalog cards, credits, and a Chromium player smoke that opens, resets, and switches scenes. This phase does not add new scenes, a scene editor, camera pan/zoom, a design-system migration, exhaustive browser/native matrices, or Linux qualification.

</domain>

<decisions>
## Implementation Decisions

### Canvas pointer gestures

- **D-01:** Keep every existing labeled preset and action as the keyboard-accessible control path. Do not remove Phase 18 buttons in favor of canvas-only interaction.
- **D-02:** Add one shared Pointer Events pipeline on the player canvas: convert current CSS canvas bounds through the same camera transform used for drawing, send world coordinates into the WASM session, and never issue per-particle JavaScript/Rust calls or expose raw engine pointers.
- **D-03:** Each scene gets exactly one documented canvas gesture that matches FEATURES.md, using a shared `pointer_action` (or equivalently named) session boundary with down/move/up/cancel plus world x/y:
  - Dam Break: pointer repositions or drops the obstacle.
  - Fountain: pointer aims the stream.
  - Float or Sink: pointer-down drops the selected body preset at the world x of the click.
  - Color Mixer: pointer drag stirs particles through engine forces, not JavaScript position animation.
  - Jelly Drop: pointer-down pokes at the world location.
  - Water Wheel: pointer aims or varies the jet through the existing native emitter/joint coupling.
- **D-04:** Do not add camera pan/zoom, pinch, hover debugging, multi-touch editing, or a scene editor. Pointer input applies scene-controller forces, drops, or aim — it does not animate physics in JavaScript.

### Stuck-pointer and page-scroll isolation

- **D-05:** On canvas `pointerdown`, `setPointerCapture`. Clear gesture state on `pointerup`, `pointercancel`, and `lostpointercapture`. Pause, reset, scene change, player leave, and Solid cleanup must release capture and drop any in-flight drag/stir/poke so no stuck force, selected body, or captured pointer remains.
- **D-06:** Apply `touch-action: none` and `preventDefault` only on the canvas while a gesture is captured. Catalog, footer, credits, and the rest of the page keep native scrolling. Do not make the document a touch trap.
- **D-07:** Resize must not rebuild the world. Reuse the existing last-frame redraw plus camera fit, then convert the next pointer sample through the updated CSS bounds. Verify dragging after resize and at a narrow width. Device pixels stay out of physics.

### Keyboard, focus, and instruction copy

- **D-08:** Do not add custom canvas keyboard handlers or global Space-to-pause shortcuts. Enter and Space continue to use native button/select/link activation. Catalog Open links, playback, presets, Apply setting, and scene actions remain keyboard-operable.
- **D-09:** Keep the Phase 17/18 dark semantic HTML plus one scoped CSS exception (D-09 / standards-overrides.md, revisit 2026-12-17). Do not adopt MysticUI, Tailwind, shadcn, or an icon package for this polish pass.
- **D-10:** Preserve 44px minimum control size, `#39D3C7` 2px `:focus-visible` outline with 4px offset on buttons, selects, and in-page links, and text status that does not rely on color alone. Polish wrapping and stacking at 480px and below; do not add a hamburger, orientation-specific physics, or breakpoint-specific scenes.
- **D-11:** Add concise per-scene interaction instructions next to the canvas (evolve the existing figcaption or an adjacent paragraph). Copy states the pointer gesture and that labeled controls work from the keyboard. Do not steal focus from the player heading except when the visitor activates a catalog or fallback link.

### Focused six-scene browser evidence

- **D-12:** Extend the existing Chromium Playwright suite against the production-base `web/dist` (`/liquidfun-rs/`). Cover all six scenes: select, visible stepping, play/pause/reset, one representative pointer gesture plus a labeled control, repeated scene changes with disposal, and hidden-tab recovery. Add a 375px pass for keyboard focus, readable contrast, wrapping, and page scroll outside the canvas.
- **D-13:** Targeted live Pages checks prove JS/WASM loading and refreshed direct `#/scene/{id}` links for the complete gallery. Record the tested URL and source revision. Do not require Firefox, Safari, WebKit, or a native Linux qualification matrix. Visual screenshot hashes are optional diagnostics, not the primary oracle.
- **D-14:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion

- Exact `pointer_action` naming, force/impulse magnitudes, and whether Dam Break's obstacle is a captured drag or a click-to-reposition, provided capture/cancel/resize rules hold and physics stay native.
- Exact instruction wording, figcaption versus adjacent-paragraph markup, and narrow-layout CSS within the locked tokens and 44px/focus contract.
- Exact Playwright file split and helper extraction, provided one focused Chromium suite covers D-12/D-13 without a new browser matrix.
- Whether Water Wheel pointer input aims the jet, varies strength, or both, as long as the wheel still rotates from native particle-body coupling.

### Folded Todos

None — `todo match-phase 19` returned no pending todos.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 playground goal and milestone boundaries.
- `.planning/REQUIREMENTS.md` — `WEB-05`, `WEB-07`, and `WEBTEST-01` owned by this phase.
- `.planning/ROADMAP.md` § Phase 19 — goal, success criteria, and UI-hint yes.
- `PROJECT-SCOPE.md` — hobby completion standard; playground polish does not revive mandatory Linux qualification.

### Browser architecture and risk research

- `.planning/research/v1.1/FEATURES.md` — one meaningful mouse/touch interaction per scene, pointer capture, canvas coordinates, scroll outside player, concise interaction hint.
- `.planning/research/v1.1/ARCHITECTURE.md` — proposed `pointerAction(kind, x, y)` session boundary, copied frames, one-session ownership.
- `.planning/research/v1.1/PITFALLS.md` — pointer drift after resize/high-DPI, capture/cancel, colorful canvas excluding understanding, preserve page scrolling.
- `.planning/research/v1.1/STACK.md` — SolidJS/Vite/Bun, Pages `/liquidfun-rs/` base already established.
- `.planning/research/v1.1/SUMMARY.md` — Phase 19 delivers responsive/playful presentation, keyboard and mouse/touch usability, focused real-browser checks.

### Phase 16–18 contracts to preserve

- `.planning/phases/16-rust-wasm-browser-bridge/16-CONTEXT.md` — private session, copied frames, Canvas 2D, camera fit/y-invert.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-CONTEXT.md` — hash URLs, one-session teardown, hidden-tab cap, D-09 HTML/CSS exception, Pages workflow.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-UI-SPEC.md` — canvas was non-interactive; pointer polish deferred here.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — labeled controls stay; pointer aiming/stirring/poking deferred to this phase; CSS exception remains.
- `.planning/phases/18-six-native-physics-demos/18-UI-SPEC.md` — 44px targets, focus ring, no custom Enter/Space handlers, no hamburger, Phase 19 owns pointer/a11y acceptance and on-canvas instructions.
- `web/src/App.tsx`, `web/src/components/PlayerPanel.tsx`, `web/src/components/SceneControls.tsx`, `web/src/physics/session.ts`, `web/src/render/camera.ts`, `web/src/app.css`, `web/e2e/player.spec.ts`, `crates/liquidfun-wasm/src/lib.rs` — current player, camera, labeled controls, and smoke to extend.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — hobby scope, standing iteration authorization, independent AI review, Phase 17 HTML/CSS exception.
- `standards/core/architecture.md` — functional-core/imperative-shell around pointer-to-world conversion and session decisions.
- `standards/core/frontend-ui.md` — dark default, public source identity, version/commit/build provenance.
- `standards/core/code-shape.md` — shallow control flow, `maybe_` naming, module sizing.
- `standards/core/testing.md` and `standards/core/verification.md` — unit-test pure conversion/capture logic; repo-native verification.
- `standards/languages/typescript-javascript.md` and `standards/languages/rust.md` — SolidJS/Bun defaults (component-library exception locked) and Rust module layout.
- `LICENSE` — MIT; truthful free-and-open-source copy remains allowed.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/render/camera.ts` already fits world bounds and inverts y for drawing; pointer hit-testing must share that transform.
- `web/src/App.tsx` already owns one session, rAF, ResizeObserver last-frame redraw, hidden-tab pause, and generation-token loads.
- `web/src/physics/session.ts` exposes `applyControl`/`applyAction`; extend with a bounded pointer action rather than a second session.
- `web/src/components/SceneControls.tsx` is the keyboard path; keep it and add canvas listeners beside it.
- `web/src/components/PlayerPanel.tsx` already has a canvas figcaption to evolve into per-scene instructions.
- `web/e2e/player.spec.ts` already loops six scenes for open/reset/switch; extend with pointer, 375px, and Pages refresh checks.
- `crates/liquidfun-wasm` scene modules already implement labeled actions (drop obstacle, poke, drop-body, aim-angle presets, stir-speed, jet). Pointer input should call into those scene controllers with world coordinates.

### Established Patterns

- Copied typed arrays cross the JS boundary; raw WASM memory is not exposed.
- Dark semantic HTML and one scoped stylesheet; no component library.
- Construction presets require Apply setting and recreate the world; runtime controls and actions stay live.
- Hash routes are `#/scene/{id}`; production Vite base is `/liquidfun-rs/`.
- Hidden tabs pause and clear catch-up; at most four steps per animation callback.

### Integration Points

- Add canvas Pointer Events in the player shell, not per catalog card.
- Add a WASM `pointer_action` (kind + world x/y) implemented per scene module.
- Evolve figcaption/instruction copy from catalog metadata so each scene states its gesture.
- Extend Playwright against the existing production-base preview; add a recorded live Pages check after deploy evidence is available.
- Keep Pages workflow and footer provenance unchanged unless a test helper needs a documented URL/SHA capture.

</code_context>

<specifics>
## Specific Ideas

- FEATURES.md already says “at least one meaningful interaction per scene,” “deliberate pointer capture and canvas coordinates,” and “normal page scrolling outside player.”
- PITFALLS.md warns that pointer interaction drifts after resize or on high-DPI screens unless drawing and hit-testing share one CSS-bound transform.
- 18-UI-SPEC already forbids custom Enter/Space handlers and a hamburger; Phase 19 polishes those contracts rather than replacing them.
- Architecture’s `pointerAction(kind, x, y)` is the intended session shape; validate finite coordinates and cancel kinds at the boundary.

</specifics>

<deferred>
## Deferred Ideas

- MysticUI/Tailwind/shadcn adoption — not justified by pointer polish; revisit on 2026-12-17 or in a later UI phase with a new override.
- Camera pan/zoom, pinch, multi-touch editing, hover debug overlays, and a scene editor — outside v1.1.
- Firefox/Safari/WebKit matrices, visual-regression goldens as the primary oracle, WASM workers/threads/zero-copy, WebGPU, npm/crates.io publication — outside this phase.
- Seed/control values in share links and saved full simulation states — outside v1.1.

</deferred>

---

*Phase: 19-interaction-polish-and-browser-verification*
*Context gathered: 2026-09-19*

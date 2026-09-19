# Phase 19: Interaction Polish and Browser Verification - Research

**Researched:** 2026-09-19
**Domain:** Pointer Events canvas input, CSS↔world camera inversion, per-scene WASM gestures, keyboard/focus polish, Chromium Playwright + live Pages evidence
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)

- MysticUI/Tailwind/shadcn adoption — not justified by pointer polish; revisit on 2026-12-17 or in a later UI phase with a new override.
- Camera pan/zoom, pinch, multi-touch editing, hover debug overlays, and a scene editor — outside v1.1.
- Firefox/Safari/WebKit matrices, visual-regression goldens as the primary oracle, WASM workers/threads/zero-copy, WebGPU, npm/crates.io publication — outside this phase.
- Seed/control values in share links and saved full simulation states — outside v1.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WEB-05 | Mouse or touch for each demo's documented interaction without a stuck pointer or preventing ordinary scrolling outside the player. | Shared Pointer Events pipeline + camera unproject + WASM `pointer_action(kind, x, y)`. Capture on down; clear on up/cancel/`lostpointercapture` and on pause/reset/switch/leave. `touch-action: none` only on the canvas. One gesture per scene mapped through existing scene controllers. |
| WEB-07 | Dark-default gallery and controls usable at desktop and narrow/mobile widths, with contrast, keyboard-operable controls, focus indication, and concise text interaction instructions. | Keep D-09 HTML/CSS tokens. Add `select:focus-visible` (currently missing). Add per-scene `interactionHint` next to the canvas. Polish 480px stacking. Do not add canvas keyboard handlers, hamburger, or a design system. |
| WEBTEST-01 | Focused real-browser smoke of the built Rust WASM artifact: all six scenes, visible stepping, playback/reset, representative pointer/control input, repeated cleanup; production-subpath and live Pages targeted checks recording URL/revision. | Extend `web/e2e/player.spec.ts` + `just web-player-smoke` on `/liquidfun-rs/`. Add 375px keyboard/scroll pass. After main deploy, record a Phase 19 HOST-style evidence file for all six hash URLs. Chromium only. Independent AI review of the digest. |
</phase_requirements>

## Summary

Phase 19 is an integration and evidence phase, not a new scene or renderer phase. The six native worlds, labeled controls, hash routes, copied-frame session, ResizeObserver last-frame redraw, and Chromium player smoke already exist. What is missing is a single canvas Pointer Events path that shares `createCamera` / `projectPoint` for hit-testing, a WASM `pointer_action` that each scene controller implements with world coordinates, stuck-gesture cleanup, per-scene instruction copy, a `select` focus ring, and browser proof that the production `/liquidfun-rs/` build plus live Pages gallery actually accept pointer and keyboard input.

The camera already fits the fixed world `[-6, 6] × [-1, 8]` in CSS pixels and inverts y for drawing. There is no unproject helper yet. Scene controllers already implement the labeled actions (`drop-obstacle`, `aim-angle`, `drop-body`, `stir-speed`, `poke-jelly`, `jet-strength`) at fixed or discrete values. Pointer input must call into those controllers with world x/y rather than inventing a second session or animating positions in JavaScript.

**Primary recommendation:** Extract a pure CSS→world converter and a tiny captured-gesture state machine; add `ProofSession.pointer_action(kind, world_x, world_y)` on `SceneHooks`; attach one canvas Pointer Events listener in the player shell; evolve catalog `interactionHint` into the figcaption; extend the existing Chromium `player.spec.ts` plus a post-deploy Pages evidence file. Do not add libraries, browsers, camera gestures, or a design system.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. Planner constraints come from `AGENTS.md` Repo-Local Guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, `PROJECT-SCOPE.md`, and the locked D-01..D-14 decisions above.

Actionable directives that materially constrain this phase:

- Hobby scope: no Linux qualification, no browser matrix, truthful evidence. [VERIFIED: PROJECT-SCOPE.md]
- Standing 2026-09-13 authorization covers ordinary non-force pushes to `main` and iteration through failed checks. Package publication remains unauthorized. [VERIFIED: AGENTS.md]
- Independent AI review is allowed; the implementing agent must not approve its own work. [VERIFIED: AGENTS.md, D-14]
- Phase 17/18 semantic HTML + one scoped CSS exception remains in force until 2026-12-17. [VERIFIED: standards-overrides.md, D-09]
- Functional-core / parse-at-boundary: pointer kinds and finite world coordinates are parsed before the session. [VERIFIED: standards/core/architecture.md]
- Unit-test the pure converter and gesture reducer; Arrange/Act/Assert. [VERIFIED: standards/core/testing.md]
- `maybe_` naming for optional gesture/session values. [VERIFIED: standards/core/code-shape.md]
- `App.tsx` is 593 lines (628-line split trigger). Pointer glue must be extracted, not piled into `App`. [VERIFIED: wc -l web/src/App.tsx]
- Repo-native verification: `just web-player-smoke`, `bun run test:unit`, `cargo test -p liquidfun-wasm`, `bun scripts/bright-builds-check.ts all` for touched surfaces. [VERIFIED: standards/core/verification.md, justfile]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SolidJS | 1.9.15 | Player/catalog shell | Already the playground UI. Pointer listeners attach beside `assignCanvas`. [VERIFIED: web/package.json] |
| Vite | 8.3.0 | Production `/liquidfun-rs/` build + preview `:4173` | Existing production-base smoke host. [VERIFIED: web/package.json, web/playwright.config.ts] |
| TypeScript | 7.0.2 | Typed session/camera/pointer core | Existing exact pin. [VERIFIED: web/package.json] |
| Bun | 1.4.2 | Install, unit tests, preview scripts | Existing packageManager pin; available locally. [VERIFIED: bun --version, web/package.json] |
| `liquidfun-wasm` / wasm-bindgen | in-repo / 0.2.x generated | One opaque session, copied frames | Extend `ProofSession`; do not add a second package. [VERIFIED: crates/liquidfun-wasm/src/lib.rs] |
| Canvas 2D + existing camera | Phase 16/17 | Draw + hit-test | Same CSS transform for both. [VERIFIED: web/src/render/camera.ts, canvas.ts] |
| Pointer Events | Baseline (2020+) | Mouse + touch + pen | Locked by D-02. `setPointerCapture` / `lostpointercapture` are Baseline. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture] |
| `@playwright/test` | 1.63.0 | One Chromium project | Existing pin; `just web-player-smoke` already rebuilds then runs `e2e/player.spec.ts`. [VERIFIED: web/package.json, justfile] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | 5.0.1 | Unit tests for unproject + gesture reducer + session wrapper | Every pure pointer/camera change. [VERIFIED: web/package.json] |
| Rust 1.97.0 + `wasm32-unknown-unknown` | installed | Native-testable scene hooks + wasm-pack rebuild | `cargo test -p liquidfun-wasm` then `just web-wasm` / `just web-build`. [VERIFIED: rustc --version, rustup target list] |
| GitHub Pages origin | `https://bright-builds-llc.github.io/liquidfun-rs/` | D-13 live evidence | Record after this phase's main deploy; Phase 17 HOST-03 file is Dam Break-only and SHA-stale. [VERIFIED: 17-HOST-EVIDENCE.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pointer Events | Separate `mousedown` + `touchstart` | Locked out by D-02. Dual listeners miss pen and double-fire on compatibility mouse. |
| Shared `pointer_action` | Reuse `applyAction("drop-obstacle")` without coordinates | Cannot express aim/stir/poke-at-x. Keep labeled actions; add a coordinate boundary. |
| Chromium-only smoke | Firefox/WebKit projects | Locked out by D-13. |
| MysticUI / Tailwind | Current scoped CSS | Locked out by D-09 until 2026-12-17. |
| Visual screenshot goldens | DOM + step-index + pointer-accepted attributes | D-13: hashes are optional diagnostics only. |

**Installation:** No new packages. Rebuild ignored WASM with the existing recipe, then run the existing player smoke.

```bash
just web-build
just web-player-smoke
cargo test -p liquidfun-wasm
cd web && bun run test:unit && bun run typecheck
```

**Version verification:** SolidJS 1.9.15, Vite 8.3.0, TypeScript 7.0.2, Vitest 5.0.1, `@playwright/test` 1.63.0, Bun 1.4.2, rustc 1.97.0 — read from `web/package.json` and local toolchain on 2026-09-19. [VERIFIED: web/package.json, bun --version, rustc --version]

## Architecture Patterns

### Recommended Project Structure

```
web/src/render/camera.ts          # add unprojectPoint; keep WORLD_BOUNDS / createCamera
web/src/input/pointer.ts          # NEW: kind parser, CSS point, gesture reducer (pure)
web/src/input/canvas-pointer.ts   # NEW: imperative capture/release adapter
web/src/physics/session.ts        # add pointerAction; poison only on generated throw
web/src/catalog/scenes.ts         # add interactionHint per scene
web/src/components/PlayerPanel.tsx# figcaption from hint; canvas class for touch-action
web/src/App.tsx                   # wire adapter; call cancel on pause/reset/switch/leave
web/src/app.css                   # canvas touch-action:none; select:focus-visible; 480 polish
web/tests/camera.test.ts          # unproject inverse of projectPoint
web/tests/pointer.test.ts         # NEW: kind/finite/gesture reduce
web/e2e/player.spec.ts            # extend; extract helpers if the file crosses ~400 lines
crates/liquidfun-wasm/src/lib.rs  # wasm-bindgen pointer_action
crates/liquidfun-wasm/src/session.rs
crates/liquidfun-wasm/src/scene.rs          # SceneHooks::apply_pointer
crates/liquidfun-wasm/src/scene/*.rs        # one gesture each
```

### Pattern 1: One CSS transform for draw and hit-test

**What:** Invert `projectPoint` using the live `Camera` plus `getBoundingClientRect()` at event time.
**When to use:** Every pointer sample. Never use `canvas.width` / device pixels as physics coordinates.

`resizeCanvasBackingStore` already sizes the backing store with capped DPR and `setTransform(dpr, …)` so drawing stays in CSS pixels. [VERIFIED: web/src/render/canvas.ts] Pointer math must use those same CSS viewport numbers (`camera.viewport`), not the backing-store pixel size.

```ts
// Inverse of projectPoint in web/src/render/camera.ts
// Source: repository camera contract [VERIFIED: web/src/render/camera.ts]
worldX = WORLD_BOUNDS.minX + (cssX - camera.offsetX) / camera.scale
worldY = WORLD_BOUNDS.maxY - (cssY - camera.offsetY) / camera.scale
cssX = event.clientX - rect.left
cssY = event.clientY - rect.top
```

Reject non-finite CSS or world values at the TypeScript boundary and do not call WASM.

### Pattern 2: Shared `pointer_action` session boundary

**What:** One wasm-bindgen method, implemented on `SceneHooks`, kinds `down|move|up|cancel`.
**When to use:** All six scenes. Labeled `apply_control` / `apply_action` stay.

Recommend names:

| Layer | Name |
|-------|------|
| Rust trait | `SceneHooks::apply_pointer(&mut self, world, system, kind, x, y)` |
| wasm-bindgen | `pointer_action(kind: String, world_x: f32, world_y: f32)` |
| TypeScript owner | `SceneSession.pointerAction(kind, worldX, worldY)` |

Parse kind + finite f32 at both boundaries. Unknown kind → `SessionError::UnknownControl` (existing fail-closed vocabulary) or a new `InvalidPointer` with a static message. Do not poison the session for a rejected kind/NaN; ignore at JS if the parser returns `undefined`.

Default trait implementations are **not** allowed to silently no-op a scene: every shipped scene must handle the four kinds.

### Pattern 3: Imperative pointer shell, pure decision core

**What:** Solid `assignCanvas` / `onCleanup` owns listeners. A pure reducer decides capture/release and whether to emit a session command.
**When to use:** Always. Do not put coordinate math or kind parsing inside JSX.

Cleanup must call, in order: `releasePointerCapture` if set → emit WASM `cancel` if a gesture is in-flight → drop local gesture state. Invoke that from pause, reset/retry (`startScene`), hash leave (`abandonScene`), Solid `onCleanup`, and `lostpointercapture`.

Paused scenes may still accept a one-shot down/up (drop/poke) the same way `applySceneAction` already presents a paused frame. A captured drag must still be cancelled if the visitor hits Pause.

### Pattern 4: Per-scene gesture mapping (prescriptive)

Keep labeled controls. Pointer is an extra live path into the same private `BodyId` / emitter / group state.

| Scene | Pointer kinds that matter | Native mapping | Cancel / pause |
|-------|---------------------------|----------------|----------------|
| Dam Break | down+move reposition; up wakes | Captured drag: `set_body_transform` on the existing `circle_body` to clamped world `(x, y)`; up applies the existing wake impulse. Labeled Drop/Reset stay at `(2.5, 7.2)` / `(2.5, 5.5)`. [VERIFIED: dam_break.rs] | Clear `dragging`; do not leave a grabbed body. Leave the last set pose (do not snap unless labeled Reset). |
| Fountain | down+move aim | Live `aim_from_up = atan2(x - 0.0, y - 0.5)` from `NOZZLE_POSITION (0, 0.5)`, matching `aim_velocity` (`sin`/`cos` from up). [VERIFIED: fountain.rs] Clamp to about `±TAU/4`. Do not rewrite the Aim angle `<select>`. | Keep last aim; no emitter leak. |
| Float or Sink | down only | `drop_body` at `(clamp(x), 6.0)` using the selected cork/wood/stone density. Reuse `MAX_DROPPED_BODIES = 4`. [VERIFIED: float_or_sink.rs `DROP_POSITION`] | Down is instantaneous; cancel is a no-op if no drag. |
| Color Mixer | down+move stir; up/cancel clear | Store `maybe_pointer: Option<Vec2>` on hooks. Each `on_advance` while captured, apply engine forces near that point. Keep labeled `stir-speed` global force (`(8,0)` / `(18,0)`). [VERIFIED: color_mixer.rs] Never move positions in JS. | Clear `maybe_pointer` so no leftover force. |
| Jelly Drop | down poke | Impulse near world location. Labeled Poke stays `(0, -8)` on the first-third contiguous slice. [VERIFIED: jelly_drop.rs] | Impulse is one-shot; cancel after down is a no-op. |
| Water Wheel | down+move aim | Keep `JET_POSITION (-3.9, 3.15)` and labeled `jet-speed`. Pointer sets emission **direction** toward the world point, scaled by current speed, still through `create_particle_with_def` in `on_advance`. [VERIFIED: water_wheel.rs] Wheel rotation stays motor-off particle coupling. | Clear override; revert to the default `(speed, -0.4)` vector. |

**Force-range pitfall (do not ignore):** `World::apply_particle_force_range` / `apply_particle_linear_impulse_range` require a **source-ordered contiguous** id range. [VERIFIED: crates/liquidfun/src/world/particle_object/system.rs] Scattered “nearby particles” cannot be passed to the range APIs. Apply per-particle `apply_particle_force` / `apply_particle_linear_impulse` **inside Rust** for localized stir/poke. That is not a JS/Rust per-particle crossing (D-02). Do not expand the public `liquidfun` API.

Clamp Dam Break / drop x to the basin interior (about `[-5.5, 5.5]` / y in `(0, 8)`) so letterboxed clicks in the camera inset do not teleport bodies outside the world.

### Pattern 5: Instruction copy and keyboard path

Add `interactionHint: string` on `SceneRecord`. Render it as the `<figcaption>` (evolve the current generic sentence) or an adjacent `<p>` bound with `aria-describedby` on the canvas. Canvas stays `role="img"`, not `tabindex`. [VERIFIED: PlayerPanel.tsx still has a single static caption]

Suggested copy shape (wording is discretion; structure is not):

- Pointer gesture in one clause.
- “Labeled controls also work from the keyboard.”

Do not steal focus on hash change. Do not add Space-to-pause.

### Anti-Patterns to Avoid

- **Mouse + touch dual stack:** Compatibility mouse events after touch will double-apply drops/pokes.
- **Using `canvas.width` for hit-test:** Backing store is DPR-scaled; physics must stay in CSS/world. [VERIFIED: canvas.ts]
- **`touch-action: none` on `html`/`body`:** Violates D-06 and WCAG 1.4.4 zoom. [CITED: https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action]
- **Checking `event.isTrusted`:** Playwright `dispatchEvent('pointercancel')` is untrusted. [CITED: https://playwright.dev/docs/touch-events]
- **Replacing labeled buttons with canvas-only input:** Violates D-01 / WEB-07.
- **JS position animation to “prove” a poke/stir:** Violates D-04 / DEMO honesty.
- **New Playwright browser projects:** Violates D-12/D-13.
- **Treating `17-HOST-EVIDENCE.md` as Phase 19 proof:** SHA `50a1556`, Dam Break only. [VERIFIED: 17-HOST-EVIDENCE.md]
- **Piling pointer code into `App.tsx`:** 593/628 line trigger. [VERIFIED: wc -l]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mouse vs touch unification | Two event stacks + `preventDefault` maze | Pointer Events + `setPointerCapture` | Capture retargets move/up after the pointer leaves the canvas. [CITED: MDN setPointerCapture] |
| Browser taking over a canvas pan | Document-level scroll lock | `touch-action: none` **on the canvas only** | Without it, the browser emits `pointercancel` and starts scrolling. [CITED: MDN touch-action] |
| CSS↔world math | Ad-hoc `x / canvas.width * 12` | Invert `projectPoint` | PITFALLS.md: draw and hit-test must share one transform. [VERIFIED: .planning/research/v1.1/PITFALLS.md] |
| Browser smoke | Ad-hoc sleeps / screenshot goldens | Existing Playwright Chromium + `data-step-index` | Suite already loops six scenes. [VERIFIED: web/e2e/player.spec.ts] |
| Live Pages proof | New browser matrix | curl/Playwright against the recorded origin + SHA | Phase 17 evidence pattern. [VERIFIED: 17-HOST-EVIDENCE.md] |
| Localized particle forces | Public engine API expansion | Existing per-particle force/impulse **in Rust** | Range APIs need contiguous ids. [VERIFIED: system.rs] |
| Focus styles | Custom canvas key map | Native buttons/selects + CSS `:focus-visible` | D-08/D-10. |
| UI kit | MysticUI/Tailwind/icons | Scoped `app.css` | D-09. |

**Key insight:** The expensive mistakes here are coordinate-space drift and leftover capture/force, not missing libraries. The stack is already chosen.

## Common Pitfalls

### Pitfall 1: Pointer drift after resize or high-DPI

**What goes wrong:** Drag misses the obstacle; aim feels offset; narrow width is worse.
**Why it happens:** Hit-test uses backing-store pixels or a stale rect/camera. [CITED: .planning/research/v1.1/PITFALLS.md]
**How to avoid:** Read `getBoundingClientRect()` per event; unproject with the current `maybeCamera` from ResizeObserver. Do not rebuild the world (existing path already only redraws `maybePreviousFrame`). [VERIFIED: App.tsx `connectResizeObserver`]
**Warning signs:** First click after a window resize or 375px viewport lands in empty space.

### Pitfall 2: Browser cancels the gesture and scrolls the page

**What goes wrong:** Touch-drag on the canvas scrolls the catalog; `pointercancel` leaves a stuck stir/drag.
**Why it happens:** Default `touch-action: auto` lets the browser claim the gesture. [CITED: MDN touch-action]
**How to avoid:** `canvas { touch-action: none; }`. `preventDefault` only after capture, only on the canvas. Handle `pointercancel` and `lostpointercapture` as cancel.
**Warning signs:** First mobile/narrow touch scrolls the page instead of aiming.

### Pitfall 3: Stuck force, grabbed body, or captured pointer

**What goes wrong:** Color Mixer keeps stirring after lift; Dam Break obstacle follows nothing; next scene inherits capture.
**Why it happens:** Cleanup disposes the session but not gesture state, or `on_advance` keeps `maybe_pointer`.
**How to avoid:** One `cancelCanvasPointer()` used by pause, `startScene`, `abandonScene`, `onCleanup`, and lost-capture. WASM `cancel` clears hook transients **before** `dispose()` when possible.
**Warning signs:** After Pause or scene switch, particles still accelerate with no pointer down.

### Pitfall 4: Contiguous-range force API rejects localized stir/poke

**What goes wrong:** Session poisons; “Rust/WASM session failed”.
**Why it happens:** Nearby particle ids are not a contiguous source range. [VERIFIED: system.rs docs]
**How to avoid:** Per-particle engine calls inside the scene module; or a contiguous slice around the nearest member (Jelly already does a first-third slice).
**Warning signs:** First Color Mixer / Jelly pointer down fails the session.

### Pitfall 5: Colorful canvas with no keyboard/instruction path

**What goes wrong:** WEB-07 fails even if gestures work.
**Why it happens:** Figcaption is still the generic Canvas 2D sentence; `select` has no `:focus-visible` rule. [VERIFIED: PlayerPanel.tsx, app.css lines 388–390]
**How to avoid:** Per-scene hint + add `select:focus-visible` to the existing 2px/4px accent outline. Keep 44px targets and 480px full-width stacking.
**Warning signs:** Keyboard Tab into Gravity/Aim angle shows the browser default or nothing.

### Pitfall 6: Live Pages proof uses a stale SHA or only Dam Break

**What goes wrong:** WEBTEST-01 looks green from Phase 17 paper.
**Why it happens:** `17-HOST-EVIDENCE.md` records `50a1556` and one hash URL. Phase 18 explicitly did not require a new Pages URL. [VERIFIED: 17-HOST-EVIDENCE.md, STATE.md Phase 18 D-16]
**How to avoid:** After this phase lands on `main`, write `19-HOST-EVIDENCE.md` with `page_url`, `source_sha`, workflow run, WASM content-type, and **six** refreshed `#/scene/{id}` checks.
**Warning signs:** Evidence file cites Phase 17 SHA or a single scene.

### Pitfall 7: Playwright timeout / flake on six-scene pointer loops

**What goes wrong:** `SIX_SCENE_TIMEOUT_MS` (120s) is already used; adding gestures + 375px can exceed 30s per test (config timeout). [VERIFIED: player.spec.ts, playwright.config.ts]
**How to avoid:** Keep `test.setTimeout(120_000)` on the six-scene loop. Assert `data-step-index` growth and a cheap `data-last-pointer-kind` (or incrementing `data-pointer-accepted`) rather than screenshot hashes. `page.setViewportSize({ width: 375, height: 812 })` **before** `goto` for the narrow pass. [CITED: https://playwright.dev/docs/api/class-page]
**Warning signs:** Tests pass locally at 1280×720 and fail only on the 375 pass, or hit the 30s default timeout.

## Code Examples

Verified patterns from this repo and official docs:

### Unproject CSS → world

```ts
// Source: invert of projectPoint [VERIFIED: web/src/render/camera.ts]
export function unprojectPoint(camera: Camera, point: Point): Point {
  return {
    x: WORLD_BOUNDS.minX + (point.x - camera.offsetX) / camera.scale,
    y: WORLD_BOUNDS.maxY - (point.y - camera.offsetY) / camera.scale,
  };
}
```

Unit-test: `unprojectPoint(camera, projectPoint(camera, world))` round-trips the basin corners and a 320×540 narrow camera.

### Capture and cancel

```ts
// Source: https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture
canvas.addEventListener("pointerdown", (event) => {
  canvas.setPointerCapture(event.pointerId);
  event.preventDefault(); // only on the canvas, only after capture (D-06)
  send("down", event);
});
canvas.addEventListener("pointermove", (event) => {
  if (!canvas.hasPointerCapture(event.pointerId)) {
    return;
  }
  send("move", event);
});
function finish(kind: "up" | "cancel", event: PointerEvent): void {
  send(kind, event);
  if (canvas.hasPointerCapture(event.pointerId)) {
    canvas.releasePointerCapture(event.pointerId);
  }
}
canvas.addEventListener("pointerup", (event) => finish("up", event));
canvas.addEventListener("pointercancel", (event) => finish("cancel", event));
canvas.addEventListener("lostpointercapture", (event) => {
  // Source: https://developer.mozilla.org/en-US/docs/Web/API/Element/lostpointercapture_event
  send("cancel", event);
});
```

Programmatic cleanup uses the stored `maybePointerId` and still emits WASM `cancel`.

### WASM boundary

```rust
// Source: extend crates/liquidfun-wasm/src/lib.rs apply_action pattern [VERIFIED]
#[wasm_bindgen(js_name = pointerAction)]
pub fn pointer_action(
    &mut self,
    kind: String,
    world_x: f32,
    world_y: f32,
) -> Result<(), JsError> {
    self.core.apply_pointer(&kind, world_x, world_y).map_err(js_error)
}
```

Reject non-finite `world_x` / `world_y` before `hooks.apply_pointer`.

### Playwright 375px + pointer + cancel

```ts
// Source: https://playwright.dev/docs/api/class-page  + existing player.spec helpers
await page.setViewportSize({ width: 375, height: 812 });
await page.goto("/liquidfun-rs/#/scene/dam-break", { waitUntil: "domcontentloaded" });
const canvas = page.locator("canvas");
const box = await canvas.boundingBox();
await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
await page.mouse.down();
await page.mouse.move(box.x + box.width * 0.7, box.y + box.height / 3);
await canvas.dispatchEvent("pointercancel", { pointerId: 1 });
await expect(page.locator("main")).toHaveAttribute("data-last-pointer-kind", "cancel");
await page.locator(".catalog-nav").evaluate((node) => {
  node.scrollIntoView();
});
expect(await page.evaluate(() => document.scrollingElement?.scrollHeight ?? 0))
  .toBeGreaterThan(await page.evaluate(() => window.innerHeight));
```

Do not add a second Playwright `projects` entry. Hidden-tab recovery already exists — keep it and run it after at least one pointer gesture. [VERIFIED: player.spec.ts hidden-tab test]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 17/18 canvas is `role="img"` with no listeners | Same canvas + Pointer Events; still not a keyboard widget | Phase 19 | Instructions + labeled controls are the a11y path |
| Labeled discrete aim/drop/poke only | Labeled path kept; canvas sends world coordinates | Phase 18 → 19 | D-01 + D-03 |
| Player smoke: open/reset/switch | Same suite + pointer + 375px + live six-hash record | Phase 18 → 19 | WEBTEST-01 |
| HOST-03 Dam Break-only Pages file | New Phase 19 evidence for the full gallery | After this deploy | Do not reuse SHA `50a1556` |

**Deprecated/outdated:**

- Dual mouse/touch stacks — Pointer Events are Baseline and locked.
- Visual SHA goldens as the primary oracle — D-13.
- MysticUI default from `standards/languages/typescript-javascript.md` — local override until 2026-12-17.
- Phase 16 `just web-smoke` forensic Dispose-session proof — not the product gate; `just web-player-smoke` is. [VERIFIED: TESTING.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Existing `#F4F7FA` / `#A9B4C0` on `#0B0F14` / `#151D28` still meet WCAG AA at 375px without new tokens | WEB-07 | Planner might schedule an unauthorized palette change. Mitigate: visual 375px pass, keep tokens. |
| A2 | Rust-side per-particle force/impulse loops satisfy D-02 | Color Mixer / Jelly | If interpreted as forbidden, localized stir/poke cannot use nearby ids. Confirm: D-02 bans JS/Rust per-particle *calls*, not Rust-internal loops. |
| A3 | `dispatchEvent('pointercancel')` is enough for Chromium cancel proof if handlers ignore `isTrusted` | WEBTEST-01 | Need CDP touch cancel instead; keep handlers trust-agnostic. |
| A4 | Live Pages after Phase 18 already hosts six ready scenes, but Phase 19 must still re-record URL/SHA | D-13 | Evidence might open a pre-18 site; always bind the evidence file to the post-phase SHA. |
| A5 | Captured Dam Break drag (vs click-to-reposition) is the better discretion default | D-03 / Discretion | Click-only still satisfies D-03 if capture/cancel/resize hold. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

Discretion items A2 and A5 do **not** need a user decision; they are planner defaults.

## Open Questions

1. **Dam Break drag vs click-to-reposition**
   - What we know: D-03 allows either; existing actions teleport to fixed poses.
   - What's unclear: which feels better after resize.
   - Recommendation: captured drag with clamped `set_body_transform`; keep labeled Drop/Reset.

2. **Water Wheel aim vs strength**
   - What we know: jet is `(speed, -0.4)` from a fixed left nozzle; wheel is motor-off.
   - What's unclear: whether visitors expect a stronger jet or a steered jet.
   - Recommendation: steer direction from nozzle to pointer; labeled Jet strength still sets speed.

3. **DOM oracle for pointer acceptance**
   - What we know: `data-step-index` proves stepping, not that WASM received a pointer.
   - What's unclear: whether a new attribute is acceptable chrome.
   - Recommendation: add `data-last-pointer-kind` and `data-pointer-accepted` on `main`, analogous to `data-step-index`. Not user-visible.

4. **Figcaption vs adjacent paragraph**
   - What we know: D-11 allows either.
   - Recommendation: replace the current figcaption string with `interactionHint` so the figure stays the instruction home.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Bun | unit/typecheck/preview | ✓ | 1.4.2 | — |
| Rust + Cargo | `liquidfun-wasm` tests + wasm-pack | ✓ | rustc 1.97.0 | — |
| `wasm32-unknown-unknown` | WASM rebuild | ✓ | installed | `rustup target add wasm32-unknown-unknown --toolchain 1.97.0` |
| `@playwright/test` + Chromium | D-12 | ✓ | 1.63.0 package | `cd web && bun run browser:install` |
| just | `web-player-smoke` | ✓ | 1.48.0 local (repo pin 1.55.1 is docs-only) | invoke `bun scripts/web-build.ts player-smoke` |
| GitHub Pages origin | D-13 | ✓ | `https://bright-builds-llc.github.io/liquidfun-rs/` | Record actual URL if settings change |
| Firefox / WebKit | — | n/a | — | Out of scope (D-13) |
| MysticUI / Tailwind | — | n/a | — | Forbidden (D-09) |

**Missing dependencies with no fallback:** none identified.

**Missing dependencies with fallback:** none that block the phase. Playwright Chromium must be installed via the package script if the local machine has not run `browser:install` recently.

Step 2.6 tools probed on 2026-09-19.

## Recommended Plan Slices

Fine granularity. UI-SPEC (`workflow.ui_phase: true`, roadmap UI hint yes) should lock instruction markup, canvas class, and 375/480 wrapping **before** implementation plans mutate CSS/copy.

1. **19-01 Camera unproject + pointer parser** — `unprojectPoint`, finite/kind parsers, gesture reducer; Vitest. No DOM.
2. **19-02 WASM `pointer_action` + six scene mappings** — trait method, wasm-bindgen, native tests per scene including cancel-clears-transient and fail-closed kinds. No JS physics.
3. **19-03 Player pipeline + cleanup** — extract `canvas-pointer.ts`; wire from `App`; CSS `touch-action: none` on canvas only; cancel on pause/reset/switch/leave/`onCleanup`.
4. **19-04 WEB-07 chrome** — `interactionHint`, figcaption, `select:focus-visible`, 480px wrapping polish. No hamburger.
5. **19-05 Chromium smoke** — extend `player.spec.ts`: six scenes × pointer + labeled control, resize-then-drag, `pointercancel`, hidden-tab after a gesture, 375px focus/scroll. `just web-player-smoke`.
6. **19-06 Live Pages + independent review** — ordinary main push, `19-HOST-EVIDENCE.md` for six refreshed hashes + WASM MIME + SHA, digest-bound AI review. Implementing agent does not acknowledge its own digest.

Do not put Playwright into `.github/workflows/pages.yml` (Phase 17 already kept it local).

## Security Domain

`security_enforcement` is not set to `false` in `.planning/config.json` (absent = enabled). This is a static GitHub Pages playground with no accounts.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | One in-tab WASM session; not an auth session |
| V4 Access Control | no | Public static site |
| V5 Input Validation | yes | Parse pointer kind + finite world x/y at JS and Rust boundaries; clamp scene poses; existing host-locked credit hrefs |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for SolidJS + WASM playground

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| NaN/Inf world coordinates poisoning the engine | Tampering / Denial of service | Reject non-finite values before `World` mutation; do not `innerHTML` error text |
| Unbounded pointer-move WASM spam | Denial of service | One captured pointer; ignore uncaptured moves; existing 1–4 step cap stays |
| Leftover stir force after navigation | Denial of service | Mandatory WASM `cancel` + hook clear on teardown |
| XSS via instruction copy | Tampering | Static catalog strings in JSX text, never `innerHTML` |
| Open redirect on credit/source links | Tampering | Keep Phase 18 host-locked URL builder; do not take pointer/hash input as hrefs |
| Raw engine pointer / BodyId leak | Information disclosure | Body/group ids stay in Rust hooks (existing contract) |

## Sources

### Primary (HIGH confidence)

- `web/src/App.tsx`, `web/src/physics/session.ts`, `web/src/render/camera.ts`, `web/src/render/canvas.ts`, `web/src/components/PlayerPanel.tsx`, `web/src/components/SceneControls.tsx`, `web/src/catalog/scenes.ts`, `web/src/app.css`, `web/e2e/player.spec.ts` — current player/camera/e2e surface.
- `crates/liquidfun-wasm/src/{lib,session,scene}.rs` and `scene/*.rs` — labeled controls and scene constants.
- `crates/liquidfun/src/world/particle_object/system.rs` — contiguous range requirement for force/impulse.
- [MDN `setPointerCapture`](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture) — capture until up/release.
- [MDN `lostpointercapture`](https://developer.mozilla.org/en-US/docs/Web/API/Element/lostpointercapture_event) — fire on capture loss.
- [MDN `touch-action`](https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action) — canvas-only `none`; document zoom warning.
- [Playwright `page.setViewportSize`](https://playwright.dev/docs/api/class-page) — 375px pass in the existing Chromium project.
- `.planning/research/v1.1/{FEATURES,ARCHITECTURE,PITFALLS,SUMMARY}.md` — one gesture/scene, `pointerAction`, resize/DPI, scroll isolation.
- `17-HOST-EVIDENCE.md`, `18-UI-SPEC.md`, `19-CONTEXT.md` — Pages pattern, tokens, locked decisions.

### Secondary (MEDIUM confidence)

- [Playwright touch events](https://playwright.dev/docs/touch-events) — `dispatchEvent` is untrusted; do not require `isTrusted`.
- Phase 17 decision to keep Playwright out of the Pages workflow — still apply.

### Tertiary (LOW confidence)

- Exact WCAG numeric contrast at 375px for muted captions — tokens already accepted by 18-UI-SPEC; verify visually rather than retoken.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — versions and recipes read from the repo and local toolchain.
- Architecture: HIGH — camera, session, scene hooks, and e2e extension points inspected; per-scene force magnitudes remain discretion.
- Pitfalls: HIGH — documented in v1.1 PITFALLS plus concrete current-code gaps (`select:focus-visible`, no unproject, generic figcaption, stale HOST-03 file).

**Research date:** 2026-09-19
**Valid until:** 2026-10-19 (stable playground stack; re-check only if Solid/Playwright pins move)

## Current Surface Inventory (planner checklist)

| Surface | Today | Phase 19 must |
|---------|-------|---------------|
| Canvas | 960×540 CSS 16:9, `role="img"`, no listeners, generic figcaption | Pointer pipeline; keep role; per-scene hint |
| Camera | `createCamera` / `projectPoint` only | Add `unprojectPoint`; tests for inverse + narrow viewport |
| Session JS | `nextFrame` / `applyControl` / `applyAction` / `dispose` | `pointerAction`; update `FakeGeneratedProofSession` |
| Session Rust | `SceneHooks` control/action | `apply_pointer`; all six modules |
| Resize | Last-frame redraw, no world rebuild | Convert next pointer through new CSS bounds; e2e after resize |
| CSS | 44px controls; focus on `button`/`a` only; 480px stack | Add `select:focus-visible`; `canvas { touch-action: none }`; wrap polish |
| e2e | Six-scene open/reset, Dam Break pause/play, construction Apply, hidden-tab, unknown hash | Pointer + labeled control, cancel, 375px, resize-drag; keep existing cases |
| Pages | Origin known; evidence SHA `50a1556` Dam Break-only | New recorded URL/SHA for all six hashes after this deploy |

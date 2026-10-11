---
phase: quick-261010-vzz
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/render/safe-frame.ts
  - web/src/render/camera.ts
  - web/src/render/canvas.ts
  - web/src/player/camera-frame.ts
  - web/src/player/hud-sliders-preference.ts
  - web/src/player/hud-obstacles.ts
  - web/src/player/frame-loop.ts
  - web/src/player/scene-surface.ts
  - web/src/player/scene-runtime.ts
  - web/src/components/CanvasStage.tsx
  - web/src/components/SceneControls.tsx
  - web/src/components/PlayerPanel.tsx
  - web/src/components/PlaygroundStage.tsx
  - web/src/App.tsx
  - web/src/styles/canvas-stage.css
  - web/src/catalog/portrait-bounds.ts
  - web/tests/safe-frame.test.ts
  - web/tests/camera.test.ts
  - web/tests/camera-frame.test.ts
  - web/tests/hud-sliders-preference.test.ts
  - web/tests/hud-sliders-toggle.test.ts
  - web/tests/portrait-bounds.test.ts
  - web/e2e/hud-safe-frame.spec.ts
  - AGENTS.md
autonomous: true
requirements: [quick-261010-vzz]
generated_by: gsd-plan-phase
lifecycle_mode: quick
phase_lifecycle_id: quick-261010-vzz
generated_at: "2026-10-11T04:06:55Z"
must_haves:
  truths:
    - "On the default canvas stage, every catalog scene at 390x844 and 1280x800 (and every scene with HUD controls also at 844x390 and 768x1024) draws its world rectangle (canvas data-world-frame) clear of every [data-hud-obstacle] box within 1px, unless data-world-frame-fallback is set, in which case hiding the sliders clears both the overlap and the fallback flag."
    - "On a wide viewport with a bottom-left slider card, the scene is placed beside the card using the full height left over by the top chrome, not squeezed above a full-width bottom band (spec Decision 2)."
    - "Scenes with at least one HUD control show a toggle at the leading edge of the bottom toolbar, labelled 'Hide sliders' when shown and 'Show sliders' when hidden, with aria-expanded and aria-controls; scenes without HUD controls show no toggle."
    - "Clicking the toggle removes the slider card from the DOM, enlarges the fitted world frame, persists 'hidden' under localStorage key liquidfun.hud-sliders.v1 across scene changes and reloads, and clicking again restores the original frame."
    - "createCamera without a frame argument returns exactly what it returns today, so the classic ?shell=form-factor layout, SVG, README, and recording exports are unchanged."
    - "Pausing and resuming a scene does not move data-world-frame (obstacle boxes do not depend on camera scale or transient text)."
    - "Tesla valve's portrait frame no longer has hand-tuned empty space below the drain, and portrait-bounds.test.ts fill and wall checks pass."
  artifacts:
    - path: web/src/render/safe-frame.ts
      provides: "Pure safeFrame(viewport, obstacles, worldAspect) best-fit search plus ScreenRect helpers"
      exports: ["ScreenRect", "SafeFrame", "safeFrame", "SAFE_FRAME_GAP_PX", "screenRectInCanvas", "sameScreenRects"]
    - path: web/src/render/camera.ts
      provides: "createCamera optional final frame parameter and projectedWorldRect"
      contains: "frame"
    - path: web/src/player/camera-frame.ts
      provides: "cameraFrame(clock) and publishWorldFrame(canvas, camera, usedFallback)"
      exports: ["cameraFrame", "publishWorldFrame"]
    - path: web/src/player/hud-sliders-preference.ts
      provides: "liquidfun.hud-sliders.v1 load/persist and toggle label"
      contains: "liquidfun.hud-sliders.v1"
    - path: web/src/player/hud-obstacles.ts
      provides: "ResizeObserver + MutationObserver shell publishing a Solid signal of canvas-space obstacle rects"
      exports: ["createHudObstacles"]
    - path: web/e2e/hud-safe-frame.spec.ts
      provides: "Catalog-driven overlap, fallback, and toggle round-trip browser checks"
      contains: "SCENE_IDS"
  key_links:
    - from: web/src/player/frame-loop.ts
      to: web/src/player/camera-frame.ts
      via: "canvas ResizeObserver passes cameraFrame(clock).frame to resizeCanvasBackingStore and calls publishWorldFrame"
      pattern: "cameraFrame\\(clock\\)"
    - from: web/src/player/scene-surface.ts
      to: web/src/player/camera-frame.ts
      via: "refreshCamera passes cameraFrame(session.clock).frame to createCamera and calls publishWorldFrame"
      pattern: "cameraFrame\\(session\\.clock\\)"
    - from: web/src/components/PlayerPanel.tsx
      to: web/src/player/hud-obstacles.ts
      via: "createHudObstacles(viewportFrame, canvas) in the canvas-stage layout, forwarded to onHudObstaclesChange"
      pattern: "createHudObstacles"
    - from: web/src/player/scene-surface.ts
      to: "session.clock.hudObstacles"
      via: "changeHudObstacles sets clock.hudObstacles, refreshes the camera, repaints the held frame"
      pattern: "clock\\.hudObstacles ="
    - from: web/src/components/CanvasStage.tsx
      to: web/src/player/hud-sliders-preference.ts
      via: "toggle signal initialised by loadHudSlidersVisibility and persisted on click"
      pattern: "persistHudSlidersVisibility"
---

<objective>
Implement the approved HUD safe frame design (docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md, commit 0f1b6f1d1): a persistent show/hide toggle for on-canvas HUD sliders, and a chrome-aware camera fit that places the world in the best obstacle-free rectangle around measured HUD chrome on the default canvas stage.

Purpose: HUD sliders and other chrome must never cover a scene on any viewport, and new scenes, sliders, and chrome must get correct framing without per-scene tuning.
Output: pure safe-frame core with unit tests, measurement shell and UI wiring, Tesla valve portrait cleanup, updated framing guidance, and a catalog-driven Playwright spec.

All five spec Decisions and Components 1-4, Cleanup, Testing, and Done criteria are LOCKED. Do not revisit them. Out of scope (do not touch): the classic `?shell=form-factor` layout behaviour, SVG/README/recording exports, gravity and other drawer controls.

Guidance that informed this plan: AGENTS.md Repo-Local Guidance (Playground canvas framing, mdformat, standing authorization), AGENTS.bright-builds.md (functional core / imperative shell, `maybe` prefix, file-length triggers, sync first), standards/core/testing.md (Arrange/Act/Assert, one concern per test), standards/languages/typescript-javascript.md (Bun, SolidJS).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@AGENTS.md
@docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md

Executor notes:
- Work from `web/` for all bun commands. Bun is the package manager. There is no lint script in web/package.json; typecheck (`bun run typecheck`) is the lint-equivalent gate. Record that in SUMMARY rather than inventing a lint step.
- vitest runs with `environment: "node"` and `include: ["tests/**/*.test.ts"]`. There is no DOM library and none may be added (minimize dependencies). The spec's "component test with the same DOM setup as scale-legend.test.ts" therefore follows scale-legend.test.ts's actual setup: reading component source with `readFileSync`. Click/aria/persistence behaviour is proven in Playwright (Task 3).
- Playwright's webServer runs `bun run preview`, which serves `web/dist`. Always run `bun run build:app` before Playwright. The generated wasm in `web/src/generated/liquidfun-wasm/` is present locally (gitignored); if it is missing, run `just web-wasm` from the repo root (needs cargo).
- On this Mac, fresh binaries can stall ~2 minutes on first launch; do not launch several fresh browser/binary processes in parallel. This task is TypeScript-only; no cargo builds are needed.
- `CLAUDE.md` is a symlink to `AGENTS.md`. Edit only `AGENTS.md`; the symlink updates both.
- `.planning/**` must never be run through mdformat.
- Tests: Arrange/Act/Assert with explicit `// Arrange`, `// Act`, `// Assert` comments, one concern per test. Nullable/optional names use the `maybe` prefix. The only swallowed errors allowed are the spec'd storage try/catch fallbacks in hud-sliders-preference.ts.

<interfaces>
From web/src/render/camera.ts (current):
```typescript
export type WorldBounds = { readonly minX: number; readonly minY: number; readonly maxX: number; readonly maxY: number };
export const WORLD_BOUNDS: WorldBounds;            // -6,-1 .. 6,8
export const VIEWPORT_INSET = 16;
export interface ViewportSize { readonly width: number; readonly height: number }
export interface Camera { readonly viewport: ViewportSize; readonly bounds: WorldBounds; readonly scale: number; readonly offsetX: number; readonly offsetY: number }
export interface CameraView { readonly zoom: number; readonly panX: number; readonly panY: number }
export const IDENTITY_CAMERA_VIEW: CameraView;
export function createCamera(width: number, height: number, view?: CameraView, bounds?: WorldBounds): Camera;
// today: fittedScale = min((w-32)/worldW, (h-32)/worldH); scale = fittedScale*clampCameraZoom(zoom)
// offsetX = (viewport.width - worldW*scale)/2 + panX; offsetY = (viewport.height - worldH*scale)/2 + panY
export function projectPoint(camera: Camera, point: Point): Point; // x = offsetX + (p.x-minX)*scale; y = offsetY + (maxY-p.y)*scale
```

From web/src/render/canvas.ts:
```typescript
export function resizeCanvasBackingStore(canvas: HTMLCanvasElement, cssWidth: number, cssHeight: number, devicePixelRatio: number, view?: CameraView, bounds?: WorldBounds): Camera;
```

From web/src/player/frame-loop.ts:
```typescript
export type FrameClock = { ...; viewportWidth: number; viewportHeight: number; maybeCamera: Camera | undefined; cameraView: CameraView; worldBounds: WorldBounds };
export function createFrameClock(): FrameClock;
export function connectResizeObserver(clock: FrameClock, canvas: HTMLCanvasElement, context: CanvasRenderingContext2D, deps: FrameLoopDeps): void;
// ResizeObserver body (~line 250-290): sets clock.viewportWidth/Height, clock.worldBounds = worldBoundsForViewport(id, w, h),
// then resizedCamera = resizeCanvasBackingStore(canvas, w, h, window.devicePixelRatio, clock.cameraView, clock.worldBounds)
```

From web/src/player/scene-surface.ts:
```typescript
function refreshCamera(session: SceneRuntime): void; // guards isUsableViewport, then session.clock.maybeCamera = createCamera(w, h, cameraView, worldBounds); session.setMaybePixelsPerMeter(scale)
function paintHeldFrame(session: SceneRuntime): void;
function applyCameraView(session, nextView) { clock.cameraView = nextView; refreshCamera; paintHeldFrame } // resetCameraView uses IDENTITY_CAMERA_VIEW
export function bindSurface(session: SceneRuntime): void; // assigns session.refreshCamera, session.paintHeldFrame, ...
```

From web/src/render/mode.ts (preference pattern to copy):
```typescript
export const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";
type RenderModeStorage = Pick<Storage, "getItem" | "setItem">;
export type RenderModeStorageProvider = () => RenderModeStorage;
export function loadRenderMode(storageProvider: RenderModeStorageProvider): RenderMode; // try { parse(storageProvider().getItem(KEY)) ?? default } catch { default }
export function persistRenderMode(storageProvider: RenderModeStorageProvider, mode: RenderMode): void; // try { setItem } catch {}
```

From web/src/components/CanvasStage.tsx (CanvasHud, ~line 132): renders `.canvas-hud-stats` (RenderedFps, DebugToggle, debugReadout), `.canvas-hud-title` (h2#player-title + output.session-status), `.canvas-hud-bottom` ({props.children} = HudControlSlider card, then `.canvas-hud-toolbar` with `.canvas-transport` and `.canvas-hud-trailing` [ScaleLegend + ViewportTools]), `.canvas-hud-corner` (SceneControlsTrigger, AccelerometerToggle, GravityArrow). Props already include `hasHudSlider: boolean` (= sceneControlsForSurface(scene.controls, "hud").length > 0, computed in PlaygroundStage.hudControls and passed by PlayerPanel). Toggles use `class="hud-button"`, `type="button"`, aria-label, inline `<svg viewBox="0 0 24 24" aria-hidden="true">` icons.

From web/src/components/SceneControls.tsx:
```typescript
export function HudControlSlider(props: HudControlSliderProps): JSX.Element; // <Show when=... keyed><div class="canvas-hud-slider"><SceneControls .../></div></Show>
```
CSS (web/src/styles/canvas-stage.css): `.canvas-hud-toolbar { display:flex; flex-wrap:wrap; align-items:flex-end; justify-content:space-between; gap:4px }` - a third direct child would break the two-ended layout, so wrap toggle + transport in a leading group.

PlayerPanel.tsx (PlayerPanelLayout ~line 117) renders `<div class="viewport-frame">` containing the `canvas.scene-canvas` (ref -> props.assignCanvas) and, when canvasStage, `<CanvasHud>`. PlaygroundStage.tsx passes props down; App.tsx binds them to `session.*` from createPlaygroundSession. SceneRuntime type lives in web/src/player/scene-runtime.ts.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Pure safe-frame core, camera frame parameter, and slider preference (TDD)</name>
  <files>web/src/render/safe-frame.ts, web/src/render/camera.ts, web/src/render/canvas.ts, web/src/player/camera-frame.ts, web/src/player/frame-loop.ts (FrameClock.hudObstacles field only), web/src/player/hud-sliders-preference.ts, web/tests/safe-frame.test.ts, web/tests/camera.test.ts, web/tests/camera-frame.test.ts, web/tests/hud-sliders-preference.test.ts</files>
  <behavior>
    safe-frame.test.ts (spec Testing list, exact cases):
    - no obstacles returns the full viewport {0,0,W,H} with usedFallback false
    - a full-width bottom band (e.g. 390x844 viewport, obstacle top 700) returns the region above it, bottom = 692 (8px gap respected; this doubles as the gap test or add a separate gap test)
    - a narrow bottom-left card on a wide viewport (1280x800, aspect 4/3, card {left:16, top:500, right:368, bottom:784}) returns the full-height region to its right: {left:376, top:0, right:1280, bottom:800}
    - equal side regions choose the one nearer the centre (e.g. 1200x400, aspect 1, full-height strip x 600..640: both sides score 368 height-limited; left [0,592] centre 296 is nearer 600 than right [648,1200] centre 924, so left wins)
    - shuffled obstacle order returns the identical frame (run several permutations of a 4-6 obstacle set)
    - a full-canvas obstacle triggers the fallback: full viewport, usedFallback true
    - best fit below 50% of the unobstructed score also falls back (e.g. bands leaving a thin strip)
    - screenRectInCanvas converts a client rect into canvas coordinates by subtracting the canvas client left/top; sameScreenRects is order-sensitive equality within 0.5px
    camera.test.ts (add, keep all 14 existing tests untouched):
    - createCamera without frame equals createCamera with frame {0,0,W,H} and equals today's numbers for 960x540 and a narrow viewport (toEqual on the whole Camera)
    - a frame centres the world inside it with the 16px inset applied inside the frame (scale = min((fw-32)/worldW,(fh-32)/worldH); projected world centre = frame centre)
    - zoom and pan compose as before inside a frame (zoom multiplies fitted scale about the frame centre; pan adds to offset)
    - a frame with no room for the inset throws "Invalid Canvas viewport"
    - projectedWorldRect(camera) returns the projected world rectangle {left: offsetX, top: offsetY, right: offsetX + worldW*scale, bottom: offsetY + worldH*scale}
    camera-frame.test.ts:
    - cameraFrame uses clock.viewportWidth/Height, clock.hudObstacles, and the aspect of clock.worldBounds (a portrait world with a bottom band yields the region above the band)
    - empty hudObstacles returns the full viewport
    hud-sliders-preference.test.ts:
    - default is "shown" when storage is empty
    - a persisted "hidden" round-trips through load
    - a throwing getItem reads as "shown"; an unknown stored value reads as "shown"; a throwing setItem does not throw
    - hudSlidersToggleLabel("shown") === "Hide sliders", ("hidden") === "Show sliders"
  </behavior>
  <action>
Step 0 (sync and baseline, per AGENTS.bright-builds.md "sync first" and spec Done criteria "before and after"): from the repo root run `git pull --rebase` (worktree is clean), then in `web/` run `bun install --frozen-lockfile` and `bun run build:app`. Start `bun run preview -- --strictPort` in the background and capture BEFORE screenshots, one at a time, with `bunx playwright screenshot --viewport-size=W,H --wait-for-timeout=4000 'http://127.0.0.1:4173/liquidfun-rs/#/scene/<id>' test-results/hud-safe-frame-visual/before-<id>-<W>x<H>.png` for ids wave-tank and tesla-valve at 390,844 and 1280,800 (4 files; test-results/ is gitignored). Stop the preview server afterwards.

RED first: write the four test files from the behavior block, run `bun run test:unit` and confirm the new tests fail (module/export missing). Commit `test(quick-261010-vzz): add failing safe-frame, camera frame, and slider preference tests`.

GREEN:
1. `web/src/render/safe-frame.ts` (new, pure, no DOM): export `ScreenRect` and `SafeFrame` exactly as in the spec (Component 2), `SAFE_FRAME_GAP_PX = 8`, `SAFE_FRAME_FALLBACK_RATIO = 0.5`, `SAFE_FRAME_SCORE_TOLERANCE = 0.005`, and `safeFrame(viewport: ViewportSize, obstacles: readonly ScreenRect[], worldAspect: number): SafeFrame`. Import `VIEWPORT_INSET` and `type ViewportSize` from `./camera`. Algorithm exactly per spec: inflate each obstacle by 8px and clip to the viewport (drop obstacles with no positive area after clipping); candidate x edges = 0, W, and every inflated obstacle's left/right (dedupe and sort); y likewise; for every ordered pair x0<x1 and y0<y1, reject a rectangle that intersects any inflated obstacle with positive area (strict overlap: `a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom`); score = `min((w - 2*VIEWPORT_INSET) / worldAspect, (h - 2*VIEWPORT_INSET) / 1)`; reject non-positive scores; among rectangles with score >= best*(1-0.005) pick the one whose centre is nearest the viewport centre (squared distance), then lowest top, then lowest left (so obstacle order cannot matter); if none remain or best < 0.5 * score of the full viewport, return `{ frame: {0,0,W,H}, usedFallback: true }`. Also export pure helpers `screenRectInCanvas(rect: ScreenRect, canvasLeft: number, canvasTop: number): ScreenRect` and `sameScreenRects(a, b): boolean` (same length, each edge within 0.5px). Add a short doc comment per export. Keep functions under ~60 lines with early returns.
2. `web/src/render/camera.ts`: add `import type { ScreenRect } from "./safe-frame";` (type-only, so no runtime import cycle). Add optional final parameter `frame?: ScreenRect` to `createCamera` (spec Component 4). When omitted, use `{left:0, top:0, right:viewport.width, bottom:viewport.height}`. Validate the frame: finite edges and `(right-left) - 2*VIEWPORT_INSET > 0` and `(bottom-top) - 2*VIEWPORT_INSET > 0`, else throw `new Error(INVALID_VIEWPORT_MESSAGE)`. Compute fittedScale from the frame's inner size; `offsetX = frame.left + (frameWidth - worldWidth*scale)/2 + view.panX`, `offsetY = frame.top + (frameHeight - worldHeight*scale)/2 + view.panY`. `camera.viewport` stays the full viewport. Output without a frame must be bit-identical to today. Export `projectedWorldRect(camera: Camera): ScreenRect`.
3. `web/src/render/canvas.ts`: `resizeCanvasBackingStore` gains optional final `frame?: ScreenRect` forwarded to `createCamera`. Nothing else changes.
4. `web/src/player/camera-frame.ts` (new): `export type CameraFrameInputs = Pick<FrameClock, "viewportWidth" | "viewportHeight" | "hudObstacles" | "worldBounds">` (use `import type { FrameClock } from "./frame-loop"`; add `hudObstacles: readonly ScreenRect[]` to FrameClock in frame-loop.ts now, initialised to `[]` in createFrameClock so the classic layout stays empty). `export function cameraFrame(clock: CameraFrameInputs): SafeFrame` calls `safeFrame({width, height}, clock.hudObstacles, aspectOf(clock.worldBounds))` where `aspectOf` = (maxX-minX)/(maxY-minY). Also `export function publishWorldFrame(canvas: HTMLCanvasElement, camera: Camera, usedFallback: boolean): void` that sets `canvas.dataset.worldFrame = "left,top,right,bottom"` from `projectedWorldRect(camera)` (round each to 2 decimals) and sets `canvas.dataset.worldFrameFallback = "true"` when usedFallback, otherwise deletes it. publishWorldFrame is DOM shell code; it is exercised by Playwright in Task 3, not unit tests.
5. `web/src/player/hud-sliders-preference.ts` (new), following `web/src/render/mode.ts`: `HUD_SLIDERS_STORAGE_KEY = "liquidfun.hud-sliders.v1"`, `type HudSlidersVisibility = "shown" | "hidden"`, `DEFAULT_HUD_SLIDERS_VISIBILITY = "shown"`, `HudSlidersStorageProvider = () => Pick<Storage, "getItem" | "setItem">`, `maybeParseHudSlidersVisibility(raw: string | null): HudSlidersVisibility | undefined`, `loadHudSlidersVisibility(provider)` and `persistHudSlidersVisibility(provider, visibility)` with every storage access (including calling the provider) inside try/catch; any failure or unknown value reads as "shown" (the only permitted swallowed errors, per spec Component 1). `hudSlidersToggleLabel(visibility)` returns "Hide sliders" / "Show sliders".

Run tests until green, then `bun run typecheck`. Commit `feat(quick-261010-vzz): add safe-frame core and chrome-aware camera frame`.
  </action>
  <verify>
    <automated>cd web && bun run test:unit -- tests/safe-frame.test.ts tests/camera.test.ts tests/camera-frame.test.ts tests/hud-sliders-preference.test.ts tests/portrait-bounds.test.ts tests/scale-legend.test.ts && bun run typecheck</automated>
  </verify>
  <done>All new tests pass after first failing in a committed RED step; all 14 pre-existing camera tests and portrait/scale-legend tests pass unchanged; typecheck is clean; four BEFORE screenshots exist under web/test-results/hud-safe-frame-visual/.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Obstacle measurement shell, slider toggle UI, and live camera wiring</name>
  <files>web/src/player/hud-obstacles.ts, web/src/player/frame-loop.ts, web/src/player/scene-surface.ts, web/src/player/scene-runtime.ts, web/src/components/CanvasStage.tsx, web/src/components/SceneControls.tsx, web/src/components/PlayerPanel.tsx, web/src/components/PlaygroundStage.tsx, web/src/App.tsx, web/src/styles/canvas-stage.css, web/tests/hud-sliders-toggle.test.ts</files>
  <behavior>
    hud-sliders-toggle.test.ts (source-read setup identical to the "playground scale legend chrome" block in tests/scale-legend.test.ts; vitest has no DOM):
    - CanvasStage.tsx gates the toggle on `props.hasHudSlider`, sets `aria-controls={HUD_SLIDERS_ID}` and `aria-expanded`, and renders it inside `.canvas-hud-toolbar` before `.canvas-transport`
    - CanvasStage.tsx wraps the slider card (props.children) in a `<Show>` keyed on the visibility signal (removed from the DOM, not visually hidden) and persists via `persistHudSlidersVisibility`
    - every spec'd chrome leaf carries `data-hud-obstacle`: title h2 and status output, `.canvas-hud-stats`, `.canvas-hud-corner`, `.canvas-transport`, `.canvas-hud-trailing`, the toggle (CanvasStage.tsx), and `.canvas-hud-slider` with `id={HUD_SLIDERS_ID}` (SceneControls.tsx); `.canvas-hud-title` and `.canvas-hud-bottom` containers do NOT carry it
    - PlayerPanel.tsx calls `createHudObstacles` only in the canvas-stage layout
  </behavior>
  <action>
RED: write tests/hud-sliders-toggle.test.ts from the behavior block (regex/includes over source text, one concern per test, AAA comments), confirm it fails, commit `test(quick-261010-vzz): add failing slider toggle chrome test`.

GREEN:
1. `web/src/player/hud-obstacles.ts` (new, imperative shell, spec Component 3): `export const HUD_OBSTACLE_SELECTOR = "[data-hud-obstacle]"`; `export function createHudObstacles(root: HTMLElement, canvas: HTMLElement): Accessor<readonly ScreenRect[]>`. Must be called inside a Solid owner. It creates `createSignal<readonly ScreenRect[]>([], { equals: sameScreenRects })`; a `measure()` that reads `canvas.getBoundingClientRect()`, queries `root.querySelectorAll(HUD_OBSTACLE_SELECTOR)`, skips elements whose rect has width or height <= 0, converts each with `screenRectInCanvas`, and sets the signal; coalesces requests through one `requestAnimationFrame` (keep `maybePendingFrameId`). One `ResizeObserver` observes the canvas and every marked element; a `MutationObserver` on `root` with `{ childList: true, subtree: true }` (not characterData, not attributes) re-syncs the ResizeObserver's observed set (disconnect, re-observe canvas + current marked elements) and schedules a measure. Schedule an initial measure. `onCleanup` disconnects both observers and cancels the pending frame. The equality guard is what stops FPS/debug text churn from refitting the camera.
2. `web/src/player/frame-loop.ts` (spec Component 4): ensure `FrameClock.hudObstacles: readonly ScreenRect[]` exists (default `[]`, so the classic layout stays empty). In `connectResizeObserver`, after updating `clock.worldBounds`, compute `const safe = cameraFrame(clock)` and pass `safe.frame` as the final argument of `resizeCanvasBackingStore`; then call `publishWorldFrame(canvas, resizedCamera, safe.usedFallback)`. Keep `worldBoundsForViewport` choosing portrait vs landscape from the canvas aspect (spec: the safe frame only places those bounds).
3. `web/src/player/scene-surface.ts`: in `refreshCamera`, compute `const safe = cameraFrame(session.clock)` and pass `safe.frame` as the final `createCamera` argument; if `session.maybeCanvas !== undefined`, call `publishWorldFrame(session.maybeCanvas, camera, safe.usedFallback)`. Add `function changeHudObstacles(session, obstacles: readonly ScreenRect[])` that sets `session.clock.hudObstacles = obstacles`, calls `refreshCamera(session)`, then `paintHeldFrame(session)` (repaints the held frame when paused; harmless while playing, matching applyCameraView). Bind it in `bindSurface` as `session.changeHudObstacles`. Zoom/pan/reset already route through refreshCamera, so user zoom multiplies the fitted scale, pan adds to the offset, and reset returns to the safe framing (spec). No easing (spec: refit is instant).
4. `web/src/player/scene-runtime.ts`: add `changeHudObstacles: (obstacles: readonly ScreenRect[]) => void;` to SceneRuntime.
5. `web/src/components/SceneControls.tsx`: `export const HUD_SLIDERS_ID = "canvas-hud-sliders"`; the `.canvas-hud-slider` div gets `id={HUD_SLIDERS_ID}` and `data-hud-obstacle`.
6. `web/src/components/CanvasStage.tsx` (spec Component 1): in CanvasHud, `const [sliders, setSliders] = createSignal(loadHudSlidersVisibility(() => window.localStorage))`. Wrap `{props.children}` in `<Show when={sliders() === "shown"}>`. Replace the toolbar's first child with `<div class="canvas-hud-leading">` containing `<Show when={props.hasHudSlider}><HudSlidersToggle .../></Show>` then the existing `.canvas-transport` (now with `data-hud-obstacle`). `HudSlidersToggle`: `<button class="hud-button canvas-sliders-toggle" type="button" data-hud-obstacle aria-label={hudSlidersToggleLabel(v)} aria-expanded={v === "shown"} aria-controls={HUD_SLIDERS_ID}>` with a slider glyph SVG (e.g. three horizontal tracks with knobs) plus a diagonal strike path rendered only when hidden; onClick flips, calls `setSliders` and `persistHudSlidersVisibility(() => window.localStorage, next)`. Add `data-hud-obstacle` to: the `h2#player-title`, the `output.session-status`, the `.canvas-hud-stats` div, `.canvas-hud-trailing`, `.canvas-hud-corner`. Do NOT add it to `.canvas-hud-title` or `.canvas-hud-bottom` (full-width containers). Change the stats class condition to `props.hasHudSlider && sliders() === "shown"`. Keep the file under ~450 lines.
7. `web/src/styles/canvas-stage.css`: add `.canvas-hud-leading { display: flex; align-items: flex-end; gap: 4px; }` so `space-between` still pins transport left and trailing right. Make obstacle boxes independent of camera scale and transient text, so a refit cannot feed back into the layout: give `.canvas-hud-trailing .scale-legend` a fixed `width: 100px` (SCALE_LEGEND_MAX_WIDTH_PX, the legend bar varies with pixelsPerMeter), and give `.canvas-hud-title .session-status` a `min-width` sized for "Playing"/"Paused" (tabular, e.g. `min-width: 5.5rem`) so pausing does not move the frame. Check `.canvas-fps` has a stable width (`font-variant-numeric: tabular-nums`, `min-width`); add one if FPS text changes the stats column width. Style `.canvas-sliders-toggle` like the other hud-buttons; no new colors.
8. `web/src/components/PlayerPanel.tsx`: add prop `readonly onHudObstaclesChange: (obstacles: readonly ScreenRect[]) => void`. In `PlayerPanelLayout`, keep `let maybeViewportFrame: HTMLDivElement | undefined` and `let maybeSceneCanvas: HTMLCanvasElement | undefined` (set from refs; the canvas ref still calls `props.assignCanvas`). In `onMount`, return early unless `layoutProps.canvasStage` and both refs exist; otherwise `const obstacles = createHudObstacles(maybeViewportFrame, maybeSceneCanvas)` and `createEffect(on(obstacles, (rects) => props.onHudObstaclesChange(rects)))`.
9. `web/src/components/PlaygroundStage.tsx` and `web/src/App.tsx`: thread `onHudObstaclesChange` through, bound to `session.changeHudObstacles`.

Run the new test and the full unit suite, typecheck, and `bun scripts/bright-builds-check.ts all` from the repo root (file-length triggers ~628 lines/file, ~161 lines/function). Commit `feat(quick-261010-vzz): add HUD slider toggle and chrome-aware camera fit`.
  </action>
  <verify>
    <automated>cd web && bun run test:unit && bun run typecheck && cd .. && bun scripts/bright-builds-check.ts all</automated>
  </verify>
  <done>The source-read chrome test passes after a committed RED step; the full unit suite, typecheck, and bright-builds checks pass; both live camera call sites (frame-loop ResizeObserver and scene-surface refreshCamera) use cameraFrame and publish data-world-frame; obstacle changes refit and repaint; exports and the classic layout still call createCamera without a frame.</done>
</task>

<task type="auto">
  <name>Task 3: Tesla valve cleanup, framing guidance, catalog-driven browser spec, and done-criteria evidence</name>
  <files>web/src/catalog/portrait-bounds.ts, web/tests/portrait-bounds.test.ts, AGENTS.md, web/e2e/hud-safe-frame.spec.ts</files>
  <action>
1. Cleanup (spec): in `web/src/catalog/portrait-bounds.ts`, remove the built-in empty space below the drain from `TESLA_VALVE_PORTRAIT`: keep minX/maxX/maxY, set `minY` to a small margin below the lowest wall endpoint (walls reach y = 0.017; use about -0.2). Rewrite its doc comment: the frame fits the valve tightly; keeping it clear of the sliders and transport is automatic through the chrome-aware camera fit. Run `bun run test:unit -- tests/portrait-bounds.test.ts`. The wall check must pass unchanged. If the fill check fails only because tesla-valve's `MIN_HEIGHT_FRACTION` of 0.80 encoded the removed empty space, lower that single entry to just below the new measured fraction (round down to 0.05) and add a one-line comment saying why; width must stay > 0.85. Do not touch any other scene's frame or threshold.
2. Guidance (spec Cleanup): edit the "### Playground canvas framing" section in `AGENTS.md` (outside the managed block; CLAUDE.md is a symlink to it). Portrait frames still fit the subject tightly (keep the existing sizing, walls, recognizability, and traveling-subject guidance), but replace "and keep the subject clear of the title and the transport controls" with: keeping the subject clear of HUD chrome is automatic, because the default canvas stage fits the world around every element marked `data-hud-obstacle` (`web/src/render/safe-frame.ts`), so do not pad portrait frames for chrome. Add: new on-canvas chrome must carry `data-hud-obstacle` on its leaf element (not on a full-width container) and must keep a box size independent of camera scale; `web/e2e/hud-safe-frame.spec.ts` checks every catalog scene. Use repeated `1.` markers if you add an ordered list. Then run `mdformat --version` (expect 1.0.0 under Python 3.13), `mdformat AGENTS.md`, and `just markdown-check` from the repo root. Never run mdformat on `.planning/**`.
3. Browser spec `web/e2e/hud-safe-frame.spec.ts` (spec Testing, Playwright). Import `SCENES`, `SCENE_IDS` from `../src/catalog/scenes`, `sceneControlsForSurface` from `../src/components/scene-controls`, `HUD_SLIDERS_STORAGE_KEY` from `../src/player/hud-sliders-preference`, and `SCENE_HASH_PATHS`, `sessionStatus`, `PLAYING_STATUS` from `./player-helpers`. Use `test.describe.configure({ mode: "parallel" })`. Generate tests from the catalog so new scenes are covered automatically: every scene at 390x844 and 1280x800; scenes whose `sceneControlsForSurface(scene.controls, "hud").length > 0` also at 844x390 and 768x1024. Per case: set the viewport, goto the scene path, wait for `PLAYING_STATUS`, then inside `expect(async () => {...}).toPass({ timeout: 5000 })` read `canvas.scene-canvas` `data-world-frame` (canvas coordinates) plus the canvas bounding box, convert to page coordinates, collect bounding boxes of every visible `[data-hud-obstacle]` (skip null boxes), and assert no overlap larger than 1px in both axes (helper `overlapPx(a, b)`; report the offending selector/class in the message). If `data-world-frame-fallback="true"`: push `test.info().annotations.push({ type: "safe-frame-fallback", description: `${id} ${w}x${h}` })`, click the "Hide sliders" button, and assert the overlap clears and the fallback attribute is gone (a fallback on a scene without HUD controls is a real failure; investigate the chrome rather than weakening the check). Add a toggle round-trip test on wave-tank at 390x844: record frame area A with sliders shown and `aria-expanded="true"`; click "Hide sliders"; assert `#canvas-hud-sliders` has count 0, the button is now "Show sliders" with `aria-expanded="false"`, `localStorage[HUD_SLIDERS_STORAGE_KEY] === "hidden"`, and frame area B > A; reload and assert the hidden state and frame B survive (within 1px); click "Show sliders" and assert the frame returns to A (within 1px). Add a test that a scene without HUD controls (pick the first catalog scene with none) renders no "Hide sliders"/"Show sliders" button. Add a regression guard: on wave-tank at 1280x800 and 390x844, pausing then resuming does not change `data-world-frame` (within 1px). Clear localStorage in a fresh context per test (Playwright's default per-test context is fresh).
4. Done criteria (spec), from `web/`: `bun run build:app`; `bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts e2e/shell.spec.ts e2e/player.spec.ts` (run once, not in parallel with other browser launches; investigate and fix any failure rather than retrying blindly, preserving the failure output). Then capture AFTER screenshots exactly like the Task 1 baseline (`after-<id>-<W>x<H>.png` for wave-tank and tesla-valve at 390x844 and 1280x800 into `web/test-results/hud-safe-frame-visual/`). View each before/after pair with the Read tool and record in SUMMARY what changed (scene clear of the slider card and transport; wide viewport placing the scene beside the card). Finally run `bun run test:unit`, `bun run typecheck`, `bun scripts/bright-builds-check.ts all` (repo root), and `just markdown-check` (repo root).
5. Commit `feat(quick-261010-vzz): tighten Tesla valve portrait frame and cover HUD safe frame in the browser`. Write `.planning/quick/261010-vzz-hud-safe-frame-slider-toggle-and-chrome-/261010-vzz-SUMMARY.md` with frontmatter `generated_by: gsd-executor`, `lifecycle_mode: quick`, `phase_lifecycle_id: quick-261010-vzz`, and `generated_at` from `date -u +%Y-%m-%dT%H:%M:%SZ` (real current time). The SUMMARY lists every fallback annotation from the spec run, the screenshot paths and observations, the absence of a web lint script, and any MIN_HEIGHT_FRACTION change.
  </action>
  <verify>
    <automated>cd web && bun run test:unit && bun run typecheck && bun run build:app && bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts e2e/shell.spec.ts e2e/player.spec.ts && cd .. && bun scripts/bright-builds-check.ts all && just markdown-check</automated>
  </verify>
  <done>Tesla valve portrait frame has no empty space below the drain and portrait-bounds tests pass; AGENTS.md framing guidance describes the automatic chrome-aware fit and the data-hud-obstacle rule and passes markdown-check; the catalog-driven Playwright spec and the existing canvas-stage, shell, and player specs pass; before/after screenshots for Wave tank and Tesla valve at 390x844 and 1280x800 are reviewed and described in SUMMARY.md with the required lifecycle frontmatter.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| localStorage -> app | `liquidfun.hud-sliders.v1` is user/browser controlled and may be absent, malformed, or throw on access (private mode, quota) |
| DOM layout -> camera | Obstacle rects come from live layout; degenerate or huge rects must not break the camera |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-vzz-01 | Tampering | hud-sliders-preference.ts | mitigate | Parse only "shown"/"hidden"; every other value or any storage exception reads as "shown" (try/catch around provider, getItem, setItem) |
| T-vzz-02 | Denial of service | safe-frame.ts | mitigate | Obstacles are clipped to the viewport and zero-area ones dropped; candidate search is O(edges^4 x obstacles) with ~8 obstacles (~20k candidates) and runs only on layout changes, never per frame; equality guard in hud-obstacles.ts suppresses churn from text updates |
| T-vzz-03 | Denial of service | camera.ts | mitigate | createCamera validates the frame (finite, room for the inset) and throws the existing "Invalid Canvas viewport" error; safeFrame falls back to the full viewport rather than returning a degenerate frame |
| T-vzz-04 | Information disclosure | data-world-frame attribute | accept | Exposes only projected canvas geometry; no user data |
</threat_model>

<verification>
- Decision coverage: D1 camera fits around measured chrome (Task 1 core, Task 2 wiring); D2 best fit among obstacle rectangles (Task 1 safeFrame + wide-card test, Task 3 e2e at 1280x800); D3 sliders shown by default, one persisted global preference (Task 1 preference, Task 2 toggle, Task 3 reload round trip); D4 toggle at the leading edge of the bottom toolbar (Task 2 leading group, source test); D5 below-50% fallback accepts overlap (Task 1 fallback tests, Task 3 fallback branch).
- Components: C1 toggle (Task 2), C2 safe frame core (Task 1), C3 obstacle shell (Task 2), C4 camera integration incl. data-world-frame hooks and refresh on obstacle change (Tasks 1-2); Cleanup and Testing (Task 3); Done criteria (Task 3 step 4).
- `git grep -n "createCamera(" web/src` shows exports (`web/src/export/record.ts`) still call it without a frame.
</verification>

<success_criteria>
- `bun run test:unit`, `bun run typecheck`, and `bun run build:app` pass in `web/`.
- `e2e/hud-safe-frame.spec.ts`, `e2e/canvas-stage.spec.ts`, `e2e/shell.spec.ts`, and `e2e/player.spec.ts` pass.
- `bun scripts/bright-builds-check.ts all` and `just markdown-check` pass at the repo root.
- Before/after screenshots for Wave tank and Tesla valve at 390x844 and 1280x800 are reviewed and described in SUMMARY.md.
</success_criteria>

<output>
After completion, create `.planning/quick/261010-vzz-hud-safe-frame-slider-toggle-and-chrome-/261010-vzz-SUMMARY.md` with frontmatter `generated_by: gsd-executor`, `lifecycle_mode: quick`, `phase_lifecycle_id: quick-261010-vzz`, and `generated_at` set from `date -u +%Y-%m-%dT%H:%M:%SZ`.
</output>

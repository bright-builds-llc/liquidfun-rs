# Phase 43: GPU Water Render Mode - Research

**Researched:** 2026-10-13
**Domain:** WebGL2 particle-surface rendering (MRT field pass, refraction shade pass), optional Rust/wasm frame lane, render-mode wiring and fallback in a SolidJS playground
**Confidence:** HIGH for code integration points (read from source at `cd401e883`); HIGH for WebGL2 API facts (Khronos/MDN plus a live headless-Chromium probe); MEDIUM for iOS float-target behavior (device-only, D-20 UAT); MEDIUM for look constants (tuned by eye)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Mode wiring and picker

- **D-01:** Add `"water"` to `RenderMode`, `maybeParseRenderMode` and `RENDER_MODE_GROUPS` in `web/src/render/mode.ts`, labeled "Water", as the last option of the existing "Surface" group (no new group). The stored preference uses the existing `liquidfun.render-mode.v1` key.
- **D-02:** `DEFAULT_RENDER_MODE` stays `"shaded-blob"`. Water is opt-in.
- **D-03:** Water appears in both pickers: the scene "Particles" select (`PlayerPanel.tsx`) and `PuzzleRenderModeSelect` (`PuzzleOptionsDrawer.tsx`, both the control row and the options sheet). No new HUD control.
- **D-04:** `particleSurface("water")` maps to a new `"webgl-water"` surface. `needsWebglSurface` returns true for both WebGL surfaces, so the existing `<canvas class="particle-surface">` is reused. Do not add a second WebGL canvas.
- **D-05:** When a capability probe shows the device cannot run Water, the Water option stays visible but disabled, with the accessible name "Water (unavailable)", following the Phase 42 D-05 pattern. A stored `"water"` preference is still parsed and kept; at draw time it falls back (D-12), so the preference returns on a capable device.

#### Look

- **D-06:** The shader does a field pass shared with shaded blob, then a shade pass with:
  - refraction: offset sampling of a background texture by the field gradient;
  - Blinn-Phong specular highlights from a fixed light direction;
  - Beer-Lambert thickness tint that keeps each particle's own color, so colored and multi-material scenes still read;
  - foam: blend toward white where the per-particle speed is high, using a smoothstep ramp between two world-speed constants (m/s).
- **D-07:** The refraction background is the cached static layer from `static-layer.ts` (walls and static geometry), uploaded as a texture only when that layer changes. Never upload a texture every frame. If the static layer is unavailable, use a plain procedural backdrop.
- **D-08:** Water shaders use `precision highp float`. Shaded blob's existing `mediump` shaders are left unchanged.
- **D-09:** Cap the field resolution with `cappedDevicePixelRatio`, matching shaded blob.

#### Speed lane (foam data)

- **D-10:** Add an optional per-particle speed lane: `particleSpeeds: Float32Array`, one float per particle, through `FrameData`, `RawProofFrame`, `RenderFrame` and `frameTransferables`. Rust computes it in `capture_frame` from the `view.velocities()` call that already happens. Do not derive speed in TypeScript from position deltas.
- **D-11:** The lane is filled only when a capture option is on. The worker turns the option on only while the active render mode is Water, and off when the mode changes. When the option is off, the frame omits the lane, so frames in other modes are the same as today. The lane never feeds physics. Gates:
  - `just playground-scene-spot` gives 25/25 fingerprints identical to the phase-base HEAD;
  - puzzle replay outcomes (RunFingerprint) are identical with Water on and off, using the Phase 42 D-05 on/off replay pattern.

#### Fallback and context loss

- **D-12:** Use one fallback chain per frame: `covered = (water && drawWater(...)) || (blobLike && drawShadedBlob(...))`, then the canvas path (`paintMetaball` soft blob). Never present a blank frame.
- **D-13:** Float render targets accept `EXT_color_buffer_float` **or** `EXT_color_buffer_half_float`, use RGBA16F, and trust `checkFramebufferStatus`. This applies to the shared field pass, so shaded blob also gains half-float support. This folds in the pending STATE.md todo.
- **D-14:** When the context is lost, fall back right away. When it is restored, rebuild the GPU resources and repaint the held frame through the existing `paintHeldFrame`, so a paused view does not stay stale or blank.
- **D-15:** Expose the surface that actually drew as a DOM attribute on the stage, for example `data-particle-surface="webgl-water" | "webgl" | "canvas"`, so tests can check fallback without pixel diffs.

#### Overlays and exports

- **D-16:** Cup, fill meter, target line, tools, labels, walls and scale legend stay on the 2D canvas above the particle surface in Water mode, as with shaded blob (38-CONTEXT).
- **D-17:** `svgRenderMode("water")` maps to `"solid"`, like the other surface modes. README SVG/WebP plans stay pinned to `"wireframe"`, and demo media stays on circle-wireframe. Unit tests confirm that the export mapping for water equals the mapping for shaded blob.

#### Verification

- **D-18:** Unit tests (Vitest with the mock GL from `web/tests/webgl-render-fixture.ts`) cover:
  - mode parsing, the group entry and the default;
  - the capability probe;
  - half-float acceptance;
  - the fallback chain when extensions are missing or the framebuffer is incomplete;
  - context loss and restore, where restore repaints;
  - speed-lane parsing, transfer and omission when off.
- **D-19:** Playwright (Chromium/SwiftShader) checks:
  - that Water engages in a scene and a puzzle;
  - that overlays are drawn;
  - that a forced `WEBGL_lose_context.loseContext()` falls back and `restoreContext()` recovers, both through D-15;
  - that the SVG export is unchanged.

  Do not screenshot-diff shader output.
- **D-20:** Viewing the effect on a real iPhone is a human UAT item. It does not block completion (PROJECT-SCOPE hobby profile).

### Claude's Discretion

- Exact constants: the foam speed thresholds, refraction strength, highlight exponent and light direction, and tint absorption. Tune them by eye on dam-break, fountain and a puzzle level.
- File split, for example `field-pass.ts` plus `webgl-water.ts` from the research. Keep each file under about 400 lines.
- How the capture option is sent to the worker (a message field or a session flag).
- Whether the capability probe result is cached per canvas or per page.

### Folded Todos

- **Half-float render targets** (STATE.md Pending Todos): accept `EXT_color_buffer_half_float` alongside `EXT_color_buffer_float` in `webgl-particles.ts`, and rely on `checkFramebufferStatus`. This is folded as D-13. Verifying it on an iPhone stays a non-blocking UAT item (D-20).

### Deferred Ideas (OUT OF SCOPE)

- Water-styled README/WebP exports, or a GPU export path. Exports stay pinned for now.
- Making Water the default mode after it has been verified on phones.
- Reflections, caustics and a dynamic-body refraction background, beyond the static layer.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WATER-01 | Player can choose a "Water" render mode with refraction, highlights and foam on fast-moving water. | Mode wiring (Pattern 5), shared field pass + MRT (Patterns 1-2), water shade shader (Code Examples), refraction backdrop (Pattern 3), speed lane end to end (Pattern 4), foam constants caveat (Pitfall 9). |
| WATER-02 | When the device cannot run it or the WebGL context is lost, the view falls back to shaded blob and then canvas, and recovers when the context is restored. | Capability probe (Pattern 6), fallback chain and drawn-surface attribute (Pattern 7), context loss/restore ordering bug (Pitfall 1), half-float acceptance (D-13, Pitfall 4), CSS transparency selector (Pitfall 2). |
| WATER-03 | Water mode works in scenes and puzzles, and exports keep their existing styles. | Both draw paths go through `scene-surface.ts:58` `drawSceneFrame`; overlays drawn after particles (`scene-surface.ts:77-83`); `svgRenderMode` fallthrough already yields `"solid"` (`mode.ts:155-161`); export pins in `plans.ts:126` and `demo-media/capture.ts:277`; RunFingerprint on/off e2e (Validation Architecture). |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Production physics stays native Rust; the speed lane is copied read-only data and must not change stepping. `crates/liquidfun` must stay untouched (prior phase gate `engine-untouched`).
- Safe Rust only (`#![forbid(unsafe_code)]` in `crates/liquidfun-wasm/src/lib.rs:3`), no `unwrap()`, propagate errors, `maybe_` naming for optionals; TS optionals use the `maybe` prefix (`maybeParticleSpeeds`, `maybeBackdrop`).
- Angles use tau (`Math.PI * 2` as `TAU`, as in `canvas.ts:56`).
- Determinism first: no nondeterministic acceleration in physics; rendering may be GPU.
- File-length check: hard trigger at 628 physical lines (`scripts/bright-builds-check.ts:8`); CONTEXT asks for about 400 lines per new file. Current sizes that matter: `session.rs` 602, `frame.rs` 551, `webgl-particles.ts` 478, `PlayerPanel.tsx` 493, `PuzzlePanel.tsx` 457, `scene-surface.ts` 455, `canvas.ts` 399.
- Bright Builds: do not hide substantial foreign-language logic inside strings -> put the water GLSL in `.glsl` files imported with `?raw` (verified working in Vitest, see Code Examples). Functional core / imperative shell: keep the capability decision, fallback selection and lane validation as pure functions with unit tests.
- Unit tests: Arrange/Act/Assert comments, one concern per test.
- No new npm or Cargo dependencies (CONTEXT scope; Phase 42 `no-new-dependencies` gate pattern).
- Pre-commit (Rust): `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. Web: `bun run typecheck`, `bun run test:unit`, `bun run build:app`, relevant Playwright. `bun scripts/bright-builds-check.ts all`. `just markdown-check` only if non-GSD Markdown changes; never mdformat `.planning/**`.
- Playground rules in AGENTS.md (README scene gallery, portrait framing, scale legend, `data-hud-obstacle`) are not triggered: no new scene, no new on-canvas chrome (D-03).
- GSD execution: run executors sequentially on the main tree (gate baselines in gitignored `target/phase43/`), stage files explicitly, never stage `.planning/config.json`, SUMMARY frontmatter must carry `generated_by`, `lifecycle_mode`, `phase_lifecycle_id: 43-2026-10-13T22-55-49`, `generated_at`.
- macOS first-launch stalls (~2 min per new test binary): run cargo test invocations sequentially with generous timeouts.

## Summary

Everything this phase needs already exists in shape, but four facts in the current code change how it should be planned:

1. **`static-layer.ts` is not a raster.** It caches `Path2D` objects per 2D context (`static-layer.ts:29-35`), keyed by camera/mode/stroke/backing size plus exact segment content (`static-layer.ts:45-71, 128-133`), and it bypasses the cache entirely when a frame has fewer than 128 segment floats (32 segments) or no `Path2D` (`static-layer.ts:113-121`). There is nothing to pass to `texImage2D`. D-07's intent (static walls, uploaded only on change, procedural fallback) is met by a small water-owned backdrop raster that reuses static-layer's "same view + same content" stability rule. See Pattern 3 and Open Question 1.
2. **Most sessions are not in the worker.** The worker backend is used only for Tesla Valve live playback (`scene-lifecycle.ts:141-153`, `live-session.ts:14`); every other scene uses a direct main-thread `SceneSession` (`session.ts:48`) and every puzzle uses a direct `PuzzleSession` (`puzzle-host.ts:107`). The "capture option" must therefore reach three owners, not one worker. The simplest robust design is a stateless per-capture flag: Rust `captureFrame(particleSpeeds?: boolean)`, with each TS owner reading a `() => boolean` accessor at capture time and the worker receiving `speeds: boolean` on its `advance`/`capture` requests. Export and replay paths keep calling `captureFrame()` and never get the lane.
3. **Context-restore repaint has a listener-order trap.** The WebGL surface registers its `webglcontextrestored` reset listener lazily on first draw (`webgl-particles.ts:328-332`). A repaint listener added earlier (for example in `assignParticleSurface`, `scene-surface.ts:429-431`) would fire first, see `maybeResources` defined and `isContextLost() === false`, and draw with dead GL objects, so a paused view stays blank. The GL module must own the order: reset first, then notify a repaint callback. The same applies to context loss (paused views need a repaint to fall back).
4. **CSS hides any non-`"webgl"` surface.** `player-viewport.css:51-53` makes the 2D scene canvas transparent only for `[data-particle-surface="webgl"]`. With `"webgl-water"` the 2D canvas keeps its opaque `#071018` background and the water is invisible under it. The selector must cover both WebGL surfaces.

The WebGL work itself is standard WebGL2: keep the existing additive point-splat into RGBA16F, add a second RGBA16F color attachment via `drawBuffers` for weighted foam, and write a `highp` full-screen shade pass (field-gradient normal, refraction offset into a backdrop texture, Blinn-Phong, Beer-Lambert, foam blend). Headless Chromium here exposes WebGL2 with `EXT_color_buffer_float`, `EXT_color_buffer_half_float`, `EXT_float_blend`, `WEBGL_lose_context`, highp precision 23 and 8 draw buffers (probed this session), so Playwright can exercise water, forced fallback (by patching prototypes in an init script) and forced context loss.

**Primary recommendation:** Split `webgl-particles.ts` into a per-canvas `field-pass.ts` (context, loss/restore with ordered repaint hook, shared buffers, RGBA16F target with float-or-half acceptance) used by both `webgl-particles.ts` (shaded-blob composite, unchanged look) and a new `webgl-water.ts` (lazy MRT target, water programs, backdrop texture); add the speed lane as a stateless per-capture Rust flag; make `presentSceneFrame` return the surface that drew and write it to a new canvas data attribute.

## Standard Stack

### Core (all existing, no installs)

| Library / API | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| WebGL2 (`#version 300 es`) | browser core | Field pass, MRT, shade pass | Already used by `webgl-particles.ts`; MRT (`drawBuffers`), `HALF_FLOAT` textures and `highp` fragment precision are WebGL2 core. [VERIFIED: headless Chromium probe this session: `MAX_DRAW_BUFFERS=8`, highp precision 23] |
| `EXT_color_buffer_float` / `EXT_color_buffer_half_float` | WebGL extensions | Make RGBA16F color-renderable | Half-float ext in WebGL2 makes R16F, RG16F, RGBA16F renderable; only RGBA16F is required to be supported. [CITED: registry.khronos.org/webgl/extensions/EXT_color_buffer_half_float/] |
| `WEBGL_lose_context` | WebGL extension | Forced loss/restore in tests; releasing the probe context | `restoreContext()` throws `INVALID_OPERATION` if the context is not lost; the context is unusable until `webglcontextrestored` fires. [CITED: developer.mozilla.org/en-US/docs/Web/API/WEBGL_lose_context/restoreContext] |
| `wasm-bindgen` | `=0.2.128` (`crates/liquidfun-wasm/Cargo.toml:41`) | Lane getter and capture flag | `Option<Box<[f32]>>` return -> `Float32Array \| undefined`; `Option<bool>` parameter supported. [CITED: wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html, .../types/bool.html] |
| Vite `?raw` imports | vite 8.3.0, vitest 5.0.1 | GLSL source files | `tsconfig.json` already has `"types": ["vite/client"]`. [VERIFIED: a throwaway `probe.glsl?raw` Vitest import passed this session, then removed] |
| SolidJS | 1.9.15 | Picker option disabled state | Existing component stack. |

### Supporting (test)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | 5.0.1 (`environment: "node"`) | Mock-GL unit tests | All D-18 items. No DOM: inject canvas factories. |
| `@playwright/test` | 1.63.0 (Chromium, SwiftShader "WebKit WebGL" renderer) | D-19 browser checks | Assert DOM attributes, never shader pixels. |
| `wasm-pack` | 0.15.0 (pinned in `scripts/web-build.ts:17`) | `just web-wasm` regenerates `web/src/generated/liquidfun-wasm/` | Required after every Rust bridge change before TS typecheck sees new getters. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| MRT foam attachment | Second splat pass into a separate foam FBO | Simpler shader, but doubles vertex work on phones. MRT is WebGL2 core; use it. |
| Per-capture flag (stateless) | Stateful `setParticleSpeedCapture(enabled)` on `ProofSession` | Stateful needs a sync call at every owner creation and mode change; missing one silently drops foam or leaks the lane. Stateless cannot drift. |
| `.glsl?raw` files | Inline template strings (as `webgl-particles.ts:8-69` does) | Inline is fine for 15-line shaders; the water shade shader is substantial, so Bright Builds asks for a language-aware file. Keep shaded blob's inline shaders unchanged (D-08). |
| three.js / twgl / pixi | none | Ruled out by milestone research and "no new dependencies". |

**Installation:** none. `cd web && bun install --frozen-lockfile` is unchanged.

## Current Code Facts (verified file:line)

| Topic | Fact |
|---|---|
| Shaded-blob pipeline | `webgl-particles.ts`: point VS/FS (`8-40`, `mediump`), composite (`42-69`), `createResources` gates on `EXT_color_buffer_float` only (`152-154`), RGBA16F/`HALF_FLOAT` field (`248-258`), FBO completeness (`267`), uploads via `fillProjectedParticleArrays` with metadata cache (`270-312`), per-canvas closure with `shadersAvailable` flag and loss/restore reset (`314-332`), context attrs `alpha:false, antialias:false, depth:false, stencil:false, premultipliedAlpha:false` (`342-348`), per-canvas `WeakMap` (`446`), public `drawShadedBlob(canvas, frame, camera, dpr, densityShading)` (`455-478`). |
| Uniforms | `blob-uniforms.ts:9-44`: 8 `getUniformLocation` calls; tests assert exactly 8 per canvas (`webgl-render-cache.test.ts:105,170-171`) and 2 `createProgram` calls (`:168-169`). Water resources must be created lazily so these counts hold for shaded blob. |
| Fallback seam | `present-frame.ts:19-39`: `webglCovered = needsWebglSurface(mode) && drawShadedBlob(...)`, then `drawRenderFrame(..., webglCovered, ...)`. `canvas.ts:292-304` clears transparent when covered, fills `#071018` otherwise. `canvas.ts:351` skips the metaball only for `renderMode === "shaded-blob" && webglCovered`; every other non-disc/triangle/contour mode falls to `paintMetaball` (`canvas.ts:355`) -> water's canvas fallback is already the soft blob once line 351 tests `webglCovered` alone. |
| DPR cap | `canvas.ts:37-42` `cappedDevicePixelRatio` (max 2, throws on non-positive/non-finite). |
| Mode module | `mode.ts`: union `2-8`, `ParticleSurface` `22`, default `35`, groups `39-59`, parse allowlist `65-84`, `svgRenderMode` falls through to `rigidRenderMode` -> `"solid"` for any non-wireframe (`141-161`), `usesParticleStride` (`178-181`), exhaustive `particleSurface` switch (`184-198`), `needsWebglSurface` (`201-203`). |
| Picker markup | Duplicated optgroup loops: `PlayerPanel.tsx:379-397` (`PlayerOptions`, rendered at `319` and `337`) and `PuzzleOptionsDrawer.tsx:55-81` (rendered at `PuzzlePanel.tsx:390` and `400`). |
| Canvas markup | Scene canvas `data-particle-surface={particleSurface(mode)}` at `PlayerPanel.tsx:196` / `PuzzlePanel.tsx:320`; particle canvas `data-active` at `PlayerPanel.tsx:211` / `PuzzlePanel.tsx:337`. CSS: `player-viewport.css:51-53` (transparent only for `"webgl"`), `:54-65`. |
| Draw path | All scene and puzzle draws go through `scene-surface.ts:58-84` `drawSceneFrame` (puzzle overlays drawn after particles at `77-83`). `paintHeldFrame` at `scene-surface.ts:43-56` uses `clock.maybePreviousFrame` + `clock.maybeCamera`. Particle canvas assigned at `scene-surface.ts:429-431`. Appearance changes call `paintHeldFrame` (`playground-session.ts:179-184`). |
| Render-mode signal | `player/appearance.ts:29-61` (`renderMode` accessor, `changeRenderMode` persists then repaints). `main[data-render-mode]` at `PlaygroundStage.tsx:156`. |
| Owners | Direct scene: `createSceneSession(await loadSceneSession(id))` (`scene-lifecycle.ts:148`); capture in `session.ts:79`. Puzzle: `createPuzzleSession(generated, id)` (`puzzle-host.ts:107`); capture in `puzzle-session.ts:295-302`. Worker (Tesla only): `createWorkerSession` (`scene-lifecycle.ts:149`), worker capture at `simulation-worker.ts:40`, frame post with `frameTransferables` at `:49-64`. |
| Worker protocol | `worker-messages.ts`: request union `9-28`, request parser `98-153`, `validateTransferredFrame` rebuilds a `RawProofFrame` from plain fields and forwards no optional getters (`156-175`), `frameTransferables` lists five lanes (`232-254`). |
| Frame contract (TS) | `frame.ts`: `RawProofFrame` optional getters pattern (`28-46`), `RenderFrame` (`85-101`), fail-closed parse with optional spreads (`254-271`); optional-lane parser precedent `frame-sound.ts:25-58`. `tsconfig` has `exactOptionalPropertyTypes: true` -> omit absent fields, never assign `undefined`. |
| Frame contract (Rust) | `frame.rs`: `FrameData` fields `48-59`, optional setters `126-149`, getters `226-311`, `mod puzzle;` submodule precedent (`frame/puzzle.rs` adds `#[wasm_bindgen] impl ProofFrame` getters returning `Option<_>`). |
| Rust capture | `session.rs:388-487`: velocities read at `425-429` (`max_particle_speed`, `97-115`), `annotate_frame` `492-516`. `SessionCore` is rebuilt wholesale on recreate controls (`*self = Self::from_source(...)`, `session.rs:346,357`) -> no flag may live on `SessionCore`. 37 Rust call sites use `capture_frame()`; keep that signature. `ProofSession::capture_frame` binding at `lib.rs:169-175`. |
| Fingerprints | Scene-spot and puzzle held hash both hash positions, segments and circles only (`scene_spot/frame_hash.rs:24-30`, `puzzle-run.ts:102-109`). The speed lane cannot affect either hash. |
| Particle order | `fillProjectedParticleArrays` with infinite cap visits every particle in index order with stride 1 (`projected-particle.ts:31-36, 56-121`), so speed index i aligns with uploaded vertex i. |
| Particle speed cap | Engine limits particle speed to `particle_diameter * inverse_time_step` (`crates/liquidfun/src/particle/solver/pressure.rs:233-235`); scene radii range from 0.005 m to 0.25 m; puzzle frames' `max_speed` saturates at 9.6 m/s (`target/phase42/sound-probe.txt`, p50 8.85, max 9.6). |
| Mock GL | `webgl-render-fixture.ts`: constants list `26-57`, `getExtension` returns `{}` for every name (`68`), `checkFramebufferStatus` always complete (`71`), Proxy auto-creates unknown methods (`81-86`), `lose()`/`restore()` dispatch events (`98-107`). Missing for water: `COLOR_ATTACHMENT1`, `MAX_DRAW_BUFFERS`, `UNSIGNED_BYTE`, `UNPACK_FLIP_Y_WEBGL`, `TEXTURE1`, `TEXTURE2`, `HIGH_FLOAT`, per-extension control, per-call FBO status. |
| Export pins | `readme-svg/plans.ts:126` `renderMode: "wireframe"`; `demo-media/capture.ts:270-290` pins `"circle-wireframe"`. `readme-svg/paint.ts:7-8` imports `render/canvas` and `render/mode` under **Bun** -> those modules must never import a `?raw` GLSL module. |
| Phase 42 gate tooling | `target/phase42/checks-05-postfix.sh` (32 `run` gates, `no_match` helper), baseline recipe in `42-01-PLAN.md:130-133`, sound on/off e2e `puzzle-sound.spec.ts:117-141, 304` using `readRunFingerprint` (`puzzle-editor-helpers.ts:171-186`). Latest 25-line fingerprint file: `target/phase42/after-postfix.txt` (identical to `base.txt`). Last Rust commit: `a47a73c03`. |

## Architecture Patterns

### Recommended file structure

```
web/src/render/
├── gl-program.ts          # compileShader / linkProgram / requireBuffer moved out of webgl-particles.ts (~50 lines)
├── field-pass.ts          # per-canvas GL surface: context, loss/restore + ordered notify, float-or-half check,
│                          #   shared position/radius/color buffers + metadata cache, RGBA16F target helpers (~300)
├── webgl-particles.ts     # shaded-blob point+composite programs and drawShadedBlob (same signature) (~200)
├── webgl-water.ts         # lazy water resources: MRT target, water point + shade programs, speed buffer,
│                          #   backdrop texture upload; drawWater(...) (~300)
├── water-shaders/         # water-point.vert.glsl, water-point.frag.glsl, water-shade.frag.glsl (highp)
├── water-backdrop.ts      # stable-walls raster cache with a revision counter; procedural flag when absent (~150)
├── water-capability.ts    # pure decision + per-page cached probe on a detached canvas (~100)
├── water-uniforms.ts      # uniform locations and tuned constants (FOAM_SPEED_LOW/HIGH, etc.) (~80)
├── present-frame.ts       # fallback chain; returns DrawnSurface
web/src/components/
└── RenderModeOptions.tsx  # shared optgroups with the disabled "Water (unavailable)" option for both pickers
crates/liquidfun-wasm/src/
├── frame/speeds.rs        # FrameData::set_particle_speeds + #[wasm_bindgen] particleSpeeds getter + tests
└── session/speeds.rs      # max_particle_speed (moved) + particle_speeds(velocities) so session.rs does not grow
```

Keep `canvas.ts`, `mode.ts` and `static-layer.ts` free of imports from any WebGL or `?raw` module (Bun imports them for README export).

### Pattern 1: One per-canvas GL surface shared by both WebGL modes (D-04, D-06, D-13)

**What:** `field-pass.ts` owns the single `webgl2` context of the particle canvas and everything both modes share. Shaded blob and water each own a lazily created "pass" (programs, VAO, render target) stored on that surface, each with its own availability flag so a water failure never disables shaded blob.

**Shape:**
- `particleSurfaceFor(canvas): ParticleGlSurface` — WeakMap lookup/creation; registers `webglcontextlost` (preventDefault + reset) and `webglcontextrestored` (reset) **at creation**.
- `watchParticleSurface(canvas, repaint: () => void): () => void` — ensures the surface exists (so its reset listeners are registered first), then adds lost/restored listeners that call `repaint` after the reset. Returns a detach function. Call it from `assignParticleSurface` (`scene-surface.ts:429`).
- `acquireContext(surface)`: same context attributes as today; on first acquisition check `getExtension("EXT_color_buffer_float") !== null || getExtension("EXT_color_buffer_half_float") !== null` (D-13); else mark the whole surface unavailable.
- `createFieldTarget(gl, attachmentCount: 1 | 2)` and `resizeFieldTarget(target, w, h): boolean` — RGBA16F + `HALF_FLOAT` for every attachment, `drawBuffers` for 2, return `checkFramebufferStatus === FRAMEBUFFER_COMPLETE`. Each target tracks its own width/height.
- `uploadParticles(surface, frame, camera)` — today's `uploadParticles` (`webgl-particles.ts:270-312`), shared buffers and metadata cache.
- `fieldPixelSize(camera, dpr)` — `cappedDevicePixelRatio` sizing (D-09), resizing `canvas.width/height` when needed.

**Why:** both modes splat the same particles with the same kernel; only the second attachment and the shade pass differ. Sharing buffers keeps one upload per frame and one context per canvas (D-04).

### Pattern 2: Water field pass with MRT foam

- Water point program: same kernel as `POINT_FRAGMENT` plus `in float aSpeed` -> `vFoam = smoothstep(uFoamLow, uFoamHigh, aSpeed)` in the vertex shader (keeps half-float sums small), two `layout(location = n) out vec4` outputs: field `(rgb*a, a)` and foam `(vFoam*a, a, 0, 0)`.
- Water VAO binds the shared position/radius/color buffers plus a water-owned speed buffer. Upload `frame.maybeParticleSpeeds.subarray(0, count)` each frame with `DYNAMIC_DRAW` (at most 64 KB at the 16,384 cap).
- Lane absent (held frame captured before Water was selected, or the flag not yet applied): `gl.disableVertexAttribArray(speedLoc)` and `gl.vertexAttrib1f(speedLoc, 0)` so foam is simply zero. No error.
- Blend `ONE, ONE` applies to both attachments (WebGL2 has no per-buffer blend without an extension; additive is what we want). 16-bit float blending needs no `EXT_float_blend`; that extension is only for 32-bit float targets. [CITED: STACK.md section 4; ASSUMED for the exact spec wording]
- `drawBuffers` state belongs to the bound framebuffer object in WebGL2, so the water target's two draw buffers do not leak into shaded blob's single-attachment target. [ASSUMED from WebGL2/GLES3 FBO state rules]

### Pattern 3: Refraction backdrop (D-07)

**Finding:** `static-layer.ts` has no bitmap; it stores `Path2D[]` and only after the second identical frame (`static-layer.ts:163-167`), and never for scenes below 32 segments (`:113-121`).

**Recommended implementation (`water-backdrop.ts`):**
- Per water pass: `{ maybeCanvas, revision, maybeKey, maybePending }` with an injected `createCanvas = () => document.createElement("canvas")` (Vitest runs in node).
- Each draw: compare `frame.rigidSegments` content (`Uint32Array` bit view, like `static-layer.ts:122-126`) and the view key (camera bounds/scale/offset, backing size, pixel ratio) with the last frame. Rasterize only when the content and view are identical to the previous frame **and** differ from what was last rasterized. That is static-layer's stability rule, so moving geometry never re-rasterizes or re-uploads every frame.
- Raster: opaque fill `#071018`, then the segments stroked with the same slate color/width the 2D path uses, projected with `projectPoint` at `setTransform(pixelRatio, ...)`. Bump `revision`.
- Water pass keeps `uploadedRevision`; call `texImage2D(TEXTURE_2D, 0, RGBA, RGBA, UNSIGNED_BYTE, canvas)` only when `revision !== uploadedRevision`. Reset `uploadedRevision` on context reset so the texture is re-uploaded after restore.
- No stable raster yet (first frames, or geometry always moving): set `uHasBackdrop = 0` and the shader refracts a procedural backdrop (a faint screen-space gradient and stripe pattern) so refraction still reads.
- Export the existing `sameContent` (`static-layer.ts:38-43`) and a view-key comparison from `static-layer.ts` and reuse them, so the "static layer" definition lives in one place. Do not add WebGL imports to `static-layer.ts`.
- Orientation: canvas row 0 is the top; sample with `vec2(vUv.x, 1.0 - vUv.y)` or set `UNPACK_FLIP_Y_WEBGL` around the upload and restore it (it is global pixel-store state).

### Pattern 4: Speed lane, stateless per capture (D-10, D-11)

Rust:
- `session/speeds.rs`: move `max_particle_speed` there and add `particle_speeds(velocities: &[Vec2]) -> Result<Vec<f32>, SessionError>` using `velocity.length_squared().sqrt()` (same math as the max, so `max(lane) == max_speed` bit for bit), rejecting non-finite values.
- `SessionCore::capture_frame_with_speeds(&self, particle_speeds: bool)`; `capture_frame()` delegates with `false`. Inside the existing view block, compute the lane only when the flag is set, from the same `velocities` slice (`session.rs:425`).
- `FrameData` gains `maybe_particle_speeds: Option<Vec<f32>>`; `frame/speeds.rs` adds `set_particle_speeds(Vec<f32>)` (length must equal particle count, values finite and `>= 0`, else `LaneLengthMismatch`/`InvalidDiagnostic`) and a `#[wasm_bindgen(js_name = particleSpeeds)] fn particle_speeds(&self) -> Option<Box<[f32]>>`.
- `lib.rs`: `captureFrame(particle_speeds: Option<bool>)` calls `capture_frame_with_speeds(particle_speeds.unwrap_or(false))`. JS callers that pass nothing (`readme-svg.ts:184`, `svg-export-worker.ts:88`, `puzzle-replay.ts:77,127,138`) are unchanged and never receive the lane.

TypeScript:
- `RawProofFrame.particleSpeeds?(): Float32Array | undefined`; `RenderFrame.maybeParticleSpeeds?: Float32Array` (repo `maybe` naming; D-10 names the lane `particleSpeeds` and the getter keeps that name). Parse in a new `physics/frame-speeds.ts`: absent -> omit; present -> must be `Float32Array`, length `=== particleCount`, every value finite and `>= 0`, else `invalidFrame()`.
- `GeneratedProofSession.captureFrame(maybeParticleSpeeds?: boolean): RawProofFrame` (existing zero-arg fakes stay type-compatible).
- Owners read an accessor at capture time: `createSceneSession(generated, maybeWantsParticleSpeeds?)` and `createPuzzleSession(generated, id, maybeWantsParticleSpeeds?)`; pass `() => particleSurface(appearance.renderMode()) === "webgl-water"` from `scene-lifecycle.ts:148` and `puzzle-host.ts:107` (`PuzzleHost` is a `Pick` of `SceneRuntime`, so add `"appearance"` to the pick or pass the accessor).
- Worker: add `speeds: boolean` to the `advance` and `capture` requests (validate with `typeof === "boolean"` like `budgeted`, `worker-messages.ts:114-115`), read it in `sendFrame` (`simulation-worker.ts:40`) as `session.captureFrame(request.speeds)`; `createWorkerSession` takes the accessor and sets the field on every frame request (`worker-session.ts:228-236`).
- `validateTransferredFrame` must forward `particleSpeeds: () => frame.maybeParticleSpeeds` and `frameTransferables` must append the lane when present (same tight-buffer check).

**Why stateless:** recreate controls rebuild `SessionCore` (`session.rs:346,357`), puzzles build a new `ProofSession` on every Play/Edit/Retry, and scene changes build new owners. A per-capture flag read from the current mode cannot be stale after any of those, and turning Water off removes the lane on the very next capture (D-11 "off when the mode changes").

### Pattern 5: Mode wiring and pickers (D-01..D-05, D-17)

- `mode.ts`: add `"water"` to the union, to `maybeParseRenderMode`, and as `{ value: "water", label: "Water" }` last in "Surface"; `ParticleSurface` gains `"webgl-water"`; `particleSurface("water") => "webgl-water"`; `needsWebglSurface` -> `surface === "webgl" || surface === "webgl-water"`. Because `particleSurface` is an exhaustive switch over the union, TypeScript forces the new case. `svgRenderMode` needs no change (falls through to `"solid"`), but add the D-17 equality test.
- Add a pure `isWaterLikeFallback`-style helper or reuse `needsWebglSurface` in `present-frame.ts`: water mode attempts water then shaded blob (D-12).
- `canvas.ts:351`: change to `if (webglCovered) return;` (callers only pass `true` for WebGL surface modes).
- `player-viewport.css:51`: `canvas.scene-canvas[data-particle-surface^="webgl"]` (or list both values).
- `RenderModeOptions.tsx`: one component rendering the optgroups with `waterAvailable: boolean`; the water `<option>` gets `disabled={!waterAvailable}` and text `"Water (unavailable)"` when unavailable (its accessible name is the text). Use it in `PlayerOptions` and `PuzzleRenderModeSelect`. A stored `"water"` stays selected and parsed (D-05).
- `waterAvailable` comes from the cached probe (Pattern 6), read once in `createPlaygroundSession` and passed down through `PlaygroundStage` props.

### Pattern 6: Capability probe (D-05, D-18)

- Pure core: `waterSupportFrom(facts): boolean` where facts = `{ webgl2, floatTargets (float || half ext), maxDrawBuffers >= 2, highpFragment > 0, mrtComplete, programsLinked }`.
- Shell: `probeWaterSupport(createCanvas)` on a **detached** canvas (so the particle canvas keeps its own context attributes; `getContext` attributes are fixed by the first call), builds a 2-attachment RGBA16F FBO at 4x4, links the water programs, then releases the context with `getExtension("WEBGL_lose_context")?.loseContext()`. Cache the result per page (module-level `maybeCachedSupport`), with a reset hook for tests.
- `drawWater` consults the cached result first and returns `false` immediately when unsupported, so D-12 goes straight to shaded blob.
- Playwright can force each branch with init-script prototype patches: `getParameter(MAX_DRAW_BUFFERS) -> 1` makes Water unavailable but leaves shaded blob working ("webgl"); `getExtension` returning `null` for both color-buffer extensions makes both fail ("canvas").

### Pattern 7: Fallback chain and drawn-surface attribute (D-12, D-15)

```ts
// present-frame.ts (sketch)
export type DrawnSurface = "webgl-water" | "webgl" | "canvas";

const surface = particleSurface(mode);
const water = surface === "webgl-water" &&
  drawWater(maybeParticleCanvas, frame, camera, dpr, densityShading);
const blob = !water && (surface === "webgl" || surface === "webgl-water") &&
  drawShadedBlob(maybeParticleCanvas, frame, camera, dpr, densityShading);
drawRenderFrame(context, frame, camera, mode, stroke, maxRendered,
  water || blob, densityShading, undefined, dpr);
return water ? "webgl-water" : blob ? "webgl" : "canvas";
```

- `drawSceneFrame` (`scene-surface.ts:58`) writes the result to the scene canvas: `canvas.dataset.drawnSurface = drawn` (precedent: `canvas.dataset.simulationBackend`, `scene-lifecycle.ts:179`). Use a new name because the scene canvas already carries `data-particle-surface` as the **requested** surface, and CSS keys off it (see Assumption A1).
- Also write `canvas.dataset.speedLane = frame.maybeParticleSpeeds === undefined ? "absent" : "present"` so e2e can prove the lane exists only in Water (criterion 4) without reading frames.

### Pattern 8: Context loss and restore (D-14)

- Surface reset listeners registered at surface creation (Pattern 1). `watchParticleSurface(canvas, () => paintHeldFrame(session))` registered in `assignParticleSurface` after that, for both `webglcontextlost` and `webglcontextrestored`.
- On loss: the reset runs, then `paintHeldFrame` redraws the held frame; both GL draws return `false` (lost context), so the 2D canvas paints the opaque soft-blob fallback and `data-drawn-surface="canvas"`.
- On restore: the reset clears resources, then `paintHeldFrame` rebuilds programs/targets/backdrop texture and draws `"webgl-water"` again.
- Guard the repaint so a canvas that is no longer `session.maybeParticleCanvas` does nothing (old puzzle canvases).

### Anti-Patterns to Avoid

- **Uploading the 2D scene canvas as the refraction texture every frame** — stalls mobile GPUs and includes overlays and particles. Use the stable-walls raster.
- **Computing speeds in TS from position deltas** — particle indices shift on eviction/drain (D-10).
- **Storing the capture flag on `SessionCore`** — it is replaced by `from_source` on recreate.
- **One `shadersAvailable` flag for both modes** — a water shader failure would disable shaded blob, breaking the D-12 chain.
- **A second WebGL canvas for water** — forbidden by D-04 and doubles contexts.
- **Screenshot-diffing shader output** — forbidden by D-19; SwiftShader output differs from GPUs.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Particle projection and color/radius caching | A second upload loop | `fillProjectedParticleArrays` + `createParticleMetadataCache` (shared in `field-pass.ts`) | Existing tests pin the upload counts and cache semantics. |
| Static-geometry stability detection | A new hashing scheme | `sameContent` + view-key compare from `static-layer.ts` | Same rule as the 2D cache; one definition of "static". |
| Float-target capability | UA sniffing for iOS | Extension check + `checkFramebufferStatus` | D-13; extension exposure differs by device. |
| Forcing context loss in tests | Fake events in e2e | `WEBGL_lose_context.loseContext()` / `restoreContext()` | Real browser path, D-19. |
| Optional-lane parsing | Ad hoc checks in the loop | A `frame-speeds.ts` parser in the `frame-sound.ts` style | Fail-closed, unit-testable, one place. |
| On/off determinism proof | New harness | `playSolution`-style e2e + `readRunFingerprint` (`puzzle-sound.spec.ts:117-141`) and the scene-spot recipe | Phase 42 precedent; same evidence format. |
| Gate runner | New tooling | Copy `target/phase42/checks-05-postfix.sh` (`run`, `no_match`) to `target/phase43/` | Proven 32-gate format. |

**Key insight:** the risky parts of this phase are lifecycle and plumbing (listener order, owner recreation, CSS, Bun import graph), not shader math. Plan the plumbing with tests first and keep shader constants as tunable data.

## Common Pitfalls

### Pitfall 1: Restore repaint draws with dead GL objects
**What goes wrong:** paused view stays blank after `restoreContext()`.
**Why:** a repaint listener registered before the surface's reset listener runs first; `resourcesFor` sees non-lost context and cached resources (`webgl-particles.ts:338`).
**How to avoid:** Pattern 8 — the GL module registers reset listeners at surface creation and exposes `watchParticleSurface` for ordered notification.
**Warning signs:** unit test "restore repaints" passes only when the draw happens twice.

### Pitfall 2: Water invisible behind an opaque 2D canvas
**What goes wrong:** Water engages (`data-drawn-surface="webgl-water"`) but the screen shows only the dark background.
**Why:** `player-viewport.css:51-53` only clears the scene canvas background for `"webgl"`.
**How to avoid:** prefix selector `[data-particle-surface^="webgl"]`; e2e asserts the computed `background-color` of `.scene-canvas` is transparent in Water mode.

### Pitfall 3: Bun export pipeline breaks on `?raw`
**What goes wrong:** `just readme-svg` fails to import a `.glsl?raw` module.
**Why:** `readme-svg/paint.ts:7-8` runs under Bun and imports `render/canvas` and `render/mode`.
**How to avoid:** GLSL imports only in `webgl-water.ts`/`water-capability.ts`; never import them from `canvas.ts`, `mode.ts`, `static-layer.ts`. Add a gate: `no_match -rn "water-shaders\|webgl-water\|field-pass" web/src/render/canvas.ts web/src/render/mode.ts web/src/render/static-layer.ts web/scripts web/src/export`.

### Pitfall 4: Half-float accepted but MRT incomplete
**What goes wrong:** a device renders RGBA16F single-target but reports an incomplete 2-attachment FBO.
**How to avoid:** water checks completeness of its own 2-attachment target (probe and at resize); on failure only the water pass becomes unavailable, shaded blob continues.

### Pitfall 5: Worker frames silently lose the lane
**Why:** `validateTransferredFrame` (`worker-messages.ts:156-175`) rebuilds the raw frame without optional getters, and `frameTransferables` lists five fixed lanes.
**How to avoid:** forward `particleSpeeds` and append it to transferables; e2e on `#/scene/tesla-valve` (worker backend, `data-simulation-backend="worker"`) asserts `data-speed-lane="present"` in Water.

### Pitfall 6: Mock-GL test counts break during extraction
**Why:** `webgl-render-cache.test.ts` asserts exactly 8 uniform lookups, 2 programs, and specific upload counts.
**How to avoid:** extract `field-pass.ts` first with no behavior change and the existing test file green; create water resources lazily on the first `drawWater`.

### Pitfall 7: `exactOptionalPropertyTypes` violations
**How to avoid:** spread `...(maybeParticleSpeeds === undefined ? {} : { maybeParticleSpeeds })` exactly as `frame.ts:269-270`.

### Pitfall 8: Held frame has no lane right after switching to Water
**What happens:** a paused frame captured in another mode has no speeds; Water draws without foam until the next step.
**How to handle:** treat as expected (constant zero attribute). Do not re-step to get a lane; never advance physics for rendering.

### Pitfall 9: One foam threshold does not fit every scene
**Why:** particle speed is capped at `diameter * inverse_time_step` (`pressure.rs:233-235`) and radii span 0.005-0.25 m, so caps range from under 1 m/s to tens of m/s; puzzles saturate at 9.6 m/s.
**How to handle:** D-06 locks world-speed constants. Start around `FOAM_SPEED_LOW = 3.0` and `FOAM_SPEED_HIGH = 7.5` m/s for puzzles/dam-break/fountain and accept that tiny-particle scenes show little foam. Record the tuned values and the scenes checked in the SUMMARY. [ASSUMED starting values]

### Pitfall 10: Probe consumes a WebGL context
**How to avoid:** detached canvas, release with `WEBGL_lose_context.loseContext()`, cache per page so it runs once.

### Pitfall 11: File-length limit
**Why:** `session.rs` is at 602/628 and `frame.rs` at 551; `webgl-particles.ts` 478.
**How to avoid:** new Rust code goes in `session/speeds.rs` and `frame/speeds.rs` (moving `max_particle_speed` out); `webgl-particles.ts` shrinks after extraction.

### Pitfall 12: Old puzzle canvases keep contexts
**What goes wrong:** many level switches accumulate contexts; Chrome drops the oldest (pre-existing for shaded blob).
**How to handle:** optional: when `assignParticleSurface` replaces a canvas, detach the watcher and call `loseContext()` on the old surface; guard repaint callbacks by canvas identity. Not a blocker.

## Code Examples

### D-13 float-or-half acceptance
```ts
// field-pass.ts — source: Khronos EXT_color_buffer_half_float (RGBA16F required renderable)
function supportsHalfFloatTargets(gl: WebGL2RenderingContext): boolean {
  // Calling getExtension enables it; either one makes RGBA16F color-renderable.
  return (
    gl.getExtension("EXT_color_buffer_float") !== null ||
    gl.getExtension("EXT_color_buffer_half_float") !== null
  );
}
```

### MRT field target (RGBA16F x2)
```ts
function attachFieldTextures(gl: WebGL2RenderingContext, fbo: WebGLFramebuffer,
  textures: readonly WebGLTexture[], width: number, height: number): boolean {
  gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
  const attachments: number[] = [];
  textures.forEach((texture, index) => {
    gl.bindTexture(gl.TEXTURE_2D, texture);
    // ... LINEAR / CLAMP_TO_EDGE parameters as in webgl-particles.ts:244-247
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA16F, width, height, 0, gl.RGBA, gl.HALF_FLOAT, null);
    const attachment = gl.COLOR_ATTACHMENT0 + index;
    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachment, gl.TEXTURE_2D, texture, 0);
    attachments.push(attachment);
  });
  gl.drawBuffers(attachments);
  return gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE;
}
```

### Water point fragment (highp, two outputs)
```glsl
#version 300 es
precision highp float;
in vec4 vColor;
in float vFoam;          // smoothstep(uFoamLow, uFoamHigh, aSpeed) from the vertex shader
layout(location = 0) out vec4 outField;
layout(location = 1) out vec4 outFoam;
void main() {
  vec2 point = gl_PointCoord * 2.0 - 1.0;
  float dist2 = dot(point, point);
  if (dist2 > 1.0) discard;
  float weight = (1.0 - dist2) * (1.0 - dist2);   // same kernel as shaded blob
  float alpha = vColor.a * weight;
  outField = vec4(vColor.rgb * alpha, alpha);
  outFoam = vec4(vFoam * alpha, alpha, 0.0, 0.0);
}
```

### Water shade fragment (sketch; constants are discretion)
```glsl
#version 300 es
precision highp float;
uniform sampler2D uField;      // unit 0
uniform sampler2D uFoam;       // unit 1
uniform sampler2D uBackdrop;   // unit 2
uniform float uHasBackdrop;
uniform vec2 uTexel;           // 1 / field size
uniform vec3 uBackground;      // #071018, same as shaded blob
uniform float uRefraction, uNormalScale, uShininess, uSpecular, uAbsorption;
uniform vec3 uLightDir;        // normalized, fixed
in vec2 vUv;
out vec4 fragColor;

vec3 backdrop(vec2 uv) {
  vec2 flipped = vec2(uv.x, 1.0 - uv.y);
  vec3 procedural = uBackground + 0.035 * vec3(step(0.5, fract(uv.y * 24.0)));
  return mix(procedural, texture(uBackdrop, flipped).rgb, uHasBackdrop);
}

void main() {
  vec4 sum = texture(uField, vUv);
  float edge = smoothstep(0.42, 0.62, sum.a);           // same edge as shaded blob
  if (edge <= 0.0) { fragColor = vec4(uBackground, 1.0); return; }
  float dx = texture(uField, vUv + vec2(uTexel.x, 0.0)).a - texture(uField, vUv - vec2(uTexel.x, 0.0)).a;
  float dy = texture(uField, vUv + vec2(0.0, uTexel.y)).a - texture(uField, vUv - vec2(0.0, uTexel.y)).a;
  vec3 normal = normalize(vec3(-dx * uNormalScale, -dy * uNormalScale, 1.0));
  vec3 refracted = backdrop(vUv + normal.xy * uRefraction);
  vec3 color = sum.rgb / max(sum.a, 0.0001);            // each particle's own color
  float transmit = exp(-uAbsorption * sum.a);           // Beer-Lambert on thickness
  vec3 body = mix(color, refracted * mix(vec3(1.0), color, 0.5), transmit);
  vec3 halfVector = normalize(uLightDir + vec3(0.0, 0.0, 1.0));
  float spec = uSpecular * pow(max(dot(normal, halfVector), 0.0), uShininess);
  vec4 foamSum = texture(uFoam, vUv);
  float foam = clamp(foamSum.r / max(foamSum.g, 0.0001), 0.0, 1.0);
  vec3 water = mix(body, vec3(1.0), foam) + spec;
  fragColor = vec4(mix(uBackground, water, edge), 1.0);
}
```

### Speed lane parse (fail closed)
```ts
// physics/frame-speeds.ts — mirrors frame-sound.ts
export function parseParticleSpeeds(raw: RawProofFrame, particleCount: number): Float32Array | undefined {
  const maybeSpeeds = raw.particleSpeeds?.();
  if (maybeSpeeds === undefined) return undefined;
  if (!(maybeSpeeds instanceof Float32Array) || maybeSpeeds.length !== particleCount) return invalidFrame();
  for (const speed of maybeSpeeds) if (!Number.isFinite(speed) || speed < 0) invalidFrame();
  return maybeSpeeds;
}
```

### Rust getter
```rust
// crates/liquidfun-wasm/src/frame/speeds.rs
#[wasm_bindgen]
impl ProofFrame {
    /// Returns per-particle speeds in m/s, or undefined unless the capture asked for them.
    #[must_use]
    #[wasm_bindgen(js_name = particleSpeeds)]
    pub fn particle_speeds(&self) -> Option<Box<[f32]>> {
        self.data.maybe_particle_speeds.clone().map(Vec::into_boxed_slice)
    }
}
```

### e2e forced loss/restore through D-15
```ts
const lose = () => page.evaluate(() => {
  const canvas = document.querySelector("canvas.particle-surface") as HTMLCanvasElement;
  canvas.getContext("webgl2")?.getExtension("WEBGL_lose_context")?.loseContext();
});
// pause first, then:
await lose();
await expect(page.locator("canvas.scene-canvas")).toHaveAttribute("data-drawn-surface", "canvas");
await page.evaluate(() => { /* same lookup */ ext.restoreContext(); });
await expect(page.locator("canvas.scene-canvas")).toHaveAttribute("data-drawn-surface", "webgl-water");
```
`getContext("webgl2")` returns the existing context for the same context id; keep the extension object from the first call because `getExtension` on a lost context may return `null`. [ASSUMED: store the extension object on `window` inside the first evaluate to be safe]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Require `EXT_color_buffer_float` for RGBA16F | Accept float or half-float ext, trust FBO status | WebKit exposed `EXT_color_buffer_half_float` on WebGL2 (bug 217107) | Shaded blob and water may now run on iPhones (D-20 UAT). |
| WebGL1 MRT via `WEBGL_draw_buffers` | WebGL2 core `drawBuffers` + `layout(location)` outputs | WebGL2 | No extension needed for MRT. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | D-15's attribute is written as `data-drawn-surface` on `.scene-canvas` (D-15 says "for example `data-particle-surface`", but that name already exists on the same canvas as the requested surface and drives CSS). | Pattern 7 | Low: rename is mechanical; confirm the name in the plan. |
| A2 | The refraction backdrop is a water-owned raster of stable walls that reuses static-layer's equality rule, because `static-layer.ts` holds no bitmap and skips scenes under 32 segments. | Pattern 3 | Medium: if the owner insists on the literal 2D-cache coupling, scenes below 32 segments would only get the procedural backdrop. |
| A3 | Capture option is a stateless per-capture flag (`captureFrame(particleSpeeds?)`) with owner accessors and a worker request field. | Pattern 4 | Low: Claude's discretion per CONTEXT. |
| A4 | Probe is cached per page on a detached canvas and released with `loseContext`. | Pattern 6 | Low: discretion. |
| A5 | 16-bit float blending needs no extension in WebGL2 when RGBA16F is renderable; `drawBuffers` state is per FBO. | Pattern 2 | Medium: if a device rejects blending, the probe/FBO path fails and water falls back (safe). |
| A6 | Starting foam thresholds ~3.0 / 7.5 m/s and the shader constants in the sketch. | Pitfall 9, Code Examples | Low: tuned by eye. |
| A7 | The browser SVG export for Water and Shaded blob produces byte-identical files for the same scene/view when compared within one page session. | Validation | Low: if output embeds variable metadata, compare after stripping it or compare the request object from `createSvgExportRequest`. |

## Open Questions

1. **D-07 wording vs the static-layer reality.** What we know: no raster exists; the 2D cache skips small scenes. Recommendation: implement Pattern 3, cite this finding in the plan, and keep `static-layer.ts` as the owner of the equality helpers.
2. **Density shading checkbox in Water.** `PlayerPanel.tsx:436-448` shows it for all surface modes, so it will appear for Water. Recommendation: pass `densityShading` into `drawWater` and apply the same `DENSITY_SHADE_*` darkening to `color` so the existing control stays meaningful.
3. **Disable-on-runtime-failure.** If the probe says yes but `drawWater` later fails (compile/FBO), should the option flip to "Water (unavailable)"? Recommendation: no; D-12 fallback already covers it, and the drawn-surface attribute makes it observable.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | bridge lane | yes | rustc/cargo 1.97.0 | — |
| wasm-pack | `just web-wasm` | yes | 0.15.0 (pinned) | — |
| bun | web scripts | yes | 1.4.2 | — |
| just | recipes | yes | 1.48.0 | — |
| jq | fingerprint extraction | yes | 1.7.1 | — |
| Playwright Chromium | e2e | yes | `chromium-1243` cache present, `@playwright/test` 1.63.0 | — |
| Headless WebGL2 features | e2e water/fallback/loss | yes | WebGL2, float + half-float ext, `EXT_float_blend`, `WEBGL_lose_context`, highp 23, 8 draw buffers | — |
| Real iPhone | D-20 UAT | not in CI | — | Non-blocking human UAT |

Note: `wasm-bindgen` on PATH is 0.2.106, but builds go through `wasm-pack` which matches the pinned crate; do not invoke the PATH CLI directly.

## Validation Architecture

(`workflow.nyquist_validation` is `false` in `.planning/config.json`; this section is included because the orchestrator requested exact commands.)

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest 5.0.1 (node), Playwright 1.63.0 (Chromium), cargo test (Rust 1.97.0) |
| Config files | `web/vitest.config.ts`, `web/playwright.config.ts`, `justfile` |
| Quick run command | `cd web && bunx vitest run tests/render-mode.test.ts tests/webgl-render-cache.test.ts tests/water-*.test.ts tests/frame-speeds.test.ts tests/worker-messages.test.ts` |
| Full suite command | `bash target/phase43/checks-final.sh` (gate script below) |

### Baseline (first task of the phase, before any Rust edit)

```bash
cd /Users/peterryszkiewicz/Repos/liquidfun-rs
mkdir -p target/phase43 && git rev-parse HEAD > target/phase43/phase-base.txt
git diff --quiet -- crates tools xtask Cargo.toml Cargo.lock   # must exit 0
just playground-scene-spot 2>&1 | tee -a target/phase43/baseline.log
S=$(ls -td target/dam-break-perf/*/ | head -1); echo "stamp $S" >> target/phase43/baseline.log
jq -r '.scenes[] | "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" | sort > target/phase43/base.txt
test "$(wc -l < target/phase43/base.txt)" -eq 25
diff target/phase42/after-postfix.txt target/phase43/base.txt && echo "phase42-cross-check: identical" >> target/phase43/baseline.log
```

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WATER-01 | `"water"` parses, is last in Surface, default unchanged, surface `"webgl-water"`, both surfaces need WebGL | unit | `cd web && bunx vitest run tests/render-mode.test.ts` | exists (update) |
| WATER-01 | Water pass: MRT target, lazy resources, speed upload, zero attribute when lane absent, backdrop uploads only on revision change | unit (mock GL) | `cd web && bunx vitest run tests/webgl-water.test.ts tests/water-backdrop.test.ts` | Wave 0 |
| WATER-01 | Speed lane: Rust capture with/without flag, lane == per-particle `sqrt(length_squared)`, `max(lane) == max_speed`, validation errors | unit (Rust) | `cargo test -p liquidfun-wasm --lib speeds` | Wave 0 |
| WATER-01 | Speed lane parse/omit/transfer; worker request `speeds` flag validation | unit | `cd web && bunx vitest run tests/frame-speeds.test.ts tests/worker-messages.test.ts tests/session.test.ts tests/puzzle-session.test.ts` | partial (new file + updates) |
| WATER-01 | Water engages in a scene (dam-break), Tesla worker scene, and a puzzle; `data-speed-lane` present only in Water; overlays still drawn | e2e | `cd web && bun run build:app && CI=true bunx playwright test e2e/water.spec.ts` | Wave 0 |
| WATER-02 | Probe decision table; half-float only accepted; incomplete FBO -> false; water failure leaves blob available | unit | `cd web && bunx vitest run tests/water-capability.test.ts tests/webgl-render-cache.test.ts` | Wave 0 + exists |
| WATER-02 | Fallback chain returns `webgl-water`/`webgl`/`canvas`; `canvas.ts` skips metaball when covered | unit | `cd web && bunx vitest run tests/present-frame.test.ts tests/canvas.test.ts` | Wave 0 + exists |
| WATER-02 | Loss falls back and repaints; restore rebuilds and repaints, in that order | unit | `cd web && bunx vitest run tests/field-pass.test.ts` | Wave 0 |
| WATER-02 | Forced unavailability (patched `MAX_DRAW_BUFFERS`) -> option "Water (unavailable)" disabled and `data-drawn-surface="webgl"`; no float ext -> `"canvas"`; forced `loseContext`/`restoreContext` while paused | e2e | `CI=true bunx playwright test e2e/water.spec.ts` | Wave 0 |
| WATER-03 | `svgRenderMode("water") === svgRenderMode("shaded-blob")`; export pins unchanged | unit + gate | `bunx vitest run tests/render-mode.test.ts`; `git diff --quiet "$BASE" -- web/scripts/readme-svg/plans.ts web/scripts/demo-media/capture.ts docs/assets/readme` | exists |
| WATER-03 | SVG export in Water equals Shaded blob export | e2e | `CI=true bunx playwright test e2e/water.spec.ts -g "export"` | Wave 0 |
| WATER-03 / D-11 | 25/25 scene fingerprints; puzzle RunFingerprint equal with Water on and off | gate + e2e | fingerprint recipe below; `CI=true bunx playwright test e2e/water.spec.ts -g "same run"` | Wave 0 |
| D-11 | Rust: capturing with speeds every step leaves positions and puzzle status bit-identical to capturing without | unit (Rust) | `cargo test -p liquidfun-wasm --lib speeds` | Wave 0 |

### Gate script (final plan; copy `target/phase42/checks-05-postfix.sh` format)

Gates in order (one `run` line each, `BASE=$(cat target/phase43/phase-base.txt)`):
1. `web-wasm`: `just web-wasm`
1. `fingerprints`: `just playground-scene-spot; S=$(ls -td target/dam-break-perf/*/ | head -1); jq -r '.scenes[] | "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" | sort > target/phase43/after-final.txt; diff target/phase43/base.txt target/phase43/after-final.txt; test "$(wc -l < target/phase43/after-final.txt)" -eq 25`
1. `engine-untouched`: `test -z "$(git diff --stat "$BASE" -- crates/liquidfun)"`
1. `puzzle-fixtures-unchanged`: `git diff --quiet "$BASE" -- crates/liquidfun-wasm/src/puzzle/fixtures crates/liquidfun-wasm/src/puzzle/level-hashes.txt`
1. `scenes-and-exports-pinned`: `git diff --quiet "$BASE" -- web/src/catalog web/scripts/readme-svg/plans.ts web/scripts/demo-media/capture.ts docs/assets/readme crates/liquidfun-wasm/src/scene.rs crates/liquidfun-wasm/src/scene`
1. `protected-tests-unedited`: `git diff --quiet "$BASE" -- web/tests/hash.test.ts web/tests/portrait-bounds.test.ts web/tests/readme-svg.test.ts web/e2e/hud-safe-frame.spec.ts web/e2e/shell.spec.ts`
1. `no-new-dependencies`: `git diff --quiet "$BASE" -- web/bun.lock Cargo.lock crates/liquidfun-wasm/Cargo.toml && test -z "$(git diff "$BASE" -- web/package.json | grep -E '^[+-] ' | grep -vE '"test:(player|puzzle|water)"')"`
1. `cargo-fmt`: `cargo fmt --all --check`
1. `cargo-clippy`: `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo-build`: `cargo build --all-targets --all-features`
1. `cargo-test`: `cargo test --all-features`
1. `wasm-crate-tests`: `cargo test -p liquidfun-wasm --all-features`
1. `speeds-tests`: `cargo test -p liquidfun-wasm --lib speeds 2>&1 | tee target/phase43/speeds-test.log | tail -3; grep -qE "test result: ok\. [1-9][0-9]* passed" target/phase43/speeds-test.log`
1. `release-replay`: `cargo test -p liquidfun-wasm --release --lib puzzle::replay` (non-zero pass count)
1. `puzzle-replay-wasm`: `just puzzle-replay-wasm 2>&1 | tee target/phase43/replay.log; test "$(grep -c "^LEVELHASH [a-z-]* pass" target/phase43/replay.log)" -eq 6; no_match -Eq "^(LEVELHASH|REPLAY) .* fail" target/phase43/replay.log`
1. `typecheck`: `(cd web && bun run typecheck)`
1. `vitest`: `(cd web && bun run test:unit)`
1. `bright-builds`: `bun scripts/bright-builds-check.ts all`
1. `player-smoke`: `just web-player-smoke`
1. `hud-safe-frame`: `(cd web && bun run build:app && CI=true bunx playwright test e2e/hud-safe-frame.spec.ts)`
1. `puzzle-e2e`: `(cd web && bun run build:app && CI=true bun run test:puzzle)`
1. `water-e2e`: `(cd web && bun run build:app && CI=true bunx playwright test e2e/water.spec.ts)`
1. `live-worker-e2e`: `(cd web && CI=true bunx playwright test e2e/live-worker.spec.ts)`
1. `markdown`: `if test -n "$(git diff --name-only "$BASE" -- '*.md' ':!.planning')"; then just markdown-check; fi`
1. `default-mode-unchanged`: `grep -q 'export const DEFAULT_RENDER_MODE: RenderMode = "shaded-blob";' web/src/render/mode.ts`
1. `water-highp`: `grep -q "precision highp float" web/src/render/water-shaders/water-shade.frag.glsl && no_match -rn "mediump" web/src/render/water-shaders`
1. `blob-shaders-unchanged`: `test "$(grep -c "precision mediump float" web/src/render/webgl-particles.ts)" -eq 2`
1. `no-gpu-libraries`: `no_match -niE '"(three|twgl|pixi)' web/package.json`
1. `bun-graph-clean`: `no_match -rnE "water-shaders|webgl-water|field-pass|water-capability" web/src/render/canvas.ts web/src/render/mode.ts web/src/render/static-layer.ts web/scripts web/src/export`
1. `lane-read-only`: `test -z "$(grep -rn "particle_speeds" crates/liquidfun-wasm/src | grep -vE "frame/speeds.rs|session/speeds.rs|session.rs|frame.rs|lib.rs")"`
1. `single-webgl-canvas`: `test "$(grep -c 'class="particle-surface"' web/src/components/PlayerPanel.tsx web/src/components/PuzzlePanel.tsx | awk -F: '{s+=$2} END {print s}')" -eq 2`

Notes: run cargo gates sequentially with long timeouts (macOS first-launch stalls). Playwright needs a fresh `bun run build:app` and `CI=true`. Preserve failed attempts as `checks-final.attempt-<n>.{results,log}`.

### Sampling Rate

- **Per task commit:** affected Vitest files + `bun run typecheck`; Rust tasks add `cargo test -p liquidfun-wasm --lib speeds` and clippy for the crate.
- **Per plan:** `bun run test:unit`, the plan's e2e spec, and the fingerprint diff if Rust changed.
- **Phase gate:** full gate script green, then `/gsd-verify-work` with D-20 iPhone UAT recorded as pending human verification.

### Wave 0 Gaps

- [ ] `web/tests/webgl-render-fixture.ts` — add constants (`COLOR_ATTACHMENT1`, `MAX_DRAW_BUFFERS`, `UNSIGNED_BYTE`, `UNPACK_FLIP_Y_WEBGL`, `TEXTURE1`, `TEXTURE2`, `HIGH_FLOAT`), an `options` argument for available extensions, per-call `checkFramebufferStatus`, `getShaderPrecisionFormat`, and `getParameter` by name. Keep existing defaults so current tests pass unchanged.
- [ ] `web/tests/field-pass.test.ts`, `webgl-water.test.ts`, `water-backdrop.test.ts`, `water-capability.test.ts`, `present-frame.test.ts`, `frame-speeds.test.ts`
- [ ] `crates/liquidfun-wasm/src/frame/speeds.rs` tests and `session/speeds.rs` tests
- [ ] `web/e2e/water.spec.ts` (+ add to a `test:player`/`test:puzzle` list or a new `test:water` script)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Allowlisted render-mode parse from localStorage (`maybeParseRenderMode`); fail-closed frame lane parse (length, finite, non-negative) in TS and Rust; worker request `speeds` must be a boolean |
| V6 Cryptography | no | — |
| V14 Configuration / dependencies | yes | No new dependencies (gate); pinned wasm-pack |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Render path alters simulation | Tampering | Capture is `&self`; flag only affects copied output; 25/25 fingerprints; Rust on/off bit-identity test; puzzle RunFingerprint on/off e2e |
| Malformed or oversized lane from worker | Tampering / DoS | Same strict parser as direct frames; transferable tight-buffer check |
| GPU exhaustion on phones | DoS | DPR cap, one context per canvas, probe context released, fallback on loss |
| Corrupt stored preference | Tampering | Allowlist parse, default on unknown |

## Sources

### Primary (HIGH confidence)
- Repository source at `cd401e883`: files and lines cited in "Current Code Facts".
- `target/phase42/checks-05-postfix.sh`, `.planning/phases/42-opt-in-sound/42-01-PLAN.md`, `42-05-PLAN.md`, `42-VERIFICATION.md` — gate mechanics.
- Live probe this session: headless Chromium via `@playwright/test` 1.63.0 — WebGL2 feature set listed above.
- Live check this session: Vitest `?raw` import of a `.glsl` file passed.
- [Khronos: EXT_color_buffer_half_float](https://registry.khronos.org/webgl/extensions/EXT_color_buffer_half_float/) — WebGL2 exposure, RGBA16F required renderable.
- [MDN: WEBGL_lose_context.restoreContext](https://developer.mozilla.org/en-US/docs/Web/API/WEBGL_lose_context/restoreContext) — error condition and restored-event requirement.
- [wasm-bindgen: boxed number slices](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html) and [bool](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/bool.html) — `Option<Box<[f32]>>` return and `Option<bool>` parameter support.

### Secondary (MEDIUM confidence)
- `.planning/research/v1.5-puzzle-mode/STACK.md` section 4, `ARCHITECTURE.md` D7, `PITFALLS.md` Pitfall 13 — design direction, verified against current code above.
- WebKit bugs 217107 and 247240 (via STACK.md) — iOS half-float targets.

### Tertiary (LOW confidence)
- Starting foam/refraction constants (by-eye tuning required).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; APIs verified by docs and a live probe.
- Architecture: HIGH — every integration point read in source; the static-layer, worker-scope, listener-order and CSS findings are direct code facts.
- Pitfalls: HIGH for plumbing, MEDIUM for device behavior and look constants.

**Research date:** 2026-10-13
**Valid until:** 2026-11-12 (stable codebase; re-check line numbers if other phases land first)

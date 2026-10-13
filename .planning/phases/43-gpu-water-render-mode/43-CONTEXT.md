---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: 2026-10-13T22:58:43.360Z
---

# Phase 43: GPU Water Render Mode - Context

**Gathered:** 2026-10-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Add one new particle render mode, "Water", to the playground and puzzle players. It is a WebGL2 shader mode with refraction of what sits behind the water, specular highlights, thickness tint and foam on fast-moving water. It extends the existing `shaded-blob` WebGL pipeline. When the device cannot run it, or the WebGL context is lost, the view falls back to shaded blob and then to the canvas path, and it recovers when the context is restored. Exports keep their existing styles. The per-particle speed data that drives foam is produced only while Water mode is active, so the 25 scene fingerprints and puzzle outcomes are unchanged.

Not in scope: new physics, new scenes or levels, changing the default mode, README/demo media restyling, or new npm/Cargo dependencies.

</domain>

<decisions>
## Implementation Decisions

### Mode wiring and picker

- **D-01:** Add `"water"` to `RenderMode`, `maybeParseRenderMode` and `RENDER_MODE_GROUPS` in `web/src/render/mode.ts`, labeled "Water", as the last option of the existing "Surface" group (no new group). The stored preference uses the existing `liquidfun.render-mode.v1` key.
- **D-02:** `DEFAULT_RENDER_MODE` stays `"shaded-blob"`. Water is opt-in.
- **D-03:** Water appears in both pickers: the scene "Particles" select (`PlayerPanel.tsx`) and `PuzzleRenderModeSelect` (`PuzzleOptionsDrawer.tsx`, both the control row and the options sheet). No new HUD control.
- **D-04:** `particleSurface("water")` maps to a new `"webgl-water"` surface. `needsWebglSurface` returns true for both WebGL surfaces, so the existing `<canvas class="particle-surface">` is reused. Do not add a second WebGL canvas.
- **D-05:** When a capability probe shows the device cannot run Water, the Water option stays visible but disabled, with the accessible name "Water (unavailable)", following the Phase 42 D-05 pattern. A stored `"water"` preference is still parsed and kept; at draw time it falls back (D-12), so the preference returns on a capable device.

### Look

- **D-06:** The shader does a field pass shared with shaded blob, then a shade pass with:
  - refraction: offset sampling of a background texture by the field gradient;
  - Blinn-Phong specular highlights from a fixed light direction;
  - Beer-Lambert thickness tint that keeps each particle's own color, so colored and multi-material scenes still read;
  - foam: blend toward white where the per-particle speed is high, using a smoothstep ramp between two world-speed constants (m/s).
- **D-07:** The refraction background is the cached static layer from `static-layer.ts` (walls and static geometry), uploaded as a texture only when that layer changes. Never upload a texture every frame. If the static layer is unavailable, use a plain procedural backdrop.
- **D-08:** Water shaders use `precision highp float`. Shaded blob's existing `mediump` shaders are left unchanged.
- **D-09:** Cap the field resolution with `cappedDevicePixelRatio`, matching shaded blob.

### Speed lane (foam data)

- **D-10:** Add an optional per-particle speed lane: `particleSpeeds: Float32Array`, one float per particle, through `FrameData`, `RawProofFrame`, `RenderFrame` and `frameTransferables`. Rust computes it in `capture_frame` from the `view.velocities()` call that already happens. Do not derive speed in TypeScript from position deltas.
- **D-11:** The lane is filled only when a capture option is on. The worker turns the option on only while the active render mode is Water, and off when the mode changes. When the option is off, the frame omits the lane, so frames in other modes are the same as today. The lane never feeds physics. Gates:
  - `just playground-scene-spot` gives 25/25 fingerprints identical to the phase-base HEAD;
  - puzzle replay outcomes (RunFingerprint) are identical with Water on and off, using the Phase 42 D-05 on/off replay pattern.

### Fallback and context loss

- **D-12:** Use one fallback chain per frame: `covered = (water && drawWater(...)) || (blobLike && drawShadedBlob(...))`, then the canvas path (`paintMetaball` soft blob). Never present a blank frame.
- **D-13:** Float render targets accept `EXT_color_buffer_float` **or** `EXT_color_buffer_half_float`, use RGBA16F, and trust `checkFramebufferStatus`. This applies to the shared field pass, so shaded blob also gains half-float support. This folds in the pending STATE.md todo.
- **D-14:** When the context is lost, fall back right away. When it is restored, rebuild the GPU resources and repaint the held frame through the existing `paintHeldFrame`, so a paused view does not stay stale or blank.
- **D-15:** Expose the surface that actually drew as a DOM attribute on the stage, for example `data-particle-surface="webgl-water" | "webgl" | "canvas"`, so tests can check fallback without pixel diffs.

### Overlays and exports

- **D-16:** Cup, fill meter, target line, tools, labels, walls and scale legend stay on the 2D canvas above the particle surface in Water mode, as with shaded blob (38-CONTEXT).
- **D-17:** `svgRenderMode("water")` maps to `"solid"`, like the other surface modes. README SVG/WebP plans stay pinned to `"wireframe"`, and demo media stays on circle-wireframe. Unit tests confirm that the export mapping for water equals the mapping for shaded blob.

### Verification

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and scope

- `.planning/ROADMAP.md` § Phase 43: goal and four success criteria.
- `.planning/REQUIREMENTS.md`: WATER-01, WATER-02, WATER-03.
- `PROJECT-SCOPE.md`: hobby scope. Linux qualification and device checks are optional.
- `.planning/seeds/SEED-003-gpu-water-rendering.md`: origin and breadcrumbs.

### Research (v1.5)

- `.planning/research/v1.5-puzzle-mode/STACK.md` (about lines 114-129, 176-204): the WebGL2 approach, MRT layout, speed lane, refraction background, float targets and testing guidance.
- `.planning/research/v1.5-puzzle-mode/ARCHITECTURE.md` (about lines 221-226): mode wiring, the fallback expression and the speed-lane origin.
- `.planning/research/v1.5-puzzle-mode/PITFALLS.md` (Pitfall 13, about lines 361-386, and the checklist at 485 and 517): precision, context restore repaint, export exclusion and DPR cap.
- `.planning/research/v1.5-puzzle-mode/SUMMARY.md`: the research flag for a phone GPU spike.

### Prior phase decisions

- `.planning/phases/38-*/38-CONTEXT.md`: overlays above the particle layer, the render-mode option on puzzle routes, and the D-24 options drawer.
- `.planning/phases/42-*/42-CONTEXT.md`: D-05 (the unavailable control pattern) and the on/off replay fingerprint gate.

### Code

- `web/src/render/mode.ts`: modes, groups, parsing, persistence, `svgRenderMode`, `particleSurface` and `needsWebglSurface`.
- `web/src/render/webgl-particles.ts`: the existing shaded-blob WebGL2 pipeline, float-target check (about line 152), context-loss handlers (about lines 321-332) and the per-canvas WeakMap.
- `web/src/render/present-frame.ts` and `web/src/render/canvas.ts`: frame presentation and canvas fallback.
- `web/src/render/static-layer.ts`: the cached static layer used as the refraction background.
- `web/src/player/scene-surface.ts`: `paintHeldFrame`, overlays and `assignParticleSurface`.
- `web/src/physics/frame.ts` and `web/src/physics/worker-messages.ts`: the frame contract and transferables.
- `crates/liquidfun-wasm/src/session.rs` (`capture_frame`, `annotate_frame`) and `crates/liquidfun-wasm/src/frame.rs`: the bridge frame.
- `web/scripts/readme-svg/plans.ts` and `web/scripts/demo-media/capture.ts`: export pins that must stay unchanged.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- The `webgl-particles.ts` field pass (an RGBA16F field, a completeness check, and a per-canvas surface in a WeakMap): factor it out and share it with Water.
- `cappedDevicePixelRatio` (`canvas.ts:37`): the field resolution cap.
- `paintHeldFrame` (`scene-surface.ts`): repaint after the context is restored.
- The mock GL fixture (`web/tests/webgl-render-fixture.ts`) and `webgl-render-cache.test.ts`, which already covers a lost-then-restored context.
- The guarded preference pattern (`loadRenderMode` and `persistRenderMode`) and the Phase 42 unavailable-control pattern.

### Established Patterns

- Optional frame fields (`maybePuzzle`, `maybeSound`) are attached in `annotate_frame` and parsed fail-closed in TypeScript. The speed lane follows the same pattern.
- Phases that touch the bridge or the frame contract record a scene-spot baseline at the phase-base HEAD and require 25/25 identical fingerprints.
- E2E checks assert DOM state (`data-render-mode`) and pixel hashes for mode switching. Shader output is not diffed.

### Integration Points

- Render-mode signal: `player/appearance.ts` and `main[data-render-mode]` in `PlaygroundStage.tsx`.
- The worker capture option, sent from the main thread when the mode changes.
- The `present-frame.ts` fallback chain and the `canvas.ts` "WebGL covered" branch.

</code_context>

<specifics>
## Specific Ideas

- Foam should read as white spray on fast jets, such as the fountain, a dam-break front or valve outflow. Slow pooled water should look clear and tinted, with highlights.
- The "Water (unavailable)" disabled option is better than silently switching modes (research, STACK.md).

</specifics>

<deferred>
## Deferred Ideas

- Water-styled README/WebP exports, or a GPU export path. Exports stay pinned for now.
- Making Water the default mode after it has been verified on phones.
- Reflections, caustics and a dynamic-body refraction background, beyond the static layer.

</deferred>

---

*Phase: 43-gpu-water-render-mode*
*Context gathered: 2026-10-13*

---
phase: 43-gpu-water-render-mode
plan: "04"
subsystem: web-render
tags: [typescript, solidjs, webgl2, water, fallback, render-mode, picker]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-14T00:15:03.522Z"
requires:
  - "43-01 RenderFrame.maybeParticleSpeeds"
  - "43-02 field-pass.ts watchParticleSurface (reset-before-repaint ordering)"
  - "43-03 drawWater and cachedWaterSupport"
provides:
  - "mode.ts: RenderMode \"water\", ParticleSurface \"webgl-water\", WATER_UNAVAILABLE_LABEL, wantsParticleSpeeds, renderModeOptionView, RenderModeOptionView"
  - "present-frame.ts: DrawnSurface and presentSceneFrame returning the surface that drew (D-12 chain)"
  - "Scene canvas data-drawn-surface (webgl-water | webgl | canvas) and data-speed-lane (present | absent) after every draw"
  - "components/RenderModeOptions.tsx shared by both Particles pickers"
  - "PlaygroundStage / PlayerPanel / PuzzlePanel / PuzzleRenderModeSelect waterAvailable prop"
affects:
  - "43-05 speed lane wiring: wantsParticleSpeeds(mode) is the requested-mode switch; data-speed-lane reports it"
  - "43-06 browser proof: data-drawn-surface, loss/restore repaint, transparency, Water (unavailable)"
tech-stack:
  added: []
  patterns:
    - "Fallback chain as short-circuit booleans that also select the reported surface"
    - "Per-canvas watcher detached on reassignment and guarded by canvas identity"
key-files:
  created:
    - web/src/components/RenderModeOptions.tsx
    - web/tests/present-frame.test.ts
  modified:
    - web/src/render/mode.ts
    - web/src/render/present-frame.ts
    - web/src/render/canvas.ts
    - web/src/player/scene-surface.ts
    - web/src/styles/player-viewport.css
    - web/src/components/PlayerPanel.tsx
    - web/src/components/PuzzleOptionsDrawer.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/components/PlaygroundStage.tsx
    - web/src/App.tsx
    - web/tests/render-mode.test.ts
    - web/tests/canvas.test.ts
decisions:
  - "data-drawn-surface reports the surface that drew; data-particle-surface stays the requested surface and keeps driving CSS (UI-D52)"
  - "drawRenderFrame skips the canvas painter whenever webglCovered is true; only presentSceneFrame passes true, and only for WebGL surface modes"
  - "renderModeOptionView returns a named RenderModeOptionView type (same shape as the planned inline type)"
  - "Water availability is read once in App.tsx right after createPlaygroundSession() and passed down as a plain boolean (UI-D48); no runtime flip to unavailable (UI-D50)"
metrics:
  duration: 4min
  completed: 2026-10-14
  tasks: 3
  files: 14
---

# Phase 43 Plan 04: Water mode wiring Summary

Players can now choose Water in scene and puzzle pickers. A single fallback chain runs on every frame: Water, then the shaded blob, then the canvas soft blob. The chain reports which surface drew in `data-drawn-surface`, and a held frame repaints on GL loss and restore. The scene canvas stays transparent so overlays draw above the water. On an incapable device the option reads "Water (unavailable)" and is disabled.

## What Was Built

- **Task 1 (6d0856d57):** `mode.ts` gains the `water` mode and the `webgl-water` surface. It also adds `WATER_UNAVAILABLE_LABEL`, `wantsParticleSpeeds` (true only for Water) and `renderModeOptionView`. `needsWebglSurface` now covers both WebGL surfaces, and `svgRenderMode("water")` is `"solid"` (D-17). The `DEFAULT_RENDER_MODE` line is byte-identical. `presentSceneFrame` returns a `DrawnSurface` from the D-12 chain. In `canvas.ts` the covered check is now `if (webglCovered) return;`, and `canvas.ts` and `mode.ts` gained no imports.
- **Task 2 (f3adfdbd8):** `drawSceneFrame` writes `dataset.drawnSurface` and `dataset.speedLane` on the 2D canvas after every draw, in both scenes and puzzles. It does this before the puzzle overlays, which keeps the D-22 order. `assignParticleSurface` detaches the previous watcher and then registers `watchParticleSurface`. Its repaint runs only while `session.maybeParticleCanvas === canvas` (Pitfall 12). The CSS transparency rule lists `"webgl"` and `"webgl-water"` explicitly (UI-D47).
- **Task 3 (857603bcd):** `RenderModeOptions` renders the optgroups for both pickers through `renderModeOptionView`. `waterAvailable` passes through `App` → `PlaygroundStage` → `PlayerPanel` / `PuzzlePanel` → both `PuzzleRenderModeSelect` renders. No new HUD control, no `data-hud-obstacle`, and no second particle canvas.

## Changed Literals and Tests (UI-SPEC "Decisions Affecting Existing Tests and Literals")

- `web/src/render/mode.ts`: `RenderMode` adds `"water"`. The `maybeParseRenderMode` allowlist adds `"water"`. The "Surface" group adds `{ value: "water", label: "Water" }` as its last entry, so it now has 4 options. `ParticleSurface` adds `"webgl-water"`. New: `export const WATER_UNAVAILABLE_LABEL = "Water (unavailable)";`. The `DEFAULT_RENDER_MODE` line is unchanged.
- `web/src/styles/player-viewport.css`: the selector is now `canvas.scene-canvas[data-particle-surface="webgl"], canvas.scene-canvas[data-particle-surface="webgl-water"]`. There is no `^=` selector.
- `PlayerPanel.tsx` and `PuzzleOptionsDrawer.tsx`: the `RENDER_MODE_GROUPS.map` loops become `<RenderModeOptions waterAvailable={...} />`. The `<label class="render-mode-control">Particles` wrapper, the `value` binding and the `maybeParseRenderMode` handler are unchanged.
- `web/tests/render-mode.test.ts`: the parse list, the surface table (now 4 modes, plus a check that the three non-surface modes need no WebGL) and the group value list now include Water. There are 5 new tests: Surface ends with Water, Water and shaded blob export as solid, the speed lane applies only to Water, the unavailable label and disabled state, and other options are never disabled. No existing assertion was weakened.
- `web/tests/canvas.test.ts`: 2 new Water `drawRenderFrame` cases: covered skips `paintMetaball`, and uncovered paints the soft blob.
- `web/tests/present-frame.test.ts` (new, 6 tests): Water draws → `webgl-water`, no blob, covered; Water fails → `webgl`; both fail → `canvas`, uncovered; shaded blob never tries Water; non-WebGL modes call neither and return `canvas`; exactly one 2D draw per frame.
- The protected specs and tests (`hud-safe-frame`, `shell`, `hash`, `portrait-bounds`, `readme-svg`) and the export pins (`plans.ts`, `demo-media/capture.ts`) were not touched.

## Final Line Counts

- `web/src/components/PlayerPanel.tsx`: 489 (was 493)
- `web/src/components/PuzzlePanel.tsx`: 460 (was 457; +3 lines for the `waterAvailable` prop and two passes)
- `web/src/player/scene-surface.ts`: 469 (was 455; limit 480)

## Verification

- `bun run typecheck` passes; `bun run test:unit` passes 119 files and 1,807 tests; `bun run build:app` succeeds; `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `CI=true bunx playwright test e2e/player.spec.ts`: 14/14 passed.
- Throwaway browser smoke check (not committed): Dam Break with stored `water` gives `data-particle-surface="webgl-water"` and `data-drawn-surface="webgl-water"`, the scene canvas background is `rgba(0, 0, 0, 0)`, and `data-speed-lane="absent"`. The lane reads absent because Plan 05 has not wired the capture flag yet.
- Acceptance greps: `RENDER_MODE_GROUPS.map` 0/0 in the two pickers; `waterAvailable` appears 3 times in `PuzzlePanel.tsx`; `class="particle-surface"` totals 2; no `data-hud-obstacle` added; `RenderModeOptions.tsx` has no WebGL/water/glsl import; `mode.ts` and `canvas.ts` have no WebGL/water/field-pass import.

## Deviations from Plan

None. The plan was executed as written. The only small difference is that `renderModeOptionView`'s return type is the exported named alias `RenderModeOptionView` instead of an inline object type; the shape is the same.

## Known Stubs

None. `data-speed-lane` reads `"absent"` for every mode until Plan 05 requests the lane through `wantsParticleSpeeds`. This is the planned sequencing, not a stub.

## Self-Check: PASSED

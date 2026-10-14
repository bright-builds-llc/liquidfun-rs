---
phase: 43-gpu-water-render-mode
plan: "03"
subsystem: web-render
tags: [typescript, webgl2, glsl, mrt, refraction, foam, capability-probe, water]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-14T00:10:18.956Z"
requires:
  - "43-01 RenderFrame.maybeParticleSpeeds"
  - "43-02 field-pass.ts shared per-canvas surface (particleSurfaceFor, pass slots, createFieldTarget(gl, 2))"
provides:
  - "web/src/render/webgl-water.ts: drawWater(canvas, frame, camera, devicePixelRatio, densityShading = true, maybeCreateBackdropCanvas = default), WaterBackdropCanvas"
  - "web/src/render/water-capability.ts: WaterSupportFacts, waterSupportFrom, liveWaterLimitsOk, probeWaterSupport, cachedWaterSupport, resetWaterSupportCacheForTests"
  - "web/src/render/water-backdrop.ts: BackdropCanvas, WaterBackdrop<C>, createWaterBackdrop"
  - "web/src/render/water-programs.ts: WaterPrograms, linkWaterPrograms (only ?raw importer)"
  - "web/src/render/water-uniforms.ts: WATER_* look constants, createWaterUniforms"
  - "web/src/render/water-shaders/*.glsl: four highp shaders"
  - "static-layer.ts exports sameContent and sameCameraView; canvas.ts exports CANVAS_COLOR, BASIN_STROKE_COLOR, outlineWidth"
  - "Fixture: backdropCanvasFactory, GlFixtureOptions.maybeIncompleteMrt"
affects:
  - "43-04 wiring: present-frame fallback chain calls drawWater first; pickers read cachedWaterSupport()"
  - "43-06 tuning: constants in water-uniforms.ts"
tech-stack:
  added: []
  patterns:
    - "GLSL in .glsl files imported with ?raw from a single module, keeping the Bun export graph free of ?raw"
    - "Revision-gated texture upload: a CPU raster bumps a revision, the GL pass uploads only when it differs"
    - "Pure capability decision over gathered facts plus a self-releasing, per-page cached probe"
key-files:
  created:
    - web/src/render/water-backdrop.ts
    - web/src/render/water-programs.ts
    - web/src/render/water-capability.ts
    - web/src/render/water-uniforms.ts
    - web/src/render/webgl-water.ts
    - web/src/render/water-shaders/water-point.vert.glsl
    - web/src/render/water-shaders/water-point.frag.glsl
    - web/src/render/water-shaders/water-shade.vert.glsl
    - web/src/render/water-shaders/water-shade.frag.glsl
    - web/tests/water-backdrop.test.ts
    - web/tests/water-capability.test.ts
    - web/tests/webgl-water.test.ts
  modified:
    - web/src/render/static-layer.ts
    - web/src/render/canvas.ts
    - web/tests/webgl-render-fixture.ts
decisions:
  - "D-07 refinement: refraction samples a water-owned stable-walls raster (water-backdrop.ts), not the Path2D static layer; it re-rasterizes only when a frame repeats and differs from the last raster, reusing static-layer's sameContent/sameCameraView; procedural backdrop otherwise"
  - "drawWater's backdrop factory is typed () => (BackdropCanvas & TexImageSource) | undefined, and createWaterBackdrop is generic, so the texture upload needs no cast in production"
  - "The backdrop texture starts as a 1x1 #071018 placeholder so the shader never samples an empty texture before the first walls raster"
  - "The walls raster strokes all segments as one path with one stroke() call; it is only ever seen refracted"
metrics:
  duration: 9min
  completed: 2026-10-14
  tasks: 3
  files: 15
---

# Phase 43 Plan 03: Water pass on the shared GL surface Summary

`drawWater` renders the Water look on the shared particle surface. A field pass splats into two RGBA16F attachments: the color field, and foam weighted by the speed lane. A highp shade pass then adds refraction of a stable-walls backdrop, Blinn-Phong highlights, Beer-Lambert tint in each particle's own color, and foam toward `#F4F7FA`. A pure capability decision with a cached, self-releasing probe feeds Plan 04's picker and fallback chain.

## What Was Built

- **Task 1 (39fd0571a):** `static-layer.ts` now exports `sameContent` and a `sameCameraView` extracted from `sameView`, so the definition of "static" lives in one place. `canvas.ts` exports `CANVAS_COLOR`, `BASIN_STROKE_COLOR` and `outlineWidth` and gains no imports. `water-backdrop.ts` keeps the previous frame's key (segment bits, camera copy, field size and pixel ratio) and copies it only when it changes. It rasterizes only when a frame equals the previous frame and differs from the last raster: it fills `#071018`, strokes the walls `#94A3B8` at the solid outline width (2 CSS px), and bumps `revision`. A frame that returns to the drawn state reuses its key, so it causes no redraw. Covered by 9 tests.
- **Task 2 (53b89a36e):** Four `#version 300 es` / `precision highp float;` shaders, each validated with `glslangValidator`. None contains `mediump` or `uTime`. `water-programs.ts` is the only `?raw` importer (`git grep -l "glsl?raw" -- web/src` lists only that file). `water-capability.ts` gathers facts in order, stops at the first missing one, builds a 4x4 2-attachment target, links the programs, and always calls `loseContext()` in `finally`. Covered by 21 tests, and a mutation check confirmed that the MRT-incomplete test fails when MRT completeness is ignored.
- **Task 3 (0c6bc8c8b):** `water-uniforms.ts` holds the look constants and a setter that writes only uniforms whose values changed. `webgl-water.ts` creates the `WATER_PASS` slot lazily and follows this order: canvas guard, surface, context, `sizeFor` (D-09), pass, 2-attachment resize (on failure it calls `disablePass` and returns false), shared upload, speed attribute, backdrop sync, field pass (`ONE, ONE`, POINTS), then the shade pass (units 0/1/2, TRIANGLE_STRIP). Covered by 13 tests. Mutation checks confirmed that both the revision gate and the constant-zero speed attribute are tested.

## Exported APIs

```ts
// webgl-water.ts
export type WaterBackdropCanvas = BackdropCanvas & TexImageSource;
export function drawWater(canvas: HTMLCanvasElement | undefined, frame: RenderFrame, camera: Camera,
  devicePixelRatio: number, densityShading = true,
  maybeCreateBackdropCanvas: () => WaterBackdropCanvas | undefined = defaultBackdropCanvas): boolean;

// water-capability.ts
export type WaterSupportFacts = { webgl2; floatTargets; maxDrawBuffers; highpFragment; mrtComplete; programsLinked };
export function waterSupportFrom(facts: WaterSupportFacts): boolean;
export function liveWaterLimitsOk(gl: WebGL2RenderingContext): boolean;
export function probeWaterSupport(createCanvas: () => HTMLCanvasElement | undefined): boolean;
export function cachedWaterSupport(createCanvas = defaultProbeCanvas): boolean;
export function resetWaterSupportCacheForTests(): void;

// water-backdrop.ts
export type BackdropCanvas = { width: number; height: number; getContext(contextId: "2d"): CanvasRenderingContext2D | null };
export type WaterBackdrop<C extends BackdropCanvas = BackdropCanvas> = { update(frame, camera, size): void; readonly revision: number; readonly matchesView: boolean; readonly maybeRaster: C | undefined; reset(): void };
export function createWaterBackdrop<C extends BackdropCanvas>(createCanvas: () => C | undefined): WaterBackdrop<C>;
```

## Starting Constants (Plan 06 tunes by eye)

| Constant | Value |
| --- | --- |
| `WATER_FOAM_SPEED_LOW` / `HIGH` | 3.0 / 7.5 m/s |
| `WATER_FOAM_MAX` | 0.85 |
| `WATER_REFRACTION_PX` | 10 CSS px; uniform `(10 * pixelRatio / width, 10 * pixelRatio / height)` |
| `WATER_NORMAL_SCALE` | 6.0 |
| `WATER_ABSORPTION` | 0.9 |
| `WATER_SHININESS` / `WATER_SPECULAR_STRENGTH` | 48 / 0.55 |
| `WATER_LIGHT_DIR` | normalize(-0.35, 0.55, 0.76) |
| `WATER_FOAM_COLOR` / `WATER_BACKGROUND` | `#F4F7FA` / `#071018` |
| Edge | `smoothstep(0.42, 0.62, sum.a)` (UI-D45); no time uniform (UI-D46) |

## File Line Counts

`webgl-water.ts` 347, `water-backdrop.ts` 211, `water-capability.ts` 139, `water-uniforms.ts` 147, `water-programs.ts` 35. Shaders: point vert 28, point frag 23, shade vert 11, shade frag 63. Tests: `webgl-water.test.ts` 228, `water-backdrop.test.ts` 174, `water-capability.test.ts` 235. `webgl-render-fixture.ts` is 198 lines, `static-layer.ts` 181 and `canvas.ts` 402. Every new file is at most 400 lines.

## Verification

- `bunx vitest run tests/webgl-water.test.ts tests/water-backdrop.test.ts tests/water-capability.test.ts tests/field-pass.test.ts tests/webgl-render-cache.test.ts`: 5 files, 67 tests passed.
- `cd web && bun run typecheck && bun run test:unit`: 118 files, 1794 tests passed. `bun scripts/bright-builds-check.ts all`: findings=0.
- `webgl-render-cache.test.ts` is unchanged (`git diff --quiet HEAD`). A canvas that draws only shaded blob still creates 2 programs and makes 8 uniform lookups (new test).
- Import graph: `git grep -nE "(import|from) .*(water-(backdrop|capability|programs|uniforms|shaders)|webgl-water|field-pass)"` over `canvas.ts`, `mode.ts`, `static-layer.ts`, `web/scripts` and `web/src/export` finds nothing. The plan's looser pattern `water-|webgl-water|field-pass` matches only the `water-wheel` scene id in `web/scripts`, and none of those matches is an import.
- `glslangValidator -S vert|frag` passes for all four shaders.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The backdrop would redraw after returning to the drawn state**
- **Found during:** Task 1
- **Issue:** In a sequence A (rasterized), B, A, A, the third frame stored a fresh snapshot of A. On the fourth frame the previous key was no longer the rasterized key object, so the backdrop redrew an identical raster and bumped the revision, which caused an extra texture upload.
- **Fix:** When the current frame equals the rasterized key, the backdrop reuses that key as the previous one.
- **Commit:** 39fd0571a

**2. [Rule 3 - Blocking] Typing the texture source**
- **Issue:** `texImage2D` needs a `TexImageSource`, and the plan's `BackdropCanvas` type does not satisfy it.
- **Fix:** `createWaterBackdrop` is now generic over its canvas type. `drawWater` takes `() => WaterBackdropCanvas | undefined` (`BackdropCanvas & TexImageSource`) and defaults to `document.createElement("canvas")` when a DOM exists. Production code has no cast; the test fake is cast once.
- **Commit:** 0c6bc8c8b

**3. [Minor] Fixture helpers**
- `backdropCanvasFactory` was added to `webgl-render-fixture.ts` in Task 1 rather than Task 2, because both the backdrop and drawWater tests use it. `GlFixtureOptions.maybeIncompleteMrt` reports a framebuffer as incomplete when its last attachment is `COLOR_ATTACHMENT1`, so a single-target shaded blob stays complete. The capability test switched to this option in the Task 3 commit. Existing defaults did not change.

**4. [Minor] 1x1 placeholder for the backdrop texture**
- The backdrop texture gets a `#071018` 1x1 image at creation, so the shade pass never samples an incomplete texture, which would trigger Chrome render warnings. The tests count only uploads from a canvas source (the 6-argument `texImage2D`).

## Notes

- Task commits are `feat` rather than separate RED/GREEN commits: tests and implementation were written together. Each test file ran before its commit, and mutation checks confirmed that the key tests catch broken implementations.
- REQUIREMENTS: WATER-01 and WATER-02 stay Pending. This plan delivers the Water pass and the inputs to capability and fallback. The mode, the pickers, the fallback chain and the browser proof land in Plans 04 to 06. Marking either requirement complete now would overstate the evidence. This follows the 43-02 precedent.

## Known Stubs

None. `PLACEHOLDER_TEXEL` in `webgl-water.ts` is an intentional initial texture image, not a data stub. Production code does not call `drawWater` until Plan 04 wires it into `present-frame.ts`.

## Self-Check: PASSED

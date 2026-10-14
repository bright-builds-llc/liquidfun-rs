---
phase: 43-gpu-water-render-mode
plan: "02"
subsystem: web-render
tags: [typescript, webgl2, field-pass, half-float, context-loss, shaded-blob]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-14T00:02:10.615Z"
requires:
  - "Existing shaded-blob pipeline (webgl-particles.ts, blob-uniforms.ts, particle-metadata.ts, projected-particle.ts)"
provides:
  - "web/src/render/gl-program.ts: compileShader, linkProgram, requireBuffer"
  - "web/src/render/field-pass.ts: PARTICLE_CONTEXT_ATTRIBUTES, supportsHalfFloatTargets, FieldTarget, createFieldTarget, resizeFieldTarget, SharedParticleBuffers, FieldSize, ParticleGlSurface, particleSurfaceFor, watchParticleSurface"
  - "Shaded blob accepts EXT_color_buffer_half_float (D-13); STATE.md half-float todo implemented"
  - "Mock GL fixture options: maybeExtensions, maybeFramebufferStatus, maybeMaxDrawBuffers, maybeHighpPrecision, WEBGL_lose_context, removeEventListener, contextRequests"
affects:
  - "43-03 water pass: surface.pass(WATER_KEY, ...) on the shared buffers, createFieldTarget(gl, 2) for MRT"
  - "43-04 wiring: assignParticleSurface calls watchParticleSurface(canvas, () => paintHeldFrame(session))"
tech-stack:
  added: []
  patterns:
    - "Per-canvas surface in a WeakMap with reset listeners registered at creation; repaint listeners attach later and run after the reset"
    - "Pass slots keyed by module-private symbols; an undefined create result disables only that key until reset"
key-files:
  created:
    - web/src/render/gl-program.ts
    - web/src/render/field-pass.ts
    - web/tests/field-pass.test.ts
  modified:
    - web/src/render/webgl-particles.ts
    - web/tests/webgl-render-fixture.ts
decisions:
  - "Particle metadata cache lives with the shared buffers, so a reset re-uploads radii and colors exactly as before (restore upload count stays 4)"
  - "Buffer creation failure is not cached; only a failed pass create or resize marks a key unavailable"
  - "resizeFieldTarget names COLOR_ATTACHMENT0/1 explicitly instead of COLOR_ATTACHMENT0 + index"
metrics:
  duration: 5min
  completed: 2026-10-14
  tasks: 2
  files: 5
---

# Phase 43 Plan 02: Shared per-canvas GL surface Summary

Shaded blob now draws through a per-canvas WebGL2 surface in `field-pass.ts`. The surface accepts `EXT_color_buffer_float` or `EXT_color_buffer_half_float` and still lets `checkFramebufferStatus` decide (D-13). Each pass has its own availability. `watchParticleSurface` repaints only after the loss/restore reset (D-14). The shaders, uniform lookups, program count and upload counts are unchanged, and `webgl-render-cache.test.ts` passes without edits.

## What Was Built

- **Task 1 (cb9a0ad07):** `gl-program.ts` now holds `compileShader`, `linkProgram` and `requireBuffer`, moved without changes. `field-pass.ts` owns the context, the half-float verdict, the shared position/radius/color buffers with their metadata cache, the scratch arrays, canvas sizing with `cappedDevicePixelRatio`, and the pass slots. `webgl-particles.ts` keeps its four inline shaders byte for byte (checked with `diff` against HEAD). It builds a `BLOB_PASS` with a 1-attachment `FieldTarget` and draws in the same order as before. The fixture gained the optional capabilities and `WEBGL_lose_context`, and existing constant values did not shift.
- **Task 2 (aabbf02e6):** `watchParticleSurface(canvas, repaint)` calls `particleSurfaceFor(canvas)` before it adds its own lost/restored listeners, and it returns a detach function. Six tests cover the lost repaint, the restore repaint, ordering when watched before the first draw, detach, pass isolation and rebuild after reset.

## field-pass.ts public API (as built)

```ts
export const PARTICLE_CONTEXT_ATTRIBUTES: Readonly<WebGLContextAttributes>;
export function supportsHalfFloatTargets(gl: WebGL2RenderingContext): boolean;
export type FieldTarget = { readonly framebuffer: WebGLFramebuffer; readonly textures: readonly WebGLTexture[]; width: number; height: number };
export function createFieldTarget(gl: WebGL2RenderingContext, attachmentCount: 1 | 2): FieldTarget | undefined;
export function resizeFieldTarget(gl: WebGL2RenderingContext, target: FieldTarget, width: number, height: number): boolean;
export type SharedParticleBuffers = { readonly gl; readonly position; readonly radius; readonly color; readonly maxPointSize: number };
export type FieldSize = { readonly width: number; readonly height: number; readonly pixelRatio: number };
export interface ParticleGlSurface {
  context(): WebGL2RenderingContext | undefined;
  buffers(): SharedParticleBuffers | undefined;
  sizeFor(camera: Camera, devicePixelRatio: number): FieldSize | undefined;
  upload(frame: RenderFrame, camera: Camera): number; // 0 before buffers exist
  pass<T>(key: symbol, create: (buffers: SharedParticleBuffers) => T | undefined): T | undefined;
  disablePass(key: symbol): void;
}
export function particleSurfaceFor(canvas: HTMLCanvasElement): ParticleGlSurface;
export function watchParticleSurface(canvas: HTMLCanvasElement, repaint: () => void): () => void;
```

A new `createFieldTarget` result has size 0 x 0, so the first draw always calls `resizeFieldTarget`. The recorded size changes only when the framebuffer is complete. `drawBuffers` is called only for 2 attachments.

## Verification

- `bunx vitest run tests/field-pass.test.ts tests/webgl-render-cache.test.ts`: 24 passed (16 + 8).
- RED checks: against the HEAD `webgl-particles.ts`, the half-float draw test failed (1 of 10). With `particleSurfaceFor(canvas)` commented out of `watchParticleSurface`, the "watched before the first draw" ordering test failed. Both changes were restored.
- `cd web && bun run typecheck && bun run test:unit`: 115 files, 1751 tests passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet a490fc708 -- web/tests/webgl-render-cache.test.ts`: unchanged. `grep -c "precision mediump float" web/src/render/webgl-particles.ts` = 2. `grep -c 'EXT_color_buffer_float") === null' web/src/render/webgl-particles.ts` = 0.
- `canvas.ts`, `mode.ts` and `static-layer.ts` import nothing from `field-pass`, `gl-program` or any WebGL module.
- Final line counts: `webgl-particles.ts` 224 (was 478), `field-pass.ts` 380 (≤ 400), `gl-program.ts` 58, `field-pass.test.ts` 244.

## Notes

- The STATE.md pending todo (accept `EXT_color_buffer_half_float`) is now implemented for shaded blob through the shared surface. The iPhone check stays a non-blocking D-20 UAT item.
- Behavior change from the old code, intended: if `cappedDevicePixelRatio` throws, the draw now returns false before building programs. Previously the programs were built first. The fallback result is the same.

## Deviations from Plan

None. The plan was executed as written. I wrapped a few lines by hand to match the existing 80-column style, because the repo has no formatter configured for `web/`.

## Requirement Status

WATER-02 is still Pending in REQUIREMENTS.md. This plan delivers its GL-layer part: half-float acceptance, per-pass fallback and ordered restore repaint. Plans 03, 04 and 06 also list WATER-02, and the fallback chain, the drawn-surface attribute and the browser proof land there. Marking it complete now would overstate the evidence.

## Known Stubs

None.

## Self-Check: PASSED

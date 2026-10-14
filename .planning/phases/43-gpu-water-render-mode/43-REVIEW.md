---
phase: 43-gpu-water-render-mode
reviewed: 2026-10-14T00:45:12Z
depth: standard
files_reviewed: 56
files_reviewed_list:
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/frame/speeds.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun-wasm/src/session/speeds.rs
  - crates/liquidfun-wasm/src/session/speeds/tests.rs
  - web/e2e/water-helpers.ts
  - web/e2e/water.spec.ts
  - web/package.json
  - web/src/App.tsx
  - web/src/components/PlayerPanel.tsx
  - web/src/components/PlaygroundStage.tsx
  - web/src/components/PuzzleOptionsDrawer.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/RenderModeOptions.tsx
  - web/src/physics/frame-speeds.ts
  - web/src/physics/frame.ts
  - web/src/physics/puzzle-session.ts
  - web/src/physics/session.ts
  - web/src/physics/simulation-worker.ts
  - web/src/physics/worker-messages.ts
  - web/src/physics/worker-session.ts
  - web/src/player/puzzle-host.ts
  - web/src/player/scene-lifecycle.ts
  - web/src/player/scene-surface.ts
  - web/src/render/canvas.ts
  - web/src/render/field-pass.ts
  - web/src/render/gl-program.ts
  - web/src/render/mode.ts
  - web/src/render/present-frame.ts
  - web/src/render/static-layer.ts
  - web/src/render/water-backdrop.ts
  - web/src/render/water-capability.ts
  - web/src/render/water-programs.ts
  - web/src/render/water-shaders/water-point.frag.glsl
  - web/src/render/water-shaders/water-point.vert.glsl
  - web/src/render/water-shaders/water-shade.frag.glsl
  - web/src/render/water-shaders/water-shade.vert.glsl
  - web/src/render/water-uniforms.ts
  - web/src/render/webgl-particles.ts
  - web/src/render/webgl-water.ts
  - web/src/styles/player-viewport.css
  - web/tests/canvas.test.ts
  - web/tests/field-pass.test.ts
  - web/tests/frame-speeds.test.ts
  - web/tests/present-frame.test.ts
  - web/tests/puzzle-host-fixture.ts
  - web/tests/puzzle-session.test.ts
  - web/tests/render-mode.test.ts
  - web/tests/session.test.ts
  - web/tests/water-backdrop.test.ts
  - web/tests/water-capability.test.ts
  - web/tests/webgl-render-fixture.ts
  - web/tests/webgl-water.test.ts
  - web/tests/worker-messages.test.ts
  - web/tests/worker-session.test.ts
findings:
  critical: 0
  warning: 0
  info: 7
  total: 7
status: issues_found
---

# Phase 43: Code Review Report

**Reviewed:** 2026-10-14T00:45:12Z
**Depth:** standard
**Files Reviewed:** 56
**Status:** issues_found

## Summary

Reviewed the Phase 43 diff (`2653fd987..HEAD`) for the GPU Water render mode. The diff covers:

- the optional Rust per-particle speed lane (`capture_frame_with_speeds`, `FrameData::set_particle_speeds`, `ProofFrame::particleSpeeds`);
- its fail-closed TypeScript parser and worker protocol flag (`speeds`) with transferable validation;
- the refactor of shaded blob onto a shared per-canvas `ParticleGlSurface` (`field-pass.ts`) with per-pass availability and context loss/restore reset;
- the new Water pass (MRT RGBA16F field and foam, walls backdrop raster, shade shader);
- the capability probe and picker presentation ("Water (unavailable)");
- the Water -> shaded blob -> canvas fallback chain with `data-drawn-surface` and `data-speed-lane` instrumentation;
- the Chromium e2e proofs.

Overall the change is careful and well structured.

- The speed lane is index-aligned with positions: `fillProjectedParticleArrays` uses an infinite cap, so stride is 1 and `count == particleCount`. The lane is validated identically on both sides of the boundary.
- Each lane of a worker frame owns its buffer.
- `bindSpeeds` avoids an out-of-range enabled attribute by falling back to a constant generic attribute.
- Context-loss listeners are registered in an order that guarantees the reset runs before the repaint.
- The fallback chain correctly drives `webglCovered`, the 2D clear, and the transparent-overlay CSS.

I found no correctness, security, or crash defects.

Verification evidence:

- `npx tsc --noEmit` in `web/` exited cleanly.
- `npx vitest run` passed 185 of 185 tests across the 12 reviewed unit test files.
- Rust and e2e suites were not re-run by the reviewer.

The remaining findings are Info-level: robustness edge cases, leaks on failure paths, duplicated constants, and naming conventions.

## Info

### IN-01: Speed lane is computed and transferred every frame even when Water can never draw

**File:** `web/src/render/mode.ts:227-229` (consumers `web/src/player/scene-lifecycle.ts:146`, `web/src/player/puzzle-host.ts:109-111`)
**Issue:** `wantsParticleSpeeds` follows the requested mode only. Suppose a user has `water` stored on a device where the probe failed. That situation is explicitly supported, because the disabled option stays selected. Every capture then still runs `particle_speeds` in Rust and copies up to 16384 floats out of WASM. Every frame is then validated element by element in `parseParticleSpeeds`, and on the Tesla Valve worker path it is also transferred. The frame is always drawn by shaded blob, which ignores the lane. Behavior is correct; the work is pure waste. D-11 documents the behavior, but it does not justify the cost on an incapable device.
**Fix:** Gate the request on the page probe result as well, for example `() => wantsParticleSpeeds(mode()) && cachedWaterSupport()`. Alternatively, record that the cost is accepted for incapable devices next to D-11.

### IN-02: Speed-lane value errors reuse `FrameError::InvalidDiagnostic`

**File:** `crates/liquidfun-wasm/src/frame/speeds.rs:18-23`
**Issue:** NaN, infinite, or negative speeds are reported as `InvalidDiagnostic`. However, `FrameError` already has `NonFiniteValue`, and the speed lane is a per-particle lane, not a scalar diagnostic. The caller maps every error to `FrameCaptureFailed`, so behavior is unaffected. The variant does mislead anyone debugging a rejected lane, and the unit test enshrines it (`speeds.rs:85`).
**Fix:** Return `FrameError::NonFiniteValue` for non-finite values. For negatives, either reuse it or add a dedicated variant such as `NegativeSpeed`, then update the test.

### IN-03: GL objects leak on partial-failure and disabled-pass paths

**File:** `web/src/render/webgl-water.ts:146-167`, `web/src/render/webgl-water.ts:335-337`, `web/src/render/field-pass.ts:55-66`, `web/src/render/gl-program.ts:32-40`
**Issue:** Each path below leaves GL objects alive until the next context reset:

- **`createWaterPass`:** If any later step fails, it returns `undefined` without deleting the linked programs, the buffers, the VAOs, the field target, or the backdrop texture.
- **`createFieldTarget`:** It returns `undefined` after creating the framebuffer when a texture allocation fails, and does not delete that framebuffer.
- **`linkProgram`:** It returns early without deleting an already compiled vertex shader when the fragment stage fails. It also does not delete either shader when `createProgram()` returns `null`.
- **`disablePass(WATER_PASS)` after a failed MRT resize:** It drops the pass from the map without freeing its GPU allocations. For a 2-attachment RGBA16F target at canvas size, those allocations are significant.

In practice these paths are rare, and most of them coincide with context loss.
**Fix:** Add a small `deleteAll` cleanup on each failure branch. For example, give `WaterPass` a `dispose(gl)` method and call it from `disablePass` through an optional pass disposer. In `linkProgram`, delete whichever shader compiled before returning `undefined`.

### IN-04: Canvas background color is defined three times

**File:** `web/src/render/canvas.ts:29-30`, `web/src/render/water-uniforms.ts:47`, `web/src/render/webgl-water.ts:36`
**Issue:** The doc comment on `CANVAS_COLOR` says it is "shared with the water backdrop and shade pass". However, the shade pass uses an independent literal `WATER_BACKGROUND = [7/255, 16/255, 24/255]`, and the placeholder texel repeats `[7, 16, 24, 255]` a third time. No test ties them together. If the theme color changes in one place only, the water edge (`mix(uBackground, water, edge)`) and the 2D background will visibly disagree.
**Fix:** Derive both values from `CANVAS_COLOR`, for example with a tiny `hexToRgb` helper, or add a unit test asserting that the three agree. Also correct the comment in `canvas.ts`.

### IN-05: Probe verdict caches transient failures for the page lifetime

**File:** `web/src/render/water-capability.ts:106-111`, `web/src/render/water-capability.ts:126-134`
**Issue:** `cachedWaterSupport` stores `false` in two cases: when `getContext("webgl2")` returns `null`, and when the probe context is already lost at startup. Both can be transient, for example after a GPU process reset or when the browser is at its context limit. Water then stays "unavailable" in both pickers until a reload, even after the live particle surface recovers, and the D-12 fallback chain would already handle a runtime failure.
**Fix:** Cache only a positive verdict, or a negative verdict derived from capability facts such as limits, extensions, or link failure. Treat `null` or a lost context as "unknown, re-probe later", for example on the next picker open.

### IN-06: `WaterBackdrop.reset()` has no production caller

**File:** `web/src/render/water-backdrop.ts:40-41`, `web/src/render/water-backdrop.ts:205-209`
**Issue:** After a context restore, the whole backdrop is recreated together with the pass (`createWaterPass`). As a result, `reset()` is exercised only by `web/tests/water-backdrop.test.ts:166`. This dead public surface implies a lifecycle that does not exist.
**Fix:** Remove `reset()` and its test, or call it where it is meant to apply. One example is when `disablePass` drops the water pass.

### IN-07: `Option` parameter lacks the `maybe_` prefix

**File:** `crates/liquidfun-wasm/src/lib.rs:173`
**Issue:** The project conventions in CLAUDE.md ("Prefix `Option` types with `maybe_`") call for this parameter to be named `maybe_particle_speeds: Option<bool>`, but it is named `particle_speeds`. The TypeScript side already uses `maybeParticleSpeeds`.
**Fix:** Rename the parameter to `maybe_particle_speeds`. The JS-facing name does not change, because wasm-bindgen positional arguments are unaffected.

---

_Reviewed: 2026-10-14T00:45:12Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

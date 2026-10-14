---
phase: 43-gpu-water-render-mode
fixed_at: 2026-10-14T01:01:07Z
review_path: .planning/phases/43-gpu-water-render-mode/43-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 6
skipped: 1
status: partial
---

# Phase 43: Code Review Fix Report

**Fixed at:** 2026-10-14T01:01:07Z
**Source review:** .planning/phases/43-gpu-water-render-mode/43-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7
- Fixed: 6
- Skipped: 1

**Verification after all fixes:**
- `npx tsc --noEmit` in `web/` exited cleanly.
- `npx vitest run` in `web/`: 119 files, 1840 of 1840 tests passed. That count includes the new IN-01, IN-03 and IN-04 tests and excludes the removed IN-06 test.
- `cargo test -p liquidfun-wasm`: 600 passed, 0 failed, 25 ignored.
- `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings` and `cargo fmt --all --check` were clean.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- File budgets: session.rs 600 of 600, frame.rs 557 of 560, scene-surface.ts 469 of 480, field-pass.ts 390 of 400.
- `crates/liquidfun` was not touched. The Rust edits only change an error variant and a parameter name, so they cannot change scene fingerprints.
- The Playwright e2e suite was re-run by the orchestrator after all fixes: `bun run build:app` then `CI=true bun run test:water` passed 13/13 (2026-10-14).

## Fixed Issues

### IN-02: Speed-lane value errors reuse `FrameError::InvalidDiagnostic`

**Files modified:** `crates/liquidfun-wasm/src/frame.rs`, `crates/liquidfun-wasm/src/frame/speeds.rs`
**Commit:** 60ada549b
**Applied fix:** `set_particle_speeds` now returns `FrameError::NonFiniteValue` for NaN or infinite speeds. Negative speeds return a new `FrameError::NegativeSpeed` variant. The combined unit test was split into two tests that assert each variant.

### IN-07: `Option` parameter lacks the `maybe_` prefix

**Files modified:** `crates/liquidfun-wasm/src/lib.rs`
**Commit:** 17a1a1d45
**Applied fix:** Renamed the `capture_frame` parameter to `maybe_particle_speeds: Option<bool>`. The JS call shape is unchanged because wasm-bindgen arguments are positional, and no generated bindings are committed.

### IN-04: Canvas background color is defined three times

**Files modified:** `web/src/render/canvas.ts`, `web/src/render/water-uniforms.ts`, `web/src/render/webgl-water.ts`, `web/tests/canvas.test.ts`
**Commit:** 0f6381ad3
**Applied fix:** `canvas.ts` now exports `CANVAS_RGB`, which is parsed from `CANVAS_COLOR` by a small `hexRgb` helper. `WATER_BACKGROUND` and the backdrop `PLACEHOLDER_TEXEL` are both derived from it. A new unit test pins the WebGL values to the 2D canvas color. The comment in `canvas.ts` is now accurate as written.

### IN-01: Speed lane is computed and transferred every frame even when Water can never draw

**Files modified:** `web/src/render/mode.ts`, `web/src/player/scene-lifecycle.ts`, `web/src/player/puzzle-host.ts`, `web/tests/render-mode.test.ts`
**Commit:** 4abd96a1d
**Status:** fixed (verified: `water.spec.ts` 13/13 after the fix)
**Applied fix:** `wantsParticleSpeeds(mode, waterAvailable)` now returns true only when the requested mode is Water and the page probe found Water available. Both call sites pass `cachedWaterSupport()`, which App.tsx has already cached at mount. A new unit test covers the unavailable case.
**Verification:** the orchestrator rebuilt the app and ran `CI=true bun run test:water` after all fixes; 13/13 passed, including the unavailable-picker and canvas-fallback cases.

### IN-03: GL objects leak on partial-failure and disabled-pass paths

**Files modified:** `web/src/render/gl-program.ts`, `web/src/render/field-pass.ts`, `web/src/render/webgl-water.ts`, `web/tests/webgl-water.test.ts`
**Commit:** 692650919
**Applied fix:**
- **`linkProgram`:** It deletes any compiled shader when the other stage fails or `createProgram()` returns null. It no longer creates a program when a stage failed.
- **`createFieldTarget`:** It deletes the framebuffer and any textures it already made when a texture allocation fails. The new exported `deleteFieldTarget` does that work.
- **`createWaterPass`:** It frees every object it made through `deleteWaterObjects` when a later step fails. `WaterPass` now keeps `clipBuffer` so that buffer can be freed too.
- **`drawWater`:** It calls `deleteWaterPass` before `surface.disablePass(WATER_PASS)` when the MRT resize fails.

Three regression tests cover the incomplete MRT, a null vertex array and a fragment compile failure. All three failed against the pre-fix source.

### IN-06: `WaterBackdrop.reset()` has no production caller

**Files modified:** `web/src/render/water-backdrop.ts`, `web/tests/water-backdrop.test.ts`
**Commit:** 95576cf7d
**Applied fix:** Removed `reset()` from the `WaterBackdrop` type and implementation, along with its only test. A context restore and the IN-03 disabled-pass path both drop the whole backdrop with its pass.

## Skipped Issues

### IN-05: Probe verdict caches transient failures for the page lifetime

**File:** `web/src/render/water-capability.ts:106-111`, `web/src/render/water-capability.ts:126-134`
**Reason:** skipped as a judgment call that conflicts with locked plan decisions. UI-D48 locks availability as read once in App.tsx and passed down as a plain boolean. UI-D50 locks "no runtime flip" of the picker. Re-probing would require a reactive availability signal in both pickers, which is a behavior change those decisions rule out. Caching only positive verdicts would also make `cachedWaterSupport()` re-probe on every call after a transient failure. Since IN-01, every frame capture calls it, so each frame would create and lose a WebGL context. The D-12 runtime fallback chain still draws Water on a capable live context even when the page probe failed.
**Original issue:** `cachedWaterSupport` stores `false` when `getContext("webgl2")` returns null or the probe context is already lost. Both can be transient, so Water stays "unavailable" in both pickers until a reload.

---

_Fixed: 2026-10-14T01:01:07Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

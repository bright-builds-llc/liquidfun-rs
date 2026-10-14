---
phase: 43-gpu-water-render-mode
plan: "06"
subsystem: web-render
tags: [playwright, webgl2, water, e2e, tuning, phase-gates, fingerprints]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-14T00:38:15.764Z"
requires:
  - "43-01 speed lane (captureFrame(true), RenderFrame.maybeParticleSpeeds)"
  - "43-02 field-pass.ts shared surface and loss/restore watcher"
  - "43-03 drawWater, cachedWaterSupport, water-uniforms.ts"
  - "43-04 data-drawn-surface / data-speed-lane, RenderModeOptions, D-12 chain"
  - "43-05 per-capture speed flag in scene, puzzle and worker owners"
provides:
  - "web/e2e/water-helpers.ts: storeRenderMode, forceWaterUnavailable, forceNoFloatTargets, loseParticleContext, restoreParticleContext, drawnSurface, openPuzzleOptions, particlesSelect, chooseRenderMode"
  - "web/e2e/water.spec.ts: 13 D-19 browser proofs"
  - "web/package.json test:water script"
  - "Tuned water look constants in web/src/render/water-uniforms.ts"
  - "target/phase43/checks-06.{sh,results,log}, after-final.txt, screens/"
affects:
  - "/gsd-verify-work for Phase 43: D-20 iPhone UAT and a real-GPU look check remain"
tech-stack:
  added: []
  patterns:
    - "Init-script prototype patches (getParameter, getExtension) force each fallback branch in a real browser"
    - "Loss/restore through a WEBGL_lose_context object kept on window across evaluates"
key-files:
  created:
    - web/e2e/water-helpers.ts
    - web/e2e/water.spec.ts
  modified:
    - web/package.json
    - web/src/render/water-uniforms.ts
decisions:
  - "Tuned look: foam 3.5/9.0 m/s, shininess 64, specular 0.4; normal scale, refraction, absorption, foam max and light direction keep their starting values"
  - "Option disabled state is asserted on the DOM disabled property: Playwright toBeDisabled reports a natively disabled <option> in a closed select as enabled"
  - "bun-graph-clean matches module paths (a slash before the module name) because mode.ts must name the \"webgl-water\" surface literal (D-04)"
metrics:
  duration: 19min
  completed: 2026-10-14
  tasks: 3
  files: 4
---

# Phase 43 Plan 06: Browser proof, look tuning and phase gates Summary

Chromium now has 13 Playwright tests for Water mode. They cover scenes, the Tesla Valve worker and puzzles, the overlay layer, run determinism, context loss and restore, the unavailable and no-float fallbacks, and the SVG export. None of them diffs shader pixels. I tuned the look constants over two rounds by viewing screenshots. All 27 phase gates passed on the first full run, and all 25 scene fingerprints match the phase base.

## What Was Built

- **Task 1 (d0987eae8, 551484b0c):** `water-helpers.ts` adds a first-navigation-only render-mode seed. It also adds init-script patches: one makes `MAX_DRAW_BUFFERS` report 1, and one hides both color-buffer float extensions. Loss and restore go through a `WEBGL_lose_context` object stored on `window`. A Particles picker opens the scene controls or the "Puzzle options" drawer as needed. `water.spec.ts` holds 13 tests (listed below). `test:water` is inserted after `test:puzzle`, and no other line of `package.json` changed. The second commit makes the drawer screenshot wait until `data-transitioning` clears. Without that wait, the 500 ms slide-in left the picker off screen in the PNG.
- **Task 2 (4f68a6aab):** Two tuning rounds on Dam Break, Fountain and First Pour. Only constants changed, plus one mis-indented line in `water-uniforms.ts`.
- **Task 3 (no commit; evidence only):** `target/phase43/checks-06.sh` runs the 27 gates. The first full run passed all 27 with `exit=0`.

## Browser Proofs (`CI=true bun run test:water`: 13 passed)

| # | Test | Proves |
| --- | --- | --- |
| 1 | Water in Dam Break | `data-render-mode="water"`, `data-drawn-surface="webgl-water"`, `data-speed-lane="present"`, `particle-surface[data-active="true"]`, scene canvas `background-color: rgba(0, 0, 0, 0)`, one enabled "Water" option (WATER-01, D-15, D-16) |
| 2 | Water back to Shaded blob | `webgl` and `data-speed-lane="absent"` |
| 3 | Tesla Valve worker | `data-simulation-backend="worker"`, then lane `present` and `webgl-water` (D-11 worker owner) |
| 4 | Puzzle overlays | Building phase in Water draws `webgl-water`. The 2D layer SHA-256 is equal after switching to Shaded blob (`webgl`). The 2D layer has more than 0 pixels with alpha > 0 (D-16) |
| 5 | Same run in every mode | First-pour seam solution, fresh contexts: Water lane `present`, Shaded blob lane `absent`, RunFingerprint deep-equal (D-11) |
| 6 | Loss/restore while paused | Dam Break in Water, paused: `canvas` after `loseContext()`, `webgl-water` after `restoreContext()`, still paused at the same step (WATER-02, D-14) |
| 7 | Loss/restore while building | Same on the First Pour building phase |
| 8 | Unavailable | Under the forced 1 draw buffer with stored `water`: the scene and "Puzzle options" selects each have a disabled "Water (unavailable)" option, no "Water" option and value `water`, and both draw `webgl` (D-05, UI-D49) |
| 9 | No float targets | `canvas` in Water and in Shaded blob; scene-canvas pixel (0, 0) alpha 255 |
| 10 | Export unchanged | 1 s Dam Break SVG SHA-256 equal in Shaded blob and Water (WATER-03, D-17) |
| 11 | Form-factor row | `?shell=form-factor` puzzle control-row Particles select has an enabled "Water" |
| 12 | Fountain | `webgl-water`, lane present, screenshot |
| 13 | Phone run | 390x844 First Pour solution past step 150: `webgl-water`, lane present, screenshot |

- **RunFingerprint (D-11):** Water `{outcomeStep 201, decidedStep 262, peakHeld 811, heldHash 0cf8c2703dcd8fac}` = Shaded blob, which also equals the Phase 42 sound on/off fingerprint.
- **SVG export hash (D-17):** both modes give `ff92a2ad59cb03b85d3daa0e86c65fd320190d0ca64f27a8dd1ef03363dd48f9` (byte-identical, no metadata stripping needed; Research A7 held).
- Annotations: `target/phase43/water-annotations.json`.

## Tuned Constants

Scenes viewed: Dam Break and Fountain at 1280x800 after 3 s of play, and First Pour at 390x844 mid-run with the solution. All were rendered in headless Chromium with SwiftShader, which is only a guide (D-20 covers real devices). Round 0, 1 and 2 PNGs are kept as `target/phase43/screens/round{0,1}-*.png` and the final `*.png`.

| Constant | Start | Final | Why |
| --- | --- | --- | --- |
| `WATER_FOAM_SPEED_LOW` | 3.0 m/s | 3.5 m/s | Fountain floor sheets were near-white; now they stay blue at the edges |
| `WATER_FOAM_SPEED_HIGH` | 7.5 m/s | 9.0 m/s | Only the jet column and fastest spray saturate |
| `WATER_FOAM_MAX` | 0.85 | 0.85 | Already keeps foam below flat white |
| `WATER_REFRACTION_PX` | 10 | 10 | unchanged |
| `WATER_NORMAL_SCALE` | 6.0 | 6.0 | Round 1 tried 4.0 with no visible change (per-particle field ripple dominates), so I reverted it |
| `WATER_ABSORPTION` | 0.9 | 0.9 | unchanged |
| `WATER_SHININESS` | 48 | 64 | Narrower lobe, fewer glitter pixels |
| `WATER_SPECULAR_STRENGTH` | 0.55 | 0.4 | Pooled-water glitter becomes small glints |
| `WATER_LIGHT_DIR` | normalize(-0.35, 0.55, 0.76) | unchanged | |

Pass criteria (final round):

- Slow pooled water is clear and tinted, with visible highlights and no foam. Dam Break's settled pool is particle blue with small white glints and no foam. Against the dark backdrop, "clear" mostly reads as the darker transmitted tint.
- Fountain jets and the Dam Break front show white spray. The fountain column and the arc tops show white spray. At 3 s the Dam Break front has already slowed against the far wall, so it shows little foam. Earlier frames were not captured.
- The cup, target line and meter stay readable over foam. In the First Pour portrait shot, the cup walls, the dashed yellow target and Brim lines and the 61% meter are all crisp above the water.
- Water keeps each particle's hue. Every scene stays in its particle blue.
- Nothing is flat white. Foam is capped at 0.85 toward `#F4F7FA`, and the brightest jet core still shows texture.
- Residual: the gradient is a one-texel difference of the kernel sum, so packed interiors keep some per-particle glitter. Smoothing it would take a shader change, which is out of scope for constant-only tuning. This goes to the real-GPU look check.

Screenshots: `target/phase43/screens/dam-break-water.png`, `fountain-water.png`, `puzzle-water-portrait.png`, `water-unavailable-drawer.png`.

## Phase Gates (`target/phase43/checks-06.results`, first full run, HEAD 4f68a6aab, base 2653fd987)

27 of 27 `exit=0` (`grep -c ' exit='` = 27):

web-wasm, fingerprints, engine-untouched, puzzle-fixtures-unchanged, scenes-and-exports-pinned, protected-tests-unedited, no-new-dependencies, cargo-fmt, cargo-clippy, cargo-clippy-wasm, cargo-build-wasm, cargo-test, wasm-crate-tests, release-replay, puzzle-replay-wasm, typecheck, vitest, bright-builds, player-smoke, build-app, hud-safe-frame, puzzle-e2e, water-e2e, live-worker-e2e, markdown, render-pins, bun-graph-clean.

- **Fingerprints:** scene-spot stamp `2026-10-14T00-27-52Z`. `target/phase43/after-final.txt` matches `base.txt` on 25/25 lines, and `diff` is empty.
- `crates/liquidfun` has no diff since the base. Fixtures and level hashes are unchanged.
- wasm crate tests: 599 passed and 25 ignored, including 11 `(session|frame)::speeds` tests. Release `puzzle::replay`: 34 passed. wasm replay: 6 LEVELHASH and 14 REPLAY passes.
- Vitest: 119 files, 1836 tests. Player smoke: 45. HUD safe frame: 64. Puzzle e2e: 99. Water e2e: 13. Live worker: 8.
- No non-GSD Markdown changed since the base. The markdown gate records that and passes.
- No failed attempts, so there are no `checks-06.attempt-*` files and no fix commits.

## Decision Coverage (D-01..D-20)

| Decision | Plan | Files | Proof |
| --- | --- | --- | --- |
| D-01 water mode, Surface group, storage key | 04 | `render/mode.ts` | `render-mode.test.ts`; e2e 1, 2 (`storeRenderMode` seeds `liquidfun.render-mode.v1`) |
| D-02 default stays shaded blob | 04 | `render/mode.ts` | `render-pins` gate; `render-mode.test.ts` |
| D-03 both pickers, no new HUD control | 04 | `RenderModeOptions.tsx`, `PlayerPanel.tsx`, `PuzzleOptionsDrawer.tsx` | e2e 1, 8, 11; `hud-safe-frame` gate |
| D-04 one particle canvas, `webgl-water` surface | 04 | `mode.ts`, panels | `render-pins` (2 `particle-surface` canvases); e2e 1 |
| D-05 "Water (unavailable)" disabled, preference kept | 03, 04 | `water-capability.ts`, `RenderModeOptions.tsx` | `water-capability.test.ts`; e2e 8 |
| D-06 field + shade pass look | 03, 06 | `webgl-water.ts`, `water-shaders/*`, `water-uniforms.ts` | `webgl-water.test.ts`; by-eye screens (Tuned Constants) |
| D-07 backdrop texture uploaded only on change | 03 | `water-backdrop.ts`, `webgl-water.ts` | `water-backdrop.test.ts`, revision-gate test in `webgl-water.test.ts` |
| D-08 water highp, blob mediump unchanged | 03 | `water-shaders/*.glsl`, `webgl-particles.ts` | `render-pins` gate |
| D-09 DPR cap | 02, 03 | `field-pass.ts` | `field-pass.test.ts`, `webgl-water.test.ts` |
| D-10 speed lane from Rust velocities | 01 | `session/speeds.rs`, `frame/speeds.rs`, `frame-speeds.ts` | 11 speeds tests (`wasm-crate-tests`), `frame-speeds.test.ts`; `render-pins` lane-read-only check |
| D-11 lane only in Water, never feeds physics | 01, 05 | owners, worker protocol | `fingerprints` 25/25; e2e 2, 3, 5 (RunFingerprint equal); Rust on/off proofs |
| D-12 one fallback chain, never blank | 04 | `present-frame.ts`, `canvas.ts` | `present-frame.test.ts`; e2e 8, 9 |
| D-13 float or half-float, framebuffer decides | 02 | `field-pass.ts` | `field-pass.test.ts`; e2e 9 (both hidden draws `canvas`) |
| D-14 loss falls back, restore repaints held frame | 02, 04 | `field-pass.ts`, `scene-surface.ts` | `webgl-render-cache.test.ts`, `field-pass.test.ts`; e2e 6, 7 |
| D-15 drawn surface attribute | 04 | `scene-surface.ts` | e2e 1-13 read `data-drawn-surface` |
| D-16 overlays on the 2D canvas above | 04 | `player-viewport.css`, `scene-surface.ts` | e2e 1 (transparent CSS), e2e 4 (overlay hash equal, opaque pixels > 0) |
| D-17 exports unchanged | 04 | `mode.ts` `svgRenderMode` | `render-mode.test.ts`; e2e 10 (SVG hash equal); `scenes-and-exports-pinned` gate |
| D-18 unit coverage | 01-05 | `web/tests/*` | `vitest` gate (1836 tests) |
| D-19 browser proofs, no shader pixel diff | 06 | `e2e/water.spec.ts`, `e2e/water-helpers.ts` | `water-e2e` gate; no `toHaveScreenshot` / `toMatchSnapshot` in the spec |
| D-20 iPhone viewing | — | — | pending human UAT (below) |

## Manual UAT for `/gsd-verify-work` (non-blocking)

1. Open a scene in Water on a real iPhone, then read the scene canvas `data-drawn-surface`. Record `webgl-water`, `webgl` or `canvas` (D-20). This also covers the half-float todo folded in as D-13.
1. On a real GPU, look at Dam Break, Fountain and First Pour in Water. Confirm that the tuned constants read as intended: clear tinted pools with glints, white jet spray, and readable cup, target line and meter. Note whether the per-particle glitter needs a smoothed-gradient shader change.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The drawer screenshot fired mid-transition**
- **Found during:** Task 2 screenshot review
- **Issue:** `water-unavailable-drawer.png` showed no drawer. The corvu drawer slides in over 500 ms, and the screenshot fired before the slide finished.
- **Fix:** Wait for `data-transitioning` to clear and for the select to be in the viewport before taking the screenshot.
- **Commit:** 551484b0c

**2. [Rule 3 - Blocking] Playwright `toBeDisabled` on a native `<option>`**
- **Found during:** Task 1, first spec run
- **Issue:** `<option disabled value="water">Water (unavailable)</option>` was in the DOM, but `toBeDisabled()` reported it enabled. That is the Playwright check misreporting, not an app bug.
- **Fix:** `expectOptionDisabled` checks the count, then the DOM `disabled` property. The helper has a comment explaining why.
- **Commit:** d0987eae8

**3. [Rule 3 - Blocking] The `bun-graph-clean` pattern matched the surface literal**
- **Found during:** Task 3, a dry check of the cheap gates before the recorded run
- **Issue:** The plan's pattern `webgl-water` matched `mode.ts`, which must declare the `"webgl-water"` surface literal (D-04). The gate would fail even though no module is imported.
- **Fix:** The pattern now requires a slash before the module name (`/(water-shaders|webgl-water|field-pass|water-capability|water-programs)|glsl\?raw`), so it matches only import paths. A positive control, `present-frame.ts` importing `./webgl-water`, still matches. This was fixed before the recorded run, so no failed record exists.
- **Files:** `target/phase43/checks-06.sh` (gitignored evidence)

**4. [Minor] Re-indented line in `water-uniforms.ts`**
- `if (maybeShadeSizeKey === undefined) setStaticShade();` had 8 spaces of indentation. It now has 6, in the tuning commit. Behavior is unchanged.

## Known Stubs

None.

## Self-Check: PASSED

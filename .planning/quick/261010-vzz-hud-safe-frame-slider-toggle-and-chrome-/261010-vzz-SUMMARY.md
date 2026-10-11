---
phase: quick-261010-vzz
plan: "01"
subsystem: web-playground
tags: [hud, camera, safe-frame, playwright, solidjs]
generated_by: gsd-executor
lifecycle_mode: quick
phase_lifecycle_id: quick-261010-vzz
generated_at: "2026-10-11T04:22:53Z"
requires: []
provides:
  - "safeFrame best-fit search around HUD obstacles (web/src/render/safe-frame.ts)"
  - "createCamera optional frame parameter and projectedWorldRect"
  - "Persistent Hide/Show sliders toggle (liquidfun.hud-sliders.v1)"
  - "data-world-frame / data-world-frame-fallback canvas hooks"
affects:
  - web/src/player/frame-loop.ts
  - web/src/player/scene-surface.ts
  - web/src/components/CanvasStage.tsx
tech-stack:
  added: []
  patterns:
    - "Functional core (safeFrame, cameraFrame) with a DOM measurement shell (createHudObstacles)"
    - "data-hud-obstacle marks chrome leaves the camera fits around"
key-files:
  created:
    - web/src/render/safe-frame.ts
    - web/src/player/camera-frame.ts
    - web/src/player/hud-obstacles.ts
    - web/src/player/hud-sliders-preference.ts
    - web/tests/safe-frame.test.ts
    - web/tests/camera-frame.test.ts
    - web/tests/hud-sliders-preference.test.ts
    - web/tests/hud-sliders-toggle.test.ts
    - web/e2e/hud-safe-frame.spec.ts
  modified:
    - web/src/render/camera.ts
    - web/src/render/canvas.ts
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
    - web/tests/camera.test.ts
    - web/tests/portrait-bounds.test.ts
    - AGENTS.md
decisions:
  - "Toggle round-trip browser test uses tesla-valve, not wave-tank: wave tank's wide world is width-limited at 390x844, so hiding the card only re-centres it"
  - "tesla-valve MIN_HEIGHT_FRACTION lowered 0.80 -> 0.75 (measured 0.754) after removing the padding below the drain"
  - "FPS pill min-width 4.75rem, status min-width 5.5rem, and trailing legend width 100px keep obstacle boxes independent of frame rate, pause state, and camera scale"
metrics:
  duration: "~14 min"
  completed: "2026-10-11"
  tasks: 3
  commits: 5
---

# Phase quick-261010-vzz Plan 01: HUD Safe Frame Summary

The default canvas stage now fits each scene's world rectangle into the largest obstacle-free region around the measured HUD chrome. The fit is a pure best-fit search over obstacle edges, wired into both live camera paths. A persistent "Hide sliders"/"Show sliders" toggle sits at the leading edge of the bottom toolbar. No catalog scene at any tested viewport hit the 50% fallback.

## Commits

| Task | Commit | Message |
| ---- | ------ | ------- |
| 1 (RED) | f4c6c7e89 | test(quick-261010-vzz): add failing safe-frame, camera frame, and slider preference tests |
| 1 (GREEN) | 0ef5ebb7d | feat(quick-261010-vzz): add safe-frame core and chrome-aware camera frame |
| 2 (RED) | be3773207 | test(quick-261010-vzz): add failing slider toggle chrome test |
| 2 (GREEN) | 0d4291d82 | feat(quick-261010-vzz): add HUD slider toggle and chrome-aware camera fit |
| 3 | 627fe13e3 | feat(quick-261010-vzz): tighten Tesla valve portrait frame and cover HUD safe frame in the browser |

## What was built

- `safeFrame(viewport, obstacles, worldAspect)`: inflates each obstacle by 8px, clips it to the viewport, and enumerates candidate rectangles from obstacle edges. It scores each by allowed scale, breaks 0.5% ties by distance from the centre, then by top, then by left. It falls back to the full viewport below 50% of the unobstructed score.
- `createCamera(..., frame?)`: without a frame the output is bit-identical to before (`toEqual` tests on 960x540 and 320x540). Exports (`web/src/export/record.ts`) and the classic layout still call it without a frame.
- `createHudObstacles`: one ResizeObserver plus a MutationObserver (childList/subtree) and one rAF-coalesced measure. Its signal uses `sameScreenRects` (0.5px) equality, so text churn does not refit the camera.
- `frame-loop` resize and `refreshCamera` both go through `cameraFrame(clock)` and publish `data-world-frame`. `changeHudObstacles` refits and repaints the held frame.
- Toggle: rendered only when the scene has HUD controls. It sets `aria-expanded` and `aria-controls="canvas-hud-sliders"`. Hiding removes the card from the DOM, and the choice persists under `liquidfun.hud-sliders.v1`.

## Verification evidence

- `bun run test:unit`: 60 files, 521 tests passed. The RED steps failed as expected before each GREEN commit.
- `bun run typecheck`: clean. **There is no lint script in `web/package.json`**, so typecheck is the lint-equivalent gate and no lint step was invented.
- `bun run build:app`: passes.
- `bun scripts/bright-builds-check.ts all`: `findings=0`.
- `just markdown-check`: passes (mdformat 1.0.0 on Python 3.13.12; `mdformat --check AGENTS.md` needed no rewrite).
- Playwright `hud-safe-frame` + `canvas-stage` + `shell` + `player`:
  - **First run: 100 passed, 1 failed.** `shell.spec.ts:138` "build timestamp" timed out waiting for `<time>` because the footer showed `Built at: Unavailable`. Plain `bun run build:app` does not inject `VITE_BUILT_AT`; the repo's provenance build (`scripts/web-build.ts`) does. This is environmental and unrelated to the change.
  - After rebuilding with `VITE_BUILT_AT` set, `shell.spec.ts` passed 14/14, and one confirmatory run of all four specs passed **101/101**.
- `hud-safe-frame.spec.ts` alone: 64/64 passed (JSON reporter: expected 64, unexpected 0, flaky 0).

### Fallback cases

**None.** The JSON report had no `safe-frame-fallback` annotations. A direct probe of `data-world-frame-fallback` on every HUD-control scene (wave-tank, wave-machine, washing-machine, tesla-valve, hydraulic-fountain) at 390x844, 1280x800, 844x390, and 768x1024 returned `null` in all 20 cases.

## Visual check (before/after)

Screenshots: `web/test-results/hud-safe-frame-visual/{before,after}-{wave-tank,tesla-valve}-{390x844,1280x800}.png` (gitignored), with copies in the session scratchpad `visual/`. A Playwright run cleans `test-results/` and deleted the first BEFORE set, so the BEFORE images were recaptured from a scratch worktree build of base commit 0f1b6f1d1 (since removed).

- **Wave tank 390x844**: before, the slider card covered the bottom of the tank. After, the tank sits fully above the card (frame y 250-351) at the same width-limited size.
- **Wave tank 1280x800**: before, the card covered the left third of the tank and its water. After, the tank is clear of the card and the toolbar, placed above the card (frame 108-1204 x 77-384). The tank's very wide world (aspect ~3.6) scores higher above the card than in the full-height region beside it, so the best fit puts it above.
- **Tesla valve 390x844**: before, the valve was large and clear of the card only because of the hand-padded portrait frame. After, the valve is clear of all chrome but **smaller (legend 50 cm -> 1 m) and off-centre to the right**. The best fit (frame 108-374 x 138-611) is the region right of the FPS/debug column, which scores about 3% above the full-width band. The portrait world rectangle is 2.7 m wide while the valve spans about 1.3 m, so the rectangle's side margins collide with chrome the valve itself would clear.
- **Tesla valve 1280x800**: same size as before. After, the valve sits about 85px further right, centred in the region right of the card, so the world frame is fully clear of the card.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Test premise] Toggle round-trip uses tesla-valve instead of wave-tank**
- **Found during:** Task 3
- **Issue:** At 390x844, wave tank's frame is width-limited. Hiding the card moved the frame (y 250-351 -> 402-502) but left its area unchanged (35936 vs 35932 px², rounding noise), so `B > A` failed.
- **Fix:** The round-trip runs on tesla-valve at 390x844. Its frame is height-limited by the card and grows from 266x473 to 324x576 when hidden. A comment in the spec explains why.
- **Files:** web/e2e/hud-safe-frame.spec.ts
- **Commit:** 627fe13e3

**2. [Rule 3 - Blocking] Stable FPS box**
- `.canvas-fps` width followed the digit count, which would change the stats obstacle box. Added `min-width: 4.75rem` inside `.canvas-hud-stats`, as the plan's conditional instructed. Commit 0d4291d82.

### Tesla valve threshold

`MIN_HEIGHT_FRACTION["tesla-valve"]` was lowered from 0.80 to 0.75. With `minY -1.75 -> -0.2` the measured fraction is 0.754, and the width fraction stays above 0.85. The wall check passed unchanged, and no other scene was touched.

## Follow-up suggestions (not done; outside the locked plan)

- Tesla valve on phones: tightening `TESLA_VALVE_PORTRAIT` minX/maxX toward the valve's real width (about ±0.7 m) would let the fit pick a larger, centred frame. The plan said to keep minX/maxX, so this was left alone.

## Known Stubs

None.

## Self-Check: PASSED

- Files: safe-frame.ts, camera-frame.ts, hud-obstacles.ts, hud-sliders-preference.ts, hud-safe-frame.spec.ts, and the four new unit test files exist.
- Commits f4c6c7e89, 0ef5ebb7d, be3773207, 0d4291d82, 627fe13e3 are present in `git log`.

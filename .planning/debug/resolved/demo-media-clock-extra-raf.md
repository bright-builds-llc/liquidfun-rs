---
status: resolved
trigger: "e2e/demo-media-clock.spec.ts fails 7 of 8 in just web-player-smoke: waitForReadyScene expects 1 pending animation callback, gets 2 (Synthetic anchor expected 1 callback, got 2)"
created: 2026-10-11T00:00:00Z
updated: 2026-10-11T01:00:00Z
---

## Current Focus

hypothesis: CONFIRMED and fixed in e0b8a268d
test: just web-player-smoke
expecting: all pass
next_action: none

## Symptoms

expected: demo-media-clock spec 8/8 pass; exactly one pending rAF (frame loop) at anchor and after every step
actual: 7 of 8 fail; pendingAnimationCallbackCount is 2 / "Synthetic anchor expected 1 callback, got 2"
errors: "Synthetic anchor expected 1 callback, got 2"; expect.poll pendingAnimationCallbackCount toBe(1) received 2
reproduction: just web-player-smoke, or cd web && CI=true bunx playwright test e2e/demo-media-clock.spec.ts after build
started: reproduces at phase base 589cb0fb9; predates Phase 38

## Eliminated

## Evidence

- timestamp: 2026-10-11T00:00:00Z
  checked: git log --follow web/src/player/hud-obstacles.ts
  found: file created in 0d4291d82 (quick-261010-vzz, chrome-aware camera fit), an ancestor of 589cb0fb9; only non-loop rAF user in src besides frame loops
  implication: prime suspect for the second pending callback

- timestamp: 2026-10-11T00:10:00Z
  checked: temporary diagnostic spec wrapping the synthetic rAF (callback source + MutationObserver log) on the HEAD build
  found: at ready, pending = [1: `()=>{i=void 0,r(Wd(e,t))}` (hud-obstacles scheduleMeasure), 2: frame-loop scheduleFrame]; advance(0) ran 2; afterwards only the frame loop re-registers (1 pending per step, steps 1->5 exact)
  implication: second callback is the one-shot HUD obstacle measurement, not a perpetual or duplicate loop; production behavior is legitimate

- timestamp: 2026-10-11T00:20:00Z
  checked: demo-media-clock.spec.ts at be3773207 (parent) vs 0d4291d82, same generated wasm, fresh vite build each
  found: parent 8 passed; 0d4291d82 7 failed with `Received: 2`
  implication: regression introduced by 0d4291d82 (quick-261010-vzz chrome-aware camera fit), not 2f3f21b67

- timestamp: 2026-10-11T00:40:00Z
  checked: capture mode CSS and a probe of pending callbacks after toggling capture mode
  found: capture mode uses `visibility: hidden`, so layout and pending count stay unchanged; toggling the HUD sliders does change layout and queues a measurement (regression test fails 3/3 with "Expected exactly 1 animation callback, got 2" when the per-advance settle is removed)
  implication: settle needed at ready and at the start of each advanceEngineSteps call

## Resolution

root_cause: 0d4291d82 added createHudObstacles (web/src/player/hud-obstacles.ts), which coalesces its layout measurement into a one-shot requestAnimationFrame. The demo-media synthetic clock queues every rAF and runs them only on advance, so that callback sat beside the frame loop: waitForReadyScene saw 2 pending and the anchor advance ran 2. Production behavior is legitimate (one-shot, cancelled on cleanup, no perpetual loop).
fix: web/scripts/demo-media/capture.ts gains settleSyntheticClock - wait two native frames (kept via a bound native rAF) so ResizeObserver/MutationObserver work lands, then replay the last delivered timestamp until only the frame loop is pending. The frame loop sees zero elapsed time and takes no step; settle asserts the step index did not move. Called before the anchor and at the start of each advanceEngineSteps call. Regression test toggles the HUD sliders between steps.
verification: demo-media-clock spec 9/9 (27/27 with --repeat-each=3); just web-player-smoke exit 0 (typecheck clean, 788 unit tests, 45 Playwright tests); bunx tsc --noEmit clean; bright-builds-check all findings=0
files_changed: [web/scripts/demo-media/capture.ts, web/e2e/demo-media-clock.spec.ts]

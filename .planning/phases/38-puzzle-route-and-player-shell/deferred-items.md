# Phase 38 deferred items

## From 38-07

- **`e2e/demo-media-clock.spec.ts` fails 7 of 8 tests in `just web-player-smoke` on this host, and it fails the same way at the phase base `589cb0fb9`.** `waitForReadyScene` (`web/scripts/demo-media/capture.ts:103`) expects 1 pending animation callback and gets 2. It reproduced three times: with the 38-07 sources, with the 38-06 sources (`585599aa5`), and with the `589cb0fb9` sources of `web/src`, `web/scripts` and `web/e2e`. So Phase 38 did not cause it. The other 37 player-smoke tests pass with 38-07 in place: `player.spec`, `shell.spec`, `reset-honesty.spec` and `animated-svg.spec`. This is out of scope for 38-07 and is not fixed here.

### Resolved after 38-07

- **Root cause:** commit `0d4291d82` (quick-261010-vzz, chrome-aware camera fit) added `createHudObstacles` in `web/src/player/hud-obstacles.ts`. It coalesces the HUD obstacle measurement into a one-shot `requestAnimationFrame`. The synthetic clock queues every animation callback and runs them only on `advance`, so the measurement sat beside the frame loop and the harness counted 2. The parent `be3773207` passes 8 of 8 and `0d4291d82` fails 7 of 8. The production behavior is a legitimate one-shot callback, not a perpetual or duplicate loop.
- **Fix:** `e0b8a268d` adds `settleSyntheticClock` to `web/scripts/demo-media/capture.ts`. It waits two native frames, then replays the last delivered timestamp until only the frame loop is pending. The frame loop sees zero elapsed time and takes no step, and the settle asserts that the step index did not move. It runs before the anchor and at the start of each `advanceEngineSteps` call. A new spec test toggles the HUD sliders between steps. No production or simulation code changed.
- **Verification:** `just web-player-smoke` exits 0: typecheck is clean, 788 unit tests pass and 45 Playwright tests pass. The clock spec passes 27 of 27 with `--repeat-each=3`. `bunx tsc --noEmit` is clean, and `bun scripts/bright-builds-check.ts all` reports 0 findings.

# Phase 38 deferred items

## From 38-07

- **`e2e/demo-media-clock.spec.ts` fails 7 of 8 tests in `just web-player-smoke` on this host, and it fails the same way at the phase base `589cb0fb9`.** `waitForReadyScene` (`web/scripts/demo-media/capture.ts:103`) expects 1 pending animation callback and gets 2. It reproduced three times: with the 38-07 sources, with the 38-06 sources (`585599aa5`), and with the `589cb0fb9` sources of `web/src`, `web/scripts` and `web/e2e`. So Phase 38 did not cause it. The other 37 player-smoke tests pass with 38-07 in place: `player.spec`, `shell.spec`, `reset-honesty.spec` and `animated-svg.spec`. This is out of scope for 38-07 and is not fixed here.

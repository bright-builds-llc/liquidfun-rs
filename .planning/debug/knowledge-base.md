# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## web-examples-appear-idle — High-refresh browser scenes remain visually idle
- **Date:** 2026-09-18
- **Error patterns:** nothing happening, browser examples, animation, step index, canvas pixels, 120 Hz
- **Root cause:** `App.tsx` reset its animation timestamp on every callback while `clock.ts` floored each callback delta independently, so sub-1/60-second deltas were discarded forever on high-refresh displays.
- **Fix:** Retain fractional elapsed time in a bounded accumulator, clear it with timestamp resets, and cover repeated 120 Hz timing with unit and Chromium step-plus-pixel regressions.
- **Files changed:** web/src/physics/clock.ts, web/src/App.tsx, web/tests/clock.test.ts, web/e2e/player-helpers.ts, web/e2e/player.spec.ts
---

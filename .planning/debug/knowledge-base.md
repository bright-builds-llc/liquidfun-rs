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

## dam-break-wall-velocity — Dam-break particles tunnel walls and the scene errors
- **Date:** 2026-09-21
- **Error patterns:** dam-break, scene error, particles stuck in walls, extreme velocity, thrashing, NonFiniteValue
- **Root cause:** After SolveForce, Rust cleared `pending_system_force` but left the force buffer filled. Collision and barrier response then added onto already-applied forces every particle iteration, velocities blew past the limit, and particles tunneled the basin walls.
- **Fix:** Zero the force buffer when consuming the pending marker in `ParticleStorage::clear_pending_system_force`, matching upstream `PrepareForceBuffer`.
- **Files changed:** crates/liquidfun/src/particle/storage/runtime.rs, crates/liquidfun/src/particle/solver/preparation.rs, crates/liquidfun-wasm/src/scene/dam_break/tests.rs
---

## demo-media-clock-extra-raf — Synthetic media clock sees 2 pending animation callbacks
- **Date:** 2026-10-11
- **Error patterns:** demo-media-clock, synthetic clock, pendingAnimationCallbackCount, Synthetic anchor expected 1 callback got 2, Expected exactly 1 animation callback, requestAnimationFrame, hud-obstacles
- **Root cause:** createHudObstacles (0d4291d82) coalesces its layout measurement into a one-shot rAF, which the demo-media synthetic clock queues beside the frame loop, so the anchor and step accounting counted 2 callbacks.
- **Fix:** Harness settle step in capture.ts: wait two native frames, then replay the last delivered timestamp (zero elapsed, no step) until only the frame loop is pending; run before the anchor and at the start of each advanceEngineSteps call.
- **Files changed:** web/scripts/demo-media/capture.ts, web/e2e/demo-media-clock.spec.ts
---

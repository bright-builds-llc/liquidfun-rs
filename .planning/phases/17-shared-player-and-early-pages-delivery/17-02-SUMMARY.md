---
phase: 17-shared-player-and-early-pages-delivery
plan: "02"
subsystem: physics
tags: [solidjs, vitest, wasm, clock, session, wasm-04]

requires:
  - phase: 16-rust-wasm-browser-bridge
    provides: SceneSession owner with one advance, one capture, finally-free, and poison-on-failure
provides:
  - Pure acceptedStepCount capped at MAX_STEPS_PER_FRAME = 4
  - nextFrame(stepCount) forwarding one legal 1-4 Rust advance plus one capture
  - TypeScript 1..=4 guard that poisons invalid counts without calling generated advance
affects: [17-05, 17-06, player-shell, hidden-tab-clock]

tech-stack:
  added: []
  patterns: [pure step-budget clock, one advance then one capture, poison invalid step counts]

key-files:
  created:
    - web/src/physics/clock.ts
    - web/tests/clock.test.ts
  modified:
    - web/src/physics/session.ts
    - web/tests/session.test.ts

key-decisions:
  - "Keep acceptedStepCount pure: no performance.now, document.hidden, leftover accumulator, or rAF."
  - "Guard 1..=4 in TypeScript before generated advance; poison invalid counts without calling advance."
  - "Forward one advance(n) plus one capture instead of calling nextFrame four times."

patterns-established:
  - "Functional-core clock: acceptedStepCount is data-in/data-out and never reads the document or animation loop."
  - "Session owner still captures once after a single bounded advance(n)."

requirements-completed: [WASM-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T11:50:27Z

duration: 2min
completed: 2026-09-17
---

# Phase 17 Plan 02: Bounded 1-4 Step Budget Summary

**Pure `acceptedStepCount` clock capped at four 1/60-second ticks, plus `nextFrame(stepCount)` that forwards one legal 1–4 Rust advance and one capture.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-17T11:48:39Z
- **Completed:** 2026-09-17T11:50:27Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `acceptedStepCount` returns 0 for non-finite and non-positive elapsed time, 1–3 for matching sixtieths, and exactly 4 for `MAX_DELTA_SECONDS` or any larger finite value.
- `MAX_STEPS_PER_FRAME` is 4 and `MAX_DELTA_SECONDS` is `4 / 60`; the clock never reads `performance.now`, `document.hidden`, or `requestAnimationFrame`.
- `nextFrame()` and `nextFrame(1)` still advance `[1]`, capture once, and free the raw frame.
- `nextFrame(2)` and `nextFrame(4)` forward one generated `advance(n)` and one capture; `nextFrame(0)` and `nextFrame(5)` poison with the existing failed/disposed messages without calling advance.

## Task Commits

Each task was committed atomically:

1. **Task 1: Compute a capped step budget**
   - `99d1260` (`test`) — failing zero, clamp, and max-4 clock cases
   - `9c1921d` (`feat`) — pure `acceptedStepCount` with `MAX_STEPS_PER_FRAME = 4`
1. **Task 2: Forward 1–4 advances in one capture**
   - `59867cf` (`test`) — failing 0/2/4/5 session cases
   - `2859ed3` (`feat`) — `nextFrame(stepCount)` guard, one advance, one capture

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `web/src/physics/clock.ts` — Pure 0–4 step budget over elapsed seconds
- `web/tests/clock.test.ts` — Non-finite, non-positive, 1–3 sixtieths, and max-4 cases
- `web/src/physics/session.ts` — Optional `stepCount`, 1..=4 guard, one `advance(stepCount)`, one capture
- `web/tests/session.test.ts` — Existing dispose/poison coverage plus 0/1/2/4/5 advance cases

## Decisions Made

- Keep the clock a functional core: leftover catch-up and hidden-tab pause stay in the later player shell.
- Reject unsafe or out-of-range step counts in TypeScript before crossing into generated WASM, then poison with the fixed `FAILED_MESSAGE`.
- Call generated `advance` once with the accepted count rather than looping `nextFrame()` or `advance(1)`.

## Deviations from Plan

None - plan executed exactly as written.

**Total deviations:** 0 auto-fixed
**Impact on plan:** None.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 17-03 (production Vite base and provenance) and later 17-05, which should call `nextFrame(acceptedStepCount(elapsed))` from the animation shell.
- WASM-04 hidden-tab pause, accumulator clear, and one-session teardown remain in Plans 17-05 and 17-06; this plan only shipped the pure clock and the session 1–4 seam.
- Phase 16 copied frames, `finally` free, and poison-on-failure remain intact. No App.tsx, Pages, or Playwright changes.

## Self-Check: PASSED

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

---
phase: 39-place-then-play-tools-and-stars
plan: "10"
subsystem: web-puzzle-frame-loop
tags: [typescript, puzzle, frame-loop, fast-forward, speed, D-24, TOOL-07]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T01:30:18Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: speedStepRequest and restoreUnrunSpeedSteps (39-03); speed on every PuzzleState (39-07); cycleSpeed dispatching setSpeed (39-09)
provides:
  - web/src/player/puzzle-loop.ts requests speedStepRequest(stepCount, speed) steps per presented frame and restores unrun steps with restoreUnrunSpeedSteps
affects:
  - 39-13 (browser proof that 2x and 4x give the same outcome)
tech-stack:
  added: []
  patterns:
    - fast-forward scales only the per-frame step request; every step is still captured with nextFrame(1) and dispatched as its own stepped event
key-files:
  created: []
  modified:
    - web/src/player/puzzle-loop.ts
    - web/tests/puzzle-loop.test.ts
key-decisions:
  - "39-10: puzzle loop requests stepCount x speed steps per frame; the 12 ms budget and MAX_STEPS_PER_FRAME 4 are unchanged, and unrun steps return as (unrun / speed) x STEP_SECONDS"
requirements-completed: []
duration: 2min
completed: 2026-10-13
---

# Phase 39 Plan 10: Puzzle Loop Fast-Forward Summary

**The puzzle frame loop now runs `stepCount x speed` single `nextFrame(1)` steps per presented frame. It is still bounded by the 12 ms budget, and any steps the budget leaves unrun are restored as wall time `(unrun / speed) x STEP_SECONDS` (D-24).**

## Performance

- **Duration:** about 2 min
- **Started:** 2026-10-13T01:28:56Z
- **Completed:** 2026-10-13T01:30:18Z
- **Tasks:** 1 (TDD: RED and GREEN commits)
- **Files modified:** 2

## Accomplishments

- `onFrame` computes `speedStepRequest(stepTime.stepCount, deps.current().speed)` and passes the result to `presentSteps`.
- `presentSteps` restores budget-stopped steps with `restoreUnrunSpeedSteps(..., speed)`. The unused `restoreUnrunSteps` import is gone. Scene playback in `frame-loop.ts` still uses it.
- The one-step-per-capture loop, held/stop logic, `FRAME_STEP_BUDGET_MS = 12` and `MAX_STEPS_PER_FRAME = 4` are unchanged.
- The test harness now supports scripted win and decide steps, a per-state speed, a configurable frame interval, and a simulated per-step wall cost through a controllable `performance.now`. It also adds 4 AAA cases:
  1. At 4x with a two-step accumulation (35 ms frame), one frame calls `nextFrame` 8 times and dispatches 8 `stepped` events.
  1. A scripted win at step 6 and decision at step 9 ends the same way at 1x, 2x and 4x: `outcomeStep 6`, `decidedStep 9`, held at `9 + SETTLE_TAIL_STEPS`, with the same total step count.
  1. With a 2.5 ms step cost at 4x, the 12 ms budget stops after 5 of 8 steps, and `frameRemainderSeconds` equals the accumulator remainder plus `(8 - 5) / 4 x STEP_SECONDS`.
  1. At 4x with the four-step cap, one frame runs 16 steps. Every call is `nextFrame(1)`, and `MAX_STEPS_PER_FRAME` is still 4.

## Task Commits

1. **Task 1 RED: failing fast-forward tests** - `f5762dc8a` (test)
1. **Task 1 GREEN: speed-aware step request and debt restore** - `2dc7d5c1d` (feat)

No refactor commit was needed.

## Verification

- `bunx vitest run tests/puzzle-loop.test.ts` passed 11 of 11 tests. At RED, 3 tests failed and the invariance case passed, as expected, because speed had no effect yet.
- The full `bunx vitest run` passed 91 files and 1221 tests.
- `bunx tsc --noEmit` exited 0.
- `bun scripts/bright-builds-check.ts all` reported 0 findings and exited 0. `web/tests/puzzle-loop.test.ts` is 386 lines, within the 450-line limit.
- The acceptance greps for `speedStepRequest`, `restoreUnrunSpeedSteps`, `nextFrame(1)` in `puzzle-loop.ts` and `export const MAX_STEPS_PER_FRAME = 4;` in `clock.ts` all matched.

## Decisions Made

- Speed is read from the state once per frame, at the start of `presentSteps`. A `stepped` dispatch keeps the speed, so the restore uses the same multiplier as the request.

## Deviations from Plan

None. The plan was executed as written.

## Threat Mitigations

- T-39-45: the 12 ms budget still stops work at 4x, unrun debt is restored at wall-time scale, and `accumulateStepTime` still clamps debt. Test case 3 covers this.
- T-39-46: every step is a separate `nextFrame(1)` capture, and the latch and decision are identical at 1x, 2x and 4x. Test cases 2 and 4 cover this.

## Requirements

TOOL-07 is only partly covered here. This plan proves speed invariance at the unit level, and Plan 13 adds the browser proof. TOOL-07 is not marked complete.

## Self-Check: PASSED

- FOUND: web/src/player/puzzle-loop.ts
- FOUND: web/tests/puzzle-loop.test.ts
- FOUND: f5762dc8a
- FOUND: 2dc7d5c1d

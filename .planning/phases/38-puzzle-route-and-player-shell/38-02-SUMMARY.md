---
phase: 38-puzzle-route-and-player-shell
plan: "02"
subsystem: web-puzzle-core
tags: [typescript, puzzle, reducer, formatting, placements, GOAL-01, GOAL-03, PUZ-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T02:42:28.080Z"
requires:
  - web/src/physics/frame.ts PuzzleFrameStatus (unchanged)
  - web/src/catalog/puzzles.ts PuzzleId (unchanged)
provides:
  - web/src/puzzle/reduce-puzzle.ts PuzzleState, PuzzlePhase, PuzzleEvent, PuzzleLatch, reducePuzzle, INITIAL_PUZZLE_STATE, SETTLE_TAIL_STEPS, remainingSimulationSteps, puzzleStepBudget
  - web/src/puzzle/controller.ts PuzzleController
  - web/src/puzzle/format.ts GoalTick, formatSimulationTime, floorPercent, goalTicks, goalText, timerText, FillMeterModel, fillMeterModel, ResultCardModel, resultCardModel, ResultActionsModel, resultActionsModel
  - web/src/puzzle/placements.ts PUZZLE_TOOL_CODES, PuzzleToolKind, PuzzlePlacement, PlacementLanes, EMPTY_PLACEMENTS, encodePlacementLanes, PUZZLE_TEST_PLACEMENTS_GLOBAL, maybeReadTestPlacements
affects:
  - 38-05 (lifecycle and puzzle loop dispatch into reducePuzzle, call puzzleStepBudget, read the seam and encode lanes)
  - 38-06 (panel renders timerText, fillMeterModel, resultCardModel, resultActionsModel through PuzzleController)
  - 38-09 (greps for wall-clock APIs under web/src/puzzle and for the seam global)
tech-stack:
  added: []
  patterns:
    - pure discriminated-union reducer with exhaustive never defaults; ignored events return the same object, contract violations throw
    - view models switch on state.kind and read live status in building/running and the latch in ended
key-files:
  created:
    - web/src/puzzle/reduce-puzzle.ts
    - web/src/puzzle/controller.ts
    - web/src/puzzle/format.ts
    - web/src/puzzle/placements.ts
    - web/tests/reduce-puzzle.test.ts
    - web/tests/puzzle-format.test.ts
    - web/tests/puzzle-placements.test.ts
  modified: []
key-decisions:
  - "reducePuzzle fails closed on a `loaded` event outside loading and on a step past tailEndStep, in addition to the planned latch and flip guards"
  - "A latch flip is detected when either the outcome or maybeOutcomeStep differs from the latch"
  - "Seam shape errors throw 'Invalid test placements'; seam values that are well-shaped but fail the lane checks (fractional, out of i32 range, unknown kind) throw the shared 'Invalid puzzle placement'"
requirements: [GOAL-01, GOAL-03, PUZ-04]
duration: 4min
completed: 2026-10-12
---

# Phase 38 Plan 02: Puzzle functional core Summary

**This plan adds a pure puzzle phase reducer. It latches the Rust outcome only at `stepIndex === outcomeStep`, runs a 120-step settle tail and then holds. Pure formatters supply the timer (`0:12.0`), the fill meter, and the result copy (`Filled! 76% of 60%`, `Out of time: 0% of 60%`). A fail-closed placement encoder and a seam that reads only a window global are included, with the `PuzzleController` contract that Plans 05 and 06 meet at.**

## Performance

- **Duration:** about 4 min
- **Tasks:** 3 of 3
- **Files:** 7 created, 0 modified

## Accomplishments

### Task 1: reducer and controller contract (commit `eb97e69ed`)

- `reducePuzzle(state, event)` handles these transitions:
  - `loaded` (step 0, outcome running) goes to building, or to running when `pendingRun`.
  - `play` goes from building to running at step 0.
  - `stepped` while running either tracks the step or latches into `ended` with `tailEndStep = outcomeStep + 120`.
  - `stepped` while ended keeps the latch and sets `held` when `stepIndex >= tailEndStep`.
  - `retry` and `edit` go from ended to `loading` with `pendingRun` true or false.
  - `reset` from any phase goes to `INITIAL_PUZZLE_STATE`.
- Errors thrown:
  - `"Puzzle level did not start at step 0"`
  - `"Puzzle latch was not observed at its outcome step"`
  - `"Puzzle outcome changed after the latch"`
  - `"Puzzle stepped outside a run"`
  - `"Puzzle level loaded outside loading"` (added)
  - `"Puzzle stepped past the settle tail"` (added)
- `remainingSimulationSteps` returns:
  - loading: undefined
  - building: stepLimit
  - running: stepLimit − stepIndex
  - ended: stepLimit − outcomeStep
- `puzzleStepBudget` allows every requested step while running. In an unheld tail it is clamped to `tailEndStep − stepIndex`. Otherwise it is 0.
- `PuzzleController` in `controller.ts` is type-only: `maybeLevelId`, `state`, `play`, `retry`, `edit`, `tryAgain`.

### Task 2: view models (commit `22c54544d`)

- `formatSimulationTime(steps)` uses `floor(max(0, steps) / 6)` tenths and returns `m:ss.t`.
- `floorPercent(count, full)` is `min(100, floor(count*100/full))` and throws `"Puzzle capacity must be positive"`.
- `goalTicks(target, full)` returns `[★ Win @ target/full, Full @ 1]`.
- `goalText` returns "Fill the cup to 60% before time runs out."
- `timerText` returns `-:--.-` while loading.
- `fillMeterModel` uses the live status in building and running and the frozen latch once ended (UI-D5).
- `resultCardModel` returns loading, goal or outcome. The outcome text is `${outcomeWord} ${scoreText}`.
- `resultActionsModel(undefined)` returns Next disabled with `aria-label "Next: No next level yet"` and primary `retry`. Given an id, it returns `maybeHref "#/puzzle/<id>"` and primary `next`.

### Task 3: placements (commit `7bfec7165`)

- `encodePlacementLanes` checks each placement with `Number.isSafeInteger`, the i32 range and `Object.hasOwn(PUZZLE_TOOL_CODES, kind)` before writing to `Uint8Array`/`Int32Array`. For `plank 8 64 23` it produces kinds `[1]` and poses `[8, 64, 23]`.
- `maybeReadTestPlacements(source, levelId)`:
  - reads only `Reflect.get(source, "__liquidfunPuzzleTestPlacements")`
  - returns undefined when the global is absent
  - parses field by field with `typeof` and `Array.isArray` into fresh placements, then runs the same lane checks
  - throws `"Test placements name another level"` or `"Invalid test placements"`

## Verification

- `bunx vitest run tests/reduce-puzzle.test.ts tests/puzzle-format.test.ts tests/puzzle-placements.test.ts`: 3 files, **96 tests** passed. That is 39 reducer tests (minimum 16), 33 format tests (minimum 14) and 24 placement tests (minimum 10).
- `bun run test:unit`: 67 files, 684 tests passed.
- `bun run typecheck`: clean after each task.
- `bun scripts/bright-builds-check.ts all`: findings=0 after each task.
- `grep -rnE "Date\.now|performance\.now|setTimeout|setInterval" web/src/puzzle/` prints nothing.
- `! grep -nE "location|URLSearchParams|localStorage|sessionStorage|document\." web/src/puzzle/placements.ts` passes. `git grep -n "__liquidfunPuzzleTestPlacements" -- web/src` lists only `web/src/puzzle/placements.ts`.
- RED was confirmed for each task: the suite failed on the missing module before the implementation existed.
- Threat mitigations:
  - T-38-04: the seam reads only through `Reflect.get`, and the source-inspection test covers it.
  - T-38-05: the safe-integer, i32 and kind allowlist checks have tests.
  - T-38-07: the reducer latches only at the outcome step and throws on a flip.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Fail closed] Extra reducer guards**
- **Found during:** Task 1
- **Issue:** The behavior list did not say what happens when `loaded` arrives outside loading, or when a step goes beyond `tailEndStep`. Ignoring either case silently could hide a lifecycle bug.
- **Fix:** Both cases now throw: `"Puzzle level loaded outside loading"` and `"Puzzle stepped past the settle tail"`. The flip check also compares `maybeOutcomeStep`. Plan 05 always dispatches `reset`, `retry` or `edit` before `loaded`, and stops stepping at `held`, so neither guard fires on the planned paths. Both guards have tests.
- **Files modified:** web/src/puzzle/reduce-puzzle.ts, web/tests/reduce-puzzle.test.ts
- **Commit:** eb97e69ed

The plan asks for one commit per task, so the TDD tasks have no separate RED commits; RED was confirmed locally before each implementation. The web package has no formatter or lint script, so typecheck and the bright-builds checks were the pre-commit checks that applied.

## Known Stubs

None. `EMPTY_PLACEMENTS` is the intended Phase 38 default under D-04/D-22, and Phase 39 will supply placements.

## Self-Check: PASSED

- FOUND: web/src/puzzle/reduce-puzzle.ts, web/src/puzzle/controller.ts, web/src/puzzle/format.ts, web/src/puzzle/placements.ts, web/tests/reduce-puzzle.test.ts, web/tests/puzzle-format.test.ts, web/tests/puzzle-placements.test.ts
- FOUND: commits eb97e69ed, 22c54544d, 7bfec7165

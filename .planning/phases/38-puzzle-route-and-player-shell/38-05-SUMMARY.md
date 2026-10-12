---
phase: 38-puzzle-route-and-player-shell
plan: "05"
subsystem: web-player-shell
tags: [typescript, puzzle, lifecycle, frame-loop, GOAL-01, GOAL-03, PUZ-03]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T02:56:11.993Z"
requires:
  - web/src/puzzle/reduce-puzzle.ts reducePuzzle, puzzleStepBudget, INITIAL_PUZZLE_STATE (Plan 02)
  - web/src/puzzle/placements.ts encodePlacementLanes, maybeReadTestPlacements, EMPTY_PLACEMENTS (Plan 02)
  - web/src/puzzle/format.ts goalTicks (Plan 02)
  - web/src/catalog/puzzle-levels.ts puzzleTitleForId (Plan 01)
  - web/src/catalog/puzzle-bounds.ts worldBoundsForPuzzle (Plan 03)
  - web/src/render/glass-cup.ts glassCup (Plan 03)
  - web/src/physics/puzzle-session.ts createPuzzleSession; web/src/physics/puzzle-loader.ts loadPuzzleSession (Plan 04)
  - web/src/player/frame-loop.ts FrameClock, cancelPendingFrame (unchanged)
  - web/src/physics/clock.ts accumulateStepTime, restoreUnrunSteps, FRAME_STEP_BUDGET_MS (unchanged)
provides:
  - web/src/player/puzzle-runtime.ts PuzzleRuntime, createPuzzleRuntime
  - web/src/player/puzzle-loop.ts PuzzleLoopDeps, schedulePuzzleFrame, requirePuzzleStatus
  - web/src/player/puzzle-lifecycle.ts PuzzleHost, beginPuzzleRoute, startPuzzleUnlessLoading, playPuzzle, retryPuzzle, editPuzzle, tryAgainPuzzle, leavePuzzle, createPuzzleController
affects:
  - 38-07 (adds `puzzle: PuzzleRuntime` to SceneRuntime, binds FrameLoopDeps.maybeStartPuzzle to startPuzzleUnlessLoading, calls beginPuzzleRoute/leavePuzzle from handleHashChange, passes createPuzzleController to the panel)
  - 38-06 (PuzzleController consumer)
tech-stack:
  added: []
  patterns:
    - dedicated puzzle rAF loop sharing clock.maybeAnimationFrameId so cancelPendingFrame stops it
    - non-reactive state mirror folded per step, Solid signal published once per presented frame
    - generation counter plus owned-session identity as the loop's isCurrent guard
key-files:
  created:
    - web/src/player/puzzle-runtime.ts
    - web/src/player/puzzle-loop.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/tests/puzzle-loop.test.ts
    - web/tests/puzzle-lifecycle.test.ts
  modified: []
key-decisions:
  - "beginPuzzleRoute increments the generation before its fallible entry block, so an in-flight older build is superseded even when the newer entry fails (seam or dispose error)"
  - "playPuzzle also requires host.maybeSession === puzzle.maybeOwned, so Play is a no-op after failScene disposed the session"
  - "requirePuzzleStatus is exported from puzzle-loop.ts and shared by the build's step-0 capture"
requirements: [GOAL-01, GOAL-03, PUZ-03]
duration: 5min
completed: 2026-10-12
---

# Phase 38 Plan 05: Puzzle runtime, frame loop and lifecycle Summary

**This plan adds the puzzle player's shell. A dedicated rAF loop steps one fixed step per capture, at most 4 per frame. It feeds every step into `reducePuzzle`, latches at the exact outcome step, runs the 120-step settle tail, then holds and stops scheduling. The lifecycle enters a puzzle route with tilt stopped, pan off, the camera at identity and the test seam read once. Each build gets a fresh direct session and shows its never-advanced step-0 frame. Play, Retry, Edit and Try again are implemented, and a title mismatch fails closed. `PuzzleController` reads the puzzle bag lazily. No existing shell file changed.**

## Performance

- **Duration:** about 5 min (02:51Z to 02:56Z)
- **Tasks:** 2 of 2
- **Files:** 5 created, 0 modified

## Accomplishments

### Task 1: runtime bag and puzzle loop (commit `797389a80`)

- `createPuzzleRuntime()` sets up the following. Call it inside a Solid root.
  - `state` and `maybeLevelId` signals
  - the `current` mirror
  - `placements` (starts as `EMPTY_PLACEMENTS`)
  - `maybeGlass` and `maybeOwned`
- `schedulePuzzleFrame(clock, context, deps)` follows the `scheduleFrame` pattern. Each callback does these checks in order:
  - clear the rAF id
  - return if `isCurrent()` is false or `puzzleStepBudget(state, 1) === 0`
  - on a hidden tab, reset the timestamp and reschedule
  - if the session or camera is missing, fail
  - use the first timestamp only to prime the clock
- After the checks, it runs `accumulateStepTime`, takes the budget from `puzzleStepBudget`, and loops on `nextFrame(1)` until the budget, held or `FRAME_STEP_BUDGET_MS` stops it. It then calls `restoreUnrunSteps`, draws, observes and publishes once.
- `requirePuzzleStatus(frame)` throws `"Puzzle frame is missing its goal status"`.

### Task 2: lifecycle and controller (commit `eaba8bf26`)

- Exported signatures that Plan 07 binds:
  ```typescript
  export type PuzzleHost = Pick<SceneRuntime, "generation" | "clock" | "maybeCanvas" | "maybeContext" | "maybeSession"
    | "maybeWorkerInitialization" | "maybeCanvasPointer" | "tiltBinding" | "constructionValues" | "view" | "setView"
    | "refreshCamera" | "drawSceneFrame" | "failScene" | "setMaybeDebugFrame" | "setStepsThisFrame" | "setFpsTicks"
    | "setPanEnabled" | "setTiltGravityEnabled" | "setTiltDebug"> & { readonly puzzle: PuzzleRuntime };
  export function beginPuzzleRoute(host: PuzzleHost, id: PuzzleId): Promise<void>;   // never rejects
  export function startPuzzleUnlessLoading(host: PuzzleHost, id: PuzzleId): void;    // bind FrameLoopDeps.maybeStartPuzzle here
  export function playPuzzle(host: PuzzleHost): void;
  export function retryPuzzle(host: PuzzleHost): void;
  export function editPuzzle(host: PuzzleHost): void;
  export function tryAgainPuzzle(host: PuzzleHost): void;
  export function leavePuzzle(host: PuzzleHost): void;                               // does not dispose host.maybeSession
  export function createPuzzleController(host: PuzzleHost): PuzzleController;
  ```
- Entry stops tilt first with `changeTiltGravity(binding, false, () => undefined, ...)`; the promise's errors go to `.catch(host.failScene)`. One `try` then wraps the rest of entry: dispose, pan off, `IDENTITY_CAMERA_VIEW`, `constructionValues = {}`, puzzle bounds, refreshing the camera, `reset`, the loading view and the seam read. Any error there goes to `failScene`.
- `buildPuzzle` is private and never rejects. A stale load frees the generated session.
- A title mismatch disposes the session and throws `Level title mismatch: expected "...", wasm reported "..."`.
- The build sets the glass from `glassCup(info.cupInterior, target, full, goalTicks(...))` and sets `dataset.simulationBackend = "direct"`. It presents the step-0 frame from `captureFrame()` with `loaded`, then starts the loop when the state is `running` (Retry).

## Verification

- `bunx vitest run tests/puzzle-loop.test.ts tests/puzzle-lifecycle.test.ts`: **27 passed**. That is 7 loop tests (minimum 7), including the 129-call hold test, and 20 lifecycle tests (minimum 13), including the throwing-dispose tests and the resize-during-rebuild test.
- `bun run test:unit`: 72 files, 762 tests passed.
- `bun run typecheck`: clean after each task.
- `bun scripts/bright-builds-check.ts all`: findings=0 after each task. The largest new file is `puzzle-lifecycle.ts` at 299 lines.
- Acceptance greps:
  - `nextFrame\([2-9]` → 0
  - `location|URLSearchParams|localStorage` → 0
  - `presentOwnedFrame|reapplyStoredTiltGravity|createWorkerSession|attachCanvasPointer` → 0
  - empty `catch` → 0
  - The other required patterns are present.
- `git diff --quiet 589cb0fb9 -- web/src/player/{frame-loop,scene-lifecycle,scene-runtime,scene-surface,playground-session}.ts web/src/physics/clock.ts`: exit 0.
- Mutation checks, done temporarily and reverted:
  - With the `startPuzzleUnlessLoading` guard removed, the resize-during-rebuild test fails.
  - With the entry generation increment removed, the new supersede test fails.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Entry failure could let an older in-flight build land**
- **Found during:** Task 2
- **Issue:** In the plan, only `buildPuzzle` increments the generation. Suppose entry fails, for example on an invalid seam or a throwing dispose, while an earlier build is still awaiting the loader. That build stays current. It would then land `loaded` on the freshly reset state and overwrite the failure view.
- **Fix:** `beginPuzzleRoute` increments the generation before its `try`. A test now covers this ("supersedes an in-flight build even when the newer entry fails").
- **Files modified:** web/src/player/puzzle-lifecycle.ts, web/tests/puzzle-lifecycle.test.ts
- **Commit:** eaba8bf26

**2. [Rule 2 - Fail closed] Play requires the owned session to still be live**
- **Found during:** Task 2
- **Issue:** `failScene` disposes `host.maybeSession` but does not know about `puzzle.maybeOwned`. Play after a failure could then step a disposed session.
- **Fix:** `playPuzzle` also requires `host.maybeSession === host.puzzle.maybeOwned`.
- **Files modified:** web/src/player/puzzle-lifecycle.ts
- **Commit:** eaba8bf26

The plan asks for one commit per task, so the TDD tasks have no separate RED commits. RED was confirmed for Task 1 because the module was missing. `web/` has no format or lint script.

## Known Stubs

None. The placements default to `EMPTY_PLACEMENTS` by design (D-04/D-22), and Phase 39 will supply them.

## Threat Flags

None. T-38-14 through T-38-17 are mitigated as planned:
- T-38-14: the seam is read only through `maybeReadTestPlacements(window, id)`, and the greps above confirm no `location` or storage access.
- T-38-15: tilt is stopped with a getter that returns no session, and the code has no pointer attach or reapply.
- T-38-16: the title check fails closed.
- T-38-17: the loop is guarded by `isCurrent`, keeps its rAF id in the shared clock slot, and stops at held.

## Self-Check: PASSED

- FOUND: web/src/player/puzzle-runtime.ts, web/src/player/puzzle-loop.ts, web/src/player/puzzle-lifecycle.ts, web/tests/puzzle-loop.test.ts, web/tests/puzzle-lifecycle.test.ts
- FOUND: commits 797389a80, eaba8bf26

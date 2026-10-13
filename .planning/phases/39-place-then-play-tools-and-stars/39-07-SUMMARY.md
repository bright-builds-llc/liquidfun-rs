---
phase: 39-place-then-play-tools-and-stars
plan: "07"
subsystem: web-puzzle-phase-machine-and-hud-models
tags: [typescript, puzzle, reducer, editor, stars, hud, functional-core, D-10, D-20, D-21, D-24, D-25, D-27, UI-D14, UI-D16]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:58:25Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: editor.ts and editor-format.ts (39-02); stars.ts, star-format.ts and speed.ts (39-03); D-11 frame scalars (39-04); PuzzleLevelInfo.toolBudget and brimCount (39-06)
provides:
  - web/src/puzzle/reduce-puzzle.ts PuzzleState with editor and speed, PuzzleLatch with decision/peak/Brim fields, stop/setSpeed/edit events, canPlay, isDeciding
  - web/src/puzzle/edit-event.ts PuzzleEditEvent and applyEditEvent
  - web/src/puzzle/format.ts goalTicks(target, full, brim), MeterTick, deciding and tone-carrying result card models, peak-held meter freeze
  - web/src/puzzle/hud-format.ts TransportModel, transportModel, trayModel, selectionBarEnabled, runStarsForState, starThresholdsFor
affects:
  - 39-09 (shell wiring dispatches the new events and must commit an inspection on building entry)
  - 39-10 (loop reads state.speed)
  - 39-11 and 39-12 (canvas, tray, transport and result card read these models)
  - 39-13 (e2e D-10 literals)
tech-stack:
  added: []
  patterns:
    - edit events return the same state object outside building and whenever the editor op is a no-op
    - HUD copy and Play enablement read only the committed inspection snapshot (UI-D14)
key-files:
  created:
    - web/src/puzzle/edit-event.ts
    - web/src/puzzle/hud-format.ts
    - web/tests/reduce-puzzle-editor.test.ts
    - web/tests/puzzle-format-decision.test.ts
  modified:
    - web/src/puzzle/reduce-puzzle.ts
    - web/src/puzzle/format.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/components/PuzzleResultCard.tsx
    - web/tests/reduce-puzzle.test.ts
    - web/tests/puzzle-format.test.ts
    - web/tests/puzzle-loop.test.ts
    - web/tests/puzzle-lifecycle.test.ts
    - web/tests/puzzle-shell-wiring.test.ts
    - web/e2e/puzzle.spec.ts
key-decisions:
  - "The stop event carries stepIndex next to status, like loaded, so the reducer can reject a non-step-0 preview ('Puzzle level did not start at step 0'); Plan 09 dispatches stop {stepIndex, status}"
  - "Edit events and their dispatch live in edit-event.ts; reduce-puzzle.ts re-exports PuzzleEditEvent and stays at 359 lines (plan ceiling 400)"
  - "Transport, tray, selection-bar and star helpers live in hud-format.ts, as the plan allowed; format.ts is 268 lines"
  - "While deciding, frames must keep the win outcome and its outcome step; a lost frame without a decided step throws the decision error instead of staying running"
  - "Retry's loaded→running path clears selection and arming, as play does"
requirements-completed: []
duration: 9min
completed: 2026-10-13
---

# Phase 39 Plan 07: Phase Machine and HUD View Models Summary

**The pure puzzle reducer now owns the building editor, speed, Stop and the D-10 decision step. Won runs stay "Filled!" deciding until Brim or the step limit, then latch the peak-held count and Brim step. Every HUD value in the UI-SPEC, including meter ticks, timer freeze, result card, transport, tray, selection bar and run stars, is now a pure, tested model.**

## Performance

- **Duration:** about 9 min
- **Started:** 2026-10-13T00:49:46Z
- **Completed:** 2026-10-13T00:58:25Z
- **Tasks:** 2 of 2
- **Files:** 4 created, 10 modified

## Accomplishments

- `reduce-puzzle.ts` (359 lines):
  - Every state carries `editor` and `speed`.
  - `reset {placements, speed}` seeds the editor, and `loaded` applies the level budget.
  - Edit events apply only in building. In every other phase, or when nothing changes, they return the same state object.
  - `play` requires `canPlay`: zero tools, or a fresh committed inspection where every tool is valid. A live-only inspection never enables Play. Play also clears selection and arming.
  - `stop` returns running → building with the run's own editor object.
  - `setSpeed` works in building, running and ended, and is ignored in loading.
  - `retry` and `edit` carry the editor and speed into loading.
- D-10 decision:
  - A won frame without `maybeDecidedStep` stays running, and `isDeciding` is true.
  - The decision must be observed at exactly `decidedStep`, or the reducer throws "Puzzle decision was not observed at its decided step".
  - The latch stores outcome and outcome step, decided step, goal count, peak held, Brim step, target, full, brim and limit.
  - The settle tail ends at `decidedStep + 120`.
  - Ended frames must keep the outcome, outcome step, decided step and Brim step.
  - `remainingSimulationSteps(ended)` is `stepLimit − decidedStep`: 458 for decided step 262, 0 for a loss.
- `format.ts`:
  - `goalTicks(target, full, brim)` returns Win, Brim, Full.
  - Meter ticks carry `star` and `reached`. Win is reached once won and Brim once its step exists.
  - A won ended run shows the peak-held percent, for example 811/900 → "90%" and "90% full, target 60%". A lost run shows the latched count.
  - The result card has these kinds:
    - `goal {message, tone}`, which in building comes from `buildingMessage` and reads only the committed snapshot;
    - `deciding {"Filled!", "Going for Brim at 90%"}`;
    - `outcome`, whose won score uses peak held ("Filled! 90% of 60%").
- `hud-format.ts`:
  - `transportModel`: accented Play while building with zero tools or committed-valid tools. Otherwise Play is disabled and the status line is the reason. Stop is offered while running and deciding. Play is disabled in ended and loading. Speed shows "2×" with the name "Speed 2×" and the title "Speed 2×. Press to change." Speed is enabled outside loading, and Clear only in building with at least one tool.
  - `trayModel` and `selectionBarEnabled`.
  - `runStarsForState`: undefined before a run, then the live latches, then the ended latch.
  - `starThresholdsFor`: whole-percent thresholds taken from the frame or the latch.
- D-27 level 1: three rounds of place, rotate, remove, re-place, rotate, commit, Play and Stop give canonical placements deep-equal to one clean `(plank, 8, 64, 23)`, with byte-equal lanes.
- UI-D14: a live move plus a live invalid inspection leaves `resultCardModel` deep-equal and `playEnabled` true. The next commit inspection switches the card to the status line and disables Play.

## Task Commits

1. **Task 1 RED:** `d12f00973` (test). Failing tests for the editor phase machine, Stop, speed and the D-10 decision.
1. **Task 1 GREEN:** `3cfa3c4a8` (feat). Phase machine with editor, speed, Stop and the D-10 decision.
1. **Task 2 RED:** `bc774a917` (test). Failing tests for the decision, transport, tray and run-star HUD models.
1. **Task 2 GREEN:** `be7d03be9` (feat). HUD view models for the meter, timer, result card, transport, tray and run stars.

No refactor commits were needed.

## Verification

- `bunx vitest run tests/reduce-puzzle.test.ts tests/reduce-puzzle-editor.test.ts tests/puzzle-loop.test.ts tests/puzzle-lifecycle.test.ts tests/puzzle-format.test.ts` (plus `puzzle-shell-wiring.test.ts`): 153 passed at the end of Task 1.
- `bun run test:unit`: 87 files, 1147 tests passed.
- `bunx tsc --noEmit`: exit 0.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `wc -l web/tests/puzzle-lifecycle.test.ts`: 601, within the 628 limit.
- Acceptance greps matched:
  - `export function canPlay`, `Puzzle decision was not observed at its decided step`, `kind: "stop"` and `kind: "setSpeed"` in `reduce-puzzle.ts`
  - `"★ Brim"` and `Going for Brim at` in `format.ts`
  - `Speed ${speedText}. Press to change.` in `hud-format.ts`
  - the test named "three place, remove, re-place and Play/Stop rounds equal one clean placement"
- Both RED runs failed as expected. Task 1 had 57 failures against the old reducer. Task 2 failed because `hud-format` was missing and three shape assertions changed.
- e2e, after `bun run build:app` with the existing generated wasm, `CI=true bunx playwright test e2e/puzzle.spec.ts --project=chromium`:
  - "opens First Pour in building" and "loses a natural run with the out-of-time card" passed.
  - "wins with the recorded solution through the test seam" **failed**: the meter `aria-valuenow` stayed 0. The seeded tool has no committed inspection yet, so the D-20 gate ignores Play. See Known Interim Gaps.

## Phase 38 Test Expectations Changed

Because of D-10 (decision step, peak-held score and timer freeze):

- `tests/reduce-puzzle.test.ts` was rewritten:
  - A won run now ends at the decision step (262), not the win latch (201). Its hold point is 382, not 321.
  - The latch has the new fields.
  - `remainingSimulationSteps(ended won)` is 458, not 519.
  - Tail and hold cases now use the decision step.
- `tests/puzzle-loop.test.ts`: the fake won frame now carries Brim and decided step equal to the latch step (peak 810). Without them a won run would never decide. The latch-step, tail-end and call-count assertions are unchanged.
- `tests/puzzle-format.test.ts`: the `ended()` helper decides at its outcome step, and the lost cases add `decidedStep: 720`. Their visible outputs are unchanged: "Filled! 76% of 60%" (peak equals count in that fixture), "0:08.6" and the loss texts.

Because of the UI-SPEC building priority and the Brim tick (D-16), not D-10:

- `tests/puzzle-format.test.ts`:
  - `goalTicks` now returns three ticks.
  - Building meter ticks carry `star` and `reached`.
  - Building with zero tools now shows the empty hint instead of the bare goal.
  - The running goal card gains `tone: "goal"`.
- `web/e2e/puzzle.spec.ts` "opens First Pour in building": the zero-placement result message is now "Fill the cup to 60% before time runs out. Pick a tool from the tray to start." (`GOAL_TEXT` became `EMPTY_BUILDING_TEXT`). It passes.

Shape-only updates (`editor`, `speed` and new latch fields in literals): `tests/puzzle-lifecycle.test.ts` `ENDED_STATE`, and `tests/puzzle-shell-wiring.test.ts` (loading literal).

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `stop` event carries `stepIndex`**

- **Found during:** Task 1
- **Issue:** The behavior requires that "a non-step-0 or non-running status throws", but `PuzzleFrameStatus` has no step index.
- **Fix:** `stop {stepIndex, status}`, validated with the same `requireStepZero` check as `loaded`.
- **Commit:** `3cfa3c4a8`

**2. [Rule 3 - Blocking] Edit events split into `edit-event.ts`**

- **Found during:** Task 1
- **Issue:** The reducer reached 419 lines, over the plan's 400-line ceiling.
- **Fix:** Moved `PuzzleEditEvent` and the editor dispatch to a new module. `reduce-puzzle.ts` re-exports the type and still imports from `./editor`, so the key link holds.
- **Commit:** `3cfa3c4a8`

**3. [Rule 3 - Blocking] `tests/puzzle-shell-wiring.test.ts` updated**

- **Found during:** Task 1
- **Issue:** Its `{kind: "loading", pendingRun: true}` literal no longer type-checked. The file was not in the plan's list.
- **Fix:** Added `editor` and `speed` to the literal.
- **Commit:** `3cfa3c4a8`

**4. [Rule 1 - Bug caused by this plan] e2e building literal**

- **Found during:** Task 2
- **Issue:** The new zero-tool empty hint broke `e2e/puzzle.spec.ts` "opens First Pour in building".
- **Fix:** Updated the expected text. The test passes.
- **Commit:** `be7d03be9`

**5. D-27 sequence interpretation**

The plan's literal sequence repeats "place …" while the previous round's tool is still placed, which would leave several tools. Rounds 2 and 3 therefore first select and remove the kept tool, then run the plan's place, rotate, remove, re-place, rotate, commit, Play and Stop. The final canonical placements equal one clean placement, which is the D-27 intent.

**6. Additional guards (Rule 2, T-39-21)**

- A deciding run must keep its win outcome and outcome step.
- A frame that drops back to running after a latch throws.
- A lost frame without a decided step throws.

## Known Interim Gaps

- The solution-win e2e (`wins with the recorded solution through the test seam`) cannot start its run until Plan 09 commits an inspection when a level lands in building (Plan 09 must-have: "the commit inspection runs synchronously before the building state is first published"). Until then, `canPlay` rejects the seeded tool, as T-39-22 intends.
- The e2e's D-10 literals ("Filled! 90% of 60%", "0:07.6", held step 382, `aria-valuenow` 90) belong to Plan 13 and are not edited here.

## Known Stubs

None. The models are pure, and later plans (09, 11, 12) wire them into the shell.

## Threat Flags

None. All three mitigations are in place and tested:

- T-39-21: the decision must be observed exactly, and ended frames must keep the latch, decided step and Brim step.
- T-39-22: Play is gated by `canPlay`, and a live-only inspection never enables it.
- T-39-23: in loading, running and ended, edit events return the same object for every event.

## Requirements

TOOL-02, TOOL-04, TOOL-07, STAR-01 and STAR-02 have reducer and view-model rules here. The shell wiring and UI come in Plans 09 to 12, so this plan marks none of them complete in REQUIREMENTS.md.

## Self-Check: PASSED

- FOUND: web/src/puzzle/edit-event.ts, web/src/puzzle/hud-format.ts, web/tests/reduce-puzzle-editor.test.ts, web/tests/puzzle-format-decision.test.ts
- FOUND commits: d12f00973, 3cfa3c4a8, bc774a917, be7d03be9

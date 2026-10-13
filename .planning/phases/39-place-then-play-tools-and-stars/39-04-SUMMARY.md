---
phase: 39-place-then-play-tools-and-stars
plan: "04"
subsystem: web-frame-boundary
tags: [typescript, puzzle, frame, fail-closed, D-11, STAR-01, STAR-02]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:19:07Z"
requires:
  - phase: 37-puzzle-core-in-the-bridge
    provides: web/src/physics/frame.ts parsePuzzleStatus (D-17 goal scalars, fail closed)
provides:
  - web/src/physics/frame.ts RawProofFrame peakHeldCount?/brimCount?/brimStep?/decidedStep? getters
  - web/src/physics/frame.ts PuzzleFrameStatus peakHeldCount, brimCount, maybeBrimStep, maybeDecidedStep
  - web/src/physics/frame-puzzle.ts parsePuzzleStatus (D-17 + D-11 rules)
  - web/src/physics/frame-scalars.ts invalidFrame, parseNonNegativeInteger, parsePositiveInteger
affects:
  - 39-05 (Rust getters must use these exact names and rules)
  - 39-06 (rebuilt wasm package must expose the four getters or every puzzle frame fails to parse)
  - 39-07 reducer and 39-08 replay (read peak held, Brim and decision step only through this parsed status)
tech-stack:
  added: []
  patterns:
    - puzzle status parsed in two passes; goal fields first, then D-11 star fields checked against the parsed goal status
    - shared fail-closed scalar helpers live in frame-scalars.ts so frame.ts and frame-puzzle.ts share them without a runtime import cycle (frame-puzzle.ts only type-imports from frame.ts)
key-files:
  created:
    - web/src/physics/frame-puzzle.ts
    - web/src/physics/frame-scalars.ts
  modified:
    - web/src/physics/frame.ts
    - web/tests/frame-puzzle.test.ts
    - web/tests/puzzle-format.test.ts
    - web/tests/puzzle-lifecycle.test.ts
    - web/tests/puzzle-loop.test.ts
    - web/tests/puzzle-session.test.ts
    - web/tests/reduce-puzzle.test.ts
key-decisions:
  - "Puzzle-status parsing moved to web/src/physics/frame-puzzle.ts because frame.ts reached 499 lines (plan ceiling 450); frame.ts keeps every exported type, so no importer changed"
  - "Shared helpers moved to frame-scalars.ts instead of importing them from frame.ts, which would have created a runtime frame.ts <-> frame-puzzle.ts import cycle"
  - "The decided step is compared by strict equality with expectedDecidedStep(outcome, brimStep, stepIndex, stepLimit); a won frame past the limit without Brim must report decidedStep == stepLimit"
  - "peakHeldCount is never compared with particleCount (eviction can drop particles after the peak)"
requirements-completed: []
duration: 5min
completed: 2026-10-13
---

# Phase 39 Plan 04: D-11 Frame Scalars Summary

**The TS frame boundary now parses `peakHeldCount`, `brimCount`, `brimStep` and `decidedStep` into `maybePuzzle`. It mirrors every Rust consistency rule and fails closed with "Invalid Rust/WASM frame". All Vitest fakes satisfy the stricter contract.**

## Performance

- **Duration:** about 5 min
- **Started:** 2026-10-13T00:15Z
- **Completed:** 2026-10-13T00:19:07Z
- **Tasks:** 2
- **Files modified:** 9 (2 created, 7 modified)

## Accomplishments

- `RawProofFrame` gains four optional getters. `PuzzleFrameStatus` gains `peakHeldCount`, `brimCount`, `maybeBrimStep` and `maybeDecidedStep`. Scene frames are unchanged, because all ten getters undefined still means no `maybePuzzle`.
- Fail-closed rules:
  - Any D-11 scalar without the goal fields is rejected, and a missing peak or brim is rejected when goal fields are present.
  - Counts must be integers, with `target < brim <= full`.
  - `won` holds exactly when `peak >= target`.
  - A Brim step is present exactly when `peak >= brim`. It appears only on won frames, with `outcomeStep <= brimStep <= min(stepIndex, stepLimit)`.
  - The decided step must equal the D-10 rule: running has none; lost uses `stepLimit`; won uses the Brim step, or `stepLimit` once `stepIndex >= stepLimit`, otherwise none.
- `peakHeldCount` above the live particle count still parses (Pitfall 4, T-39-13).
- `tests/frame-puzzle.test.ts` has 26 new cases: 6 parse cases, 19 `it.each` rejection cases and 1 case for star scalars without goal fields. Four existing cases were adjusted to the new contract: the all-undefined scene case, running, won (peak 3) and lost (decided 600).
- Five other test files now use fakes that match the stricter contract. No assertion changed meaning.

## Task Commits

1. **Task 1 RED: failing D-11 parse tests** - `26c1901e6` (test)
1. **Task 1 GREEN: parse D-11 scalars fail-closed** - `75c079045` (feat)
1. **Task 2: update puzzle status and raw-frame fakes** - `ee38f2c33` (test)

No REFACTOR commit was needed. The tidy pass happened before the GREEN commit.

## Files Created/Modified

- `web/src/physics/frame.ts` (253 lines): exported types including the new getters and fields; `parseRenderFrame` delegates to `parsePuzzleStatus`.
- `web/src/physics/frame-puzzle.ts` (created, 246 lines): `readPuzzleScalars`, `parseGoalStatus`, `parseOutcomeStep`, `parseBrimStep`, `expectedDecidedStep`, `parseStarFields` and exported `parsePuzzleStatus`.
- `web/src/physics/frame-scalars.ts` (created): `invalidFrame`, `parseNonNegativeInteger`, `parsePositiveInteger`.
- `web/tests/frame-puzzle.test.ts` (471 lines): extended builder and the D-11 cases.
- `web/tests/puzzle-format.test.ts`, `web/tests/reduce-puzzle.test.ts`: the `status()` helpers default to peak 0 and brim 810. `WON_STATUS` uses peak 541. `LOST_STATUS` and the flipped lost status decide at 720.
- `web/tests/puzzle-loop.test.ts`: `statusAt` uses brim 810, running peak 0 and won peak 540.
- `web/tests/puzzle-session.test.ts`: running raw fake uses peak 0 and brim 4, because target 3 and full 4 apply.
- `web/tests/puzzle-lifecycle.test.ts` (592 lines, at most 628): the running raw fake uses peak 0 and brim 810.

## Verification

- `bunx vitest run tests/frame-puzzle.test.ts tests/frame.test.ts`: 69 passed. `tests/frame.test.ts` is unedited.
- `bun run test:unit`: 82 files, 1023 tests passed.
- `bunx tsc --noEmit`: exit 0.
- `bun scripts/bright-builds-check.ts all`: `findings=0`.
- `grep -n "decidedStep?()" web/src/physics/frame.ts` matches at line 41. `peakHeldCount` appears in frame.ts and frame-puzzle.ts.
- `grep -nE "peakHeldCount\s*>\s*particleCount|peakHeldCount > particleCount" web/src/physics/*.ts` prints nothing (exit 1).
- `grep -c peakHeldCount` reports at least 1 in each of the five Task 2 files.

The commit after the Task 1 RED commit failed whole-project `tsc` until Task 2 updated the fakes. This was expected for the split, and the final state is green.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Plan-sanctioned split

**1. [Rule 3 - Blocking] Puzzle parsing split out of frame.ts**
- **Found during:** Task 1
- **Issue:** After the D-11 parsing landed, `frame.ts` had 499 lines, above the plan's 450-line ceiling.
- **Fix:** The plan's prescribed split into `web/src/physics/frame-puzzle.ts`. In addition, `web/src/physics/frame-scalars.ts` now holds the three shared scalar helpers. Without it, `frame-puzzle.ts` would need runtime imports from `frame.ts`, which imports `frame-puzzle.ts`, creating a cycle. `frame.ts` still owns all exported types and the `RawProofFrame` getters, so the acceptance greps hold and no importer changed.
- **Files:** web/src/physics/frame.ts, web/src/physics/frame-puzzle.ts, web/src/physics/frame-scalars.ts
- **Commit:** 75c079045

The plan was otherwise executed as written.

## Issues Encountered

None.

## Known Stubs

None.

## Next Phase Readiness

- Plan 05 must implement the Rust getters with these exact JS names (`peakHeldCount`, `brimCount`, `brimStep`, `decidedStep`) and the same rules in `status_is_consistent`.
- The generated `web/src/generated/liquidfun-wasm` package does not yet expose the getters. Until Plan 06 rebuilds it, real puzzle frames fail to parse, because the goal fields are present and peak/brim are missing. Puzzle e2e and the Bun replay (`scripts/puzzle-replay.ts`) were deliberately not run in this plan.

## Self-Check: PASSED

- FOUND: web/src/physics/frame.ts, web/src/physics/frame-puzzle.ts, web/src/physics/frame-scalars.ts, web/tests/frame-puzzle.test.ts
- FOUND commits: 26c1901e6, 75c079045, ee38f2c33

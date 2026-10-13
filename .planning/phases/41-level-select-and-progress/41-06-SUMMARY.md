---
phase: 41-level-select-and-progress
plan: "06"
subsystem: puzzle-select
tags: [typescript, solid, navigation, level-select, playwright, vitest]
requires:
  - "41-03 PuzzleController.progressDegraded and PuzzleProgress.degraded"
  - "41-05 PUZZLES_HASH, PROGRESS_NOTE, .puzzle-progress-note styling and the #/puzzles grid"
provides:
  - "PuzzleLevelLinks (All levels link + degraded note) in web/src/components/PuzzleLevelLinks.tsx"
  - "PuzzleOptionsSheet now requires progressDegraded and always starts with PuzzleLevelLinks"
  - "Unknown-level fallback primary link All levels -> #/puzzles"
  - "resultActionsModel(undefined): enabled primary All levels link to #/puzzles"
affects:
  - "41 progress e2e can reach the grid from the options sheet, fallback and last-level result card"
tech-stack:
  added: []
  patterns:
    - "Level links live inside PuzzleOptionsSheet itself so every sheet (puzzle panel and fallback) gets them"
key-files:
  created:
    - web/src/components/PuzzleLevelLinks.tsx
  modified:
    - web/src/components/PuzzleOptionsDrawer.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/components/PuzzleFallbackPanel.tsx
    - web/src/components/PlaygroundStage.tsx
    - web/src/styles/puzzle-select.css
    - web/src/puzzle/format.ts
    - web/src/components/PuzzleResultCard.tsx
    - web/src/styles/puzzle.css
    - web/tests/puzzle-chrome.test.ts
    - web/tests/puzzle-format.test.ts
    - web/e2e/puzzle.spec.ts
    - web/e2e/puzzle-levels.spec.ts
decisions:
  - "41-06: PuzzleOptionsSheet takes a required progressDegraded prop and renders PuzzleLevelLinks first, so the fallback's sheet carries the All levels link and note too (Pitfall 6)"
  - "41-06: the last level's Next slot is an accented primary All levels link to #/puzzles after a win or a loss; Retry is never primary now; the disabled No next level yet button is deleted"
  - "41-06: ResultActionsModel.next.label widens to \"Next\" | \"All levels\"; enabled and maybeTitle stay in the shape (always true / undefined now)"
metrics:
  duration: 3min
  completed: 2026-10-13
  tasks: 2
  files: 13
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:22:38.228Z"
---

# Phase 41 Plan 06: All levels links back to the grid Summary

Every puzzle page now leads back to the `#/puzzles` grid. The Puzzle options sheet and the form-factor options row start with an `All levels` link, with the degraded-storage note under it when progress cannot be saved. The unknown-level fallback's primary link is `All levels`, and on the last level the result card's Next slot is an accented `All levels` link after a win or a loss. The canvas HUD is unchanged.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | All levels link and note in the options sheet, form-factor row and unknown-level fallback | 9e03309f6 | web/src/components/PuzzleLevelLinks.tsx, PuzzleOptionsDrawer.tsx, PuzzlePanel.tsx, PuzzleFallbackPanel.tsx, PlaygroundStage.tsx, web/src/styles/puzzle-select.css, web/tests/puzzle-chrome.test.ts, web/e2e/puzzle.spec.ts |
| 2 (RED) | Failing test for the last-level All levels action | 2cc21ab8a | web/tests/puzzle-format.test.ts |
| 2 (GREEN) | Last-level All levels result action in the Next slot | 946f36dbf | web/src/puzzle/format.ts, web/src/components/PuzzleResultCard.tsx, web/src/styles/puzzle.css, web/e2e/puzzle-levels.spec.ts |

## What was built

- **`PuzzleLevelLinks` (D-04, D-15):** `div.puzzle-level-links` holding `a.product-control[href="#/puzzles"]` "All levels" and, when degraded, `p.puzzle-progress-note[data-progress-note]` with `PROGRESS_NOTE`. It has no `data-hud-obstacle` and no `innerHTML`. CSS: grid with an 8 px gap, link at `justify-self: start` and at least 44 px tall.
- **Options sheet and form-factor row:** `PuzzleOptionsSheet({ progressDegraded, children })` renders the links before the children, `GitHubSourceLink` and `SiteFooter`. `PuzzlePanel` passes `props.controller.progressDegraded()` to the sheet and puts `PuzzleLevelLinks` first in the form-factor options row.
- **Unknown-level fallback:** the primary link is `All levels` → `PUZZLES_HASH`, and the secondary link is still `Back to scenes`. The body reads `No level is called “{raw}”. Pick one from All levels, or go back to the scenes.`, or for an empty id `This puzzle link does not name a level. Pick one from All levels, or go back to the scenes.` The raw id is still truncated with `truncateRaw` and shown through a text binding (T-41-22). `PlaygroundStage` passes `progressDegraded={props.puzzleProgress.degraded()}` through to the fallback's sheet.
- **Result card (D-04, D-12, UI-D25):** `resultActionsModel(undefined)` returns `{ next: { enabled: true, label: "All levels", ariaLabel: "All levels", maybeTitle: undefined, maybeHref: "#/puzzles" }, primary: "next" }`. On other levels, `Next` still links to the next level even after a loss. The disabled-button fallback in `PuzzleResultCard.tsx` is gone: `disabled` appears 0 times, down from 1 at the phase base. The landscape result-action rule adds `white-space: nowrap`, and slot sizes are unchanged (`puzzle.css` is now 451 lines).

## Tests whose literals changed (T-41-24)

- `web/tests/puzzle-format.test.ts`: the test "disables Next and promotes Retry when there is no next level" became "turns Next into an All levels link to the grid on the last level". The expected value changed from `{enabled: false, label: "Next", ariaLabel: "Next: No next level yet", maybeTitle: "No next level yet", maybeHref: undefined}, primary "retry"` to `{enabled: true, label: "All levels", ariaLabel: "All levels", maybeTitle: undefined, maybeHref: "#/puzzles"}, primary "next"`. The next-level test is unchanged.
- `web/e2e/puzzle-levels.spec.ts`:
  - `const NO_NEXT_LEVEL = "Next: No next level yet"` became `const ALL_LEVELS = "All levels"`, and the comment changed to match.
  - The test title "keeps Next disabled after the last level" became "links All levels after the last level".
  - The old check that the button was disabled became checks on the `.puzzle-result-actions` `All levels` link: its href is `#/puzzles`, it has the class `product-control--accent`, and clicking it shows the `Puzzles` h2. The `Next level` count of 0 and the screenshot are kept.
- `web/e2e/puzzle.spec.ts` (unknown-level fallback, at both viewports): the `Open First Pour` link check (href `/#\/puzzle\/first-pour$/`) became a check that the scoped `.fallback-panel .control-row` `All levels` link has href `#/puzzles`, plus a check that the body contains `Pick one from All levels, or go back to the scenes.`. The title, `Back to scenes`, HTML-payload and empty-id checks are kept.
- `web/tests/puzzle-chrome.test.ts` (added, nothing removed): `components/PuzzleLevelLinks.tsx` joins `PUZZLE_COMPONENTS`, and a new test checks that neither `PuzzleLevelLinks.tsx` nor `PuzzleOptionsDrawer.tsx` contains `data-hud-obstacle`.

## Verification

- `bun run typecheck`, `bun run test:unit` (104 files, 1541 tests) and `bun run build:app` pass.
- `CI=true bunx playwright test e2e/puzzle.spec.ts e2e/hud-safe-frame.spec.ts`: 80 passed. `CI=true bunx playwright test e2e/puzzle-levels.spec.ts e2e/puzzle.spec.ts`: 34 passed, including "grand-finale links All levels after the last level at 1280x800".
- `e2e/hud-safe-frame.spec.ts` is unchanged from the phase base (`git diff --quiet` exit 0). Nothing matching `No next level yet` is left in `web/src`, `web/tests` or `web/e2e`.
- `bun scripts/bright-builds-check.ts all`: findings=0.

## Deviations from Plan

None. The plan was executed as written. (Prettier reports style warnings in several touched files, but the same files already had them at HEAD, and the project has no Prettier config or format script, so they were left alone.)

## Notes for later plans

- UI-SPEC asks for 390×844 and 1280×800 screenshots of the last-level ended card and an e2e check that the grid is clickable and scrollable after following the sheet's `All levels` link. This plan covers the 1280×800 last-level screenshot and the click to the grid from the result card. The portrait screenshot and the navigation from the sheet are left to the phase's progress e2e plan.

## Known Stubs

None.

## Self-Check: PASSED

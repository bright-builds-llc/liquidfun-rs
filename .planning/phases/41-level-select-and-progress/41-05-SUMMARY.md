---
phase: 41-level-select-and-progress
plan: "05"
subsystem: puzzle-select
tags: [typescript, solid, routing, level-select, css, playwright, vitest]
requires:
  - "41-02 levelCardModels / solvedCount view model"
  - "41-03 PuzzleProgress (current, degraded) on session.puzzle.progress"
provides:
  - "PlayableRoute {kind:\"puzzles\"} for #/puzzles exactly, with title, identity and no playable target"
  - "puzzlesNavCurrent(route) for the sidebar link's aria-current"
  - "PUZZLES_HASH, PUZZLES_INTRO, PROGRESS_NOTE in web/src/catalog/puzzle-levels.ts"
  - "web/src/components/PuzzleSelectPanel.tsx and web/src/styles/puzzle-select.css (the level grid page)"
  - "LockIcon in web/src/components/PuzzleIcons.tsx"
  - "Text-only sidebar Puzzles link (nav.puzzles-navigation > a.puzzles-nav-link)"
affects:
  - "41 PuzzleLevelLinks plan reuses PUZZLES_HASH and PROGRESS_NOTE (and .puzzle-progress-note styling)"
  - "41 progress e2e spec targets [data-level-id], data-locked/solved/next-up and data-progress-degraded"
tech-stack:
  added: []
  patterns:
    - "Non-canvas page in the canvas shell is a fixed self-scrolling container with an inline SiteFooter"
    - "Card DOM built only from the pure LevelCardModel; locked card is a named role=group, unlocked a whole-card link"
key-files:
  created:
    - web/src/components/PuzzleSelectPanel.tsx
    - web/src/styles/puzzle-select.css
  modified:
    - web/src/catalog/puzzle-levels.ts
    - web/src/routing/playable-route.ts
    - web/src/player/playable-target.ts
    - web/src/components/PuzzleIcons.tsx
    - web/src/components/PlaygroundStage.tsx
    - web/src/components/PlaygroundShell.tsx
    - web/src/App.tsx
    - web/src/app.css
    - web/tests/playable-route.test.ts
    - web/tests/playable-target.test.ts
    - web/tests/puzzle-chrome.test.ts
    - web/e2e/puzzle.spec.ts
    - web/e2e/shell.spec.ts
decisions:
  - "41-05: #/puzzles and #/puzzles/ parse to {kind:\"puzzles\"}; #/puzzles/<x> stays the scene parser's unknown route; the grid's identity is \"puzzles\" and its title \"Puzzles · liquidfun-rs playground\""
  - "41-05: sidebar Puzzles link sits above the Demos list with aria-current page on the grid and true on level routes; it adds one focusable to the mobile drawer, so the shell.spec focus-trap cycle is 27, not 26"
  - "41-05: puzzle-chrome splits HUD_STYLESHEETS (data-puzzle-layout scope check) from PUZZLE_STYLESHEETS (motion scan, now including puzzle-select.css)"
  - "41-05: the inline canvas-stage SiteFooter is capped at the 960 px content width so it lines up with the grid"
metrics:
  duration: 7min
  completed: 2026-10-13
  tasks: 3
  files: 15
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:18:20.186Z"
---

# Phase 41 Plan 05: Puzzles section, level grid and sidebar link Summary

`#/puzzles` is now its own route. It shows a level grid built only from TS metadata and the persisted progress document, so it renders before wasm loads. The grid has a heading, an intro line, an "N of 6 solved" line, the degraded-storage note when it applies, and six cards in registry order. Each card shows Win, Brim and Thrifty slots and its lock state. A text-only "Puzzles" link above the Demos list opens the grid from any page. It is marked as the current page on the grid and as the current section on level routes.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | `#/puzzles` route variant, target, title, identity and nav-current helper | 1e6e8cc6c | web/src/routing/playable-route.ts, web/src/player/playable-target.ts, web/src/catalog/puzzle-levels.ts, web/tests/playable-route.test.ts, web/tests/playable-target.test.ts |
| 2 | Level grid page, styles, lock icon and stage wiring | ea3db8a6f | web/src/components/PuzzleSelectPanel.tsx, web/src/styles/puzzle-select.css, web/src/components/PuzzleIcons.tsx, web/src/components/PlaygroundStage.tsx, web/src/App.tsx, web/tests/puzzle-chrome.test.ts, web/e2e/puzzle.spec.ts |
| 3 | Text-only sidebar Puzzles link with current-page state | 792c2f644 | web/src/components/PlaygroundShell.tsx, web/src/components/PlaygroundStage.tsx, web/src/app.css, web/e2e/shell.spec.ts |

## What was built

- **Route (D-01, D-02, UI-D26):** `maybeParsePlayableRoute` returns `{kind:"puzzles"}` only when the hash has exactly one segment, `puzzles` (a trailing slash collapses to that). Any other `puzzles/...` hash goes to the unchanged scene parser and becomes an unknown scene. `normalizePlayableRoute` never rewrites the grid. `maybePlayableTarget` returns undefined, so the existing teardown paths run. The title is `Puzzles · liquidfun-rs playground` and the identity is `puzzles`. `hash.ts` and `hash.test.ts` are unchanged.
- **Grid (D-05..D-07, D-15):** `PuzzleSelectPanel` renders `levelCardModels(progress, degraded)`. An unlocked card is an `<a class="puzzle-level-card" href="#/puzzle/<id>">` that covers the whole card. A locked card is a non-focusable `div role="group"` named `Level {n}, {Title}, locked`, with `LockIcon` and "Locked". Each star slot is a `role="img"` span with the label "{Criterion}: earned" or "{Criterion}: not yet". Data hooks: `data-level-id`, `data-locked`, `data-solved` and `data-next-up` on each card, and `data-progress-degraded` on the section. The page has no `data-hud-obstacle`, no `innerHTML` and no wall-clock API.
- **Canvas shell (Pitfall 1):** `.puzzle-select--canvas-stage` is a fixed container that scrolls itself, using the same viewport and safe-area padding as `.fallback-panel--canvas-stage`, with `overflow-x: hidden`. `SiteFooter`, which holds the source link, renders inline 48 px below the grid. In the form-factor shell the page scrolls normally and the shell footer shows.
- **Sidebar (D-03, UI-D24):** `nav.puzzles-navigation[aria-label="Puzzles"] > a.puzzles-nav-link` sits before `DemoNavigation`. It has no SVG, no preview and no description. Clicking it closes the mobile drawer.

## Tests whose literals changed

- `web/tests/playable-route.test.ts`: "leaves the plural puzzles hash as an unknown scene route" became "parses the plural puzzles hash as the level grid". Added tests for the trailing slash, the extra-segment unknown route, and normalize-without-replacement.
- `web/tests/playable-target.test.ts`: "keys every route kind distinctly" now includes `{kind:"puzzles"}` and expects `"puzzles"`. Added a no-target test, a title test and three `puzzlesNavCurrent` tests.
- `web/tests/puzzle-chrome.test.ts`: added the panel to `PUZZLE_COMPONENTS` and `styles/puzzle-select.css` to `PUZZLE_STYLESHEETS`. Added "keeps data-hud-obstacle off the level grid". The motion test is renamed "in any puzzle stylesheet".
- `web/e2e/puzzle.spec.ts` "keeps levels out of the scene catalog routes": on `#/puzzles` the test now expects the `Puzzles` h2 and 6 `[data-level-id]` cards instead of "Scene not found".
- `web/e2e/shell.spec.ts` "traps repeated forward and reverse Tab navigation inside the mobile drawer": the focus cycle is 27 instead of 26, and both loops run 28 times instead of 27. See Deviations.

## Verification

- `bunx vitest run tests/playable-route.test.ts tests/playable-target.test.ts tests/hash.test.ts` passes 44 tests. `bun run test:unit` passes 104 files and 1540 tests. `bun run typecheck` is clean. `bun run build:app` succeeds. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `CI=true bunx playwright test e2e/puzzle.spec.ts` passed 16 of 16 after Task 2. The final run of `CI=true bunx playwright test e2e/shell.spec.ts e2e/canvas-stage.spec.ts e2e/puzzle.spec.ts` passed 39 of 39 (logs: `target/phase41/e2e-05-task2.log`, `target/phase41/e2e-05-task3-rerun.log`).
- These files are unchanged from the phase base `f277ecad9`: `hash.ts`, `hash.test.ts`, `DemoNavigation.tsx`, `CanvasStage.tsx`, `canvas-stage.css`, `PlayerPanel.tsx`, `scenes.ts`, `readme-svg/plans.ts` and `portrait-bounds.ts`.
- An ad-hoc layout check against `vite preview` on port 4174 found:
  - At 390×844: `scrollWidth` is 390, which equals `innerWidth`. There is one column of 358 px cards, and the section scrolls (scrollHeight 1880 against clientHeight 844).
  - At 1280×800 on canvas stage: three 309 px columns. The form-factor shell also shows three columns.
  - On every layout: 0 `data-hud-obstacle` elements in the page. The canvas stage shows 1 GitHub source link.
- A sidebar check found:
  - `aria-current` is `page` on `#/puzzles`, `true` on `#/puzzle/first-pour`, and absent on `#/scene/dam-break`.
  - On a 390 px phone, "Demos", then dialog "Demos", then link "Puzzles" opens the grid, closes the dialog and sets the title.
- Screenshots (gitignored):
  - `target/phase41/screens/puzzles-390x844.png`
  - `target/phase41/screens/puzzles-390x844-scrolled.png`
  - `target/phase41/screens/puzzles-1280x800.png`
  - `target/phase41/screens/puzzles-ff-1280x800.png`
  - `target/phase41/screens/sidebar-ff-1280x800.png`
  - `target/phase41/screens/sidebar-dialog-390x844.png`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `e2e/shell.spec.ts` could not stay byte-unchanged**
- **Found during:** Task 3, in the first run of the shell spec.
- **Issue:** "traps repeated forward and reverse Tab navigation inside the mobile drawer" hard-codes 26 focusables in the drawer (Dismiss plus 25 scene links). The Puzzles link is the feature D-03 asks for, and it adds a 27th focusable, so 27 reverse tabs land on Dismiss instead of Tesla Valve. D-03 only requires the 25-preview and 25-icon counts to stay valid without edits, and they do: that test passes unedited. The plan's wider "byte-unchanged" rule rested on the same assumption. The only alternative that keeps the file unchanged is to take the link out of the tab order, which breaks the accessibility contract.
- **Fix:** I changed only the cycle literals and their comments (26 to 27 focusables, 27 to 28 tab presses). The test still checks that focus wraps inside the drawer in both directions. No other line in the file changed (4 insertions, 4 deletions).
- **Files modified:** web/e2e/shell.spec.ts
- **Commit:** 792c2f644

**2. [Rule 1 - Test scope] The `data-puzzle-layout` scope test would have run on the grid stylesheet**
- **Found during:** Task 2.
- **Issue:** Adding `puzzle-select.css` to `PUZZLE_STYLESHEETS` made `it.each(PUZZLE_STYLESHEETS)("scopes data-puzzle-layout selectors …")` fail. That test requires at least one `[data-puzzle-layout` selector per sheet, which only makes sense for the HUD stylesheets.
- **Fix:** Added `HUD_STYLESHEETS` (`puzzle.css`, `puzzle-editor.css`) for that scope test. `PUZZLE_STYLESHEETS` is now `[...HUD_STYLESHEETS, "styles/puzzle-select.css"]`, so the motion scan covers the new sheet as the plan asks.
- **Files modified:** web/tests/puzzle-chrome.test.ts
- **Commit:** ea3db8a6f

### Environment note (not a code change)

The "build timestamp" test in `e2e/shell.spec.ts` needs a build made with `VITE_BUILT_AT`, which `scripts/web-build.ts` injects. A plain `bun run build:app` has no `<time>` element, so that test times out. This happens without this plan's changes too. The final e2e run used `VITE_BUILT_AT=<now> bun run build:app`, and all 39 tests passed.

## Known Stubs

None. Every card is wired to the live `session.puzzle.progress` accessors.

## Threat Flags

None. T-41-18: the panel binds text only, its strings come from `PUZZLE_TITLES`, `PUZZLE_CAPTIONS` and the copy constants, and it is in the `innerHTML` and wall-clock scan. T-41-19: the grid matches only one segment, and `#/puzzles/<x>` falls through to the unchanged scene parser, as tested. T-41-21: the container scrolls itself with `overflow-x: hidden`, and `scrollWidth` equals `innerWidth` at 390 px.

## Self-Check: PASSED

Both created files and the 13 modified files exist. Commits 1e6e8cc6c, ea3db8a6f and 792c2f644 are in `git log`.

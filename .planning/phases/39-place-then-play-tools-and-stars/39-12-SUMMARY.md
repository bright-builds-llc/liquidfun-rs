---
phase: 39-place-then-play-tools-and-stars
plan: "12"
subsystem: web-puzzle-hud
tags: [solidjs, puzzle, hud, tray, selection-bar, transport, stars, keyboard, accessibility, css, D-10, D-16, D-17, D-19, D-21, D-23, D-24, D-26, UI-D9, UI-D12, UI-D14, UI-D15, UI-D17, UI-D18]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T01:52:55Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: trayModel/transportModel/selectionBarEnabled/runStarsForState/starThresholdsFor (39-07 hud-format.ts); starRows/starListLabel and puzzleKeyCommand/keyTargetOf (39-03); the final PuzzleController with announcement, toggleArm, placeAtCenter, rotateSelected, removeSelected, clear, cycleSpeed, stop and command (39-09); the editor pointer that sets puzzle-canvas--over-tool/--dragging (39-11)
provides:
  - web/src/components/PuzzleIcons.tsx PlayIcon, StopIcon, ClearIcon, RotateCounterclockwiseIcon, RotateClockwiseIcon, RemoveIcon, PlankIcon, StarIcon
  - web/src/components/PuzzleToolTray.tsx PuzzleToolTray (div.puzzle-tray leaf)
  - web/src/components/PuzzleSelectionBar.tsx PuzzleSelectionBar (div.puzzle-selection-bar leaf)
  - web/src/components/PuzzleTransport.tsx PuzzleTransport (div.puzzle-transport leaf, Play/Stop + Speed + Clear)
  - web/src/components/PuzzleStarList.tsx PuzzleStarList (ul.puzzle-stars)
  - web/src/styles/puzzle-editor.css editor leaf boxes, states, star rows, reached tick, canvas cursors and touch-action
  - canvas tabindex=0 role=application aria-describedby=puzzle-keyboard-help; #puzzle-editor-status polite region; #puzzle-status-line on the invalid status message
affects:
  - 39-13 (e2e selects by role/name; landscape Play is icon-only with aria-label "Play"; deciding state is transient, see Notes for Plan 13)
tech-stack:
  added: []
  patterns:
    - leaves are built once by the panel as JSX factories and passed into PuzzleHud, so the canvas stage and the form-factor row share one instance shape
    - per-index rendering (Index) for tray items and star rows, so per-frame state publishes never recreate focused buttons
    - base editor CSS is the portrait/form-factor look; landscape narrows rail controls to joined 44 px icon buttons
key-files:
  created:
    - web/src/components/PuzzleIcons.tsx
    - web/src/components/PuzzleToolTray.tsx
    - web/src/components/PuzzleSelectionBar.tsx
    - web/src/components/PuzzleTransport.tsx
    - web/src/components/PuzzleStarList.tsx
    - web/src/styles/puzzle-editor.css
  modified:
    - web/src/components/PuzzleHud.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/components/PuzzleResultCard.tsx
    - web/src/styles/puzzle.css
    - web/tests/puzzle-chrome.test.ts
key-decisions:
  - "PuzzlePlayButton and the old PlayIcon were removed: the form-factor path now uses PuzzleTransport too, and the Play rules moved from puzzle.css to puzzle-editor.css"
  - "Tray items and star rows render with Index, not For: trayModel returns fresh objects every publish, and For would recreate the focused tray button after an Enter placement"
  - "Tray names show via data-kind-count (1 kind everywhere, 2 kinds in portrait and form-factor), not by measuring width"
  - "Joined landscape groups are three 44 px buttons with border-left-width 0 on the 2nd and 3rd, giving 132 px with 1 px shared seams"
  - "The focus rescue moves focus to Play/Stop only when document.activeElement is body, detached or aria-disabled, so Space on the canvas keeps canvas focus"
  - "The status line id lives on the result message p only while its tone is status; Play's aria-describedby is set only while a disabled reason exists, so the reference always resolves"
requirements-completed: [TOOL-01, TOOL-04, TOOL-06, STAR-01]
duration: 13min
completed: 2026-10-13
---

# Phase 39 Plan 12: Editor HUD, Transport, Star List and Keyboard Routing Summary

**The puzzle HUD now has the Phase 39 controls from the approved UI-SPEC: a tool tray with remaining counts and arming, a selection bar (rotate counterclockwise and clockwise 15°, Remove), a transport (Play/Stop on one element, speed 1×/2×/4×, Clear), and a star checklist and run rows in the fixed result slot, with the "Filled! Going for Brim at 90%" deciding state. The canvas is a focusable `role="application"` described by keyboard help. Panel-level keys route through `puzzleKeyCommand` to `controller.command`. Every leaf keeps one fixed box per layout in every phase.**

## Performance

- **Duration:** about 13 min
- **Started:** 2026-10-13T01:40:00Z
- **Completed:** 2026-10-13T01:52:55Z
- **Tasks:** 2 of 2
- **Files:** 6 created, 5 modified

## Accomplishments

### Task 1: leaves, icons and styles

- **Icons:** `PuzzleIcons.tsx` holds hand-drawn inline SVGs: a 24-unit viewBox in a 22 px box, `currentColor` strokes at width 2 with round caps and joins, and filled shapes for Play, Stop and Plank. The star has outer radius 10 and inner radius 4.5, centred at 12,12. All icons are `aria-hidden`.
- **Tray (`PuzzleToolTray`):** a `data-hud-obstacle` leaf, `role="group"` named "Tools". Each item is a button with:
  - `aria-pressed`, `aria-disabled`, an accessible name ("Plank, 2 left" / "Plank, none left") and a title ("Place a plank (2 left)");
  - `data-tool-kind` and `data-remaining`.
  - Click or Space arms the kind, and pressing again disarms it. Enter calls `preventDefault` and then places at the centre.
- **Selection bar (`PuzzleSelectionBar`):** a `role="toolbar"` named "Selected tool" with three icon buttons, each titled with its name: "Rotate counterclockwise 15°" (+1), "Rotate clockwise 15°" (−1) and "Remove selected tool". The buttons are `aria-disabled` unless the puzzle is in building with a selection.
- **Transport (`PuzzleTransport`):** a `role="group"` named "Run controls".
  - **Play/Stop** is one element, so focus survives. It is accented only when `playAccent`. While a disabled reason exists, it gets `aria-describedby="puzzle-status-line"` and the reason as its title.
  - **Speed** shows `1×`, named "Speed 1×", with the title "Speed 1×. Press to change.".
  - **Clear** is named "Clear all tools". It shows the text "Clear" in portrait and form-factor, and the eraser icon in landscape.
- **Star list (`PuzzleStarList`):** `ul.puzzle-stars` with `role="list"`, one row each for Win, Brim and Thrifty. Each row has a glyph and label that are both `aria-hidden`, a visually hidden span with the full row text and state phrase, and `data-criterion` / `data-glyph` attributes.
- **`puzzle-editor.css` (new, 312 lines):**
  - every new leaf box in landscape, portrait and form-factor;
  - joined 44 px landscape groups;
  - armed, disabled and empty-count states;
  - star rows, 16 px in landscape (12 px glyph, 12/600) and 20 px in portrait and form-factor (14 px glyph, 14/400);
  - the deciding and status-line colors and the reached-tick halo;
  - canvas `touch-action: none`, the `copy`/`grab`/`grabbing` cursors and the focus ring at `outline-offset: -4px`;
  - icon buttons at `padding: 0` and portrait transport buttons at `padding: 0 8px`.
- **`puzzle.css` (420 → 391 lines):**
  - portrait result card at bottom `inset + 104px`, height 160;
  - portrait transport 236 px wide;
  - landscape result actions: Next across the full width, then Retry and Edit at 56 px each (UI-D15);
  - form-factor result height 160;
  - the old Play rules moved out.

### Task 2: mounting, result card, canvas, keys and focus

- **`PuzzleHud`:** DOM order is title, meter, timer, tray, selection bar, transport, result card, options, legend. Every leaf is always mounted. Meter ticks carry `data-reached`. `PuzzlePlayButton` and its export were removed.
- **`PuzzleResultCard`:**
  - **Loading, goal and hint:** the plain message.
  - **Status tone:** gets `id="puzzle-status-line"` and the `#F07178` status class.
  - **Deciding:** shows `<span class="puzzle-result-outcome">Filled!</span> <span class="puzzle-result-pending">Going for Brim at 90%</span>`.
  - **Outcome:** spans as in Phase 38.
  - **Stars:** `PuzzleStarList` shows in every phase except loading. Thresholds come from `starThresholdsFor(state, levelInfo.parTools)` and rows from `starRows(…, maybeBestStars, runStarsForState(…))`. The label is "Stars this run" once ended.
  - No component reads `editor.maybeInspection` (UI-D14).
- **`PuzzlePanel` (394 lines):**
  - imports `puzzle-editor.css`;
  - the canvas has `tabindex="0"`, `role="application"`, the unchanged Phase 38 `aria-label`, `aria-describedby="puzzle-keyboard-help"` and the `puzzle-canvas--armed` class while armed in building;
  - a visually hidden `#puzzle-keyboard-help` paragraph holds the exact copy;
  - `#puzzle-editor-status` (polite, atomic) shows `controller.announcement()`;
  - the section `onKeyDown` ignores keys while the options drawer is open. Otherwise it maps through `puzzleKeyCommand` and `keyTargetOf`, and calls `preventDefault` only when `controller.command` returns true.
- **Focus rules:**
  - Entering ended focuses the primary result action.
  - Entering building or running focuses Play/Stop only when the active element is body, detached or `aria-disabled`.
  - Remove keeps its own focus.
- **Form-factor control row:** transport, tray, selection bar, then the result card.

## Task Commits

1. **Task 1:** `9a56f6ccb` (feat). Tool tray, selection bar, transport, star list, icons and editor styles.
1. **Task 2:** `c1108e43a` (feat). Mounts the editor HUD, star rows, focusable canvas and keyboard routing.

## Verification

- **Task 1 checks:**
  - `bunx tsc --noEmit` exited 0.
  - The `transition|animation|innerHTML` grep over the new files printed nothing.
  - The ` disabled[ =>]` count was 0 in all three control files.
  - The `setTimeout|setInterval|Date.now|performance.now|tilt` grep over `Puzzle*.tsx` printed nothing, after one icon comment was reworded from "tilted" to "turned".
  - `puzzle.css` is 391 lines (≤ 440). `bun scripts/bright-builds-check.ts all` reported findings=0.
- **Task 2 checks:**
  - `bunx vitest run tests/puzzle-chrome.test.ts`: 20 passed.
  - `bun run test:unit`: 94 files, 1267 tests passed.
  - `bunx tsc --noEmit` exited 0, and `bun run build:app` built.
  - bright-builds-check reported findings=0. `PuzzlePanel.tsx` is 394 lines (≤ 420).
- **Acceptance greps:**
  - These match: `role="application"`, `puzzle-keyboard-help`, `puzzle-editor-status` and `puzzleKeyCommand` in `PuzzlePanel.tsx`; `puzzle-status-line` and `PuzzleStarList` in `PuzzleResultCard.tsx`.
  - `maybeInspection` in `Puzzle*.tsx` prints nothing.
  - The new-component name count in the chrome test is 8 (≥ 5).
- **Real-browser check:** a temporary Chromium spec ran and was deleted afterwards; `git status` was clean apart from the orchestrator's `config.json`.
  - **Leaf boxes** stayed identical across building, after edits and running:
    - **1280×800:** tray 8,236 136×44; selection bar 1136,712 136×44; transport 8,748 136×44; result 136×220.
    - **844×390:** tray 8,236; selection bar 700,302; transport 8,338.
    - **390×844:** tray 8,740 218×44; selection bar 234,740 148×44; transport 8,792 236×44; result 8,572 374×160.
    - **360×780:** tray 188 px wide.
  - **Safe frame:** `overlappingObstacles` was `[]` and the world-frame fallback was `false` in all four layouts.
  - **Editing flow:**
    1. Clicking the tray item set `aria-pressed` and the armed cursor class.
    1. Enter on the tray item placed `plank:10,64,0` and kept focus on the item.
    1. Rotate clockwise gave `plank:10,64,23`.
    1. Two ArrowLeft presses on the canvas gave `plank:8,64,23`.
    1. Space started the run with focus kept on the canvas, and the Stop button appeared.
    1. Stop kept the placements and focused Play.
    1. Clear emptied the placements and announced "All tools cleared.".
  - **Seeded solution at 15 Hz:**
    - **Deciding:** the card read "Filled! Going for Brim at 90%". Rows were Win and Thrifty `earned` and Brim `none`. The Win tick had `data-reached="true"`.
    - **Ended:** the card read "Filled! 90% of 60%", the list was labelled "Stars this run" and all three rows were `earned`.
  - **Screenshots** in the session scratchpad showed the joined landscape groups, the landscape card with Next over Retry and Edit, the portrait 160 px card and the form-factor row.
- **Required e2e:** `CI=true bunx playwright test e2e/hud-safe-frame.spec.ts e2e/puzzle.spec.ts --project=chromium` gave **79 passed, 1 failed**.
  - The only failure is the known D-10 literal in "wins with the recorded solution through the test seam": expected `"Filled! 76% of 60%"`, received `"Filled! 90% of 60%"`. Plan 13 owns that literal edit.
  - All HUD safe-frame cases passed, as did both camera-stability tests (building → running → ended) and the scene-chrome-off and form-factor tests.

## Phase 38 Chrome Assertions Changed (UI-D17)

The `web/tests/puzzle-chrome.test.ts` changes, as the plan requires:

- **HUD leaf test:** "marks every HUD leaf in PuzzleHud with data-hud-obstacle" no longer lists `puzzle-transport`, because the transport leaf now lives in `PuzzleTransport.tsx`. A new test checks `data-hud-obstacle` on the `puzzle-tray`, `puzzle-selection-bar` and `puzzle-transport` opening tags in their component files.
- **Sizing test:** "gives the meter, timer, result, play and legend slots a fixed width and height" became "… result, transport and legend slots …". It now reads `puzzle.css` plus `puzzle-editor.css` and checks `.puzzle-transport` instead of `.puzzle-play`, because Play/Stop is a button inside the fixed transport leaf, not a leaf.
- **Motion test:** "has no transitions or animations" now covers both puzzle stylesheets.
- **Scoping test:** "scopes every data-puzzle-layout selector to the canvas stage" runs once per stylesheet (`it.each`).
- **Role test:** no Phase 38 test queried the puzzle canvas `role="img"`. A new test asserts `role="application"`, `tabindex="0"`, the keyboard-help reference and the absence of `role="img"`.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Index instead of For for tray items and star rows**
- **Found during:** Task 1
- **Issue:** `trayModel` returns new item objects on every state publish. `<For>` keys by object identity, so it would recreate the tray button on every frame. That drops focus after an Enter placement, which breaks the spec's "focus survives the last Enter-placement".
- **Fix:** both lists render with `<Index>`. The browser check confirmed the tray item keeps focus after Enter.
- **Commit:** `9a56f6ccb`

**2. [Rule 3 - Blocking] Stylesheet import and sizing test moved into Task 1**
- **Found during:** Task 1
- **Issue:** Moving the Play rules out of `puzzle.css` made the Phase 38 sizing test fail on `.puzzle-play` before Task 2.
- **Fix:** Task 1 imports `puzzle-editor.css` in `PuzzlePanel.tsx` and points the sizing test at both stylesheets and the transport leaf, so every commit keeps the suite green.
- **Commit:** `9a56f6ccb`

**3. [Rule 2 - Correctness] Editor leaves do not shrink in the form-factor row**
- **Found during:** Task 1
- **Issue:** The form-factor `.control-row` is a wrapping flex row, and items shrink by default.
- **Fix:** the tray, selection bar and transport get `flex: 0 0 auto`, so their fixed form-factor boxes hold.
- **Commit:** `9a56f6ccb`

**4. Comment wording**
- The eraser icon comment said "tilted", which matches the forbidden `tilt` substring checked by the chrome test and the acceptance grep. It now says "turned".

## Notes for Plan 13

- **Transient deciding state:** under the synthetic 15 Hz clock, the whole solution run takes about 2 s of wall time. The deciding window (steps about 203–260) lasts only about 300 ms. A temporary spec polled with `expect(main[data-puzzle-deciding="true"]).toBeVisible()` and missed it in 3 of 8 repeats.
  - An in-page `MutationObserver` saw `"Filled! Going for Brim at 90%"` on 8 of 8 repeats, so the product renders the state every time.
  - A deciding-state e2e should record it in the page, or use a slower clock, rather than rely on Playwright polling.
- **Landscape transport selectors:** in landscape, Play/Stop and Clear are icon-only buttons named "Play"/"Stop" and "Clear all tools". Role-and-name selectors keep working in every layout.

## Known Stubs

None. The star list is hidden while loading, and until `levelInfo` loads. This is the spec's reserved-space loading state inside the fixed card, not a stub.

## Threat Flags

None. The planned mitigations are in place:

- **T-39-35:** text bindings only. The chrome test's `innerHTML` and wall-clock grep now covers all five new components.
- **T-39-36:** Space and Enter on buttons are skipped by `keyTargetOf` and `puzzleKeyCommand`. Tab cycles only from the canvas and falls through when `command` returns false. Keys are ignored while the options drawer is open.
- **T-39-37:** fixed leaf boxes, always mounted, were measured identical across building, editing and running in four viewports. The camera-stability e2e passed.

## Requirements

- **Marked complete:** TOOL-01 (the tray with remaining counts), TOOL-04 (Play, Stop keeping placements, Clear), TOOL-06 (keyboard place, nudge, rotate, remove and play) and STAR-01 (the three criteria visible before Play). Each was checked in a real browser above.
- **Left pending for Plan 13's e2e:**
  - **TOOL-02:** the touch flow needs the `hasTouch` project.
  - **TOOL-07:** the 1×/2×/4× same-outcome e2e is still to come.
  - **STAR-02:** best-of persistence across Retry and Stop has not been checked in the browser here.

## Self-Check: PASSED

All 6 created files exist, and both task commits (`9a56f6ccb`, `c1108e43a`) are in git history.

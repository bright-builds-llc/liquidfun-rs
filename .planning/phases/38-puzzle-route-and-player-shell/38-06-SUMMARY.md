---
phase: 38-puzzle-route-and-player-shell
plan: "06"
subsystem: web-puzzle-chrome
tags: [typescript, solid, puzzle, hud, css, accessibility, PUZ-02, PUZ-04, GOAL-01, GOAL-03]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T03:03:28.761Z"
requires:
  - web/src/puzzle/format.ts fillMeterModel, timerText, resultCardModel, resultActionsModel (Plan 02)
  - web/src/puzzle/controller.ts PuzzleController (Plan 02)
  - web/src/catalog/puzzle-levels.ts puzzleTitleForId, maybeNextPuzzleId, FIRST_PUZZLE_ID, puzzleHash (Plan 01)
  - web/src/player/hud-obstacles.ts createHudObstacles (unchanged)
  - web/src/components/ScaleLegend.tsx, SiteHeader.tsx GitHubSourceLink, SiteFooter.tsx, ui/drawer.tsx (unchanged)
provides:
  - web/src/components/PuzzleHud.tsx PuzzleHud, PuzzleHudProps, PuzzleTitle, PuzzleFillMeter, PuzzleTimer, PuzzlePlayButton
  - web/src/components/PuzzleResultCard.tsx PuzzleResultCard, PuzzleResultCardProps
  - web/src/components/PuzzleOptionsDrawer.tsx PuzzleOptionsTrigger, PuzzleOptionsSheet, PuzzleRenderModeSelect
  - web/src/components/PuzzlePanel.tsx PuzzlePanel, PuzzlePanelProps
  - web/src/components/PuzzleFallbackPanel.tsx PuzzleFallbackPanel, PuzzleFallbackPanelProps
  - web/src/puzzle/fallback-text.ts MAX_RAW_DISPLAY_LENGTH, truncateRaw
  - web/src/styles/puzzle.css (imported once by PuzzlePanel)
affects:
  - 38-07 (mounts PuzzlePanel / PuzzleFallbackPanel in PlaygroundStage)
  - 38-08 (Chromium e2e drives the rendered chrome)
  - 38-09 (greps for innerHTML and scene chrome in puzzle components)
tech-stack:
  added: []
  patterns:
    - fixed CSS boxes per layout for every HUD leaf; content swaps inside, so the obstacle set never changes
    - layout attribute derived from viewport geometry only, and layout CSS scoped to the canvas-stage class
    - source-inspection tests in node Vitest for component rules, pure view models for behavior
key-files:
  created:
    - web/src/components/PuzzleHud.tsx
    - web/src/components/PuzzleResultCard.tsx
    - web/src/components/PuzzleOptionsDrawer.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/components/PuzzleFallbackPanel.tsx
    - web/src/puzzle/fallback-text.ts
    - web/src/styles/puzzle.css
    - web/tests/puzzle-chrome.test.ts
  modified: []
key-decisions:
  - "Fill meter rows use the 4px xs gap, not 8px: a 64px chip with 8px padding and a 1px border holds 24px + 16px rows only with a 4px gap"
  - "Outcome lines use a 22px line height so message plus actions fit the fixed 136x220 (landscape) and 92px (portrait) result boxes with their border"
  - "The failure overlay container stays mounted (empty when not failed), so its aria-live=assertive region exists before the heading appears"
  - "Result actions fill the chip content box (width 100%) instead of a literal 120px, which would overflow the 1px border"
requirements: [PUZ-02, PUZ-04, GOAL-01, GOAL-03]
duration: 7min
completed: 2026-10-12
---

# Phase 38 Plan 06: Puzzle player chrome Summary

**This plan adds the puzzle player UI as new Solid components that read only `PuzzleController` and the pure view models. It includes a fill meter (`role=meter`, "Cup fill"), a simulation timer (`role=timer`, "Simulation time left"), a Play transport that uses `aria-disabled`, a result card that announces politely and offers Next, Retry and Edit, and a "Puzzle options" drawer that holds only the render mode, the source link and the footer. Every leaf carries `data-hud-obstacle` and has a fixed box for both the landscape rails and the portrait bands. The unknown-level fallback shows the raw id as truncated text only. A 14-test source-inspection suite checks these chrome rules.**

## Performance

- **Duration:** about 7 min (02:57Z to 03:04Z)
- **Tasks:** 3 of 3
- **Files:** 8 created, 0 modified

## Accomplishments

### Task 1: HUD leaves, result card and stylesheet (commit `8ef18f3f9`)

- `PuzzleHud.tsx`:
  - `PuzzleTitle` is `h2#player-title` with a `title` attribute.
  - `PuzzleFillMeter` shows the percent, the ` / 60%` target, the bar, and ticks with `data-star`.
  - `PuzzleTimer` is a `div` with `role="timer"`, not an `<output>`.
  - `PuzzlePlayButton` has the accessible name "Play". It is accented in building; in every other phase it keeps focus through `aria-disabled="true"` and ignores clicks.
  - `PuzzleHud` renders, in DOM and focus order: title, meter, timer, transport, result card, options slot, then an always-present 100x28 legend slot.
- `PuzzleResultCard.tsx`:
  - The root is `section.puzzle-result` with `data-hud-obstacle` and `data-outcome`.
  - The message `p` uses `aria-live="polite"` and `aria-atomic="true"`. Outcome spans are joined by a literal space, so `textContent` equals the model `text`.
  - Actions appear only at the outcome. Next is an accent link when a next level exists. Otherwise it is a native `disabled` button named "Next: No next level yet" with the title "No next level yet". Retry is accented when it is the primary action, and Edit is a plain button.
  - `primaryRef` is attached to the primary action.
  - All copy comes from `resultCardModel` and `resultActionsModel`.
- `puzzle.css`:
  - Inset variables use `env(safe-area-inset-*)`.
  - The chip surface is `rgb(21 29 40 / 86%)` with a `#2A3441` border and `tabular-nums`.
  - Landscape rails and portrait bands follow the UI-SPEC boxes exactly. Every layout selector is prefixed with `.puzzle-panel--canvas-stage[data-puzzle-layout=…]`.
  - Form-factor chips stay in normal flow at the portrait sizes (meter 232x64, timer 104x64, result `min(100%, 480px)`x92, Play 136x44), with the legend slot absolute at 8/8.
  - The won bar is `#3DDC97` and the lost bar is `#F07178`. Star ticks are amber `#E6C35C` and the fill is `#4DA3FF`.
  - Canvas cursor is `default`. There is no `transition` or `animation`.
  - Play rules use `.puzzle-play.product-control`, so they beat the global `.product-control`/`button` rules regardless of CSS order.

### Task 2: panel, options drawer and fallback (commit `bf3eca73e`)

- `PuzzleOptionsDrawer.tsx`:
  - The `DrawerTrigger` is `.hud-button` and is named "Puzzle options". It uses the Options icon path copied from CanvasStage.
  - The sheet reuses the `canvas-controls-sheet` look and contains the children (the render-mode select), then `GitHubSourceLink`, then `SiteFooter`.
  - `PuzzleRenderModeSelect` is the "Particles" select built from `RENDER_MODE_GROUPS` and `maybeParseRenderMode`.
- `PuzzlePanel.tsx`:
  - The root is `section.puzzle-panel.player-panel` with the canvas-stage classes, `data-pixels-per-meter` and `data-puzzle-layout`.
  - `data-puzzle-layout` comes from a signal. It starts from `window` and is then updated by a `ResizeObserver` on `.viewport-frame`, using portrait iff `clientHeight > clientWidth`.
  - The canvas markup mirrors PlayerPanel, with the aria-label "First Pour puzzle: Rust particles pour toward a glass cup."
  - The loading overlay is "Loading First Pour" / "Starting the physics engine.". The failure overlay shows "This level could not start", the body text, a "Try again" button that calls `controller.tryAgain()`, and `details.failure-debug` "Failure details".
  - In canvas stage, `createHudObstacles` is wired to `onHudObstaclesChange`, and the options drawer is controlled and closes on level change.
  - The form-factor layout has a status row (title, meter, timer), the frame with the legend slot, a control row (Play and the result card) and an options row with the render-mode select.
  - Focus management follows UI-D8. `focusin` and `focusout` on the root track whether focus is inside, and `focusout` clears that only for a non-null outside `relatedTarget`. A deferred effect on the phase focuses Play in building or running, and focuses the primary action through `queueMicrotask` in ended.
- `fallback-text.ts`: `MAX_RAW_DISPLAY_LENGTH = 80` and `truncateRaw`.
- `PuzzleFallbackPanel.tsx`:
  - The heading is "Level not found". The body shows the raw id truncated, and falls back to the empty-id body when the raw id is empty.
  - Links: "Open First Pour" goes to `#/puzzle/first-pour` and "Back to scenes" goes to `DEFAULT_SCENE_HASH`.
  - In canvas stage only, it adds the Puzzle options drawer without the render-mode select.

### Task 3: source-inspection test (commit `7fa35a97f`)

`web/tests/puzzle-chrome.test.ts` has 14 tests, each following Arrange/Act/Assert:

- obstacle markers on the six HUD leaves and on the result slot
- `<ScaleLegend` present in both layouts
- no scene-only chrome, including any `tilt` text
- the timer's role and name, and no `<output`
- no `innerHTML` or wall-clock APIs
- no motion in the CSS
- fixed width and height for the meter, timer, result, play and legend slots
- every `[data-puzzle-layout` preceded by `.puzzle-panel--canvas-stage`
- a 100x28 legend slot
- three `truncateRaw` cases (80 kept, 81 cut with an ellipsis, `<img src=x>` verbatim)
- the fallback using `truncateRaw(props.maybeRaw)`

## PuzzlePanelProps (what Plan 07 must pass)

```typescript
export type PuzzlePanelProps = {
  readonly canvasStage: boolean;
  readonly levelId: PuzzleId;
  readonly controller: PuzzleController;           // createPuzzleController(...) from Plan 05
  readonly status: PlayerStatus;                   // "failed" shows the failure overlay
  readonly maybeFailureDetails: string | undefined;
  readonly renderMode: RenderMode;
  readonly onRenderModeChange: (mode: RenderMode) => void;
  readonly maybePixelsPerMeter: number | undefined;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly assignParticleSurface: (canvas: HTMLCanvasElement) => void;
  readonly onHudObstaclesChange: (obstacles: readonly ScreenRect[]) => void;
};
// PuzzleFallbackPanel props: { canvasStage: boolean; maybeRaw: string }
```

## Verification

- `bun run typecheck` passed after each task.
- `bunx vitest run tests/puzzle-chrome.test.ts tests/scale-legend.test.ts tests/hud-sliders-toggle.test.ts`: 3 files, 44 tests passed. `puzzle-chrome` contributes **14 tests** (minimum 8).
- `bun run test:unit`: **73 files, 777 tests** passed.
- `bun scripts/bright-builds-check.ts all`: findings=0. The largest new file is `puzzle.css` at 420 lines, under the 628-line limit.
- Task 1 greps:
  - `data-hud-obstacle` appears 6 times in PuzzleHud and once in PuzzleResultCard.
  - `<output` appears 0 times.
  - The meter, timer, `aria-live` and `resultCardModel(` greps pass.
  - The unscoped-layout-selector grep is empty.
- Task 2 greps:
  - The forbidden scene-chrome and `tilt` grep is empty.
  - `<ScaleLegend` appears twice (once in PuzzleHud, once in PuzzlePanel).
  - `innerHTML` appears 0 times.
  - `git diff --quiet 589cb0fb9 -- PlayerPanel.tsx CanvasStage.tsx FallbackPanel.tsx canvas-stage.css` exits 0.
- Threat mitigations:
  - T-38-18: text binding only, truncated to 80 characters, fixed link hashes, enforced by a source test.
  - T-38-20: the source link and footer are in the Puzzle options drawer, in both the panel and the fallback.

## Deviations from Plan

### D-22 test delivery (planned deviation)

D-22 asks for "Vitest component tests". They are delivered as pure view-model tests (Plan 02's `format.ts`, 33 tests in `tests/puzzle-format.test.ts`) plus source inspection in `tests/puzzle-chrome.test.ts`. Vitest runs in node with no DOM library, and this phase adds none. The Plan 08 Chromium e2e proves how the rendered components behave.

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed-box arithmetic in the UI-SPEC did not fit its own borders**
- **Found during:** Task 1
- **Issue:** The chips use `box-sizing: border-box`, an 8px padding and a 1px border. Two boxes overflowed their fixed height:
  - The meter (8px row gap, 24px and 16px rows) needs 66 to 72px inside a 64px chip.
  - The result card's outcome lines at 20px/1.2 need 222px in the 220px landscape card and 76px in the 92px portrait card.
  
  `overflow: hidden` would have clipped the bar or the Edit button.
- **Fix:**
  - The meter row gap uses the 4px xs token (the UI-SPEC lists xs as the "gap between text and bar inside a chip").
  - Outcome lines use a 22px line height.
  - Result actions fill the content box at `width: 100%` (118px) instead of a literal 120px.
- **Files modified:** web/src/styles/puzzle.css
- **Commit:** 8ef18f3f9

**2. [Rule 1 - Bug] CSS specificity ties**
- **Found during:** Task 1
- **Issue:** Two rules lost or could lose to other rules:
  - The landscape, portrait and form-factor goal-message rules had the same specificity as the outcome-message rule and came later, so they would have shrunk outcome text to 14px or 16px.
  - `.puzzle-play` tied with the global `.product-control`, so the winner depended on CSS bundle order.
- **Fix:**
  - The layout message rules now target `.puzzle-result:not([data-outcome])`.
  - The Play rules use `.puzzle-play.product-control`.
- **Files modified:** web/src/styles/puzzle.css
- **Commit:** 8ef18f3f9

**3. [Rule 2 - Accessibility] Assertive region mounted before failure**
- **Found during:** Task 2
- **Issue:** The plan's markup placed the `aria-live="assertive"` div inside the conditionally rendered failure overlay. That conflicts with the UI-SPEC rule that the region "stays mounted and empty until failure". A live region inserted together with its content is often not announced.
- **Fix:** The `.empty-state.puzzle-failure` container and its assertive div are always mounted. When there is no failure the container is empty and does not intercept pointers. The heading, body, Try again and details render inside it only on failure.
- **Files modified:** web/src/components/PuzzlePanel.tsx
- **Commit:** bf3eca73e

**Process notes:**
- Task 3 is marked `tdd="true"`, but the plan orders it after the components it inspects, so there was no failing RED run. The 14 tests passed on the first run against the finished files.
- The web package has no formatter or lint script, so typecheck, Vitest and bright-builds-check were the pre-commit checks that applied.
- This plan stages only its own files; `.planning/config.json` is left unstaged.

## Known Stubs

None. The components are complete and ready for Plan 07 to mount. Nothing renders them until `PlaygroundStage` wires them in, which is intended and is Plan 07's scope.

## Self-Check: PASSED

- FOUND: web/src/components/PuzzleHud.tsx, PuzzleResultCard.tsx, PuzzleOptionsDrawer.tsx, PuzzlePanel.tsx, PuzzleFallbackPanel.tsx, web/src/puzzle/fallback-text.ts, web/src/styles/puzzle.css, web/tests/puzzle-chrome.test.ts
- FOUND: commits 8ef18f3f9, bf3eca73e, 7fa35a97f

---
phase: 40-timed-valve-and-teaching-levels
plan: "05"
subsystem: web-puzzle-hud
tags: [typescript, solid, puzzle, hud, caption, hint, localstorage, playwright, LEVEL-03, LEVEL-01]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T05:26:01.202Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-04 HUD boxes per layout (landscape bottom-centre selection bar, portrait bar and raised result card) and overlappingLeaves"
provides:
  - PUZZLE_CAPTIONS, FIRST_LEVEL_HINT and puzzleCaptionForId in puzzle-levels.ts
  - first-hint-preference.ts (FIRST_HINT_STORAGE_KEY, loadHintDismissed, persistHintDismissed, createFirstHintMemory, firstHintMemory)
  - caption.ts captionModel(levelId, hintDismissed)
  - PuzzleCaption leaf in the canvas-stage HUD and the form-factor status row
  - e2e puzzlePath(id), fixturePath(file), worldToPage/gridToPage levelId parameter
  - e2e/puzzle-hint.spec.ts in test:puzzle
affects:
  - 40-06..40-10 (each new level must add a PUZZLE_CAPTIONS entry; the Record type enforces it; e2e can target any level id)
  - Phase 41 progress.ts (may absorb first-hint-preference.ts)
tech-stack:
  added: []
  patterns:
    - one always-mounted HUD leaf whose children swap; the box never changes
    - lazy storage read on first dismissed() call, boolean only turns true
key-files:
  created:
    - web/src/player/first-hint-preference.ts
    - web/src/puzzle/caption.ts
    - web/src/components/PuzzleCaption.tsx
    - web/tests/puzzle-first-hint.test.ts
    - web/tests/puzzle-caption.test.ts
    - web/e2e/puzzle-hint.spec.ts
  modified:
    - web/src/catalog/puzzle-levels.ts
    - web/src/components/PuzzleHud.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/styles/puzzle.css
    - web/tests/puzzle-chrome.test.ts
    - web/e2e/puzzle-helpers.ts
    - web/e2e/puzzle-editor-helpers.ts
    - web/package.json
key-decisions:
  - "Landscape caption box is 48px tall, not 44: the chip's 1px border leaves 42px of content, so a 44px Dismiss would be clipped; 48 still leaves a 4px gap above the title and result card at inset+52px"
  - "Placing the first tool dismisses through an effect on building state with tools.length > 0, so the test-placement seam also dismisses the hint in that test's context"
  - "A focused Dismiss button hands focus to the scene canvas when it is removed"
metrics:
  duration: 7min
  completed: 2026-10-13
  tasks: 3
  files: 14
---

# Phase 40 Plan 05: Caption leaf and one-time first-level hint Summary

Every puzzle route now mounts one fixed caption HUD leaf that shows the level's one-line caption. On first-pour, that same leaf first shows the placement hint "Tap the plank in the tray, then tap the canvas to place it." with a 44 px "Got it" button. The hint goes away for good when the player presses that button or places the first tool. The dismissal is stored under `liquidfun.puzzle-first-hint.v1`, and every storage access is guarded. If storage is blocked, the dismissal still holds in memory for the page session. Dismissing never moves the camera.

## Task 1: Captions, guarded storage and the caption model (commit 4069947f6)

- `puzzle-levels.ts`: adds `PUZZLE_CAPTIONS` with "Water runs downhill. Tilt a plank to pour it into the cup." for first-pour, plus `FIRST_LEVEL_HINT` and `puzzleCaptionForId`. `puzzles.ts` is untouched; a Rust `include_str!` test parses that file.
- `first-hint-preference.ts`:
  - Only the exact `"dismissed"` token counts as dismissed.
  - The provider (`() => window.localStorage`) is called inside each `try`.
  - `createFirstHintMemory` does not read storage until `dismissed()` is first called. Its flag can only change from false to true.
- `caption.ts`: `captionModel` shows the hint plus Dismiss only when the level is `FIRST_PUZZLE_ID` and the hint has not been dismissed.
- Tests, 18 in total:
  - Exact token, null and `"yes"`.
  - A throwing provider and a throwing `getItem`.
  - A throwing `setItem` and a throwing provider on persist.
  - The memory falls back to in-memory state, an already-dismissed value is read at start, and storage is read lazily.
  - The caption model.
  - Every caption is non-empty, one line, and at most 80 characters.

## Task 2: Caption leaf and dismissal wiring (commit cd102b0af)

- `PuzzleCaption.tsx` renders `div.puzzle-caption.puzzle-chip[data-hud-obstacle][data-hint]`, which contains `p.puzzle-caption-text[title]`. While the hint shows, it also contains `button.product-control.puzzle-caption-dismiss[aria-label="Dismiss hint"]` with the label "Got it".
- `PuzzleHud` takes a new `caption` prop and renders it right after the timer, so Dismiss comes before the tray in focus order.
- `PuzzlePanel`:
  - Holds a `hintDismissed` signal, seeded from `firstHintMemory.dismissed()`.
  - `dismissHint()` saves the dismissal and moves focus to the canvas if Dismiss had focus.
  - An effect dismisses the hint once first-pour is building with at least one tool placed.
  - The caption appears in the canvas-stage HUD and in the desktop `status-row`, after the timer.

### Final caption boxes

| Layout | Box |
| --- | --- |
| Landscape canvas stage | `top: inset; left: inset+52px; right: inset+52px; height: 48px`; text 13/16 px, 2-line clamp |
| Portrait canvas stage | `top: inset+124px; left: inset; right: inset; height: 52px`; text 14/18 px, 2-line clamp |
| Form-factor shell | `position: static; width: min(100%, 480px); height: 52px` |

Dismiss is at least 64×44 in every layout. The canvas-stage rule `.puzzle-hud .puzzle-caption button` restores `pointer-events: auto`.

## Task 3: Hint e2e and level-aware helpers (commit 782683bb5)

- `puzzle-helpers.ts`:
  - Adds `puzzlePath(id)`; `PUZZLE_PATH` is now `puzzlePath("first-pour")`.
  - Adds `fixturePath(file)`; `SOLUTION_FIXTURE` now uses it.
- `puzzle-editor-helpers.ts`: `worldToPage` and `gridToPage` take a trailing `levelId = "first-pour"`, so existing callers do not change.
- `puzzle-hint.spec.ts` has 6 tests:
  - At each of 1280×800, 390×844 and 844×390:
    - The hint shows and the Dismiss box is at least 44 px tall.
    - After Dismiss, the caption shows.
    - The `worldFrame` and `pixelsPerMeter` strings are identical before and after.
    - No overlapping leaves or obstacles, before or after.
  - After a reload, the hint stays dismissed and storage holds `"dismissed"`.
  - Arming the plank and clicking grid (8, 64) dismisses the hint, and it stays dismissed after a reload.
  - With a throwing `localStorage` getter, Dismiss still hides the hint, and it stays hidden after a round trip through `#/scene/dam-break`.
- `test:puzzle` now includes the new spec.

### Screenshots

- `target/phase40/screens/hint-1280x800-before.png`, `hint-1280x800-after.png`
- `target/phase40/screens/hint-390x844-before.png`, `hint-390x844-after.png`
- `target/phase40/screens/hint-844x390-before.png`, `hint-844x390-after.png`

### Verification

- `bunx tsc --noEmit` is clean, and `bun run test:unit` passes 97 files and 1373 tests.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- After `just web-wasm` and `bun run build:app`:
  - `CI=true bun run test:puzzle` passes 38/38.
  - The unedited `e2e/hud-safe-frame.spec.ts` passes 64/64.
- `hud-safe-frame.spec.ts`, `tests/{hash,portrait-bounds,readme-svg}.test.ts` and `crates/liquidfun` have no diff from the phase base.
- `localStorage` does not appear in `src/puzzle`, `src/components/Puzzle*.tsx` or `src/player/puzzle-*.ts`.
- No Phase 38/39 e2e literals changed. The caption leaf shrinks the first-pour safe frame, but every existing spec reads the published camera.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Layout] Landscape caption is 48 px tall instead of 44 px**
- **Found during:** Task 2
- **Issue:** The chip is `border-box` with a 1 px border, so a 44 px box has only 42 px of content height. With `overflow: hidden`, a Dismiss button of at least 44 px would be clipped.
- **Fix:** The landscape box is 48 px. It ends at inset+48, which leaves 4 px above the title and result card at inset+52. No leaves overlap at any of the three viewports.
- **Commit:** cd102b0af

**2. [Rule 2 - Gate coverage] Chrome gate test covers the caption**
- **Found during:** Task 2
- **Issue:** `puzzle-chrome.test.ts` lists the puzzle components for the no-innerHTML and no-wall-clock gate (T-40-19), but the new component was not in that list.
- **Fix:** Added `PuzzleCaption.tsx` to `PUZZLE_COMPONENTS`. Also added tests that the leaf carries `data-hud-obstacle`, that it is mounted in both the HUD and the status row, and that it has a fixed height in both canvas layouts. `.puzzle-caption` joined the fixed width/height check.
- **Files modified:** web/tests/puzzle-chrome.test.ts
- **Commit:** cd102b0af

**Process note:** Task 1's tests were written first and confirmed failing. They were committed together with the implementation, as in earlier Phase 40 plans, so that every commit passes the typecheck.

## Known Stubs

None. Only first-pour has a caption entry. Later level plans must add theirs, and the `Record<PuzzleId, string>` type enforces this.

## Self-Check: PASSED

- FOUND: web/src/player/first-hint-preference.ts, web/src/puzzle/caption.ts, web/src/components/PuzzleCaption.tsx, web/e2e/puzzle-hint.spec.ts, both Vitest files and all six screenshots
- FOUND: commits 4069947f6, cd102b0af and 782683bb5

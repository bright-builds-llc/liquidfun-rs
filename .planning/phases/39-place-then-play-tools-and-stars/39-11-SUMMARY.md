---
phase: 39-place-then-play-tools-and-stars
plan: "11"
subsystem: web-puzzle-canvas-editor
tags: [typescript, puzzle, canvas, overlay, pointer-events, touch, debug-attributes, D-17, D-18, D-20, D-22, D-27, D-28, UI-D13, UI-D14]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T01:39:41Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: editor.ts snapWorldToGrid/canonicalPlacements and hit-test.ts pickTool (39-02); reducer canPlay/isDeciding and the editor inspection fields (39-07); applyPuzzleEdit, inspectNow, PuzzleRuntime and the controller's maybeHeldHash (39-09)
provides:
  - web/src/render/puzzle-tools.ts toolOverlay, drawPuzzleTools, ToolStyle, OverlayTool
  - web/src/player/puzzle-editor-input.ts attachPuzzleEditorInput, dragTargetCells, TOUCH_LIFT_CSS_PX, MOUSE_DRAG_THRESHOLD_CSS_PX, TOUCH_DRAG_THRESHOLD_CSS_PX
  - web/src/puzzle/debug-attributes.ts puzzleDebugAttributes
  - PuzzleRuntime.maybeEditorInput (bound by assignPuzzleCanvas, detached by leavePuzzle)
  - main data-puzzle-placements/selected/armed/speed/play-enabled/deciding/outcome-step/decided-step/peak-held/held-hash
affects:
  - 39-12 (CSS for puzzle-canvas--armed/over-tool/dragging and touch-action; tray arming feeds the armed tap path)
  - 39-13 (e2e reads the data-puzzle-* attributes and taps through the editor pointer)
tech-stack:
  added: []
  patterns:
    - tool overlay is a pure styling step (toolOverlay) plus a thin painter that returns early for no tools and otherwise wraps every pass in save/try/finally restore
    - the editor pointer is a closure-held single-press state machine with a building-only guard; it routes all edits through applyPuzzleEdit and never touches scene pointer state
key-files:
  created:
    - web/src/render/puzzle-tools.ts
    - web/src/player/puzzle-editor-input.ts
    - web/src/puzzle/debug-attributes.ts
    - web/tests/puzzle-tools-render.test.ts
    - web/tests/puzzle-editor-input.test.ts
    - web/tests/puzzle-debug-attributes.test.ts
  modified:
    - web/src/player/scene-surface.ts
    - web/src/player/puzzle-runtime.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/components/PlaygroundStage.tsx
    - web/tests/puzzle-shell-wiring.test.ts
    - web/tests/puzzle-lifecycle.test.ts
key-decisions:
  - "drawPuzzleTools returns before save() when there are no tools, so loading and empty-editor frames leave the context untouched"
  - "Armed tap-to-place uses the pointerdown world point, not the release point, and never applies the touch lift (Pitfall 9)"
  - "Pressing a tool selects it on pointerdown; deselecting on empty canvas happens only on a tap release that never passed the drag threshold"
  - "A press counts as a drag only after moving strictly past the threshold (> 4 CSS px mouse, > 8 CSS px touch); only tool drags settle with inspectNow"
  - "One pointer edits at a time: a second pointerdown during an active press is ignored"
  - "The editor pointer does not call preventDefault on pointerdown, so a click can still focus the canvas for Plan 12's keyboard editing"
  - "leavePuzzle detaches and clears maybeEditorInput so a scene route keeps no puzzle listeners"
requirements-completed: []
duration: 7min
completed: 2026-10-13
---

# Phase 39 Plan 11: Canvas Tool Overlay, Editor Pointer and Debug Attributes Summary

**Puzzle tools are now drawn on the canvas between the particles and the glass cup. They follow the UI-SPEC styles: normal, selected halo, red dashed invalid, and locked. In building, a separate pointer handle supports tap-to-place, select-on-press and threshold drags. Drags keep the grab offset, snap to the grid live and lift touch drags 48 CSS px; empty taps deselect. `main` exposes deterministic `data-puzzle-*` attributes for e2e.**

## Performance

- **Duration:** about 7 min
- **Started:** 2026-10-13T01:32:04Z
- **Completed:** 2026-10-13T01:38:43Z
- **Tasks:** 3 of 3
- **Files:** 6 created, 6 modified

## Accomplishments

### Tool overlay (Task 1)

- `toolOverlay(state)` reads `editor.maybeInspection`, which is the latest inspection, live or commit, so the tint follows a drag right away (UI-D14).
  - **Building:** styles are `normal`, `selected`, `invalid` or `selectedInvalid`.
  - **Running and ended:** every tool is `locked`.
  - **Loading:** nothing is drawn.
  - Tools without an outline are skipped. If the inspection's outline or status count does not match the tool count (a stale inspection), nothing is drawn.
- `drawPuzzleTools` projects each quad corner with `projectPoint`, sets `lineJoin = "round"`, and draws four passes per tool:
  1. halo (selected only): `rgba(244, 247, 250, 0.40)`, width 8;
  1. keyline: `rgba(7, 16, 24, 0.80)`, width 4;
  1. fill: `rgba(51, 65, 85, 0.92)`, or `rgba(90, 36, 48, 0.92)` when invalid;
  1. stroke, width 2: `#CBD5E1`, `#F4F7FA` when selected, or `#F07178` with `setLineDash([6, 4])` and then `setLineDash([])` when invalid.
- Everything is drawn inside `save()` / `try` / `finally { restore() }`.
- `drawSceneFrame` calls `drawPuzzleTools(context, camera, toolOverlay(session.puzzle.current))` before `drawGlassCup`, behind the existing route check, so scene routes never read puzzle state.

### Editor pointer (Task 2)

- `attachPuzzleEditorInput(canvas, deps)` listens to `pointerdown`, `pointermove`, `pointerup`, `pointercancel` and `lostpointercapture`. It gets world points from `cssPointFromClient` plus `unprojectPoint` and drops non-finite results (T-39-31).
- **Armed press:** a tap places `{toolKind, x, y}` at the snapped press point, with no lift. If the press moves past the threshold, nothing is placed.
- **Press on a tool:** hit-testing uses `pickTool` with the latest outlines and a 22 CSS px slop. A hit commits `select` at once. Past the threshold:
  - the canvas gets `puzzle-canvas--dragging`;
  - `move` is dispatched live, and only when the snapped cell changes;
  - the target cell is `dragTargetCells(grabOffset, pointerWorld, lift)`;
  - release or cancel calls `settle`, which is `inspectNow`.
- **Tap on empty canvas:** commits `select {maybeId: undefined}`.
- **Mouse hover in building:** toggles `puzzle-canvas--over-tool`.
- Handlers do nothing outside building, and the binding never sets `canvas-interactive` or uses timers.
- `assignPuzzleCanvas` detaches any previous `session.puzzle.maybeEditorInput` and binds a new one. It still never calls `attachCanvasPointer` and still leaves `session.maybeCanvasPointer` undefined.

### Debug attributes (Task 3)

- `puzzleDebugAttributes(state, maybeHeldHash)` always returns these keys:
  - `data-puzzle-placements`: canonical `kind:x,y,angle` joined by `;`, or `""`;
  - `data-puzzle-selected`: the creation-order index, or `""`;
  - `data-puzzle-armed`;
  - `data-puzzle-speed`;
  - `data-puzzle-play-enabled`;
  - `data-puzzle-deciding`.
- When ended, it adds `data-puzzle-outcome-step`, `data-puzzle-decided-step` and `data-puzzle-peak-held`. It adds `data-puzzle-held-hash` whenever a hash is provided.
- `PlaygroundStage` spreads these attributes onto `main` only on puzzle routes. The Phase 38 attributes are unchanged.

## Task Commits

1. **Task 1 RED:** `90b549018` (test). Failing tests for the puzzle tool overlay.
1. **Task 1 GREEN:** `601a47224` (feat). Draws puzzle tools between the particles and the glass cup, and adds the D-22 order wiring test.
1. **Task 2 RED:** `218734158` (test). Failing tests for the building-only editor pointer and the `assignPuzzleCanvas` wiring.
1. **Task 2 GREEN:** `819b46b9a` (feat). Building-only canvas editor pointer.
1. **Task 3 RED:** `03d66b0e0` (test). Failing tests for the puzzle debug attributes.
1. **Task 3 GREEN:** `42dec9667` (feat). Deterministic puzzle debug attributes on `main`.

No refactor commits were needed.

## Verification

- **RED runs:** each new test file failed to load because its module did not exist yet. The new wiring test failed on the missing `maybeEditorInput`.
- **Task 1:** `bunx vitest run tests/puzzle-tools-render.test.ts tests/puzzle-shell-wiring.test.ts` passed 25 tests, and `bunx tsc --noEmit` exited 0.
- **Task 2:** `bunx vitest run tests/puzzle-editor-input.test.ts tests/puzzle-shell-wiring.test.ts tests/puzzle-lifecycle.test.ts tests/puzzle-editing.test.ts` passed 75 tests, and tsc exited 0.
- **Task 3:** `bunx vitest run tests/puzzle-debug-attributes.test.ts` passed 5 tests.
- **Full Vitest** (`bun run test:unit`): 94 files, 1261 tests passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `bun run build:app`: built.
- **Acceptance greps:**
  - `export function drawPuzzleTools` and `setLineDash([6, 4])` match in `puzzle-tools.ts`.
  - In `scene-surface.ts`, `drawPuzzleTools(` is on line 76, before `drawGlassCup(context` on line 78.
  - `export function attachPuzzleEditorInput` and `TOUCH_LIFT_CSS_PX = 48` match.
  - `canvas-interactive` and `setTimeout|setInterval|Date.now` print nothing for `puzzle-editor-input.ts`.
  - `data-puzzle-held-hash` matches in `debug-attributes.ts`, and `puzzleDebugAttributes` matches in `PlaygroundStage.tsx`.
  - `tests/puzzle-shell-wiring.test.ts` still asserts that `attachCanvasPointer` is not called. Its original assertions are unchanged.
- **Line counts:** `scene-surface.ts` 453 (≤ 470), `puzzle-editor-input.ts` 317, `puzzle-tools.ts` 159.
- **e2e:** `CI=true bunx playwright test e2e/puzzle.spec.ts --project=chromium` gave **15 passed, 1 failed**. This matches the result after 39-09.
  - The failing test is "wins with the recorded solution through the test seam". It expects `"Filled! 76% of 60%"` and receives `"Filled! 90% of 60%"`.
  - This is the pre-D-10 literal that Plan 13 owns. This plan does not cause it.
- **Real-browser smoke:** a temporary Chromium spec ran at 1280×800 with the test seam seeding `plank:8,64,23`. It was deleted after the run, and `git status` was clean. It checked:
  - the building attributes;
  - a click on the plank set `data-puzzle-selected="0"`;
  - a 60 px mouse drag set `puzzle-canvas--dragging` during the drag;
  - after release, `data-puzzle-placements="plank:24,64,23"` (16 cells at 76.13 px/m), with no snap back;
  - a click on empty canvas cleared the selection;
  - `data-pointer-accepted` stayed `"0"`, and `canvas-interactive` never appeared.
- **Screenshots:** they showed the plank with the dark keyline, slate fill and selected halo above the walls and below the glass.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Correctness] `leavePuzzle` detaches the editor pointer**
- **Found during:** Task 2
- **Issue:** The plan binds `maybeEditorInput` in `assignPuzzleCanvas` but never releases it when the route leaves the puzzle. The old canvas would keep listeners and a closure over the session.
- **Fix:** `leavePuzzle` now calls `maybeEditorInput?.detach()` and clears the field. A test in `tests/puzzle-lifecycle.test.ts` covers it.
- **Files:** `web/src/player/puzzle-lifecycle.ts` and `web/tests/puzzle-lifecycle.test.ts`. Neither file is in `files_modified`.
- **Commit:** `819b46b9a`

**2. [Rule 2 - Correctness] Single-pointer guard and hover reset**
- **Found during:** Task 2
- **Issue:** A second finger's `pointerdown` would replace the active press. The hover class could also stay after the phase left building.
- **Fix:** A new `pointerdown` is ignored while a press is active, and a `pointermove` outside building clears `puzzle-canvas--over-tool`. A test covers the second pointer.
- **Commit:** `819b46b9a`

**3. [Rule 3 - Blocking] Wiring-test fake canvas gained `addEventListener`/`removeEventListener`**
- **Found during:** Task 2
- **Issue:** `assignPuzzleCanvas` now attaches listeners, and the existing fake canvas had no listener methods.
- **Fix:** I added the two methods to the fixture. No existing assertion changed.
- **Commit:** `218734158`

**4. Doc comment wording**
- The acceptance grep requires no `canvas-interactive` text in `puzzle-editor-input.ts`. The module comment now says "the scene crosshair class" instead.

## Known Stubs

None. The debug attributes' empty strings are the specified "none" values. The cursor classes `puzzle-canvas--over-tool` and `puzzle-canvas--dragging` are set here, but their CSS and `touch-action: none` belong to Plan 12's `puzzle-editor.css`. Until Plan 12 lands, the browser may still scroll on a touch drag. Arming from the tray, and therefore armed taps in the UI, also arrives with Plan 12's `PuzzleToolTray`.

## Threat Flags

None. The planned mitigations are in place and tested:

- **T-39-31:** non-finite client or world points are dropped, and snapping yields integers.
- **T-39-32:** the editor uses a separate handle with a building-only guard. Wiring tests prove that `attachCanvasPointer` is never called and that `maybeCanvasPointer` stays undefined.
- **T-39-33:** accepted. The attributes hold only integers, booleans, the tool kind and a simulation hash.
- **T-39-34:** `save`/`try`/`finally restore` is in place, the dash is reset after invalid tools, and a test proves one `save` and one `restore` even when a draw call throws.

## Requirements

- No requirement is marked complete by this plan.
- **TOOL-02** (place, move, rotate and remove with mouse and touch) is partly covered here: place, select and move with ≥ 44 px hit areas. Rotate and remove need Plan 12's selection bar.
- **TOOL-04** (Play, Stop/Reset and Clear) needs Plan 12's transport. This plan adds only the locked tool drawing while running and ended.
- **TOOL-03** was already marked complete. This plan adds the red, dashed canvas tint.

## Self-Check: PASSED

All 6 created files exist and all 6 task commits (90b549018, 601a47224, 218734158, 819b46b9a, 03d66b0e0, 42dec9667) are in git history.

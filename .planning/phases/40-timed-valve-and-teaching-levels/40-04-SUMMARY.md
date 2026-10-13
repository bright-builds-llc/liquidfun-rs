---
phase: 40-timed-valve-and-teaching-levels
plan: "04"
subsystem: web-puzzle-editor
tags: [typescript, solid, puzzle, valve, hud, css, playwright, TOOL-05]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T05:18:54.380Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-02 PuzzlePlacement.openTick, defaultOpenTick, updateTool openTick equality, ValveIcon"
provides:
  - valve-format.ts (STEPS_PER_OPEN_TICK 30, maxOpenTick, openStepFor, valveSecondsText, valveCountdownText)
  - editor selectedValve and stepSelectedValveTime (clamped 1..=maxTick through updateTool)
  - SetValveTimeEvent in PuzzleEvent, BuildingEditEvent (PuzzleEditEvent | SetValveTimeEvent)
  - "[ / ] keys -> { kind: valveTime, direction }"
  - valveStepperModel and the "Valve opens at 2.5 s" announcement
  - overlay maybeLabel, the "open" ToolStyle and drawToolLabels
  - PuzzleController.stepValveTime and puzzle-editing stepValveTime
  - always-mounted valve stepper group in the selection bar
  - e2e overlappingLeaves helper and SCREENS_DIR_40
affects:
  - 40-07 (browser valve flow: select a valve, step its time, watch the countdown and open outline)
  - any later HUD leaf (new landscape bottom-centre bar box, raised portrait result card)
tech-stack:
  added: []
  patterns:
    - edit events that need frame status (stepLimit) live in PuzzleEvent and are handled by the reducer, not applyEditEvent
    - overlay labels drawn from the same OverlayTool list as the outlines
key-files:
  created:
    - web/src/puzzle/valve-format.ts
    - web/tests/puzzle-valve-format.test.ts
  modified:
    - web/src/puzzle/editor.ts
    - web/src/puzzle/reduce-puzzle.ts
    - web/src/puzzle/keyboard.ts
    - web/src/puzzle/hud-format.ts
    - web/src/puzzle/editor-announce.ts
    - web/src/puzzle/controller.ts
    - web/src/render/puzzle-tools.ts
    - web/src/player/puzzle-editing.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/player/scene-surface.ts
    - web/src/components/PuzzleSelectionBar.tsx
    - web/src/components/PuzzleIcons.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/styles/puzzle-editor.css
    - web/src/styles/puzzle.css
    - web/tests/reduce-puzzle-editor.test.ts
    - web/tests/puzzle-keyboard.test.ts
    - web/tests/puzzle-tools-render.test.ts
    - web/tests/puzzle-editor-announce.test.ts
    - web/tests/puzzle-chrome.test.ts
    - web/e2e/puzzle-helpers.ts
    - web/e2e/puzzle-editor-helpers.ts
    - web/e2e/puzzle-editor.spec.ts
key-decisions:
  - "setValveTime is a PuzzleEvent handled by reduceValveTime (it needs status.stepLimit); applyPuzzleEdit and editAnnouncement take BuildingEditEvent so it shares the commit, inspect and announce path"
  - "The open style also drops the dark keyline (ToolPaint.keyline false), so the gone gate reads as a faint dashed outline"
  - "The form-factor shell bar is 300 px, not 280: it keeps Phase 39's 8 px gaps (144 + 8 + 3x44 + 2x8)"
  - "Valve labels draw right after the tools, before the glass cup, per the plan"
metrics:
  duration: 10min
  completed: 2026-10-13
  tasks: 2
  files: 25
---

# Phase 40 Plan 04: Valve open-time stepper, keys and overlay Summary

While building, the player can now set a valve's open time. There is a −/+ stepper in the selection bar and the `[` / `]` keys, both moving the time by 0.5 s within 0.5 s to step_limit/30 ticks. The canvas labels each valve with its time ("2.0 s"). While running, the label counts down from the frame step. From the open step on, the valve is drawn as a faint dashed outline. Every value is derived from `stepIndex` and the placement. Each HUD layout keeps one fixed box for the selection bar, and no two leaves overlap at 1280×800, 390×844 or 844×390.

## Task 1: Pure valve time model (commit 466a637a1)

- `valve-format.ts`:
  - `maxOpenTick(720) = 24` and `openStepFor(4) = 120`.
  - `valveSecondsText(4) = "2.0 s"`.
  - `valveCountdownText` rounds up to 0.1 s, so 59 steps left reads "1.0 s".
- `editor.ts`:
  - `selectedValve` returns the selected tool when it is a valve.
  - `stepSelectedValveTime(editor, direction, maxTick)` clamps to `1..=maxTick` through `updateTool`. A real change bumps the revision; when nothing changes it returns the same editor.
- `reduce-puzzle.ts`:
  - `SetValveTimeEvent` is part of `PuzzleEvent`.
  - `reduceValveTime` acts only in building and returns the same state object when nothing changes.
  - `BuildingEditEvent` is exported for the shell.
- `keyboard.ts`: `[` maps to `valveTime -1` and `]` to `+1`. The modifier and form-control rules are unchanged.
- `hud-format.ts`: `valveStepperModel(state)` disables each direction at its clamp. With no valve selected, or outside building, every flag is false and the text is "–".
- `editor-announce.ts`: `setValveTime` announces "Valve opens at 2.5 s". If validity also changes, ". <change>" is appended.
- `puzzle-tools.ts`:
  - `ToolStyle` gains `"open"`: transparent fill, stroke `rgba(203, 213, 225, 0.35)`, dashed, no keyline.
  - `OverlayTool.maybeLabel` labels valves only.
  - `drawToolLabels` draws each label at the projected quad centroid in `600 12px system-ui`, with a 3 px keyline stroke and a `#F4F7FA` fill. It restores the canvas state.
- `puzzle-editing.ts`:
  - `applyPuzzleEdit` now accepts `BuildingEditEvent`.
  - `stepValveTime(host, direction)` returns true only in building with a valve selected.
  - `applyKeyCommand` handles `valveTime`. The exhaustive switch required this for the typecheck, so the change moved into Task 1.
- Tests (all Arrange/Act/Assert):
  - New `puzzle-valve-format.test.ts` covers the arithmetic, the formatting and the stepper model.
  - Reducer tests cover +1, both clamps, a plank, no selection, and loading/running/ended.
  - Key tests cover `[`, `]`, the modifiers and form controls.
  - Overlay tests cover the building label, countdown 1.5 s at step 30, open at step 120 or later (running and ended), the open paint, and the label draw.
  - An announcement test covers the new copy.

## Task 2: Stepper UI, wiring, labels and layout (commit 10061a412)

- `PuzzleSelectionBar.tsx`: `div.puzzle-valve-stepper[role=group][aria-label="Valve opening time"]` sits before the rotate buttons. It holds "Open the valve 0.5 s earlier" (`MinusIcon`), a `span.puzzle-valve-time[aria-hidden][data-enabled]` and "Open the valve 0.5 s later" (`PlusIcon`). It is always mounted and disabled through `aria-disabled`.
- `PuzzleController.stepValveTime` is implemented in `puzzle-lifecycle.ts`, and `PuzzlePanel` wires `onStepValveTime`. `KEYBOARD_HELP` gained the bracket sentence.
- `scene-surface.ts`: the overlay list is computed once, then `drawPuzzleTools` and `drawToolLabels` both use it.

### Final CSS boxes

| Layout | Selection bar | Other changes |
| --- | --- | --- |
| Landscape canvas stage | `bottom: inset; left: 50%; translateX(-50%); 280×44`, one joined row (− 44, readout 56, + 44, CCW, CW, Remove) | none |
| Portrait canvas stage | `right: inset; bottom: inset+52px; 148×96`, a grid of `repeat(3, 44px)` × `44px 44px` with an 8 px gap; the stepper spans the top row (readout stretches to 60 px) | `.puzzle-result` moves from `bottom: inset+104px` to `inset+156px`; the tray keeps its box |
| Form-factor shell | `300×44`, one row, 8 px gaps | none |

### Screenshots

- `target/phase40/screens/selection-bar-1280x800.png`
- `target/phase40/screens/selection-bar-390x844.png`
- `target/phase40/screens/selection-bar-844x390.png`

### Verification

- `bunx tsc --noEmit` is clean, and `bun run test:unit` passes 95 files and 1352 tests.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- After `just web-wasm` and `bun run build:app`:
  - `CI=true bun run test:puzzle` passes 32/32, including the 3 new stepper/no-overlap tests.
  - `e2e/hud-safe-frame.spec.ts`, unedited, passes 64/64.
- `hud-safe-frame.spec.ts`, `tests/{hash,portrait-bounds,readme-svg}.test.ts` and `crates/liquidfun` have no diff from the phase base.
- The wall-clock / innerHTML / trig grep over `web/src/puzzle/*.ts` and `web/src/render/puzzle-tools.ts` finds nothing.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Key command wiring moved into Task 1**
- **Found during:** Task 1
- **Issue:** Adding `valveTime` to `PuzzleKeyCommand` broke the exhaustive `never` switch in `applyKeyCommand`, so the typecheck failed.
- **Fix:** `stepValveTime` and the `valveTime` case went into `web/src/player/puzzle-editing.ts` in the Task 1 commit. Task 2 only exposes it on the controller.
- **Commit:** 466a637a1

**2. [Rule 1 - Layout] Form-factor shell bar is 300 px instead of 280 px**
- **Found during:** Task 2
- **Issue:** The shell keeps Phase 39's separate 8 px-gapped buttons. The stepper (144), plus 8, plus three 44 px buttons with two 8 px gaps comes to 300 px, which does not fit in 280.
- **Fix:** Set the shell box to 300×44. The shell has no HUD obstacles, so the safe frame is unaffected.
- **Commit:** 10061a412

**3. [Rule 1 - Visual] The open style skips the dark keyline**
- **Issue:** Without this, the 4 px dark keyline would still draw around the "faint" open outline.
- **Fix:** `ToolPaint.keyline` is false only for `open`.

**4. Chrome unit test updated for the portrait bar height**
- `puzzle-chrome.test.ts` required `height: 44px` for every bar box. It now pins 44 px per leaf, except the portrait selection bar, which is 96 px.

**Process note:** At one point a stray `prettier` run (the repo has no Prettier config) rewrapped files to 80 columns. Before the commit, every touched file was rebuilt from HEAD with only the real delta applied. The committed diffs contain no formatting churn.

## Known Stubs

None. No level allows a valve yet, so the stepper is always disabled in the shipped levels. This is intentional per the plan facts, and Plan 07 proves the browser valve flow.

## Self-Check: PASSED

- FOUND: web/src/puzzle/valve-format.ts, web/tests/puzzle-valve-format.test.ts and all three screenshots
- FOUND: commits 466a637a1 and 10061a412

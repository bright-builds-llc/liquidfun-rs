---
phase: 42-opt-in-sound
plan: "04"
subsystem: puzzle-sound-ui
tags: [typescript, solid, sound, puzzle, hud, css, vitest, playwright]
requires:
  - "42-03 PuzzleController soundEnabled/soundAvailable/soundFailed/setSoundEnabled"
  - "42-03 SOUND_CONTROL_SELECTOR (.puzzle-sound-toggle, .puzzle-sound-section input) gesture exclusion"
provides:
  - "web/src/components/PuzzleSoundToggle.tsx: PuzzleSoundToggle, PuzzleSoundSection, SOUND_LABEL, SOUND_EFFECTS_LABEL, SOUND_HINT, SOUND_UNAVAILABLE_LABEL, SOUND_UNAVAILABLE_LINE, SOUND_FAILED_NOTE, SOUND_HELP_ID"
  - "web/src/components/PuzzleIcons.tsx: SoundOnIcon, SoundOffIcon, SoundUnavailableIcon"
  - "web/src/styles/puzzle-sound.css: non-accent pressed toggle and section layout"
  - "PuzzleHudProps.soundToggle, rendered first in the 96x44 puzzle-options-slot"
affects:
  - "42-05 phase gate and puzzle-sound e2e find the button by role name Sound and the checkbox by Sound effects"
tech-stack:
  added: []
  patterns:
    - "One shared controller signal drives the HUD button and both mirror checkboxes"
    - "Availability fallback swaps the whole button through Show; on and off reuse one element so focus stays"
key-files:
  created:
    - web/src/components/PuzzleSoundToggle.tsx
    - web/src/styles/puzzle-sound.css
  modified:
    - web/src/components/PuzzleIcons.tsx
    - web/src/components/PuzzleHud.tsx
    - web/src/components/PuzzlePanel.tsx
    - web/src/styles/puzzle.css
    - web/tests/puzzle-chrome.test.ts
decisions:
  - "42-04: slot gap: 8px goes on the shared transport/options-slot flex rule; transport computed gap is unchanged (landscape 0 from a more specific rule, portrait already 8 from puzzle-editor.css), puzzle.css 451 -> 452 lines"
  - "42-04: the unavailable toggle is a separate disabled button behind Show; on/off share one button so Enter/Space keep focus"
  - "42-04: the section checkbox is checked only when available and enabled, so unavailable is always unchecked"
metrics:
  duration: 5min
  completed: 2026-10-13
  tasks: 2
  files: 7
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: "2026-10-13T19:45:34.586Z"
---

# Phase 42 Plan 04: Sound toggle, drawer section and HUD slot Summary

Puzzle routes now have visible sound controls that match the approved UI-SPEC. The canvas HUD has a 44x44 speaker button named "Sound" with `aria-pressed`. It sits first inside the `puzzle-options-slot` leaf, which widened to 96x44. The options drawer and the form-factor control row both have a "Sound" section with a "Sound effects" mirror checkbox and the silent-switch hint. After a runtime audio failure the section also shows a failure note. All of these controls read the controller signals added in Plan 03, and none carries `data-hud-obstacle`.

## Performance

- **Duration:** about 5 min
- **Started:** 2026-10-13T19:41:28Z
- **Completed:** 2026-10-13T19:45:34Z
- **Tasks:** 2 (2 commits)
- **Files:** 7 (2 created, 5 modified)

## What shipped

- `PuzzleSoundToggle` uses `class="hud-button puzzle-sound-toggle"`. When sound is available it sets `aria-pressed` to "true" or "false", `aria-label="Sound"`, `title` to the hint, `aria-describedby="puzzle-sound-help"` and `data-sound-state` to on or off, and shows the On or Off icon. When sound is unavailable it renders a `disabled` button with `aria-pressed="false"`, label and title "Sound unavailable in this browser", `data-sound-state="unavailable"` and the slashed icon.
- `PuzzleSoundSection` contains `h3#puzzle-sound-heading`, a `.player-check` "Sound effects" checkbox described by `#puzzle-sound-hint`, an optional `#puzzle-sound-note` and the hint or unavailable line. When sound is unavailable the checkbox is disabled and unchecked.
- `PuzzlePanel` mounts the toggle in the canvas-stage HUD. It puts the section in the drawer after Particles and at the end of the second form-factor `.control-row`. It also adds `<p id="puzzle-sound-help" class="visually-hidden">` after the keyboard help paragraph. The unknown-level fallback is unchanged.
- The doc comment in `PuzzleHud` now gives the focus order as: caption Dismiss, tray, selection bar, run controls, result actions, Sound, options.

## Existing literals changed (UI-SPEC "Decisions Affecting Existing Tests and Literals")

| File | Was | Now | Why |
| --- | --- | --- | --- |
| `web/src/styles/puzzle.css` shared `.puzzle-hud .puzzle-transport, .puzzle-hud .puzzle-options-slot` | `display: flex;` | adds `gap: 8px;` | 8 px between the sound button and the options trigger. The transport's computed gap does not change (see decisions). |
| `puzzle.css` landscape `.puzzle-options-slot` | `width: 44px;` | `width: 96px;` | 44 + 8 + 44 for two buttons (UI-D31). |
| `puzzle.css` portrait `.puzzle-options-slot` | `width: 44px;` | `width: 96px;` | Same. |
| `puzzle.css` landscape `.puzzle-caption` | `right: calc(var(--puzzle-inset-right) + 52px);` | `right: calc(var(--puzzle-inset-right) + 104px);` | Keeps the caption 8 px clear of the wider slot. |
| `puzzle.css` portrait `.puzzle-title` | `width: calc(100% - 120px);` | `width: calc(100% - var(--puzzle-inset-left) - var(--puzzle-inset-right) - 156px);` | Ends the title 8 px left of the slot at any safe-area inset. |
| `web/tests/puzzle-chrome.test.ts` `PUZZLE_COMPONENTS` | 13 entries | adds `components/PuzzleSoundToggle.tsx` | Brings the new component under the wall-clock, `innerHTML` and scene-chrome gates. |

No existing test assertion changed. The HUD-leaf list, the caption-height check and the sized-leaf checks are unchanged and still pass. Every edited block kept its `width` and `height`.

New chrome tests: the sound component has no obstacle marker, all six copy strings appear verbatim, and it references no `AudioContext`, `localStorage` or `audioSession`. `puzzle-sound.css` contains no `#4DA3FF` and no motion. Both slot blocks are 96x44. The caption `right` and portrait title `width` match the spec. The toggle renders before the options trigger.

## Verification

- `bun run typecheck` passes. `bun run test:unit` passes 113 files and 1718 tests (1713 after Task 1, 1708 before). `tests/puzzle-chrome.test.ts` passes 35 tests.
- `bun run build:app` succeeds. `CI=true bun run test:puzzle` passes 81 tests, including every `overlappingLeaves` / `overlappingObstacles` check. `CI=true bunx playwright test e2e/hud-safe-frame.spec.ts` passes 64 tests.
- `bun scripts/bright-builds-check.ts all` reports 0 findings after each task.
- From `web/`, `git diff --quiet $(cat ../target/phase42/phase-base.txt) -- e2e/hud-safe-frame.spec.ts e2e/shell.spec.ts src/components/CanvasStage.tsx src/styles/canvas-stage.css src/components/PlayerPanel.tsx` exits 0.
- Acceptance greps: `width: 96px;` appears twice in `puzzle.css`, which is 452 lines. `PuzzleSoundSection` appears three times in `PuzzlePanel.tsx`, which is 457 lines (+12). There are 3 sound icon exports, and `data-hud-obstacle` does not appear in `PuzzleSoundToggle.tsx`.
- A temporary geometry probe spec was run and then deleted. Plan 05 owns the real sound e2e. Measured boxes match the UI-SPEC fit table exactly:

| Viewport | Slot x | Sound x | Options x | Neighbour right | Clearance |
| --- | --- | --- | --- | --- | --- |
| 390x844 | 286-382 | 286-330 | 338-382 | title 278 | 8 |
| 360x780 | 256-352 | 256-300 | 308-352 | title 248 | 8 |
| 844x390 | 740-836 | 740-784 | 792-836 | caption 732 | 8 |
| 667x375 | 563-659 | 563-607 | 615-659 | caption 555 | 8 |
| 1280x800 | 1176-1272 | 1176-1220 | 1228-1272 | caption 1168 | 8 |

  At every viewport, the probe toggled with Enter and checked the following. Focus stayed on the toggle. The slot box and the full obstacle list were identical before and after. There were no leaf or world-frame overlaps. Exactly one button was named "Sound". In the drawer the checkbox mirrored the state and unchecking it turned the HUD button off. In form-factor there was no HUD button and one checkbox. With `AudioContext` and `webkitAudioContext` removed, the button was disabled and named "Sound unavailable in this browser", the checkbox was disabled, and the section showed the unavailable line.
- Screenshots are in `target/phase42/screens/`: `390x844-{off,on}.png`, `844x390-{off,on}.png`, `390x844-drawer.png` and `form-factor-options.png`, plus the other probe viewports.

## Task Commits

1. **Task 1:** `4f7f1b194` (feat) adds the speaker icons, `PuzzleSoundToggle`, `PuzzleSoundSection`, `puzzle-sound.css` and the chrome tests.
1. **Task 2:** `a9a97949b` (feat) mounts the controls in the HUD, drawer and form-factor row, widens the slot and moves the offsets.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The module doc comment tripped the obstacle gate**

- **Found during:** Task 1
- **Issue:** The first module comment said "Neither carries `data-hud-obstacle`". That text made the new "no `data-hud-obstacle`" chrome test fail, and the plan's `! grep` verify step failed too.
- **Fix:** Reworded the comment to "Neither is a HUD obstacle leaf; the `puzzle-options-slot` that holds the toggle is."
- **Files modified:** `web/src/components/PuzzleSoundToggle.tsx`
- **Commit:** `4f7f1b194`

### Interpretations

**2. `gap: 8px` on the shared flex rule**

- The plan offered two options: add the gap to the slot, or split the selector if needed while keeping the line count. Splitting would have added lines. I added the gap to the shared rule instead. The transport's computed gap does not change: landscape keeps `gap: 0` from a more specific `puzzle-editor.css` rule, and portrait already gets 8 px from `.puzzle-transport`. `puzzle.css` grew from 451 to 452 lines, which is within the at-most-452 criterion.

## Known Stubs

None.

## Threat Flags

None. T-42-12: every string is a TS constant rendered as text, with no `innerHTML` (covered by the chrome gate). T-42-13: the button is named "Sound" and the checkbox "Sound effects", and the probe confirmed exactly one "Sound" button. T-42-14: the slot is a fixed 96x44 leaf, and the overlap and safe-frame specs are green.

## Self-Check: PASSED

- `web/src/components/PuzzleSoundToggle.tsx` and `web/src/styles/puzzle-sound.css` exist.
- Commits `4f7f1b194` and `a9a97949b` are in `git log`.

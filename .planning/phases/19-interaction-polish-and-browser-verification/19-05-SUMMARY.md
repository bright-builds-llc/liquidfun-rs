---
phase: 19-interaction-polish-and-browser-verification
plan: "05"
subsystem: playground-ui
tags: [figcaption, focus-visible, solidjs, css, tdd]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: six ready scenes, labeled controls, and canvas pointer pipeline from 19-01 through 19-04
provides:
  - locked interactionHint catalog strings
  - figcaption plus canvas aria-describedby binding
  - select:focus-visible accent ring and 480px stack polish
affects: [19-06, WEB-07, playground-copy, keyboard-focus]

tech-stack:
  added: []
  patterns:
    - static catalog strings rendered as JSX text
    - shared 2px / 4px #39D3C7 focus ring on buttons, selects, and links
    - 480px full-width stack without a hamburger or Space shortcut

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/tests/scenes.test.ts
    - web/src/components/PlayerPanel.tsx
    - web/src/App.tsx
    - web/src/app.css

key-decisions:
  - "Render locked two-sentence interactionHint as figcaption text, never innerHTML or a third Space-to-pause sentence."
  - "Add select:focus-visible to the existing 2px / 4px #39D3C7 rule instead of a new focus system."
  - "Keep App.tsx under the 628-line cap by inlining the ready-scene helper when passing the hint."

patterns-established:
  - "Instruction copy lives on SceneRecord and binds only on the ready-scene PlayerPanel branch."
  - "Fallback views omit figcaption so leftover Dam Break hints cannot appear."

requirements-completed: [WEB-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:23:25Z

duration: 2min
completed: 2026-09-19
---

# Phase 19 Plan 05: Per-Scene Figcaption, Select Focus, and 480px Polish Summary

**Ready scenes now show the locked two-sentence interactionHint, selects share the accent focus ring, and 480px controls stack full width without a hamburger or Space-to-pause shortcut.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-19T02:21:36Z
- **Completed:** 2026-09-19T02:23:25Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Every catalog scene carries the exact UI-SPEC `interactionHint`, including the keyboard reminder and no Space-to-pause or leftover Canvas 2D caption.
- Ready-scene `PlayerPanel` binds that hint as `<figcaption id="scene-interaction-hint">` and `aria-describedby`; fallback views still have no figcaption.
- Native selects use the same 2px `#39D3C7` / 4px-offset `:focus-visible` outline as buttons and links.
- The 480px block keeps `min-width: 0`, full-width `.scene-control` groups, wrapping figcaption text, and 44px minimum targets.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add locked interactionHint strings to the catalog**
   - `bb8278f` (test): failing hint and labeled-control tests
   - `c30145c` (feat): `interactionHint` on `SceneRecord` and the six scenes
2. **Task 2: Bind figcaption, select focus, and 480px stacking**
   - `aaebf42` (feat): figcaption binding, `select:focus-visible`, 480px polish

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `web/src/catalog/scenes.ts` — `interactionHint` type plus six locked strings
- `web/tests/scenes.test.ts` — `UI_SPEC_HINTS` and hint/label assertions
- `web/src/components/PlayerPanel.tsx` — figcaption and `aria-describedby`; `CANVAS_CAPTION` removed
- `web/src/App.tsx` — ready-scene hint prop only; 626 lines
- `web/src/app.css` — `select:focus-visible` and 480px stack polish

## Decisions Made

- Keep instruction copy as a TypeScript catalog string rendered through JSX text so hint XSS cannot come from `innerHTML`.
- Add `select:focus-visible` to the existing accent rule rather than introducing a new focus or design-system stack.
- Inline the ready-scene helper when passing the hint so `App.tsx` stays under the 628-line file trigger.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-06 Chromium 375px, pointer, and Pages evidence. WEB-07 copy, focus, and 480px CSS are in source; visual 375px proof still belongs to Plan 19-06. Do not deploy Pages from this plan.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: 19-05-SUMMARY.md
- FOUND: web/src/catalog/scenes.ts
- FOUND: web/src/components/PlayerPanel.tsx
- FOUND: web/src/App.tsx
- FOUND: web/src/app.css
- FOUND: web/tests/scenes.test.ts
- FOUND: bb8278f
- FOUND: c30145c
- FOUND: aaebf42

---
phase: 33-stacked-drip-fidget
plan: "03"
subsystem: testing
tags: [playwright, chromium, smoke, shell]

requires:
  - phase: 33-stacked-drip-fidget
    provides: stacked-drip catalog scene and hash path from plans 33-01 and 33-02
provides:
  - Chromium smoke that opens, pauses, plays, and resets Stacked Drip
  - Shell counts for twenty-three demos and a drawer wrap onto Stacked Drip
affects: [33-04 return stroke]

tech-stack:
  added: []
  patterns:
    - "Watch-first scenes stay out of POINTER_CONTROL so the existing loop covers play, pause, and reset"

key-files:
  created: []
  modified:
    - web/e2e/shell.spec.ts

key-decisions:
  - "Left ALL_SCENE_TIMEOUT_MS at 380_000 because Chromium smoke finished in 38.8s."
  - "This summary records the smoke evidence and does not approve the work."

patterns-established:
  - "The drawer last link is /Stacked Drip\\b/ and Fountain stays /Static preview Fountain\\b/."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-28T00:49:30Z

duration: 3min
completed: 2026-09-28
---

# Phase 33 Plan 03: Stacked Drip Smoke Summary

**Chromium smoke opens Stacked Drip, pauses, plays, and resets it, and the existing catalog still opens**

## Performance

- **Duration:** 3 min
- **Started:** 2026-09-28T00:46:26Z
- **Completed:** 2026-09-28T00:49:13Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- The shell footer expects `All twenty-three demos`. The desktop sidebar expects 23 static previews and 23 hidden SVGs.
- The mobile drawer counts 24 focusables. Twenty-five Tab presses and twenty-five Shift+Tab presses wrap onto `/Stacked Drip\b/`.
- `just web-player-smoke` passed 44 Chromium tests in 38.8s against the production preview. Watch-first coverage includes Stacked Drip because it is absent from `POINTER_CONTROL`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Point the shell at twenty-three scenes** - `5960463` (feat)
2. **Task 2: Run Chromium player smoke** - no source commit. The suite passed with the shell counts from Task 1.

**Plan metadata:** recorded in the docs commit for this summary.

## Files Created/Modified

- `web/e2e/shell.spec.ts` - Twenty-three demo copy, sidebar counts, and drawer wrap onto Stacked Drip

## Decisions Made

- `ALL_SCENE_TIMEOUT_MS` stayed `380_000`. The suite finished inside that budget, so the timeout was not raised.
- `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stayed at 4. Particle counts were not cut.
- The Fountain locator stayed `/Static preview Fountain\b/`. Stacked Drip did not join `POINTER_CONTROL`.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. Passing smoke is evidence of the browser gate, and it is not a review acknowledgment.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 33-04. Smoke proves open, play, pause, and reset for Stacked Drip, plus the existing catalog still opening. It does not assert that a particle has returned above the top tray.
- Independent review is still required. This summary does not approve the implementation.

## Test commands

- `cd web && bun run typecheck`
- `just web-player-smoke` — 44 passed (38.8s). The script set `CI=true` and Playwright served `vite preview` on port 4173.

## Self-Check: PASSED

- FOUND: web/e2e/shell.spec.ts
- FOUND: 5960463

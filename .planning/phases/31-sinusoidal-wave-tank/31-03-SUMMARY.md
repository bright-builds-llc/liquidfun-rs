---
phase: 31-sinusoidal-wave-tank
plan: "03"
subsystem: testing
tags: [playwright, chromium, web-player-smoke, wave-tank]

requires:
  - phase: 31-sinusoidal-wave-tank
    provides: Ready watch-first catalog id wave-tank and hash path #/scene/wave-tank
provides:
  - Chromium smoke that opens, pauses, plays, and resets Wave Tank and still runs the earlier catalog scenes
  - Shell asserts for twenty-one demos, twenty-one static previews, and a drawer wrap that lands on Wave Tank
affects: [phase-31-verification]

tech-stack:
  added: []
  patterns:
    - "Drawer last-link locators use /Wave Tank/ so they do not also match Wave Machine"
    - "Watch-first scenes stay out of POINTER_CONTROL so the existing pause, play, and reset loop covers them"

key-files:
  created: []
  modified:
    - web/e2e/shell.spec.ts

key-decisions:
  - "Left ALL_SCENE_TIMEOUT_MS at 380_000 and left both step caps at 4"
  - "Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work."

patterns-established:
  - "Shell demo counts follow the catalog length, including drawer wrap onto Wave Tank"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-09-27T17-22-43
generated_at: 2026-09-27T18:36:00Z

duration: 3min
completed: 2026-09-27
---

# Phase 31 Plan 03: Chromium Smoke Summary

**Chromium smoke opens, pauses, plays, and resets Wave Tank with the other catalog scenes, while the player still advances at most four steps**

## Performance

- **Duration:** 3 min
- **Started:** 2026-09-27T18:32:23Z
- **Completed:** 2026-09-27T18:36:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- The shell expects `All twenty-one demos`, 21 static previews, 21 preview SVGs, and zero catalog cards.
- Drawer focus uses 22 focusables. Twenty-three Tab presses wrap forward, and twenty-three Shift+Tab presses land on `/Wave Tank/`.
- `wave-tank` stays out of `POINTER_CONTROL`, so the existing watch-first loop opens `#/scene/wave-tank`, pauses, plays, and resets it.
- `just web-player-smoke` exited 0 on the production preview: Playwright **44 passed** in 36.1s. `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stay 4. `ALL_SCENE_TIMEOUT_MS` stayed `380_000`.

## Task Commits

Each task that changed files was committed atomically:

1. **Task 1: Point the shell at twenty-one scenes** - `255bbd7` (test)
2. **Task 2: Run Chromium player smoke** - no source commit. The first `just web-player-smoke` run exited 0, so no locator, timeout, or scene fix was required.

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/e2e/shell.spec.ts` - Twenty-one-demo footer, twenty-one sidebar previews, and drawer wrap onto Wave Tank. The original Fountain link stays `Static preview Fountain`.

## Decisions Made

- Left the suite timeout and both step caps unchanged. The Chromium run finished in 36.1s, so `ALL_SCENE_TIMEOUT_MS` stayed `380_000` and `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stayed 4.
- Left `wave-tank` out of `POINTER_CONTROL`. Gravity on this scene is not a pointer gesture, and the watch-first loop already opens `#/scene/wave-tank`.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. A passing smoke command is evidence of the Chromium browser gate, and it is not a review acknowledgment.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plans 31-01 through 31-03 are executed. Chromium proof is recorded. Phase verification stays with the orchestrator. Independent AI review remains eligible, and this implementing agent did not approve the work. `just readme-svg` was not run, and no GitHub Pages site was redeployed. No Firefox or Safari project was added.

---

*Phase: 31-sinusoidal-wave-tank*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: .planning/phases/31-sinusoidal-wave-tank/31-03-SUMMARY.md
- FOUND: web/e2e/shell.spec.ts
- FOUND: 255bbd7

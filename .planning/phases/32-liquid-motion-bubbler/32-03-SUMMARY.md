---
phase: 32-liquid-motion-bubbler
plan: "03"
subsystem: testing
tags: [playwright, chromium, web-player-smoke, liquid-bubbler]

requires:
  - phase: 32-liquid-motion-bubbler
    provides: Ready watch-first catalog id liquid-bubbler and hash path #/scene/liquid-bubbler
provides:
  - Chromium smoke that opens, pauses, plays, and resets Liquid Bubbler and still runs the earlier catalog scenes
  - Shell asserts for twenty-two demos, twenty-two static previews, and a drawer wrap that lands on Liquid Bubbler
affects: [phase-32-verification]

tech-stack:
  added: []
  patterns:
    - "Drawer last-link locators use /Liquid Bubbler/ so they do not also match Water Wheel or Wave Tank"
    - "Watch-first scenes stay out of POINTER_CONTROL so the existing pause, play, and reset loop covers them"

key-files:
  created: []
  modified:
    - web/e2e/shell.spec.ts

key-decisions:
  - "Left ALL_SCENE_TIMEOUT_MS at 380_000 and left both step caps at 4"
  - "Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work."

patterns-established:
  - "Shell demo counts follow the catalog length, including drawer wrap onto Liquid Bubbler"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T20:05:01Z

duration: 4 min
completed: 2026-09-27
---

# Phase 32 Plan 03: Chromium Smoke Summary

**Chromium smoke opens, pauses, plays, and resets Liquid Bubbler with the other catalog scenes, while the player still advances at most four steps**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-27T20:01:03Z
- **Completed:** 2026-09-27T20:05:01Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- The shell expects `All twenty-two demos`, 22 static previews, 22 preview SVGs, and zero catalog cards.
- Drawer focus uses 23 focusables. Twenty-four Tab presses wrap forward, and twenty-four Shift+Tab presses land on `/Liquid Bubbler/`.
- `liquid-bubbler` stays out of `POINTER_CONTROL`, so the existing watch-first loop opens `#/scene/liquid-bubbler`, pauses, plays, and resets it.
- `just web-player-smoke` exited 0 on the production preview: Playwright **44 passed** in 38.5s. `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stay 4. `ALL_SCENE_TIMEOUT_MS` stayed `380_000`.

## Task Commits

Each task that changed files was committed atomically:

1. **Task 1: Point the shell at twenty-two scenes** - `802f82b` (feat)
2. **Task 2: Run Chromium player smoke** - no source commit. The first `just web-player-smoke` run exited 0, so no locator, timeout, or scene fix was required.

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/e2e/shell.spec.ts` - Twenty-two-demo footer, twenty-two sidebar previews, and drawer wrap onto Liquid Bubbler. The original Fountain link stays `Static preview Fountain`.

## Decisions Made

- Left the suite timeout and both step caps unchanged. The Chromium run finished in 38.5s, so `ALL_SCENE_TIMEOUT_MS` stayed `380_000` and `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stayed 4.
- Left `liquid-bubbler` out of `POINTER_CONTROL`. Gravity on this scene is not a pointer gesture, and the watch-first loop already opens `#/scene/liquid-bubbler`.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. A passing smoke command is evidence of the Chromium browser gate, and it is not a review acknowledgment.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Chromium gate evidence is recorded. `just web-player-smoke` exited 0, and the step cap stayed at 4. Independent AI review remains eligible. This implementing agent did not approve the work. Ready for orchestrator verification. Phase completion stays with that verification.

## Known Stubs

None.

---
*Phase: 32-liquid-motion-bubbler*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: web/e2e/shell.spec.ts
- FOUND: .planning/phases/32-liquid-motion-bubbler/32-03-SUMMARY.md
- FOUND: 802f82b

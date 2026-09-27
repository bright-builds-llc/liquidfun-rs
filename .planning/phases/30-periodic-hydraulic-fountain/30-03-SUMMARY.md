---
phase: 30-periodic-hydraulic-fountain
plan: "03"
subsystem: testing
tags: [playwright, chromium, web-player-smoke, hydraulic-fountain]

requires:
  - phase: 30-periodic-hydraulic-fountain
    provides: Ready watch-first catalog id hydraulic-fountain and hash path #/scene/hydraulic-fountain
provides:
  - Chromium smoke that opens, pauses, plays, and resets Hydraulic Fountain and still runs the earlier catalog scenes
  - Shell asserts for twenty demos, twenty static previews, and a drawer wrap that lands on Hydraulic Fountain
affects: [phase-30-verification]

tech-stack:
  added: []
  patterns:
    - "Sidebar links that share a title word use the Static preview caption plus a word boundary"
    - "Watch-first scenes stay out of POINTER_CONTROL so the existing pause, play, and reset loop covers them"

key-files:
  created: []
  modified:
    - web/e2e/shell.spec.ts

key-decisions:
  - "Matched the original Fountain sidebar link as Static preview Fountain so Hydraulic Fountain does not share the click"
  - "Left ALL_SCENE_TIMEOUT_MS at 380000 and left both step caps at 4"
  - "Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work"

patterns-established:
  - "Shell demo counts follow the catalog length, including drawer wrap onto the last scene title"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-09-27T14-01-51
generated_at: 2026-09-27T15:37:44Z

duration: 5min
completed: 2026-09-27
---

# Phase 30 Plan 03: Chromium Smoke Summary

**Chromium smoke opens, pauses, plays, and resets Hydraulic Fountain with the other catalog scenes, while the player still advances at most four steps**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-27T15:32:48Z
- **Completed:** 2026-09-27T15:37:44Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- The shell expects `All twenty demos`, 20 static previews, 20 preview SVGs, and zero catalog cards.
- Drawer focus uses 21 focusables. Twenty-two Tab presses wrap forward, and twenty-two Shift+Tab presses land on Hydraulic Fountain.
- `hydraulic-fountain` stays out of `POINTER_CONTROL`, so the existing watch-first loop opens `#/scene/hydraulic-fountain`, pauses, plays, and resets it.
- `just web-player-smoke` exited 0 on the production preview: Playwright **44 passed** in 42.6s. `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stay 4. `ALL_SCENE_TIMEOUT_MS` stayed `380_000`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Point the shell at twenty scenes** - `1a5a3fb` (test)
2. **Task 2: Run Chromium player smoke** - `1e37b53` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/e2e/shell.spec.ts` - Twenty-demo footer, twenty sidebar previews, and drawer wrap onto Hydraulic Fountain. The original Fountain link is `Static preview Fountain`.

## Decisions Made

- Kept the original Fountain click. A `/Fountain/` role name also matched Hydraulic Fountain, so the sidebar locator now requires `Static preview Fountain`.
- Did not raise the suite timeout. The failing run was a locator collision, and the passing run finished in 42.6s.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. A passing smoke command is evidence of the Chromium browser gate, and it is not a review acknowledgment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Disambiguated the Fountain sidebar link from Hydraulic Fountain**
- **Found during:** Task 2 (Run Chromium player smoke)
- **Issue:** `getByRole("link", { name: /Fountain/ })` resolved to both Fountain and Hydraulic Fountain, so the desktop sidebar navigation test failed strict mode.
- **Fix:** Match `Static preview Fountain` with a word boundary. Drawer wrap still uses `/Hydraulic Fountain/`.
- **Files modified:** `web/e2e/shell.spec.ts`
- **Verification:** `just web-player-smoke` exited 0 with 44 Chromium tests passed.
- **Committed in:** `1e37b53` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The new title shares the word Fountain. The tighter locator keeps the existing Fountain navigation test on the original scene.

## Issues Encountered

The first `just web-player-smoke` run failed one shell test because of the Fountain name collision (43 passed, 1 failed). The same command then exited 0 (44 passed, 42.6s).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plans 30-01 through 30-03 are executed. Chromium proof is recorded. Phase verification stays with the orchestrator. Independent AI review remains eligible, and this implementing agent did not approve the work. `just readme-svg` was not run, and no GitHub Pages site was redeployed.

---
*Phase: 30-periodic-hydraulic-fountain*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: .planning/phases/30-periodic-hydraulic-fountain/30-03-SUMMARY.md
- FOUND: web/e2e/shell.spec.ts
- FOUND: 1a5a3fb
- FOUND: 1e37b53

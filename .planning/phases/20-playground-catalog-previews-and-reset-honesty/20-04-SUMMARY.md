---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "04"
subsystem: playground-ui
tags: [playwright, chromium, catalog-previews, demo-sidebar, demos-dialog]

requires:
  - phase: 20-playground-catalog-previews-and-reset-honesty
    provides: ScenePreview switch and DemoNavigation Static preview captions
provides:
  - Desktop .demo-sidebar six Static preview and aria-hidden SVG counts
  - Mobile Demos dialog preview visibility without catalog-card
affects: [20-05 Reset honesty Playwright, 20-06 phase verification]

tech-stack:
  added: []
  patterns:
    - Scope desktop preview counts to .demo-sidebar
    - Scope mobile preview asserts to dialog named Demos
    - Pause playing scenes before Tab-heavy Chromium checks

key-files:
  created: []
  modified:
    - web/e2e/shell.spec.ts
    - web/e2e/player.spec.ts

key-decisions:
  - "Scope desktop Static preview counts to .demo-sidebar so dual DemoNavigation cannot count 12."
  - "Keep mobile preview asserts on getByRole dialog named Demos, not .demo-sidebar."
  - "Pause playing scenes before Tab-heavy drawer and 375px checks so compact previews cannot starve keyboard smoke."

patterns-established:
  - "Global page.getByText(Static preview) is forbidden; sidebar and drawer mounts share DemoNavigation."
  - "D-11: DOM text plus svg visibility only; no toHaveScreenshot, firefox, or webkit."

requirements-completed: [WEB-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T18:44:27Z

duration: 20min
completed: 2026-09-20
---

# Phase 20 Plan 04: Chromium Preview Smoke Summary

**Chromium shell specs prove six desktop-sidebar Static preview captions and SVGs plus one Demos-dialog preview, with global `.catalog-card` count 0.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-09-20T18:24:19Z
- **Completed:** 2026-09-20T18:44:27Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Desktop 1280×800 asserts six exact `Static preview` captions and six `svg[aria-hidden='true']` inside `.demo-sidebar` only.
- Mobile 390×812 opens the Demos dialog, asserts one visible caption and SVG, keeps `.catalog-card` at 0, and checks `scrollWidth <= innerWidth`.
- Existing Tab-trap and 375px keyboard tests now pause the playing scene first so compact preview paint cannot starve `keyboard.press("Tab")`.
- `web/e2e/shell.spec.ts` is 228 lines. No `package.json` or `playwright.config.ts` change. No Firefox/WebKit project. No screenshot hashes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Assert six desktop sidebar static previews**
   - `8ae4370` (test): scoped sidebar caption and SVG counts
2. **Task 2: Assert one mobile drawer preview and no catalog cards**
   - `c508667` (test): named Demos dialog preview plus Tab-heavy pause

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `web/e2e/shell.spec.ts` — desktop six-preview test, mobile dialog preview test, paused Tab trap
- `web/e2e/player.spec.ts` — pause before the 375px tab-to-control check
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/deferred-items.md` — 20-03 reset step-ceiling and demo-media 240-frame notes

## Decisions Made

- Scope desktop counts to `.demo-sidebar` because PlaygroundShell mounts DemoNavigation twice (Pitfall 5).
- Keep the new mobile test on `getByRole("dialog", { name: "Demos" })` and leave the Tab-trap locator unnamed so title scroll cannot hide the dialog name.
- Pause Playing before Tab-heavy checks; do not add browsers or PNG oracles (D-11).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pause before Tab-heavy Chromium checks**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** After compact previews, `keyboard.press("Tab")` in the open Demos drawer hung the 30s test timeout while Dam Break kept playing.
- **Fix:** Wait for ready chrome, click Pause scene, then run the existing trap loop. Pause the 375px player tab-to-control test the same way.
- **Files modified:** `web/e2e/shell.spec.ts`, `web/e2e/player.spec.ts`
- **Verification:** Isolated `traps repeated forward and reverse Tab navigation` and `tabs to a scene control` passed; both new preview tests passed in `just web-player-smoke`.
- **Committed in:** `c508667` (Task 2)

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Needed for existing keyboard smoke after 20-02 previews. No new browser matrix.

## Deferred Issues

`just web-player-smoke` exited 1 with 27 passed / 3 failed. The two new WEB-01 tests passed. Failures are out of this plan's files:

- `player.spec.ts` Reset step-ceiling (`Expected < 8, Received 29`) after 20-03 keyed remount. Belongs to 20-05 Reset honesty.
- `demo-media-clock.spec.ts` 240-frame capture 30s timeout. Pre-existing media clock, not catalog previews.

Logged in `deferred-items.md`. Do not treat those failures as 20-04 preview regressions.

## Authentication Gates

None.

## Issues Encountered

Full smoke still fails on the 20-03 Reset step series and the 240-frame media clock. Preview locators and `.catalog-card` count 0 are green.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 20-05 Reset label honesty. Desktop and drawer preview oracles exist. 20-05 must diagnose why Reset leaves `data-step-index` at 29 instead of below `RESET_STEP_CEILING` 8.

## Self-Check: PASSED

---
*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Completed: 2026-09-20*

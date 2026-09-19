---
phase: 19-interaction-polish-and-browser-verification
plan: "06"
subsystem: browser-smoke
tags: [playwright, chromium, pointer-events, webtest-01]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: canvas pointer pipeline, teardown cancel, figcaption, and 480px polish from 19-01 through 19-05
provides:
  - Chromium player smoke for six-scene pointer plus labeled control
  - pointercancel, resize-then-drag, hidden-tab-after-gesture, and 375px keyboard/scroll cases
  - README/TESTING local WEBTEST-01 gate wording
affects: [19-07, WEBTEST-01, playground-docs]

tech-stack:
  added: []
  patterns:
    - DOM attributes data-step-index, data-last-pointer-kind, and data-pointer-accepted are the player-smoke oracle
    - one Chromium Playwright project against /liquidfun-rs/; scroll canvas into view before gestures

key-files:
  created:
    - web/e2e/player-helpers.ts
  modified:
    - web/e2e/player.spec.ts
    - README.md
    - TESTING.md

key-decisions:
  - "Keep a single chromium Playwright project and assert DOM attributes, not PNG hashes."
  - "Extract player-helpers.ts so player.spec.ts stays under the 400-line split trigger."
  - "Scroll the canvas into view before pointer gestures so the catalog cannot intercept the hit."

patterns-established:
  - "just web-player-smoke is the local WEBTEST-01 gate; Pages CI still has no Playwright job."
  - "Catalog sits above the player, so canvas gestures must scrollIntoViewIfNeeded first."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:28:15Z

duration: 4min
completed: 2026-09-19
---

# Phase 19 Plan 06: Chromium Pointer, 375px, and Cleanup Smoke Summary

**`just web-player-smoke` now proves six-scene pointer plus labeled-control input, pointercancel, resize-then-drag, hidden-tab recovery, and a 375px keyboard/scroll pass against production-base `/liquidfun-rs/`.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-19T02:24:22Z
- **Completed:** 2026-09-19T02:28:15Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Every catalog scene receives one mapped canvas gesture and one labeled control while `data-step-index` advances and playback stays `Playing`.
- Dam Break `pointercancel` sets `data-last-pointer-kind` to `cancel`; a 1280×720 to 375×812 resize-then-drag increments `data-pointer-accepted` without asserting a rebuilt world.
- Hidden-tab recovery still caps the next step delta at 4 after a Dam Break drag.
- A 375×812 pass Tabs to the first select, asserts the locked Dam Break figcaption, and proves the catalog page can scroll.
- README and TESTING name `just web-player-smoke` as the local WEBTEST-01 gate and do not claim Firefox, Safari, complete parity, or Phase 17 SHA `50a1556` as Phase 19 Pages proof.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend player.spec.ts with pointer, cancel, resize, and 375px**
   - `4fb01ac` (test): helpers plus new Chromium cases
2. **Task 2: Run just web-player-smoke and document the local gate**
   - `08a0fba` (docs): scroll-into-view fix, README/TESTING gate wording

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

## Files Created/Modified

- `web/e2e/player-helpers.ts` — shared open/reset, hidden-tab, and canvas gesture helpers
- `web/e2e/player.spec.ts` — six-scene pointer loop, cancel, resize, hidden-tab-after-gesture, 375px
- `README.md` — honest Chromium WEBTEST-01 local-gate sentence
- `TESTING.md` — same gate; Pages workflow still has no Playwright job

## Decisions Made

- Keep Playwright at one `chromium` / Desktop Chrome project and treat DOM attributes as the oracle.
- Extract helpers once the spec would exceed about 400 lines.
- Scroll the canvas into view before mouse/pointer samples because the six-card catalog pushes the player below the fold.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Canvas gestures hit empty viewport above the player**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** Catalog cards sit above the canvas. Playwright `boundingBox` y was below the viewport, so `data-pointer-accepted` stayed 0 and `pointercancel` never set `data-last-pointer-kind`.
- **Fix:** `scrollIntoViewIfNeeded` before measuring the canvas box; cancel uses the same press helper.
- **Files modified:** `web/e2e/player-helpers.ts`, `web/e2e/player.spec.ts`
- **Verification:** `just web-player-smoke` exits 0; 11/11 Chromium tests passed
- **Committed in:** `08a0fba`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Required for the smoke gate. No browser-matrix or Pages-CI scope creep.

## Issues Encountered

The first `just web-player-smoke` failed four pointer cases because the canvas was off-screen. Scrolling the canvas into view fixed all four without product-code changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-07 live Pages evidence. Local Chromium WEBTEST-01 coverage is in source and `just web-player-smoke` passed. Do not treat this plan as the hosted revision proof, and do not mark WEBTEST-01 complete until the deployed site checks land.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: 19-06-SUMMARY.md
- FOUND: web/e2e/player.spec.ts
- FOUND: web/e2e/player-helpers.ts
- FOUND: README.md
- FOUND: TESTING.md
- FOUND: 4fb01ac
- FOUND: 08a0fba


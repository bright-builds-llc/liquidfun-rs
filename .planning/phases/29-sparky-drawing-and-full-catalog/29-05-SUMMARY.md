---
phase: 29-sparky-drawing-and-full-catalog
plan: "05"
subsystem: testing
tags: [playwright, chromium, web-player-smoke, drawing-particles, sparky, data-particle-count]

requires:
  - phase: 29-sparky-drawing-and-full-catalog
    provides: Drawing paint and Sparky powder scenes in the nineteen-scene catalog
provides:
  - Chromium smoke for nineteen scenes, one Drawing drag, and a watch-first Sparky loop
  - data-particle-count on the playground main
affects: [playground-smoke, catalog-shell]

tech-stack:
  added: []
  patterns:
    - "Pointer smoke is keyed only by POINTER_CONTROL; gravity-only scenes stay watch-first"
    - "Sidebar reveal retries use timers so a synthetic capture clock sees one animation callback"

key-files:
  created: []
  modified:
    - web/src/components/PlaygroundStage.tsx
    - web/e2e/player.spec.ts
    - web/e2e/player-helpers.ts
    - web/e2e/shell.spec.ts
    - web/src/components/sidebar-scroll.ts
    - web/src/components/ui/sidebar.tsx
    - web/src/styles/player-chrome.css

key-decisions:
  - "Interactive smoke is the POINTER_CONTROL map, including drawing-particles drag on Material. Sparky and gravity-only scenes stay watch-first."
  - "Sidebar current-link retries use short timers instead of requestAnimationFrame so demo capture still expects one callback."
  - "The mobile drawer close control is a Dismiss button. Overlay and hidden-content asserts follow the live shell."

patterns-established:
  - "Pattern: publish data-particle-count from the current frame, using 0 when no frame exists."
  - "Pattern: do not treat a Gravity slider as a pointer gesture."

requirements-completed: [PLAY-01, FX-01, FX-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T07:30:56Z

duration: 22min
completed: 2026-09-27
---

# Phase 29 Plan 05: Nineteen-scene Chromium smoke Summary

**Chromium `just web-player-smoke` covers nineteen scenes, one Drawing drag that leaves particles, and a watch-first Sparky loop**

## Performance

- **Duration:** 22 min
- **Started:** 2026-09-27T07:08:50Z
- **Completed:** 2026-09-27T07:30:56Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- The playground main publishes `data-particle-count` from the current frame, or `0` when there is no frame.
- Pointer smoke is exactly the `POINTER_CONTROL` map. Drawing Particles is a Material drag. Sparky, Particles, Liquid Tumbler, and the other gravity-only scenes stay on play, pause, and reset.
- The shell expects nineteen demos, nineteen static previews, zero `.catalog-card` elements, and a drawer wrap whose reverse-tab last link is Sparky.
- `just web-player-smoke` exited 0: Playwright **44 passed** in 40.0s on the production preview. `MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stay 4.

## Task Commits

Each task was committed atomically:

1. **Task 1: Retarget smoke and publish particle count** - `fef71f5` (test)
2. **Task 2: Run Chromium player smoke** - `03d3887` (fix)

## Files Created/Modified

- `web/src/components/PlaygroundStage.tsx` - `data-particle-count` on `<main>`
- `web/e2e/player.spec.ts` - pointer map versus watch-first split, including the Drawing particle-count drag
- `web/e2e/player-helpers.ts` - nineteen-scene timeout and Drawing/Sparky hash paths
- `web/e2e/shell.spec.ts` - nineteen-demo copy, preview counts, overlay, and drawer wrap
- `web/src/components/sidebar-scroll.ts` - current-link retries stay off `requestAnimationFrame`
- `web/src/components/ui/sidebar.tsx` - mobile drawer Dismiss control
- `web/src/styles/player-chrome.css` - Dismiss button sizing and colors

## Decisions Made

- Interactive ids are keys of `POINTER_CONTROL`. Watch-first ids are every other catalog id. The watch-first test asserts the pointer map is empty for that id and does not assert `controls.length === 0`.
- A Drawing drag reads `data-particle-count` as `0` before the gesture and greater than `0` after the pointer is accepted.
- Gravity stays a construction slider, not a pointer gesture.
- This plan does not record a review acknowledgment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Map Material so the labeled select can change**
- **Found during:** Task 1 (Retarget smoke and publish particle count)
- **Issue:** `activateLabeledControl` only changes selects listed in `SELECT_NEXT_VALUE`. Material would have fallen through to a missing button click.
- **Fix:** Added `Material: "elastic"` beside the other labeled selects.
- **Files modified:** `web/e2e/player-helpers.ts`
- **Verification:** `bun run typecheck` in `web`, then the interactive smoke loop in `just web-player-smoke`
- **Committed in:** `fef71f5`

**2. [Rule 1 - Bug] Sidebar reveal was a second animation callback**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** Current-link reveal queued `requestAnimationFrame` for twelve frames. Under the synthetic capture clock that callback never drained, so pending animation callbacks stayed at 2.
- **Fix:** Retry the reveal with short timers. The scene loop remains the only animation callback.
- **Files modified:** `web/src/components/sidebar-scroll.ts`
- **Verification:** `just web-player-smoke` demo-media clock tests passed
- **Committed in:** `03d3887`

**3. [Rule 1 - Bug] Drawer overlay and hidden-content asserts targeted removed nodes**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** The overlay class `.demo-drawer-overlay` is gone, and outside content is marked with `aria-hidden` on `.app-shell`.
- **Fix:** Click `[data-slot='drawer-overlay']` and assert `aria-hidden` on `.app-shell`.
- **Files modified:** `web/e2e/shell.spec.ts`
- **Verification:** Overlay dismiss and background-scroll tests passed in `just web-player-smoke`
- **Committed in:** `03d3887`

**4. [Rule 2 - Missing Critical] The mobile drawer had no Dismiss control**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** The open drawer exposed nineteen links and no close button, so reverse-tab could not start on Dismiss or land on Sparky.
- **Fix:** Added a Dismiss button as the first mobile-drawer control and waited for focus to enter the dialog before the wrap loop.
- **Files modified:** `web/src/components/ui/sidebar.tsx`, `web/src/styles/player-chrome.css`, `web/e2e/shell.spec.ts`
- **Verification:** Drawer tab-wrap test passed in `just web-player-smoke`
- **Committed in:** `03d3887`

**5. [Rule 1 - Bug] Exact Particles label no longer matches the grouped render select**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** `getByLabel("Particles", { exact: true })` matched nothing after render modes were grouped. The combobox accessible name is still Particles.
- **Fix:** The render-mode smoke uses `getByRole("combobox", { name: "Particles", exact: true })`.
- **Files modified:** `web/e2e/player.spec.ts`
- **Verification:** Render-mode persistence test passed in `just web-player-smoke`
- **Committed in:** `03d3887`

---

**Total deviations:** 5 auto-fixed (3 bugs, 2 missing critical)
**Impact on plan:** The fixes keep the existing Chromium gate green for nineteen scenes. Step caps stay 4. No second browser, Linux qualification, or Pages deploy.

## Issues Encountered

None. The first smoke run failed on the items above. The same `just web-player-smoke` command then exited 0.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 29 plans are complete. Chromium proof covers the catalog, including Drawing paint and watch-first Sparky. This summary is not an independent review acknowledgment, and it does not claim sealed C++ parity.

## Self-Check: PASSED

- FOUND: web/src/components/PlaygroundStage.tsx
- FOUND: web/e2e/player.spec.ts
- FOUND: web/e2e/player-helpers.ts
- FOUND: web/e2e/shell.spec.ts
- FOUND: commit fef71f5
- FOUND: commit 03d3887
- `just web-player-smoke` exited 0 (44 passed, 40.0s)

---
*Phase: 29-sparky-drawing-and-full-catalog*
*Completed: 2026-09-27*

---
phase: 30-periodic-hydraulic-fountain
plan: "02"
subsystem: playground-catalog
tags: [solidjs, catalog, portrait-frame, hydraulic-fountain]

requires:
  - phase: 30-periodic-hydraulic-fountain
    provides: SceneId::HydraulicFountain and the finished wall and piston constants
provides:
  - A ready watch-first catalog record for hydraulic-fountain immediately after sparky
  - A static piston, throat, and two-chamber preview, plus a phone frame that contains the finished walls
affects: [30-03-browser-proof]

tech-stack:
  added: []
  patterns:
    - "Watch-first original scenes use withGravitySlider([]) and a single LiquidFun showcase credit"
    - "Portrait frames copy the finished scene wall and piston endpoints and share one rectangle with viewBounds"

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/previews.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/src/player/runtime.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/e2e/player-helpers.ts

key-decisions:
  - "Copied the Plan 01 wall and piston endpoints unchanged: floor (±1.18, 0), sides x=±1.14 y=0..1.39, divider x=0 y=0.12..1.39, piston faces x=-0.86 and x=-0.51 at y=0.05 and y=0.95"
  - "Raised the shared view rectangle maxY from 1.55 to 1.6 so the phone frame clears the 0.28 height minimum after the 16px camera inset"

patterns-established:
  - "Hydraulic Fountain is an original playground scene: showcase credit only, no pinned LiquidFun test"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-09-27T14-01-51
generated_at: 2026-09-27T15:30:10Z

duration: 7min
completed: 2026-09-27
---

# Phase 30 Plan 02: Hydraulic Fountain Catalog Summary

**Hydraulic Fountain is a ready watch-first catalog scene after Sparky, with a static piston preview and a phone frame around the finished chamber walls**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-27T15:22:32Z
- **Completed:** 2026-09-27T15:29:57Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- The catalog lists twenty ready scenes. `hydraulic-fountain` follows `sparky`, and the existing fountain record is unchanged.
- The record is watch-first aside from the shared gravity slider. Credits name `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` and the LiquidFun showcase only.
- The still preview shows a piston, a throat, and two chambers, with water marks only in the left chamber. `DemoNavigation` already captions it `Static preview`.
- The portrait frame and the landscape view bounds are the same rectangle, and the portrait test includes the finished floor, side walls, divider, and piston-face endpoints.

## Task Commits

Each task was committed atomically:

1. **Task 1: Append the watch-first catalog record and credit** - `3ef6893` (feat)
2. **Task 2: Add the preview, portrait frame, and README plan** - `0227f2e` (feat)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/src/catalog/scenes.ts` - Appends `hydraulic-fountain` after `sparky`
- `web/src/catalog/scene-records.ts` - Ready watch-first record, showcase credit, and shared view rectangle
- `web/src/player/runtime.ts` - Page summary now says all twenty demos
- `web/src/catalog/previews.tsx` - Still drawing of a piston, a throat, and two chambers
- `web/src/catalog/portrait-bounds.ts` - Phone rectangle `minX -1.3`, `minY -0.15`, `maxX 1.3`, `maxY 1.6`
- `web/scripts/readme-svg/plans.ts` - README plan after sparky with `cues: []`
- `web/scripts/demo-media/model.ts` - Center-click capture plan
- `web/e2e/player-helpers.ts` - Hash path for the new id
- `web/tests/scenes.test.ts` - Twenty-scene descriptions, hints, and length asserts
- `web/tests/scene-catalog-controls.test.ts` - Gravity-only controls
- `web/tests/scene-catalog-credits.test.ts` - Showcase-only inspiration
- `web/tests/portrait-bounds.test.ts` - Height fraction 0.28 and finished wall and piston endpoints
- `web/tests/demo-media-model.test.ts` - Capture list length 20, last id `hydraulic-fountain`

## Decisions Made

- Kept Plan 01's starting geometry. The crossing test did not retune the walls or the piston, so the portrait points are those constants.
- The shared view rectangle uses `maxY: 1.6` instead of the planned `1.55`. At `1.55` the fitted phone height was 0.277, under the required 0.28, because the camera keeps a 16px inset. `1.6` clears that minimum and still contains the walls and piston stroke.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Raised the shared view rectangle so the phone fill test passes**
- **Found during:** Task 2 (Add the preview, portrait frame, and README plan)
- **Issue:** The planned rectangle `minY -0.15` to `maxY 1.55` at `x = ±1.30` fills 0.277 of a 390 by 844 phone after `VIEWPORT_INSET` of 16px. The portrait test requires a height fraction greater than 0.28.
- **Fix:** Set `maxY` to `1.6` on both `viewBounds` and `PORTRAIT_VIEW_BOUNDS["hydraulic-fountain"]`. Wall and piston endpoints stayed the Plan 01 numbers. Physics files were not edited.
- **Files modified:** `web/src/catalog/scene-records.ts`, `web/src/catalog/portrait-bounds.ts`
- **Verification:** `bun run test:unit -- tests/portrait-bounds.test.ts` passes, and every listed wall and piston point is inside the frame.
- **Committed in:** `0227f2e` (Task 2 commit)

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The camera inset made the planned height miss the phone-fill minimum by a small margin. The extra 5 cm of headroom keeps the chambers, throat, and piston on screen.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 30-03. The hash route `#/scene/hydraulic-fountain` is in the catalog, the hash-path table, and the capture plan. Chromium open, play, pause, and reset proof is still Plan 03. `just readme-svg` was not run. `shell.spec.ts` still mentions nineteen demos; Plan 03 owns the Playwright specs.

---
*Phase: 30-periodic-hydraulic-fountain*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: web/src/catalog/scenes.ts
- FOUND: web/src/catalog/scene-records.ts
- FOUND: web/src/catalog/previews.tsx
- FOUND: web/src/catalog/portrait-bounds.ts
- FOUND: 3ef6893
- FOUND: 0227f2e

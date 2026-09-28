---
phase: 33-stacked-drip-fidget
plan: "02"
subsystem: ui
tags: [solidjs, catalog, preview, portrait-bounds, readme-svg]

requires:
  - phase: 33-stacked-drip-fidget
    provides: stacked-drip scene module and allowlisted scene id from plan 33-01
provides:
  - Catalog record stacked-drip immediately after liquid-bubbler
  - Static teal tray preview, phone frame, empty README cues, and showcase-only credit
affects: [33-03 playground framing, 33-04 return stroke]

tech-stack:
  added: []
  patterns:
    - "Watch-first catalog records use withGravitySlider([]) and SHOWCASE-only inspiration"
    - "Stacked Drip preview uses a local teal DRIP so the shared gold DRIP stays unchanged"

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
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/tests/demo-media-model.test.ts

key-decisions:
  - "Kept the planned view rectangle because Plan 01 did not retune walls, pivots, or stroke."
  - "Set the stacked-drip phone height floor to 0.39 because the fitted iPhone fraction is 0.391."
  - "This summary records the catalog evidence and does not approve the work."

patterns-established:
  - "Stacked Drip stays watch-first: play, pause, reset, and the shared gravity slider."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-28T00:43:35Z

duration: 7min
completed: 2026-09-28
---

# Phase 33 Plan 02: Stacked Drip Catalog Summary

**Catalog lists Stacked Drip after Liquid Bubbler as a watch-first scene with a teal tray preview, a phone frame, and showcase-only credit**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-28T00:36:15Z
- **Completed:** 2026-09-28T00:43:35Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- `stacked-drip` is the twenty-third ready scene, immediately after `liquid-bubbler`, at `#/scene/stacked-drip`.
- The static preview is three tilted trays under a teal drip. The shell caption stays `Static preview`.
- The phone rectangle contains the reservoir, three trays at rest and at the pour angle, the walls, the divider, and the plate at rest and at the top of the stroke.
- Credits point at `crates/liquidfun-wasm/src/scene/stacked_drip.rs` and the LiquidFun showcase link only.

## Task Commits

Each task was committed atomically:

1. **Task 1: Append the Stacked Drip catalog entry** - `5a82e7d` (feat)
2. **Task 2: Extend the catalog unit contracts** - `19a6867` (feat)

## Files Created/Modified

- `web/src/catalog/scenes.ts` - Appends `stacked-drip` after `liquid-bubbler`
- `web/src/catalog/scene-records.ts` - Watch-first Stacked Drip record and view bounds
- `web/src/catalog/previews.tsx` - Still SVG of three trays under a teal drip
- `web/src/catalog/portrait-bounds.ts` - Same rectangle as `viewBounds`
- `web/src/player/runtime.ts` - Page summary counts twenty-three demos
- `web/scripts/readme-svg/plans.ts` - `{ id: "stacked-drip", cues: [] }`
- `web/scripts/demo-media/model.ts` - Center-click capture stub
- `web/e2e/player-helpers.ts` - Hash path for `stacked-drip`
- `web/tests/scenes.test.ts` - Twenty-three-scene order, titles, and watch-first list
- `web/tests/scene-catalog-controls.test.ts` - Gravity-only Stacked Drip controls
- `web/tests/scene-catalog-credits.test.ts` - Scene module path and showcase-only inspiration
- `web/tests/portrait-bounds.test.ts` - Height floor and wall, tray, and plate points
- `web/tests/demo-media-model.test.ts` - Capture plan length and last id

## Decisions Made

- The view rectangle stays `{ minX: -0.86, minY: -0.16, maxX: 1.68, maxY: 2.18 }`. Plan 01's finished walls, pivots, stroke, and plate match the endpoints that rectangle was sized for.
- `MIN_HEIGHT_FRACTION["stacked-drip"]` is **0.39**. The fitted 390 by 844 frame covers about 0.391 of the phone height, under the starting 0.40 floor. The rectangle itself was not enlarged.
- The preview fill is a local `#40C4C4`. The module-level `DRIP` stays `#F2B040` for Liquid Bubbler.
- This summary records that evidence. It does not approve the work.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Appended Stacked Drip's gravity id to the recreating-control list**
- **Found during:** Task 2 (Extend the catalog unit contracts)
- **Issue:** The catalog control test compares every recreating control id, in scene order, to a locked list. Stacked Drip's gravity slider added one more `"gravity"` and the comparison failed.
- **Fix:** Appended one `"gravity"` to `RECREATING_CONTROL_IDS`.
- **Files modified:** web/tests/scene-catalog-controls.test.ts
- **Verification:** `bun run test:unit` for the catalog suite exits 0.
- **Committed in:** `19a6867` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The extra gravity id is the shared slider the plan already required. No scene behavior changed.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 33-03. The catalog, portrait frame, and empty-cue README plan exist. README raster export was not run.
- Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, and Liquid Tumbler records were not retuned.
- Independent review is still required. This summary does not approve the implementation.

## Test commands

- `cd web && bun run typecheck`
- `cd web && bun run test:unit -- tests/scenes.test.ts tests/scene-catalog-controls.test.ts tests/scene-catalog-credits.test.ts tests/portrait-bounds.test.ts tests/readme-svg.test.ts tests/demo-media-model.test.ts`

## Self-Check: PASSED

- FOUND: web/src/catalog/scene-records.ts
- FOUND: web/src/catalog/previews.tsx
- FOUND: web/src/catalog/portrait-bounds.ts
- FOUND: web/scripts/readme-svg/plans.ts
- FOUND: 5a82e7d
- FOUND: 19a6867

---

*Phase: 33-stacked-drip-fidget*
*Completed: 2026-09-28*

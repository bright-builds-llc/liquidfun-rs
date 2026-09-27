---
phase: 31-sinusoidal-wave-tank
plan: "02"
subsystem: playground-catalog
tags: [catalog, solidjs, wave-tank, preview, portrait]

requires:
  - phase: 31-01
    provides: Native wave-tank scene with the starting pool geometry
provides:
  - Catalog id wave-tank after hydraulic-fountain, watch-first with the shared gravity slider
  - Static raised-end preview, phone frame, empty-cue README plan, and showcase-only credit
affects: [31-03-browser-proof]

tech-stack:
  added: []
  patterns:
    - "Watch-first catalog records use WATCH_FIRST_HINT and withGravitySlider([])"
    - "Portrait frame matches viewBounds and includes the platform at rest and at the crest"

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
  - "Used the plan pool rectangle because Plan 01 kept the starting wall and stroke constants"
  - "Passing catalog unit tests are evidence only. This implementing agent did not approve the work."

patterns-established:
  - "Wave Tank credits stay host-locked to scene/wave_tank.rs with the LiquidFun showcase link only"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-09-27T17-22-43
generated_at: 2026-09-27T18:29:38Z

duration: 5min
completed: 2026-09-27
---

# Phase 31 Plan 02: Wave Tank Catalog Summary

**Wave Tank is a watch-first catalog scene after Hydraulic Fountain, with a static raised-end preview, a phone frame around the pool, and showcase-only credit**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-27T18:24:49Z
- **Completed:** 2026-09-27T18:29:38Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- The catalog lists twenty-one ready scenes. `wave-tank` follows `hydraulic-fountain`. `wave-machine` stays first, and its rocking-tank record is unchanged.
- Wave Tank is watch-first aside from the shared gravity slider. The static preview shows a level pool and a raised end platform. The caption `Static preview` stays in `DemoNavigation`.
- The phone frame is `{ minX: -0.12, minY: -0.28, maxX: 1.52, maxY: 1.02 }` for both `viewBounds` and `PORTRAIT_VIEW_BOUNDS`. The README plan is `{ id: "wave-tank", cues: [] }`. Credits point at `scene/wave_tank.rs` and the LiquidFun showcase link only.

## Task Commits

Each task was committed atomically:

1. **Task 1: Append the Wave Tank catalog entry** - `8ab8d17` (feat)
2. **Task 2: Extend the catalog unit contracts** - `98ff0e8` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/src/catalog/scenes.ts` - `wave-tank` after `hydraulic-fountain`
- `web/src/catalog/scene-records.ts` - Watch-first Wave Tank record and showcase-only credit
- `web/src/catalog/previews.tsx` - Still pool with a raised end slab, vertical face, and water band
- `web/src/catalog/portrait-bounds.ts` - Phone rectangle matching `viewBounds`
- `web/src/player/runtime.ts` - Page summary says twenty-one demos
- `web/scripts/readme-svg/plans.ts` - `{ id: "wave-tank", cues: [] }`
- `web/scripts/demo-media/model.ts` - Center-click capture stub
- `web/e2e/player-helpers.ts` - Hash path `#/scene/wave-tank`
- `web/tests/scenes.test.ts` - Twenty-one scene order, title, ready count, and watch-first hint
- `web/tests/scene-catalog-controls.test.ts` - Gravity-only controls
- `web/tests/scene-catalog-credits.test.ts` - Scene path and showcase href
- `web/tests/portrait-bounds.test.ts` - Height floor `0.20` and pool endpoints at rest and at the crest
- `web/tests/demo-media-model.test.ts` - Capture plans end at `wave-tank`

## Decisions Made

- Used the plan rectangle `{ minX: -0.12, minY: -0.28, maxX: 1.52, maxY: 1.02 }`. Plan 01 kept stroke `0.16` and the starting wall endpoints, so those points sit inside the frame with the platform at rest and at the crest.
- Passing catalog unit tests are evidence only. Per D-11 this implementing agent did not approve the work.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Counted Wave Tank's gravity slider in the recreating-control list**
- **Found during:** Task 2 (Extend the catalog unit contracts)
- **Issue:** `RECREATING_CONTROL_IDS` lists every recreating control in catalog order. Wave Tank's shared gravity slider added one `gravity` id and the full-catalog assertion failed.
- **Fix:** Appended one `"gravity"` entry so the expected list matches the twenty-one scene walk.
- **Files modified:** `web/tests/scene-catalog-controls.test.ts`
- **Verification:** `bun run test:unit` for the named catalog tests passed, including the recreates assertion
- **Committed in:** `98ff0e8`

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The extra list entry is the gravity slider the plan already required. No Wave Machine retune, no new control, and no shell edit.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 31-03. The catalog advertises `#/scene/wave-tank`. Browser smoke, shell copy in `web/e2e/shell.spec.ts`, and `.catalog-card` coverage stay in that plan. README raster export is not this plan's gate.

This summary records evidence and does not approve the work.

## Self-Check: PASSED

- FOUND: web/src/catalog/scenes.ts
- FOUND: web/src/catalog/scene-records.ts
- FOUND: web/src/catalog/previews.tsx
- FOUND: web/src/catalog/portrait-bounds.ts
- FOUND: 8ab8d17
- FOUND: 98ff0e8

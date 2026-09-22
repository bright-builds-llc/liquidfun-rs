---
phase: 26-catalog-shell-and-basin-scenes
plan: "04"
subsystem: testing
tags: [catalog, vitest, demo-media, particles, liquid-timer, play-02, play-03]

requires:
  - phase: 26-catalog-shell-and-basin-scenes
    provides: Eight-scene catalog records with ready:true, empty controls, and pinned credits
provides:
  - Vitest eight-scene catalog contract with pinned inspiration labels and hrefs
  - SCENE_CAPTURE_PLANS entries for particles and liquid-timer matching SCENE_IDS order
affects:
  - 26-05 player smoke / e2e eight-scene coverage
  - Demo media capture allowlist for new basin scenes

tech-stack:
  added: []
  patterns:
    - Capture plans for watch-first scenes use no-op center click with interactionStep 180
    - Catalog credit tests assert label+href pairs, not href-only arrays

key-files:
  created: []
  modified:
    - web/tests/scenes.test.ts
    - web/scripts/demo-media/model.ts

key-decisions:
  - "Kept Plan 03 eight-scene Vitest tables; Task 1 only strengthened credit label asserts"
  - "Watch-first capture stubs use center click SceneAction so capture scripts keep a required action"

patterns-established:
  - "SCENE_CAPTURE_PLANS must append in SCENE_IDS order; assertSceneCapturePlanCoverage fails import on drift"
  - "Basin credit regression locks Pinned *.js/*.h labels at commit 7f204021"

requirements-completed: [PLAY-02, PLAY-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:45:24Z

duration: 2min
completed: 2026-09-22
---

# Phase 26 Plan 04: Eight-Scene Catalog Contract Summary

**Vitest locks eight-scene catalog metadata plus pinned credit labels, and demo-media capture plans now cover Particles and Liquid Timer in SCENE_IDS order.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-22T00:43:40Z
- **Completed:** 2026-09-22T00:45:24Z
- **Tasks:** 2/2
- **Files modified:** 2

## Accomplishments

- Strengthened `scenes.test.ts` to assert pinned Particles/LiquidTimer inspiration **labels and hrefs**, empty controls, and host-locked implementation paths (eight-scene order/copy already from Plan 03).
- Appended `particles` and `liquid-timer` `SCENE_CAPTURE_PLANS` after `water-wheel` with watch-first center-click stubs; coverage assert no longer throws on import.
- Confirmed `navigation.test.ts` and `demo-media-model.test.ts` needed no length-6 hardcoding updates.

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand scenes.test.ts to eight scenes and credits** - `cf88627` (test)
2. **Task 2: Sync SCENE_CAPTURE_PLANS with eight SCENE_IDS** - `77b187e` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `web/tests/scenes.test.ts` - Pinned basin inspiration label+href asserts; empty-control length checks
- `web/scripts/demo-media/model.ts` - Capture plans for particles and liquid-timer

## Decisions Made

- Did not revert or rewrite Plan 03 eight-scene Vitest tables; only completed remaining credit-label contract from 26-04.
- Left `demo-media-model.test.ts` unchanged because it already derives expected ids from `SCENE_IDS`.

## Deviations from Plan

None - plan executed exactly as written (Task 1 scope reduced to remaining asserts after Plan 03 already extended eight-scene tables).

## Issues Encountered

None. Baseline `demo-media-model` suite failed import until Task 2 as expected from `assertSceneCapturePlanCoverage`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 26-05 (player smoke / e2e for eight scenes and watch-first play/pause/reset).
- Capture allowlist matches catalog; Chromium smoke can include the new routes without plan-length drift.

## Self-Check: PASSED

- FOUND: `web/tests/scenes.test.ts` pinned credit label asserts
- FOUND: `web/scripts/demo-media/model.ts` particles and liquid-timer capture plans
- FOUND: `cf88627` (Task 1)
- FOUND: `77b187e` (Task 2)

---
phase: 27-material-flag-groups
plan: "05"
subsystem: testing
tags: [vitest, catalog-contract, demo-media, capture-plans, watch-first]

requires:
  - phase: 27-material-flag-groups
    provides: Eleven-scene catalog records with pinned material credits and static previews
provides:
  - Vitest eleven-scene catalog metadata and credit contract
  - SCENE_CAPTURE_PLANS synchronized with eleven SCENE_IDS
affects: [27-06, demo-media-capture, web-player-smoke]

tech-stack:
  added: []
  patterns:
    - Watch-first capture stubs use center click SceneAction with interactionStep 180
    - assertSceneCapturePlanCoverage fails import when plan length/order diverges from SCENE_IDS

key-files:
  created: []
  modified:
    - web/scripts/demo-media/model.ts
    - web/tests/scenes.test.ts

key-decisions:
  - "Task 1 eleven-scene Vitest contract was already locked in 27-04 for typecheck; 27-05 verified and did not rewrite it"
  - "Watch-first material capture stubs reuse center click no-op SceneAction like Particles and Liquid Timer"

patterns-established:
  - "SCENE_CAPTURE_PLANS must grow whenever SCENE_IDS grows; coverage assert is the fail-closed gate"

requirements-completed: [MAT-01, MAT-02, MAT-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:58:24Z

duration: 1min
completed: 2026-09-22
---

# Phase 27 Plan 05: Catalog unit contract Summary

**Vitest locks the eleven-scene catalog order, UI-SPEC copy, empty controls, and pinned 7f204021 credits; demo-media capture plans cover surface-tension, elastic-particles, and rigid-particles**

## Performance

- **Duration:** 1 min
- **Started:** 2026-09-22T05:57:00Z
- **Completed:** 2026-09-22T05:58:24Z
- **Tasks:** 2
- **Files modified:** 1 new in this plan (`web/scripts/demo-media/model.ts`); Task 1 verified prior `web/tests/scenes.test.ts` contract

## Accomplishments

- Confirmed eleven-scene Vitest contract: order length 11, UI-SPEC descriptions/hints, empty controls for the three material scenes, six pinned inspiration blob URLs, host-locked implementation paths, and no `google/liquidfun` implementationPath
- Appended watch-first center-click `SCENE_CAPTURE_PLANS` for Surface Tension, Elastic Particles, and Rigid Particles so `assertSceneCapturePlanCoverage` matches `SCENE_IDS`
- `navigation.test.ts` needed no length-8 hardcode updates; it already derives from `SCENE_IDS`

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand scenes.test.ts to eleven scenes and credits** - `8df513c` (feat, completed during 27-04 typecheck; verified green in 27-05 with no further diff)
2. **Task 2: Sync SCENE_CAPTURE_PLANS with eleven SCENE_IDS** - `85526d7` (feat)

**Plan metadata:** pending final docs commit

## Files Created/Modified

- `web/tests/scenes.test.ts` - Eleven-scene catalog contract (verified; prior 27-04 commit)
- `web/scripts/demo-media/model.ts` - Three material watch-first capture plan stubs after liquid-timer
- `web/tests/navigation.test.ts` - No change required
- `web/tests/demo-media-model.test.ts` - No change required (already asserts against `SCENE_IDS`)

## Decisions Made

- Kept Task 1 as verification of the 27-04 eleven-scene Vitest tables rather than rewriting a green contract
- Reused the Particles/Liquid Timer center-click capture stub pattern for the three material scenes

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written.

### Pre-satisfied work

**1. [Prior plan] Task 1 catalog Vitest contract already present**
- **Found during:** Task 1
- **Issue:** `web/tests/scenes.test.ts` already listed eleven scenes, UI-SPEC tables, empty controls, pinned credits, and the `google/liquidfun` negative assert from 27-04 Task 2 typecheck work
- **Fix:** Verified acceptance criteria and unit tests; no additional edit or rewrite
- **Files modified:** none in 27-05
- **Verification:** `bun run test:unit -- tests/scenes.test.ts tests/navigation.test.ts` exits 0; acceptance `rg` checks pass
- **Committed in:** `8df513c` (27-04)

---

**Total deviations:** 0 auto-fixed; 1 pre-satisfied carry-in from 27-04
**Impact on plan:** No scope creep; Task 2 was the remaining unfinished work for this plan.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Catalog unit contract and capture-plan allowlist are synchronized at eleven scenes
- Ready for 27-06 Chromium smoke / player proof for the three material scenes

## Self-Check: PASSED

- `web/scripts/demo-media/model.ts` FOUND
- `web/tests/scenes.test.ts` FOUND
- Commit `85526d7` FOUND
- Commit `8df513c` FOUND (Task 1 prior)

---
*Phase: 27-material-flag-groups*
*Completed: 2026-09-22*

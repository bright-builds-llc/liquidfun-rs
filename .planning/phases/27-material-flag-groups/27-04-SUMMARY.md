---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T06:05:00Z
phase: 27-material-flag-groups
plan: "04"
subsystem: ui
tags: [catalog, solidjs, static-preview, credits, watch-first]

requires:
  - phase: 27-material-flag-groups
    provides: WASM surface-tension, elastic-particles, and rigid-particles SceneId factories
provides:
  - Eleven-scene catalog with ready material flag-group entries
  - Pinned inspiration credits at 7f204021
  - Static SVG previews for Surface Tension, Elastic Particles, Rigid Particles
  - PAGE_SUMMARY eleven-demo copy
affects: [27-05, 27-06, playground-shell]

tech-stack:
  added: []
  patterns:
    - Append-only SCENE_IDS after liquid-timer with watch-first empty controls
    - Host-locked implementationPath via sceneSource; pinned blob inspiration URLs
    - Still token-only SVG preview cases captioned Static preview by DemoNavigation

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/previews.tsx
    - web/src/player/runtime.ts
    - web/tests/scenes.test.ts

key-decisions:
  - "Extended scenes.test.ts eleven-scene Record tables in Task 2 so typecheck passes after SceneId grew"
  - "Rigid preview uses stroke outlines; Elastic uses soft ellipses with tilted blue box to contrast solidity"

patterns-established:
  - "Material watch-first scenes reuse WATCH_FIRST_HINT and controls: [] like Particles/Liquid Timer"
  - "VerticalWallBasin shared SVG helper for the three material previews"

requirements-completed: [MAT-01, MAT-02, MAT-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:55:42Z

duration: 2min
completed: 2026-09-22
---

# Phase 27 Plan 04: Catalog credits previews Summary

**Eleven-scene catalog advertises Surface Tension, Elastic Particles, and Rigid Particles with pinned 7f204021 credits, watch-first empty controls, still SVG previews, and PAGE_SUMMARY eleven-demo copy**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-22T05:52:55Z
- **Completed:** 2026-09-22T05:55:42Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Appended `surface-tension`, `elastic-particles`, and `rigid-particles` after the eight existing demos with `ready: true`, UI-SPEC descriptions, and `WATCH_FIRST_HINT`
- Host-locked implementation paths plus six pinned JS/C++ inspiration labels at commit `7f20402173fd143a3988c921bc384459c6a858f2`
- Replaced PAGE_SUMMARY eight → eleven; added still SVG preview cases using red/green/blue/rigid tokens
- Scene vitest tables updated so `bun run typecheck` and `vitest run tests/scenes.test.ts` pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Append scene records and pinned credits** - `56829f9` (feat)
2. **Task 2: Static SVG previews for three material scenes** - `8df513c` (feat)

**Plan metadata:** `02bcce1` (docs: complete plan)

## Files Created/Modified

- `web/src/catalog/scenes.ts` - Eleven SCENE_IDS/SCENES with material credits and empty controls
- `web/src/player/runtime.ts` - PAGE_SUMMARY eleven-demo sentence
- `web/src/catalog/previews.tsx` - Surface Tension / Elastic / Rigid still SVG cases
- `web/tests/scenes.test.ts` - Eleven-scene Record tables, titles, credits asserts

## Decisions Made

- Updated `web/tests/scenes.test.ts` in Task 2 (not listed in plan files) so exhaustive `Record<SceneId, …>` tables compile after catalog growth
- Preview art: filled soft ellipses for Elastic; stroke-only firmer outlines for Rigid; upright blue box for Surface Tension

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended scenes.test.ts for eleven SceneIds**
- **Found during:** Task 2 (typecheck after catalog append)
- **Issue:** `UI_SPEC_DESCRIPTIONS`, `UI_SPEC_HINTS`, and `expectedPaths` were `Record<SceneId, string>` still keyed for eight scenes, so `bun run typecheck` failed
- **Fix:** Added the three material entries, watch-first ids, titles/order asserts, and pinned credit expects
- **Files modified:** `web/tests/scenes.test.ts`
- **Verification:** `cd web && bun run typecheck` exit 0; `bunx vitest run tests/scenes.test.ts` 13/13 pass
- **Committed in:** `8df513c` (Task 2 commit)

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for plan verification success criteria; no scope creep beyond catalog honesty.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Catalog/player chrome advertises all three material scenes; Plans 27-05 / 27-06 can extend e2e smoke and shell focus-wrap to eleven demos / Rigid Particles last
- Do not rewrite WASM scene modules from this plan

## Self-Check: PASSED

- FOUND: `.planning/phases/27-material-flag-groups/27-04-SUMMARY.md`
- FOUND: commits `56829f9`, `8df513c` in `git log --oneline --grep=27-04`

***
*Phase: 27-material-flag-groups*
*Completed: 2026-09-22*

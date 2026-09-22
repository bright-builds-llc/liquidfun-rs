---
phase: 26-catalog-shell-and-basin-scenes
plan: "03"
subsystem: ui
tags: [catalog, particles, liquid-timer, watch-first, solidjs, static-preview]

requires:
  - phase: 26-catalog-shell-and-basin-scenes
    provides: Particles and Liquid Timer WASM SceneId tokens and factory builds
provides:
  - Eight-scene catalog records with ready:true and empty controls
  - Pinned JS and C++ inspiration credits at commit 7f20402173fd143a3988c921bc384459c6a858f2
  - Static SVG previews for particles and liquid-timer
  - SceneControls hidden when controls.length === 0
  - PAGE_SUMMARY eight-demo copy
affects:
  - 26-04 player smoke / e2e eight-scene coverage
  - DemoNavigation sidebar and Kobalte drawer list length

tech-stack:
  added: []
  patterns:
    - Append-only SCENE_IDS after existing six; never reorder
    - Watch-first scenes use controls: [] and hide .scene-controls
    - New preview water fills use #39D3C7; existing six preview colors unchanged

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/previews.tsx
    - web/src/player/runtime.ts
    - web/src/components/SceneControls.tsx
    - web/tests/scenes.test.ts

key-decisions:
  - "Catalog appends particles then liquid-timer after water-wheel with ready:true and empty controls"
  - "Inspiration cites pinned Particles/LiquidTimer JS and C++ tests at 7f204021; implementation stays host-locked sceneSource paths"
  - "SceneControls returns null when controls.length === 0 rather than empty chrome"

patterns-established:
  - "WATCH_FIRST_HINT shared string for basin scenes without pointer/control chrome"
  - "ACCENT_WATER #39D3C7 for Phase 26 preview water; keep WATER #4DA3FF on older previews"

requirements-completed: [PLAY-02, PLAY-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:42:37Z

duration: 2min
completed: 2026-09-22
---

# Phase 26 Plan 03: Catalog Shell Append Summary

**Eight-scene SolidJS catalog advertises Particles and Liquid Timer as ready watch-first demos with pinned credits, static SVG previews, and no empty controls chrome.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-22T00:40:19Z
- **Completed:** 2026-09-22T00:42:37Z
- **Tasks:** 2/2
- **Files modified:** 5

## Accomplishments

- Appended `particles` and `liquid-timer` to `SCENE_IDS` / `SCENES` after the six existing demos with `ready: true`, watch-first hints, and `controls: []`.
- Wired pinned inspiration labels to JS and C++ blob URLs at commit `7f20402173fd143a3988c921bc384459c6a858f2`; implementation paths stay under `crates/liquidfun-wasm/src/scene/`.
- Added still SVG previews (accent water `#39D3C7`) and hide empty `SceneControls`; updated `PAGE_SUMMARY` to eight demos.

## Task Commits

Each task was committed atomically:

1. **Task 1: Append scene records and pinned credits** - `5cf5a21` (feat)
2. **Task 2: Static previews and hide empty SceneControls** - `9e52f21` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `web/src/catalog/scenes.ts` - Eight-scene catalog, credits, empty controls
- `web/src/catalog/previews.tsx` - Particles and Liquid Timer static SVGs
- `web/src/player/runtime.ts` - PAGE_SUMMARY eight-demo sentence
- `web/src/components/SceneControls.tsx` - Null render when no controls
- `web/tests/scenes.test.ts` - Eight-scene order, copy, credits, empty controls

## Decisions Made

- Kept optional LiquidFun showcase inspiration entries for consistency with Fountain-style demos.
- Preferred hide-in-`SceneControls` over gating in `PlayerSceneChrome`.
- Did not rewrite WASM scene modules; catalog only uses factory tokens already exposed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended catalog unit tests for eight SceneId keys**
- **Found during:** Task 2 (typecheck after catalog append)
- **Issue:** `web/tests/scenes.test.ts` `Record<SceneId, string>` maps omitted `particles` / `liquid-timer`, so `bun run typecheck` failed even after `previews.tsx` was exhaustive.
- **Fix:** Updated descriptions, hints, order/title/ready counts, credit hrefs, empty-control asserts, and keyboard-reminder scope to interactive scenes only.
- **Files modified:** `web/tests/scenes.test.ts`
- **Verification:** `bun run typecheck` exit 0; `bun test tests/scenes.test.ts` 13 pass
- **Committed in:** `9e52f21` (Task 2)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for Task 2 acceptance (`typecheck` exit 0). No scope creep beyond catalog advertisement.

## Issues Encountered

None beyond the expected typecheck gap until previews and tests caught up.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 26-04 (player smoke / e2e for eight scenes and watch-first play/pause/reset).
- Kobalte drawer and DemoNavigation patterns unchanged; `.catalog-card` remains absent.

## Self-Check: PASSED

- FOUND: `web/src/catalog/scenes.ts` eight ids and pinned credits
- FOUND: `web/src/catalog/previews.tsx` particles/liquid-timer cases
- FOUND: `5cf5a21` (Task 1)
- FOUND: `9e52f21` (Task 2)
- FOUND: `bun run typecheck` exit 0
- FOUND: `bun test tests/scenes.test.ts` 13 pass

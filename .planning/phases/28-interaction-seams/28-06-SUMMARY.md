---
phase: 28-interaction-seams
plan: "06"
subsystem: web-catalog
tags: [catalog, scenes, static-preview, PAGE_SUMMARY, ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]

requires:
  - phase: 28-interaction-seams
    provides: Five WASM interaction scenes from Plans 01–05 ready for catalog advertisement
provides:
  - Sixteen-scene catalog with soup family controls, pinned credits, and Static preview SVGs
  - PAGE_SUMMARY sixteen-demo copy
affects:
  - 28-07 Vitest catalog contract
  - 28-08 Chromium player-smoke

tech-stack:
  added: []
  patterns:
    - Append-only SCENE_IDS after rigid-particles with ready:true and host-locked sceneSource paths
    - Watch-first empty controls vs action/runtimePreset live controls without recreating

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/previews.tsx
    - web/src/player/runtime.ts
    - web/tests/scenes.test.ts

key-decisions:
  - "Left PlaygroundShell and THIRD_PARTY_NOTICES unchanged — no eleven-demo hardcoding and no new adapted notices"
  - "Extended scenes.test.ts Record tables in Task 2 so typecheck passes after SceneId grew"

patterns-established:
  - "Pattern: Interaction scenes reuse WATCH_FIRST_HINT for soup/wave-machine; live presets use recreates:false"
  - "Pattern: Pinned inspiration at 7f204021… with Pinned *.js / *.h labels plus optional SHOWCASE"

requirements-completed: [ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:30:04Z

duration: 2min
completed: 2026-09-22
---

# Phase 28 Plan 06: Catalog Chrome Summary

**Sixteen-scene catalog with Soup/Stirrer/Impulse/Wave/Theo controls, pinned credits, Static preview SVGs, and sixteen-demo PAGE_SUMMARY**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-22T14:27:14Z
- **Completed:** 2026-09-22T14:30:04Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Appended `soup`, `soup-stirrer`, `impulse`, `wave-machine`, `theo-jansen` after the eleven existing ids with `ready: true`
- Locked UI-SPEC controls: Soup/Wave watch-first empty; Stirrer `Toggle paddle rail`; Impulse `Push` force/impulse; Theo `Motor direction` forward/reverse
- Pinned inspiration hrefs at `7f20402173fd143a3988c921bc384459c6a858f2` with ten Pinned *.js / *.h labels
- Added five still token-only SVG preview cases; caption remains `Static preview` via existing DemoNavigation
- `PAGE_SUMMARY` now says sixteen demos; no `.catalog-card`; no sealed parity copy

## Task Commits

Each task was committed atomically:

1. **Task 1: Append scene records, controls, credits, PAGE_SUMMARY** - `9f3f72c` (feat)
2. **Task 2: Static SVG previews for five interaction scenes** - `294215e` (feat)

**Plan metadata:** `d4bb727` (docs: complete plan)

## Files Created/Modified

- `web/src/catalog/scenes.ts` - Sixteen SCENE_IDS, records, controls, pinned credits
- `web/src/catalog/previews.tsx` - Five interaction Static preview SVGs
- `web/src/player/runtime.ts` - Sixteen-demo PAGE_SUMMARY
- `web/tests/scenes.test.ts` - Record tables extended for typecheck after SceneId growth

## Decisions Made

- PlaygroundShell unchanged — it already maps `SCENES` and has no eleven-demo / Rigid-Particles-last hardcoding
- THIRD_PARTY_NOTICES unchanged — no new adapted upstream geometry/text entries required for this catalog-only plan
- Minimal scenes.test.ts Record extensions in Task 2 (Plan 07 owns full Vitest contract)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended scenes.test.ts Record tables for typecheck**
- **Found during:** Task 2
- **Issue:** Growing `SceneId` left incomplete `Record<SceneId, string>` tables in `scenes.test.ts`, so `bun run typecheck` failed
- **Fix:** Added five interaction descriptions, hints, and implementation paths matching UI-SPEC
- **Files modified:** `web/tests/scenes.test.ts`
- **Verification:** `cd web && bun run typecheck` exits 0
- **Committed in:** `294215e`

---

**Total deviations:** 1 auto-fixed (1× Rule 3)
**Impact on plan:** Necessary for Task 2 verification; Plan 07 still owns fuller catalog asserts and capture plans.

## Issues Encountered

None.

## Known Stubs

None — catalog advertises ready WASM scenes already built in Plans 01–05; previews are intentional still SVGs captioned Static preview.

## Threat Flags

None — implementation paths stay host-locked under `crates/liquidfun-wasm/src/scene/`; no `.catalog-card`; no sealed-parity copy.

## Self-Check: PASSED

- FOUND: `web/src/catalog/scenes.ts`
- FOUND: `web/src/catalog/previews.tsx`
- FOUND: `web/src/player/runtime.ts`
- FOUND: `.planning/phases/28-interaction-seams/28-06-SUMMARY.md`
- FOUND: commit `9f3f72c`
- FOUND: commit `294215e`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Catalog chrome ready for Plan 07 Vitest contract and Plan 08 Chromium smoke
- Do not restore card grid; keep `MAX_ADVANCE_STEPS` at 4
- Plan 08 must update `SCENE_HASH_PATHS` and shell sixteen-demo asserts

---
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*

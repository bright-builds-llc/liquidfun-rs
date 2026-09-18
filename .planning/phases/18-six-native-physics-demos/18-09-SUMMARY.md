---
phase: 18-six-native-physics-demos
plan: "09"
subsystem: player
tags: [solidjs, wasm-session, scene-controls, scene-credits, web-04, web-08]

requires:
  - phase: 18-six-native-physics-demos
    provides: Six native scene modules, catalog metadata, and Dam Break-only player
provides:
  - All six catalog ids ready:true and constructible through loadSceneSession
  - SceneControls with Apply setting and construction reset sentence
  - SceneCredits with host-locked implementation, inspiration, and notices
  - One WASM session with dispose-on-switch via startScene/abandonScene
affects: [18-10, player-controls, scene-credits, pages-copy]

tech-stack:
  added: []
  patterns:
    - applyControl/applyAction poison and free like nextFrame
    - construction presets require Apply setting; runtime presets apply on change
    - host-locked sceneBlobUrl/noticesBlobUrl credits beside SiteFooter

key-files:
  created:
    - web/src/components/SceneControls.tsx
    - web/src/components/SceneCredits.tsx
    - web/src/components/scene-controls.ts
    - web/src/components/scene-credits.ts
    - web/src/player/runtime.ts
    - web/tests/controls.test.ts
    - web/tests/credits.test.ts
    - web/tests/runtime.test.ts
  modified:
    - web/src/physics/session.ts
    - web/src/catalog/scenes.ts
    - web/src/App.tsx
    - web/src/components/PlayerPanel.tsx
    - web/src/app.css
    - web/tests/session.test.ts
    - web/tests/scenes.test.ts

key-decisions:
  - "Construction CTA is Apply setting, not the single word Apply."
  - "Extract control/credit helpers to .ts so vitest node can test them without Solid JSX."
  - "Reset keeps last applied construction presets and startScene/abandonScene dispose the prior owner."

patterns-established:
  - "isReadySceneRoute plus loadSceneSession(id) construct any ready catalog id."
  - "Recreating presets show the reset sentence before Apply setting; runtime presets apply on change."
  - "SceneCredits builds implementation and notices hrefs only through host-locked helpers."

requirements-completed: [WEB-01, WEB-04, WEB-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T05:32:46Z

duration: 6min
completed: 2026-09-18
---

# Phase 18 Plan 09: Ready Flags, Player Controls, and Credits Summary

**All six catalog ids are ready and construct one WASM world each, with Apply setting construction presets, host-locked Scene source credits, and dispose-on-switch.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-09-18T05:26:47Z
- **Completed:** 2026-09-18T05:32:46Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- TypeScript session owner forwards `applyControl`/`applyAction` once and poisons plus frees on generated throw with the fixed `FAILED_MESSAGE`.
- Every `SCENE_IDS` record is `ready: true`; `isReadySceneRoute` replaced Dam Break-only routing; `App` constructs via `loadSceneSession(id)`.
- Recreating presets show `Changing this setting recreates the scene from its documented initial state.` before the noun-bearing `Apply setting` button. Runtime presets apply on change. Actions keep locked verb+noun labels.
- Player heading, status, and canvas name use the current scene title. Credits sit in the player as `Scene source` with `View scene source`, inspiration labels, and `Third-party notices`, `rel="noopener noreferrer"`, never `google/liquidfun` or `innerHTML`.
- Generation tokens, hidden-tab cap, copied frames, play/pause/reset, and dispose-on-switch remain. `App.tsx` is 593 lines.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: failing ready-flag and apply tests** - `01b41be` (test)
2. **Task 1 GREEN: session apply, ready flags, SceneControls, SceneCredits** - `165a4b3` (feat)
3. **Task 2: wire ready ids through one-session App lifecycle** - `3129b90` (feat)

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

## Files Created/Modified

- `web/src/physics/session.ts` — applyControl/applyAction with poison/free
- `web/src/catalog/scenes.ts` — all six ready:true
- `web/src/components/scene-controls.ts` — constructionHintVisible and Apply setting copy
- `web/src/components/SceneControls.tsx` — labeled presets/actions
- `web/src/components/scene-credits.ts` — implementationHref host lock
- `web/src/components/SceneCredits.tsx` — Scene source aside
- `web/src/player/runtime.ts` — isReadySceneRoute, titles, construction preset bag
- `web/src/App.tsx` — startScene/abandonScene, PAGE_SUMMARY, controls/credits mount
- `web/src/components/PlayerPanel.tsx` — scene-title chrome and children slot
- `web/src/app.css` — control/credit spacing and 44px selects
- `web/tests/session.test.ts` — apply forwarding and poison
- `web/tests/scenes.test.ts` — six ready ids
- `web/tests/controls.test.ts` — reset hint visibility
- `web/tests/credits.test.ts` — no google/liquidfun implementation href
- `web/tests/runtime.test.ts` — ready-route and construction entries

## Decisions Made

- Construction CTA is `Apply setting` (UI-SPEC FLAG), with the reset sentence visible first.
- Helper modules stay `.ts` so vitest's node environment can prove copy and URL rules without Solid JSX.
- Reset/retry call `startScene` with the last construction presets; scene switch clears that bag and disposes the prior owner.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extracted control/credit helpers to `.ts`**
- **Found during:** Task 1
- **Issue:** Vitest node + `include: tests/**/*.test.ts` cannot parse Solid JSX, so importing `SceneControls.tsx` failed import analysis.
- **Fix:** Moved `constructionHintVisible`, `implementationHref`, and locked copy constants into `scene-controls.ts` / `scene-credits.ts`; components keep the same literal strings for acceptance `rg`.
- **Files modified:** `web/src/components/scene-controls.ts`, `web/src/components/scene-credits.ts`, `web/src/components/SceneControls.tsx`, `web/src/components/SceneCredits.tsx`, `web/tests/controls.test.ts`, `web/tests/credits.test.ts`
- **Verification:** `bun run test:unit` for the four Task 1 suites exits 0
- **Committed in:** `165a4b3` (Task 1 GREEN)

**2. [Rule 2 - Missing Critical] Added runtime helper tests**
- **Found during:** Task 2
- **Issue:** `isReadySceneRoute` and construction-preset persistence are business logic with no unit coverage in the plan file list.
- **Fix:** Added `web/tests/runtime.test.ts` for ready-route, titles, and recreating-only construction entries.
- **Files modified:** `web/tests/runtime.test.ts`
- **Verification:** `cd web && bun run test:unit && bun run typecheck` exits 0
- **Committed in:** `3129b90` (Task 2)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Needed for testability and correctness. No scope creep. No pointer handlers or Pages deploy.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-10 polish/verification. All six ids construct one world, show locked controls/credits, and dispose on switch. Do not add canvas pointer handlers. Do not change Pages workflow or Vite base.

---
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: web/src/components/SceneControls.tsx
- FOUND: web/src/components/SceneCredits.tsx
- FOUND: web/src/player/runtime.ts
- FOUND: web/src/physics/session.ts
- FOUND: 01b41be
- FOUND: 165a4b3
- FOUND: 3129b90

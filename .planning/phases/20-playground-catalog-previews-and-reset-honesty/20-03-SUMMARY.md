---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "03"
subsystem: playground-ui
tags: [solidjs, show-keyed, reset-honesty, scene-controls]

requires:
  - phase: 20-playground-catalog-previews-and-reset-honesty
    provides: sceneControlsIdentity truthy sceneId:generation remount string
  - phase: 18-six-native-physics-demos
    provides: constructionEntriesForScene and DEFAULT_PRESET_VALUES
provides:
  - viewport prefersReducedMotion and isUsableViewport extracted from App
  - PlayerSceneChrome keyed Show remount of SceneControls
  - recreateScene clears constructionValues then bumps resetGeneration
affects: [20-05 Chromium Reset honesty, App.tsx file-lengths cap]

tech-stack:
  added: []
  patterns:
    - Solid Show keyed on sceneControlsIdentity remounts pendingValue
    - Reset/Retry assign a new empty construction bag before startScene
    - SceneCredits stay outside the keyed remount

key-files:
  created:
    - web/src/player/viewport.ts
    - web/src/components/PlayerSceneChrome.tsx
  modified:
    - web/src/App.tsx

key-decisions:
  - "Start resetGeneration at 1 and pass sceneControlsIdentity into Show keyed; never pass a raw generation number."
  - "recreateScene assigns constructionValues = {} then bumps resetGeneration then startScene."
  - "Keep SceneCredits outside the keyed Show so credits do not remount with selects."

patterns-established:
  - "Keyed remount lives in PlayerSceneChrome, not a React key= on SceneControls."
  - "Play and pause leave construction drafts and resetGeneration alone."

requirements-completed: [WEB-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T18:22:20Z

duration: 4min
completed: 2026-09-20
---

# Phase 20 Plan 03: Keyed SceneControls Remount Summary

**Reset and Retry clear the construction bag and remount SceneControls through Solid `Show keyed` so selects return to `DEFAULT_PRESET_VALUES` without growing App.tsx past 628 lines.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-20T18:18:27Z
- **Completed:** 2026-09-20T18:22:20Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Viewport helpers `prefersReducedMotion` and `isUsableViewport` live in `web/src/player/viewport.ts`; App.tsx imports them.
- `PlayerSceneChrome` remounts `SceneControls` with `<Show keyed>` on `sceneControlsIdentity(scene.id, resetGeneration)` and keeps `SceneCredits` outside that remount.
- `recreateScene` (Reset and Retry) assigns `constructionValues = {}`, bumps `resetGeneration`, then `startScene`. Play and pause do not clear the bag or remount controls.
- `App.tsx` is 617 physical lines. `cd web && bun run typecheck` exits 0. Runtime and controls unit tests pass (19 tests).

## Task Commits

Each task was committed atomically:

1. **Task 1: Extract viewport helpers and keyed PlayerSceneChrome**
   - `4105005` (feat): viewport helpers plus keyed chrome; App.tsx under the file-lengths cap
2. **Task 2: Clear the construction bag and bump resetGeneration only on recreateScene**
   - `a5deb59` (feat): empty bag then generation bump then startScene on Reset/Retry

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)._

## Files Created/Modified

- `web/src/player/viewport.ts` — exact `prefersReducedMotion` and `isUsableViewport` exports
- `web/src/components/PlayerSceneChrome.tsx` — keyed `SceneControls` plus unkeyed `SceneCredits`
- `web/src/App.tsx` — `resetGeneration` signal, chrome mount, Reset bag clear
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/deferred-items.md` — pre-existing xtask file-length note for 20-03

## Decisions Made

- Start `resetGeneration` at `1` and key `Show` on `sceneControlsIdentity`, because a raw `0` would hide all scene controls.
- Clear the construction bag in `recreateScene` before `startScene` so Apply-setting Large/High cannot survive Reset.
- Keep credits outside the keyed remount so only preset selects snapshot `pendingValue` again.
- Do not bump generation inside `applySceneControl`; unapplied construction drafts stay until Apply or Reset.

## Deviations from Plan

None - plan executed as written for in-scope files.

Task 1 omitted unused `setResetGeneration` until Task 2 because `noUnusedLocals` rejected the unused setter. Keyed `Show` children use `(_controlsIdentity) =>` because Solid's typed keyed `Show` requires a value parameter; the plan's `{() =>` form failed typecheck.

The Task 2 `file-lengths` command still fails on pre-existing `tools/xtask/tests/upstream_cli.rs` (642 lines). That file was already over the cap on HEAD; this plan did not edit it. Logged in `deferred-items.md`. Changed files: `App.tsx` 617, `PlayerSceneChrome.tsx` 41, `viewport.ts` 7.

## Authentication Gates

None.

## Issues Encountered

None beyond the pre-existing file-lengths finding above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 20-04 catalog-preview Chromium smoke and 20-05 Reset label honesty. Native scene files and `SceneControls` visuals are unmodified. WEB-03 chrome wiring is in place; Chromium proof remains 20-05 and 20-06.

## Self-Check: PASSED

---
*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Completed: 2026-09-20*

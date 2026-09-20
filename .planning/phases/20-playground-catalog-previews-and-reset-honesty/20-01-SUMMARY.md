---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "01"
subsystem: player-runtime
tags: [vitest, tdd, reset-honesty, scene-controls, solid-show]

requires:
  - phase: 18-six-native-physics-demos
    provides: DEFAULT_PRESET_VALUES, initialPresetValue, constructionEntriesForScene
provides:
  - sceneControlsIdentity truthy sceneId:generation remount string
  - empty construction bag emits no applyControl rows
  - documented DEFAULT_PRESET_VALUES initials via initialPresetValue
affects: [20-03, App.tsx keyed Show remount, recreateScene construction bag]

tech-stack:
  added: []
  patterns:
    - truthy sceneId:generation identity for Solid Show keyed remount
    - empty construction bag means WASM build([]) defaults
    - catalog maybeSceneById as the source of preset option tables in tests

key-files:
  created: []
  modified:
    - web/src/player/runtime.ts
    - web/tests/runtime.test.ts
    - web/tests/controls.test.ts

key-decisions:
  - "sceneControlsIdentity interpolates `${sceneId}:${generation}` so Solid Show stays truthy at generation 0."
  - "An empty construction bag yields no constructionEntriesForScene rows so Reset can rebuild WASM build([]) defaults."
  - "initialPresetValue tests load real catalog presets from maybeSceneById instead of duplicating option tables."

patterns-established:
  - "Keyed remount identity is a non-empty string, never a raw generation number."
  - "Reset honesty unit tests use catalog metadata, not copied option tables."

requirements-completed: [WEB-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T18:11:19Z

duration: 2min
completed: 2026-09-20
---

# Phase 20 Plan 01: Reset Identity and Documented Initials Summary

**Truthy `sceneId:generation` remount identity plus Vitest coverage that empty construction bags and `DEFAULT_PRESET_VALUES` drive Reset-honest chrome.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-20T18:09:09Z
- **Completed:** 2026-09-20T18:11:19Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `sceneControlsIdentity("dam-break", 0)` returns `"dam-break:0"` and is never `""`, so Solid `Show` can remount at generation 0.
- `constructionEntriesForScene` with `{}` returns `[]` for Dam Break and Color Mixer.
- Representative D-08 initials resolve through `DEFAULT_PRESET_VALUES`: Fountain `emission-rate` → `medium`, Float or Sink `body` → `wood`, Color Mixer `stir-speed` → `slow` / `mix-strength` → `strong`, Water Wheel `jet-strength` → `medium`, Dam Break `water-amount` → `medium`.
- Construction-bag comment now says last applied presets last until Reset or scene switch clears the bag.

## Task Commits

Each task was committed atomically:

1. **Task 1: Prove documented initials and an empty construction bag**
   - `a210d77` (test): empty-bag and `initialPresetValue` cases against existing helpers
2. **Task 2: Add sceneControlsIdentity and the Reset bag comment**
   - `892f24a` (test): failing identity cases
   - `3250fde` (feat): `${sceneId}:${generation}` export plus Reset bag comment

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)._

## Files Created/Modified

- `web/src/player/runtime.ts` — exports `sceneControlsIdentity`; Reset-honest construction-bag comment
- `web/tests/runtime.test.ts` — empty-bag and identity unit tests
- `web/tests/controls.test.ts` — documented `initialPresetValue` cases from catalog presets

## Decisions Made

- Interpolate `sceneId` with generation rather than stringify generation alone, because Solid `Show` treats `0` as falsy.
- Keep `constructionEntriesForScene` body unchanged; empty bags already return `[]`.
- Load real preset controls via `maybeSceneById` so tests cannot drift from catalog option ids.

## Deviations from Plan

None - plan executed exactly as written.

Task 1 tests passed on first run because `initialPresetValue` and empty-bag `constructionEntriesForScene` already existed; the plan required documenting that contract without production edits.

## Authentication Gates

None.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 20-02 catalog preview restore. `sceneControlsIdentity` is available for 20-03 keyed `Show` remount; App.tsx and `recreateScene` bag-clearing are still later-plan work. WEB-03 remains pending until 20-03, 20-05, and 20-06 wire and prove Reset chrome.

## Self-Check: PASSED

---
*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Completed: 2026-09-20*

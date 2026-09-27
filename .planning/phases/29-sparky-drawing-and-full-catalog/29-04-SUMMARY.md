---
phase: 29-sparky-drawing-and-full-catalog
plan: "04"
subsystem: ui
tags: [catalog, solid, svg, playground]

requires:
  - phase: 29-sparky-drawing-and-full-catalog
    provides: Drawing Particles and Sparky native scene ids
provides:
  - Nineteen-scene catalog ending liquid-tumbler, drawing-particles, sparky
  - Static Drawing and Sparky previews, phone frames, and README plan ids
affects: [playground-smoke, readme-svg]

tech-stack:
  added: []
  patterns:
    - "Scene records live in scene-records.ts and scenes.ts re-exports SCENES"
    - "New scene ids stay in SCENE_IDS order after liquid-tumbler"

key-files:
  created:
    - web/src/catalog/scene-records.ts
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/previews.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/src/components/scene-controls.ts
    - web/src/player/runtime.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts

key-decisions:
  - "Pinned credit constants stayed in scene-records.ts because the file remained under 629 lines"
  - "Drawing Material recreates the world as false and defaults to water; Sparky stays gravity-only"

patterns-established:
  - "Catalog records split from SCENE_IDS so adding a scene does not push scenes.ts over the file-length limit"

requirements-completed: [PLAY-01, FX-01, FX-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T07:07:06Z

duration: 8min
completed: 2026-09-27
---

# Phase 29 Plan 04: Catalog Scenes Summary

**The playground catalog lists nineteen ready scenes, with Drawing Particles and Sparky after Liquid Tumbler, each with a static preview, phone frame, and README plan**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-27T06:58:57Z
- **Completed:** 2026-09-27T07:07:06Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Split the seventeen existing records into `scene-records.ts` and re-exported `SCENES` plus the camera frames from `scenes.ts`.
- Appended Drawing Particles (Water and Elastic, material does not recreate the world) and watch-first Sparky after Liquid Tumbler.
- Added captioned static SVG previews, portrait rectangles that include the walls, README plan ids, and the nineteen-demo page summary.

## Task Commits

Each task was committed atomically:

1. **Task 1: Split scene records without new ids** - `1c6c373` (refactor)
2. **Task 2: Append both scenes and every closed SceneId companion** - `5cd57ca` (feat)

**Plan metadata:** docs commit records this summary only. State and roadmap stay with the orchestrator.

## Files Created/Modified

- `web/src/catalog/scene-records.ts` - `SCENES` records, pinned credits, and record helpers
- `web/src/catalog/scenes.ts` - `SCENE_IDS` ending `drawing-particles` then `sparky`, plus re-exports
- `web/src/catalog/previews.tsx` - still Drawing and Sparky SVGs
- `web/src/catalog/portrait-bounds.ts` - phone rectangles for both new ids
- `web/src/components/scene-controls.ts` - default Material value `water`
- `web/src/player/runtime.ts` - nineteen-demo page summary
- `web/scripts/readme-svg/plans.ts` - Drawing pointer-up cue and empty Sparky cues
- `web/scripts/demo-media/model.ts` - center-click capture plans for both ids
- `web/tests/scenes.test.ts` - nineteen-id order, descriptions, and hints
- `web/tests/scene-catalog-credits.test.ts` - pinned blob hrefs and implementation paths
- `web/tests/scene-catalog-controls.test.ts` - Material options and gravity-only Sparky
- `web/tests/portrait-bounds.test.ts` - height fraction 0.40 and wall points
- `web/tests/demo-media-model.test.ts` - capture order ending on Sparky

## Decisions Made

- Left the pinned credit constants in `scene-records.ts`. After both records were added the file was 600 lines, under the 629-line limit, so a `scene-credits.ts` split was unnecessary.
- Drawing uses `runtimePreset` so Material has `recreates: false`. Sparky uses `WATCH_FIRST_HINT` and gravity only.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Catalog ids, static previews, portrait frames, and README plan ids are in place for the nineteen-scene smoke suite. README raster export and Playwright updates stay with the later plan. No review acknowledgment was recorded for this plan.

## Self-Check: PASSED

- FOUND: web/src/catalog/scene-records.ts
- FOUND: 1c6c373
- FOUND: 5cd57ca

---
*Phase: 29-sparky-drawing-and-full-catalog*
*Completed: 2026-09-27*

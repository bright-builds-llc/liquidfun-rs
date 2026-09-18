---
phase: 18-six-native-physics-demos
plan: "02"
subsystem: catalog
tags: [solidjs, catalog, credits, tdd, blob-url]

requires:
  - phase: 17-shared-player-and-early-pages-delivery
    provides: Six-id catalog records and 40-hex SHA gate in readBuildInfo
provides:
  - Locked UI-SPEC descriptions and D-05 control contracts on SCENES
  - Host-locked sceneBlobUrl and noticesBlobUrl helpers
  - Dam Break-only ready flags so App cannot construct stub worlds
affects: [18-08, 18-09, catalog-cards, player-controls, scene-credits]

tech-stack:
  added: []
  patterns:
    - functional-core catalog metadata without Solid or WASM
    - host-locked GitHub blob URLs with 40-hex SHA or main fallback
    - construction presets inlined with recreates true

key-files:
  created:
    - web/src/catalog/links.ts
    - web/tests/links.test.ts
  modified:
    - web/src/catalog/scenes.ts
    - web/tests/scenes.test.ts

key-decisions:
  - "Include the UI-SPEC Poke jelly action on jelly-drop even though the machine-id table omitted it."
  - "Throw Scene source path is not allowlisted for traversal or non-scene implementation paths."
  - "Copy the 40-hex SHA pattern into links.ts instead of importing Vite env from build-info."

patterns-established:
  - "Catalog records own descriptions, preview ids, controls, and credits; chrome reads them later."
  - "Implementation hrefs stay on bright-builds-llc/liquidfun-rs; google/liquidfun is inspiration only."

requirements-completed: [WEB-04, WEB-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:18:19Z

duration: 2min
completed: 2026-09-18
---

# Phase 18 Plan 02: Catalog Control and Credit Metadata Summary

**Locked six-scene catalog records with UI-SPEC descriptions, D-05 controls, and host-locked GitHub blob URLs, while only Dam Break stays ready.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-18T04:15:36Z
- **Completed:** 2026-09-18T04:18:19Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Every `SCENES` record now carries the locked one-sentence description, its own `previewId`, two or three labeled controls, and credit paths.
- Recreating presets are only `water-amount`, `gravity`, `mix-strength`, `shape`, and `softness`.
- `sceneBlobUrl` and `noticesBlobUrl` host-lock `bright-builds-llc/liquidfun-rs` with a 40-hex SHA or `main` fallback, and reject traversal or upstream C++ implementation paths.
- `isReadySceneId` remains true only for `dam-break`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add descriptions, controls, and credits to SCENES**
   - `e6f4895` (test): failing catalog description, control, and credit tests
   - `6fab03c` (feat): SceneRecord extras with locked copy and Dam Break-only ready
2. **Task 2: Host-lock implementation and notices blob URLs**
   - `6026c68` (test): failing SHA, fallback, and rejected-path URL tests
   - `0cf9345` (feat): `sceneBlobUrl` and `noticesBlobUrl` helpers

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `web/src/catalog/scenes.ts` — descriptions, preview ids, control contracts, credits
- `web/tests/scenes.test.ts` — locked copy, recreates flags, Dam Break-only ready
- `web/src/catalog/links.ts` — host-locked blob URL helpers
- `web/tests/links.test.ts` — SHA gate, main fallback, allowlist rejection

## Decisions Made

- Keep `poke-jelly` on Jelly Drop because UI-SPEC and D-05 include the poke action.
- Reject non-allowlisted scene paths by throwing `Scene source path is not allowlisted`.
- Copy the SHA regex from `readBuildInfo` instead of coupling catalog URLs to Vite env.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-03 Float or Sink spike. Catalog metadata and credit URL helpers are in place. Do not flip later `ready` flags or construct stub worlds from this data. Cards, player controls, and credits chrome remain later plans.

---
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: web/src/catalog/scenes.ts
- FOUND: web/src/catalog/links.ts
- FOUND: web/tests/scenes.test.ts
- FOUND: web/tests/links.test.ts
- FOUND: e6f4895
- FOUND: 6fab03c
- FOUND: 6026c68
- FOUND: 0cf9345

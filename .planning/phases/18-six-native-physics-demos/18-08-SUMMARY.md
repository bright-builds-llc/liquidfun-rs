---
phase: 18-six-native-physics-demos
plan: "08"
subsystem: catalog
tags: [solidjs, catalog-cards, svg, semantic-html, web-01]

requires:
  - phase: 18-six-native-physics-demos
    provides: Locked UI-SPEC descriptions and Dam Break-only ready flags on SCENES
provides:
  - Six static inline SVG catalog cards with Open hash links
  - Token-only ScenePreview art keyed by scene id
  - Empty and unknown fallback copy matching 18-UI-SPEC
affects: [18-09, catalog-cards, player-controls]

tech-stack:
  added: []
  patterns:
    - static inline SVG previews with no ProofSession
    - CSS grid cards on existing dark tokens
    - current-card accent only when the id is ready

key-files:
  created:
    - web/src/catalog/previews.tsx
  modified:
    - web/src/app.css
    - web/src/components/CatalogNav.tsx
    - web/src/components/FallbackPanel.tsx
    - web/tests/scenes.test.ts

key-decisions:
  - "Mark current only when maybeCurrentSceneId matches a ready id so stub cards never get aria-current."
  - "Use an inset 4px accent bar via box-shadow so the 1px card border stays visible."
  - "Keep the not-ready fallback branch as unused defensive copy; do not edit PAGE_SUMMARY."

patterns-established:
  - "Catalog cards render ScenePreview plus Static preview, title, description, and Open; never a WASM thumbnail."
  - "Open hrefs are `#/scene/${scene.id}` from SCENES only."

requirements-completed: [WEB-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:57:45Z

duration: 2min
completed: 2026-09-18
---

# Phase 18 Plan 08: Six Static SVG Catalog Cards Summary

**Six-card catalog grid with token-only inline SVG previews, locked Static preview / Open chrome, and UI-SPEC empty/unknown fallback copy, while five ids stay ready:false.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-18T04:55:22Z
- **Completed:** 2026-09-18T04:57:28Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `ScenePreview` switches on `SceneId` and returns still inline SVG using only `#39D3C7`, `#F87171`, `#CBD5E1`, `#F4F7FA`, and `#071018`. No `ProofSession`, canvas thumbnail, or animation.
- Catalog nav is a `<ul>` of six `<li>` cards in locked order: preview, `Static preview`, title paragraph, description, and `#/scene/{id}` Open. Ready / Not ready yet chips are gone.
- Empty heading/body and unknown heading/body match 18-UI-SPEC. The not-ready fallback branch remains unused defensive code. `PAGE_SUMMARY` and `App.tsx` are unchanged.

## Task Commits

Each task was committed atomically:

1. **Task 1: Author static SVG previews and card CSS** - `f054135` (feat)
2. **Task 2: Render six cards and update fallback/summary copy** - `091ce49` (feat)

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

## Files Created/Modified

- `web/src/catalog/previews.tsx` — static token-only SVG per scene id
- `web/src/app.css` — three/two/one-column card grid, 16px card padding, 16:9 preview, 44px Open, 4px current accent
- `web/src/components/CatalogNav.tsx` — six-card nav with hash-only Open
- `web/src/components/FallbackPanel.tsx` — locked empty/unknown copy
- `web/tests/scenes.test.ts` — stub scenes stay unready so cards remain static

## Decisions Made

- Mark current only via `isReadySceneId` so five stub cards never receive `aria-current="page"` or the accent bar.
- Draw the 4px current accent with an inset box-shadow so the existing 1px `#2A3441` card border remains.
- Leave `PAGE_SUMMARY` and Dam Break-only ready flags for Plan 09, and keep the unused not-ready fallback implementation.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-09 to flip the five `ready` flags and wire construction. Cards, Open hash links, and fallback copy are in place. Do not start WASM from catalog code. Do not restore Ready / Not ready yet chips.

---
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: web/src/catalog/previews.tsx
- FOUND: web/src/components/CatalogNav.tsx
- FOUND: web/src/components/FallbackPanel.tsx
- FOUND: web/src/app.css
- FOUND: web/tests/scenes.test.ts
- FOUND: f054135
- FOUND: 091ce49

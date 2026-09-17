---
phase: 17-shared-player-and-early-pages-delivery
plan: "01"
subsystem: routing
tags: [solidjs, vitest, hash-routing, catalog, web-02]

requires:
  - phase: 16-rust-wasm-browser-bridge
    provides: SolidJS/Vite web package and focused Vitest unit lane
provides:
  - Locked SCENE_IDS and honest six-scene catalog records
  - Pure maybeParseSceneRoute for #/scene/{id}
  - Distinct empty vs unknown hash results
affects: [17-04, 17-05, 17-06, catalog-nav, player-shell]

tech-stack:
  added: []
  patterns: [pure catalog data, allowlisted hash parser, empty-vs-unknown routes]

key-files:
  created:
    - web/src/catalog/scenes.ts
    - web/src/routing/hash.ts
    - web/tests/scenes.test.ts
    - web/tests/hash.test.ts
  modified: []

key-decisions:
  - "Keep #/scene and #/scene/ as empty so later UI can show the empty-hash fallback instead of unknown copy."
  - "Do not lowercase hash tokens; Dam-Break stays unknown with maybeRaw preserved."
  - "Known not-ready ids parse as scene; only dam-break is ready in catalog data."

patterns-established:
  - "Functional-core routing: maybeParseSceneRoute is data-in/data-out and allowlists SCENE_IDS only."
  - "Readiness is catalog metadata, not parser output."

requirements-completed: [WEB-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T11:47:12Z

duration: 2min
completed: 2026-09-17
---

# Phase 17 Plan 01: Allowlisted Hash Scene Routes Summary

**Locked six-scene catalog plus a pure `#/scene/{id}` parser that allowlists ids and keeps empty and unknown hashes as distinct useful states.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-17T11:45:58Z
- **Completed:** 2026-09-17T11:47:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Catalog data now names Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel in locked order.
- Only Dam Break is `ready: true`; later scenes stay honest not-ready records with no Phase 18 card fields.
- `maybeParseSceneRoute` accepts `#/scene/{allowlisted-id}` or the hashless `/scene/{id}` form and never reads `window` or a router library.
- Empty hashes (`""`, `#`, `#/`, `#/scene`, `#/scene/`) stay distinct from unknown tokens so later UI can show matching fallback copy.

## Task Commits

Each task was committed atomically:

1. **Task 1: Lock the six-scene catalog data**
   - `ed5f8fb` (`test`) — failing catalog contract tests
   - `53c69cb` (`feat`) — SCENE_IDS, SCENES, maybeSceneById, isReadySceneId
1. **Task 2: Parse hash routes without a router library**
   - `9d69242` (`test`) — failing empty, unknown, dam-break, and fountain cases
   - `cb88d9e` (`feat`) — allowlisted `maybeParseSceneRoute`

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `web/src/catalog/scenes.ts` — Six locked ids, titles, and ready flags with lookup helpers
- `web/src/routing/hash.ts` — Pure hash parser returning scene, empty, or unknown
- `web/tests/scenes.test.ts` — Catalog order, titles, readiness, and lookup
- `web/tests/hash.test.ts` — Allowlist, empty, unknown, fountain, and full-URL cases

## Decisions Made

- Treat `#/scene` and `#/scene/` as empty identifiers so D-06 empty-hash copy can apply later.
- Never coerce case: `#/scene/Dam-Break` is unknown with `maybeRaw: "Dam-Break"`.
- Fountain and other known not-ready ids remain `{ kind: "scene" }`; readiness stays in catalog data so later code cannot play them from parser output alone.

## Deviations from Plan

None - plan executed exactly as written.

**Total deviations:** 0 auto-fixed
**Impact on plan:** None.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 17-02 (bounded clock) and later player/fallback plans that import `SCENE_IDS`, `SCENES`, and `maybeParseSceneRoute`.
- WEB-02 visitor-facing share/reload and fallback UI remain in Plans 17-04 through 17-06; this plan only shipped the catalog and parser foundation.
- No router package, path-based `/scene/` navigation, or Phase 18 card fields were added.

## Self-Check: PASSED

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

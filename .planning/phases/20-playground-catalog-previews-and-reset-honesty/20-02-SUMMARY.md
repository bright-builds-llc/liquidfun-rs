---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "02"
subsystem: playground-ui
tags: [solidjs, svg, catalog-previews, demo-navigation, kobalte-shell]

requires:
  - phase: 18-six-native-physics-demos
    provides: token-only ScenePreview geometries and locked scene descriptions
  - phase: 19-interaction-polish-and-browser-verification
    provides: PlaygroundShell DemoNavigation list without catalog-card
provides:
  - ScenePreview switch on SceneId with classless token-only SVG
  - compact Static preview caption inside DemoNavigation hash links
  - .demo-nav-preview and .demo-nav-preview-caption CSS
affects: [20-04 sidebar and drawer Playwright, PlaygroundShell dual DemoNavigation mount]

tech-stack:
  added: []
  patterns:
    - compact 72px 16:9 SVG frames inside existing hash links
    - classless inline SVG styled by parent .demo-nav-preview
    - Static preview caption stays a visible text node, never aria-hidden

key-files:
  created:
    - web/src/catalog/previews.tsx
  modified:
    - web/src/components/DemoNavigation.tsx
    - web/src/app.css

key-decisions:
  - "Restore f054135 ScenePreview geometries with a classless SVG so .demo-nav-preview owns the compact frame."
  - "Keep Static preview caption in DemoNavigation, not inside ScenePreview, so the helper stays illustration-only."
  - "Add only .demo-nav-preview* rules; do not restore .catalog-card or the three-column card grid."

patterns-established:
  - "Catalog stills are token-only inline SVG with no WASM, WebP, innerHTML, or element ids."
  - "Sidebar and drawer share DemoNavigation; preview CSS must not depend on a single mount."

requirements-completed: [WEB-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T18:16:20Z

duration: 2min
completed: 2026-09-20
---

# Phase 20 Plan 02: Compact Catalog Previews Summary

**Token-only ScenePreview SVGs packed into existing DemoNavigation hash links with a Static preview caption and compact 72px 16:9 frames.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-20T18:13:44Z
- **Completed:** 2026-09-20T18:15:59Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Restored six still illustrations (Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel) as `ScenePreview` with exhaustive `SceneId` switch, `aria-hidden="true"`, and no `catalog-preview` class.
- Each demo hash link now contains preview, `Static preview`, title, and locked description in that order; no nested Open control.
- Compact `.demo-nav-preview*` CSS uses `max-block-size: 72px`, 16:9, canvas ground `#071018`, and 14px caption; `web/src/app.css` stays at 628 lines.
- `PlaygroundShell.tsx` is unchanged. `web/src` has zero `.catalog-card`, `.catalog-grid`, or `CatalogNav`.
- `cd web && bun run typecheck` exits 0.

## Task Commits

Each task was committed atomically:

1. **Task 1: Restore token-only ScenePreview SVG**
   - `2602ff0` (feat): classless `web/src/catalog/previews.tsx` from `f054135` geometries
2. **Task 2: Pack compact previews into DemoNavigation and app.css**
   - `ceef3ac` (feat): preview + caption in hash links and `.demo-nav-preview*` rules

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `web/src/catalog/previews.tsx` — token-only `ScenePreview`; no WASM, WebP, `innerHTML`, or `id` attributes
- `web/src/components/DemoNavigation.tsx` — preview span and `Static preview` inside existing `<a class="demo-nav-link">`
- `web/src/app.css` — compact `.demo-nav-preview`, `svg`, and `.demo-nav-preview-caption` after `.demo-nav-description`
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/deferred-items.md` — pre-existing xtask file-length note

## Decisions Made

- Restore historical geometries and drop `class="catalog-preview"` so compact demo-nav CSS owns the frame.
- Keep caption copy in DemoNavigation so `ScenePreview` remains illustration-only and both sidebar and drawer mounts stay id-free.
- Pack with 72px 16:9 frames and 14px caption; leave frozen 13px `.demo-nav-description` unchanged.

## Deviations from Plan

None - plan executed exactly as written for in-scope files.

The Task 2 `file-lengths` command still fails on pre-existing `tools/xtask/tests/upstream_cli.rs` (642 lines). That file was already over the cap on HEAD; this plan did not edit it. Logged in `deferred-items.md`. Changed files: `previews.tsx` 120, `DemoNavigation.tsx` 40, `app.css` 628.

## Authentication Gates

None.

## Issues Encountered

None beyond the pre-existing file-lengths finding above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 20-03 keyed SceneControls remount. Compact previews are in both DemoNavigation mounts; 20-04 must scope Playwright to `.demo-sidebar` and the Demos dialog and keep `.catalog-card` count 0. WEB-01 implementation is in place; Chromium proof remains 20-04 and 20-06.

## Self-Check: PASSED

---
*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Completed: 2026-09-20*

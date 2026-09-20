---
phase: 21-playground-leftover-cleanup
plan: "02"
subsystem: web-fallback
tags: [FallbackPanel, unknown-hash, Scene-not-found, Open-Dam-Break, solidjs]

requires:
  - phase: 20-playground-catalog-previews-and-reset-honesty
    provides: empty-hash Dam Break normalizeSceneRoute replaceState
  - phase: 17-shared-player-and-early-pages-delivery
    provides: unknown-hash Scene not found plus Open Dam Break
provides:
  - no-prop unknown-only FallbackPanel
  - Show fallback without fallbackProps
affects: [21-04 player-smoke and independent review]

tech-stack:
  added: []
  patterns:
    - FallbackPanel is unknown-only and takes no props
    - Empty hashes still history-replace to Dam Break and never render Choose a scene

key-files:
  created: []
  modified:
    - web/src/components/FallbackPanel.tsx
    - web/src/App.tsx
    - web/e2e/player.spec.ts

key-decisions:
  - "Collapse FallbackPanel to a no-prop unknown-only component; delete empty and not-ready kinds, copy, and fallbackProps."
  - "Keep empty-hash Dam Break normalizeSceneRoute replaceState; parser kind empty remains for tests and never renders FallbackPanel chrome."
  - "Strengthen the existing unknown-hash Playwright test with locked body copy and absent Choose a scene; do not add a browser matrix."

patterns-established:
  - "Show fallback is fallback={<FallbackPanel />} with no props."
  - "Unknown hash still shows Scene not found, locked body, and hash-only Open Dam Break."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-09-20T19-45-21
generated_at: 2026-09-20T20:19:30.445Z

duration: 2min
completed: 2026-09-20
---

# Phase 21 Plan 02: Unknown-Only FallbackPanel Summary

**Collapsed FallbackPanel to a no-prop unknown-only panel so `#/scene/not-a-scene` still shows Scene not found, locked body copy, and Open Dam Break, while empty hashes keep history-replacing to Dam Break.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-20T20:17:45Z
- **Completed:** 2026-09-20T20:19:30Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Rewrote `FallbackPanel` as a no-prop unknown-only component with heading `Scene not found`, locked body copy, and hash-only `Open Dam Break`.
- Deleted `empty` and `not-ready` kinds, copy, `FallbackKind`, `FallbackPanelProps`, `fallbackCopy`, and App `fallbackProps`.
- Mounted `Show` fallback as `fallback={<FallbackPanel />}` after `normalizeSceneRoute`.
- Strengthened the existing unknown-hash Playwright test to assert the locked body and that `Choose a scene` is absent.

## Task Commits

Each task was committed atomically:

1. **Task 1: Collapse FallbackPanel to unknown-only and delete fallbackProps** - `332860a` (refactor)
2. **Task 2: Strengthen unknown-hash Playwright assertions without a browser matrix** - `e3a52a7` (test)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `web/src/components/FallbackPanel.tsx` — unknown-only no-prop fallback chrome
- `web/src/App.tsx` — no-prop Show fallback; `normalizeSceneRoute` unchanged
- `web/e2e/player.spec.ts` — Scene not found, locked body, absent Choose a scene, Open Dam Break

## Decisions Made

- Collapsed to a no-prop unknown-only component instead of keeping a dedicated unknown union member (D-04/D-05 discretion).
- Left `kind: "empty"` in the parser and `routeIdentity`'s `"empty"` branch; empty hashes still replace to `#/scene/dam-break` (D-06).
- Did not run `just web-player-smoke` in this plan; Plan 21-04 owns that gate (D-10).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 21-03 scene file-length confirmation. Unknown-hash chrome is the only visitor fallback. Empty hashes still become Dam Break. Do not run `just web-player-smoke` until Plan 21-04. Do not grow `app.css`. Do not restore Choose a scene.

## Self-Check: PASSED

---
*Phase: 21-playground-leftover-cleanup*
*Completed: 2026-09-20*

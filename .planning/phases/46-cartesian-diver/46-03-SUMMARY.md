---
phase: 46-cartesian-diver
plan: "03"
subsystem: playground-player
tags: [scene-canvas, long-press, callout, contextmenu, mobile, e2e]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: "2026-10-14T19:04:00Z"
requires:
  - "46-02: cartesian-diver registered in SCENE_IDS and SCENE_HASH_PATHS"
provides:
  - "canvas.scene-canvas declares -webkit-touch-callout, -webkit-user-select and user-select none for every scene"
  - "Direct contextmenu preventDefault listener on the scene canvas (PlayerPanel ref callback)"
  - "web/tests/canvas-callout.test.ts CSS source-text guard"
  - "web/e2e/canvas-callout.spec.ts 390x844 touch-viewport e2e"
affects: [46-04, 46-05]
tech-stack:
  added: []
  patterns:
    - "WebKit-only CSS that Chromium cannot compute is guarded by a source-text unit test plus a built-stylesheet text check in e2e"
key-files:
  created:
    - web/tests/canvas-callout.test.ts
    - web/e2e/canvas-callout.spec.ts
  modified:
    - web/src/styles/player-viewport.css
    - web/src/components/PlayerPanel.tsx
key-decisions:
  - "46-03: D-14 assertion split: Chromium drops -webkit-touch-callout from computed style, so e2e asserts computed user-select/-webkit-user-select/touch-action none and a prevented contextmenu, while a Vitest source-text test and the e2e built-stylesheet fetch guard the callout declaration; a real-iPhone long press stays an optional manual check"
requirements-completed: [DIVER-03]
duration: about 3 min
completed: 2026-10-14
---

# Phase 46 Plan 03: Scene Canvas Long-Press Suppression Summary

**Every scene canvas now declares `-webkit-touch-callout: none`, `-webkit-user-select: none` and `user-select: none`, and a direct `contextmenu` listener calls `preventDefault`. A source-text unit test and a 390x844 touch-viewport Chromium e2e test cover the change.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-14T19:01Z
- **Completed:** 2026-10-14T19:04Z
- **Tasks:** 2 of 2
- **Files:** 2 created, 2 modified

## Accomplishments

- **Task 1 (3292912ec):** Wrote `web/tests/canvas-callout.test.ts` first. The red run showed 1 failed and 1 passed, because the stacking declarations already existed. The task then added the three declarations to the `canvas.scene-canvas` rule after `z-index: 1;`. It also added a module-level `preventCanvasContextMenu` in `PlayerPanel.tsx`, registered in the scene canvas `ref` callback. `touch-action: none` still comes from the shared `canvas` rule. The selector also covers the puzzle canvas, which shares the class, as intended. `web/src/input` and `PuzzlePanel.tsx` are unchanged.
- **Task 2 (135750037):** Added `web/e2e/canvas-callout.spec.ts` with `test.use({ viewport: 390x844, hasTouch, isMobile })`, running in the existing `chromium` project with no config change. Its three tests on `#/scene/cartesian-diver` check:
  - computed `user-select`, `-webkit-user-select` and `touch-action` are all `none`;
  - a dispatched cancelable `contextmenu` returns `false`;
  - a fetched `link[rel=stylesheet]` text matches `/-webkit-touch-callout:\s*none/`.

## Verification

- `bunx vitest run tests/canvas-callout.test.ts`: 2 passed.
- `bun run test:unit`: 1849 passed (120 files).
- `bun run typecheck`, covering both the app and the e2e tsconfig, passed.
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- `just web-build`, then `CI=true bunx playwright test e2e/canvas-callout.spec.ts --project chromium`: 3 passed.
- `e2e/player.spec.ts --project chromium`: 14 passed.
- `web/playwright.config.ts` is unchanged. The change is web-only, so it has no effect on scene fingerprints.

## Decisions Made

- The CSS test checks for `" user-select: none;"` with a leading space, so the `-webkit-user-select` line cannot satisfy it.

## Deviations from Plan

### Documented deviation (D-14 assertion wording)

**1. The D-14 check of computed styles is split between e2e and a source-text test**
- D-14 says the e2e test "asserts the computed styles". Chromium drops `-webkit-touch-callout` from both computed style and the CSSOM (UI-D64, research Pitfall 5), and WebKit is not installed for this Playwright pin.
- In e2e, the computed checks cover `user-select`, `-webkit-user-select` and `touch-action`, which are all `none`, plus a default-prevented `contextmenu`.
- The callout declaration is guarded in two other places. The first is `web/tests/canvas-callout.test.ts`, which checks the CSS source text. The second is the e2e test that fetches the built stylesheet, which confirms the declaration survives the Vite build.
- A long press on a real iPhone is still an optional manual check, not a gate.
- Commits: 3292912ec, 135750037

Otherwise the plan was executed as written, with no auto-fixes. Prettier is not configured in this project and the touched files did not follow its style at HEAD, so formatting with it was not applied.

## Known Stubs

None.

## Threat Flags

None. This change adds only CSS and one event listener scoped to `canvas.scene-canvas`. Page text, links and controls keep their native menus (T-46-08 accepted).

## Self-Check: PASSED

- FOUND: web/tests/canvas-callout.test.ts, web/e2e/canvas-callout.spec.ts
- FOUND: commit 3292912ec (Task 1), commit 135750037 (Task 2)

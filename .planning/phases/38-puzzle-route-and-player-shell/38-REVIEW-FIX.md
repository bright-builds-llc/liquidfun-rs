---
phase: 38-puzzle-route-and-player-shell
fixed_at: 2026-10-12T04:02:35Z
review_path: .planning/phases/38-puzzle-route-and-player-shell/38-REVIEW.md
iteration: 1
fix_scope: critical_warning + IN-02
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 38: Code Review Fix Report

**Fixed at:** 2026-10-12T04:02:35Z
**Source review:** .planning/phases/38-puzzle-route-and-player-shell/38-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01, WR-02, plus the optional one-line IN-02)
- Fixed: 3
- Skipped: 0
- Out of scope, left as is: IN-01, IN-03, IN-04, IN-05. The new WR-02 e2e case partly covers IN-04, because it enters a puzzle from a running scene.

## Fixed Issues

### WR-01: A resize after a mid-run failure silently restarts the level, so the failure is lost

**Files modified:** `web/src/player/puzzle-lifecycle.ts`, `web/tests/puzzle-lifecycle.test.ts`
**Commit:** a3b5e3214
**Status:** fixed: requires human verification (state-handling logic)
**Applied fix:** `startPuzzleUnlessLoading` now returns as soon as `host.puzzle.maybeLevelId() === id`, so the resize path starts a level only before it has been entered. Route changes (`handleHashChange` and `leavePuzzle`, which clears the level id) and Try again own every later rebuild.

Paths that still work:
- A direct page load still starts from the resize observer, because the level id is unset until `beginPuzzleRoute`.
- Scene → puzzle entry starts from `handleHashChange`.
- A resize during a Retry/Edit rebuild is still ignored. The existing test passes.

Canvas remounts do not dispose the session, so they do not depend on the resize start path. New unit test: `keeps the failure view when a resize follows a mid-run failure`. It failed before the fix (the loader was called twice) and passes after it.

### WR-02: Entering a puzzle in-app shows building with an obstacle-free camera, then refits a frame later

**Files modified:** `web/src/player/hud-obstacles.ts`, `web/e2e/puzzle-helpers.ts`, `web/e2e/puzzle.spec.ts`
**Commit:** 6fceb3ae3
**Applied fix:** This is the root-cause fix. `createHudObstacles` now takes its first measurement synchronously when it is created, which happens in `onMount` after the chrome is attached. It no longer waits for the first animation frame, so the panel effect publishes real obstacles before `beginPuzzleRoute` fits the camera.

Scene behaviour and demo capture:
- The initial `ResizeObserver` callback still schedules the one-shot rAF measurement on load. The `settleSyntheticClock` assumption in `web/scripts/demo-media/capture.ts` therefore still holds, and that file is unchanged.
- Scenes now get their obstacles one frame earlier. Later refits are unchanged.

New e2e case: `keeps the camera fixed from the first building frame after in-app entry` at 1280x800 and 390x844. It plays Dam Break, then sets `#/puzzle/first-pour` in-app. A MutationObserver records `data-world-frame` and `data-pixels-per-meter` at the moment `data-puzzle-phase` first becomes `building`. The test asserts that this camera equals the settled building, running and ended cameras.

Before the fix, both viewports failed. At desktop, `159.03,16,1120.97,784` at 77.58 px/m became `168,23.16,1112,776.84` at 76.13 px/m. After the fix, both pass.

### IN-02: Next-level href duplicates `puzzleHash`

**Files modified:** `web/src/puzzle/format.ts`
**Commit:** 96c10546b
**Applied fix:** `maybeHref: puzzleHash(maybeNextId)`, imported from `../catalog/puzzle-levels`. That module imports only `./puzzles`, so there is no import cycle.

## Verification

- `cd web && bun run typecheck`: pass
- `cd web && bun run test:unit`: 74 files and 789 tests pass
- `just web-build`, then `CI=true bun run test:puzzle`: 16 passed (re-run after the final commit)
- `CI=true npx playwright test e2e/hud-safe-frame.spec.ts` (unedited): 64 passed
- `just web-player-smoke`: 45 passed, including `demo-media-clock.spec.ts`
- `bun scripts/bright-builds-check.ts all`: 0 findings
- Protected files were not edited: `web/tests/hash.test.ts`, `web/tests/portrait-bounds.test.ts`, `web/e2e/hud-safe-frame.spec.ts`, `web/tests/readme-svg.test.ts`. Nothing under `crates/` changed.

---

_Fixed: 2026-10-12T04:02:35Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

---
phase: 38-puzzle-route-and-player-shell
reviewed: 2026-10-12T03:55:35Z
depth: standard
files_reviewed: 48
files_reviewed_list:
  - web/e2e/demo-media-clock.spec.ts
  - web/e2e/puzzle-helpers.ts
  - web/e2e/puzzle.spec.ts
  - web/package.json
  - web/scripts/demo-media/capture.ts
  - web/scripts/puzzle-replay/wasm.ts
  - web/src/App.tsx
  - web/src/catalog/puzzle-bounds.ts
  - web/src/catalog/puzzle-levels.ts
  - web/src/components/PlaygroundStage.tsx
  - web/src/components/PuzzleFallbackPanel.tsx
  - web/src/components/PuzzleHud.tsx
  - web/src/components/PuzzleOptionsDrawer.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/PuzzleResultCard.tsx
  - web/src/input/gravity-slider.ts
  - web/src/physics/puzzle-loader.ts
  - web/src/physics/puzzle-session.ts
  - web/src/player/frame-loop.ts
  - web/src/player/playable-target.ts
  - web/src/player/playground-session.ts
  - web/src/player/puzzle-lifecycle.ts
  - web/src/player/puzzle-loop.ts
  - web/src/player/puzzle-runtime.ts
  - web/src/player/runtime.ts
  - web/src/player/scene-lifecycle.ts
  - web/src/player/scene-runtime.ts
  - web/src/player/scene-surface.ts
  - web/src/puzzle/controller.ts
  - web/src/puzzle/fallback-text.ts
  - web/src/puzzle/format.ts
  - web/src/puzzle/placements.ts
  - web/src/puzzle/reduce-puzzle.ts
  - web/src/render/glass-cup.ts
  - web/src/routing/playable-route.ts
  - web/src/styles/puzzle.css
  - web/tests/glass-cup.test.ts
  - web/tests/playable-route.test.ts
  - web/tests/playable-target.test.ts
  - web/tests/puzzle-bounds.test.ts
  - web/tests/puzzle-chrome.test.ts
  - web/tests/puzzle-format.test.ts
  - web/tests/puzzle-lifecycle.test.ts
  - web/tests/puzzle-loop.test.ts
  - web/tests/puzzle-placements.test.ts
  - web/tests/puzzle-session.test.ts
  - web/tests/puzzle-shell-wiring.test.ts
  - web/tests/reduce-puzzle.test.ts
findings:
  critical: 0
  warning: 2
  info: 5
  total: 7
status: issues_found
---

# Phase 38: Code Review Report

**Reviewed:** 2026-10-12T03:55:35Z
**Depth:** standard
**Files Reviewed:** 48
**Status:** issues_found

## Summary

I reviewed the Phase 38 diff against `589cb0fb9`: the puzzle route, the puzzle lifecycle and frame loop, the puzzle HUD and panels, the glass overlay, the playable-route parser, and the changes to the shared shell (`scene-lifecycle.ts`, `frame-loop.ts`, `scene-surface.ts`, `playground-session.ts`, `PlaygroundStage.tsx`).

**Scene path.** I found no change in scene behaviour. Every new branch in the shared shell is gated on a puzzle route or target:

- `drawSceneFrame` draws the glass only when `route().kind === "puzzle"`.
- `syncCanvasInteractive` adds a `route().kind === "scene"` conjunct. Ready scene routes always satisfy it, so scenes are unaffected.
- The resize observer picks bounds through `worldBoundsForTarget`, which passes scenes straight to `worldBoundsForViewport`. It still calls `startScene` for scenes.
- `handleHashChange` only adds a puzzle branch before the existing scene logic, plus a `leavePuzzle` call when the previous route was a puzzle.
- `PlaygroundStage` replaced `Show` with a non-keyed `Switch`/`Match`. The scene branch and its fallback are unchanged.

**Session and rAF teardown.** These are sound on the reachable paths:

- Scene to puzzle: `beginPuzzleRoute` increments the generation, cancels the shared rAF id and disposes the scene or worker session.
- Puzzle to scene: `leavePuzzle` cancels the puzzle rAF. `beginScene` then increments the generation, which makes any in-flight `buildPuzzle` free its session, and disposes the owned puzzle session.
- The puzzle loop's `isCurrent` guard, plus storing its rAF id in `clock.maybeAnimationFrameId`, means `cancelPendingFrame` stops it.
- Panel `ResizeObserver`s and `createHudObstacles` clean up through `onCleanup`.

**Camera.** Fixed HUD boxes (`puzzle.css`) and orientation-only bounds keep the camera fixed across building, running and ended on a direct page load. One in-app entry path still refits the camera once during building (WR-02).

## Warnings

### WR-01: A resize after a mid-run failure silently restarts the level, so the failure is lost

**File:** `web/src/player/puzzle-lifecycle.ts:214-220` (with `web/src/player/frame-loop.ts:295-303` and `web/src/player/scene-lifecycle.ts:77-86`)

**Issue:** `failScene` disposes the session (`maybeSession = undefined`) and shows the failure view. It does not touch the puzzle state machine, so `host.puzzle.current` keeps its phase (`running`, `building` or `ended`). On the next canvas resize, `connectResizeObserver` finds no session and calls `maybeStartPuzzle`. The resize can come from a phone rotation, a mobile URL-bar collapse in the canvas stage, or a window resize. `startPuzzleUnlessLoading` only bails out when the phase is `loading`, so it calls `beginPuzzleRoute`. That clears the failure view and its details, rebuilds the level into `building`, and re-reads the test seam.

Build failures behave differently, because the phase stays `loading` and the resize is ignored. The docstring says "Try again recovers failures", which this path contradicts.

A related problem: the stale phase means `main[data-puzzle-phase]` keeps reporting `running` (for example) while the view is `failure`. `host.puzzle.maybeOwned` also keeps a reference to the disposed session.

**Fix:** Start from the resize path only when the route has not been entered yet. `handleHashChange` and Try again own every other start.

```ts
export function startPuzzleUnlessLoading(host: PuzzleHost, id: PuzzleId): void {
  // Route entry already happened (building, running, ended, failed or loading):
  // only Try again or a route change may rebuild.
  if (host.puzzle.maybeLevelId() === id) {
    return;
  }

  void beginPuzzleRoute(host, id);
}
```

Optionally, also clear `host.puzzle.maybeOwned` when a puzzle session is disposed by `failScene`. Add a lifecycle test that fails a running session, fires the resize callback, and asserts the view is still `failure`.

### WR-02: Entering a puzzle in-app shows building with an obstacle-free camera, then refits a frame later

**File:** `web/src/player/puzzle-lifecycle.ts:190-195` and `:144-153` (with `web/src/components/PuzzlePanel.tsx:121-122` and `web/src/player/hud-obstacles.ts:49,70`)

**Issue:** This happens on in-app navigation to a puzzle (scene to puzzle, fallback to puzzle, or "Open First Pour"). `setRoute` mounts `PuzzlePanel` synchronously. Its `createEffect(on(obstacles, ...))` runs at once with the initial `[]` value, so `changeHudObstacles([])` sets `clock.hudObstacles = []`.

`beginPuzzleRoute` then calls `refreshCamera()` with no obstacles. The wasm module is already initialized, so `loadPuzzleSession` resolves in microtasks and `buildPuzzle` presents the `building` frame with that camera.

`createHudObstacles` takes its first real measurement only in a `requestAnimationFrame` callback. When that measurement lands, `changeHudObstacles` refits the camera while the level is already in `building`. The user sees the world jump one or two frames after the level appears. The D-19 e2e check (`keeps the camera fixed across building, running and ended`) misses this because it loads the puzzle URL directly, where the wasm fetch outlasts the first obstacle measurement.

**Fix:** Make the first obstacle set available synchronously, before the route starts building. For example, measure once when `createHudObstacles` is created. `onMount` runs after the panel is attached, so `getBoundingClientRect` returns real boxes:

```ts
// hud-obstacles.ts, replacing the trailing scheduleMeasure()
observeMarked();
mutationObserver.observe(root, { childList: true, subtree: true });
setObstacles(measureObstacles(root, canvas));
```

Alternatively, have `buildPuzzle` wait for the first non-empty obstacle publication before presenting step 0. Add an e2e case that opens a scene, navigates to `#/puzzle/first-pour`, and compares the camera at the first `building` read with the camera after `stablePuzzleCamera`.

## Info

### IN-01: `leavePuzzle` relies on the scene branch to dispose the session and supersede in-flight builds

**File:** `web/src/player/puzzle-lifecycle.ts:279-287`, `web/src/player/scene-lifecycle.ts:278-296`

**Issue:** `leavePuzzle` does not increment the generation and does not dispose `host.maybeSession`. It assumes `beginScene` or `abandonScene` runs next. The `handleHashChange` scene branch can return after only `setView({ kind: "loading" })` when the canvas or context is missing. In that case a pending `buildPuzzle` would install a puzzle session on a scene route, and the resize observer would then never start the scene because `maybeSession` is set. Today this needs `getContext("2d")` to return `null`, so it is latent.

**Fix:** Make teardown self-contained. In `leavePuzzle`, increment the generation and dispose the session, for example by calling `disposeCurrentSession` inside a `try` that reports through `failScene`. The two dispose helpers (`disposeCurrentSession` rethrows, `disposeOwnedSession` swallows) could also share one implementation, with the error policy documented in one place.

### IN-02: Next-level href duplicates `puzzleHash`

**File:** `web/src/puzzle/format.ts:186`

**Issue:** `` `#/puzzle/${maybeNextId}` `` re-implements `puzzleHash` from `web/src/catalog/puzzle-levels.ts:26-28`. A future route-shape change would have to be made in two places.

**Fix:** `maybeHref: puzzleHash(maybeNextId)`.

### IN-03: `puzzle-chrome.test.ts` asserts source text instead of behaviour

**File:** `web/tests/puzzle-chrome.test.ts:25,73,111-113,187-188`

**Issue:** The tests `readFileSync` component and CSS sources and assert substrings such as `data-hud-obstacle`, `role="timer"` and `width: 100px;`. A harmless refactor (attribute order, a CSS variable, a different file) breaks them, while real regressions in the rendered DOM can pass. This conflicts with the repository testing standard ("Test behavior, not implementation details").

**Fix:** Render the components (Solid `render` in jsdom), or rely on the existing Playwright geometry checks, and assert on the DOM and computed boxes.

### IN-04: E2E does not cover leaving a running puzzle or in-app puzzle entry

**File:** `web/e2e/puzzle.spec.ts:389-403`

**Issue:** `returns to scenes cleanly` only leaves from `building`. Neither of these paths is exercised:

- Leaving while `running`, where an active puzzle rAF and session must be torn down.
- Entering a puzzle from a scene, which exercises the WR-02 path, the scene and worker dispose, and tilt shutdown.

**Fix:** Add one test that presses Play, navigates to a scene mid-run and asserts a single scene loop. For example, `data-step-index` advances at the expected rate and `data-simulation-backend` matches the scene. Add another that navigates from a scene to the puzzle and checks the building camera.

### IN-05: The failure overlay sits below the HUD layer

**File:** `web/src/components/PuzzlePanel.tsx:222-226`, `web/src/styles/puzzle.css:212-217`

**Issue:** The failure overlay uses `.empty-state` (`z-index: 2`), while `.puzzle-hud` is `z-index: 3`. On a canvas-stage phone in portrait, the centred heading, body and "Try again" button can render beneath the full-width result card or the meter and timer band. The HUD Play button also stays visually enabled after a failure in `building`, even though `playPuzzle` ignores the press. This is cosmetic, but the failure state is the one place the user most needs an unobstructed action.

**Fix:** Raise the failure overlay above the HUD while `data-failed="true"`, or hide the HUD leaves during failure. Consider marking Play `aria-disabled` when `status === "failed"`.

---

_Reviewed: 2026-10-12T03:55:35Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

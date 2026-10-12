---
phase: 38-puzzle-route-and-player-shell
plan: "07"
subsystem: web-player-shell
tags: [typescript, solid, puzzle, routing, frame-loop, PUZ-02, PUZ-03, PUZ-05, PUZ-06]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T03:14:13.866Z"
requires:
  - web/src/routing/playable-route.ts PlayableRoute, normalizePlayableRoute (Plan 01)
  - web/src/player/playable-target.ts maybePlayableTarget, samePlayableTarget, titleForPlayableRoute, playableRouteIdentity (Plan 01)
  - web/src/catalog/puzzle-bounds.ts worldBoundsForTarget; web/src/render/glass-cup.ts drawGlassCup (Plan 03)
  - web/src/player/puzzle-runtime.ts createPuzzleRuntime; web/src/player/puzzle-lifecycle.ts beginPuzzleRoute, startPuzzleUnlessLoading, leavePuzzle, createPuzzleController (Plan 05)
  - web/src/components/PuzzlePanel.tsx, PuzzleFallbackPanel.tsx (Plan 06)
provides:
  - SceneRuntime.route as PlayableRoute plus puzzle, assignPuzzleCanvas, beginPuzzleRoute, leavePuzzle, puzzleController
  - handleHashChange puzzle arm (scene arm unchanged)
  - FrameLoopDeps.maybeStartPuzzle (optional) and target-based resize refit
  - glass overlay after presentSceneFrame on puzzle routes
  - PlaygroundStage Switch over scene | puzzle | unknown-puzzle | fallback, with data-puzzle* attributes on main
affects:
  - 38-08 (Chromium e2e drives #/puzzle/first-pour through this wiring and reads data-puzzle-phase/held/outcome)
  - 38-09 (fingerprint and protected-file gate)
tech-stack:
  added: []
  patterns:
    - dispatch on PlayableTarget at the documented points only; scene branches keep the old calls
    - route check before any session.puzzle read, so scene-route fakes without a puzzle bag keep working
    - optional FrameLoopDeps key so typed fakes keep compiling
key-files:
  created:
    - web/tests/puzzle-shell-wiring.test.ts
    - .planning/phases/38-puzzle-route-and-player-shell/deferred-items.md
  modified:
    - web/src/player/scene-runtime.ts
    - web/src/player/runtime.ts
    - web/src/input/gravity-slider.ts
    - web/src/player/playground-session.ts
    - web/src/player/scene-lifecycle.ts
    - web/src/player/frame-loop.ts
    - web/src/player/scene-surface.ts
    - web/src/components/PlaygroundStage.tsx
    - web/src/App.tsx
key-decisions:
  - "The scene PlayerPanel JSX moved under <Match> with two extra spaces of indentation; `git diff -w` shows no change inside it"
  - "data-puzzle-held is \"true\" only when the state is ended and held; it is \"false\" on other puzzle phases and absent on non-puzzle routes"
  - "The demo-media-clock e2e failure (7 tests) is pre-existing at 589cb0fb9 and is logged in deferred-items.md, not fixed"
requirements: [PUZ-02, PUZ-03, PUZ-05, PUZ-06]
duration: 10min
completed: 2026-10-12
---

# Phase 38 Plan 07: Puzzle shell wiring Summary

**This plan connects the puzzle modules to the existing playground shell, changing only the documented dispatch points.**

- The route is widened to `PlayableRoute`, and the scene helper bodies are unchanged.
- Hash changes, the route effect and the resize observer now dispatch on `PlayableTarget`, so a puzzle canvas is no longer wiped (Pitfall 1). A resize during a build or rebuild does not restart the level.
- The glass is drawn after `presentSceneFrame` on puzzle routes only.
- Puzzle canvases get no pointer handler and no crosshair.
- `PlaygroundStage` now switches over scene, puzzle, unknown-puzzle and fallback.
- `#/puzzle/first-pour` loads in Chromium, and scenes behave as before.

## Performance

- **Duration:** about 10 min (03:04Z to 03:14Z)
- **Tasks:** 3 of 3
- **Files:** 9 modified, 1 test created (plus `deferred-items.md`)

## Accomplishments

### Task 1: widen the route type (commit `588ef2caa`)

- `runtime.ts` and `gravity-slider.ts`: only the parameter types and imports changed. The plan's diff filter prints nothing.
- `scene-runtime.ts`:
  - `route` is now `Accessor<PlayableRoute>` and `setRoute` is `Setter<PlayableRoute>`.
  - New fields, each with a doc comment: `puzzle`, `assignPuzzleCanvas`, `beginPuzzleRoute`, `leavePuzzle`, `puzzleController`.
- `playground-session.ts`:
  - `normalizePlayableRoute` replaces the scene normalizer.
  - The initial view comes from `maybePlayableTarget`.
  - The session literal gets `puzzle: createPuzzleRuntime()`.
  - `syncCanvasInteractive` is gated on `route().kind === "scene"`.
  - The route effect now calls `abandonScene()` only when the route has no playable target, and it sets the title with `titleForPlayableRoute`.
- `PlaygroundStage.tsx`: only the route prop type changed in this task.

### Task 2: dispatch and the glass hook (commit `cdaffe3da`)

- `scene-lifecycle.ts`:
  - `frameDeps` gets `maybeStartPuzzle: (id) => startPuzzleUnlessLoading(session, id)`.
  - `handleHashChange` uses `normalizePlayableRoute` and adds a puzzle arm. The arm returns early when the target is the same puzzle, calls `session.beginPuzzleRoute(id)` when a canvas and context exist, and otherwise sets the view to loading.
  - `session.leavePuzzle()` runs before the unchanged scene lines when the previous target was a puzzle.
  - `bindLifecycle` binds `beginPuzzleRoute`, `leavePuzzle` and `puzzleController`.
- `frame-loop.ts`:
  - `FrameLoopDeps.route` is now `() => PlayableRoute`, and there is a new optional key `maybeStartPuzzle?`.
  - The resize observer sets `worldBoundsForTarget(...)`. When no session exists, it calls `startScene(id)` for a scene or `maybeStartPuzzle(id)` for a puzzle.
- `scene-surface.ts`:
  - `drawSceneFrame` checks `route().kind === "puzzle"` before reading `session.puzzle.maybeGlass`, then calls `drawGlassCup(context, camera, maybeGlass)`.
  - The new `assignPuzzleCanvas` detaches any old pointer handler, removes `canvas-interactive` and `canvas-panning`, stores the 2D context (or calls `failScene` when it is missing) and connects the resize observer. It never calls `attachCanvasPointer`.
- `tests/puzzle-shell-wiring.test.ts` has 11 tests, each in Arrange/Act/Assert form:
  - glass drawn after present, checked through call order
  - scene route with a `puzzle` getter that throws: no throw and no glass
  - a held repaint draws the glass
  - `assignPuzzleCanvas`: no pointer attach, the old handler is detached, the crosshair class is removed
  - `assignPuzzleCanvas`: the context is stored and the observer is attached to the canvas
  - resize on a puzzle uses puzzle bounds and calls `maybeStartPuzzle`
  - resize on a scene uses scene bounds and calls `startScene`
  - `maybeStartPuzzle` while a Retry rebuild is loading: no loader call, `current` unchanged
  - hash change from a scene to a puzzle: `beginPuzzleRoute` is called and the canvas is kept
  - hash change from a puzzle to a scene: `leavePuzzle` runs before `loadSceneSession`
  - the same puzzle hash again: nothing happens

### Task 3: mount the panels (commit `f77dc57e2`)

- `PlaygroundStage.tsx`:
  - `<Switch fallback={<FallbackPanel/>}>` replaces the old `<Show>`, with three arms:
    - the scene `<Match>`, holding the unchanged `PlayerPanel` JSX
    - a puzzle `<Match>` that renders `<PuzzlePanel>` with the Plan 06 props, including `assignCanvas={props.assignPuzzleCanvas}`
    - an unknown-puzzle `<Match>` that renders `<PuzzleFallbackPanel>`. It matches on the route object, so an empty raw id still matches.
  - `routeIdentity` now uses `playableRouteIdentity`.
  - `main` gets `data-puzzle`, `data-puzzle-phase`, `data-puzzle-held` and `data-puzzle-outcome`.
- `App.tsx` passes `puzzleController` and `assignPuzzleCanvas`.

## Final line counts (limit 628)

| File | Lines |
| --- | --- |
| scene-surface.ts | 440 |
| scene-lifecycle.ts | 336 |
| frame-loop.ts | 315 |
| PlaygroundStage.tsx | 249 |
| playground-session.ts | 183 |
| scene-runtime.ts | 106 |
| runtime.ts | 94 |
| App.tsx | 52 |
| gravity-slider.ts | 45 |

## Verification

- `bun run typecheck` passed after every task.
- `bun run test:unit`: **74 files, 788 tests passed.** The 5 targeted files (wiring, scene-surface, scene-lifecycle-worker, frame-loop-worker, worker-control-races) ran 32 tests, all passing.
- `bun run build:app` passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- RED was confirmed before the implementation: 8 of 11 tests failed. The `maybeStartPuzzle` test was then made non-vacuous by throwing when the dep is unbound.
- Mutation check, done temporarily and reverted: binding `maybeStartPuzzle` straight to `beginPuzzleRoute` makes the rebuild test fail.
- `just web-player-smoke`: **37 passed, 7 failed.** All 7 failures are in `e2e/demo-media-clock.spec.ts` ("expected 1 pending animation callback, got 2"). They fail the same way with the 38-06 sources and with the phase-base `589cb0fb9` sources, so this plan did not cause them. Every scene test in `player.spec`, `shell.spec`, `reset-honesty.spec` and `animated-svg.spec` passed. The failure is logged in `deferred-items.md`.
- A throwaway Chromium spec, run once against the preview build and then deleted, passed the following checks with no page errors:
  - a direct load of `#/puzzle/first-pour` reaches `data-puzzle-phase="building"`, with `data-simulation-backend="direct"`, no `canvas-interactive` class and the title "First Pour · puzzle · liquidfun-rs playground"
  - after navigating to dam-break, the scene plays and `data-puzzle` is gone
  - after a hash change back to the puzzle, it reaches building (Pitfall 1); pressing Play moves it to running
  - `#/puzzle/nope` shows "Level not found"
- These protected files show no diff against `589cb0fb9`: `PlayerPanel.tsx`, `CanvasStage.tsx`, `hash.ts`, `portrait-bounds.ts`, `puzzles.ts`, `canvas.ts`, `scene-records.ts`, `session.ts`, `loader.ts`, `tests/hash.test.ts`, `tests/portrait-bounds.test.ts`, `e2e/hud-safe-frame.spec.ts` and `tests/readme-svg.test.ts`.

## Deviations from Plan

None in the implementation. The plan was executed as written.

- **Verification gap:** the acceptance line "`just web-player-smoke` exits 0" is not met, because the `demo-media-clock` failure described above already existed. It is recorded in `deferred-items.md` under the scope-boundary rule.
- **Process note:** each task got a single commit, as the plan asks. The RED run for Task 2 was confirmed locally but not committed separately. The web package has no formatter or lint script, so typecheck, Vitest and bright-builds-check were the pre-commit checks that applied. `.planning/config.json` was left unstaged.

## Known Stubs

None.

## Threat Flags

None. T-38-21, T-38-22 and T-38-23 are mitigated and covered by tests:

- T-38-21: the glass is drawn only after the route check, `leavePuzzle` runs before the scene arm, and the browser check confirmed a puzzle-to-scene switch.
- T-38-22: puzzle canvases get no pointer attach, the old handler is detached and the crosshair class is removed.
- T-38-23: the scene branches keep their old calls, the protected files are unchanged, and the scene smoke tests pass.

## Self-Check: PASSED

- FOUND: web/tests/puzzle-shell-wiring.test.ts and all 9 modified shell files
- FOUND: commits 588ef2caa, cdaffe3da, f77dc57e2

---
phase: 38-puzzle-route-and-player-shell
plan: "03"
subsystem: web-render-camera
tags: [typescript, puzzle, camera, canvas, overlay, PUZ-03, PUZ-04, PUZ-05]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T02:46:27.048Z"
requires:
  - web/src/catalog/portrait-bounds.ts worldBoundsForViewport (unchanged)
  - web/src/render/camera.ts createCamera, projectPoint, IDENTITY_CAMERA_VIEW (unchanged)
  - web/src/catalog/puzzles.ts PUZZLE_IDS / PuzzleId (unchanged)
provides:
  - web/src/catalog/puzzle-bounds.ts PUZZLE_VIEW_BOUNDS, PUZZLE_PORTRAIT_BOUNDS, worldBoundsForPuzzle, worldBoundsForTarget
  - web/src/render/glass-cup.ts GlassTick, GlassCup, glassCup, glassHeightAt, targetLineY, drawGlassCup
  - target/phase38/level-probe.txt (wasm cross-check of first-pour geometry)
affects:
  - 38-05/38-07 (frame-loop resize and camera use worldBoundsForTarget for PlayableTarget)
  - drawSceneFrame hook (calls drawGlassCup after presentSceneFrame on puzzle routes)
tech-stack:
  added: []
  patterns:
    - per-PuzzleId orientation-keyed frame registry separate from the scene portrait registry
    - canvas overlay as pure validated geometry plus one draw function wrapped in save/try/finally/restore
key-files:
  created:
    - web/src/catalog/puzzle-bounds.ts
    - web/src/render/glass-cup.ts
    - web/tests/puzzle-bounds.test.ts
    - web/tests/glass-cup.test.ts
  modified: []
key-decisions:
  - "Puzzle frames use the exact worldBoundsForViewport orientation rule (portrait iff height > width and width > 0); first-pour landscape {-6.2,-0.7,6.2,9.2}, portrait {-6.1,-0.6,6.1,9.1}"
  - "Glass tint uses fillRect; the star is a 10-vertex closed path filled in amber; ticks start at the projected interior minX and extend 8 CSS px outward"
  - "glassCup additionally rejects non-finite counts and tick fractions outside 0..1 (fail closed)"
requirements: [PUZ-03, PUZ-04, PUZ-05]
duration: 4min
completed: 2026-10-12
---

# Phase 38 Plan 03: Puzzle bounds registry and glass-cup overlay Summary

**This plan adds a per-`PuzzleId` camera frame registry, kept separate from `PORTRAIT_VIEW_BOUNDS`. Puzzles get one landscape and one portrait frame, chosen by the same rule scenes use, and `worldBoundsForTarget` passes scene targets to `worldBoundsForViewport` unchanged. It also adds a state-safe glass-cup canvas overlay: a tint, glass edges, a highlight, an etched dashed amber target line at 1.08 m, and ticks driven by data, with no text. The first-pour geometry was checked once against the generated wasm.**

## Performance

- **Duration:** about 4 min (02:43Z to 02:47Z)
- **Tasks:** 2 of 2
- **Files:** 4 created, 0 modified

## Accomplishments

### Task 1: puzzle bounds registry (commit `5c557b565`)

- `bun scripts/web-build.ts wasm` rebuilt the package. The probe `target/phase38/probe-level.ts` was not committed because `target/` is gitignored. Its output is saved in `target/phase38/level-probe.txt`:
  ```
  title First Pour
  targetCount 540
  fullCount 900
  stepLimit 720
  gridMeters 0.05000000074505806
  cupInterior [1.600000023841858,0,3.4000000953674316,1.7999999523162842]
  buildArea [-20,44,40,84]
  rigidSegments {"minX":-6,"maxX":6,"minY":-0.5,"maxY":9}
  particles count=1989 {"minX":-2.940000057220459,"maxX":-0.6600018739700317,"minY":4.859999656677246,"maxY":7.859996795654297}
  ```
  Every value matches the plan's `<facts>`, so the test uses the hardcoded values from the plan unchanged.
- First-pour frames:
  - `PUZZLE_VIEW_BOUNDS` is `{minX:-6.2, minY:-0.7, maxX:6.2, maxY:9.2}`.
  - `PUZZLE_PORTRAIT_BOUNDS` is `{minX:-6.1, minY:-0.6, maxX:6.1, maxY:9.1}`.
- `worldBoundsForPuzzle` returns the portrait frame only when `viewportHeight > viewportWidth && viewportWidth > 0`. `worldBoundsForTarget` takes an inline structural union. It does not import from Plan 01.
- Fitted fractions (same fit rule as `portrait-bounds.test.ts`, `IDENTITY_CAMERA_VIEW`, no safe frame):

  | Viewport | Width | Height | px/m |
  | --- | --- | --- | --- |
  | 390x844 (portrait) | 0.9179 | 0.3373 | 29.34 |
  | 1280x800 (landscape) | 0.7515 | 0.9600 | 77.58 |

### Task 2: glass-cup overlay (commit `8b446d1bc`)

- `glassCup(interior, target, full, ticks)` returns `targetFraction = target/full`. It throws `Invalid glass cup` in any of these cases:
  - an edge is not finite
  - `minX >= maxX` or `minY >= maxY`
  - `full <= 0`
  - the target is outside `0..full`
  - a tick fraction is outside `0..1`
- `glassHeightAt` is `minY + fraction * height`. `targetLineY` gives 1.08 m for first-pour.
- `drawGlassCup` calls `save()`, then draws inside `try`, and calls `restore()` in `finally`. It draws these steps in the order of the UI-SPEC table:
  1. tint (`fillRect`)
  2. open-top edges
  3. highlight streak
  4. shadow pass, +1 px
  5. `[6, 4]` dashed `#E6C35C` target line, alpha 0.95
  6. ticks: 2 px, 8 px long, with a 4 / 1.8 px star 6 px beyond the end for `★` labels

  `TAU = 2 * Math.PI` is the only use of `Math.PI`.

## Verification

- `bunx vitest run tests/puzzle-bounds.test.ts tests/portrait-bounds.test.ts`: 12 passed (7 puzzle-bounds tests, at least 5 required; 5 portrait-bounds tests, unedited).
- `bunx vitest run tests/glass-cup.test.ts`: 17 passed (at least 8 required). Coverage:
  - geometry: 2 tests
  - invalid input: 6 tests
  - overlay: 9 tests (save/restore balance and restored state, restore after a throw, dashed target line position, shadow drawn first at +1 px, two ticks with one star, tick y positions, a third ★ tick, no text, tint rectangle)
- `bun run test:unit`: 69 files, 708 tests passed.
- `bun run typecheck`: clean after each task. `bun scripts/bright-builds-check.ts all`: findings=0 after each task.
- `git diff --quiet 589cb0fb9 -- web/src/catalog/portrait-bounds.ts web/tests/portrait-bounds.test.ts web/src/render/canvas.ts`: exit 0. `grep -c first-pour web/src/catalog/portrait-bounds.ts` prints 0.
- RED was confirmed for both tasks: each suite failed on the missing module before the implementation existed.
- Threat mitigations:
  - T-38-08 is covered by the validation tests.
  - T-38-09 is covered by the restored-state tests, including the case where drawing throws.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Fail closed] Extra glass validation**
- **Found during:** Task 2
- **Issue:** The plan validates edges, ordering and counts. A non-finite count, such as `NaN` target, would make every comparison false and pass silently.
- **Fix:** `glassCup` also requires finite counts. Tick fractions are checked to be in 0..1, as the plan's action text says. A test covers a tick at 1.2.
- **Files modified:** web/src/render/glass-cup.ts, web/tests/glass-cup.test.ts
- **Commit:** 8b446d1bc

The plan asks for one commit per task, so neither TDD task has a separate RED commit. RED was confirmed locally. The web package has no formatter or lint script, so the pre-commit checks that applied were typecheck, the unit suites and the bright-builds checks. The SUMMARY uses `generated_by: gsd-execute-phase` as the orchestrator instructed, not `gsd-executor` from the plan output section.

## Known Stubs

None. The overlay is not yet called from `drawSceneFrame`. The plan does not require that, and a later Phase 38 plan wires it after `presentSceneFrame`.

## Self-Check: PASSED

- FOUND: web/src/catalog/puzzle-bounds.ts, web/src/render/glass-cup.ts, web/tests/puzzle-bounds.test.ts, web/tests/glass-cup.test.ts
- FOUND: commits 5c557b565, 8b446d1bc
- FOUND: target/phase38/level-probe.txt (contains `First Pour` and the cup interior values)

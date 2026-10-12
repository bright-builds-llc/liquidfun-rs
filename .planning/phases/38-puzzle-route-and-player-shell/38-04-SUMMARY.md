---
phase: 38-puzzle-route-and-player-shell
plan: "04"
subsystem: web-physics-session
tags: [typescript, puzzle, wasm, session, GOAL-01, GOAL-03]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T02:49:48.613Z"
requires:
  - web/src/physics/session.ts GeneratedProofSession / SceneSession (unchanged)
  - web/src/physics/frame.ts parseRenderFrame / RenderFrame.maybePuzzle (unchanged)
  - web/src/catalog/puzzles.ts PuzzleId (unchanged)
  - generated ProofSession.newPuzzle / levelInfo (Phase 37 bridge)
provides:
  - web/src/physics/puzzle-session.ts RawPuzzleLevelInfo, GeneratedPuzzleSession, PuzzleLevelInfo, PuzzleSession, parsePuzzleLevelInfo, createPuzzleSession, PUZZLE_SCENE_INPUT_MESSAGE
  - web/src/physics/puzzle-loader.ts loadPuzzleSession(id, {kinds, poses})
affects:
  - 38-05/38-07 puzzle lifecycle (stores createPuzzleSession result in session.maybeSession; building uses captureFrame)
  - web/scripts/puzzle-replay/wasm.ts (now re-exports the moved types)
tech-stack:
  added: []
  patterns:
    - puzzle adapter mirrors the scene adapter's exactly-once ownership and 1..4 step bound, plus a step-0 capture without advance
    - scene-input methods fail fast with a fixed message and never touch wasm
    - generated level info parsed once into a meters domain type, freed in finally, fail closed
key-files:
  created:
    - web/src/physics/puzzle-session.ts
    - web/src/physics/puzzle-loader.ts
    - web/tests/puzzle-session.test.ts
  modified:
    - web/scripts/puzzle-replay/wasm.ts
key-decisions:
  - "Puzzle adapter poisoning does not swallow a failing generated free: it rethrows `Rust/WASM session failed` with an AggregateError of the original cause and the free error, instead of copying session.ts's commented `catch {}`"
  - "An out-of-range nextFrame count poisons with a RangeError cause, so the failure explains itself; advance is never called"
  - "No generated-type mismatch: the real ProofSession returned by newPuzzle typechecks as GeneratedPuzzleSession without casts"
requirements: [GOAL-01, GOAL-03]
duration: 3min
completed: 2026-10-12
---

# Phase 38 Plan 04: Direct puzzle session adapter Summary

**This plan adds `createPuzzleSession`, a direct-backend puzzle owner. Its `captureFrame()` returns the untouched step-0 frame without advancing (D-07). Its `nextFrame(n)` keeps the scene adapter's 1..4 step bound. All five scene-input methods throw `Puzzle sessions accept no scene input` without calling wasm (D-16). `levelInfo()` is parsed once into a `PuzzleLevelInfo` in meters, validated fail closed against the expected id, and freed. A separate `loadPuzzleSession` calls `ProofSession.newPuzzle` after the idempotent `init` (D-04). `session.ts` and `loader.ts` are unchanged.**

## Performance

- **Duration:** about 3 min (02:47Z to 02:50Z)
- **Tasks:** 1 of 1
- **Files:** 3 created, 1 modified

## Accomplishments

- `web/src/physics/puzzle-session.ts`: a pure module that does not import the generated package. It holds the moved `RawPuzzleLevelInfo` and `GeneratedPuzzleSession extends GeneratedProofSession`, the `PuzzleLevelInfo` domain type, `parsePuzzleLevelInfo` and `createPuzzleSession`.
  - Validation checks the id match, a non-empty title, and safe positive integer counts with `target <= full`. `cupInterior` needs four finite lanes with `min < max`. `buildArea` needs four safe integer lanes with `min <= max`, and is converted to meters with cells × `gridMeters`. `gridMeters` must be finite and above zero. Each indexed read is guarded for `noUncheckedIndexedAccess`.
  - Invalid info frees the raw info (in `finally`) and the generated session, then rethrows `Invalid puzzle level info`.
- `web/src/physics/puzzle-loader.ts`: `loadPuzzleSession(id, lanes)`. The lanes parameter is the inline structural type that matches Plan 02's `PlacementLanes`.
- `web/scripts/puzzle-replay/wasm.ts`: the local type declarations are replaced with `export type { GeneratedPuzzleSession, RawPuzzleLevelInfo } from "../../src/physics/puzzle-session"`. Its other exports are unchanged.
- I checked the adapter against the real generated wasm with a scratch Bun script, not committed. `levelInfo` came back as First Pour, 540/900, settle 30, limit 720, grid 0.05 (f32), cup `{1.6, 0, 3.4, 1.8}` and build area `{-1, 2.2, 2, 4.2}` m. `captureFrame` gave step 0 `running`, `nextFrame(1)` gave step 1, and `setGravity` threw the scene-input message.

## Task Commits

1. **Task 1: Puzzle session adapter, level-info parser and direct loader** - `9c154e284` (feat). TDD RED was confirmed first: the suite failed because the module was missing. The plan asks for one commit, so RED and GREEN share it.

## Verification

- `bunx vitest run tests/puzzle-session.test.ts tests/session.test.ts tests/puzzle-replay.test.ts tests/scene-lifecycle-worker.test.ts`: 4 files, 70 tests passed. `puzzle-session.test.ts` has 28 tests, above the minimum of 12.
- `bun run typecheck` (`tsc --noEmit`, which covers src, tests and scripts): clean.
- `bun run test:unit`: 70 files, 736 tests passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `(cd web && bun scripts/puzzle-replay.ts replay)`: exit 0. first-pour-solution won at step 201 with 689; first-pour-empty lost at step 720 with 0.
- `git diff --quiet 589cb0fb9 -- web/src/physics/session.ts web/src/physics/loader.ts`: exit 0.
- Acceptance greps: `grep -c newPuzzle web/src/physics/loader.ts` gives 0, and `grep -c "interface RawPuzzleLevelInfo" web/scripts/puzzle-replay/wasm.ts` gives 0.
- `web/` has no format or lint script, so neither ran.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Error handling] Poisoning does not swallow a failing generated free**
- **Found during:** Task 1
- **Issue:** The plan says to mirror `session.ts`'s `disposeAfterFailure`, which uses a commented `catch {}` around `free()`. The plan's error rule ("no `catch {}`") and the global error-handling rule both forbid that.
- **Fix:** `failWith(cause)` marks the session disposed and calls `free()`. If `free()` throws, it raises `Rust/WASM session failed` with `cause: AggregateError([cause, freeError])`. Otherwise it raises the same message with the original cause. The public messages match the scene adapter.
- **Files modified:** web/src/physics/puzzle-session.ts
- **Commit:** 9c154e284

**Generated-type mismatch:** none. `ProofSession.newPuzzle(...)` is assignable to `GeneratedPuzzleSession` with no cast.

## Known Stubs

None.

## Threat Flags

None. T-38-11, T-38-12 and T-38-13 are mitigated as planned and covered by tests: invalid info cases, the scene-input "not called" assertions, and the step-count bounds.

## Self-Check: PASSED

- FOUND: web/src/physics/puzzle-session.ts
- FOUND: web/src/physics/puzzle-loader.ts
- FOUND: web/tests/puzzle-session.test.ts
- FOUND: commit 9c154e284

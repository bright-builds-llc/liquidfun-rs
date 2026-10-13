---
phase: 39-place-then-play-tools-and-stars
plan: "03"
subsystem: web-puzzle-stars-keys-speed-core
tags: [typescript, puzzle, stars, keyboard, fast-forward, functional-core, STAR-01, STAR-02, TOOL-06, TOOL-07]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:14:13Z"
requires:
  - phase: 37-puzzle-core-in-the-bridge
    provides: web/src/catalog/puzzles.ts PUZZLE_IDS and PuzzleId
  - phase: 38-puzzle-route-and-player-shell
    provides: web/src/physics/clock.ts STEP_SECONDS and MAX_STEPS_PER_FRAME
provides:
  - web/src/puzzle/stars.ts RunStars, NO_STARS, LevelStars, RunStarsInput, runStars, mergeLevelStars, StarName, starNames
  - web/src/puzzle/star-store.ts StarStore, StarStoreSnapshot, createStarStore
  - web/src/puzzle/star-format.ts StarCriterion, StarGlyphState, StarThresholds, StarRowModel, starRows, starListLabel
  - web/src/puzzle/keyboard.ts KeyTarget, PuzzleKeyInput, PuzzleKeyCommand, NUDGE_SHIFT_CELLS, puzzleKeyCommand, keyTargetOf
  - web/src/puzzle/speed.ts PuzzleSpeed, PUZZLE_SPEEDS, INITIAL_PUZZLE_SPEED, isPuzzleSpeed, nextPuzzleSpeed, speedLabel, speedStepRequest, restoreUnrunSpeedSteps
affects:
  - later Phase 39 shell plans (runtime owns a StarStore; HUD renders starRows; panel keydown uses puzzleKeyCommand; puzzle loop uses the speed helpers)
  - Phase 41 (serializes StarStoreSnapshot to localStorage unchanged)
tech-stack:
  added: []
  patterns:
    - star store is a per-runtime Map with allowlisted keys; snapshots are fresh null-prototype copies
    - key map reads a plain PuzzleKeyInput record so it stays pure and DOM-free; target classification is a separate guarded helper
key-files:
  created:
    - web/src/puzzle/stars.ts
    - web/src/puzzle/star-store.ts
    - web/src/puzzle/star-format.ts
    - web/src/puzzle/keyboard.ts
    - web/src/puzzle/speed.ts
    - web/tests/puzzle-stars.test.ts
    - web/tests/puzzle-star-format.test.ts
    - web/tests/puzzle-keyboard.test.ts
    - web/tests/puzzle-speed.test.ts
  modified: []
key-decisions:
  - "mergeLevelStars returns the previous best unchanged for a lost run, including bestPeakHeldPct, because peak held is a won-run figure"
  - "mergeLevelStars throws 'Invalid peak held percent' outside 0..100 so bad data never enters the store Phase 41 will persist"
  - "StarStore.get and record return fresh copies, so callers cannot mutate the stored best"
  - "R without Shift (for example with Caps Lock) rotates +1; Shift decides the direction, not the letter case"
  - "keyTargetOf never returns 'control'; that KeyTarget value is for callers that classify HUD focus themselves, and it only differs from 'canvas' for Tab"
  - "speedStepRequest throws 'Invalid step count' for a negative or fractional count"
requirements-completed: []
duration: 3min
completed: 2026-10-13
---

# Phase 39 Plan 03: Stars, Keyboard and Speed Cores Summary

**Five pure TypeScript modules now cover star scoring, a never-lowering best-of store with a null-prototype snapshot, UI-SPEC star row copy, the D-23 key-to-command map and the 1×/2×/4× fast-forward math. Nothing here touches the DOM (apart from a guarded `instanceof` classifier), storage, wasm or frames.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-13T00:11:30Z
- **Completed:** 2026-10-13T00:14:13Z
- **Tasks:** 2 of 2
- **Files created:** 9 (5 source, 4 test)

## Accomplishments

- `stars.ts`: `runStars` returns `NO_STARS` for any loss. A win sets Win, sets Brim from the latch, and sets Thrifty when placements at Play are at most `parTools`. Par and thresholds are inputs, never constants. `mergeLevelStars` ORs each criterion and takes the max peak-held %. A loss leaves the previous best (or undefined) as it was. `starNames` lists latched stars in Win, Brim, Thrifty order, ready for the "Stopped. Stars kept: …" announcement.
- `star-store.ts`: `createStarStore()` is backed by a `Map` and has no module state and no web storage. `record` throws "Unknown puzzle level" for ids outside `PUZZLE_IDS`. `snapshot()` builds an `Object.create(null)` object holding fresh entry copies of recorded levels only, and it survives a JSON round trip.
- `star-format.ts`: `starRows` produces "Win 60%", "Brim 90%" and "Thrifty ≤ 1 tool" (plural for other par values). The hidden text is "Win: fill the cup to 60%." and so on, followed by the state phrase. In building, glyphs come from the best-of ("Earned." / "Not earned yet."). During and after a run they come from the run, then the best, then none ("Earned this run." / "Earned before." / "Not earned."). `starListLabel` returns "Stars" or "Stars this run".
- `keyboard.ts`: `puzzleKeyCommand` maps:
  - arrows to 1-cell nudges, with ArrowUp as +y;
  - Shift+arrows to 10-cell nudges;
  - R / Shift+R to rotate +1 / −1;
  - Delete and Backspace to remove;
  - Space to togglePlay and Escape to escape;
  - Tab / Shift+Tab to cycleSelection, on the canvas only.

  It returns undefined for form controls, Ctrl/Meta/Alt chords, Space or Enter on buttons, and unmapped keys. `keyTargetOf` returns "other" under the node test environment.
- `speed.ts`: 1 → 2 → 4 → 1 cycle; `speedLabel` uses U+00D7; `speedStepRequest(3, 4) = 12`; `restoreUnrunSpeedSteps(0, 12, 8, 4) = STEP_SECONDS`. `MAX_STEPS_PER_FRAME` is still 4.

## Task Commits

1. **Task 1 RED:** `35e7d6b96` (test). Failing tests for stars, the best-of store and star rows.
1. **Task 1 GREEN:** `775075ed4` (feat). Star scoring, the best-of store and star rows.
1. **Task 2 RED:** `485ad4d6d` (test). Failing tests for the keyboard map and the speed helpers.
1. **Task 2 GREEN:** `69bade837` (feat). Keyboard command map and fast-forward speed helpers.

No refactor commits were needed.

## Verification

- `bunx vitest run tests/puzzle-stars.test.ts tests/puzzle-star-format.test.ts`: 39 passed.
- `bunx vitest run tests/puzzle-keyboard.test.ts tests/puzzle-speed.test.ts`: 43 passed.
- Full `bunx vitest run`: 82 files, 997 tests passed.
- `bunx tsc --noEmit`: exit 0.
- `bun scripts/bright-builds-check.ts all`: findings=0, exit 0.
- Acceptance greps:
  - `Object.create(null)` matched in `star-store.ts`, and the `localStorage|sessionStorage|indexedDB` grep printed nothing.
  - `Thrifty ≤` matched in `star-format.ts`.
  - `export type PuzzleSpeed = 1 | 2 | 4`, `NUDGE_SHIFT_CELLS = 10` and `MAX_STEPS_PER_FRAME = 4` (`clock.ts`, unchanged) all matched.
- Both RED runs failed because the modules did not exist yet, as expected.
- The web package has no lint, format or build script, so type checking and Vitest are its applicable checks.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

None. The plan ran as written.

The plan left some choices open, and these were made:

- Input guards that fail fast, added under Rule 2 as small validation:
  - `mergeLevelStars` rejects a peak-held % that is not finite or is outside 0..100.
  - `speedStepRequest` rejects a step count that is negative or not an integer.
- `StarStore.get` and `record` return copies, so mutating a result cannot reach the store.
- `RunStarsInput` is exported as a named type for the `runStars` parameter.

## Requirements

STAR-01, STAR-02, TOOL-06 and TOOL-07 now have pure, tested rules. The star list, the panel key handler, the Speed button and the loop multiplier are not wired into the player yet; later Phase 39 plans do that. These requirements are therefore not marked complete in REQUIREMENTS.md.

## Issues Encountered

None.

## Known Stubs

None. The app does not import these modules yet; later shell plans in this phase wire them.

## Threat Flags

None. Mitigations:

- T-39-09: the store is Map-backed and allowlisted against `PUZZLE_IDS`, and `snapshot()` uses `Object.create(null)` with fresh entries. A test shows that recording `__proto__` throws.
- T-39-10: the key map produces a closed command union with no DOM writes. It ignores form controls, modified keys, and Space or Enter on buttons.
- T-39-11: the merge only ORs and takes the max, and tests show a weaker win or a loss never lowers a star.

## Next Phase Readiness

- The runtime can create one `createStarStore()` per app root. It records at the decision step, or on Stop after a win (D-15), using `runStars({ won, brimLatched, placementsAtPlay, parTools })` with the frame's peak-held %.
- The panel keydown can call `puzzleKeyCommand({ ...event, target: keyTargetOf(event.target) })` and pass "canvas" only when the canvas is focused. It should call `preventDefault` only for handled commands, and only when a Tab cycle has a next tool (UI-D18).
- The puzzle loop requests `speedStepRequest(stepTime.stepCount, speed)` steps and restores debt with `restoreUnrunSpeedSteps(...)` in place of `restoreUnrunSteps`.

## Self-Check: PASSED

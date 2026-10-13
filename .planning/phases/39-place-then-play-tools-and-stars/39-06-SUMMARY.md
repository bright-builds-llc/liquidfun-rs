---
phase: 39-place-then-play-tools-and-stars
plan: "06"
subsystem: web-wasm-boundary
tags: [typescript, wasm, puzzle, level-info, inspector, fail-closed, parity, TOOL-01, TOOL-03, STAR-01, D-01, D-02, D-05, D-06]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:47:13Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: "39-01 inspectPuzzlePlacements export, PlacementStatus codes 0..12, levelInfo brimCount/parTools; 39-02 placement-status.ts and ToolAllowance; 39-05 D-11 frame getters"
provides:
  - web/src/physics/puzzle-session.ts PuzzleLevelInfo.brimCount, parTools, toolBudget, buildAreaCells (GridCellBounds)
  - web/src/physics/puzzle-inspector.ts RawPlacementInspection, RawInspectFn, decodeInspection, inspectPlacements
  - web/src/physics/puzzle-loader.ts buildPuzzleSessionSync, inspectPlacementLanes (loadPuzzleSession kept, init deduplicated)
  - web/tests/placement-status-parity.test.ts Rust/TS status code parity guard
affects:
  - 39-07 and later shell plans (tray from toolBudget, Play gating from inspectPlacements, synchronous run-session build)
  - star plans (brim and par read from levelInfo, never TS constants)
tech-stack:
  added: []
  patterns:
    - wasm boundary decoders take plain typed arrays and never import the generated module, so Vitest loads them without wasm
    - one module-level init flag for the singleton wasm module, with a cached init promise
key-files:
  created:
    - web/src/physics/puzzle-inspector.ts
    - web/tests/puzzle-level-info.test.ts
    - web/tests/puzzle-inspector.test.ts
    - web/tests/placement-status-parity.test.ts
  modified:
    - web/src/physics/puzzle-session.ts
    - web/src/physics/puzzle-loader.ts
    - web/tests/puzzle-session.test.ts
    - web/tests/puzzle-lifecycle.test.ts
key-decisions:
  - "Tool budget lanes must be non-empty and aligned; each kind code maps back through PUZZLE_TOOL_CODES, kinds are unique and counts are safe integers 1..16"
  - "Star thresholds parse as target < brim <= full and 0 <= par <= sum of budget counts; every failure stays on the existing 'Invalid puzzle level info' path"
  - "buildAreaCells keeps the integer grid rectangle; buildArea (meters) is derived from it"
  - "decodeInspection also rejects a negative or fractional count; non-geometric codes decode to undefined even when their floats are non-finite"
  - "loadPuzzleSession caches the init promise (cleared on failure) and flips initialized only after init resolves; the sync entry points throw 'Puzzle wasm is not initialized' before that"
requirements-completed: []
duration: 5min
completed: 2026-10-13
---

# Phase 39 Plan 06: TypeScript Wasm Boundary for Level Info, Inspector and Status Parity Summary

**The wasm package is rebuilt from the Phase 39 bridge. Parsed level info now carries brim, par, a typed tool budget and integer build-area cells, all fail-closed. Inspector lanes decode fail-closed into statuses and quads. Once wasm is initialized, run sessions and inspections are synchronous. A Vitest parity test pins all 13 Rust `PlacementStatus` names and codes.**

## Performance

- **Duration:** about 5 min
- **Started:** about 2026-10-13T00:43Z
- **Completed:** 2026-10-13T00:47:13Z
- **Tasks:** 2 of 2
- **Files:** 8 (4 created, 4 modified)

## Accomplishments

- **Wasm rebuild.** `bun scripts/web-build.ts wasm` succeeded. The generated `liquidfun_wasm.d.ts` now lists `inspectPuzzlePlacements`, `class PlacementInspection`, `PuzzleLevelInfo.brimCount()`/`parTools()` and `ProofFrame.peakHeldCount()`/`brimCount()`/`decidedStep()`/`brimStep()`. The grep count is 12, against a required 5. The build printed the known `support.rs:98` unreachable-code warning, which is a recorded v1.4 leftover. The generated package is gitignored and was not committed.
- **`puzzle-session.ts`:**
  - `RawPuzzleLevelInfo` gains `brimCount()` and `parTools()`.
  - `PuzzleLevelInfo` gains `brimCount`, `parTools`, `toolBudget: readonly ToolAllowance[]` and `buildAreaCells`.
  - New helpers: `parseToolBudget`, `parseToolCount`, `parseStarThresholds`, `parseBuildAreaCells` and `cellsToWorld`.
  - The file is 337 lines, under the 360-line limit.
- **`puzzle-inspector.ts` (new; it does not import the generated module):**
  - `decodeInspection(count, statuses, outlines)` checks that `count` is a non-negative safe integer, that the status lane has `count` entries, that the outline lane has `8 × count` entries, and that every code is known.
  - Geometric codes need 8 finite floats and decode to `{x, y}` points in vertex order. Non-geometric codes (1, 4, 5) decode to `undefined`.
  - `inspectPlacements` encodes lanes with `encodePlacementLanes`, calls the raw inspector and frees the result in `finally`.
- **`puzzle-loader.ts`:**
  - `loadPuzzleSession` initializes once, through a cached promise that is cleared if init fails, and then delegates to `buildPuzzleSessionSync`.
  - `buildPuzzleSessionSync` and `inspectPlacementLanes` wrap `ProofSession.newPuzzle` and the generated `inspectPuzzlePlacements`. Both throw "Puzzle wasm is not initialized" before the first load.
  - A comment explains that the init state is module state because the wasm module is a page-wide singleton.
- **Parity test.** It reads `crates/liquidfun-wasm/src/puzzle/validate.rs`, slices the `enum PlacementStatus { … }` body, regex-matches 13 `Name = code,` pairs and compares them in order with `PLACEMENT_STATUS`, converting keys from camelCase to PascalCase.

## Task Commits

1. **Task 1 RED:** `006fd03b6` (test). Failing tests for brim, par and tool-budget level info.
1. **Task 1 GREEN:** `2ecbfd363` (feat). Parses brim, par, the tool budget and the build-area cells, and updates the fakes.
1. **Task 2 RED:** `b8deb6c42` (test). Failing inspector decode tests plus the status parity test.
1. **Task 2 GREEN:** `deabd90e7` (feat). Inspector decoding and the synchronous loader entry points.

No refactor commits were needed.

## Verification

- `grep -c "inspectPuzzlePlacements\|brimCount\|parTools\|peakHeldCount\|decidedStep" web/src/generated/liquidfun-wasm/liquidfun_wasm.d.ts`: 12.
- `bunx vitest run tests/puzzle-level-info.test.ts tests/puzzle-session.test.ts tests/puzzle-lifecycle.test.ts`: 65 passed.
- `bunx vitest run tests/puzzle-inspector.test.ts tests/placement-status-parity.test.ts`: 20 passed.
- `bun run test:unit`: 85 files, 1059 tests passed.
- `bunx tsc --noEmit`: exit 0.
- `bun run build:app` (vite): built successfully.
- `bun scripts/bright-builds-check.ts all`: `findings=0`.
- Mutation check on the parity test: changing `overlapsNoPlace` to 14 made it fail (1 failed, 1 passed). The file was restored with `git checkout`, and `git diff --quiet` confirmed it.
- RED runs: the level-info test failed 16 of 16 before the parser change. The inspector test failed because the module did not exist yet.
- Acceptance greps matched:
  - `readonly toolBudget: readonly ToolAllowance[]`
  - `export function decodeInspection`
  - `inspectPuzzlePlacements` in `puzzle-loader.ts`
  - `export function buildPuzzleSessionSync`
- `puzzle-lifecycle.test.ts` is 594 lines, within the 628-line limit.
- The web package has no lint or format script.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

**1. [Rule 2 - Correctness] Extra fail-closed cases beyond the plan bullets**
- **Found during:** Task 1 and Task 2
- **Issue:** The behavior bullets did not cover empty tool lanes, a negative `parTools`, a negative or fractional inspection `count`, or a non-geometric outline that holds NaN.
- **Fix:**
  - Empty lanes and a negative par now throw "Invalid puzzle level info".
  - A bad count throws "Invalid placement inspection".
  - A non-geometric status ignores its floats, as the plan describes.
  - Each of these has a test.
- **Commits:** `006fd03b6`, `2ecbfd363`, `b8deb6c42`, `deabd90e7`

**2. [Rule 1 - Robustness] Init deduplication in `loadPuzzleSession`**
- **Issue:** The plan asked for a module-level `initialized` flag. Two loads that start before the first init resolves would each call `init`.
- **Fix:** The init promise is cached and cleared on failure. `initialized` flips only after init succeeds.
- **Commit:** `deabd90e7`

**3. Fakes already reported `toolCounts [2]`.** Plan 01 or 04 had already added `toolCounts: () => new Uint32Array([2])`, so Task 1 added only `brimCount: () => 810` and `parTools: () => 1`.

## Requirements

- This plan delivers the data layer only.
- TOOL-01 (tray counts) and STAR-01 (criteria visible before Play) still need their UI, so they stay Pending in REQUIREMENTS.md.
- TOOL-03 was already Complete from 39-01.
- `requirements-completed` is empty.

## Issues Encountered

None.

## Known Stubs

None. `buildPuzzleSessionSync`, `inspectPlacementLanes` and `inspectPlacements` are not called by the app yet. Later Phase 39 shell plans wire them in.

## Threat Flags

None. T-39-17 through T-39-20 are mitigated as planned:
- **T-39-17:** exact lane lengths, known codes only and finite outlines.
- **T-39-18:** aligned and allowlisted unique kinds, bounded counts, `target < brim <= full` and `0 <= par <= total`.
- **T-39-19:** lanes are always built by `encodePlacementLanes`, and a test checks that a non-integer placement is rejected before the inspector is called.
- **T-39-20:** `free()` runs in `finally`, and tests cover it on both success and decode failure.

## Self-Check: PASSED

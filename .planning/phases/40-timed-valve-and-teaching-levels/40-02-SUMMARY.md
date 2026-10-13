---
phase: 40-timed-valve-and-teaching-levels
plan: "02"
subsystem: web-puzzle-lanes
tags: [typescript, wasm, puzzle, valve, pose-lanes, fixture-grammar, TOOL-05]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T04:38:47.767Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-01 Rust POSE_STRIDE 4, ToolKind::Valve = 2, PlacementStatus 13, 5/6-token fixture grammar"
provides:
  - Bun fixture grammar mirroring Rust parse_placement_line (plank 5 tokens, valve 6 tokens, kind checked before arity)
  - TOOL_CODES { plank 1, valve 2 }; fixtureLanes emits (x, y, angle, openTick)
  - FIXTURE_FILES derived from PUZZLE_IDS (<id>-solution.txt, <id>-empty.txt) plus EXTRA_FIXTURE_FILES (empty)
  - PUZZLE_TOOL_CODES valve 2, PuzzlePlacement.openTick, POSE_STRIDE 4, DEFAULT_VALVE_OPEN_TICK 4, defaultOpenTick(kind)
  - Seam openTick (absent -> 0, non-number -> "Invalid test placements")
  - PLACEMENT_STATUS.valveTimeOutOfRange 13 with reason "Valve time out of range"
  - Editor equality, seeded-cell check and canonical sort include openTick; placeTool uses defaultOpenTick
  - Debug attribute plank:x,y,angle (unchanged) and valve:x,y,angle,openTick
  - ValveIcon and the exhaustive tray case for the valve kind
affects:
  - 40-04 (stepSelectedValveTime goes through updateTool, which now compares openTick)
  - 40-07 (timed-gate early/late fixtures go in EXTRA_FIXTURE_FILES)
  - every later level (registering a PuzzleId adds its fixture pair to just puzzle-replay-wasm)
tech-stack:
  added: []
  patterns:
    - registry-derived fixture list so a new level cannot skip the wasm replay
    - plank tokens keep the Phase 39 three-number form; only valves carry a fourth number
key-files:
  created: []
  modified:
    - web/scripts/puzzle-replay/fixture.ts
    - web/tests/puzzle-replay.test.ts
    - web/src/puzzle/placements.ts
    - web/src/puzzle/placement-status.ts
    - web/src/puzzle/editor.ts
    - web/src/puzzle/debug-attributes.ts
    - web/src/components/PuzzleIcons.tsx
    - web/src/components/PuzzleToolTray.tsx
key-decisions:
  - "Bun parsePlacementLine evaluates openTick before x, y and angle, like Rust, so a line with several bad integers reports the same token first"
  - "A seam entry without openTick reads 0, so the Phase 39 e2e seam literals stay valid unchanged"
  - "The tray label \"Valve\" comes from the existing generic toolKindLabel; hud-format.ts needed no change"
  - "ValveIcon uses the actual 24-unit Icon viewBox (the plan said 20x20): a shorter filled gate bar under a small clock face"
metrics:
  duration: 5min
  completed: 2026-10-13
  tasks: 2
  files: 26
---

# Phase 40 Plan 02: TS stride-4 lanes and valve mirror Summary

The TypeScript side now uses the stride-4 contract from Plan 01. This covers the Bun fixture grammar and lanes (the fixture list now comes from the level registry), the app's lane encoder and test seam, valve code 2, status 13, editor equality and sort, the debug attribute, and a valve tray icon. The generated wasm replays both first-pour fixtures exactly, and the Phase 39 puzzle e2e still passes 29/29.

## Task 1: Bun fixture grammar and registry-derived fixtures (commit ba91a4613)

- `fixture.ts`:
  - `TOOL_CODES = { plank: 1, valve: 2 }` and `FixtureToolKind`; `FixturePlacement` gains `openTick`.
  - `parsePlacementLine` follows the Rust order: a missing kind token is `expected 5 space-separated tokens`, an unknown kind is `unknown tool`, then the per-kind arity `{ plank: 5, valve: 6 }`.
  - `fixtureLanes` emits 4 values per placement.
  - `FIXTURE_FILES` is derived as `PUZZLE_IDS.flatMap(...)` followed by `EXTRA_FIXTURE_FILES`, which is empty.
- `puzzle-replay.ts` did not need any change, because it only iterates over `FIXTURE_FILES`.

### Wasm replay (`target/phase40/replay-wasm-02.log`)

```
REPLAY first-pour-solution.txt pass (outcome won step 201 final_count 689 peak_held 811 brim_step 262)
REPLAY first-pour-empty.txt pass (outcome lost step 720 final_count 0 peak_held 0 brim_step none)
```

### Test literals changed in `web/tests/puzzle-replay.test.ts`

- `parseFixture rejects an unknown level or tool` was split in two. One test expects `level dam-break` to fail with `fixture line 1: unknown level`. The other expects `placement gate 1 2 3` to fail with `fixture line 2: unknown tool`. The old case expected `placement valve 1 2 3` to be rejected; it is now the 6-token rule test.
- The expected parse of the single-plank fixture gained `openTick: 0`.
- In `fixtureLanes flattens placements`, poses changed from `[8, 64, 23, -3, 50, 0]` to `[8, 64, 23, 0, -3, 50, 0, 0]`.
- New tests:
  - A valve line parses with tick 4.
  - A plank line means tick 0.
  - A 6-token plank fails with `expected 5`, and a 5-token valve fails with `expected 6`.
  - A bare `placement` fails with `expected 5`.
  - Plank plus valve lanes give kinds `[1, 2]` and poses `[8, 64, 23, 0, -16, 87, 0, 4]`.
  - `FIXTURE_FILES` matches the derived list.

## Task 2: App lanes, seam, valve code, status 13, editor, attribute and icon (commit ce905edec)

- `placements.ts`:
  - Valve is code 2, `openTick` is required, and `POSE_STRIDE = 4`.
  - `checkPlacement` also requires an i32-safe `openTick`.
  - The seam reads `openTick` through `readSeamOpenTick`.
  - Added `DEFAULT_VALVE_OPEN_TICK = 4` and `defaultOpenTick`.
- `placement-status.ts`: added `valveTimeOutOfRange: 13`, `MAX_STATUS_CODE` is now 13, and 13 maps to "Valve time out of range". `isGeometricStatus` is unchanged, so 13 counts as geometric.
- `editor.ts`:
  - `updateTool` now also compares `openTick` when it decides a change is unchanged.
  - `editorFromPlacements` checks the `openTick` cells.
  - `placeTool` takes the tick from `defaultOpenTick(kind)`.
  - `comparePlacements` breaks ties by `openTick` after `angle`.
  - Rotation and nudges spread the placement, so they keep `openTick`.
- `debug-attributes.ts`: `placementToken` writes `plank:x,y,angle` and `valve:x,y,angle,openTick`.
- `PuzzleIcons.tsx` gains `ValveIcon`, and `PuzzleToolTray.tsx` gains `case "valve"`. The `never` default keeps the switch exhaustive.
- `puzzle-session.ts` and `hud-format.ts` needed no edit. `TOOL_KIND_BY_CODE` is derived from `PUZZLE_TOOL_CODES`, and the label comes from `toolKindLabel`.

### Test literals changed

- 37 plank object literals gained `openTick: 0`, spread across 13 files: `puzzle-debug-attributes` (2), `puzzle-editing` (3), `puzzle-editor-announce` (2), `puzzle-editor-format` (1, multi-line), `puzzle-editor-input` (1), `puzzle-editor` (12), `puzzle-inspector` (2), `puzzle-placements` (3), `puzzle-run` (3), `puzzle-shell-wiring` (1), `puzzle-tools-render` (3), `reduce-puzzle-editor` (2) and `reduce-puzzle` (2). `puzzle-lifecycle.test.ts` also gained `openTick: 0` in its expected seam placement.
- `placement-status-parity.test.ts`: `EXPECTED_STATUS_COUNT` went from 13 to 14, and the test is renamed "fourteen".
- `puzzle-placement-status.test.ts`:
  - The wire-code table gained `["valveTimeOutOfRange", 13]`.
  - The accept list is now `[0, 6, 12, 13]`, and the reject list changed from `13` to `14`.
  - 13 was added to the geometric list, and the reason `[13, "Valve time out of range"]` was added.
- `puzzle-placements.test.ts`:
  - Poses changed from `[8, 64, 23]` to `[8, 64, 23, 0]`, and the i32-ends poses gained a trailing `0`.
  - The unknown kind changed from `"valve"` to `"gate"`.
  - New tests: stride-4 plank plus valve lanes, a fractional or out-of-range `openTick` rejected, `defaultOpenTick` for valve and plank, a valve seam with `openTick: 6`, a seam without `openTick` reading 0, and a string `openTick` seam rejected.
- `puzzle-inspector.test.ts`: the unknown status code changed from 13 to 14 in two places, and the lanes from `[8, 64, 23]` to `[8, 64, 23, 0]`. A new test decodes status 13 with its outline.
- `puzzle-level-info.test.ts`: the unknown kind code changed from 2 to 3. A new test shows that `[1, 2]` parses to `["plank", "valve"]`.
- `puzzle-editing.test.ts`: the inspector poses changed from `[10, 64, 0]` to `[10, 64, 0, 0]`.
- `puzzle-run.test.ts`: in both places, poses changed from `[8, 64, 23, 12, 64, 0]` to `[8, 64, 23, 0, 12, 64, 0, 0]`.
- `puzzle-editor.test.ts` new tests:
  - A placed valve starts at tick 4, and a placed plank at tick 0.
  - Rotating or nudging a valve keeps its tick.
  - A fractional `openTick` seed throws.
  - Canonical order puts planks before valves and breaks ties by tick.
- `puzzle-debug-attributes.test.ts`: new test for `plank:8,64,23;valve:-16,87,0,4`.
- `puzzle-format-decision.test.ts`: new test that a valve budget item is labeled "Valve".

## Verification

- `bunx tsc --noEmit` passed. `bun run test:unit` passed: 94 files, 1317 tests.
- `bun scripts/bright-builds-check.ts all` reported 0 findings.
- `just puzzle-replay-wasm` exited 0, with both first-pour fixtures exact (log above).
- `just web-wasm`, `bun run build:app` and `CI=true bun run test:puzzle` passed 29/29 (`target/phase40/test-puzzle-02.log`).
- No Rust or `crates/` change was made in this plan (`git diff 0f5cf01c4 HEAD -- crates` is empty), so the 25 scene fingerprints cannot move.
- The web package has no formatter or linter script, so typecheck and Vitest are the web checks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Stride-3 expectations in test files outside the plan's list**
- **Found during:** Task 2
- **Issue:** `PuzzlePlacement.openTick` and stride 4 broke typecheck or assertions in 11 test files that the plan did not list: `puzzle-editing`, `puzzle-editor-announce`, `puzzle-editor-format`, `puzzle-editor-input`, `puzzle-inspector`, `puzzle-lifecycle`, `puzzle-run`, `puzzle-shell-wiring`, `puzzle-tools-render`, `reduce-puzzle-editor` and `reduce-puzzle`.
- **Fix:** Added `openTick: 0` to the plank literals and moved the lane expectations to stride 4 (listed above). The inspector's "unknown status" cases moved from 13 to 14.
- **Commit:** ce905edec

**Other choices:**
- `hud-format.ts` and `puzzle-session.ts` were left unchanged because no change was needed (see Task 2).
- Nothing in this plan changes `openTick` after placement, so `updateTool`'s new `openTick` comparison is not exercised yet. Plan 04's `stepSelectedValveTime` goes through `updateTool` and tests that the revision bumps. The valve icon has no DOM render test, because Vitest runs in `node`. The exhaustive `never` switch enforces the case at compile time.

## Known Stubs

None. `EXTRA_FIXTURE_FILES` is empty on purpose; Plan 07 fills it.

## Threat Flags

None. T-40-06 is mitigated: `checkPlacement` requires an i32-safe `openTick`, and the seam rejects a non-number `openTick`. T-40-07 holds: no URL or storage reads were added, and the existing grep test still passes.

## Self-Check: PASSED

---
phase: 41-level-select-and-progress
plan: "04"
subsystem: puzzle-progress
tags: [typescript, progress, persistence, placements, restore, vitest]
requires:
  - "41-01 levelHash on PuzzleLevelInfo"
  - "41-02 restoreChoice, savedPlacementsFor, maybeParsePlacement"
  - "41-03 PuzzleProgress.savePlacements / maybeRestore and the createHost({ maybeStorage }) fixture"
provides:
  - "web/src/player/puzzle-saved-placements.ts: restorePendingPlacements, saveEditorPlacements"
  - "PuzzleRuntime.maybePendingRestoreId and PuzzleRuntime.maybeLastSavedKey"
  - "Restore hook in buildPuzzle; save hooks in commitNow and playPuzzle"
affects:
  - "41 grid page: entering a level from a card restores its last placements"
  - "41-08 gates: storage gate still holds for web/src/player/puzzle-*.ts"
tech-stack:
  added: []
  patterns:
    - "One-shot restore flag armed on route entry, consumed under loading && !pendingRun"
    - "Restore re-dispatches the existing reset event; the reducer is unchanged"
    - "Dedupe key id|levelHash|JSON(tools) prevents rewrites on no-op commits"
key-files:
  created:
    - web/src/player/puzzle-saved-placements.ts
    - web/tests/puzzle-restore.test.ts
    - web/tests/puzzle-save-placements.test.ts
  modified:
    - web/src/player/puzzle-runtime.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/player/puzzle-run.ts
    - web/src/player/puzzle-editing.ts
    - web/tests/puzzle-progress-stars.test.ts
decisions:
  - "41-04: restore runs in buildPuzzle after adoptPuzzleSession and before loaded, once per route entry; the test seam disarms it; Retry, Edit, Stop and Try again never restore"
  - "41-04: placements save in commitNow (building only) and in playPuzzle after its guards; live drag frames never save; dedupe key id|levelHash|JSON(tools) is reset on entry and leave"
  - "41-04: restored tools on first-pour dismiss the first-level hint for good (accepted: a saved tool means the player already placed one, Phase 40 D-20)"
metrics:
  duration: 5min
  completed: 2026-10-13
  tasks: 2
  files: 8
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:10:28.091Z"
---

# Phase 41 Plan 04: Restore and save level placements Summary

Each level now opens with the placements it had last time. They are restored once per route entry, and only when the saved `levelHash` matches the live Rust hash and every entry parses strictly. The player saves placements after every committed edit and at Play. Saves are deduplicated per level, and a storage failure only degrades the progress shell. The pure reducer is unchanged.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | One-shot restore on level entry, before `loaded` (D-18) | 5d389b90d | web/src/player/puzzle-saved-placements.ts, puzzle-runtime.ts, puzzle-lifecycle.ts, puzzle-run.ts, web/tests/puzzle-restore.test.ts |
| 2 | Save after committed edits and at Play, deduplicated per level (D-16) | b47945d72 | web/src/player/puzzle-saved-placements.ts, puzzle-editing.ts, puzzle-run.ts, web/tests/puzzle-save-placements.test.ts, web/tests/puzzle-progress-stars.test.ts |

## Restore point

- `beginPuzzleRoute` reads the test seam once. When there is no seam, it sets `maybePendingRestoreId = id`. When there is a seam, it clears the flag, so the seam wins and the saved record is never touched. Entry also clears `maybeLastSavedKey`.
- `buildPuzzle` calls `restorePendingPlacements(host, id, info.levelHash)` right after `const info = owned.levelInfo;` and before `captureStepZero` and `loaded`. The helper acts only when the flag names this level and the state is `loading` with `pendingRun` false. It consumes the flag, calls `progress.maybeRestore`, and on a match re-dispatches the existing `reset` event with the saved list and the current speed. It publishes nothing, because `buildPuzzle` publishes after `loaded` and the committed inspection marks invalid tools again.
- A mismatched hash, a malformed entry or more than 16 entries clears the saved placements in one write (through `maybeRestore`) and keeps the stars. The level then opens with no tools.
- `leavePuzzle` clears both fields, so entering the level again restores from the latest saved state. `tryAgainPuzzle` and `rebuildFromEnded` (Retry and Edit) never set the flag.
- The preview session is still built with empty lanes, because the `lanesFor(current, run)` call is unchanged.

## Save hooks and dedupe key

- `saveEditorPlacements(host)` returns early when no level id or level info is loaded. Otherwise it builds the key `${id}|${levelHash}|${JSON.stringify(savedPlacementsFor(levelHash, placementsInOrder(editor)).tools)}` and returns if the key is unchanged. If the key changed, it stores the key and calls `progress.savePlacements`.
- `commitNow` in `puzzle-editing.ts` calls the save after `publishState`, only in `building`. This covers place, drag end (`inspectNow`), rotate, nudge, remove, clear and valve time. The `live` branch of `applyPuzzleEdit` never saves. Select, arm and disarm commits leave the key unchanged, so they do not rewrite.
- `playPuzzle` saves after both guards and before `incrementGeneration`. As a result, a restored list that no edit has re-saved is written at Play.
- Placements with an invalid (red) status are saved like any other.

## First-pour hint interplay

`PuzzlePanel.tsx` dismisses the first-level hint for good once building shows at least one tool. If first-pour restores saved tools, the hint is therefore dismissed on entry. This is accepted: a saved tool means the player already placed one (Phase 40 D-20).

## Verification

- `bunx vitest run tests/puzzle-restore.test.ts` passes 13 tests (the plan asks for at least 8). `tests/puzzle-save-placements.test.ts` passes 13 tests (at least 9 required). Both suites failed (RED) against the code before the change: 7 of 13 failed for restore and 12 of 13 for save.
- `tests/puzzle-lifecycle.test.ts`, `tests/puzzle-run.test.ts` and `tests/puzzle-editing.test.ts` still pass.
- `bun run typecheck` is clean. `bun run test:unit` passes 104 files and 1531 tests. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `git diff --quiet -- web/src/puzzle/reduce-puzzle.ts` exits 0.
- `puzzle-saved-placements.ts` contains no `localStorage`, `sessionStorage`, `Date.now`, `performance.now`, `setTimeout` or `setInterval`.
- `grep -c "saveEditorPlacements(host)"` returns 1 for `puzzle-editing.ts` (in `commitNow`) and 1 for `puzzle-run.ts` (in `playPuzzle`). `grep -c "maybePendingRestoreId" web/src/player/puzzle-run.ts` returns 0.
- Playwright e2e was not run in this plan. The puzzle specs use a fresh browser context per test, and none of them asserts an empty editor after a reload without the seam.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] An existing star test expected no progress writes for a lost run**
- **Found during:** Task 2, full `test:unit` run.
- **Issue:** `tests/puzzle-progress-stars.test.ts` "writes nothing to the progress key for a lost run" failed. Play now saves the placements (D-16), so the progress key gets one write.
- **Fix:** The test keeps its intent, that a loss persists no stars. It now asserts exactly one progress-key write (Play's placement save) and that `maybeLevelStars` for first-pour is undefined.
- **Files modified:** web/tests/puzzle-progress-stars.test.ts
- **Commit:** b47945d72

**2. TDD commits.** I confirmed RED for both tasks by stashing the source changes and running the new suites. Each task is still one green commit, because the pre-commit typecheck cannot pass on a RED-only commit. Earlier plans in this phase did the same.

## Known Stubs

None.

## Threat Flags

None. T-41-14: restored lists go only through `progress.maybeRestore` and `restoreChoice` (hash equality, at most 16 entries, `maybeParsePlacement`), and the inspector checks them again on load. T-41-15: the one-shot flag is covered by the Retry, Edit, Try again and re-entry tests. T-41-16: the storage-failure test shows that editing continues with no `failScene` call, and both the dedupe and live-drag tests pass. T-41-17: the seam-precedence test shows the stored value is unchanged byte for byte.

## Self-Check: PASSED

The three created files and five modified files exist. Commits 5d389b90d and b47945d72 are in `git log`.

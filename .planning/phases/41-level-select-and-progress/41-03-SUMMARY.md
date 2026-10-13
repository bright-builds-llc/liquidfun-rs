---
phase: 41-level-select-and-progress
plan: "03"
subsystem: puzzle-progress
tags: [typescript, progress, persistence, localStorage, degraded-mode, solid, vitest]
requires:
  - "41-02 pure progress core (parseProgress, serializeProgress, mergeProgress, withStars, withPlacements, maybeLevelStars, restoreChoice, savedPlacementsFor)"
  - "Phase 39 StarStore and recordStars funnel in puzzle-run.ts"
provides:
  - "web/src/player/puzzle-progress.ts: createPuzzleProgress, PuzzleProgress, ProgressStorage, ProgressStorageProvider, PROGRESS_PROBE_KEY, UNAVAILABLE_PROGRESS_STORAGE"
  - "PuzzleRuntime.progress and a star store hydrated from persisted wins"
  - "PuzzleController.progressDegraded"
  - "web/src/puzzle/stars.ts: sameLevelStars"
  - "web/tests/progress-storage-fixture.ts: createFakeStorage (write log, failSetItem/failRemoveItem/failGetItem)"
  - "web/tests/puzzle-host-fixture.ts: createHost({ maybeStorage })"
affects:
  - "41 grid page and options-sheet note read progress.current() / progressDegraded()"
  - "41 restore/save plan calls progress.maybeRestore and progress.savePlacements"
tech-stack:
  added: []
  patterns:
    - "Injected storage provider; browser provider lives only in playground-session.ts"
    - "Every storage access in its own try; each catch switches to documented degraded mode (D-14)"
    - "Write path: change(local) -> re-read -> change(mergeProgress(stored, local)) -> setItem"
key-files:
  created:
    - web/src/player/puzzle-progress.ts
    - web/tests/progress-storage-fixture.ts
    - web/tests/puzzle-progress-shell.test.ts
    - web/tests/puzzle-progress-stars.test.ts
  modified:
    - web/src/player/puzzle-runtime.ts
    - web/src/player/puzzle-run.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/player/playground-session.ts
    - web/src/puzzle/controller.ts
    - web/src/puzzle/stars.ts
    - web/tests/puzzle-host-fixture.ts
    - web/tests/puzzle-stars.test.ts
decisions:
  - "41-03: progress shell degrades once and drops its storage reference; startup, re-read and setItem failures and any unusable stored value all degrade, and degraded mode never calls setItem on the progress key"
  - "41-03: recordStars folds the progress-merged best back into the StarStore only when it differs (sameLevelStars), so a decision still records the run exactly once when no other tab wrote"
  - "41-03: createPuzzleRuntime() without an argument is degraded (UNAVAILABLE_PROGRESS_STORAGE), so node tests need no window"
metrics:
  duration: 6min
  completed: 2026-10-13
  tasks: 2
  files: 12
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:05:08.102Z"
---

# Phase 41 Plan 03: Progress storage shell and star write-through Summary

This plan adds the imperative progress shell. It takes an injected storage provider, checks that storage works by writing a separate probe key, and switches to degraded mode on any storage failure or unusable stored value. Before every write it reads storage again and merges with `mergeProgress`. The shell is wired into the puzzle runtime. The page-session star store is filled from persisted wins at startup. Each recorded win is written to storage before the ended state is published. The degraded flag is available as `PuzzleController.progressDegraded()`.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | Progress shell with probe, degraded mode and re-read merge writes | 0a26a89a5 | web/src/player/puzzle-progress.ts, web/tests/progress-storage-fixture.ts, web/tests/puzzle-progress-shell.test.ts |
| 2 | Hydrate the star store, write stars through, inject the browser provider, expose `progressDegraded` | 175090d95 | web/src/player/puzzle-runtime.ts, puzzle-run.ts, puzzle-lifecycle.ts, playground-session.ts, web/src/puzzle/controller.ts, web/src/puzzle/stars.ts, web/tests/puzzle-host-fixture.ts, web/tests/puzzle-progress-stars.test.ts |

## Shell API (`web/src/player/puzzle-progress.ts`)

- `PROGRESS_PROBE_KEY = "liquidfun.puzzle-progress.probe"`.
- `ProgressStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">` and `ProgressStorageProvider = () => ProgressStorage | null`. The provider may throw.
- `UNAVAILABLE_PROGRESS_STORAGE = () => null` always gives degraded progress.
- `createPuzzleProgress(provider): PuzzleProgress` must be called inside a Solid root. It provides:
  - `degraded(): boolean` and `current(): ProgressDocument`, both Solid accessors.
  - `recordStars(id, best): LevelStars` returns the merged best. A loss leaves the document unchanged (41-02 `withStars`).
  - `savePlacements(id, levelHash, placements)`.
  - `maybeRestore(id, liveHash)` returns the placements for a matching hash. A mismatched or malformed record is cleared with one write and returns undefined. When nothing was saved, it returns undefined and writes nothing.

## Startup sequence

1. The shell calls `provider()` inside a `try`. A throw, `null` or a non-object result means degraded.
1. It probes with `setItem(PROBE, "1")` and then `removeItem(PROBE)` in one `try`. A throw means degraded. The probe never touches the progress key.
1. It calls `getItem(PROGRESS_STORAGE_KEY)` in its own `try`. A throw means degraded.
1. It runs `parseProgress`. Any result other than `ok` (json, shape or future-version) means degraded with `EMPTY_PROGRESS`, and the stored value is never written.

Write path (`commit(change)`): compute `local = change(current())`. If degraded, set `current` to `local` and stop. Otherwise read storage again. If that read throws or the value is unusable, degrade with no write. Otherwise compute `merged = change(mergeProgress(stored, local))` and call `setItem`. If `setItem` throws, degrade and keep `local`. On success, set `current` to `merged`. Degrading drops the storage reference, so later calls never touch storage.

## Runtime wiring

- `createPuzzleRuntime(progress = createPuzzleProgress(UNAVAILABLE_PROGRESS_STORAGE))` builds the star store, then calls `starStore.record` for each `PUZZLE_IDS` level with a persisted win (`maybeLevelStars`).
- `puzzle-run.ts` `recordStars` merges the run into the `StarStore`. When that gives a best, it calls `host.puzzle.progress.recordStars`. If the returned best is different, because another tab wrote stars, it records that best back into the store. Then it publishes `maybeBestStars`. `recordDecision` and `recordStop` are unchanged. Both still run before publish (D-11).
- `playground-session.ts` `createShell` passes `createPuzzleProgress(() => window.localStorage)`.

## `progressDegraded` implementations updated

- `PuzzleController` interface (`web/src/puzzle/controller.ts`): adds `readonly progressDegraded: Accessor<boolean>`.
- `createPuzzleController` (`web/src/player/puzzle-lifecycle.ts`): adds `progressDegraded: () => host.puzzle.progress.degraded()`. This is the only implementation. No test fake implements `PuzzleController`, and typecheck reported no other site.

## Verification

- `bunx vitest run tests/puzzle-progress-shell.test.ts` passes 20 tests (the plan asked for at least 16).
- `bunx vitest run tests/puzzle-progress-stars.test.ts` passes 8 tests (the plan asked for at least 6).
- `bun run typecheck` is clean. `bun run test:unit` passes 102 files and 1505 tests. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `git grep -l localStorage -- web/src` prints exactly `web/src/components/CanvasStage.tsx web/src/player/first-hint-preference.ts web/src/player/playground-session.ts`. No file under `web/src/puzzle` and no `web/src/player/puzzle-*.ts` file names `localStorage`, `sessionStorage` or `indexedDB`.
- `puzzle-progress.ts` uses no wall-clock API. `grep -c D-14` gives 6, and every `catch` comment documents degraded mode.
- `git diff --quiet -- web/src/player/first-hint-preference.ts` exits 0.
- The ordering test wraps `host.puzzle.setState`. When the first ended state is published, `progress.current()` already holds the win.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Writing a win to storage recorded it twice in the star store**
- **Found during:** Task 2, in `tests/puzzle-run.test.ts` "records the run's stars ... once when a won run is decided". The test expected `record` to be called once and saw 2 calls.
- **Issue:** The plan's snippet always records the merged best back into the store. That gives two `record` calls per decision even when no other tab wrote, which breaks the existing record-once contract.
- **Fix:** Added a pure `sameLevelStars(a, b)` to `web/src/puzzle/stars.ts`, with 2 unit tests in `tests/puzzle-stars.test.ts`. The merged best is recorded back only when it differs from the local best. Stars from another tab are still picked up, and the another-tab test covers this.
- **Files modified:** web/src/puzzle/stars.ts, web/src/player/puzzle-run.ts, web/tests/puzzle-stars.test.ts
- **Commit:** 175090d95

**2. [Test design] The another-tab test runs a full win, not a no-Brim win.** In the fake, a no-Brim win never decides: its `decidedStep` stays undefined until Stop. So the test stores `{win, brim:false, thrifty:false, 95}` from the other tab, and the local run wins with Brim and Thrifty at 90%. The expected union `{win, brim, thrifty, 95}` therefore needs data from both tabs.

**3. TDD commits.** I confirmed RED for Task 1 (the module did not exist) before writing the code. Each task is one commit containing its code and tests, because the pre-commit typecheck cannot pass with a RED-only commit. Earlier plans did the same.

## Known Stubs

None.

## Threat Flags

None. T-41-10: degraded mode returns before any `setItem`, the probe uses a separate key, and byte-identical tests cover the corrupt, shape and future-version seeds. T-41-11: each storage access has its own `try` that degrades. T-41-12: every write re-reads storage and merges, and the another-tab tests cover this in the shell and the runtime.

## Self-Check: PASSED

All four created files and eight modified files exist. Commits 0a26a89a5 and 175090d95 are in `git log`.

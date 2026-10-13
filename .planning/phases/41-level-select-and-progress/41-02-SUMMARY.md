---
phase: 41-level-select-and-progress
plan: "02"
subsystem: puzzle-progress
tags: [typescript, progress, persistence, parsing, unlocks, vitest]
requires:
  - "41-01 levelHash (16 lowercase hex) for SavedPlacements.levelHash"
  - "Phase 39 mergeLevelStars / LevelStars / RunStars"
provides:
  - "web/src/puzzle/progress.ts: PROGRESS_STORAGE_KEY, PROGRESS_VERSION, LEVEL_HASH_PATTERN, EMPTY_PROGRESS, ProgressDocument, LevelProgress, SavedPlacements, ParsedProgress, UnusableReason, parseProgress, serializeProgress, mergeProgress, withStars, withPlacements, maybeLevelStars"
  - "web/src/puzzle/progress-view.ts: MAX_SAVED_PLACEMENTS, isSolved, unlockedLevelIds, solvedCount, LevelCardModel, levelCardModels, RestoreChoice, restoreChoice, savedPlacementsFor"
  - "web/src/puzzle/placements.ts: maybeParsePlacement (strict, non-throwing)"
affects:
  - "41 storage shell plan (provider, probe, degraded mode) consumes parseProgress/serializeProgress/mergeProgress"
  - "41 grid page consumes levelCardModels/solvedCount; restore/save hooks consume restoreChoice/savedPlacementsFor"
tech-stack:
  added: []
  patterns:
    - "Null-prototype records for every untrusted-key map, filled via Object.entries"
    - "Known ids only through a PUZZLE_IDS allowlist set; unknown data kept raw and written back"
    - "Single star-merge helper delegating to mergeLevelStars"
key-files:
  created:
    - web/src/puzzle/progress.ts
    - web/src/puzzle/progress-view.ts
    - web/tests/puzzle-progress.test.ts
    - web/tests/puzzle-progress-view.test.ts
  modified:
    - web/src/puzzle/placements.ts
    - web/tests/puzzle-placements.test.ts
decisions:
  - "41-02: unknown top-level keys are preserved verbatim in `extra`; unknown fields inside a known level record are ignored and not written back"
  - "41-02: a known level record with brim or thrifty but no win is a `shape` failure (every star requires a win, as in mergeLevelStars)"
  - "41-02: placements of a not-won level are stored with zero stars; maybeLevelStars and the card model treat win:false as no best"
  - "41-02: withStars returns the input document unchanged for a loss; mergeProgress keeps stored placements first, then local, and takes unknownLevels/extra from stored"
  - "41-02: serialize order is version, levels (known ids in PUZZLE_IDS order, then unknown ids in stored order), then extra keys"
metrics:
  duration: 6min
  completed: 2026-10-13
  tasks: 2
  files: 6
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T16:59:06.206Z"
---

# Phase 41 Plan 02: Pure progress core Summary

This plan adds the pure progress core. The v1 document (`liquidfun.puzzle-progress.v1`) is parsed fail-closed, with the typed reasons `json`, `shape` and `future-version`, and unknown level ids and top-level keys are written back unchanged. Star updates never lower anything, because they all go through `mergeLevelStars`. The plan also adds the soft linear unlock rule, a six-card grid model built from TS metadata only, and a restore decision that applies saved placements only for the live `levelHash`, with at most 16 entries that each pass a strict parse. None of this code touches storage, the DOM or Solid.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | Progress document core: parse, serialize, merge, copy-on-write updates | 4ec5f6338 | web/src/puzzle/progress.ts, web/tests/puzzle-progress.test.ts |
| 2 | Unlock rule, grid view model, restore decision, strict placement parser | e39cd9ab8 | web/src/puzzle/progress-view.ts, web/src/puzzle/placements.ts, web/tests/puzzle-progress-view.test.ts, web/tests/puzzle-placements.test.ts |

## Exported API

- `progress.ts`
  - Constants: `PROGRESS_STORAGE_KEY`, `PROGRESS_VERSION = 1`, `LEVEL_HASH_PATTERN = /^[0-9a-f]{16}$/` and a frozen `EMPTY_PROGRESS`.
  - `parseProgress(maybeRaw)` returns `{kind:"ok", progress}` or `{kind:"unusable", reason}`. `serializeProgress(progress)` returns the JSON text.
  - `mergeProgress(stored, local)`, `withStars(progress, id, stars)` and `withPlacements(progress, id, maybeSaved)` all return new documents and never change their inputs.
  - `maybeLevelStars(progress, id)` returns undefined until the level's first win, which matches `StarStore.get`.
- `progress-view.ts`
  - `MAX_SAVED_PLACEMENTS = 16`, `isSolved`, `unlockedLevelIds(progress, degraded)` and `solvedCount`.
  - `levelCardModels(progress, degraded)` returns six `LevelCardModel` records: `id`, `number`, `title`, `caption`, `stars`, `maybeBestFillPct` (floored), `locked`, `solved`, `isNextUp`, `footer`, `lockedLabel` and `href`.
  - `restoreChoice(progress, id, liveHash)` returns `none`, `restore` with the placements, or `discard`. `savedPlacementsFor(levelHash, placements)` builds the saved form.
- `placements.ts`: `maybeParsePlacement(entry)` checks the kind against the allowlist and requires safe i32 integers for `x`, `y`, `angle` and `openTick`. It returns a fresh object or undefined. `parseSeamPlacement` and its messages are unchanged.

## Verification

- `bunx vitest run tests/puzzle-progress.test.ts`: 45 passed. The plan's minimum was 25.
- `bunx vitest run tests/puzzle-progress-view.test.ts`: 24 passed. The plan's minimum was 15.
- `bunx vitest run tests/puzzle-placements.test.ts`: 44 passed, including the 12 new `maybeParsePlacement` tests. The storage and URL scan test still passes.
- `bun run typecheck` is clean. `bun run test:unit` passes 100 files with 1475 tests. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- No storage, window or location identifiers appear in `progress.ts`, `progress-view.ts` or `placements.ts`.
- `git diff` of `placements.ts` removes no line that contains "Invalid test placements" or "Invalid puzzle placement".
- `mergeLevelStars` is referenced only in `mergeStarFields`. The parse-time `(brim || thrifty) && !win` check only validates; it does not combine stars.
- File lengths: progress.ts 299, progress-view.ts 139, placements.ts 174 and puzzle-progress.test.ts 403. All are under 628.
- A test shows that a JSON `"__proto__"` level id leaves `Object.prototype` clean and is written back as an own key.

## Deviations from Plan

1. **[Acceptance proxy] `grep -c "Object.create(null)"` is 2, not 3 or more.** Every keyed record is null-prototype: parsed `levels`, `unknownLevels` and `extra`, the serialized `levels` and the top-level output object. All of them are built through the two helpers `emptyLevels()` and `emptyRaw()`, so the call appears only twice. I kept the helpers rather than repeat the call to reach the count. The intent behind the check is met.
1. **TDD commits.** I confirmed RED for both tasks: the missing module, and `maybeParsePlacement is not a function` in 12 failing tests. The failing tests were not committed on their own, though. The project rules require typecheck to pass before every commit, and a RED commit cannot pass it. Each task is one green commit with its tests, as in earlier plans.
1. **`withStars` on a loss returns the input document itself.** No absent-level record is created and nothing is copied. This satisfies "leaves it unchanged" and keeps zero-star records from appearing without placements.

## Known Stubs

None.

## Threat Flags

None. The plan's threat register covers every boundary added here. T-41-05: typed unusable reasons and a strict star schema. T-41-06: null-prototype records, `Object.entries` and the allowlist set, with a `__proto__` test. T-41-07: the 16-entry cap and per-entry `maybeParsePlacement`. T-41-08: models carry only constants and numbers. T-41-09: unknown ids and keys are written back unchanged.

## Self-Check: PASSED

All four created source and test files, the two modified files and commits 4ec5f6338 and e39cd9ab8 exist.

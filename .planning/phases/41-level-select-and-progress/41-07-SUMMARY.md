---
phase: 41-level-select-and-progress
plan: "07"
subsystem: puzzle-select
tags: [playwright, e2e, level-select, progress, localStorage]
requires:
  - "41-03 PuzzleProgress shell (degraded mode, save/restore placements)"
  - "41-05 #/puzzles grid, sidebar Puzzles link and data-level-id hooks"
  - "41-06 All levels links in the options sheet and the last-level result card"
provides:
  - "web/e2e/puzzle-progress.spec.ts: D-20 browser coverage at 1280x800 and 390x844 (24 tests)"
  - "test:puzzle now includes e2e/puzzle-progress.spec.ts"
affects:
  - "41-08 phase close-out gates run the extended test:puzzle"
tech-stack:
  added: []
  patterns:
    - "Seam runs stay on their own page; seam-free checks open page.context().newPage() so storage is shared but init scripts are not"
    - "Seeds go through setItem then reload, so the app reopens storage instead of using its in-memory document"
key-files:
  created:
    - web/e2e/puzzle-progress.spec.ts
  modified:
    - web/package.json
decisions:
  - "41-07: progress e2e reads the live first-pour hash from level-hashes.txt via parseLevelHashes, and a seeded correct-hash record must restore plank:8,64,23 so the wrong-hash discard check is not vacuous"
  - "41-07: runs use a synthetic 15 Hz clock at 4x speed like puzzle-levels.spec.ts; no wall-clock waits"
metrics:
  duration: 4min
  completed: 2026-10-13
  tasks: 2
  files: 2
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:29:29.831Z"
---

# Phase 41 Plan 07: Level select and progress e2e Summary

A new Playwright spec, `web/e2e/puzzle-progress.spec.ts`, covers each D-20 scenario at 1280×800 and 390×844. It tests the fresh grid, the sidebar link, deep links to locked levels, the options sheet's `All levels` link, blocked, future-version and corrupt storage, persisted stars, the never-lowering merge, placement restore, wrong-hash discard and the last level's `All levels` action. The spec is now part of `bun run test:puzzle`, and the full puzzle and player suites pass.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | Grid, navigation, deep link and storage-failure scenarios | 198cfd3a2 | web/e2e/puzzle-progress.spec.ts |
| 2 | Persistence, restore and last-level scenarios; spec added to `test:puzzle` | 6b6342694 | web/e2e/puzzle-progress.spec.ts, web/package.json |

## Scenarios and pass counts

Each scenario below runs once at each viewport (1280×800 and 390×844), so the spec has 24 tests and all 24 pass (`target/phase41/e2e-07-task2.log`).

| Scenario | Proves | Passed |
| --- | --- | --- |
| shows six cards with level 1 up next on fresh storage | PUZ-01, D-05..D-07: cards in registry order with titles and captions; L1 is an `a` and Up next; L2–L6 are `role="group"` with "Solve {prev} to unlock"; all 18 stars "not yet"; "0 of 6 solved"; no note; no horizontal scroll; 3 columns on desktop, 1 on phone; last card and the GitHub source link reachable by scrolling | 2/2 |
| opens the grid from the sidebar Puzzles link | D-03: Demos dialog, then Puzzles, opens `#/puzzles` with `aria-current="page"`; on a level route the link has `aria-current="true"` | 2/2 |
| opens a locked level from a deep link | PROG-02: `#/puzzle/timed-gate` opens in building with no redirect and no "locked" text; the grid still shows it locked | 2/2 |
| leaves the drawer with no overlay when All levels is used | D-04: no dialog remains, the grid scrolls to its last card and card 1 is clickable (the deferred 41-06 check) | 2/2 |
| unlocks every level and stays playable with blocked storage | PROG-03: `data-progress-degraded="true"`, all cards unlocked, the note on the grid and in the options sheet, a seam run is won and shows its result card, the session's stars appear on the grid | 2/2 |
| keeps a future-version record byte-for-byte | PROG-03, T-41-25: the note shows, a won run is played, and the stored value still `toBe(FUTURE_RAW)` | 2/2 |
| keeps a corrupt record byte-for-byte | PROG-03, T-41-25: `"{not json"` is unchanged after opening first-pour | 2/2 |
| persists stars and unlocks level 2 after a win | PROG-01/02, D-11: the grid updates live on the same page; a second page without the seam shows Win, Brim and Thrifty earned, "Best fill 90%", L2 unlocked and "1 of 6 solved" | 2/2 |
| never lowers saved stars after a worse run | PROG-01, D-09: the second page restores `plank:8,64,23`, then Clear and an empty run lose; stored star fields stay `{win, brim, thrifty: true, bestPeakHeldPct: 90}`, and after a reload all three stars still show | 2/2 |
| restores placements after a reload | PROG-04: a tray-then-canvas `plank:8,64,0` survives a reload; after a seam page plays, a third page restores `plank:8,64,23` | 2/2 |
| discards placements saved for a changed level | PROG-04, T-41-26: a record under hash `0000000000000000` opens with no tools, loses `placements` and keeps `win: true, bestPeakHeldPct: 61`; the same record under the live pinned hash restores `plank:8,64,23` | 2/2 |
| sends Next on the last level to the grid | D-04, UI-D25: after grand-finale ends, `All levels` has href `#/puzzles`, is accented and focused, and clicking it lands on the grid | 2/2 |

## Verification

- `bun run typecheck` is clean, and `bun run test:unit` passes 104 files and 1541 tests.
- The final runs used a `VITE_BUILT_AT=<now> bun run build:app` build:
  - `CI=true bun run test:puzzle` passed 81 tests (`target/phase41/test-puzzle-07.log`).
  - `CI=true bun run test:player` passed 45 tests (`target/phase41/test-player-07.log`).
- `git diff --quiet <phase-base> -- e2e/hud-safe-frame.spec.ts` and `git diff --quiet 792c2f644 -- e2e/shell.spec.ts` both exit 0.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `tsc` does not cover `web/e2e` in this repository. A one-off check with a temporary tsconfig found no errors in the spec other than the missing `node:` type definitions, which the existing e2e helpers also lack.

## Screenshots (gitignored)

- `target/phase41/screens/grid-fresh-1280x800.png`
- `target/phase41/screens/grid-fresh-390x844.png`
- `target/phase41/screens/last-level-ended-1280x800.png`
- `target/phase41/screens/last-level-ended-390x844.png` (the portrait screenshot deferred from 41-06)

## Existing assertions adapted for PROG-04

None. Every existing puzzle and player spec passed unchanged.

## Deviations from Plan

None. The plan was executed as written. One addition: the restore scenario uses a third page so that the reload check (no seam) and the Play-saves check (seam page, then a seam-free page) each start from a clean state.

## Known Stubs

None.

## Self-Check: PASSED

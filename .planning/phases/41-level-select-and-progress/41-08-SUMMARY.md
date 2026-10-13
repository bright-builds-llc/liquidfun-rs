---
phase: 41-level-select-and-progress
plan: "08"
subsystem: phase-gate
tags: [gate, fingerprints, wasm, playwright, vitest, progress, level-select]
requires:
  - "41-01..41-07 (level hash, progress core and shell, restore/save, grid, All levels links, progress e2e)"
  - "target/phase41/base.txt and target/phase41/phase-base.txt (41-01 baseline)"
provides:
  - "target/phase41/checks-08.{sh,results,log}: 27 D-21 gates, all exit 0 on attempt 1"
  - "target/phase41/after-final.txt: 25 post-phase fingerprints, identical to base.txt"
  - "D-01..D-21 decision-to-evidence map for the verifier"
affects:
  - "Phase 41 verification and completion (PUZ-01, PROG-01..PROG-04)"
tech-stack:
  added: []
  patterns:
    - "Gate script runs every gate in a `set -euo pipefail` subshell, records `<name> exit=<code> <UTC>`, and uses no_match instead of `! grep` mid-gate"
key-files:
  created:
    - target/phase41/checks-08.sh (gitignored)
    - target/phase41/checks-08.results (gitignored)
    - target/phase41/checks-08.log (gitignored)
    - target/phase41/after-final.txt (gitignored)
    - target/phase41/level-hash-08.log (gitignored)
    - target/phase41/replay-08.log (gitignored)
  modified: []
decisions:
  - "41-08: D-21 holds on 5e10940e9; 27/27 gates exit 0 on the first run, scene-spot 2026-10-13T17-31-31Z 25/25 fingerprints identical to base, engine untouched, six level-hash pins equal native and wasm, 14/14 wasm fixtures pass, puzzle e2e 81/81; no fix commit"
  - "41-08: shell.spec.ts is checked against 792c2f644 (the sanctioned 41-05 focus-trap count edit), all other protected tests against the phase base f277ecad9"
metrics:
  duration: 9min
  completed: 2026-10-13
  tasks: 2
  files: 0
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T17:39:21.068Z"
---

# Phase 41 Plan 08: D-21 final gates and decision record Summary

Every D-21 done criterion passes on the final Phase 41 tree (`5e10940e9`). All 27 gates exited 0 on the first run. The 25 scene fingerprints are identical to the phase base, `crates/liquidfun` is unchanged, and the six pinned level hashes match in native Rust and in wasm. The unit, player smoke, HUD safe-frame and puzzle Playwright suites (including `puzzle-progress.spec.ts`) all pass. No fix was needed, so this plan made no code commit and no failed attempt files exist.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | Run and record every D-21 gate | none (all gates passed; evidence in gitignored `target/`) | target/phase41/checks-08.{sh,results,log}, after-final.txt |
| 2 | Phase decision record for the verifier | none (this SUMMARY) | .planning/phases/41-level-select-and-progress/41-08-SUMMARY.md |

## Gate results (`target/phase41/checks-08.results`, attempt 1)

Phase base `f277ecad9f85ded5cf71b8a44966b893cf9efd0d`, HEAD `5e10940e91e2a73a57c28d06f3aa9d85308ff15c`. Scene-spot stamp: `target/dam-break-perf/2026-10-13T17-31-31Z/`. Result: **25/25 identical** (`diff target/phase41/base.txt target/phase41/after-final.txt` is empty, 25 lines).

| # | Gate | Result | Evidence |
| - | ---- | ------ | -------- |
| 1 | fingerprints | exit=0 17:31:31Z | 25/25 identical to base.txt |
| 2 | engine-untouched | exit=0 | `git diff --stat $BASE -- crates/liquidfun` empty |
| 3 | puzzle-fixtures-unchanged | exit=0 | `crates/liquidfun-wasm/src/puzzle/fixtures` unchanged |
| 4 | scenes-untouched | exit=0 | scenes.ts, scene-records.ts, portrait-bounds.ts, readme-svg plans, DemoNavigation, CanvasStage, canvas-stage.css, PlayerPanel, hash.ts, scene.rs, scene/ unchanged |
| 5 | protected-tests-unedited | exit=0 | hash, portrait-bounds, readme-svg tests and hud-safe-frame spec unchanged since base; shell.spec.ts unchanged since 792c2f644 |
| 6 | first-hint-unchanged | exit=0 | first-hint-preference.ts unchanged |
| 7 | cargo-fmt | exit=0 | |
| 8 | cargo-clippy | exit=0 | `-D warnings` |
| 9 | cargo-clippy-wasm | exit=0 | `-p liquidfun-wasm`, `-D warnings` |
| 10 | cargo-build-wasm | exit=0 | all targets, all features |
| 11 | wasm-crate-tests | exit=0 17:37:28Z | 574 passed, 24 ignored |
| 12 | level-hash-pin | exit=0 | `puzzle::level_hash`: 9 passed, 1 ignored (printer) |
| 13 | puzzle-replay-wasm | exit=0 | 6/6 LEVELHASH pass, 14/14 REPLAY pass |
| 14 | typecheck | exit=0 | |
| 15 | vitest | exit=0 | 104 files, 1541 tests |
| 16 | bright-builds | exit=0 | findings=0 |
| 17 | player-smoke | exit=0 | 45 passed |
| 18 | hud-safe-frame | exit=0 | 64 passed |
| 19 | puzzle-e2e | exit=0 | 81 passed (test:puzzle includes puzzle-progress.spec.ts) |
| 20 | markdown | exit=0 | no non-GSD Markdown changed since base |
| 21 | grep-no-wall-clock | exit=0 | |
| 22 | grep-storage-allowlist | exit=0 | localStorage only in CanvasStage.tsx, first-hint-preference.ts, playground-session.ts; seam only in puzzle/placements.ts |
| 23 | grep-no-innerhtml | exit=0 | |
| 24 | grep-no-hud-obstacle-on-grid | exit=0 | PuzzleSelectPanel, PuzzleLevelLinks, PuzzleOptionsDrawer |
| 25 | grep-levels-out-of-scenes | exit=0 | six level ids absent from scene catalogs, README plans, DemoNavigation, scene.rs, scene_spot.rs |
| 26 | grep-progress-key | exit=0 | progress, probe and first-hint keys as shipped |
| 27 | grep-no-next-level-yet | exit=0 | nothing left in web/src, web/tests, web/e2e |

## Pinned level hashes (native test == wasm replay, `target/phase41/replay-08.log`)

```
first-pour f0d2fdf9d2efb050
thick-syrup 1acd4f5d5af07d9c
jelly-pad d48dd7deeeb39443
float-up a94c5fd51fb29304
timed-gate bebe1822f5fb7ea9
grand-finale 88c192beb348641e
```

Encoding: FNV-1a 64 over `level-hash-v1|{LevelSpec:?}|grid={GRID_METERS:?}|tick={STEPS_PER_OPEN_TICK}` (Debug encoding with the `level-hash-v1` prefix), 16 lowercase hex digits, pinned in `crates/liquidfun-wasm/src/puzzle/level-hashes.txt`.

## Progress storage as shipped

- Document key: `liquidfun.puzzle-progress.v1` (`PROGRESS_STORAGE_KEY`, `web/src/puzzle/progress.ts`).
- Probe key: `liquidfun.puzzle-progress.probe` (`PROGRESS_PROBE_KEY`, `web/src/player/puzzle-progress.ts`). The probe never touches the document key.
- First-hint key unchanged: `liquidfun.puzzle-first-hint.v1` (not migrated).
- v1 schema: `{ "version": 1, "levels": { "<levelId>": { "win": bool, "brim": bool, "thrifty": bool, "bestPeakHeldPct": 0..100, "placements"?: { "levelHash": /^[0-9a-f]{16}$/, "tools": [{kind, x, y, angle, openTick}] } } } }`. A known record with `brim` or `thrifty` but no `win` is a shape failure. Placements of a not-won level are stored with zero stars. Serialize order: `version`, `levels` (known ids in `PUZZLE_IDS` order, then unknown ids in stored order), then extra top-level keys.

## Discretion choices

- Modules: `web/src/puzzle/progress.ts` (pure document core), `web/src/puzzle/progress-view.ts` (unlock rule, card models, restore choice), `web/src/player/puzzle-progress.ts` (storage shell), `web/src/player/puzzle-saved-placements.ts` (restore/save hooks), components `PuzzleSelectPanel` (grid) and `PuzzleLevelLinks` (All levels link and note).
- Level hash: Debug encoding with the `level-hash-v1` prefix (D-17 preferred form).
- Grid shows "Best fill N%" (floored `bestPeakHeldPct`) on cards and "{N} of 6 solved" under the intro.
- Intro: "Get enough water into the cup before time runs out." Note: "Progress can't be saved in this browser, so every level is unlocked."
- Unknown top-level keys and unknown level ids are preserved verbatim; unknown fields inside known level records are dropped on write.
- Mismatched or malformed saved placements are discarded immediately on restore with one write (stars kept), not lazily on the next write.
- The last level's result action is an enabled accented `All levels` link with `primary: "next"`, after a win or a loss.
- Route: `#/puzzles` and `#/puzzles/` parse to the grid; `#/puzzles/<x>` stays an unknown scene. Title `Puzzles · liquidfun-rs playground`, identity `puzzles`.

## First-pour hint interplay with restored tools

`PuzzlePanel.tsx` dismisses the first-level hint for good once building shows at least one tool. When first-pour restores saved tools, the hint is dismissed on entry. Accepted in 41-04: a saved tool means the player already placed one (Phase 40 D-20). The test seam takes precedence over restore and leaves the stored record untouched.

## Existing test literals changed in Phase 41 (and why)

- `web/tests/playable-route.test.ts` (41-05): "leaves the plural puzzles hash as an unknown scene route" became "parses the plural puzzles hash as the level grid" (D-01).
- `web/tests/playable-target.test.ts` (41-05): the route-kind identity test includes `{kind:"puzzles"}` → `"puzzles"` (D-02).
- `web/tests/puzzle-chrome.test.ts` (41-05, 41-06): `HUD_STYLESHEETS` split from `PUZZLE_STYLESHEETS` so the `data-puzzle-layout` scope test runs only on HUD sheets; grid sheet and new components added to the scans (additions only otherwise).
- `web/e2e/puzzle.spec.ts` (41-05, 41-06): `#/puzzles` now expects the Puzzles heading and six cards instead of "Scene not found" (D-01); the unknown-level fallback primary link is `All levels` → `#/puzzles` instead of `Open First Pour` (D-04).
- `web/e2e/shell.spec.ts` (41-05, commit 792c2f644): drawer focus-trap cycle 26 → 27 focusables and loops 27 → 28, because the Puzzles link joins the drawer Tab order (D-03). The 25-preview and 25-icon counts are unchanged. Gate 5 checks the file against that commit.
- `web/tests/puzzle-format.test.ts` and `web/e2e/puzzle-levels.spec.ts` (41-06): the last level's disabled "No next level yet" Next became an enabled `All levels` link to `#/puzzles`, primary `next` (D-04, D-12).
- `web/tests/puzzle-progress-stars.test.ts` (41-04): "writes nothing to the progress key for a lost run" now expects exactly one write (Play's placement save, D-16) and no stars for the loss.
- 41-03 added `sameLevelStars` so a decided run still records exactly once in the StarStore (existing record-once test unchanged).

## Decision coverage (D-01..D-21)

| Decision | Plan(s) | Files | Proof |
| -------- | ------- | ----- | ----- |
| D-01 `#/puzzles` route, exact match, scene parser unchanged | 41-05 | web/src/routing/playable-route.ts | tests/playable-route.test.ts; gate 4 (hash.ts unchanged), gate 5 (hash.test.ts unchanged); puzzle.spec.ts |
| D-02 non-canvas grid page, no target, title, identity, no HUD obstacle, footer | 41-05 | web/src/player/playable-target.ts, PuzzleSelectPanel.tsx, PlaygroundStage.tsx | tests/playable-target.test.ts; puzzle-chrome.test.ts; gate 24; puzzle-progress.spec.ts fresh-grid (source link reachable) |
| D-03 text-only sidebar Puzzles link with current state | 41-05 | web/src/components/PlaygroundShell.tsx | puzzle-progress.spec.ts "opens the grid from the sidebar Puzzles link"; shell.spec.ts 25-preview/icon counts unedited; gate 5 |
| D-04 All levels in options drawer, fallback and last-level Next | 41-06 | PuzzleLevelLinks.tsx, PuzzleOptionsDrawer.tsx, PuzzleFallbackPanel.tsx, format.ts, PuzzleResultCard.tsx | puzzle-format.test.ts; puzzle.spec.ts fallback; puzzle-levels.spec.ts; puzzle-progress.spec.ts drawer and last-level scenarios; gate 27 |
| D-05 six cards in registry order with number, title, caption, stars, lock; no wasm | 41-02, 41-05 | progress-view.ts, PuzzleSelectPanel.tsx | puzzle-progress-view.test.ts; puzzle-progress.spec.ts fresh-grid |
| D-06 unlocked card is a full link; locked card non-interactive "Solve {prev} to unlock" | 41-02, 41-05 | progress-view.ts, PuzzleSelectPanel.tsx, PuzzleIcons.tsx | puzzle-progress.spec.ts fresh-grid and deep-link scenarios |
| D-07 responsive grid, no horizontal scroll at 390 px, heading and intro | 41-05 | web/src/styles/puzzle-select.css | puzzle-progress.spec.ts (3 columns desktop, 1 phone, no horizontal scroll); screenshots |
| D-08 separate pure core plus injected-provider shell | 41-02, 41-03 | puzzle/progress.ts, puzzle/progress-view.ts, player/puzzle-progress.ts | puzzle-progress.test.ts, puzzle-progress-shell.test.ts; gate 22 |
| D-09 StarStore hydrated, write-through, never-lowering merge, re-read before write | 41-03 | puzzle-runtime.ts, puzzle-run.ts, stars.ts | puzzle-progress-stars.test.ts (another-tab), puzzle-progress-shell.test.ts; puzzle-progress.spec.ts "never lowers saved stars" |
| D-10 pure linear unlock rule, degraded unlocks all, deep links always open | 41-02 | progress-view.ts | puzzle-progress-view.test.ts; puzzle-progress.spec.ts deep-link and blocked-storage scenarios |
| D-11 stars recorded before the result card renders | 41-03 | puzzle-run.ts | puzzle-progress-stars.test.ts ordering test; puzzle-progress.spec.ts "persists stars and unlocks level 2" (grid live on same page) |
| D-12 Next goes to next PUZZLE_IDS entry even after a loss; last level All levels | 41-06 | puzzle/format.ts | puzzle-format.test.ts; puzzle-levels.spec.ts |
| D-13 one v1 key, unknown ids preserved, first-hint key separate | 41-02 | puzzle/progress.ts | puzzle-progress.test.ts (unknown ids, `__proto__`); gates 6 and 26 |
| D-14 degraded mode on getter/probe/JSON/shape/future version/mid-session setItem; never writes | 41-03 | player/puzzle-progress.ts | puzzle-progress-shell.test.ts (byte-identical seeds); puzzle-progress.spec.ts future-version and corrupt records byte-for-byte |
| D-15 degraded note under grid heading and in options drawer, not in HUD | 41-05, 41-06 | PuzzleSelectPanel.tsx, PuzzleLevelLinks.tsx, puzzle-levels.ts | puzzle-progress.spec.ts blocked storage (note on grid and sheet); gate 18 (HUD safe frame 64 passed, spec unchanged) |
| D-16 save after committed edits and at Play, not on drag frames; invalid saved too | 41-04 | player/puzzle-saved-placements.ts, puzzle-editing.ts, puzzle-run.ts | puzzle-save-placements.test.ts; puzzle-progress.spec.ts "restores placements after a reload" |
| D-17 Rust levelHash, FNV-1a 64 over Debug, pinned, native == wasm, engine untouched | 41-01 | crates/liquidfun-wasm/src/puzzle/level_hash.rs, info.rs, level-hashes.txt | gates 12 and 13 (pins native and wasm), gate 2 (engine), gate 1 (25/25) |
| D-18 restore precedence seam > matching saved > empty; discard on mismatch, stars kept | 41-02, 41-04 | progress-view.ts, puzzle-saved-placements.ts, puzzle-lifecycle.ts | puzzle-restore.test.ts; puzzle-progress.spec.ts "discards placements saved for a changed level" |
| D-19 Vitest coverage of core, shell, unlocks, placements, result action, route | 41-02..41-06 | web/tests/puzzle-progress*.test.ts, puzzle-restore, puzzle-save-placements, puzzle-format, playable-route | gate 15 (1541 tests) |
| D-20 puzzle-progress.spec.ts on desktop and phone portrait; existing specs pass | 41-07 | web/e2e/puzzle-progress.spec.ts, web/package.json | gate 19 (81 passed, spec in test:puzzle), gate 17 (45 passed) |
| D-21 phase done criteria | 41-08 | target/phase41/checks-08.sh | 27/27 gates exit 0 |

## Screenshots (gitignored)

- `target/phase41/screens/grid-fresh-1280x800.png`, `grid-fresh-390x844.png`
- `target/phase41/screens/last-level-ended-1280x800.png`, `last-level-ended-390x844.png`
- `target/phase41/screens/puzzles-1280x800.png`, `puzzles-390x844.png`, `puzzles-390x844-scrolled.png`, `puzzles-ff-1280x800.png`
- `target/phase41/screens/sidebar-ff-1280x800.png`, `sidebar-dialog-390x844.png`

## Deviations from Plan

None. The plan executed as written. The script uses the `no_match` helper (from the Phase 40 gate script) in place of `! grep`, because errexit ignores a negated command in the middle of a gate and `! grep` would also pass when grep errors on a missing file. All gates use the plan's commands otherwise.

## Fix commits and failed attempts

None. Attempt 1 passed all 27 gates, so there are no `checks-08.attempt-<n>.*` files.

## Known Stubs

None.

## Threat Flags

None. T-41-28: gates 1-6, 17 and 18. T-41-29: gate 22. T-41-30: gate 23. T-41-31: one attempt, all 27 recorded at exit 0, no failed evidence relabelled.

## Self-Check: PASSED

The four gate artifacts in target/phase41 and this SUMMARY exist; checks-08.results has 27 exit=0 lines; no task commits were claimed.

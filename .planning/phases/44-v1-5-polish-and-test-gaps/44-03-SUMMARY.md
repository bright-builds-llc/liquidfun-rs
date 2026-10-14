---
phase: 44-v1-5-polish-and-test-gaps
plan: "03"
subsystem: web-e2e
tags: [puzzle, e2e, playwright, valve, accessibility, gates]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 44-2026-10-14T01-34-05
generated_at: "2026-10-14T02:08:54.692Z"
requirements-completed: []
requires:
  - "44-01 valve ARIA wiring (PuzzleSelectionBar.tsx) and the grand-finale 390x844 screenshot test"
  - "44-02 e2e typecheck project (new tests typecheck under the strict base)"
provides:
  - "Next-from-a-won-level e2e (first-pour -> thick-syrup)"
  - "timed-gate 1x/2x/4x fast-forward e2e with the valve stepper description checks"
  - "Phase 44 final gate evidence: target/phase44/checks-03.{sh,results,log}, 10/10 exit=0"
affects: []
tech-stack:
  added: []
  patterns:
    - "Delete the configurable test-placements seam global before a hash-link navigation to another level"
    - "One shared buildReference step for valve specs; Retry at each speed reuses the build"
key-files:
  created: []
  modified:
    - web/e2e/puzzle-levels.spec.ts
    - web/e2e/puzzle-valve.spec.ts
decisions:
  - "44-03: Next e2e deletes the seam global with Reflect.deleteProperty before clicking the hash link; without the deletion the route never reaches building (checked by a temporary revert)"
  - "44-03: timed-gate 1x/2x/4x runs share one build and give identical fingerprints (outcome 429, decided 519, peak 582, held hash e405de2194871266)"
  - "44-03: 10/10 phase gates exit 0 on the first run at 813516547; crates/ untouched since 9e6efcc23, so no scene-spot rerun; no fix commit"
metrics:
  duration: 5min
  completed: 2026-10-14
  tasks: 3
  files: 2
---

# Phase 44 Plan 03: Next and valve fast-forward e2e, final gates Summary

Two missing e2e flows now exist. One wins first-pour and follows "Next level" into a clean thick-syrup build. The other plays the timed-gate reference at 1×, 2× and 4× with identical fingerprints and checks the valve stepper's accessible description in the browser. All 10 phase gates passed on the first run on the final tree.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | 29b792625 | test(44-03): cover Next from a won level |
| 2 | 813516547 | test(44-03): cover timed-gate fast-forward and the valve stepper description |
| 3 | (none) | Gate evidence lives in gitignored `target/phase44/` |

## Task 1: Next from a won level (D-07)

- The new test is `wins first-pour and opens thick-syrup ready to build from Next at 1280x800` in `web/e2e/puzzle-levels.spec.ts`.
- It installs a 15 Hz clock and the solution seam, switches to 4× and wins first-pour.
- It then deletes `__liquidfunPuzzleTestPlacements` with `Reflect.deleteProperty`, checks that the global is gone, and clicks "Next level".
- It asserts all of the following:
  - the URL ends `#/puzzle/thick-syrup`;
  - `data-puzzle="thick-syrup"` and `data-puzzle-phase="building"` are set;
  - the placements are empty;
  - the title is "Thick Syrup" (`PUZZLE_TITLES`);
  - the failure panel shows `data-failed="false"`;
  - `usedFallback` is false.
- **Non-vacuity check:** I temporarily replaced the deletion with a no-op. The test then failed because the route never reached building. That matches the seam's "Test placements name another level" path. I restored the file afterwards.
- `puzzle-levels.spec.ts` passed in full, 22/22.

## Task 2: timed-gate at 1×, 2× and 4× (D-08) and the valve ARIA hooks (D-04)

- `buildReference(page)` opens timed-gate, places the fixture tools through the tray, canvas, keys and stepper, and asserts the placements. Both tests share it. `placeFixtureTool(page, placement)` now appears exactly once. The rest of the existing replay test is unchanged.
- **ARIA checks:** while the valve is still selected after placement, the readout has `id="puzzle-valve-time"` and no `aria-hidden`. The `Valve opening time` group has `aria-describedby="puzzle-valve-time"` and the accessible description `valveSecondsText(9)`. After Escape, the selection is empty and the group has no `aria-describedby`.
- **Run fingerprints** (captured with a temporary log line that I removed before the commit):

| Speed | outcomeStep | decidedStep | peakHeld | heldHash |
|-------|-------------|-------------|----------|----------|
| 1× | 429 | 519 | 582 | e405de2194871266 |
| 2× | 429 | 519 | 582 | e405de2194871266 |
| 4× | 429 | 519 | 582 | e405de2194871266 |

- **Runtime:** in the Playwright list reporter, `plays the timed-gate reference at 2× and 4× with the 1× outcome` took **11.3 s**, and 12.0 s on the fingerprint-capture run. The existing replay test took 6.5 s. Both are well inside the 120 s and 240 s budgets.

## Task 3: Phase gates (D-11, D-12)

`target/phase44/checks-03.sh` ran once against HEAD `813516547` with phase base `9e6efcc23`. It is the first and only attempt, so no failed attempt files exist.

| # | Gate | Result | Evidence in `checks-03.log` |
|---|------|--------|-----------------------------|
| 1 | typecheck | exit=0 | `tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json`; the script line is present |
| 2 | vitest | exit=0 | 119 files, 1843 tests passed |
| 3 | bright-builds | exit=0 | findings=0 |
| 4 | crates-untouched | exit=0 | no diff and no untracked files under `crates/` |
| 5 | pinned-files | exit=0 | the protected tests, `web/src/catalog`, `bun.lock`, `tsconfig.json` and `PuzzleToolTray.tsx` are unchanged; the only `package.json` change is `typecheck` |
| 6 | markdown-untouched | exit=0 | no non-planning `.md` changed |
| 7 | build-app | exit=0 | `just web-wasm` (unchanged crates) + `vite build` |
| 8 | puzzle-e2e | exit=0 | 104 passed, including both new tests; the screenshot exists |
| 9 | water-e2e | exit=0 | 13 passed |
| 10 | hud-safe-frame | exit=0 | 64 passed |

The wasm build printed the existing `support.rs:98` unreachable-code warning. It is a v1.4 leftover recorded in REQUIREMENTS.md, not new.

**Screenshot:** `target/phase44/screens/level-grand-finale-390x844.png` was regenerated by the puzzle-e2e gate. I viewed it, and the portrait tray shows "Plank" over "×2" and "Valve" over "×1", each beside its icon, with no ellipsis.

## Decision coverage

| Decision | Plan | File(s) | Proof |
|----------|------|---------|-------|
| D-01 full tray names on phones | 01 | `web/src/styles/puzzle-editor.css` | `puzzle-levels.spec.ts` "shows every tray name in full at 390x844 / 360x780" (in puzzle-e2e gate); screenshot |
| D-02 tray HUD box contract kept | 01 | same CSS; `PuzzleToolTray.tsx` unchanged | same tests (`expectTrayFits`); pinned-files gate; hud-safe-frame gate |
| D-03 landscape/non-stage unchanged, accessible names unchanged | 01 | CSS scoped to portrait two-kind trays | "keeps grand-finale's landscape tray as icon and count at 1280x800"; accessible-name asserts |
| D-04 readout describes the stepper group, no live region | 01, 03 | `web/src/components/PuzzleSelectionBar.tsx`; `web/e2e/puzzle-valve.spec.ts` | "plays the timed-gate reference at 2× and 4× with the 1× outcome" (`toHaveAccessibleDescription`, `not.toHaveAttribute("aria-describedby")`) |
| D-05 TS tick cap 255 | 01 | `web/src/puzzle/valve-format.ts` | `puzzle-valve-format.test.ts` cap tests (vitest gate) |
| D-06 seam valve default open tick | 01 | `web/src/puzzle/placements.ts` | `puzzle-placements.test.ts` valve-default test (vitest gate) |
| D-07 Next navigation e2e | 03 | `web/e2e/puzzle-levels.spec.ts` | "wins first-pour and opens thick-syrup ready to build from Next at 1280x800" (puzzle-e2e gate) |
| D-08 valve fast-forward e2e | 03 | `web/e2e/puzzle-valve.spec.ts` | 1×/2×/4× fingerprints equal, 429/582 (puzzle-e2e gate) |
| D-09 e2e tsconfig and two-project typecheck | 02 | `web/tsconfig.e2e.json`, `web/package.json` | typecheck gate |
| D-10 16 e2e errors fixed at root | 02 | e2e helpers/specs, `playwright.config.ts`, `e2e/tesla-benchmark.d.ts` | typecheck gate (0 errors, no escape hatches) |
| D-11 no behavior change | 01-03 | none under `crates/` | crates-untouched, pinned-files, vitest, puzzle-e2e, water-e2e, hud-safe-frame, bright-builds gates |
| D-12 no human UAT, screenshot evidence | 01, 03 | `target/phase44/screens/level-grand-finale-390x844.png` | puzzle-e2e gate `test -f`; viewed (Plank/×2, Valve/×1) |

## Success criteria coverage

| Criterion | Evidence |
|-----------|----------|
| SC1 phone tray names not truncated | 44-01 tray-fit tests re-run green in the puzzle-e2e gate (104/104); hud-safe-frame 64/64; screenshot shows full names |
| SC2 valve readout accessible, tick capped at 255 | vitest gate (cap and seam tests); new valve e2e proves the description is present with a selection and absent without one |
| SC3 Next and timed-gate fast-forward e2e | both new tests pass inside the puzzle-e2e gate; fingerprints table above |
| SC4 e2e typechecked; behavior unchanged | `cd web && bun run typecheck` (runs both projects) exit 0 in the typecheck gate; puzzle outcomes unchanged (test:puzzle 104/104); crates/ untouched since 9e6efcc23, so the 25 scene fingerprints stay valid without a rerun |

## Deviations from Plan

None. The plan was executed exactly as written. I ran `prettier` once by mistake. The repo has no prettier config, so it reformatted unrelated lines in `puzzle-levels.spec.ts`. I reverted that before the commit and reapplied only the planned edit.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: web/e2e/puzzle-levels.spec.ts, web/e2e/puzzle-valve.spec.ts, target/phase44/checks-03.sh, target/phase44/checks-03.results, target/phase44/checks-03.log, target/phase44/screens/level-grand-finale-390x844.png
- FOUND commits: 29b792625, 813516547
- `grep -c 'exit=0' target/phase44/checks-03.results` = 10; `grep -c ' exit='` = 10

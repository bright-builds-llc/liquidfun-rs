---
phase: 39-place-then-play-tools-and-stars
plan: "13"
subsystem: web-puzzle-e2e
tags: [playwright, e2e, puzzle, editor, touch, keyboard, stars, fast-forward, D-10, D-26, D-27, D-28, D-30]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T02:02:58Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: D-10 decision step and peak-held latch (39-08/09), the editor pointer and data-puzzle-* debug attributes (39-11), the tray, selection bar, transport, star list and keyboard routing (39-12)
provides:
  - web/e2e/puzzle-editor-helpers.ts (worldToPage, gridToPage, readPlacements, expectPlayEnabled, solutionExpectation, readRunFingerprint, SCREENS_DIR_39)
  - web/e2e/puzzle-editor.spec.ts (12 browser tests on the chromium project)
  - web/e2e/puzzle-touch.spec.ts (tap win flow on the touch project)
  - Playwright `touch` project (Pixel 7 Chromium descriptor, 390x844, hasTouch, isMobile)
affects:
  - 39-14 (phase gate: `CI=true bun run test:puzzle` now covers the Phase 38 spec, the editor spec and the touch spec)
tech-stack:
  added: []
  patterns:
    - won-run e2e literals are derived from the recorded solution fixture, not hard-coded
    - page coordinates come from the published camera (`data-world-frame` + `worldBoundsForPuzzle`), failing fast if the scale disagrees with `data-pixels-per-meter`
    - run determinism is compared as a four-attribute fingerprint (outcome step, decided step, peak held, held hash)
key-files:
  created:
    - web/e2e/puzzle-editor-helpers.ts
    - web/e2e/puzzle-editor.spec.ts
    - web/e2e/puzzle-touch.spec.ts
  modified:
    - web/playwright.config.ts
    - web/package.json
    - web/e2e/puzzle.spec.ts
    - web/e2e/puzzle-helpers.ts
key-decisions:
  - "Won-run literals come from `solutionExpectation()`, which parses first-pour-solution.txt (outcome_step 201, peak_held 811, brim_step 262) and applies full 900, target 60% and limit 720, with an independent copy of the timer formula rather than importing production formatting"
  - "Delete-removal is its own test instead of a second page inside the keyboard test, and a STAR-02 best-of test was added: together they bring the literal test count to 10"
  - "Edit-history rounds 2 and 3 first select the existing plank by clicking it and remove it with 'Remove selected tool'. Re-arming over a placed plank would add a second, overlapping plank"
  - "No deciding-state assertion was added. The 39-12 note shows Playwright polling misses the ~300 ms window, and this plan does not require that check"
requirements-completed: [TOOL-02, TOOL-07, STAR-02]
duration: 9min
completed: 2026-10-13
---

# Phase 39 Plan 13: Place-then-Play browser e2e Summary

**Browser tests now cover the full place-then-play flow. Mouse, drag, keyboard and touch input all reach the fixture-derived "Filled! 90% of 60%" win with three stars. Stop, edit history and 1×/2×/4× runs give the same fingerprint as a clean seam run, and the camera stays fixed in three viewports.**

## Performance

- **Duration:** about 9 min (first commit 20:57:35 −0500, last task commit 21:02:54 −0500; work began after the previous plan's commit at 20:54:21)
- **Completed:** 2026-10-13
- **Tasks:** 3/3
- **Files:** 3 created, 4 modified

## Accomplishments

- **Touch project:** `chromium` ignores `puzzle-touch.spec.ts`, and a new `touch` project matches only that spec. It uses the Pixel 7 Chromium descriptor with a 390×844 viewport, `hasTouch` and `isMobile`, because `browser:install` installs Chromium only. `test:puzzle` runs all three puzzle specs.
- **Phase 38 spec:** only the D-10 won-run literals changed (table below).
- **Editor spec:** 12 tests on the chromium project:
  - a mouse win
  - a drag that keeps the grab offset, with `data-pointer-accepted` still `0`
  - a keyboard-only win: Enter, ArrowLeft ×2, Shift+R, Shift+Tab to `role="application"`, then Space
  - Delete removes the tool
  - cup and tool overlaps show the exact status lines and disable Play; Clear re-enables it
  - Stop then replay equals a clean seam run in a fresh context
  - the camera stays fixed and the HUD stays clear of the world at 1280×800, 390×844 and 844×390
  - the D-27 three-round edit history matches the clean held hash
  - 1×/2×/4× give equal fingerprints that match the fixture
  - STAR-02: the best stars survive Edit, Stop and a losing run
- **Touch spec:** taps arm, place, rotate and play to the win card with three earned stars. Every tray, selection-bar and transport button measures at least 44×44 before and after the run. The screenshot is `target/phase39/screens/place-win-touch.png`.

## Changed Phase 38 e2e literals (D-10, D-30)

All are in `web/e2e/puzzle.spec.ts` and now come from `solutionExpectation()`:

| Literal | Old | New (derived) | Derivation |
|---|---|---|---|
| Win card text | `"Filled! 76% of 60%"` | `"Filled! 90% of 60%"` | floor(811×100/900) = 90 |
| Timer at the end | `"0:08.6"` | `"0:07.6"` | 720 − 262 = 458 steps → 76 tenths |
| Held step (`SOLUTION_HELD_STEP`) | `"321"` | `"382"` | brim step 262 + 120 tail steps |
| Meter `aria-valuenow` and text | `"76"` / `"76%"` | `"90"` / `"90%"` | the same peak-held percent |

There is one other non-D-10 literal change in this spec. It came from an earlier plan (39-07), and this plan did not touch it: the building-card text `EMPTY_BUILDING_TEXT` became "Fill the cup to 60% before time runs out. Pick a tool from the tray to start." No `expectPlayEnabled` waits were added to the Phase 38 spec, and the lost-run assertions are unchanged.

## Task Commits

1. **Task 1: touch project, helpers and D-10 literals:** `61eff262c` (test)
1. **Task 2: editor e2e suite:** `e6ce23474` (test)
1. **Task 3: touch spec and full suite:** `08d20038e` (test)

## Verification evidence

All runs used fresh artifacts: `bun scripts/web-build.ts wasm && bun run build:app`, then `build:app` again before the final suite.

| Command | Result |
|---|---|
| `CI=true bunx playwright test e2e/puzzle.spec.ts --project=chromium` | 16 passed |
| the same with `--repeat-each=5` | 80/80 passed |
| `CI=true bunx playwright test e2e/puzzle-editor.spec.ts --project=chromium --repeat-each=5` (11 tests, before the STAR-02 test) | 55/55 passed |
| the same with all 12 tests | 60/60 passed |
| edit-history test with the held-hash attribute assertion, `--repeat-each=3` | 3/3 passed |
| `CI=true bunx playwright test e2e/puzzle-touch.spec.ts --project=touch --repeat-each=5` | 5/5 passed |
| `CI=true bun run test:puzzle` (both projects) | 29/29 passed (chromium 16 + 12, touch 1) |
| `bun run typecheck` | clean |
| `bun scripts/bright-builds-check.ts all` | findings=0 |

`web/tsconfig.json` does not include `e2e/`, so a scratch tsconfig typechecked the puzzle e2e files. The new files have no errors.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exported `SOLUTION_FIXTURE` from `web/e2e/puzzle-helpers.ts`**
- **Issue:** `solutionExpectation()` has to parse the same fixture file that `installSolutionPlacements` uses, and the path constant was module-private.
- **Fix:** added `export` to the constant. Behavior is unchanged.
- **Commit:** `61eff262c`

**2. Test shape: Delete is a separate test, and a STAR-02 test was added**
- **Issue:** the plan put the Delete check on a second page inside the keyboard test. That gives 9 literal `test(` lines, but the acceptance grep needs at least 10. STAR-02 (best-of across runs) also had no browser proof yet (39-12 left it pending).
- **Fix:** "removes the selected tool with Delete" is now its own test, and "keeps the best stars across Edit, Stop and a losing run" was added. All plan assertions are kept; none are weakened.
- **Commit:** `e6ce23474`

**3. Edit-history rounds 2 and 3 remove the existing tool first**
- **Issue:** after a round, plank:8,64,23 is still placed. Repeating "arm, click (8,64)" would add a second, overlapping plank, and Play would be disabled.
- **Fix:** later rounds click the placed plank to select it (`data-puzzle-selected="0"`), then remove it before the plan's arm/place/remove/re-place/rotate/Play/Stop sequence. This adds more edit history, not less.
- **Commit:** `e6ce23474`

**Product fixes:** none. No failures exposed bugs in Plans 07–12, so no files outside e2e and config changed.

## Deferred Issues (pre-existing, out of scope)

- `web/playwright.config.ts` line 26 fails `tsc` under `exactOptionalPropertyTypes`, because the reporter tuple widens to `"line"[][]`. The pre-plan config at `674191e61` gives the same error, so it predates this plan. `web/e2e/player-helpers.ts` lines 160–169 also have pre-existing canvas `HTMLElement | SVGElement` typing errors. Neither file is in the `typecheck` include.

## Known Stubs

None.

## Threat Flags

None. T-39-38: placements still enter only through `page.addInitScript` (`installSolutionPlacements`), never through a URL or storage. T-39-39: only the D-10 won-run literals changed in the Phase 38 spec; they are derived from the fixture and listed above.

## Requirements

- **TOOL-02:** the touch-project tap flow and the 44 px target check pass, and the mouse place/drag/rotate/remove tests pass. Marked complete.
- **TOOL-07:** the 1×/2×/4× fingerprints are equal to each other and to the fixture. Marked complete.
- **STAR-02:** the win earns three stars, and the best-of glyphs survive Edit, Stop and a losing run. Marked complete.
- TOOL-01, TOOL-03, TOOL-04, TOOL-06 and STAR-01 have further e2e coverage here. Plan 12 had already marked TOOL-01, TOOL-04, TOOL-06 and STAR-01 complete.

## Self-Check: PASSED

The three created files exist, and commits `61eff262c`, `e6ce23474` and `08d20038e` are in git history.

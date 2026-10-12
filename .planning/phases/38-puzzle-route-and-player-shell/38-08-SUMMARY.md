---
phase: 38-puzzle-route-and-player-shell
plan: "08"
subsystem: web-e2e
tags: [playwright, e2e, puzzle, GOAL-01, GOAL-03, PUZ-02, PUZ-03, PUZ-04, PUZ-05]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T03:31:02.079Z"
requires:
  - main[data-puzzle*] attributes, PuzzlePanel, PuzzleFallbackPanel and puzzle shell wiring (Plans 06, 07)
  - __liquidfunPuzzleTestPlacements seam (web/src/puzzle/placements.ts)
  - parseFixture (web/scripts/puzzle-replay/fixture.ts) and first-pour-solution.txt
provides:
  - web/e2e/puzzle-helpers.ts (paths, viewports, phase/held waits, camera reads, HUD geometry, seam installer)
  - web/e2e/puzzle.spec.ts (14 Chromium tests)
  - "web/package.json test:puzzle script"
  - target/phase38/screens/{desktop-building,phone-lost,desktop-won}.png
affects:
  - 38-09 (fingerprint and protected-file gate; can run test:puzzle as browser evidence)
tech-stack:
  added: []
  patterns:
    - controlled rAF refresh (15 Hz = 4 steps per frame, 60 Hz = 1) keeps run step counts deterministic
    - camera attributes compared as raw strings across building, running, ended and held
    - Retry waits on data-puzzle-held="false", not the short running phase
key-files:
  created:
    - web/e2e/puzzle-helpers.ts
    - web/e2e/puzzle.spec.ts
  modified:
    - web/package.json
key-decisions:
  - "One commit for both tasks, as the plan's Task 1 step 3 specifies"
  - "readHudGeometry and overlappingObstacles are copied from hud-safe-frame.spec.ts, which stays untouched"
requirements: [GOAL-01, GOAL-03, PUZ-02, PUZ-03, PUZ-04, PUZ-05]
duration: 9min
completed: 2026-10-12
---

# Phase 38 Plan 08: Puzzle route e2e Summary

**The puzzle player is now proven end to end in Chromium against the built `dist`. A 14-test Playwright suite covers building, the natural loss, the seam-driven win with deterministic Retry and Edit, a fixed camera at desktop and phone sizes, the HUD obstacle rules, scene chrome and input turned off, the unknown-level fallback, and a clean return to scenes.**

## Performance

- **Duration:** about 9 min
- **Tasks:** 2 of 2, in a single commit as the plan specifies
- **Files:** 2 created, 1 modified

## Accomplishments

### Task 1: helpers and script

- `web/e2e/puzzle-helpers.ts` exports:
  - `PUZZLE_PATH`, `UNKNOWN_PUZZLE_PATH`, `DESKTOP`, `PHONE_PORTRAIT` and `SCREENS_DIR`
  - `puzzleMain`, `waitForPuzzlePhase` and `waitForHeld`
  - `readPuzzleCamera` (raw strings; throws when an attribute is missing) and `stablePuzzleCamera`
  - `installSolutionPlacements`: parses `first-pour-solution.txt` with `parseFixture` and passes plain objects through `addInitScript`. Nothing goes through the URL.
  - `readHudGeometry` and `overlappingObstacles`, copied from the protected spec
- `web/package.json`: adds `"test:puzzle": "playwright test e2e/puzzle.spec.ts"`. `test:player` is unchanged.

### Task 2: the spec (commit `a559b5da8`)

The plan lists 10 test cases. Running some at two viewports gives 14 tests:

| Test | Viewport(s) | Proves |
| --- | --- | --- |
| opens First Pour in building | 1280x800 | step 0, the title, the document title, meter `0%` ` / 60%`, timer `0:12.0`, Play enabled, the goal text, the direct backend |
| loses a natural run | 390x844 | `Out of time: 0% of 60%`, timer `0:00.0`, Next disabled, Retry and Edit present, held at step 840 (GOAL-03) |
| wins with the recorded solution | 1280x800 | the meter rises above 0 while running; `Filled! 76% of 60%`, `0:08.6`, held at 321; the meter shows the latched `aria-valuenow=76`; Retry replays to the same result; Edit returns to building at step 0 with `0:12.0` (GOAL-01, D-11, D-22) |
| keeps the camera fixed | both | `data-world-frame` and `data-pixels-per-meter` are identical in building, running, ended and held, with no fallback; layout is `landscape` or `portrait` to match the viewport (D-19) |
| keeps every HUD leaf clear | both | no obstacle overlaps the world frame in building or ended, the obstacle count is the same in both, and the legend is visible (D-18) |
| turns scene interactions off | both | none of the 9 scene chrome buttons and no range input; Puzzle options holds Particles, the source link and the footer, and no tilt control; a canvas drag leaves `data-pointer-accepted="0"` and the canvas is not `canvas-interactive` (D-16, D-17) |
| form-factor layout | 1280x800 | no canvas stage; Play, meter, timer and result card are visible; the legend shows after Play |
| unknown-level fallback | both | `Level not found` with the raw id as text and both links; an `<img onerror>` payload creates no `img` and no `__pwned` (T-38-24); `#/puzzle` shows the empty-id copy (D-05) |
| levels out of scene routes | 1280x800 | `#/puzzles` shows `Scene not found`; the form-factor sidebar has Dam Break and no `#/puzzle/` link and no First Pour link (D-03) |
| returns to scenes cleanly | 1280x800 | after a hash change to dam-break, the scene plays, `data-puzzle` is gone, and a drag is accepted (PUZ-06) |

## Verification

- `bun run build:app`, then `CI=true bun run test:puzzle`: **14 passed (9.2 s)**.
- To check for flakiness, `CI=true bunx playwright test e2e/puzzle.spec.ts --repeat-each=3` ran: **42 passed (18.6 s)**.
- Screenshots were written to:
  - `target/phase38/screens/desktop-building.png`
  - `target/phase38/screens/phone-lost.png`
  - `target/phase38/screens/desktop-won.png`
- I looked at the won and lost screenshots. They show the 76% and 0% result cards, the frozen timers and the fixed HUD around the world.
- `bun run typecheck` is clean. The puzzle e2e files are outside `tsconfig` `include`, so I also type-checked them with a temporary config that adds `types: ["node"]`. It reported no errors, and the temporary config was deleted.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet 589cb0fb9 -- web/e2e/player-helpers.ts web/e2e/hud-safe-frame.spec.ts web/tests/hash.test.ts web/tests/portrait-bounds.test.ts web/tests/readme-svg.test.ts` exits 0.
- The suite does not use the synthetic media clock, and it does not count pending animation callbacks. It overrides rAF timestamps only, so the one-shot HUD measurement frame from `e0b8a268d` does not affect it.

## Deviations from Plan

None. The plan was executed as written, and no product bugs turned up.

- **Process note:** both tasks share one commit (`a559b5da8`), as the plan's Task 1 step 3 instructs. `.planning/config.json` was left unstaged.

## Known Stubs

None.

## Threat Flags

None. All three planned threats are mitigated:

- T-38-24: the HTML payload test passes.
- T-38-25: placements enter only through `addInitScript`, and the verify grep for URL placement data finds nothing.
- T-38-26: controlled refresh keeps step counts deterministic, timeouts are generous, and `CI=true` stops a stale preview server being reused.

## Self-Check: PASSED

- FOUND: web/e2e/puzzle-helpers.ts, web/e2e/puzzle.spec.ts, test:puzzle in web/package.json
- FOUND: target/phase38/screens/desktop-building.png, phone-lost.png, desktop-won.png
- FOUND: commit a559b5da8

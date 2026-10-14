---
phase: 44-v1-5-polish-and-test-gaps
plan: "01"
subsystem: web-puzzle-hud
tags: [puzzle, tray, valve, accessibility, css, e2e]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 44-2026-10-14T01-34-05
generated_at: "2026-10-14T02:00:20.046Z"
requirements-completed: []
requires: []
provides:
  - "maxOpenTick saturating at MAX_OPEN_TICK = 255"
  - "seam open tick defaulting by kind"
  - "stacked portrait two-kind tray chip"
  - "valve readout as the stepper group's accessible description"
  - "tray-fit e2e at 390x844 and 360x780"
affects: [44-02, 44-03]
tech-stack:
  added: []
  patterns:
    - "CSS-only grid override scoped to portrait two-kind trays; DOM unchanged"
key-files:
  created: []
  modified:
    - web/src/puzzle/valve-format.ts
    - web/src/puzzle/placements.ts
    - web/tests/puzzle-valve-format.test.ts
    - web/tests/puzzle-placements.test.ts
    - web/src/styles/puzzle-editor.css
    - web/src/components/PuzzleSelectionBar.tsx
    - web/e2e/puzzle-levels.spec.ts
    - web/e2e/puzzle-editor-helpers.ts
decisions:
  - "44-01: maxOpenTick saturates at MAX_OPEN_TICK 255 like Rust max_open_tick; seam valves without openTick read DEFAULT_VALVE_OPEN_TICK (kind checked before the open-tick read)"
  - "44-01: portrait two-kind tray chip is a 22px + max-content grid (name 14/20 over count 12/16); measured names fit at 390 (37/37, 36/36) and 360 wide inside the unchanged 218/188 x 44 tray"
  - "44-01: #puzzle-valve-time is no longer aria-hidden and describes the stepper group only while a valve is selected; no live region added"
metrics:
  duration: 3min
  completed: 2026-10-14
  tasks: 2
  files: 8
---

# Phase 44 Plan 01: Phone tray names and valve lane polish Summary

The portrait two-kind tray chip now stacks the count under the full tool name with a CSS-only grid. The valve readout became the stepper group's accessible description. The TypeScript valve tick saturates at 255 like the Rust `u8` lane, and a seam valve without `openTick` reads the valve default of 4.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 (RED) | a3a842ddb | test(44-01): add failing valve tick cap and seam open tick tests |
| 1 (GREEN) | f57c59b9a | fix(44-01): cap the valve tick at 255 and default seam valve open ticks |
| 2 | 877aba559 | fix(44-01): show full tray names on phones and describe the valve stepper by its time |

## Decision coverage

| Decision | File | Test |
|----------|------|------|
| D-01 full names, no ellipsis | `web/src/styles/puzzle-editor.css` (four rules after the portrait tray box) | `puzzle-levels.spec.ts` "shows every tray name in full at 390x844 / 360x780" |
| D-02 fixed 218/188 x 44 tray, 44 px items, equal widths, 14 px name / 12 px count | same CSS; the tray box block is byte-identical | same tests (`expectTrayFits`) |
| D-03 landscape, one-kind, three-kind and form-factor trays unchanged; accessible names unchanged | CSS scoped to `[data-puzzle-layout="portrait"] .puzzle-tray[data-kind-count="2"]` | "keeps grand-finale's landscape tray as icon and count at 1280x800"; accessible-name asserts in both portrait tests |
| D-04 / UI-D55 / UI-D56 readout describes the stepper group | `web/src/components/PuzzleSelectionBar.tsx` (`VALVE_TIME_ID`, `aria-describedby` only while `valve().enabled`, `aria-hidden` removed, no `aria-live`) | `puzzle-valve.spec.ts` and `puzzle-editor.spec.ts` still pass; grep criteria |
| D-05 tick cap 255 | `web/src/puzzle/valve-format.ts` (`MAX_OPEN_TICK`) | `puzzle-valve-format.test.ts` "caps the open tick at the u8 maximum above 7,650 steps", "keeps 255 at exactly 7,650 steps" |
| D-06 seam valve default open tick | `web/src/puzzle/placements.ts` (`readSeamOpenTick(entry, kind)`) | `puzzle-placements.test.ts` "reads a valve seam entry without an open tick as the valve default" (plank-without-tick test still passes) |
| D-12 evidence screenshot | `web/e2e/puzzle-levels.spec.ts`, `SCREENS_DIR_44` | `target/phase44/screens/level-grand-finale-390x844.png` |

## Measurements

I measured these with a throwaway Chromium spec that I deleted afterwards. The table shows name `clientWidth/scrollWidth`.

| Viewport | Level(s) | Kinds | Tray width | Item width | Names |
|----------|----------|-------|-----------|-----------|-------|
| 390x844 | first-pour, thick-syrup, jelly-pad, float-up | 1 | 218 | 218 | Plank 150/150 |
| 390x844 | timed-gate | 1 | 218 | 218 | Valve 150/150 |
| 390x844 | grand-finale | 2 | 218 | 105 / 105 | Plank 37/37, Valve 36/36 |
| 360x780 | first-pour .. float-up | 1 | 188 | 188 | Plank 120/120 |
| 360x780 | timed-gate | 1 | 188 | 188 | Valve 120/120 |
| 360x780 | grand-finale | 2 | 188 | 90 / 90 | Plank 37/37, Valve 36/36 |

The tray widths match the 39-UI-SPEC values (218 and 188), so the tray box did not need to change.

Non-vacuity check: I restored the pre-plan CSS temporarily, rebuilt, and ran the new tray-name tests. Both failed: name `scrollWidth` 42 > `clientWidth` 37 at 390 wide and 42 > 22 at 360 wide. I then restored the new CSS and rebuilt.

## Screenshot

`target/phase44/screens/level-grand-finale-390x844.png` (gitignored evidence): the portrait tray reads "Plank" over "×2" and "Valve" over "×1", each beside its icon. Neither name shows an ellipsis. The selection bar, transport and scale legend keep their positions.

## Verification

- `cd web && bun run typecheck && bun run test:unit`: 119 files, 1843 tests pass.
- `bun run build:app` succeeds.
- `CI=true bunx playwright test e2e/puzzle-levels.spec.ts e2e/puzzle-valve.spec.ts e2e/puzzle-editor.spec.ts`: 37 passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet 9e6efcc23 -- web/src/components/PuzzleToolTray.tsx crates/` exits 0. The CSS diff contains only additions.
- I type-checked the two edited e2e files under `noUncheckedIndexedAccess` with a scratch config and they are clean. The only errors were pre-existing ones in `e2e/player-helpers.ts`, which Plan 02 owns.

## Deviations from Plan

None. The plan was executed exactly as written.

The plan asked me to record one changed error message. A seam entry with an unknown kind and a non-number `openTick` used to throw the invalid-seam message. It now throws `INVALID_PLACEMENT_MESSAGE` ("Invalid puzzle placement") because the kind check moved before the open-tick read. Every other message is unchanged.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: web/src/puzzle/valve-format.ts, web/src/puzzle/placements.ts, web/src/styles/puzzle-editor.css, web/src/components/PuzzleSelectionBar.tsx, web/e2e/puzzle-levels.spec.ts, web/e2e/puzzle-editor-helpers.ts, target/phase44/screens/level-grand-finale-390x844.png
- FOUND commits: a3a842ddb, f57c59b9a, 877aba559

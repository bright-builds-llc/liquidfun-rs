---
phase: 26-catalog-shell-and-basin-scenes
plan: "05"
subsystem: testing
tags: [playwright, chromium, web-player-smoke, watch-first, particles, liquid-timer, play-02, basin-01, basin-02]

requires:
  - phase: 26-catalog-shell-and-basin-scenes
    provides: Eight-scene catalog with ready:true empty-control basins and capture plans
provides:
  - Chromium smoke covering eight scenes open/play/pause/reset
  - Watch-first e2e path when controls.length === 0 without pointer/control requirements
  - Eight Static preview sidebar asserts with .catalog-card still zero
affects:
  - Phase 26 verification / independent AI review
  - Phase 27 material scenes reusing watch-first e2e filter

tech-stack:
  added: []
  patterns:
    - Split player e2e by scene.controls.length rather than a hard-coded watch-first id list
    - ALL_SCENE_TIMEOUT_MS budgets eight-scene open/reset loops; SIX_SCENE_TIMEOUT_MS aliases it

key-files:
  created: []
  modified:
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
    - web/e2e/reset-honesty.spec.ts
    - web/e2e/shell.spec.ts

key-decisions:
  - "Watch-first scenes selected by controls.length === 0 so Phase 27 can reuse the rule"
  - "POINTER_CONTROL is Partial so Particles and Liquid Timer never require gestures"
  - "Drawer reverse-tab last link is Liquid Timer; wrap count raised to 10 for nine focusables"

patterns-established:
  - "Interactive scenes: gesture + labeled control; watch-first: play/pause/reset + credits only"
  - "Desktop sidebar smoke expects SCENES.length Static preview captions and aria-hidden SVGs"

requirements-completed: [PLAY-02, BASIN-01, BASIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:50:07Z

duration: 3min
completed: 2026-09-22
---

# Phase 26 Plan 05: Chromium Eight-Scene Smoke Summary

**Chromium `just web-player-smoke` now proves all eight catalog scenes open/play/pause/reset, with watch-first basins skipping pointer controls and eight static sidebar previews.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-09-22T00:46:56Z
- **Completed:** 2026-09-22T00:50:07Z
- **Tasks:** 2/2
- **Files modified:** 4

## Accomplishments

- Extended `SCENE_HASH_PATHS` for `particles` and `liquid-timer`; raised `ALL_SCENE_TIMEOUT_MS` to 160_000 with a deprecated `SIX_SCENE_TIMEOUT_MS` alias.
- Split player e2e: six interactive scenes keep gesture+control coverage; watch-first (`controls.length === 0`) prove play/pause/reset and credits without pointers.
- Updated shell desktop asserts to eight `Static preview` captions and eight `svg[aria-hidden='true']`; kept `.catalog-card` at 0.
- Fixed drawer reverse-tab last-link assumption for the new catalog order; `just web-player-smoke` passed 38/38.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend helpers and split watch-first player e2e** - `aa073fb` (feat)
2. **Task 2: Eight-preview shell asserts and run web-player-smoke** - `9757800` (feat)

**Plan metadata:** `f1b5fa7` (docs: complete plan)

## Files Created/Modified

- `web/e2e/player-helpers.ts` - Eight hash paths; `ALL_SCENE_TIMEOUT_MS`
- `web/e2e/player.spec.ts` - Interactive vs watch-first player coverage
- `web/e2e/reset-honesty.spec.ts` - Timeout import renamed to `ALL_SCENE_TIMEOUT_MS`
- `web/e2e/shell.spec.ts` - Eight previews; Liquid Timer last-link reverse-tab wrap

## Decisions Made

- Filter watch-first by `controls.length === 0` (D-14 / research Open Question 2).
- Keep `POINTER_CONTROL` as `Partial<Record<SceneId, …>>` covering only the original six interactive scenes.
- Do not raise `MAX_ADVANCE_STEPS` or cut particle counts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Drawer reverse-tab last link broke after eight scenes**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** Tab-trap test still expected Water Wheel as the last focusable link and used an 8-step wrap tuned for seven focusables.
- **Fix:** Point last link at Liquid Timer; raise reverse-tab iterations to 10 so wrap ≡ 1 (mod 9).
- **Files modified:** `web/e2e/shell.spec.ts`
- **Verification:** `just web-player-smoke` → 38 passed
- **Committed in:** `9757800` (Task 2)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Required for smoke green after catalog growth; no scope creep.

## Issues Encountered

First smoke run failed 37 passed / 1 failed on the mobile drawer reverse-tab assertion; fixed as above and re-ran to 38/38.

## Verification Evidence

- `cd web && bun run typecheck` — exit 0
- `just web-player-smoke` — exit 0; Playwright **38 passed**, 0 failed (initial Task 2 run)
- Re-verified 2026-09-22T01:51Z: `just web-player-smoke` — exit 0; Playwright **38 passed**, 0 failed (Vitest 186 passed)
- `MAX_ADVANCE_STEPS: u32 = 4` unchanged in `crates/liquidfun-wasm/src/session.rs`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 26 plans 01–05 complete; ready for phase verification / independent AI review (D-15; implementing agent must not self-approve).
- Watch-first e2e filter is reusable for Phase 27 material scenes with empty controls.

## Self-Check: PASSED

- FOUND: `web/e2e/player-helpers.ts` particles and liquid-timer hash paths
- FOUND: `web/e2e/player.spec.ts` watch-first / interactive split
- FOUND: `web/e2e/shell.spec.ts` eight Static preview asserts
- FOUND: `aa073fb` (Task 1)
- FOUND: `9757800` (Task 2)
- FOUND: `f1b5fa7` (docs metadata)

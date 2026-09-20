---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "05"
subsystem: playground-ui
tags: [playwright, chromium, reset-honesty, live-presets, construction-presets]

requires:
  - phase: 20-playground-catalog-previews-and-reset-honesty
    provides: keyed SceneControls remount and empty construction bag on Reset
  - phase: 20-playground-catalog-previews-and-reset-honesty
    provides: Desktop .demo-sidebar six Static preview and aria-hidden SVG counts
provides:
  - Chromium live and construction Reset toHaveValue proofs
  - Play/pause negative that keeps a dirty live select
  - test:player allowlist includes e2e/reset-honesty.spec.ts
affects: [20-06 phase verification]

tech-stack:
  added: []
  patterns:
    - Observe the first restarted data-step-index after Reset before 4-step catch-up
    - Construction Reset tests wait through Apply setting then Reset
    - test:player is an explicit Playwright file list

key-files:
  created:
    - web/e2e/reset-honesty.spec.ts
  modified:
    - web/package.json
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
    - web/e2e/demo-media-clock.spec.ts

key-decisions:
  - "resetNearZero watches the first restarted data-step-index instead of sampling after catch-up frames."
  - "Construction Reset tests use SIX_SCENE_TIMEOUT_MS because Apply large then Reset rebuilds two WASM worlds."
  - "The 240-frame demo-media clock test uses a 120s timeout so just web-player-smoke can finish."

patterns-established:
  - "Reset honesty proofs use toHaveValue and status text, never screenshot hashes."
  - "Play and pause must not remount live selects; only Reset scene restores documented initials."

requirements-completed: [WEB-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T19:08:35Z

duration: 22min
completed: 2026-09-20
---

# Phase 20 Plan 05: Reset Honesty Chromium Proofs Summary

**Chromium `reset-honesty.spec.ts` proves representative live and construction selects return to documented initials after Reset, play/pause keeps a dirty live select, and `test:player` actually runs that file.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-09-20T18:46:37Z
- **Completed:** 2026-09-20T19:08:35Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Live Reset proofs: Fountain Emission rate high→medium, Float or Sink Body cork→wood, Color Mixer Stir speed fast→slow, Water Wheel Jet strength strong→medium.
- Construction Reset proofs: Dam Break Water amount large+Apply→medium, Color Mixer Mix strength gentle+Apply→strong.
- Fountain play/pause keeps Emission rate high; no Reset in that test.
- `web/package.json` `test:player` lists `e2e/reset-honesty.spec.ts` and still lists `e2e/shell.spec.ts`.
- WEB-03 native rebuild: `resetNearZero` now records the first restarted `data-step-index` so 4-step catch-up cannot hide a real Reset.
- `just web-player-smoke` exited 0 with 37 passed.

## Task Commits

Each task was committed atomically:

1. **WEB-03 Reset step observation (deviation)**
   - `98c87f7` (fix): observe the restarted step before catch-up frames
2. **Task 1: Write live preset Reset honesty tests**
   - `b93810e` (test): four live `toHaveValue` Reset proofs
3. **Task 2: Construction Reset, play/pause negative, and test:player allowlist**
   - `9006371` (test): construction Reset, play/pause negative, allowlist, 120s demo-media timeout

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `web/e2e/reset-honesty.spec.ts` — live, construction, and play/pause honesty (140 lines)
- `web/package.json` — `test:player` four-file Playwright allowlist
- `web/e2e/player-helpers.ts` — MutationObserver-backed `resetNearZero`
- `web/e2e/player.spec.ts` — pause/play/reset uses `resetNearZero`
- `web/e2e/demo-media-clock.spec.ts` — 240-frame capture timeout 120s
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/deferred-items.md` — WEB-03 and demo-media timeout notes

## Decisions Made

- Treat the 20-04 `RESET_STEP_CEILING` 8 received 29 failure as a sampling race around an actually-reset world (`resetStep < seriesStep` already passed). Watch the first restarted index instead of changing `App.tsx`.
- Scope Dam Break Apply setting to the Water amount control so Gravity's duplicate Apply button cannot match.
- Give construction Reset tests `SIX_SCENE_TIMEOUT_MS` because Large then documented-initial rebuilds two native worlds inside one 30s default.
- Raise the 240-frame demo-media clock test to 120s so the required smoke allowlist can finish.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Observe Reset restart before 4-step catch-up**
- **Found during:** Extra WEB-03 diagnosis required before Task 1
- **Issue:** After 20-03 keyed remount, Playwright's single `data-step-index` read landed on 29. `resetStep < seriesStep` passed, so the native world did rebuild; `presentOwnedFrame` plus 4-step catch-up raced past ceiling 8 before the next CDP sample.
- **Fix:** Install a `data-step-index` MutationObserver before clicking Reset scene and assert the first restarted index below `RESET_STEP_CEILING`. Reuse that helper from the pause/play/reset test.
- **Files modified:** `web/e2e/player-helpers.ts`, `web/e2e/player.spec.ts`, `deferred-items.md`
- **Verification:** Isolated Dam Break pause/play/reset and six-scene `resetNearZero` passed; later `just web-player-smoke` 37/37.
- **Committed in:** `98c87f7`

**2. [Rule 3 - Blocking] Construction Reset and 240-frame capture timeouts**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** Dam Break Apply Large then Reset and the 240-frame demo-media capture exceeded the 30s Playwright default. A failure snapshot already showed Water amount Medium and Playing after Reset.
- **Fix:** `test.setTimeout(SIX_SCENE_TIMEOUT_MS)` on construction honesty tests; `test.setTimeout(120_000)` on the 240-frame capture test.
- **Files modified:** `web/e2e/reset-honesty.spec.ts`, `web/e2e/demo-media-clock.spec.ts`, `deferred-items.md`
- **Verification:** Isolated re-run passed; `just web-player-smoke` exited 0, 37 passed.
- **Committed in:** `9006371`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Required for WEB-03 native-rebuild proof and smoke exit 0. No `App.tsx` change. No Firefox/WebKit. No screenshot hashes.

## Authentication Gates

None.

## Issues Encountered

20-04's Reset ceiling 29 was a late sample after a real rebuild, not a leftover session. The 240-frame media-clock test was over the 30s cap even serially; 120s unblocked smoke.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 20-06 phase verification. Reset labels match documented initials. `just web-player-smoke` includes `e2e/reset-honesty.spec.ts`. Native scene files and `App.tsx` are unmodified.

## Self-Check: PASSED

---
*Phase: 20-playground-catalog-previews-and-reset-honesty*
*Completed: 2026-09-20*

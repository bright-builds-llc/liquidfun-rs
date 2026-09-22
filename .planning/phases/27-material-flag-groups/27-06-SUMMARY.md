---
phase: 27-material-flag-groups
plan: "06"
subsystem: testing
tags: [playwright, chromium, e2e, web-player-smoke, watch-first]

requires:
  - phase: 27-material-flag-groups
    provides: "Eleven ready catalog scenes including Surface Tension, Elastic Particles, Rigid Particles"
provides:
  - "Chromium just web-player-smoke covering eleven scenes"
  - "Watch-first play/pause/reset for five empty-control scenes"
  - "Eleven Static preview desktop asserts and Rigid Particles drawer last link"
affects:
  - phase-28-interaction-seams
  - independent-ai-review

tech-stack:
  added: []
  patterns:
    - "sessionStatus(.session-status) avoids ambiguous getByRole(status) with wireframe/tilt outputs"
    - "openDesktopDemo selects by href so Particles does not match Elastic/Rigid Particles"
    - "player-smoke sets CI=true so Playwright does not reuse a stray Vite dev server on 4173"

key-files:
  created: []
  modified:
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
    - web/e2e/shell.spec.ts
    - web/e2e/reset-honesty.spec.ts
    - scripts/web-build.ts

key-decisions:
  - "Raised ALL_SCENE_TIMEOUT_MS to 220_000 for eleven-scene open/reset loops"
  - "Forced CI=true for player-smoke so production preview is the gate, not a reused Vite dev server"
  - "Scoped session status to .session-status after tilt/wireframe also expose status roles"

patterns-established:
  - "Watch-first coverage stays controls.length === 0; material scenes need no pointer/control asserts"
  - "Drawer focus wrap: Dismiss + 11 links = 12 focusables; reverse-tab last link is Rigid Particles"

requirements-completed: [MAT-01, MAT-02, MAT-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T06:09:24Z

duration: 9min
completed: 2026-09-22
---

# Phase 27 Plan 06: Chromium Material Smoke Summary

**Extended Chromium `just web-player-smoke` to eleven scenes with watch-first Surface Tension / Elastic Particles / Rigid Particles play-pause-reset and green 38/38 gate**

## Performance

- **Duration:** 9 min
- **Started:** 2026-09-22T06:00:05Z
- **Completed:** 2026-09-22T06:09:24Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `surface-tension`, `elastic-particles`, and `rigid-particles` to `SCENE_HASH_PATHS` and raised `ALL_SCENE_TIMEOUT_MS` to `220_000`
- Desktop sidebar asserts eleven `Static preview` captions and eleven `svg[aria-hidden='true']`; drawer reverse-tab last link is Rigid Particles; `.catalog-card` stays 0
- Watch-first path (`controls.length === 0`) covers Particles, Liquid Timer, and the three material scenes without pointer/control requirements; interactive six keep gesture+control coverage
- `just web-player-smoke` exited 0: **38 passed, 0 failed** (Chromium). `MAX_ADVANCE_STEPS` remains 4

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend helpers and keep watch-first player e2e** - `4a022ac` (feat)
2. **Task 2: Eleven-preview shell asserts and run web-player-smoke** - `308e167` (feat)

**Plan metadata:** pending docs commit

## Files Created/Modified

- `web/e2e/player-helpers.ts` - Eleven hash paths, 220s timeout, `sessionStatus`, href-based desktop nav, `**/*.wasm*` gate
- `web/e2e/player.spec.ts` - Use scoped session status; watch-first/interactive split unchanged
- `web/e2e/shell.spec.ts` - Eleven previews, Rigid Particles wrap, eleven-demo footer copy
- `web/e2e/reset-honesty.spec.ts` - Scoped session status
- `scripts/web-build.ts` - `CI=true` for `test:player` under player-smoke

## Decisions Made

- Prefer `220_000` ms all-scene timeout for eleven open/reset loops rather than proving `160_000` still passes
- Force `CI=true` during player-smoke so Playwright starts production `vite preview` instead of reusing a local Vite dev server on port 4173
- Scope playback assertions to `.session-status` because wireframe stroke `<output>` and tilt-debug also expose status roles

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ambiguous getByRole("status") after tilt/wireframe chrome**
- **Found during:** Task 2 (`just web-player-smoke`)
- **Issue:** Strict-mode violation: three status roles (session, wireframe stroke output, tilt-debug)
- **Fix:** Added `sessionStatus(page)` → `.session-status` and updated player/shell/reset-honesty callers
- **Files modified:** `web/e2e/player-helpers.ts`, `web/e2e/player.spec.ts`, `web/e2e/shell.spec.ts`, `web/e2e/reset-honesty.spec.ts`
- **Verification:** Smoke progressed past status asserts
- **Committed in:** `308e167`

**2. [Rule 1 - Bug] Particles nav regex matched Elastic/Rigid Particles**
- **Found during:** Task 2 (opens-each-native-scene test)
- **Issue:** `getByRole('link', { name: /Particles/ })` matched three sidebar links
- **Fix:** `openDesktopDemo` clicks `a.demo-nav-link[href="#/scene/${id}"]`
- **Files modified:** `web/e2e/player-helpers.ts`
- **Verification:** Eleven-scene open/reset loop passed
- **Committed in:** `308e167`

**3. [Rule 3 - Blocking] Player-smoke reused Vite dev on :4173**
- **Found during:** Task 2 (Loading Dam Break gate never observed)
- **Issue:** `reuseExistingServer: !process.env.CI` attached to a leftover Vite dev server; WASM URLs with `?t=` bypassed `**/*.wasm`
- **Fix:** Broaden route to `**/*.wasm*`; set `CI=true` when player-smoke runs `test:player`
- **Files modified:** `web/e2e/player-helpers.ts`, `scripts/web-build.ts`
- **Verification:** `just web-player-smoke` → 38 passed
- **Committed in:** `308e167`

**4. [Rule 3 - Blocking] Drawer focus-trap assert raced under eleven links**
- **Found during:** Task 2 (shell reverse/forward tab trap)
- **Issue:** Immediate `contains(activeElement)` check returned false mid-Tab
- **Fix:** Use `expect.poll` around dialog containment after each Tab / Shift+Tab
- **Files modified:** `web/e2e/shell.spec.ts`
- **Verification:** Drawer trap test passed in full smoke
- **Committed in:** `308e167`

---

**Total deviations:** 4 auto-fixed (1 bug, 3 blocking)
**Impact on plan:** Required for a green Chromium gate; no scope expansion beyond MAT-01/02/03 proof

## Issues Encountered

- Local Vite on port 4173 caused the first smoke attempts to hit the wrong server; freed the port and forced CI for player-smoke

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 27 plans 01–06 have SUMMARY files; ready for phase verification / independent AI review (D-17 — implementing agent must not self-approve)
- Independent review remains eligible; no Pages redeploy and no Firefox/Safari matrix in this phase

## Self-Check: PASSED

- FOUND: `.planning/phases/27-material-flag-groups/27-06-SUMMARY.md`
- FOUND: commit `4a022ac`
- FOUND: commit `308e167`
- Smoke: `just web-player-smoke` → 38 passed, 0 failed

---
*Phase: 27-material-flag-groups*
*Completed: 2026-09-22*

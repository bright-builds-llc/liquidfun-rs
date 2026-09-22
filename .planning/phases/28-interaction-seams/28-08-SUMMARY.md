---
phase: 28-interaction-seams
plan: "08"
subsystem: web-testing
tags: [playwright, chromium, web-player-smoke, POINTER_CONTROL, ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]

requires:
  - phase: 28-interaction-seams
    provides: Sixteen-scene catalog, Vitest contract, and SCENE_CAPTURE_PLANS from Plans 06–07
provides:
  - Chromium player-smoke covering sixteen open/play/pause/reset scenes
  - Interactive gestures for Soup Stirrer rail toggle, Impulse click, Theo Jansen motor reverse
  - Shell sixteen-demo / Theo Jansen last-link / catalog-card=0 asserts
affects:
  - Phase 28 verification and independent AI review (D-26)

tech-stack:
  added: []
  patterns:
    - Watch-first scenes selected by controls.length === 0; POINTER_CONTROL Partial for interactive only
    - Catalog action buttons must call apply_action, not apply_control
    - Frame caps raised to 64 segments / 48 circles for Theo walker wireframe

key-files:
  created: []
  modified:
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
    - web/e2e/shell.spec.ts
    - crates/liquidfun-wasm/src/scene/soup_stirrer.rs
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs
    - web/src/physics/frame.ts
    - web/tests/frame.test.ts

key-decisions:
  - "Raised MAX_RIGID_SEGMENTS 16→64 and MAX_RIGID_CIRCLES 8→48 so Theo Jansen frame capture fits walker+balls"
  - "Routed Soup Stirrer toggle-paddle-rail through apply_action to match catalog action kind"
  - "Independent AI review remains eligible under D-26; implementing agent did not self-approve"

patterns-established:
  - "Pattern: Sixteen SCENE_HASH_PATHS + SELECT_NEXT_VALUE for Push/Motor direction"
  - "Pattern: Shell drawer wrap uses Dismiss+16 focusables with Theo Jansen as last link"

requirements-completed: [ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:46:01Z

duration: 10min
completed: 2026-09-22
---

# Phase 28 Plan 08: Chromium Sixteen-Scene Smoke Summary

**Chromium `just web-player-smoke` proves sixteen scenes open/play/pause/reset plus Stirrer/Impulse/Theo gestures; `.catalog-card` stays 0**

## Performance

- **Duration:** 10 min
- **Started:** 2026-09-22T14:36:12Z
- **Completed:** 2026-09-22T14:46:01Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Extended `SCENE_HASH_PATHS` and `POINTER_CONTROL` for Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen
- Updated shell e2e to sixteen-demo copy, sixteen Static preview captions, Theo Jansen last drawer link, and catalog-card count 0
- Fixed Soup Stirrer labeled toggle via `apply_action` and raised frame rigid caps so Theo Jansen capture succeeds
- `CI=true just web-player-smoke` exited 0 with 38 passed (32.1s Playwright run)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend helpers and player e2e for sixteen scenes + gestures** - `7f2efb2` (feat)
2. **Task 2: Update shell e2e and run Chromium web-player-smoke** - `4863ea2` (test)
3. **Task 2 follow-up: smoke failure fixes** - `56d7c03` (fix)

**Plan metadata:** `7c748e0` (docs: complete plan)

## Files Created/Modified

- `web/e2e/player-helpers.ts` - Five hash paths; Push/Motor direction select maps
- `web/e2e/player.spec.ts` - POINTER_CONTROL for stirrer/impulse/theo
- `web/e2e/shell.spec.ts` - Sixteen-demo asserts and Theo Jansen focus wrap
- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs` - `apply_action` for toggle-paddle-rail
- `crates/liquidfun-wasm/src/frame.rs` - Segment/circle caps 64/48
- `crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs` - Capture-success assert
- `web/src/physics/frame.ts` - Mirrored JS caps
- `web/tests/frame.test.ts` - Rejection thresholds updated

## Decisions Made

- Raised frame rigid caps rather than thinning Theo wireframe export so the walker and ground balls stay visible
- Kept `MAX_ADVANCE_STEPS` at 4; no particle-count cuts; no Firefox/Safari/Linux/Pages gates
- D-26: independent AI review remains eligible; this implementing agent must not self-approve

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Soup Stirrer action button called unknown apply_action**
- **Found during:** Task 2 (`CI=true just web-player-smoke`)
- **Issue:** Catalog defines `action("toggle-paddle-rail")` but WASM only handled it in `apply_control`, so the labeled button failed the session
- **Fix:** Implement toggle in `apply_action`; update unit tests to call `apply_action`
- **Files modified:** `crates/liquidfun-wasm/src/scene/soup_stirrer.rs`
- **Verification:** Soup Stirrer interactive smoke path passes in full player-smoke
- **Committed in:** `56d7c03`

**2. [Rule 1 - Bug] Theo Jansen frame capture exceeded rigid caps**
- **Found during:** Task 2 (`CI=true just web-player-smoke`)
- **Issue:** Walker exports ~43 segments and 41 circles; caps were 16 / 8, so `captureFrame` failed at scene start
- **Fix:** Raise `MAX_RIGID_SEGMENTS` to 64 and `MAX_RIGID_CIRCLES` to 48 in Rust and TS mirrors; assert capture in Theo unit test
- **Files modified:** `crates/liquidfun-wasm/src/frame.rs`, `web/src/physics/frame.ts`, `web/tests/frame.test.ts`, `crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs`
- **Verification:** Focused debug then full smoke green; Theo create+capture unit test passes
- **Committed in:** `56d7c03`

---

**Total deviations:** 2 auto-fixed (2× Rule 1)
**Impact on plan:** Required for ACT-02/ACT-05 browser proof; no scope creep beyond smoke green.

## Issues Encountered

- First smoke run failed on Theo open and Soup Stirrer control activation; diagnosed via Playwright failure details (`Rust/WASM frame capture failed` / action UnknownControl), fixed, and re-ran to green.

## Known Stubs

None.

## Threat Flags

None — no new trust-boundary surfaces beyond allowlisted scene routes and labeled controls already in the plan threat model.

## Smoke Evidence

```text
CI=true just web-player-smoke
… 38 passed (32.1s)
[web-build] complete player-smoke
EXIT=0
```

`MAX_ADVANCE_STEPS` remains 4. `.catalog-card` asserts remain `toHaveCount(0)`.

## Self-Check: PASSED

- FOUND: `web/e2e/player-helpers.ts`
- FOUND: `web/e2e/player.spec.ts`
- FOUND: `web/e2e/shell.spec.ts`
- FOUND: `.planning/phases/28-interaction-seams/28-08-SUMMARY.md`
- FOUND: commit `7f2efb2`
- FOUND: commit `4863ea2`
- FOUND: commit `56d7c03`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 28 Chromium smoke gate is green for ACT-01–ACT-05
- Independent AI review (D-26) may proceed; implementing agent did not self-approve
- Ready for phase verification / milestone handoff without Pages redeploy

---
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*

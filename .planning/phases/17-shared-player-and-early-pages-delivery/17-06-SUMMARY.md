---
phase: 17-shared-player-and-early-pages-delivery
plan: "06"
subsystem: testing
tags: [playwright, chromium, production-base, wasm, hidden-tab, web-02, web-03, web-06, wasm-04, host-03]

requires:
  - phase: 17-shared-player-and-early-pages-delivery
    provides: One-session Dam Break playground shell with Play/Pause/Reset/Retry
provides:
  - Production-base Chromium player spec for Dam Break, fallback, reset, and hidden-tab bounds
  - just web-player-smoke recipe that rebuilds then runs player Playwright
  - Phase 16 forensic smoke skipped unless PHASE16_CLOSURE_ATTEMPT_DIR is set
affects: [17-07, 17-08, pages-delivery, player-smoke]

tech-stack:
  added: []
  patterns: [playwright-baseURL-origin-plus-project-path, opt-in-phase16-forensic-skip, hidden-tab-defineProperty]

key-files:
  created:
    - web/e2e/player.spec.ts
  modified:
    - web/playwright.config.ts
    - web/e2e/rust-wasm-proof.spec.ts
    - web/package.json
    - justfile
    - scripts/web-build.ts
    - web/src/App.tsx

key-decisions:
  - "Player smoke is a web-build player-smoke mode that reuses the existing build and does not allocate a Phase 16 closure attempt."
  - "Hidden-tab proof uses Object.defineProperty plus a MutationObserver so the first resume frame is measured, not a later poll."

patterns-established:
  - "Playwright baseURL stays http://127.0.0.1:4173 and goto uses /liquidfun-rs/#/scene/... so leading slashes cannot drop the project prefix."
  - "Default Chromium suite is the product player spec; Phase 16 Dispose-session forensic smoke is opt-in via PHASE16_CLOSURE_ATTEMPT_DIR."
  - "Dam Break starts only after the canvas reports a usable CSS viewport, including hash returns from fallback."

requirements-completed: [WEB-02, WEB-03, WEB-06, WASM-04, HOST-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T12:16:25Z

duration: 6min
completed: 2026-09-17
---

# Phase 17 Plan 06: Production-Base Player Smoke Summary

**Chromium player smoke on `/liquidfun-rs/#/scene/dam-break` that proves play/pause/reset, unknown-hash fallback, Fountain leave without a second WASM session, and hidden-tab catch-up capped at 4 steps.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-09-17T12:10:12Z
- **Completed:** 2026-09-17T12:16:25Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Default Playwright no longer requires `PHASE16_CLOSURE_ATTEMPT_DIR`; the Phase 16 forensic spec skips unless that env is set.
- `web/e2e/player.spec.ts` opens `/liquidfun-rs/#/scene/dam-break`, asserts a `.wasm` request under `/liquidfun-rs/`, and exercises Pause, Play, Reset, unknown-hash fallback, Fountain not-ready leave, and hidden-tab bounding.
- `just web-player-smoke` rebuilds WASM and the production site, installs package-pinned Chromium, and runs only the player spec. `just web-smoke` remains the opt-in Phase 16 forensic path.
- Dam Break now waits for a usable canvas viewport and starts after hash return from fallback, so production-base preview no longer fails on first layout.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the production-base player spec** - `15c2394` (test)
1. **Task 2: Add a player-smoke recipe and run it** - `ed940f7` (feat)

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `web/e2e/player.spec.ts` — Production-base Dam Break, fallback, reset, Fountain leave, and hidden-tab proofs
- `web/playwright.config.ts` — Origin `baseURL`, preview URL `/liquidfun-rs/`, no required Phase 16 attempt dir
- `web/e2e/rust-wasm-proof.spec.ts` — Skips unless `PHASE16_CLOSURE_ATTEMPT_DIR` is set
- `web/package.json` — `test:player` script
- `justfile` — `web-player-smoke` recipe
- `scripts/web-build.ts` — `player-smoke` mode that reuses build and does not allocate `closure-attempt-N`
- `web/src/App.tsx` — Start Dam Break after a usable viewport and when returning from fallback

## Decisions Made

- Player smoke is a `web-build` `player-smoke` mode that reuses the existing build and does not allocate a Phase 16 closure attempt. Forensic `just web-smoke` still sets `PHASE16_CLOSURE_ATTEMPT_DIR`.
- Hidden-tab proof uses `Object.defineProperty(document, "hidden", { configurable: true, get })` plus `visibilitychange`, then a `MutationObserver` on `data-step-index` so the first resume frame is measured instead of a later Playwright poll.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Start Dam Break only after the canvas has a usable size**
- **Found during:** Task 2 (Add a player-smoke recipe and run it)
- **Issue:** `assignCanvas` called `resizeCanvasBackingStore` on a 0×0 first layout, failed the session, and never connected a successful start. Production preview showed `Dam Break failed` with the HTML 960×540 backing size left unchanged.
- **Fix:** Connect `ResizeObserver` first and start only when CSS width/height are positive and finite.
- **Files modified:** `web/src/App.tsx`
- **Verification:** Diagnostic preview showed `Playing` and a resized backing store; `just web-player-smoke` exits 0
- **Committed in:** `ed940f7` (Task 2 commit)

**2. [Rule 1 - Bug] Start Dam Break after hash return from fallback**
- **Found during:** Task 2 (Add a player-smoke recipe and run it)
- **Issue:** Open Dam Break left `view` as `fallback`, so the observer refused to start and the player stayed on `Loading Dam Break…`.
- **Fix:** Enter `loading` on dam-break hash arrival and start when the route is dam-break and no session exists.
- **Files modified:** `web/src/App.tsx`
- **Verification:** Unknown-hash test reaches `Playing` after `Open Dam Break`
- **Committed in:** `ed940f7` (Task 2 commit)

**3. [Rule 1 - Bug] Measure the first hidden-tab resume frame in-page**
- **Found during:** Task 2 (Add a player-smoke recipe and run it)
- **Issue:** Playwright `expect.poll` sampled a later animation callback and saw a step delta of 5, which is not a catch-up storm but failed the "next frame ≤ 4" assertion.
- **Fix:** Install a `MutationObserver` and restore visibility in one `page.evaluate` so the first `data-step-index` mutation is the measured frame.
- **Files modified:** `web/e2e/player.spec.ts`
- **Verification:** Hidden-tab test passes; `just web-player-smoke` exits 0
- **Committed in:** `ed940f7` (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (3 bugs)
**Impact on plan:** Required for production-base smoke to pass. No six-scene WEBTEST-01 scope or Pages workflow was added.

## Issues Encountered

None beyond the auto-fixed startup and measurement races above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 17-07. Local Chromium proves D-17 player proofs on `/liquidfun-rs/`. Playwright stays in `just web-player-smoke`; do not add it to the Pages job. Phase 16 `just web-smoke` remains opt-in forensic and still expects the retired Dispose-session chrome if invoked.

## Self-Check: PASSED

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

---
phase: 17-shared-player-and-early-pages-delivery
plan: "05"
subsystem: player
tags: [solidjs, wasm, hash-routing, playback, generation-token, wasm-04, web-02, web-03, web-06]

requires:
  - phase: 17-shared-player-and-early-pages-delivery
    provides: Allowlisted hash parser, bounded clock, and presentational playground chrome
provides:
  - Pure nextGeneration and isStaleGeneration helpers
  - One-session Dam Break playground shell with Play/Pause/Reset/Retry
  - Hidden-tab pause with cleared catch-up and max 4 steps per frame
  - Hash-route fallback with no WASM world except dam-break
affects: [17-06, 17-07, player-smoke, pages-delivery]

tech-stack:
  added: []
  patterns: [generation-token stale-load discard, one-session rAF owner, hidden-tab timestamp reset]

key-files:
  created:
    - web/src/player/generation.ts
    - web/tests/generation.test.ts
    - web/src/player/observe.ts
  modified:
    - web/src/App.tsx
    - web/src/components/CatalogNav.tsx
    - web/src/components/PlayerPanel.tsx
    - web/src/physics/loader.ts
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/lib.rs

key-decisions:
  - "Reuse loadProofSession/ProofSession as named Dam Break instead of renaming the generated class."
  - "Extract observeFrame helpers so App.tsx stays under the 628-line file trigger."
  - "Allow explicit undefined on optional chrome props for exactOptionalPropertyTypes."

patterns-established:
  - "Functional-core generation tokens decide stale async loads before the shell keeps a WASM world."
  - "The Solid shell owns exactly one SceneSession and one rAF id; reset, leave, and cleanup increment generation and dispose."
  - "Hidden-tab suspension keeps the Playing label, clears maybeLastTimestamp, and never steps more than 4 ticks."

requirements-completed: [WASM-04, WEB-02, WEB-03, WEB-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T12:07:27Z

duration: 7min
completed: 2026-09-17
---

# Phase 17 Plan 05: One-Session Dam Break Player Summary

**One-session Dam Break playground that loads from `#/scene/dam-break`, controls Play/Pause/Reset/Retry, discards stale WASM loads, and bounds hidden-tab catch-up to 4 steps.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-17T11:59:37Z
- **Completed:** 2026-09-17T12:07:27Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Pure generation-token helpers make later start/reset/retry/route changes discard in-flight `loadProofSession()` results.
- The Phase 16 proof page is replaced by playground chrome: catalog, Dam Break player or honest fallback, and site footer.
- Play resumes stepping, Pause freezes without dispose, and Reset/Retry dispose then recreate the documented 192-particle basin.
- Hidden tabs keep the Playing label, clear the timestamp accumulator, and call `nextFrame(n)` at most once with n in 1–4.

## Task Commits

Each task was committed atomically:

1. **Task 1: Test generation-token stale-load discard**
   - `3bf6d2f` (`test`) — failing increment, stale, and dispose-decision cases
   - `6024481` (`feat`) — `nextGeneration` and `isStaleGeneration`
1. **Task 2: Wire the one-session Dam Break playground shell** - `94a9eba` (feat)

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `web/src/player/generation.ts` — Pure generation increment and stale-load decision
- `web/tests/generation.test.ts` — Increment, matching, stale, and dispose-decision cases
- `web/src/player/observe.ts` — Frame movement observation extracted from the proof shell
- `web/src/App.tsx` — One-session hash-routed Dam Break player
- `web/src/components/CatalogNav.tsx` — Optional current-scene prop accepts explicit undefined
- `web/src/components/PlayerPanel.tsx` — Optional details prop accepts explicit undefined
- `web/src/physics/loader.ts` — Documents `loadProofSession` as the named Dam Break constructor
- `crates/liquidfun-wasm/src/scene.rs` — Documents Dam Break reset constants without retuning them
- `crates/liquidfun-wasm/src/lib.rs` — Names the generated constructor as Dam Break

## Decisions Made

- Keep the generated `ProofSession` class name and reuse `loadProofSession()` as Dam Break; document the existing basin constants as the reset target.
- Split frame-observation helpers out of `App.tsx` so the shell stays well under the 628-line trigger.
- Widen optional chrome props with `| undefined` so Solid can pass absent current-scene and failure-details values under `exactOptionalPropertyTypes`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Accept explicit undefined on optional chrome props**
- **Found during:** Task 2 (Wire the one-session Dam Break playground shell)
- **Issue:** `exactOptionalPropertyTypes` rejected `maybeCurrentSceneId` and `maybeDetails` when the shell passed `undefined`.
- **Fix:** Typed those optional props as `T | undefined` so absent values typecheck without extra JSX branches.
- **Files modified:** `web/src/components/CatalogNav.tsx`, `web/src/components/PlayerPanel.tsx`
- **Verification:** `cd web && bun run typecheck` exits 0
- **Committed in:** `94a9eba` (Task 2 commit)

**2. [Rule 3 - Blocking] Document Dam Break reset constants on the existing constructor**
- **Found during:** Task 2 (Wire the one-session Dam Break playground shell)
- **Issue:** The WASM wrapper still only exposed a proof-scene constructor; the plan and user instruction require a named Dam Break with documented reset constants.
- **Fix:** Documented the existing gravity, 192-particle basin, circle, cap, and 1/60 timestep constants. Did not retune values or add the other five scenes.
- **Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/lib.rs`, `web/src/physics/loader.ts`
- **Verification:** `just web-build` exits 0
- **Committed in:** `94a9eba` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required for typecheck and the named Dam Break constructor. No extra physics scenes were added.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 17-06. Playwright selectors can target Play/Pause/Reset/Retry, `data-playback`, `data-scene`, and `#/scene/dam-break`. Do not run Phase 16 `just web-smoke` until that plan updates proof-page selectors.

## Self-Check: PASSED

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

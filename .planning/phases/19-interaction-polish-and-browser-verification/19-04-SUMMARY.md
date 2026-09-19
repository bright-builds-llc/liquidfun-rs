---
phase: 19-interaction-polish-and-browser-verification
plan: "04"
subsystem: pointer-input
tags: [pointer-events, canvas, solidjs, wasm, tdd]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: CSS-to-world inverse, captured-gesture reducer, and WASM pointer_action from 19-01 through 19-03
provides:
  - attachCanvasPointer capture/release adapter
  - SceneSession.pointerAction finite-kind gate
  - Canvas-only touch-action none and teardown cancel
affects: [19-05, 19-06, canvas-pointer, wasm-pointer-action]

tech-stack:
  added: []
  patterns:
    - one Pointer Events pipeline on the player canvas
    - parse pointer kinds before generated pointerAction
    - cancel capture on pause, reset, switch, fail, hidden, and cleanup

key-files:
  created:
    - web/src/input/canvas-pointer.ts
  modified:
    - web/src/physics/session.ts
    - web/tests/session.test.ts
    - web/src/App.tsx
    - web/src/app.css

key-decisions:
  - "TypeScript forwards only allowlisted finite pointer samples and poisons only on generated failure."
  - "One canvas Pointer Events adapter captures, unprojects through the live camera, and cancels on every teardown path."
  - "touch-action: none stays on canvas only so page chrome keeps native scroll."

patterns-established:
  - "Device pixels never enter physics; getBoundingClientRect plus unprojectPoint is the only hit-test path."
  - "App.tsx pointer glue stays in canvas-pointer.ts so the shell remains at the 628-line cap."

requirements-completed: [WEB-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:20:08Z

duration: 4min
completed: 2026-09-19
---

# Phase 19 Plan 04: Canvas Pointer Pipeline and Teardown Cancel Summary

**One canvas Pointer Events adapter converts CSS bounds through the live camera into WASM `pointerAction`, and every teardown path releases capture and emits cancel.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-19T02:15:46Z
- **Completed:** 2026-09-19T02:20:08Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `SceneSession.pointerAction` forwards `down|move|up|cancel` with finite world coordinates and poisons only when generated throws.
- Unknown kinds and NaN return without calling generated or disposing the owner.
- `attachCanvasPointer` listens for the five Pointer Events, `setPointerCapture` on down, and unprojects through `cssPointFromClient` plus `unprojectPoint`.
- Pause, Reset/Retry (`startScene`), scene leave, fail, hidden-tab, Apply setting recreate, and Solid cleanup cancel leftover capture.
- `touch-action: none` and `cursor: crosshair` apply only to the player canvas; `App.tsx` stays at 628 lines.

## Task Commits

Each task was committed atomically:

1. **Task 1: Forward pointerAction on the TypeScript session owner**
   - `bc99fee` (test): failing forward, reject, and poison tests
   - `76753be` (feat): `pointerAction` on the TypeScript owner
2. **Task 2: Attach canvas capture and cancel on every teardown path**
   - `305d5e6` (feat): `canvas-pointer.ts`, App wiring, canvas CSS

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `web/src/physics/session.ts` — `pointerAction` parse-and-forward gate
- `web/tests/session.test.ts` — Fake `pointerActionCalls` and owner tests
- `web/src/input/canvas-pointer.ts` — capture, unproject, cancel, and send helpers
- `web/src/App.tsx` — assign/cancel wiring and `data-last-pointer-kind` / `data-pointer-accepted`
- `web/src/app.css` — canvas `touch-action: none` and `.canvas-interactive` crosshair

## Decisions Made

- Keep one session owner: parse with `parsePointerKind` and reject without poisoning.
- Honor capture/release even when CSS, camera, or world is missing so leftover capture cannot stick.
- Extract `forwardScenePointer` and `syncCanvasInteractive` into `canvas-pointer.ts` so `App.tsx` stays at the 628-line trigger.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Regenerated stale wasm-pack types**
- **Found during:** Task 1
- **Issue:** Local generated `ProofSession` lacked `pointerAction`, so `bun run typecheck` failed after the owner interface gained the method.
- **Fix:** Ran `just web-wasm` to regenerate ignored bindings from the current checkout.
- **Files modified:** `web/src/generated/liquidfun-wasm/` (gitignored)
- **Verification:** `cd web && bun run typecheck` exits 0
- **Committed in:** none (generated output is ignored)

**2. [Rule 2 - Missing Critical] Cancel before Apply setting recreate**
- **Found during:** Task 2
- **Issue:** Construction recreate does not go through `startScene`, so an in-flight drag/stir could survive a world rebuild.
- **Fix:** Call `maybeCanvasPointer?.cancel()` before `setView({ kind: "loading" })` in `applySceneControl`.
- **Files modified:** `web/src/App.tsx`
- **Verification:** `rg -n "maybeCanvasPointer\\?\\.cancel" web/src/App.tsx` matches pause, dispose, apply, and hidden paths
- **Committed in:** `305d5e6`

**3. [Rule 3 - Blocking] Extracted send/cursor helpers to keep App.tsx at 628 lines**
- **Found during:** Task 2
- **Issue:** Inlined send and cursor wiring pushed `App.tsx` to 650 lines.
- **Fix:** Moved `forwardScenePointer` and `syncCanvasInteractive` into `canvas-pointer.ts`.
- **Files modified:** `web/src/input/canvas-pointer.ts`, `web/src/App.tsx`
- **Verification:** `wc -l web/src/App.tsx` prints 628
- **Committed in:** `305d5e6`

---

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** Required for typecheck, leftover-capture correctness, and the locked file-length cap. No scope creep.

## Issues Encountered

Local wasm-pack output was older than the 19-02 `pointerAction` export. Regenerating ignored bindings unblocked typecheck; no source commit was needed for that output.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-05 figcaption, select focus, and 480px polish. The canvas pipeline and teardown cancel are in place. WEB-05 remains pending until 19-06 Chromium pointer/resize/cleanup proofs. Do not treat this plan as player-smoke or Pages evidence.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: web/src/input/canvas-pointer.ts
- FOUND: web/src/physics/session.ts
- FOUND: web/src/App.tsx
- FOUND: web/src/app.css
- FOUND: web/tests/session.test.ts
- FOUND: bc99fee
- FOUND: 76753be
- FOUND: 305d5e6

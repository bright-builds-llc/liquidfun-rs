---
phase: 19-interaction-polish-and-browser-verification
plan: "01"
subsystem: pointer-input
tags: [camera, unproject, pointer-events, vitest, tdd, solidjs]

requires:
  - phase: 18-six-native-physics-demos
    provides: Shared createCamera / projectPoint CSS fit for the six-scene player
provides:
  - unprojectPoint inverse of projectPoint on WORLD_BOUNDS
  - parsePointerKind allowlist for down|move|up|cancel
  - cssPointFromClient CSS-pixel parser that rejects non-finite or empty rects
  - Captured-gesture reducer with one maybePointerId
affects: [19-02, 19-03, 19-04, canvas-pointer, wasm-pointer-action]

tech-stack:
  added: []
  patterns:
    - one CSS transform for draw and hit-test
    - parse pointer kind and finite CSS points at the TypeScript boundary
    - captured-gesture reducer without DOM

key-files:
  created:
    - web/src/input/pointer.ts
    - web/tests/pointer.test.ts
  modified:
    - web/src/render/camera.ts
    - web/tests/camera.test.ts

key-decisions:
  - "Invert projectPoint with WORLD_BOUNDS on the shared Camera; never use canvas.width or devicePixelRatio."
  - "cssPointFromClient returns CSS pixels only and rejects non-finite samples and empty rects."
  - "reducePointerEvent stores one maybePointerId, ignores uncaptured moves, and clears on up, cancel, and lostpointercapture without reading isTrusted."

patterns-established:
  - "Hit-test uses the same Camera as drawing."
  - "Pointer kinds and CSS points fail closed before any session call."
  - "Capture and cancel decisions live in a DOM-free reducer."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:04:10Z

duration: 2min
completed: 2026-09-19
---

# Phase 19 Plan 01: CSS-to-world Inverse and Pointer Reducer Summary

**Pure CSS→world camera inverse and captured-gesture reducer with Vitest coverage, so later player wiring cannot invent a second coordinate space or a mouse+touch stack.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-19T02:02:17Z
- **Completed:** 2026-09-19T02:04:10Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `unprojectPoint` inverts `projectPoint` for basin corners and `{ x: 0, y: 3.5 }` on 960×540 and 320×540 cameras.
- Camera source stays in CSS pixels; no `canvas.width` or `devicePixelRatio` terms.
- `parsePointerKind` allowlists only `down|move|up|cancel`.
- `cssPointFromClient` converts client coordinates to CSS pixels and rejects NaN, Infinity, and empty rects.
- `reducePointerEvent` captures on down, emits move only for the stored id, and clears on up, cancel, and lostpointercapture.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add unprojectPoint as the inverse of projectPoint**
   - `14cd332` (test): failing inverse round-trip tests
   - `b2fbd19` (feat): `unprojectPoint` using WORLD_BOUNDS
2. **Task 2: Parse pointer kinds and reduce capture or cancel**
   - `0194382` (test): failing kind, CSS-point, and reducer tests
   - `f4d72c4` (feat): DOM-free parser and captured-gesture reducer

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `web/src/render/camera.ts` — `unprojectPoint` inverse of `projectPoint`
- `web/tests/camera.test.ts` — inverse round-trips on design and narrow cameras
- `web/src/input/pointer.ts` — kind parser, CSS point parser, gesture reducer
- `web/tests/pointer.test.ts` — Arrange/Act/Assert coverage for parse and reduce

## Decisions Made

- Invert the existing `Camera` with `WORLD_BOUNDS`; do not add a second camera type or accept backing-store pixels.
- Keep pointer decisions in a DOM-free module: no listeners, no `mousedown`/`touchstart`, no `isTrusted`.
- Store one `maybePointerId`. Ignore uncaptured moves. Treat `lostpointercapture` with empty state as a no-op so cleanup cannot emit a second cancel.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-02 WASM `pointer_action`. Hit-testing can invert the same camera used to draw. Kind, CSS point, and capture-or-cancel decisions are unit-tested without a browser. WEB-05 remains pending until later plans wire scene gestures, canvas capture, and browser proof.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: web/src/render/camera.ts
- FOUND: web/tests/camera.test.ts
- FOUND: web/src/input/pointer.ts
- FOUND: web/tests/pointer.test.ts
- FOUND: 14cd332
- FOUND: b2fbd19
- FOUND: 0194382
- FOUND: f4d72c4

---
phase: 19-interaction-polish-and-browser-verification
plan: "02"
subsystem: pointer-input
tags: [wasm, pointer-action, scene-hooks, tdd, rust]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: CSS-to-world inverse and captured-gesture reducer from 19-01
provides:
  - PointerKind parse and SceneHooks::apply_pointer
  - wasm-bindgen pointer_action on ProofSession
  - Dam Break captured obstacle drag
  - Fountain pointer aim from the nozzle
  - Float or Sink click-drop at world x
affects: [19-03, 19-04, wasm-pointer-action, canvas-pointer]

tech-stack:
  added: []
  patterns:
    - parse pointer kind and finite world coordinates at SessionCore
    - one apply_pointer method on every SceneHooks impl
    - compile-only stubs for force-range scenes until 19-03

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/src/scene/dam_break.rs
    - crates/liquidfun-wasm/src/scene/fountain.rs
    - crates/liquidfun-wasm/src/scene/float_or_sink.rs
    - crates/liquidfun-wasm/src/scene/color_mixer.rs
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/scene/water_wheel.rs

key-decisions:
  - "Parse down|move|up|cancel and finite f32 values in SessionCore before any World mutation."
  - "Color Mixer, Jelly Drop, and Water Wheel compile with no-op apply_pointer until 19-03."
  - "Dam Break captured drag clamps to the basin; Fountain aims from the nozzle; Float or Sink drops at clamped x and y=6.0."

patterns-established:
  - "Unknown pointer kinds reuse UnknownControl; non-finite coordinates are InvalidPointer."
  - "Rejected kind or NaN does not dispose or poison a fresh session."
  - "Labeled Drop/Reset, Aim angle, and Drop body stay as the keyboard path."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:09:24Z

duration: 5min
completed: 2026-09-19
---

# Phase 19 Plan 02: WASM pointer_action and Three Scene Gestures Summary

**Shared `pointer_action(kind, x, y)` boundary with native Dam Break drag, Fountain nozzle aim, and Float or Sink click-drop, so world coordinates stay inside Rust scene controllers.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-19T02:04:36Z
- **Completed:** 2026-09-19T02:09:24Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- `parse_pointer_kind` allowlists `down|move|up|cancel` and rejects `Down` and `click` as `UnknownControl`.
- Non-finite coordinates return `InvalidPointer` before any World mutation; Dam Break particle count stays 192.
- `ProofSession::pointer_action` is exported as `pointerAction`.
- Dam Break captured drag relocates `circle_body` with basin clamps; `Up` wakes; `Cancel` leaves the last pose.
- Fountain down+move aims from `(0, 0.5)` with `atan2` clamped to `±TAU/4`; labeled Aim angle still applies live.
- Float or Sink pointer-down drops at `(clamped x, 6.0)` and still caps at `MAX_DROPPED_BODIES = 4`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add PointerKind, apply_pointer, and pointer_action**
   - `0266e27` (test): failing parse, NaN, and export tests
   - `5605608` (feat): shared boundary plus compile stubs
2. **Task 2: Map Dam Break drag, Fountain aim, and Float or Sink drop**
   - `b3b6017` (test): failing native gesture tests
   - `45ea20f` (feat): three live mappings
   - `6d714ff` (refactor): rustfmt on the shared boundary and stubs

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene.rs` — `PointerKind`, `parse_pointer_kind`, `SceneHooks::apply_pointer`
- `crates/liquidfun-wasm/src/session.rs` — `InvalidPointer` and `SessionCore::apply_pointer`
- `crates/liquidfun-wasm/src/lib.rs` — wasm-bindgen `pointerAction`
- `crates/liquidfun-wasm/src/scene/dam_break.rs` — captured obstacle drag
- `crates/liquidfun-wasm/src/scene/fountain.rs` — pointer aim from the nozzle
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` — `drop_body_at` and click-drop
- `crates/liquidfun-wasm/src/scene/color_mixer.rs` — compile-only `apply_pointer` stub
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — compile-only `apply_pointer` stub
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — compile-only `apply_pointer` stub

## Decisions Made

- Parse kind and finite coordinates at `SessionCore` so a rejected sample never reaches `World`.
- Keep Color Mixer, Jelly Drop, and Water Wheel compiling with `Ok(())` stubs so a later player click is not `UnknownControl`.
- Use captured Dam Break drag, Fountain `atan2` from the nozzle, and Float or Sink drop at world x with `y = 6.0`.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

- `crates/liquidfun-wasm/src/scene/color_mixer.rs` `apply_pointer` — matches all four kinds and returns `Ok(())` with no mutation. Plan 19-03 replaces this with stir forces.
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` `apply_pointer` — same compile stub. Plan 19-03 replaces this with a location poke.
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` `apply_pointer` — same compile stub. Plan 19-03 replaces this with jet aim.

These stubs are required by the plan so the crate compiles without poisoning later player clicks.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-03 Color Mixer stir, Jelly poke, and Water Wheel jet aim. The shared `pointer_action` contract is native-tested. WEB-05 remains pending until later plans wire canvas capture and the remaining three gestures.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: crates/liquidfun-wasm/src/lib.rs
- FOUND: crates/liquidfun-wasm/src/scene/dam_break.rs
- FOUND: crates/liquidfun-wasm/src/scene/fountain.rs
- FOUND: crates/liquidfun-wasm/src/scene/float_or_sink.rs
- FOUND: 0266e27
- FOUND: 5605608
- FOUND: b3b6017
- FOUND: 45ea20f
- FOUND: 6d714ff

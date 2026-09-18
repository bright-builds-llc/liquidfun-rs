---
phase: 18-six-native-physics-demos
plan: "03"
subsystem: wasm-scene
tags: [liquidfun-wasm, float-or-sink, buoyancy, tdd, particle-body]

requires:
  - phase: 18-six-native-physics-demos
    provides: SceneId factory routing FloatOrSink to float_or_sink::build(presets)
provides:
  - Native Float or Sink water pool with reusable basin geometry
  - Live cork/wood/stone density presets and drop-body action
  - Cork-versus-stone y-separation from engine particle-body coupling
affects: [18-04, 18-08, 18-09, catalog-ready, player-controls]

tech-stack:
  added: []
  patterns:
    - live density preset stored on SceneHooks
    - drop-body creates a public fixture without recreating the world
    - native honesty test compares body y after equal step counts

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/float_or_sink.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Keep build(presets) on Float or Sink and ignore the bag; density is live on drop-body."
  - "Drop a dynamic circle at (0, 6) with fixture densities 0.3/0.6/2.0 against particle density 1.0."
  - "Cork-versus-stone y-separation is proven by native coupling; no engine API expansion."

patterns-established:
  - "Wave-2 scene files own their tests so later plans do not edit them."
  - "Body presets return ControlEffect::Live; construction names stay UnknownControl."

requirements-completed: [DEMO-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:30:14Z

duration: 10min
completed: 2026-09-18
---

# Phase 18 Plan 03: Float or Sink Native Spike Summary

**Native 180-particle water pool with live cork/wood/stone density drops that separate in y from particle-body coupling, not fake buoyancy.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-09-18T04:20:01Z
- **Completed:** 2026-09-18T04:30:14Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `SessionCore::create(SceneId::FloatOrSink)` builds a teal `WATER` pool of 180 particles, three basin segments, and no circles until a drop.
- After 120 native steps from spawn `(0, 6)`, cork world y stays above stone world y using only engine particle-body impulses.
- `apply_control("body", "cork"|"wood"|"stone")` returns `ControlEffect::Live`; `drop-body` creates one dynamic circle at the locked density without rebuilding the pool.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the pool and prove cork versus stone y-separation**
   - `a4d40df` (test): failing factory, budget, and y-separation tests
   - `d0a8d50` (feat): native pool, live density preset, drop-body, stub-list update
2. **Task 2: Apply body preset and drop-body without reset**
   - `857c429` (test): live preset, sequential cork-then-stone densities, unknown construction controls

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: Task 2 TDD tests were already green because Task 1 needed live drop-body for the honesty test._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` — pool builder, density allowlist, drop-body, native honesty tests
- `crates/liquidfun-wasm/src/session.rs` — Float or Sink removed from unimplemented stub-scene list

## Decisions Made

- Keep `build(presets)` from Plan 01 even though this plan's snippet showed `build()`. The bag is ignored because density is a live drop preset, not a construction reset.
- Use a 15×12 = 180 particle pool, radius `0.2`, spacing `0.32`, density `1.0`, drop circle radius `0.5` at `(0, 6)`, default Wood `0.6`.
- First composition separated cork above stone after 120 steps; no density/count retune and no public `liquidfun` API change.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Float or Sink left the unimplemented stub list**
- **Found during:** Task 1 (native pool implementation)
- **Issue:** `create_stub_scenes_fail_closed_without_a_live_world` still expected `SceneId::FloatOrSink` to return `SceneUnimplemented`.
- **Fix:** Removed Float or Sink from that stub list so Dam Break and remaining stubs still fail closed.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib -- --test-threads=1` exits 0
- **Committed in:** `d0a8d50`

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required so the factory can construct Float or Sink without breaking Plan 01 stub tests. No scope creep.

## Issues Encountered

- Task 2 RED tests compiled and passed on the first run because Task 1 already implemented live `body` and `drop-body` for the cork-versus-stone honesty test. New tests still lock sequential fixture densities `0.3` then `2.0` and reject `water-amount` / `mix-strength`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-04 Jelly Drop spike. Float or Sink is a native world with live density/drop controls. Do not mark the catalog scene ready until later chrome plans wire it. Do not add pointer input.

***
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/float_or_sink.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: a4d40df
- FOUND: d0a8d50
- FOUND: 857c429

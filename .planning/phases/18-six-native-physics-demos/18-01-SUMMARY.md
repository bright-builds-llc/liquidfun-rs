---
phase: 18-six-native-physics-demos
plan: "01"
subsystem: wasm-session
tags: [liquidfun-wasm, scene-factory, wasm-bindgen, solidjs, tdd]

requires:
  - phase: 17-shared-player-and-early-pages-delivery
    provides: Dam Break-only ProofSession constructor and loadProofSession wrapper
provides:
  - Allowlisted SceneId parse for the six locked hyphenated tokens
  - SceneHooks factory with extracted Dam Break Medium/Normal world
  - Fail-closed stub build(presets) modules for the other five scenes
  - ProofSession::new(scene_id) plus apply_control/apply_action
  - loadSceneSession(sceneId) with Dam Break loadProofSession wrapper
affects: [18-02, 18-03, 18-04, 18-05, 18-06, 18-07, 18-08, scene-modules, player-loader]

tech-stack:
  added: []
  patterns:
    - checked scene-id factory before World::new
    - SceneHooks dispatch with construction-preset bag
    - fail-closed unimplemented scene stubs

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/dam_break.rs
    - crates/liquidfun-wasm/src/scene/fountain.rs
    - crates/liquidfun-wasm/src/scene/float_or_sink.rs
    - crates/liquidfun-wasm/src/scene/color_mixer.rs
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/scene/water_wheel.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - web/src/physics/loader.ts

key-decisions:
  - "parse_scene_id accepts only the six lowercase hyphenated tokens and does not coerce case."
  - "SessionCore stores a Vec<(String, String)> preset bag and rebuilds on ControlEffect::Recreated so later scene files do not edit session.rs."
  - "Native ProofSession error-path tests use build_core because wasm-bindgen JsError cannot be constructed on non-wasm targets."
  - "Keep build(presets) on every scene module, including stubs that ignore the bag."

patterns-established:
  - "scene.rs plus scene/*.rs factory; no scene/mod.rs."
  - "capture_frame copies whatever segments and circles the active SceneHooks report."
  - "loadProofSession remains a dam-break wrapper around loadSceneSession."

requirements-completed: [DEMO-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:13:54Z

duration: 10min
completed: 2026-09-18
---

# Phase 18 Plan 01: Allowlisted Scene-Id Factory Summary

**Checked six-id WASM factory with extracted Dam Break Medium/Normal world, fail-closed scene stubs, and a TypeScript loader that can pass a scene id.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-09-18T04:03:28Z
- **Completed:** 2026-09-18T04:13:54Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- `parse_scene_id` maps only `dam-break`, `fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, and `water-wheel`.
- Dam Break still constructs gravity `(0, -10)`, 192 teal water particles, three basin segments, and one dynamic circle through the factory.
- Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel `build(presets)` return `SceneUnimplemented` before `World::new()`.
- `ProofSession::new(scene_id)` plus `apply_control`/`apply_action` use fixed `&str` messages; `loadSceneSession` passes the id and `loadProofSession` still wraps Dam Break for `App.tsx`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Parse SceneId and extract Dam Break behind a factory**
   - `04a0642` (test): failing factory parse/create tests
   - `fc50c1f` (feat): SceneId factory, Dam Break extract, fail-closed stubs
2. **Task 2: Export the scene-id constructor and apply_* WASM methods**
   - `9d194c7` (test): failing ProofSession constructor/control tests
   - `a482d00` (feat): scene-id constructor, apply methods, loadSceneSession

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene.rs` — SceneId parse, SceneHooks, build_scene, shared basin helper
- `crates/liquidfun-wasm/src/scene/dam_break.rs` — documented Medium/Normal Dam Break builder
- `crates/liquidfun-wasm/src/scene/fountain.rs` — fail-closed stub
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` — fail-closed stub
- `crates/liquidfun-wasm/src/scene/color_mixer.rs` — fail-closed stub
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — fail-closed stub
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — fail-closed stub
- `crates/liquidfun-wasm/src/session.rs` — factory-owned SessionCore, generalized capture, preset bag
- `crates/liquidfun-wasm/src/lib.rs` — `ProofSession::new(String)` and apply_* WASM methods
- `web/src/physics/loader.ts` — `loadSceneSession` plus Dam Break wrapper

## Decisions Made

- Keep one opaque `SessionCore` with `Box<dyn SceneHooks>` and a generic preset bag so wave-2 scene files only edit their own module.
- Do not lowercase caller input; `Dam-Break` is `UnknownScene`.
- Test WASM error mapping through native `build_core` because `JsError::new` panics on non-wasm targets.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Native JsError construction panics off wasm**
- **Found during:** Task 2
- **Issue:** `ProofSession::new("not-a-scene")` and failing `apply_*` call `JsError::new`, which panics on native `cargo test`.
- **Fix:** Extracted `build_core` for native error-path assertions; WASM methods still map through `js_error`.
- **Files modified:** `crates/liquidfun-wasm/src/lib.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib` exits 0
- **Committed in:** `a482d00`

**2. [Rule 3 - Blocking] Regenerated ignored wasm-pack output for typecheck**
- **Found during:** Task 2
- **Issue:** Generated `ProofSession` constructor was still zero-arg, so `new ProofSession(sceneId)` failed `tsc`.
- **Fix:** Ran `just web-wasm` to refresh ignored bindings. Not committed.
- **Files modified:** `web/src/generated/liquidfun-wasm/` (gitignored)
- **Verification:** `cd web && bun run typecheck` exits 0
- **Committed in:** n/a (generated)

**3. [Rule 2 - Missing Critical] Clippy-clean unused factory seams**
- **Found during:** Task 1
- **Issue:** `ControlEffect`, preset bag, and `store_preset` are required for later Recreated controls but unused in lib until those scenes ship.
- **Fix:** Kept `build(presets)` with `let _ = presets`; allowed dead-code on Recreated-only fields until later plans construct those variants.
- **Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/session.rs`, stub `build` functions
- **Verification:** `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings` exits 0
- **Committed in:** `fc50c1f`

***

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical)
**Impact on plan:** Required for native clippy/tests and TypeScript typecheck. No scope creep.

## Known Stubs

- `crates/liquidfun-wasm/src/scene/fountain.rs` — `build(presets)` returns `SceneUnimplemented` (Plan 07)
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` — `build(presets)` returns `SceneUnimplemented` (Plan 02)
- `crates/liquidfun-wasm/src/scene/color_mixer.rs` — `build(presets)` returns `SceneUnimplemented` (Plan 08)
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — `build(presets)` returns `SceneUnimplemented` (Plan 03)
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — `build(presets)` returns `SceneUnimplemented` (Plan 04)

These stubs are intentional fail-closed placeholders so later plans edit only their scene file.

## Issues Encountered

- wasm-bindgen `JsError` cannot be constructed during native lib tests; error paths are asserted through `build_core` / `SessionCore`.
- Regenerating ignored WASM bindings was required for `bun run typecheck` after the constructor arity change.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-02 Float or Sink spike. Factory, hooks, preset bag, and loader id passing are in place. Do not mark catalog scenes ready until their native worlds exist. `App.tsx` still special-cases Dam Break via `loadProofSession`.

***
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene.rs
- FOUND: crates/liquidfun-wasm/src/scene/dam_break.rs
- FOUND: crates/liquidfun-wasm/src/scene/fountain.rs
- FOUND: crates/liquidfun-wasm/src/scene/float_or_sink.rs
- FOUND: crates/liquidfun-wasm/src/scene/color_mixer.rs
- FOUND: crates/liquidfun-wasm/src/scene/jelly_drop.rs
- FOUND: crates/liquidfun-wasm/src/scene/water_wheel.rs
- FOUND: crates/liquidfun-wasm/src/lib.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: web/src/physics/loader.ts
- FOUND: 04a0642
- FOUND: fc50c1f
- FOUND: 9d194c7
- FOUND: a482d00

---
phase: 29-sparky-drawing-and-full-catalog
plan: "03"
subsystem: particles
tags: [sparky, powder, contact-transitions, particle-color, liquidfun-wasm]

requires:
  - phase: 29-sparky-drawing-and-full-catalog
    provides: World::set_particle_colors and the DrawingParticles scene id that Sparky follows
provides:
  - SceneId::Sparky builds six sparkable circles in the tall chamber
  - SessionCore calls SceneHooks::on_after_step with contact transitions after NoDecisionHook
  - Sparky spawns a 16-slot powder ring that fades and is destroyed at 0.75 s
affects:
  - 29-sparky-drawing-and-full-catalog

tech-stack:
  added: []
  patterns:
    - "Other scenes keep NoDecisionHook; only Sparky overrides on_after_step"
    - "Burst color is original channel times the half-life coefficient, written once per slot per step"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/sparky.rs
    - crates/liquidfun-wasm/src/scene/sparky/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Keep NoDecisionHook on both step paths and drain Begin transitions only after the world unlocks"
  - "Use ring length 16, splash radius 1.5 m, speed 15, and lifetime 0.75 s as the documented playground choices inside the upstream bands"
  - "The fixed horizontal stagger produced sparkable begins without an extra initial velocity"

patterns-established:
  - "SceneHooks::on_after_step defaults to a no-op so existing scenes stay unchanged"
  - "Powder lifetime is scene-owned seconds plus destroy_particle_group_particles, with destruction-by-age left off"

requirements-completed: [FX-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T06:57:23Z

duration: 35min
completed: 2026-09-27
---

# Phase 29 Plan 03: Sparky Post-Step Powder Bursts Summary

**Sparky throws a 16-slot ring of fading powder from new sparkable circle contacts after each step unlocks**

## Performance

- **Duration:** 35 min
- **Started:** 2026-09-27T06:22:33Z
- **Completed:** 2026-09-27T06:57:23Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `parse_scene_id("sparky")` maps to `SceneId::Sparky`, and construction exports six circles with zero particles.
- After `world.step` returns through `NoDecisionHook`, Sparky turns each sparkable `Begin` into one powder group. Persist and End contacts do not spark. Wall-only begins are ignored.
- Each burst fades in the Rust color lane from its stored color and is destroyed at 0.75 s. A full ring slot is destroyed and compacted before reuse. Particle count stayed under 4000 across 300 steps.
- Pointer input is a no-op. Unknown controls return `UnknownControl`. Destruction-by-age stays off. `MAX_ADVANCE_STEPS` stays 4, and the rigid caps stay 64 segments and 48 circles.

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Sparky allowlist and burst tests** - `d78b172` (test)
2. **Task 2: Drain begins after unlock and fade the ring** - `116d97b` (feat)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/sparky.rs` - Tall chamber, six sparkable circles, and the bounded powder ring
- `crates/liquidfun-wasm/src/scene/sparky/tests.rs` - Allowlist, powder lifetime, fade, bound, pointer, and age-destruction tests
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::Sparky`, parse token, build arm, and the default `on_after_step`
- `crates/liquidfun-wasm/src/session.rs` - Passes `contact_transitions()` after a successful step in `advance` and `advance_profiled`
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Scene list grows to nineteen ids with Sparky at index 18
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist pair for `"sparky"`

## Decisions Made

- Both step paths still pass `&mut NoDecisionHook`. The new hook runs only after `Ok(report)`, including `advance_profiled`.
- Splash radius 1.5 m, speed 15, and lifetime 0.75 s sit inside the upstream random bands. Ring length 16 is the recorded adaptation under upstream `c_maxVFX` of 50.
- The staggered circle centers produced a sparkable begin within 180 steps, so no circle needed an initial velocity.
- Fade writes `channel = original * coefficient` from the stored spawn color. Alpha stays 255. The recipe lifetime stays at the infinite default.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exposed a test-only particle reader**
- **Found during:** Task 1 (Failing Sparky allowlist and burst tests)
- **Issue:** `SessionCore` keeps the world private, so the powder and fade tests could not read group flags or colors after `advance`.
- **Fix:** Added `SessionCore::read_particles` under `cfg(test)` and used it from `sparky/tests.rs`.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`, `crates/liquidfun-wasm/src/scene/sparky/tests.rs`
- **Verification:** The powder, lifetime, and fade tests compile and fail until Task 2, then pass.
- **Committed in:** `d78b172` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The reader is test-only and does not widen `WorldCommand` or the public session. No scope creep.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Sparky is constructible from the Rust allowlist and proves FX-01 in native tests. Catalog pages, portrait frames, and browser smoke are still owned by the later plans in this phase. No review acknowledgment was recorded for this plan.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/sparky.rs
- FOUND: crates/liquidfun-wasm/src/scene/sparky/tests.rs
- FOUND: commit d78b172
- FOUND: commit 116d97b

---
*Phase: 29-sparky-drawing-and-full-catalog*
*Completed: 2026-09-27*

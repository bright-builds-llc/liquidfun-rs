---
phase: 29-sparky-drawing-and-full-catalog
plan: "01"
subsystem: particles
tags: [particle-color, batch-write, liquidfun]

requires:
  - phase: 26-particles-and-basin-scenes
    provides: particle groups and the optional color lane that a non-zero group color allocates
provides:
  - World::set_particle_colors writes one color into every resolved index of an existing lane
  - ParticleEditError::MissingColorLane when the lane is absent, without allocating it
affects:
  - 29-sparky-drawing-and-full-catalog

tech-stack:
  added: []
  patterns:
    - "Resolve every particle id and confirm the color lane before any color write"

key-files:
  created:
    - crates/liquidfun/src/world/object/tests/particle_color_batch.rs
  modified:
    - crates/liquidfun/src/world/particle_object/system.rs
    - crates/liquidfun/src/particle/storage/runtime.rs
    - crates/liquidfun/src/particle/editor.rs
    - crates/liquidfun/src/world/object/tests.rs

key-decisions:
  - "Stage resolved indices and check has_color_lane before mutation so a stale id or a missing lane leaves stored colors unchanged"
  - "An empty slice returns Ok(()) and writes nothing"
  - "ParticleColor::ZERO does not allocate a lane; MissingColorLane is the failure"

patterns-established:
  - "Batch color edits follow set_particle_velocity's resolve-then-commit shape and stay out of WorldCommand and WASM"

requirements-completed: [FX-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T05:25:55Z

duration: 15min
completed: 2026-09-27
---

# Phase 29 Plan 01: Batch Particle Color Write Summary

**Native `World::set_particle_colors` updates an existing color lane in one call and refuses to allocate a lane for a zero-color group**

## Performance

- **Duration:** 15 min
- **Started:** 2026-09-27T05:10:45Z
- **Completed:** 2026-09-27T05:25:55Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- One Rust call replaces every member of a non-zero color group with `ParticleColor::new(10, 20, 30, 255)` without changing the particle count. Alpha stays 255.
- An empty slice leaves the previous colors unchanged.
- A group created with `ParticleColor::ZERO` returns `ParticleEditError::MissingColorLane` and `maybe_colors()` stays `None`.
- A stale id returns `InvalidHandle` and does not change colors already stored.

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing batch color tests** - `a8852d9` (test)
2. **Task 2: Implement set_particle_colors** - `a666b58` (feat)

## Files Created/Modified

- `crates/liquidfun/src/world/object/tests/particle_color_batch.rs` - Arrange/Act/Assert coverage for replacement, empty input, a missing lane, and a stale id
- `crates/liquidfun/src/world/object/tests.rs` - Declares the `particle_color_batch` test module
- `crates/liquidfun/src/world/particle_object/system.rs` - `World::set_particle_colors` validates ids, then writes the existing lane
- `crates/liquidfun/src/particle/storage/runtime.rs` - Index resolution and in-lane color mutation beside `set_particle_velocity_internal`
- `crates/liquidfun/src/particle/editor.rs` - `ParticleEditError::MissingColorLane` with display text `particle color lane is not allocated`

## Decisions Made

- Writes are staged. Every id is resolved, then each owning system must already have a color lane, and only then are colors stored.
- A missing lane maps to `MissingColorLane` and does not allocate storage, including when the requested color is `ParticleColor::ZERO`.
- The public method is not exported through WASM and does not add a `WorldCommand` variant.

## Deviations from Plan

None - plan executed exactly as written.

The stale-id test is the feature behavior for a failed call that must leave stored colors unchanged. It sits beside the three cases named in Task 1.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Sparky fade can call `World::set_particle_colors` on a live burst and read the same lane through `ParticleSystemView::maybe_colors`. Zero-color groups still have no lane. No catalog, scene, or WASM files changed.

## Self-Check: PASSED

- FOUND: crates/liquidfun/src/world/object/tests/particle_color_batch.rs
- FOUND: crates/liquidfun/src/world/particle_object/system.rs
- FOUND: commit a8852d9
- FOUND: commit a666b58

---
*Phase: 29-sparky-drawing-and-full-catalog*
*Completed: 2026-09-27*

---
phase: 27-material-flag-groups
plan: "03"
subsystem: wasm-scenes
tags: [rigid-particles, rigid, solid, basin-family, liquidfun-wasm, tdd]

requires:
  - phase: 27-material-flag-groups
    provides: Private basin_family helper and Elastic Particles poses from Plans 01–02
provides:
  - Rigid Particles WASM scene with RIGID|SOLID clumps and spinning box
  - rigid-particles allowlist token in parse_scene_id
  - All three Phase 27 material SceneIds constructible before catalog advertising
affects:
  - 27-04 catalog append
  - 27-05 credits and smoke

tech-stack:
  added: []
  patterns:
    - ParticleGroupFlags::RIGID | SOLID without ELASTIC/SPRING particle flags
    - Same Elastic poses on basin_family with default WATER particle flags

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/rigid_particles.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks"
  - "Reuse basin_family and Elastic poses; RIGID|SOLID group flags only, no elastic particle flags"

patterns-established:
  - "Pattern: Rigid Particles red/green circles and blue spinning box use RIGID|SOLID group flags"
  - "Pattern: Headless asserts forbid ELASTIC/SPRING particle flags on rigid clumps"

requirements-completed: [MAT-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:51:03Z

duration: 2min
completed: 2026-09-22
---

# Phase 27 Plan 03: Rigid Particles Scene Summary

**Pinned Rigid Particles WASM scene with shared basin, RIGID|SOLID solid clumps, and spinning blue box without elastic particle flags**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-22T05:48:32Z
- **Completed:** 2026-09-22T05:51:03Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Rigid Particles constructs through `SceneId::RigidParticles` / `"rigid-particles"` with vertical-wall basin, three RIGID|SOLID groups, and a falling dynamic ball
- Groups use `ParticleGroupFlags::RIGID | SOLID` with default water particle flags; no `ELASTIC` or `SPRING` particle flags
- Blue box uses transform angle `-0.5` and `with_angular_velocity(2.0)` at Elastic poses
- Headless tests cover construction, flag recipe, spin pose, capped advance, unknown-control rejection, and pointer no-op
- `MAX_ADVANCE_STEPS` remains 4; no catalog/TS edits; no engine API changes; Surface Tension and Elastic Particles untouched beyond factory wiring

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing Rigid Particles allowlist and group-flag tests** - `cbca181` (test)
2. **Task 2: Implement Rigid Particles scene and factory wiring** - `1a2f3f5` (feat)

**Plan metadata:** pending docs commit

_Note: TDD tasks used RED stub then GREEN implementation_

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/rigid_particles.rs` - Pinned Rigid Particles scene + headless flag/construction tests
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::RigidParticles`, parse, build arm, `mod rigid_particles`
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist entry for `rigid-particles`

## Decisions Made

- Wired a minimal `SceneId::RigidParticles` stub in Task 1 so RED tests compile under sequential hooks instead of a compile-fail-only RED
- Reused Plan 01 `basin_family` and Plan 02 Elastic poses; solidity comes from group `RIGID | SOLID` only

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Minimal SceneId wiring during RED**
- **Found during:** Task 1 (failing Rigid Particles tests)
- **Issue:** Plan expected compile-fail until Task 2, but sequential commits with hooks need a compiling crate for RED verification
- **Fix:** Added enum variant, parse token, build arm, and stub `build` returning `SceneConstruction` so allowlist passes and behavioral tests fail
- **Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene/rigid_particles.rs`, `crates/liquidfun-wasm/src/session/tests.rs`
- **Verification:** Six `rigid_particles` tests failed on construction; allowlist test passed
- **Committed in:** `cbca181`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary for TDD under hooks; no scope creep into catalog, Elastic rewrite, or engine APIs

## Issues Encountered

None beyond the documented deviation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for plan 27-04 (catalog append) — all three material SceneIds construct (`surface-tension`, `elastic-particles`, `rigid-particles`)
- Catalog/TS advertising of Rigid Particles remains deferred to later Phase 27 plans
- No sealed C++ parity claim

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/rigid_particles.rs`
- FOUND: commit `cbca181`
- FOUND: commit `1a2f3f5`

---
*Phase: 27-material-flag-groups*
*Completed: 2026-09-22*

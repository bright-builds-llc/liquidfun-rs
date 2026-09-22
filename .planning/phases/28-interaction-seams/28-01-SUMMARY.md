---
phase: 28-interaction-seams
plan: "01"
subsystem: wasm-scenes
tags: [soup, destroy-in-shape, soup-family, liquidfun-wasm, tdd, ACT-01]

requires:
  - phase: 27-material-flag-groups
    provides: Watch-first SceneId factory, SessionCore, Particles-style basin patterns
provides:
  - Public World::destroy_particles_in_shape carve helper with focused unit test
  - Private soup_family builder returning ground BodyId for Soup Stirrer reuse
  - Watch-first Soup scene via SceneId::Soup / "soup"
affects:
  - 28-02 soup-stirrer
  - 28-06 catalog chrome

tech-stack:
  added: []
  patterns:
    - Public thin destroy-in-shape over query_aabb_with_particles + test_point + mark + compact
    - Private soup_family shared builder returning ground for prismatic composition

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/soup_family.rs
    - crates/liquidfun-wasm/src/scene/soup.rs
  modified:
    - crates/liquidfun/src/world/particle_object/particle.rs
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId::Soup wiring in Task 1 RED so construction tests compile under hooks"
  - "destroy_particles_in_shape marks then compact_pending_particles so the pocket is empty before first frame"
  - "Private soup_family returns ground BodyId for Plan 02 Soup Stirrer prismatic rail"

patterns-established:
  - "Pattern: soup_family builds basin + water + solids + carve; Soup wraps watch-first hooks"
  - "Pattern: carve under each solid fixture transform before BuiltScene returns"

requirements-completed: [ACT-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T13:48:19Z

duration: 24min
completed: 2026-09-22
---

# Phase 28 Plan 01: Soup Scene + Destroy-in-Shape Summary

**Public destroy-in-shape carve helper plus watch-first Soup WASM scene with shared soup_family builder matching pinned testSoup.js broth and bobbing solids**

## Performance

- **Duration:** 24 min
- **Started:** 2026-09-22T13:23:54Z
- **Completed:** 2026-09-22T13:48:19Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `World::destroy_particles_in_shape` queries shape AABB, filters with `test_point`, marks, and compacts so the fixture pocket is particle-free
- Focused liquidfun unit test proves an empty pocket under a placed circle after carve
- Private `soup_family` builds pinned Soup ground, water box at radius `0.035`, circle + two boxes + three edge noodles, carves under the three solids, and returns `ground` for Stirrer
- Watch-first `SceneId::Soup` / `"soup"` constructs through the WASM factory; unknown controls rejected; pointer is a no-op; `MAX_ADVANCE_STEPS` stays 4; destruction-by-age stays off

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing destroy-in-shape and Soup allowlist tests** - `e441f4b` (test)
2. **Task 2: Implement destroy helper, soup_family, Soup scene, factory wiring** - `e4f65ce` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_object/particle.rs` - Public carve helper + focused unit test
- `crates/liquidfun-wasm/src/scene/soup_family.rs` - Shared Soup basin/group/solids/carve builder
- `crates/liquidfun-wasm/src/scene/soup.rs` - Watch-first Soup BuiltScene and construction tests
- `crates/liquidfun-wasm/src/scene.rs` - SceneId::Soup allowlist + build_scene arm + soup_family module
- `crates/liquidfun-wasm/src/session/tests.rs` - `"soup"` allowlist table entry

## Decisions Made

- Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks (same sequential-TDD pattern as Phase 27)
- Carve marks then immediately `compact_pending_particles` so position queries see an empty pocket before presentation
- `soup_family` returns `ground: BodyId` for Plan 02 prismatic paddle rail composition

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Compiling RED stubs instead of compile-fail-only**
- **Found during:** Task 1
- **Issue:** Plan text allowed compile-fail until enum exists, but Phase 27 decision and hooks need compiling RED
- **Fix:** Stub `destroy_particles_in_shape` returning `Ok(0)` and stub `soup::build` returning `SceneConstruction` with minimal `SceneId::Soup` wiring
- **Files modified:** `particle.rs`, `soup.rs`, `scene.rs`, `session/tests.rs`
- **Verification:** `cargo test -p liquidfun destroy_particles_in_shape` and `cargo test -p liquidfun-wasm soup` failed RED as expected
- **Committed in:** `e441f4b`

**2. [Rule 1 - Bug] ParticleCreationReceipt must_use in carve test**
- **Found during:** Task 1
- **Issue:** Test create loop denied by `-D unused-must-use` on `ParticleCreationReceipt`
- **Fix:** Bind with `let _ = ...`
- **Files modified:** `particle.rs`
- **Verification:** Test compiled and failed on carve assertion
- **Committed in:** `e441f4b`

**3. [Rule 1 - Bug] WorldQueryOccurrence::Particle wraps ParticleQueryOccurrence**
- **Found during:** Task 2
- **Issue:** Helper treated the variant payload as `ParticleId`
- **Fix:** Use `hit.system()` / `hit.particle()` and filter by system in the visitor
- **Files modified:** `particle.rs`
- **Verification:** `cargo test -p liquidfun destroy_particles_in_shape` exits 0
- **Committed in:** `e4f65ce`

---

**Total deviations:** 3 auto-fixed (1× Rule 2, 2× Rule 1)
**Impact on plan:** Necessary for correct TDD under hooks and typed query API; no scope creep. No `lib.rs` change — `World` method is already public.

## Issues Encountered

None beyond the auto-fixed type and must_use issues above.

## Known Stubs

None — Soup constructs fully; carve is implemented.

## Threat Flags

None — no new trust-boundary surface beyond the planned `"soup"` allowlist token and build-time carve of known fixture shapes.

## Self-Check: PASSED

- FOUND: `crates/liquidfun/src/world/particle_object/particle.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/soup_family.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/soup.rs`
- FOUND: commit `e441f4b`
- FOUND: commit `e4f65ce`

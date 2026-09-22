---
phase: 28-interaction-seams
plan: "02"
subsystem: wasm-scenes
tags: [soup-stirrer, prismatic-joint, paddle-rail, liquidfun-wasm, tdd, ACT-02]

requires:
  - phase: 28-interaction-seams
    provides: Private soup_family builder returning ground BodyId for prismatic composition
provides:
  - Soup Stirrer BuiltScene composing soup_family with paddle carve and Option<JointId> prismatic rail
  - SceneId::SoupStirrer / soup-stirrer allowlist with toggle-paddle-rail and InSoup pointer toggle
  - on_advance stir force guarded by attached rail, InSoup AABB, and max speed
affects:
  - 28-06 catalog chrome
  - ACT-02 visitor free/restore paddle rail

tech-stack:
  added: []
  patterns:
    - Compose soup_family then add paddle/carve/prismatic rather than forking soup layout
    - Shared private toggle used by apply_control and apply_pointer; Option<JointId> for attach state

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/soup_stirrer.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId::SoupStirrer wiring in Task 1 RED so construction tests compile under hooks"
  - "Reuse ParticleSystemDef default damping 1.0 as pinned SetDamping(1.0); document PARTICLE_DAMPING explicitly"
  - "Map destroy_joint failures to SceneConstruction rather than UnknownControl"

patterns-established:
  - "Pattern: Option<JointId> prismatic toggle with InSoup AABB pointer and on_advance stir guards"
  - "Pattern: Soup Stirrer exports soup circle plus paddle circle; boxes/edges stay segments"

requirements-completed: [ACT-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:04:51Z

duration: 15min
completed: 2026-09-22
---

# Phase 28 Plan 02: Soup Stirrer Summary

**Soup Stirrer WASM scene composing soup_family with a prismatic paddle rail, InSoup pointer toggle, and guarded on_advance stir force**

## Performance

- **Duration:** 15 min
- **Started:** 2026-09-22T13:49:37Z
- **Completed:** 2026-09-22T14:04:51Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `SceneId::SoupStirrer` / `"soup-stirrer"` constructs through the WASM factory over shared `soup_family`
- Paddle circle at `(0,0.7)` r=`0.4` with carve, prismatic ground→paddle rail, and `toggle-paddle-rail` Live toggle ×2
- Pointer Up inside InSoup AABB shares the toggle path; outside is a success no-op; remount restores rail attached
- Stir force magnitude `10` uses τ-based oscillation only while joint is `Some`, paddle InSoup, and speed `< 2.0`
- `MAX_ADVANCE_STEPS` stays 4; destruction-by-age stays off

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Soup Stirrer allowlist and toggle tests** - `5098b9a` (test)
2. **Task 2: Implement Soup Stirrer scene and factory wiring** - `b0fa80d` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/soup_stirrer.rs` - Soup Stirrer build, hooks, and module tests
- `crates/liquidfun-wasm/src/scene.rs` - SceneId::SoupStirrer allowlist + build_scene arm
- `crates/liquidfun-wasm/src/session/tests.rs` - `"soup-stirrer"` allowlist table entry

## Decisions Made

- Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks (same sequential-TDD pattern as Plan 01)
- Default particle-system damping is already `1.0`; pin explicitly via `PARTICLE_DAMPING` rather than adding a new engine setter
- `destroy_joint` errors map to `SceneConstruction` so they are not confused with unknown control names

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Compiling RED stubs instead of compile-fail-only**
- **Found during:** Task 1
- **Issue:** Plan text allowed compile-fail until enum exists, but Phase 27/28-01 decision and hooks need compiling RED
- **Fix:** Stub `soup_stirrer::build` returning `SceneConstruction` with minimal `SceneId::SoupStirrer` wiring
- **Files modified:** `soup_stirrer.rs`, `scene.rs`, `session/tests.rs`
- **Verification:** `cargo test -p liquidfun-wasm soup_stirrer` failed RED; allowlist parse passed
- **Committed in:** `5098b9a`

**2. [Rule 1 - Bug] destroy_joint error mapped to UnknownControl**
- **Found during:** Task 2
- **Issue:** Joint destroy failures would look like an allowlist miss
- **Fix:** Map to `SessionError::SceneConstruction`
- **Files modified:** `soup_stirrer.rs`
- **Verification:** `cargo test -p liquidfun-wasm soup_stirrer` exits 0
- **Committed in:** `b0fa80d`

---

**Total deviations:** 2 auto-fixed (1× Rule 2, 1× Rule 1)
**Impact on plan:** Necessary for correct TDD under hooks and honest error typing; no scope creep.

## Issues Encountered

Soup Stirrer construction is heavy in debug tests (multi-create remount coverage); slimmed remount to two SessionCore creates after GREEN.

## Known Stubs

None — Soup Stirrer constructs fully with paddle rail toggle and guarded stir.

## Threat Flags

None — control/pointer surface matches the planned `toggle-paddle-rail` allowlist and InSoup AABB gate; stir force remains bounded by attach + InSoup + max-speed guards.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/soup_stirrer.rs`
- FOUND: `crates/liquidfun-wasm/src/scene.rs`
- FOUND: commit `5098b9a`
- FOUND: commit `b0fa80d`

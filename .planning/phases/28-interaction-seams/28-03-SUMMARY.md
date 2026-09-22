---
phase: 28-interaction-seams
plan: "03"
subsystem: wasm-scenes
tags: [impulse, particle-group-shove, push-mode, liquidfun-wasm, tdd, ACT-03]

requires:
  - phase: 28-interaction-seams
    provides: Soup/Soup Stirrer SceneId factory patterns and Sequential TDD RED stub wiring
provides:
  - Impulse BuiltScene with chain-loop box and full member_ids group force/impulse shove
  - SceneId::Impulse / impulse allowlist with live push-mode force|impulse preset
  - Focused momentum test proving inside-box shove vs outside no-op
affects:
  - 28-06 catalog chrome
  - ACT-03 visitor click-to-shove

tech-stack:
  added: []
  patterns:
    - Whole-blob shove copies full member_ids and scales by n into range force/impulse APIs
    - Live push-mode stored on hooks; remount restores Force default without presets

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/impulse.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId::Impulse wiring in Task 1 RED so construction tests compile under hooks"
  - "Reject non-empty presets on Impulse build because push-mode is Live-only"

patterns-established:
  - "Pattern: box AABB hit-test + normalize(pointer − boxCenter) + member_ids × magnitude shove"
  - "Pattern: twin settle worlds to prove shove momentum against gravity-matched baseline"

requirements-completed: [ACT-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:11:11Z

duration: 5min
completed: 2026-09-22
---

# Phase 28 Plan 03: Impulse Summary

**Impulse WASM scene with chain-loop box, whole-group force/impulse shove via native range APIs, and live push-mode preset**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-22T14:06:05Z
- **Completed:** 2026-09-22T14:11:11Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `SceneId::Impulse` / `"impulse"` constructs through the WASM factory with radius `0.025` and damping `0.2`
- Chain-loop box `[-2,2]×[0,4]` plus group box `0.8×1.0` at `(0,1.01)`; Pointer Up inside shoves full `member_ids`
- Default push mode is force (`direction * 1.0 * n`); live `push-mode=impulse` uses `0.005 * n` without recreating
- Outside-box pointer is a success no-op; unknown controls and values reject with `UnknownControl`
- `MAX_ADVANCE_STEPS` stays 4; twin-world momentum test proves inside shove diverges from outside settle

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Impulse allowlist and shove tests** - `cf74c65` (test)
2. **Task 2: Implement Impulse scene and factory wiring** - `b110b2b` (feat)

**Plan metadata:** `2dcfc36` (docs: complete plan)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/impulse.rs` - Impulse build, shove hooks, and module tests
- `crates/liquidfun-wasm/src/scene.rs` - SceneId::Impulse allowlist + build_scene arm
- `crates/liquidfun-wasm/src/session/tests.rs` - `"impulse"` allowlist table entry

## Decisions Made

- Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks (same sequential-TDD pattern as Plans 01–02)
- Impulse rejects non-empty presets because `push-mode` is `ControlEffect::Live` only; Reset remounts with Force default

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Compiling RED stubs instead of compile-fail-only**
- **Found during:** Task 1
- **Issue:** Plan text allowed compile-fail until enum exists, but Phase 27/28 decisions and hooks need compiling RED
- **Fix:** Stub `impulse::build` returning `SceneConstruction` with minimal `SceneId::Impulse` wiring
- **Files modified:** `impulse.rs`, `scene.rs`, `session/tests.rs`
- **Verification:** `cargo test -p liquidfun-wasm impulse` failed RED; allowlist parse passed
- **Committed in:** `cf74c65`

***

**Total deviations:** 1 auto-fixed (1× Rule 2)
**Impact on plan:** Necessary for correct TDD under hooks; no scope creep.

## Issues Encountered

None.

## Known Stubs

None — Impulse constructs fully with box hit-test and whole-group shove.

## Threat Flags

None — `push-mode` allowlists exact `force`/`impulse`; shove magnitudes stay fixed (`1.0` / `0.005` × n); outside-box clicks remain no-ops.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/impulse.rs`
- FOUND: `crates/liquidfun-wasm/src/scene.rs`
- FOUND: commit `cf74c65`
- FOUND: commit `b110b2b`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ACT-03 Impulse native seam is ready for catalog chrome (28-06) and Wave Machine / Theo Jansen plans
- Do not raise `MAX_ADVANCE_STEPS` or silently change Impulse radius

***
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*

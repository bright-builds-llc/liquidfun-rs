---
phase: 18-six-native-physics-demos
plan: "07"
subsystem: wasm-scene
tags: [liquidfun-wasm, color-mixer, color-mixing, tdd]

requires:
  - phase: 18-six-native-physics-demos
    provides: SceneId factory, Recreated preset bag, shared basin helpers, COLOR_MIXING solver
provides:
  - Two COLOR_MIXING particle-color groups in a bowl
  - Construction mix-strength Off/Gentle/Strong
  - Live stir-speed Off/Slow/Fast via apply_particle_force_range
affects: [18-09, catalog-ready, player-controls]

tech-stack:
  added: []
  patterns:
    - mix-strength uses with_color_mixing_strength at construct and returns Recreated
    - stir-speed applies one tangential force range each on_advance and returns Live
    - honesty tests compare captured Rust color-lane bytes, not canvas blends

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/color_mixer.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Two filled circles of radius 1.15 at (-1.0, 2.2) teal and (1.0, 2.2) red sit in contact so COLOR_MIXING can run."
  - "Mix-strength Off 0.0 / Gentle 0.25 / Strong 0.5 recreates; default Strong. Stir-speed Off/Slow/Fast is live; default Slow."
  - "Replace the leftover Color Mixer stub assertion with a live-world create check; keep SceneUnimplemented for fail-closed vocabulary."

patterns-established:
  - "Particle-color mixing is engine COLOR_MIXING plus color_mixing_strength; capture copies raw color lanes."
  - "Construction-only system-def coefficients recreate; per-step forces stay Live."

requirements-completed: [DEMO-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T05:25:38Z

duration: 13min
completed: 2026-09-18
---

# Phase 18 Plan 07: Color Mixer Contact-Driven Mixing Summary

**Color Mixer now builds two overlapping COLOR_MIXING groups whose captured color-lane bytes stay stable at Off and change under default Strong, with live stir forces and construction-only mix-strength.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-18T05:11:47Z
- **Completed:** 2026-09-18T05:25:38Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `SessionCore::create(SceneId::ColorMixer)` builds a bowl with teal `(57, 211, 199, 255)` and destructive red `(248, 113, 113, 255)` filled groups. Count is in `40..=220`. Every particle carries `COLOR_MIXING`.
- `mix-strength=off` recreates with `with_color_mixing_strength(0.0)` and keeps `particle_colors()` equal after 120 Slow-stirred steps. Default Strong `0.5` changes captured color-lane bytes after 120 steps. Particle count stays constant.
- `stir-speed` `off`/`slow`/`fast` returns `ControlEffect::Live` and applies `apply_particle_force_range` on the contiguous particle range. Unknown mix/stir tokens fail closed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build two COLOR_MIXING groups**
   - `326e60a` (test): failing create/count/color/flag tests
   - `5c4898a` (feat): two filled mixing groups in a shared basin bowl
2. **Task 2: Prove Off is stable and Strong changes color lanes**
   - `0988aa4` (test): failing Off-recreate and live stir tests
   - `121bb70` (feat): mix-strength recreate, live stir, leftover stub-test update

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/color_mixer.rs` — two particle-color mixing groups, construction mix-strength, live stir
- `crates/liquidfun-wasm/src/session.rs` — leftover Color Mixer stub assertion replaced with a live create check

## Decisions Made

- Keep two filled circles of radius `1.15` centered at `(-1.0, 2.2)` and `(1.0, 2.2)` so the groups sit in contact without extra emission.
- Mix-strength is construction-only because there is no live system-def setter. Stir is a single tangential force on the current contiguous particle range.
- Label this particle-color mixing in comments only; do not claim pigment chemistry.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Leftover Color Mixer stub assertion**
- **Found during:** Task 2
- **Issue:** `session.rs` still expected `SceneId::ColorMixer` to return `SceneUnimplemented`. Full lib tests would fail after the scene constructed. Plan said not to put mixing logic in `session.rs`; the leftover stub test still had to move.
- **Fix:** Replaced `create_stub_scenes_fail_closed_without_a_live_world` with `create_color_mixer_constructs_a_live_world` and allowed the unused `SceneUnimplemented` variant for fail-closed vocabulary.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib` exits 0
- **Committed in:** `121bb70`

**2. [Rule 2 - Missing Critical] Allowlisted mix/stir tokens**
- **Found during:** Task 2
- **Issue:** Threat T-18-07-01 requires fail-closed unknown mix/stir values.
- **Fix:** Parse only `off`/`gentle`/`strong` and `off`/`slow`/`fast`; added `unknown_mix_and_stir_tokens_fail_closed`.
- **Files modified:** `crates/liquidfun-wasm/src/scene/color_mixer.rs`
- **Verification:** that test passes
- **Committed in:** `121bb70`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Required for full-lib verification and the declared allowlist. No scope creep.

## Issues Encountered

- Default Strong already changed color lanes after Task 1 because `ParticleSystemDef` defaults to `color_mixing_strength` `0.5`. Task 2 RED still failed on Off recreate and live stir, which was the remaining honesty gap.
- `cargo fmt --all --check` also reported pre-existing wrap diffs in other scene files; only Color Mixer and the leftover session test were formatted.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-09 catalog/player wiring of Color Mixer controls. Mix-strength must show recreate copy; stir-speed is live. Do not advertise pigment chemistry.

---
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/color_mixer.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: 326e60a
- FOUND: 5c4898a
- FOUND: 0988aa4
- FOUND: 121bb70

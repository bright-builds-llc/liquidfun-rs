---
phase: 37-puzzle-core-in-the-bridge
plan: "04"
subsystem: wasm-session
tags: [rust, session, refactor, escape-limits, fingerprint-gate, GOAL-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-11T23:56:38.844Z"
requires:
  - crate::scene::{build_scene, particle_iterations, SceneId} (unchanged)
  - target/phase37/base.txt (Plan 01 baseline, phase base 958d6fad7)
provides:
  - crate::session::SessionSource (Scene arm only)
  - SessionCore::from_source(source, presets) and the escape_limits field
  - crate::session::escape::{EscapeLimits, EscapeLimits::SHARED, EscapeLimits::for_scene, particle_escaped_within}
affects:
  - 37-06 (adds SessionSource::Puzzle(LevelId, Placements) with its own build path, iterations and EscapeLimits::SHARED)
  - 37-08 (final 25/25 gate)
tech-stack:
  added: []
  patterns:
    - session behavior derived once at construction from a source enum, stored as precomputed values
key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/session/escape.rs
    - crates/liquidfun-wasm/src/session/tests.rs
key-decisions:
  - "SessionCore stores source: SessionSource and precomputed escape_limits instead of scene_id; the scene arm derives particle_iterations and EscapeLimits::for_scene exactly as before, proven 25/25 identical fingerprints (stamp 2026-10-11T23-48-25Z)"
  - "particle_has_escaped(scene, ..) survives only as a cfg(test) wrapper over particle_escaped_within(limits, ..), so session/tests.rs changed by one line"
requirements: [GOAL-04]
duration: 11min
completed: 2026-10-11
---

# Phase 37 Plan 04: SessionSource and EscapeLimits refactor Summary

**`SessionCore` now builds from `SessionSource::Scene(SceneId)` and stores precomputed `EscapeLimits`. Every scene's particle iterations, escape limits (wave-tank 80/80, others 12/48) and control-driven recreation stay as they were, and the gate shows 25/25 end-state fingerprints identical to the phase baseline.**

## Performance

- **Duration:** about 11 min (23:46:05Z to 23:56:38Z). Most of it was the scene-spot run and the full check script, whose test step took about 4 min.
- **Tasks:** 2 of 2
- **Files:** 3 modified

## Fingerprint Gate (D-21)

- Command: `just playground-scene-spot` (release, runs 3), logged to `target/phase37/spot-after-04.log`
- Stamp directory: `target/dam-break-perf/2026-10-11T23-48-25Z/`
- `target/phase37/after-04.txt` has 25 lines, and `diff target/phase37/base.txt target/phase37/after-04.txt` prints nothing.
- **Result: `25/25 identical`**, recorded in `target/phase37/gate-04.txt`.

## Accomplishments

- `EscapeLimits { fall, escape }` with `SHARED` (12 m / 48 m) and `const fn for_scene` (only `WaveTank` widens to 80/80). `particle_escaped_within(limits, ..)` keeps every other line and operation of the old body in the same order.
- `evict_escaped_particles` and `escaped_particle_ids` take `limits: EscapeLimits`. `advance` and `advance_profiled` pass `self.escape_limits`.
- `SessionSource` enum (Scene arm). `from_source` replaces `from_built`. `create(id: SceneId)` keeps its signature. Both recreation paths in `apply_control` call `Self::from_source(self.source.clone(), self.presets.clone())`.
- 3 new escape unit tests in AAA form: `shared_limits_are_twelve_down_and_forty_eight_away`, `wave_tank_widens_both_limits_to_eighty`, `other_scenes_use_the_shared_limits`.
- `session/tests.rs` changed by exactly one line (1 added, 1 removed) and stays at 580 lines. Its `use super::*;` glob already brings in `SessionSource`, so no import edit was needed.

## Task Commits

1. **Task 1: EscapeLimits and SessionSource.** TDD: I added the three escape tests along with the types. Before the commit, `cargo test -p liquidfun-wasm --lib session::` reported 27 passed, including the 3 new tests.
1. **Task 2: Gate, checks and commit.** `feb4804ff` (`refactor(37-04): derive session behavior from SessionSource and EscapeLimits`). It contains exactly the three plan files.

The plan asks for one commit, so both tasks share `feb4804ff`.

## Check Results (`target/phase37/checks-04.results`)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | exit=0 |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| `cargo test -p liquidfun-wasm --all-features` (353 passed, 1 ignored) | exit=0 |
| `bun scripts/bright-builds-check.ts all` | exit=0 |
| `bun scripts/web-build.ts wasm` | exit=0 |
| `git diff --stat 958d6fad7 -- crates/liquidfun` is empty | exit=0 |

The wasm build printed one warning, the known pre-existing `crates/liquidfun/src/particle/solver/boundary/support.rs:98` unreachable expression. It is listed in the v1.4 leftovers and out of scope here.

## Deviations from Plan

None. The plan was executed as written. One note: `generated_by` is `gsd-execute-phase`, as the orchestrator asked, while the plan's output section says `gsd-executor`.

GOAL-04 is still open in REQUIREMENTS.md, as the orchestrator instructed. This plan only makes the scene path source-based. The replay itself comes in Plans 05 through 08.

## Known Stubs

None. `SessionSource` has only the `Scene` arm on purpose. Plan 06 adds `Puzzle(LevelId, Placements)`.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/session.rs (466 lines), crates/liquidfun-wasm/src/session/escape.rs (186 lines), crates/liquidfun-wasm/src/session/tests.rs (580 lines)
- FOUND: commit feb4804ff
- FOUND: target/phase37/gate-04.txt (`25/25 identical`), target/phase37/after-04.txt (25 lines), target/phase37/checks-04.results (7 lines, all exit=0)

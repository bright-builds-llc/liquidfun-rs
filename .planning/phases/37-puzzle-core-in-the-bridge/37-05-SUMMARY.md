---
phase: 37-puzzle-core-in-the-bridge
plan: "05"
subsystem: puzzle-core
tags: [rust, puzzle, tools, placements, level-spec, builder, hooks, GOAL-02, GOAL-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-12T00:08:30.122Z"
requires:
  - crate::puzzle::goal::{GoalRect, GoalTracker, Outcome, PuzzleStatus, count_in_goal} (Plan 03)
  - crate::puzzle::{LevelId, LEVEL_IDS} (Plan 03)
  - crate::scene::{BuiltScene, SceneHooks, attach_basin_fixture, RigidSegment} (unchanged apart from one default method)
provides:
  - crate::puzzle::angles::TAU_24
  - crate::puzzle::tools::{ToolKind, AngleIndex, GridRect, ToolAllowance, Placement, Placements, PlacementError, MAX_PLACEMENTS, POSE_STRIDE, GRID_METERS}
  - crate::puzzle::spec::{LevelSpec, LiquidBlock}
  - crate::puzzle::first_pour::{FIRST_POUR, REFERENCE_PLANK_POSE (cfg(test))}
  - crate::puzzle::level_spec(id)
  - crate::puzzle::builder::build_level(spec, &Placements)
  - crate::puzzle::hooks::PuzzleHooks
  - SceneHooks::maybe_puzzle_status (default None)
affects:
  - 37-06 (SessionSource::Puzzle builds through build_level; tunes FIRST_POUR against measured outcomes)
  - 37-07 (SessionError::InvalidPlacement(PlacementError), newPuzzle, levelInfo; removes the two temporary dead_code allows)
  - 37-08 (fixtures use ToolKind::token and REFERENCE_PLANK_POSE)
tech-stack:
  added: []
  patterns:
    - literal trig table indexed by an integer angle step, so static tool geometry is bit-identical across targets
    - fail-closed lane parsing with a size bound before allocation and canonical sort by derived Ord
    - declarative const level spec consumed by one generic builder
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/angles.rs
    - crates/liquidfun-wasm/src/puzzle/tools.rs
    - crates/liquidfun-wasm/src/puzzle/tools/tests.rs
    - crates/liquidfun-wasm/src/puzzle/spec.rs
    - crates/liquidfun-wasm/src/puzzle/first_pour.rs
    - crates/liquidfun-wasm/src/puzzle/builder.rs
    - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
    - crates/liquidfun-wasm/src/puzzle/hooks.rs
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/scene.rs
key-decisions:
  - "Plank corners are TAU_24[angle] rotations of integer-grid centres in the fixed order cos*lx - sin*ly + cx, sin*lx + cos*ly + cy; no tool or level geometry calls sin, cos, oriented_box or from_angle"
  - "Placements::parse checks count (<= 16, before allocation), lane length, then per placement kind code, level allowlist, angle, i16 range and build area, then per-kind budget, and sorts by derived (kind, x, y, angle) Ord"
  - "build_level validates the spec, then creates one static body for walls and cup walls, one static body per plank, then the particle system and one group per liquid block, and records counted group ids for count_in_goal"
  - "First-pour starting layout builds 1989 particles and 32 outline segments (36 with one plank); Plan 06 tunes it"
requirements: [GOAL-02, GOAL-04]
duration: 12min
completed: 2026-10-12
---

# Phase 37 Plan 05: Plank placements, first-pour level, builder and puzzle hooks Summary

**Plank geometry now comes from integer grid poses and a literal τ/24 `(cos, sin)` table. `Placements::parse` rejects bad lanes with 8 distinct typed errors and sorts the rest canonically. `FIRST_POUR` is const data. `build_level` turns a spec plus placements into a `BuiltScene` whose `PuzzleHooks` count the cup and latch the outcome once per fixed step.**

## Performance

- **Duration:** about 12 min (2026-10-11T23:57:34Z to 2026-10-12T00:08:30Z). The full `cargo test -p liquidfun-wasm` step took about 3 min of that.
- **Tasks:** 3 of 3
- **Files:** 10 (8 created, 2 modified)

## Measured Values

| Item | Value |
| --- | --- |
| First-pour particle count (empty placements) | 1989 (inside 1500..=3000, and at least `full_count` 900) |
| Outline segments, empty build | 32 (6 walls + 2 cup walls, 4 edges each) |
| Outline segments, reference plank `[8, 64, 23]` | 36 |
| Initial goal count | 0 |
| `TAU_24` literal changes forced by clippy | None. `excessive_precision` did not fire, so the table matches the research literals exactly. |

## Accomplishments

- `angles::TAU_24` has 3 tests. Every entry is within 1e-7 of the f64 cos/sin, has unit length within 1e-6, and the quarter turns are exact.
- `tools.rs` (299 lines) provides:
  - `ToolKind::Plank = 1` (code 0 fails closed), with code and token round trips.
  - `AngleIndex` (< 24) and the inclusive `GridRect`.
  - `ToolAllowance` and `Placement`. `Placement` derives `Ord` with the field order (kind, x, y, angle).
  - `Placement::center` (`f32::from(i16) / 20.0`) and `Placement::vertices`.
  - `Placements::{empty, parse, as_slice}`.
  - `PlacementError` derives `Debug, Clone, Copy, PartialEq, Eq` and has a `const fn message`, ready for Plan 07's `SessionError::InvalidPlacement`.
- `spec.rs` holds `LiquidBlock` and `LevelSpec`, with docs on every field.
- `first_pour.rs` builds `FIRST_POUR` from a `const fn quad` helper, using the plan's layout:
  - 6 walls with the 0.4 m outlet.
  - The cup left wall and a tall backstop.
  - Interior (1.6, 0)..(3.4, 1.8).
  - One counted block at 0.06 m stride.
  - Target 540, full 900, settle 30, limit 720.
  - One plank, build area (-20, 44)..(40, 84).
  - `REFERENCE_PLANK_POSE` (test only).
- `puzzle::level_spec(id)` is a `const fn` match.
- `builder::build_level` keeps the D-09 order. It validates first, then calls `create_geometry`, `create_planks`, `create_liquid`/`create_block` and `outline_segments`, then does the initial count. Every engine error maps to its `SceneError`, and any `SceneError` maps to `SessionError::SceneConstruction`.
- `hooks::PuzzleHooks`:
  - `on_after_step` advances its own counter with `checked_add(1)`, then reads `positions()` and `group_ids()` and calls `count_in_goal` and `GoalTracker::observe`.
  - Controls, actions and pointer input return `UnknownControl`.
  - `collect_segments` clones the precomputed outline, and `collect_circles` is empty.
  - `maybe_puzzle_status` reports the full `PuzzleStatus`.
- `SceneHooks::maybe_puzzle_status` is a new default method that returns `None`. No scene overrides it, and `scene_hooks_report_no_puzzle_status` checks this on Dam Break.

## Task Commits

1. **Task 1: τ/24 table, plank tool and placement parsing.** TDD RED: the build failed on unresolved imports against a placeholder `tools.rs`. GREEN: `puzzle::tools` passed 19 tests and `puzzle::angles` passed 3.
1. **Task 2: First-pour spec, builder and hooks.** `puzzle::builder` passed 13 tests.
1. **Task 3: Checks and commit.** `7ef3f738b` (`feat(37-05): add plank placements, first-pour level, builder and puzzle hooks`). It contains exactly the ten plan files.

The plan asks for one commit, so all three tasks share `7ef3f738b`.

## Check Results (`target/phase37/checks-05.results`)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | exit=0 |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| `cargo test -p liquidfun-wasm --all-features` (388 passed, 1 ignored) | exit=0 |
| `bun scripts/bright-builds-check.ts all` | exit=0 |
| `bun scripts/web-build.ts wasm` | exit=0 |
| `git diff --stat 958d6fad7 -- crates/liquidfun` is empty | exit=0 |

The wasm build printed one warning, the known `crates/liquidfun/src/particle/solver/boundary/support.rs:98` unreachable expression. It is a v1.4 leftover and out of scope here.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extra tests for items that `cfg(test)` builds never use**
- **Found during:** Tasks 1 and 2
- **Issue:** The temporary `#![cfg_attr(not(test), allow(dead_code))]` does not cover the lib test build. `ToolKind::from_token`, `GridRect::bounds`/`contains`, `GRID_METERS` and `LevelSpec::particle_iterations` would otherwise be dead in that build, and clippy `-D warnings` rejects dead code. This is the same issue Plan 03 hit.
- **Fix:** I added four AAA tests instead of widening the allow:
  - `tool_kind_codes_and_tokens_round_trip`
  - `grid_rect_reports_its_inclusive_bounds`
  - `grid_cell_size_matches_cells_per_meter`, which compares bits to satisfy `float_cmp`
  - `levels_use_the_shared_particle_iterations`
- **Files modified:** puzzle/tools/tests.rs, puzzle/builder/tests.rs
- **Commit:** 7ef3f738b

**2. [Rule 1 - Lint] `ToolKind::code` uses a `match`, not `self as u8`**
- **Issue:** The acceptance grep forbids ` as u8` in tools.rs.
- **Fix:** A one-arm `match` that returns 1. The behavior is the same.

### Other notes

- `generated_by` is `gsd-execute-phase`, as the orchestrator asked. The plan's output section says `gsd-executor`.
- I did not mark GOAL-02 or GOAL-04 complete in REQUIREMENTS.md, as the orchestrator instructed. The session wiring comes in Plans 06 and 07, and the replay comes in Plan 08.
- I did not touch `session.rs`, `session/escape.rs` or `session/tests.rs`.

## Known Stubs

- `scene.rs` has a temporary `#[cfg_attr(not(test), allow(dead_code))]` on `maybe_puzzle_status`, and `puzzle.rs` line 2 has the same kind of allow. These are intentional, and Plan 37-07 removes both when `newPuzzle` wires the module.
- The `FIRST_POUR` numbers are a starting layout. Plan 06 tunes them against measured win and loss outcomes (D-07).

## Self-Check: PASSED

- FOUND: all 8 created files and both modified files, each within its line limit (tools.rs 299, tools/tests.rs 330, builder/tests.rs 292).
- FOUND: commit 7ef3f738b, which contains only the ten plan files.
- FOUND: target/phase37/checks-05.results (7 lines, all exit=0).

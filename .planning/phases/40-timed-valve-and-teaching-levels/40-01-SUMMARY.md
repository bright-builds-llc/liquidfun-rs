---
phase: 40-timed-valve-and-teaching-levels
plan: "01"
subsystem: wasm-puzzle-core
tags: [rust, wasm, puzzle, valve, pose-lanes, fixture-grammar, fingerprint-baseline, TOOL-05]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T04:32:05.182Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: PlacementStatus 0..12, placement_statuses/first_overlap validator, PuzzleHooks, replay fixtures and contracts
provides:
  - POSE_STRIDE = 4 `(x, y, angle, open_tick)` for every kind; Placements::parse(kinds, poses, &LevelSpec)
  - ToolKind::Valve = 2 (token `valve`, 1.2 m x 0.25 m outline, VALVE_HALF_LENGTH 0.6 / VALVE_HALF_THICKNESS 0.125)
  - PlacementError::ValveTimeOutOfRange and PlacementStatus::ValveTimeOutOfRange = 13 (geometric, outline present)
  - LevelSpec::max_open_tick (step_limit / 30, saturated to u8)
  - puzzle/valve.rs STEPS_PER_OPEN_TICK, open_step, ValveGate; test-only VALVE_TEST_SPEC and REFERENCE_VALVE_POSE
  - PuzzleHooks::on_advance destroys due gates with World::destroy_body before world.step
  - SessionCore::from_built and #[cfg(test)] SessionCore::create_puzzle_from_spec
  - Fixture grammar `placement plank x y angle` (5 tokens) / `placement valve x y angle open_tick` (6 tokens)
  - target/phase40/base.txt (25 scene fingerprints at phase base 65a37d35b)
affects:
  - 40-02 (TS lane encoder, Bun fixture parser, placement-status mirror 0..13, e2e helpers move to stride 4)
  - 40-07 (timed-gate level builds on ValveGate and create_puzzle_from_spec)
  - 40-11 (final 25/25 fingerprint gate against target/phase40/base.txt)
tech-stack:
  added: []
  patterns:
    - open tick validated in parse_one after the build-area check, so status 13 keeps its outline
    - valves contribute frame segments through their gates; static segments keep the Phase 37 order
    - pre-step valve opening keyed only to the hooks' own observed_step
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/valve.rs
    - crates/liquidfun-wasm/src/puzzle/valve/tests.rs
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/tools.rs
    - crates/liquidfun-wasm/src/puzzle/tools/tests.rs
    - crates/liquidfun-wasm/src/puzzle/validate.rs
    - crates/liquidfun-wasm/src/puzzle/validate/tests.rs
    - crates/liquidfun-wasm/src/puzzle/spec.rs
    - crates/liquidfun-wasm/src/puzzle/first_pour.rs
    - crates/liquidfun-wasm/src/puzzle/inspect.rs
    - crates/liquidfun-wasm/src/puzzle/sat.rs
    - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
    - crates/liquidfun-wasm/src/puzzle/replay/fixture/tests.rs
    - crates/liquidfun-wasm/src/puzzle/builder.rs
    - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
    - crates/liquidfun-wasm/src/puzzle/hooks.rs
    - crates/liquidfun-wasm/src/puzzle/session_tests.rs
    - crates/liquidfun-wasm/src/puzzle/api_tests.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
key-decisions:
  - "Valve open tick is checked in parse_one after the build-area rule (valve 1..=step_limit/30 via u8::try_from + range, other kinds exactly 0); decode_pose stays geometry-only so status 13 always has an outline"
  - "Valves open in PuzzleHooks::on_advance when observed_step + 1 >= open_tick x 30; the frame at step S-1 has the gate's 4 segments and the frame at S does not; no SceneHooks or session step-order change"
  - "Fixture placement arity depends on the kind (plank 5 tokens, valve 6); the kind token is checked before arity, so `placement gate 1 2 3` reports `unknown tool` and a bare `placement` reports `expected 5 space-separated tokens`"
  - "Valve test cell (-16, 87, 0, 4) is valid under VALVE_TEST_SPEC: centre (-0.8, 4.35) m under the first-pour outlet, 0.125 m below the reservoir floor"
metrics:
  duration: 21min
  completed: 2026-10-13
  tasks: 3
  files: 20
---

# Phase 40 Plan 01: Timed valve Rust core Summary

The Rust bridge now has a stride-4 pose lane, `ToolKind::Valve` (code 2) with open-tick validation and placement status 13, and a 5/6-token fixture grammar. A valve is a static body from step 0. The pre-step puzzle hook destroys it just before the session runs step `open_tick × 30`. The first-pour fixtures replay bit-identically, and all 25 scene fingerprints match the phase base.

## Task 1: Fingerprint baseline (D-24)

- Phase base commit: `65a37d35b2d5f907f8fc08fa1c8c9aa90303895e` (`target/phase40/phase-base.txt`).
- `git diff --quiet 67ba6c483 HEAD -- crates tools xtask Cargo.toml Cargo.lock web/src` exited 0, and the Rust working tree was clean.
- Scene-spot stamp: `target/dam-break-perf/2026-10-13T04-11-43Z/`. `target/phase40/base.txt` has 25 lines. The cross-check against `target/phase39/after-final.txt` was identical (`phase39-cross-check: identical` in `baseline.log`).
- After Task 3, stamp `target/dam-break-perf/2026-10-13T04-31-51Z/`. `diff target/phase40/base.txt target/phase40/after-01.txt` was empty (25/25).
- Nothing was committed, because `target/` is gitignored.

## Task 2: Stride-4 lanes, valve kind, status 13, grammar (commit 7275316e9)

- `tools.rs`: `POSE_STRIDE = 4`, `ToolKind::Valve = 2` and `half_extents()`. `vertices()` picks the half extents by kind and keeps the expression order, so planks are bit-identical. `Placement.open_tick` is the last field, so the canonical order is `(kind, x, y, angle, open_tick)`. `Placements::parse` and `parse_one` now take `&LevelSpec`.
- `validate.rs`: `ValveTimeOutOfRange = 13` is in `from_error` and `error`.
- `spec.rs`: `LevelSpec::max_open_tick()`. first-pour gives 24.
- `first_pour.rs`: `REFERENCE_PLANK_POSE: [i32; 4] = [8, 64, 23, 0]`.
- `replay/fixture.rs`: added `FixturePlacement.open_tick`, per-kind arity, and `lanes()` now emits 4 values per placement.

### Flipped Rust tests

- `tools/tests.rs`: `rejects_a_reserved_fourth_pose_value` became `rejects_stride_three_lanes`. `rejects_a_pose_lane_that_is_not_three_per_kind` became `..._four_per_kind`, with a 5-value long lane. The unknown-code case moved from `parse(&[2], …)` to code 3. `tool_kind_codes_and_tokens_round_trip` covers Plank and Valve and no longer asserts that `from_token("valve") == None`.
- `validate/tests.rs`: `kind_codes_zero_and_two_are_unknown` became `kind_codes_zero_and_three_are_unknown` (`&[0, 3]`). In `inspector_and_new_puzzle_agree`, `&[2]` became `&[3]`, and three cases were added: valve on first-pour, plank tick 1 and plank tick -1.
- `replay/fixture/tests.rs`: `rejects_an_unknown_level_or_tool` was split into `rejects_an_unknown_level` and `rejects_an_unknown_tool` (`placement gate 1 2 3`). `placement valve 1 2 3` now fails with `expected 6 space-separated tokens`.
- About 60 stride-3 pose literals gained a trailing `0`. They are in `tools/tests.rs`, `validate/tests.rs`, `api_tests.rs`, `session_tests.rs` and `sat.rs`, plus the helper strides (`/ 3`, `3 * …`, `[i32; 3]`). `builder/tests.rs` and `session_tests.rs` now pass `spec` to `Placements::parse`. `goal/tests.rs` had no pose literals.

### New tests

- Tools: valve code and token, codes 0 and 3 unknown, valve parse with tick, ticks 1 and 24 valid, ticks 0, 25, -1 and 300 rejected, plank tick 1 rejected, `max_open_tick` 24, planks before valves, ties broken by open tick, and bit-exact valve corners at (0, 0).
- Validate: the reference valve is valid, ticks 1 and 24 are valid, out-of-range ticks give status 13 with the `Placement::vertices()` outline, code 13, plank tick gives 13, a valve through the floor gives OverlapsWall, valve × plank gives OverlapsTool both ways, a valve on first-pour gives KindNotAllowed, and the build rejects tick 25.
- Fixture: a valve line parses, a plank line means tick 0, a 6-token plank is rejected, a 5-token valve is rejected, a bare `placement` is rejected, and `lanes()` carries the valve tick.

## Task 3: Valve lifecycle (commit 7c07ed971)

- `builder.rs`: `create_tools` returns `Vec<ValveGate>`. `static_segments` covers walls, cup walls and non-valve tools in the old order. A shared `edges()` helper builds the segments.
- `hooks.rs`: `on_advance` destroys due gates. `collect_segments` returns the static segments, then the edges of each closed gate.
- `session.rs`: `from_source` now only matches on the source and delegates to `from_built`, which is pure code motion. `#[cfg(test)] create_puzzle_from_spec` was added.
- `valve/tests.rs` has 7 tests:
  - `open_step(4) == 120`.
  - `is_due` is false at 119 and true at 120 and 121.
  - An opened gate is never due and has no segments.
  - Hook-level order: calling `on_advance` before step 119 keeps the body. Calling it before step 120 removes one body and 4 segments, without `world.step` running in between.
  - The frame at step 119 has exactly 4 more segments than the frame at 120, and those segments are the valve's outline edges. The body count also drops by one.
  - More particles fall below y = 4.2 in steps 120..150 than in steps 90..120.
  - Two identical builds give the same `frame_hash` at step 240.
- Valve test cell: `[-16, 87, 0, 4]` is valid as planned, so no substitute was needed.

## Release first-pour RECORD lines

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 peak_held 811 brim_step 262 profile release
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
```

## Verification

- `cargo test -p liquidfun-wasm` (debug): 522 passed, 3 ignored.
- `cargo test -p liquidfun-wasm --release --lib puzzle::`: 201 passed.
- Also passed: `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features` and `bun scripts/bright-builds-check.ts all` (0 findings).
- `crates/liquidfun` is unchanged since the phase base. `scene.rs` and `scene/` are unchanged, and `puzzle/fixtures` is unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Stride-3 literals in `puzzle/sat.rs` tests**
- **Found during:** Task 2
- **Issue:** The SAT unit tests build planks through `pose_placement` with `[i32; 3]` poses. Under stride 4 they panicked, and `sat.rs` was not in the plan's file list.
- **Fix:** The helper now takes `[i32; 4]`, and its three literals gained a trailing `0`.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/sat.rs
- **Commit:** 7275316e9

**2. [Rule 3 - Blocking] `Placement::kind`/`open_tick` were dead code in Task 2**
- **Issue:** Clippy `-D warnings` flagged both accessors, because nothing outside tests uses them until the builder does in Task 3.
- **Fix:** Both stayed `#[cfg(test)]` in the Task 2 commit. The Task 3 commit made them non-test, once `create_tools` and `static_segments` used them.
- **Commits:** 7275316e9, 7c07ed971

**Other choices:**
- `VALVE_TEST_SPEC` and `REFERENCE_VALVE_POSE` live as `#[cfg(test)]` items in `valve.rs`, so the validate and valve tests share one spec.
- `parse_one` destructures `[_, _, _, raw_tick]` instead of the planned `[.., raw_tick]` plus a length check. It rejects the same lanes in one step.

## Known Stubs

None.

## Threat Flags

None. The only new input surface is the 4th pose value, which the plan covers in T-40-02. It is range-checked, it fails closed with status 13, and its message never echoes input.

## Self-Check: PASSED

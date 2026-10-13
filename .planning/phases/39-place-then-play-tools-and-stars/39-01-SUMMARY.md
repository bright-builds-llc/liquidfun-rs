---
phase: 39-place-then-play-tools-and-stars
plan: "01"
subsystem: wasm-puzzle-core
tags: [rust, wasm, puzzle, sat, placement-validation, fingerprint-baseline, TOOL-03]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:04:42Z"
requires:
  - phase: 37-puzzle-core-in-the-bridge
    provides: Placements::parse, Placement::vertices (τ/24 baked), LevelSpec, build_puzzle_core, SessionError::InvalidPlacement
provides:
  - crates/liquidfun-wasm/src/puzzle/sat.rs separated_by_more_than, rect_quad
  - crates/liquidfun-wasm/src/puzzle/validate.rs PlacementStatus (codes 0..12), Inspection, inspect, validate_for_build, inspect_level, OUTLINE_STRIDE
  - wasm export inspectPuzzlePlacements(levelId, kinds, poses) -> PlacementInspection { statuses(): Uint8Array, outlines(): Float32Array }
  - PuzzleLevelInfo.brimCount() / parTools()
  - LevelSpec brim_count, par_tools, no_place; first-pour budget 2 planks, par 1, brim 810
  - target/phase39/base.txt (25 scene fingerprints for phase base 424735672)
affects:
  - 39-02 (web/src/puzzle/placement-status.ts mirrors the status codes)
  - 39-05 (peak held / Brim uses brim_count 810)
  - 39-06 (status-code parity test)
  - 39-14 (final 25/25 fingerprint gate against target/phase39/base.txt)
tech-stack:
  added: []
  patterns:
    - one validator (placement_statuses + first_overlap) behind both the stateless inspector and newPuzzle
    - SAT with unnormalized edge normals and a squared gap comparison (no trig, no square root)
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/sat.rs
    - crates/liquidfun-wasm/src/puzzle/validate.rs
    - crates/liquidfun-wasm/src/puzzle/validate/tests.rs
    - crates/liquidfun-wasm/src/puzzle/inspect.rs
  modified:
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/spec.rs
    - crates/liquidfun-wasm/src/puzzle/first_pour.rs
    - crates/liquidfun-wasm/src/puzzle/tools.rs
    - crates/liquidfun-wasm/src/puzzle/tools/tests.rs
    - crates/liquidfun-wasm/src/puzzle/info.rs
    - crates/liquidfun-wasm/src/puzzle/builder.rs
    - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
    - crates/liquidfun-wasm/src/puzzle/api_tests.rs
key-decisions:
  - "Placement status wire codes: 0 Valid, 1 UnknownKind, 2 KindNotAllowed, 3 OverBudget, 4 AngleOutOfRange, 5 PositionOutOfRange, 6 OutsideBuildArea, 7 OverlapsWall, 8 OverlapsCupWall, 9 InsideCup, 10 OverlapsWater, 11 OverlapsTool, 12 OverlapsNoPlace"
  - "validate_for_build runs Placements::parse (Phase 37 error order) and then the inspector's own placement_statuses, returning the first invalid status in input order, so inspector and newPuzzle cannot diverge"
  - "Budget counts every placement of an allowed kind in input order; placements past the allowance that are otherwise valid become OverBudget"
  - "first-pour: 2 planks, par_tools 1, brim_count 810 (90% of full 900), no_place empty; builder rejects specs unless target < brim <= full and par <= total budget"
  - "D-30 baseline: scene-spot stamp 2026-10-12T23-50-30Z on a tree whose Rust equals 424735672; identical to target/phase38/after-final.txt"
requirements-completed: [TOOL-03]
duration: 15min
completed: 2026-10-13
---

# Phase 39 Plan 01: Rust Placement Validator, Inspector and Star Spec Fields Summary

**A convex SAT validator with a 2 × particle-radius gap now backs both the stateless `inspectPuzzlePlacements` wasm export and `newPuzzle`. First Pour allows 2 planks with par 1 and brim 810, and the pre-phase 25-scene fingerprint baseline is recorded.**

## Performance

- **Duration:** about 15 min
- **Started:** 2026-10-12T23:50:05Z
- **Completed:** 2026-10-13T00:04:42Z
- **Tasks:** 3 of 3
- **Files modified:** 14 (4 created, 10 modified)

## Accomplishments

- D-30 baseline: `target/phase39/phase-base.txt` = `424735672beffd5989a89f12a321f429e55bd8b5`. `git diff --quiet <base> -- crates tools Cargo.toml Cargo.lock` was clean, so the survey ran on the main tree with no worktree. Stamp `target/dam-break-perf/2026-10-12T23-50-30Z/`, 25 lines in `base.txt`, `phase38-cross-check: identical`.
- `sat.rs`: `separated_by_more_than(a, b, gap)` projects both quads onto the four edge normals of each quad. The normals are unnormalized. Two quads are separated only when `separation > 0 && separation² > gap²·|n|²`, so touching at the gap counts as overlap. A non-finite vertex or gap never separates. `rect_quad` is a const constructor.
- `validate.rs`: `PlacementStatus` codes 0 through 12 (see key decisions). `inspect` checks the lanes before allocating and returns statuses and 8-float outlines in input order. Outlines are zeros for UnknownKind, AngleOutOfRange and PositionOutOfRange. Overlap precedence is walls, cup walls, cup interior, liquid blocks, `no_place`, then other tools. An overlap marks both tools, and a tool that is invalid for another reason still blocks the tools it overlaps. The budget check runs last.
- `newPuzzle` → `build_puzzle_core` → `validate_for_build`, so every set the inspector marks invalid fails with `SessionError::InvalidPlacement(_)`.
- `PuzzleLevelInfo.brimCount()` returns 810 and `parTools()` returns 1. `toolCounts()` now returns `[2]`.
- Six new `PlacementError` variants (`OverlapsWall`, `OverlapsCupWall`, `InsideCup`, `OverlapsWater`, `OverlapsTool`, `OverlapsNoPlace`). Their messages are static, contain no digits and never echo input.

## Task Commits

1. **Task 1: 25-scene fingerprint baseline (D-30).** No commit, because `target/` is gitignored. The files are `target/phase39/{phase-base.txt,base.txt,baseline.log}`.
1. **Task 2: SAT, PlacementStatus validator and LevelSpec additions.** `5f256e45f` (feat)
1. **Task 3: levelInfo brim/par getters and inspector parity tests.** `61c68c511` (feat)

## Status Codes and Measured Cases (first-pour)

| Lanes | Status |
|---|---|
| `(8,64,23)` reference / fixture | 0 Valid |
| `(-20,84,6)` | 7 OverlapsWall (reservoir floor) |
| `(40,44,0)` | 8 OverlapsCupWall (backstop) |
| `(38,44,6)` | 9 InsideCup |
| `(-20,84,6)` with `walls: &[]` | 10 OverlapsWater |
| `[(8,64,23),(8,64,0)]` | [11, 11] OverlapsTool |
| `(10,64,0)` with `no_place: [GridRect(0,60,20,70)]` | 12 OverlapsNoPlace (Valid on plain first-pour) |
| `(41,64,0)` | 6 OutsideBuildArea (outline still emitted) |
| `[(8,64,23),(30,84,0),(-20,44,0)]` | [0, 0, 3 OverBudget] |
| angle 24 / kind 0 or 2 / x 40000 | 4 / 1 / 5 (outline zeros) |
| 17 kinds / misaligned poses | Err(TooManyPlacements) / Err(LaneLengthMismatch) |

## Pitfall 10 Test Changes

- `builder/tests.rs`: `first_pour_allows_exactly_one_plank` was renamed `first_pour_allows_two_planks_with_par_one`. It asserts budget `[Plank × 2]`, `par_tools == 1`, `brim_count == 810` and an empty `no_place`. The new test `specs_with_unordered_star_counts_are_rejected` covers brim == target, brim > full and par > budget.
- `api_tests.rs`: `tool_counts() == [2]`. The `ToolBudgetExceeded` case now uses three planks `[8,64,23, 30,84,0, -20,44,0]`. With a budget of 2, the old two-plank pair would now fail as a tool overlap instead. Also added: brim/par assertions, `two_valid_planks_build_and_overlapping_planks_fail`, `overlap_error_messages_are_bounded_and_echo_no_input`, and a function-pointer existence check for `inspect_puzzle_placements`.
- `tools/tests.rs`: `placement_error_messages_are_bounded_and_distinct` now includes the six overlap variants.
- The fixtures and `REFERENCE_PLANK_POSE` are unchanged. The replay still reads won @201 final 689 and lost @720 final 0.

## Verification

- `cargo fmt --all --check`: clean.
- `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`: exit 0. The plan's `cargo clippy --all-targets --all-features` also exits 0, but it checks only the workspace default members, so the explicit `-p liquidfun-wasm` run is the meaningful one.
- `cargo test -p liquidfun-wasm`: 456 passed, 0 failed, 3 ignored (the ignored tests existed before this plan). `cargo test -p liquidfun-wasm --lib puzzle::`: 135 passed. `puzzle::validate`: 22 tests.
- `cargo test -p liquidfun-wasm --release --lib puzzle::replay`: 14 passed.
- `bun scripts/bright-builds-check.ts all`: 0 findings. Line counts: `lib.rs` 311, `validate.rs` 284, `validate/tests.rs` 384, `sat.rs` 146.
- `grep -cE "fn (sin|cos)|\.sin\(|\.cos\(|sqrt"` on `sat.rs` and `validate.rs`: 0 and 0.
- `git diff --stat 424735672 -- crates/liquidfun`: empty.

## Decisions Made

See `key-decisions` in the frontmatter. This plan only sets brim 810. Research already measured the reference peak held count against it (Brim latches at step 262 with peak 811). Plan 05 records the measured peak held fill, as D-07 requires.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Moved the `newPuzzle` wiring and the `inspect.rs` export into the Task 2 commit**
- **Found during:** Task 2
- **Issue:** Before Task 3 wired them in, `inspect`, `Inspection`, `inspect_level` and `validate_for_build` were unused in the non-test lib build. That fails Task 2's own clippy `-D warnings` verify with dead-code errors.
- **Fix:** The Task 2 commit includes the `build_puzzle_core` → `validate_for_build` swap in `lib.rs`, the new `inspect.rs` and its re-exports. Task 3 still carries the `info.rs` getters, the `newPuzzle` doc update and the parity tests. Both commits are clippy-clean and pass their tests.
- **Commit:** `5f256e45f`

**2. [Design refinement] `validate_for_build` reuses the inspector's status pass instead of a second pass over canonical placements**
- **Issue:** The plan asked for "the same overlap pass on the canonical placements", with `first_overlap` shared between the two paths.
- **Fix:** `validate_for_build` runs `Placements::parse`, which keeps the Phase 37 error order. It then calls the same `placement_statuses` (and therefore `first_overlap`) that `inspect` uses, and returns the first invalid status in input order. Pairwise overlap is symmetric, so canonical and input order give the same accept/reject result. Only which overlap variant gets reported for a multi-fault set can differ. Parity holds by construction, and `inspector_and_new_puzzle_agree` checks it over 16 lane sets.

**3. [Rule 2 - Correctness] Explicit finiteness guard in SAT**
- **Issue:** `f32::min`/`max` drop NaN, so a projection fold could ignore one NaN vertex and report a separation.
- **Fix:** `separated_by_more_than` returns `false` (overlap) when any vertex or the gap is non-finite. Tests `a_nan_vertex_never_separates` and `a_nan_gap_never_separates` cover this.

Added helpers in `tools.rs`: `check_lanes` (shared by `Placements::parse` and the inspector, with unchanged behavior) and `pose_placement` (outline geometry for out-of-area or disallowed tools).

TDD note: each task landed as one feat commit, with its tests and implementation together, rather than separate RED and GREEN commits.

## Issues Encountered

None.

## Known Stubs

None.

## Threat Flags

None. The new `inspectPuzzlePlacements` boundary is covered by T-39-01 through T-39-04: lanes are checked before allocation, n ≤ 16, there is no `unwrap` or unchecked indexing, messages are static, and one validator serves both entry points.

## Next Phase Readiness

- Plan 02 can mirror the status codes 0 through 12 in `web/src/puzzle/placement-status.ts` and decode `statuses()`/`outlines()` (outline stride 8).
- Plan 14 compares its final fingerprints against `target/phase39/base.txt`.

## Self-Check: PASSED

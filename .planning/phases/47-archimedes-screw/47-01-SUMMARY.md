---
phase: 47-archimedes-screw
plan: "01"
subsystem: playground-scene-rust
tags: [scene, archimedes-screw, kinematic, vane-train, snap-wrap, particles, release-tests, spike, d-02]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: "2026-10-14T21:32:03.623Z"
requires:
  - "45-09: docs/adding-a-scene.md per-scene checklist (R1-R12)"
  - "46-01: Cartesian Diver module layout, release-only test gating, build/build_parts split"
provides:
  - "SceneId::ArchimedesScrew registered for parse/build/variant_index/session token (NOT in ALL_SCENE_IDS or SURVEY_SCENES; Plan 03 adds those with the web SCENE_IDS entry)"
  - "scene/archimedes_screw.rs: build(), build_parts(), ArchimedesScrewHooks (one kinematic vane train, closed-loop velocity, snap-one-pitch wrap), axial_speed, advance_phase, train_position"
  - "scene/archimedes_screw/geometry.rs: tube frame, housings, roof, floor slab, reservoir + return ramp, basin, weir, chute, vane lattice, spawn grid, clipped vane outlines, FRAME_* bounds, test-only regions"
  - "scene/archimedes_screw/spike_tests.rs: 8 release-only D-02 regression guards (P1-P5)"
  - "Candidate A verdict: PASS (iteration 3 of 4); candidate B never built"
affects: [47-02, 47-03, 47-04, 47-05]
tech-stack:
  added: []
  patterns:
    - "Snap-one-pitch kinematic lattice: set_body_transform only while the swapped vane is inside a solid housing, then closed-loop velocity toward the next lattice point"
    - "Non-parallel closing faces: a static face a vane closes on leans away from the vane so the squeeze is an open wedge, never a parallel film"
    - "Tube-frame geometry with literal cos/sin/tan constants (no trig in production code), checked against TAU in unit tests"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/archimedes_screw.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/tests.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/test_support.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/spike_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs
key-decisions:
  - "Candidate A (one kinematic vane train that snaps back one pitch) passes the written D-02 rules; candidate B is not needed"
  - "Tube length 6 m with 7 vanes (not 5 m / 6 vanes): the basin must sit above a reservoir deep enough to fill pockets, and 5 m left no head between them"
  - "Bottom housing face leans 45 degrees (BOTTOM_FACE_TAN = 1.0) and the inlet end of the floor slab is undercut at -20 degrees: vane-parallel faces pinned a film of particles when a vane closed on them under water"
  - "Reservoir spawn fill 1.15 m (settles to a p95 surface of about 0.50-0.66 m) so sealed pockets hold 24+ particles at the default speed"
  - "visible_vane_outline returns Option<Vec<Vec2>>: the leaning bottom face clips a vane into a trapezoid, not a parallelogram"
requirements-completed: [SCREW-01]
duration: 26min
completed: 2026-10-14
---

# Phase 47 Plan 01: Archimedes Screw Scene Core and D-02 Spike Summary

**Candidate A passes. One kinematic train of 7 slanted vanes slides up a 30 degree closed tube and snaps back one pitch inside solid housings. It lifts 24-43 particle pockets into a weir basin that overflows down a chute, with zero particles inside any wall or vane, forward and reverse, at 30 and 60 rpm.**

## Performance

- **Duration:** about 26 min
- **Started:** 2026-10-14T21:05:40Z
- **Completed:** 2026-10-14T21:32:03Z
- **Tasks:** 3/3
- **Files:** 6 created, 3 modified

## Accomplishments

- Native screw scene: static walls (10 convex quads), one kinematic vane train, and 1,729 water particles. Every spawn point is at least r clear of every wall and vane.
- `on_advance` snaps the train back one pitch with `set_body_transform` only when the phase wraps. It then sets a clamped velocity toward the next lattice point. Speed 0 holds the train bit-for-bit still.
- Compile-time guard `MAX_AXIAL_SPEED < 0.5 * PARTICLE_DIAMETER / SIM_DT`: 1.0 m/s against a 3.0 m/s limit.
- Controls, actions and non-empty presets fail closed with `UnknownControl`. Pointer input is accepted and does nothing (D-11).
- `collect_segments` draws each wall quad as a closed loop, plus each vane clipped only against the two housing faces. There is no shaft line and no circles; the frame has 40 wall segments plus about 28 vane segments.
- Rust registration covers the parse arm, build arm, `variant_index => 26` and the session token. `ALL_SCENE_IDS` and `SURVEY_SCENES` are untouched.
- The D-02 pass rules stay in the repository as 8 release-only regression guards.

## D-02 Spike Measurements (candidate A)

All counts start after a 60-step settle. Each run is 1,200 measured steps at 60 Hz. Leak and escape values are maxima over samples taken every 4th step. Pocket values are the minimum sealed-pocket count over steps 300/600/900/1,200, and are judged only at forward default speed. The basin value is the basin-region count at the end.

| Iter | Geometry change | rpm | Dir | Inside walls | Inside vanes | Escapes | Max snap-step speed | Max other-step speed | Min sealed pocket | Final basin | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Baseline: L 6 m, 7 vanes, all faces vane-parallel, fill 0.95 m, 1,509 particles | 30 | fwd | 0 | **1** | 0 | 5.63 | 12.00 | 16 | 136 | FAIL P1 (inlet slab-end pinch, Pitfall 5) |
| 1 | same | 60 | fwd | 0 | 0 | 0 | 5.92 | 8.76 | (13) | 143 | pass P1-P3 |
| 1 | same | 30 | rev | 0 | **6** | 0 | 4.58 | 12.00 | n/a | 0 | FAIL P1 (film pinned on the submerged bottom-housing face) |
| 1 | same | 60 | rev | **2** | **12** | 0 | 4.86 | 12.00 | n/a | 0 | FAIL P1 (same) |
| 2 | Bottom housing face leans 45 degrees; inlet slab end undercut -20 degrees; 1,540 particles | 30 | fwd | 0 | 0 | 0 | 6.13 | 8.01 | **13** | 140 | FAIL P4 (one pocket at 13 at step 600) |
| 2 | same | 60 | fwd | 0 | 0 | 0 | 5.09 | 6.85 | (1) | 122 | pass P1-P3 |
| 2 | same | 30 | rev | 0 | 0 | 0 | 3.94 | 7.35 | n/a | 0 | pass P1-P3 |
| 2 | same | 60 | rev | 0 | 0 | 0 | 4.27 | 5.50 | n/a | 0 | pass P1-P3 |
| 3 | Reservoir fill 0.95 to 1.15 m; 1,729 particles | 30 | fwd | 0 | 0 | 0 | 5.45 | 6.54 | 24 | 147 | **PASS P1-P5** |
| 3 | same | 60 | fwd | 0 | 0 | 0 | 6.07 | 7.11 | (9) | 149 | **PASS P1-P3** |
| 3 | same | 30 | rev | 0 | 0 | 0 | 4.00 | 6.12 | n/a | 0 | **PASS P1-P3** |
| 3 | same | 60 | rev | 0 | 0 | 0 | 5.08 | 5.82 | n/a | 0 | **PASS P1-P3** |

Parenthesized pocket values are informational; P4 applies to forward default speed only. In the `SessionCore` run (default, forward, 1,260 steps), `live_particle_count() == particle_count()` (P2), so nothing is evicted.

Final forward-default trace (iteration 3, every 2 s):

- Reservoir p95 surface: 0.66, 0.63, 0.56, 0.53, 0.54, 0.50, 0.51, 0.53, 0.53, 0.53 m.
- Sealed pockets: 24 to 43 particles.
- Basin: 30, 60, 92, 132, 146, 156, then a plateau of about 145-156. The basin reaches its lip and overflows down the chute.
- Particles outside every region: 0 to 9.

**Verdict for candidate A: PASS within the time box (3 of 4 iterations). Candidate B was not built.**

### What the spike taught (for later scenes)

- The research probe never drove a vane *into* a submerged housing. With vane-parallel faces, a layer of particles resting flush on the bottom-housing face (q of about 0.005) was pinned between two parallel closing faces in reverse. The film could not drain through the floor opening in time.
- The fix is geometric. Lean every face a vane closes on away from the vane, so contact starts at one end and the water leaves through an open wedge:
  - Bottom housing face at 45 degrees: contact starts at the roof, and the wedge opens down into the inlet.
  - Inlet slab end undercut at -20 degrees: contact starts at the floor line, and the wedge opens downward.
  - The top housing and the outlet slab end stay vane-parallel. Vanes only close on them where they are above the basin lip (dry), and they measured clean.
- The snap itself never produced a speed burst. Snap-step maxima stayed below non-snap maxima in every run, as the research's engine analysis predicted.

## Final Geometry Constants (`geometry.rs`)

| Constant | Value | Note |
|---|---|---|
| `PARTICLE_RADIUS` / `PARTICLE_DIAMETER` / `PARTICLE_STRIDE` | 0.05 / 0.1 / 0.075 m | D-12 |
| `AXIS` / `NORMAL` | (0.866_025_4, 0.5) / (-0.5, 0.866_025_4) | `TAU / 12` (30 degrees), unit-tested |
| `TUBE_ORIGIN` | (-4.0, -0.3) | s = 0, t = 0 |
| `TUBE_GAP` h / `SLAB_THICKNESS` W | 0.7 (7 d) / 0.35 m | D-12 |
| `TUBE_LENGTH` L / `HOUSING_LENGTH` C | **6.0** / 1.0 m | lift 3.0 m |
| `INLET_LENGTH` a / `OUTLET_LENGTH` b | 1.4 / 1.4 m (at least `PITCH + VANE_OVERHANG` = 1.291) | q at the floor line |
| `PITCH` | 1.0 m | |
| `VANE_THICKNESS` / `VANE_EMBED` / `VANE_COUNT` | 0.3 m (3 d) / 0.1 m (1 d) / **7** | D-04 |
| `TRAIN_START_S` | -0.5 | vane 0 mid-housing at phase 0 |
| `VANE_TAN` / `VANE_COS` | 0.363_970_23 / 0.939_692_6 | `TAU / 18` (20 degrees), unit-tested |
| `BOTTOM_FACE_TAN` | 1.0 | `tan(TAU / 8)`, 45-degree bottom housing face |
| `INLET_END_TAN` (private) | -`VANE_TAN` | undercut inlet slab end |
| `RESERVOIR_LEFT_X` / `RESERVOIR_FLOOR_Y` / `RAMP_START_X` / `RAMP_END` | -5.6 / -1.5 / -4.0 / (2.45, 0.85) | ramp about 20 degrees |
| `RESERVOIR_FILL_TOP_Y` | **1.15** m (settles to a p95 surface of about 0.5-0.66) | D-08 |
| `BASIN_FLOOR_Y` / `BASIN_FLOOR_BOTTOM_Y` / `BASIN_FLOOR_LEFT_X` | 1.6 / 1.35 / -0.5 | |
| `WEIR_INNER_X` / `WEIR_OUTER_X` / `WEIR_LIP_Y` | 1.45 / 1.7 / **2.05** | outlet floor at q = L - b is 1.936; top housing low corner is 2.2695 (D-06) |
| `CHUTE_INNER_X` / `CHUTE_OUTER_X` / `CHUTE_TOP_Y` | 2.2 / 2.45 / 2.8 | D-07 |
| `INLET_SEAL_Y` (test-only) | 0.3363 | floor line at q = a |
| INLET_COVER_Y (not yet defined; Plan 02) | -0.0137 | floor line at q = a / 2 |
| Particle count | **1,729** (`with_maximum_count` = exact spawn count) | at most 3,000 |

Mechanism constants (`archimedes_screw.rs`): `DEFAULT_SCREW_RPM = 30` (0.5 m/s), `MAX_SCREW_RPM = 60` (1.0 m/s), `SIM_DT = 1/60`, particle damping 0.2, pressure strength 0.25, strict contact check, vane friction 0.2.

## FRAME_* (landscape, for `ARCHIMEDES_SCREW_VIEW_BOUNDS`, UI-D71)

`FRAME_MIN_X = -6.1`, `FRAME_MIN_Y = -2.0`, `FRAME_MAX_X = 2.65`, `FRAME_MAX_Y = 4.45`. This is the wall bounding box (x -5.9 to 2.45, y -1.8 to 4.2367) plus 0.2 m, rounded outward. A unit test checks a margin of at least 0.1 m.

## Portrait Wall Points for Plan 03 (world meters)

| Point | Coordinates |
|---|---|
| Reservoir left wall top (outer, inner) | (-5.9, 1.6), (-5.6, 1.6) |
| Reservoir floor corners: left inner, ramp start, ramp top end | (-5.6, -1.5), (-4.0, -1.5), (2.45, 0.85) |
| Reservoir floor outer bottom | (-5.9, -1.8) |
| Bottom housing far corners (back face) | (-4.9117, -1.2305), (-5.1704, 0.2367) |
| Bottom housing face corners (45 degrees) | (-4.4312, -0.9531), (-3.9188, 0.9593) |
| Top housing far corners (back face) | (2.0165, 2.7695), (1.7578, 4.2367) |
| Top housing face corners | (1.1505, 2.2695), (0.8918, 3.7367) |
| Weir lip (inner, outer) | (1.45, 2.05), (1.7, 2.05) |
| Chute outer wall top (inner, outer) | (2.2, 2.8), (2.45, 2.8) |

## Decision Coverage

- **D-01:** Vanes are one kinematic body driven by velocity each fixed step. Axial speed = `PITCH * rpm / 60`.
- **D-02:** Candidate A was measured against the written rules P1-P5 and passed in iteration 3. The rules are committed as release-only guards.
- **D-03:** The vane added or removed at a snap lies wholly inside a housing; `test_point` tests cover phase 0 and phase `PITCH - 1e-4`. Vane tips embed 0.1 m into 0.35 m slabs.
- **D-04:** Vanes are 3 d thick with a 1 d embed. The compile-time speed guard holds (1.0 m/s against 3.0 m/s), and the velocity is clamped to `MAX_AXIAL_SPEED`.
- **D-05:** The vanes slant 20 degrees from the tube normal, top leaning uphill.
- **D-11:** `apply_pointer` returns `Ok(())` for every kind and changes nothing.
- **D-12:** r = 0.05, 30-degree tilt, 7 d gap, 1,729 particles.
- **D-14:** Walls are drawn as closed loops and vanes clipped at the housing faces; there is no shaft line.
- **D-06, D-07, D-08** are covered by the geometry tests: lip between the outlet floor and the housing, chute, basin empty at load, and reservoir at or above the inlet seal.

## Task Commits

1. **Tasks 1 and 2 (geometry, hooks, registration):** `655f9fa66` feat(47-01): add Archimedes Screw bridge scene core
1. **Task 3 (spike, geometry iterations 2-3, guards):** `cd4511cdb` test(47-01): measure the screw wrap and leak spike

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tube length 6 m and 7 vanes instead of 5 m and 6 vanes**

- **Found during:** Task 1 geometry design.
- **Issue:** With L = 5, the weir lip must sit between 1.44 and 1.77 m. Pockets only fill if the reservoir sits near 0.9 m. That leaves no room for a basin floor above the reservoir and below the lip.
- **Fix:** L = 6 (`VANE_COUNT = 7`, still `(N + 1) * p - C`), which raises the outlet by 0.5 m. The research listed L = 6 as the alternative, and D-12 leaves dimensions to discretion. The README timing at 30 rpm with L = 6 is tighter (research Pitfall 7: about 36 rpm), and Plan 02 owns rpm tuning.
- **Files modified:** `geometry.rs`.
- **Commit:** `655f9fa66`.

**2. [Rule 1 - Bug] Non-parallel closing faces (spike iteration 2)**

- **Found during:** Task 3, iteration 1.
- **Issue:** The vane-parallel bottom housing face pinned a film of particles under water in reverse: 6 to 12 inside vanes and 2 inside walls. The vane-parallel inlet slab end pinned 1 particle in forward. The plan's facts called for vane-parallel faces.
- **Fix:** The bottom housing face leans 45 degrees and the inlet slab end is undercut at -20 degrees. `visible_vane_outline` now clips against the leaning face with one Sutherland-Hodgman pass and returns `Option<Vec<Vec2>>`. The opening, snap-containment and outline tests were updated and kept; a new test checks the bottom-face clip.
- **Commit:** `cd4511cdb`.

**3. [Rule 1 - Bug] Reservoir fill 1.15 m (spike iteration 3)**

- **Issue:** One sealed pocket held 13 particles, below the P4 minimum of 15. The 0.95 m spawn column compresses to a surface near 0.4 m.
- **Fix:** Raised the fill to 1.15 m. The particle count went from 1,540 to 1,729.
- **Commit:** `cd4511cdb`.

**4. [Rule 3 - Blocking] Test helpers trimmed to what this plan calls**

- Per the plan's own clippy rule, these are deferred to Plan 02: `RegionCounts`/`region_counts`, `in_chute`, `train_transform` and `reservoir_surface_y`. `basin_count` replaces `RegionCounts.basin` for P5. `FRAME_*`, `VANE_OVERHANG` and `INLET_SEAL_Y` are `#[cfg(test)]`, because only tests read them in Rust. Plan 03 mirrors `FRAME_*` in TypeScript.

**5. [Rule 3 - Blocking] Hooks tests split into `hooks_tests.rs`**

- `tests.rs` reached 626 of the 628 allowed lines. The hooks tests moved to `archimedes_screw/hooks_tests.rs`, in the same lib test binary, which leaves Plan 02 room.

**6. [Minor] `MAXIMUM_PARTICLE_COUNT` is not a constant**

- The particle system's maximum count is `water_positions().len()` at build: exact, and at most 3,000 by test.

TDD note: the geometry and hooks tests were written alongside the implementation in the same task rather than as a separate failing commit. Tasks 1 and 2 share one commit, as the plan specifies.

## Known Stubs

- `apply_control` returns `Err(UnknownControl)` for every name, including `screw-speed` and `screw-direction`. This is intentional; Plan 02 adds the two live controls. The hooks hold `rpm = 30` and `reverse = false`.

## Notes for Plan 02

- Forward at 60 rpm still lifts water (final basin 149), but its sealed pockets dip to 9. D-19 at max speed does not assert pockets.
- The reservoir p95 surface settles at about 0.50-0.66 m. That is far above `INLET_COVER_Y` (-0.014) and `INLET_SEAL_Y` (0.336).
- The basin plateaus at about 145-156 particles once it overflows. A reverse run that starts after the basin fills drains it only while the basin level is above the outlet floor at q = L - b (1.936 m, lip 2.05 m). Expect a modest drop.
- With L = 6, the README clip (inlet to spill) likely needs about 36 rpm (research Pitfall 7). The speed guard has room up to 180 rpm.

## Deferred Issues

- A pre-existing release-only `unreachable expression` warning in `crates/liquidfun/src/particle/solver/boundary/support.rs:98` (engine, out of scope). It is logged in `deferred-items.md`.

## Verification

- `cargo fmt --all --check`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`: all clean.
- `cargo test -p liquidfun-wasm --release --lib archimedes_screw`: 43 passed, 0 failed, 0 ignored (8 spike tests).
- `cargo test -p liquidfun-wasm --all-features --lib archimedes_screw::spike_tests` (debug): 8 ignored.
- `cargo test -p liquidfun-wasm --all-features`: 670 passed, 0 failed, 44 ignored.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet HEAD~2 -- crates/liquidfun web docs/benchmarks/scene-fingerprints.json`: unchanged.
- No `eprintln!`/`println!`, `paddle` or `candidate b` in the screw module. No forces, impulses, dynamic bodies or `unwrap()` in production code.

## Self-Check: PASSED

All six module files, the SUMMARY and deferred-items.md exist; commits 655f9fa66 and cd4511cdb are in history.

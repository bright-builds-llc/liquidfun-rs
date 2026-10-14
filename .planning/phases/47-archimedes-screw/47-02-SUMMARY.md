---
phase: 47-archimedes-screw
plan: "02"
subsystem: playground-scene-rust
tags: [scene, archimedes-screw, live-controls, release-tests, tuning, d-19, d-20]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: "2026-10-14T21:57:23.481Z"
requires:
  - "47-01: ArchimedesScrewHooks, build_parts, geometry.rs, test_support.rs, spike guards"
provides:
  - "Live control `screw-speed` (canonical whole-number rpm 0..=60) and `screw-direction` (`forward` | `reverse`), both ControlEffect::Live, intent applied in on_advance"
  - "archimedes_screw/control_tests.rs: 9 debug-safe allowlist, intent-only and reverse-motion tests"
  - "archimedes_screw/flow_tests.rs: 8 release-only D-19/D-20/D-17 outcome tests"
  - "test_support: RegionCounts, region_counts, reservoir_surface_y, positions, train_transform, FlowReport, run_checked"
  - "geometry: INLET_COVER_Y, in_chute, TOP_FACE_TAN, hood quad, floor under the ramp start"
  - "Final rpm for Plan 03: DEFAULT 30, MAX 60 (unchanged)"
affects: [47-03, 47-04, 47-05]
tech-stack:
  added: []
  patterns:
    - "Particle identity (`particle_ids()`) to prove that water scooped after load reaches the basin, instead of comparing raw region counts"
    - "Bury every polygon seam that would reach a wet concave corner: overlap quads past the corner instead of sharing an edge"
    - "Cap open gaps above splashing water: low gravity turns ordinary 5 m/s kicks into multi-meter arcs"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/flow_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene/archimedes_screw.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/test_support.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/tests.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs
key-decisions:
  - "DEFAULT_SCREW_RPM stays 30 and MAX_SCREW_RPM stays 60: every D-19 rule and the README timing pass at 30 rpm with the 6 m tube"
  - "README timing is measured by particle identity (reservoir particles at load found in the basin at step 597, at least 20), because the plan's 'basin > tube count at load + 20' cannot hold: the basin overflows at about 150 particles and the tube holds 177 at load"
  - "Top housing face stands square to the axis (TOP_FACE_TAN = 0): the vane-parallel face pinned 2 particles inside the top vane at gravity 2"
  - "The flat reservoir floor runs 0.5 m under the ramp start: at gravity 80 a particle slid down the shared x = -4.0 seam between floor and ramp quads and fell out of the scene"
  - "A hood quad (x 1.9 to 2.45, y 2.72 to 3.0) closes the gap between the top housing and the chute wall: at gravity 2 a weir splash arced over the chute wall and left the scene"
requirements-completed: [SCREW-01, SCREW-02]
duration: 22min
completed: 2026-10-14
---

# Phase 47 Plan 02: Live Screw Controls and Release Outcome Tests Summary

**`screw-speed` (0 to 60 rpm) and `screw-direction` (forward or reverse) are live intent-only controls. Eight release tests show:**

- The screw lifts at least 145 particles into the basin over 60 s at 30 and at 60 rpm, with zero leaks, zero evictions and zero lost particles.
- Reverse returns water to the reservoir, and speed 0 holds the vanes still.
- Gravity 2 and 80 stay sealed.
- Reservoir water reaches the basin inside the 10 s README clip.

Three small geometry fixes were needed for the gravity 2 and 80 smokes.

## Performance

- **Duration:** about 22 min
- **Started:** 2026-10-14T21:35Z
- **Completed:** 2026-10-14T21:57Z
- **Tasks:** 2/2
- **Files:** 2 created, 5 modified

## Accomplishments

- `parse_screw_rpm` is a copy of `parse_drum_rpm` capped at `MAX_SCREW_RPM`. `apply_control` matches `(name, value)`: speed sets `rpm`, and the exact tokens `forward` and `reverse` set `reverse`. Every other name or value returns `Err(UnknownControl)` and leaves the state unchanged. `apply_control` never touches the world; `on_advance` still reads `axial_speed(self.rpm, self.reverse)` once per fixed step.
- Control tests (9, debug-safe):
  - The parser accepts canonical tokens and rejects malformed ones (`""`, `01`, `-5`, `+5`, `1.0`, padded values, `abc`, MAX + 1, `99999`).
  - A speed change stores intent without moving the train (position and velocity bits are unchanged).
  - Reverse sets the flag and forward clears it; bad direction values and bad speeds are rejected and leave state unchanged.
  - Foreign controls (`drum-speed`, `motor-direction`) and actions fail closed.
  - Through the session, speed and direction return `Ok(false)` and gravity returns `Ok(true)`.
  - One reversed step moves the train by `-axial_speed(DEFAULT) * SIM_DT` modulo PITCH.
- Release flow tests (8, each `#[cfg_attr(debug_assertions, ignore = ...)]`) use `run_checked`. It advances in chunks of 4 steps; after a 60-step settle it samples leaks every chunk and the reservoir p95 surface every 60th step. At the end it records region counts and the live particle count.

## Tuning Record

Settle 60 steps; leak and surface rules as in the plan facts. "Lost" is the count outside the reservoir, tube, basin and chute at the end. "README" is the number of reservoir-at-load particles in the basin at step 597 (basin total in parentheses).

| Iter | Change | DEFAULT/MAX rpm | Basin after 60 s (default / max) | Min reservoir surface (default / max) vs INLET_COVER_Y -0.0137 | Max inside walls / vanes | Lost | First pocket (step basin >= 10; basin at 300) | README | Reverse basin / reservoir delta | g 2 | g 80 | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 47-01 geometry | 30 / 60 | 161 / 139 | 0.452 / 0.506 | 0 / 0 | 0 | 71; 65 | 47 (148) | -45 / +132 | **2 inside vanes** (top vane against the vane-parallel top housing face) | **1 eviction** (fell through the floor/ramp seam at x = -4.0) | FAIL D-20 |
| 1 | Flat floor runs 0.5 m under the ramp start; rpm 24/30/36 probed | 30 / 60 | not rerun | not rerun | not rerun | not rerun | not rerun | not rerun | not rerun | still 1-2 inside the top vane at 24, 30 and 36 rpm (geometric, not speed) | sealed, 0 evictions at 30 and 36 | FAIL D-20 (g 2) |
| 2 | Top housing face square to the axis (`TOP_FACE_TAN = 0`) | 30 / 60 | not rerun | not rerun | not rerun | not rerun | not rerun | not rerun | not rerun | 0 inside vanes; at 30 rpm one weir splash (vy 4.9 m/s) arced through the 0.18 m gap between the housing's back corner and the chute wall top and left the frame | sealed | FAIL D-20 (g 2 escape) |
| 3 | Hood quad over the chute, buried in the top housing and the chute wall | 30 / 60 | **152 / 145** | **0.465 / 0.501** | **0 / 0** | **0** | **71; 65** | **41 (142)** | **-41 / +121** (basin 132 to 91) | **sealed, 0 evictions** | **sealed, 0 evictions** | **PASS** |

Final release run: `cargo test -p liquidfun-wasm --release --lib archimedes_screw` reports 60 passed, 0 failed and 0 ignored. That includes the 8 Plan 01 spike guards, re-run against the new geometry.

Thresholds (none below the plan's floors):

| Threshold | Value | Measured |
|---|---|---|
| `BASIN_GAIN_MIN` | 100 | 152 / 145; half the measured value is below the floor |
| `FIRST_POCKET_MIN` | **30** (floor 10) | 65 at step 300 |
| `README_SCOOPED_MIN` | 20 | 41 |
| `REVERSE_START_BASIN_MIN` | 120 | reached at step 480 |
| `REVERSE_BASIN_DROP_MIN` | 20 | 41 |
| `REVERSE_RESERVOIR_RISE_MIN` | **60** (floor 20) | 121 |
| `LOST_PERCENT_MAX` | 2 % | 0 |

## Final Values for Plan 03

- **DEFAULT_SCREW_RPM = 30**, **MAX_SCREW_RPM = 60**. Both are unchanged from 47-01; copy them into `web/src/catalog/archimedes-screw-speed.ts`.
- **Particle count: 1,729**, unchanged. The geometry changes do not touch any spawn point.
- **FRAME_\*** are unchanged: `FRAME_MIN_X = -6.1`, `FRAME_MIN_Y = -2.0`, `FRAME_MAX_X = 2.65`, `FRAME_MAX_Y = 4.45`. The new hood stays inside the old wall bounding box.
- **Geometry changes since 47-01-SUMMARY** (`WALL_QUAD_COUNT` 10 to 11):

| Point | 47-01 | Now |
|---|---|---|
| Top housing face corners (lower, upper) | (1.1505, 2.2695), (0.8918, 3.7367) | **(1.3712, 2.3969), (0.6712, 3.6093)** |
| Top housing far corners (back face) | (2.0165, 2.7695), (1.7578, 4.2367) | unchanged |
| Hood (new axis box) | none | **x 1.9 to 2.45, y 2.72 to 3.0** |
| Flat reservoir floor box right end | x = -4.0 | **x = -3.5** (under the ramp; its top edge is drawn inside the ramp quad as a short overlap line) |

  The top housing's lowest point is now 2.3969, which is still above the weir lip at 2.05 (D-06).

## Decision Coverage

- **D-09:** `screw-speed` is live, accepts canonical whole-number rpm from 0 through 60 and rejects everything else. The first pocket reaches the basin at step 71, within 5 s.
- **D-10:** `screw-direction` is live with exactly `forward` and `reverse`. Reverse negates the axial velocity at the current speed; both are applied in `on_advance`.
- **D-11:** Pointer input is unchanged (`Ok(())`, no effect).
- **D-08:** The minimum reservoir p95 surface over 60 s is 0.465 m at default and 0.501 m at maximum, far above `INLET_COVER_Y` (-0.0137).
- **D-17:** 41 reservoir particles reach the basin before step 597 at the default speed, with no cue.
- **D-19:** Default and maximum speed pass the 60 s checks, reverse passes (basin -41, reservoir +121), speed 0 leaves the drawn segments and the train position bit-identical, and the allowlist tests pass.
- **D-20:** Gravity 2 and 80 pass the 1,200-step smokes with zero leaks and zero evictions.

## Task Commits

1. **Task 1:** `97f66ad75` feat(47-02): add live screw speed and direction controls
1. **Task 2:** `24a0eecca` test(47-02): prove screw lift, reverse and stability in release runs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] README timing criterion measured by particle identity**

- **Found during:** Task 2.
- **Issue:** The plan's check was "basin count at step 597 > tube-region count at load + 20". It cannot pass at any speed. The weir caps the basin at about 140-160 particles, while the tube holds 177 at load: the flooded inlet plus the pre-filled pockets.
- **Fix:** Record the `particle_ids()` of reservoir-region particles at load, then count those ids inside the basin at step 597 (at least `README_SCOOPED_MIN = 20`). This tests the plan's intent directly: water scooped after load reaches the basin before the clip ends. The measured value is 41.
- **Commit:** `24a0eecca`.

**2. [Rule 1 - Bug] Reservoir floor/ramp seam let a particle out at gravity 80**

- **Issue:** The flat floor box and the ramp quad shared the edge x = -4.0, y -1.8 to -1.5. Under pressure, a particle in the wet corner slid down that internal edge and fell 12 m, so it was evicted.
- **Fix:** `FLOOR_UNDER_RAMP = 0.5` extends the floor box under the ramp, which buries the ramp's start edge.

**3. [Rule 1 - Bug] Vane-parallel top housing face pinned particles at gravity 2**

- **Issue:** Two particles stayed more than r/2 inside the top vane, pressed against the vane-parallel top housing face. This happened at 24, 30 and 36 rpm, so changing the rpm would not fix it. It is the same parallel-closing-face pinch that 47-01 fixed on the bottom face.
- **Fix:** `TOP_FACE_TAN = 0.0` stands the face square to the axis, so a rising vane meets it at the roof first and the wedge opens down into the outlet. `visible_vane_outline` now clips against both faces with one generic `clip_by_face`. The top-face outline test now checks that no drawn corner lies inside the housing, like the bottom-face test. The forward-snap containment test still passes.

**4. [Rule 1 - Bug] Weir splash left the scene at gravity 2**

- **Issue:** A particle kicked at the weir lip (vy of about 4.9 m/s) arced through the 0.18 m gap between the top housing's lower back corner and the chute wall top. It flew out of the frame and would have been evicted.
- **Fix:** A hood quad buried in both the top housing and the chute wall closes the gap. The new test `the_hood_closes_the_gap_between_the_top_housing_and_the_chute` guards it.

**5. [Plan-directed] Plan 01 fail-closed control test moved**

- `controls_and_actions_fail_closed` left `hooks_tests.rs`. Its behavior is covered by `other_scene_controls_and_actions_fail_closed` plus the new allowlist tests in `control_tests.rs`.

**6. [Minor] Thresholds raised where the margin was large**

- `FIRST_POCKET_MIN` went from 10 to 30 (measured 65). The reverse reservoir rise went from 20 to 60 (measured 121) and became its own constant, separate from the basin drop (20, measured 41).

TDD note: Task 1 went red first (unresolved `SCREW_*`/`parse_screw_rpm` imports), then green. Tests and implementation share one commit, as the plan's single commit message specifies.

## Known Stubs

None.

## Deferred Issues

- The pre-existing release-only `unreachable expression` warning in `crates/liquidfun/src/particle/solver/boundary/support.rs:98` is still out of scope; it is already in `deferred-items.md`.

## Verification

- `cargo fmt --all --check`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo build --all-targets --all-features`: all clean.
- `cargo test -p liquidfun-wasm --release --lib archimedes_screw`: 60 passed, 0 failed, 0 ignored.
- `cargo test -p liquidfun-wasm --all-features`: 679 passed, 0 failed, 52 ignored. `archimedes_screw::flow_tests` in debug: 8 ignored.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `grep -c 'cfg_attr(debug_assertions, ignore'` in `flow_tests.rs` is 8, which equals its 8 `#[test]` attributes. There is no `eprintln!`/`println!` in the screw module and no `unwrap()` in `archimedes_screw.rs`.
- `git diff --quiet HEAD~2 -- crates/liquidfun web docs/benchmarks/scene-fingerprints.json`: unchanged.

## Self-Check: PASSED

Both new test files, this SUMMARY, and commits 97f66ad75 and 24a0eecca exist.

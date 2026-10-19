---
phase: 49-rayleigh-taylor-instability
plan: "02"
subsystem: playground-scene-rust
tags: [scene, rayleigh-taylor, particle-force, release-tests, determinism, d-18]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: "2026-10-19T02:29:32Z"
requires:
  - "49-01: RayleighTaylorHooks, build_parts, TUNING, test_support metrics, spike_tests runner"
  - "48-01: particle_force::{GravityFrame, ForceScale, apply_group_acceleration}"
provides:
  - "scene/rayleigh_taylor/force_lane_tests.rs: 6 debug-safe settle-twin and force-lane tests (D-02, D-03, D-18)"
  - "scene/rayleigh_taylor/flow_tests.rs: 9 release-only D-18 session outcome tests (RT-01, RT-02, SCENE-02)"
  - "scene/rayleigh_taylor/test_support.rs: force_rows, vector_bits, velocity_bits, fixed_step, centroid, Stability + assert_bounded, session advance/heavy_group_of/session_layers/session_bits/run_session_stability"
affects: [49-03, 49-04, 49-05]
tech-stack:
  added: []
  patterns:
    - "Force-lane snapshot diff: read forces() right before and right after on_advance (before world.step) and assert on the difference, since wall reaction forces already sit in the lane"
    - "Session tests find the heavy group as the group of the highest particle at step 0"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/force_lane_tests.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/flow_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/test_support.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs
key-decisions:
  - "The plan's hooks_tests.rs is named force_lane_tests.rs, so it is not confused with Plan 01's hook_tests.rs (435 lines, too full to absorb the six tests under the 628-line limit)"
  - "TUNING is unchanged from 49-01-SUMMARY; every D-18 release rule passed on the first run"
  - "The S4 stability recorder and the bare-parts fixed step moved from spike_tests into test_support, with no change to any spike rule or threshold"
patterns-established:
  - "Snapshot-diff force-lane assertions for scenes whose walls leave reaction forces in the lane"
requirements-completed: []
duration: "~16 min"
completed: 2026-10-18
---

# Phase 49 Plan 02: Rayleigh–Taylor D-18 Outcome Tests Summary

**Six debug twin and force-lane tests show three things. The settle window adds no extra weight. After the ramp, the extra weight follows a 45-degree tilted gravity onto heavy rows only. Nine release session tests show fingers at step 114 and overturn at step 156. They also show bit-identical replays at every ratio, a 0.74 m heavy-layer shift toward a sideways gravity, and stable 60 s and gravity 2/80 runs. `TUNING` is unchanged.**

## Performance

- **Duration:** about 16 min
- **Started:** 2026-10-19T02:13:56Z
- **Completed:** 2026-10-19T02:29:32Z
- **Tasks:** 2 of 2
- **Files modified:** 5 (2 created, 3 modified)

## Accomplishments

- `force_lane_tests.rs` (debug-safe, not ignored): the twin worlds' velocities are bit-identical for all 36 settle steps, and the force lane is bit-identical across `on_advance` during the settle. The first ramped step (step 36, ramp 1/24) adds `scale.force(-up * 1.0 * |g| / 24)` to every heavy row. Under gravity (7.071068, −7.071068) with the ramp complete, every heavy row gains `scale.force(-up * |g|)` within 1e-5 relative, and every light row stays bit-identical. Zero gravity returns Ok and leaves the whole lane bit-identical.
- A mutation check confirmed the tilt test is meaningful. With a temporary change to `on_advance` that always pushes along −y, `extra_weight_follows_tilted_gravity` fails, and the file was restored before commit.
- `flow_tests.rs` (release-only): all nine D-18 session outcomes pass on the first run. No tuning iteration was needed.

## Task Commits

1. **Task 1: debug settle-window twin and force-lane tests**: `8a2ea1b14` (test)
1. **Task 2: release session tests for fingers, overturn, replay, sideways gravity, 60 s ratio 3 and gravity smokes**: `05e83b20f` (test)

**Plan metadata:** the docs commit that follows this SUMMARY

## Measured Outcomes (release, native)

| Test | Measured | Limit |
|------|----------|-------|
| `default_ratio_grows_three_fingers_by_four_seconds` | first ≥ 3 fingers at step 114 (1.9 s), counted against the settled interface at step 36 | ≤ 240 |
| `default_ratio_overturns_by_ten_seconds` | first overturned sample at step 156 (2.6 s) | ≤ 600 |
| `each_ratio_replays_to_the_same_end_state` | 1.2, 1.5, 2 and 3: bit-identical positions and velocities after 600 steps (two fresh sessions each, `apply_control("weight-ratio", ..)` → `Ok(true)`) | identical |
| `ratio_three_differs_from_the_default_ratio` | default and ratio-3 sessions differ after 120 steps | differ |
| `sideways_gravity_moves_the_heavy_layer_toward_it` | heavy − light centroid along (−1, 0): 0.005 m → 0.744 m, gain 0.738 m in 120 steps | ≥ 0.2 m |
| `ratio_three_runs_sixty_seconds_without_eviction` | live 1,484 = built 1,484; out of tank 0; max speed early 4.29, late 2.36, overall 4.29 m/s; every step Ok | ≤ 6.0, late ≤ early |
| `gravity_two_smoke_stays_stable` | live = built = 1,484; out of tank 0; max speed 2.65 m/s | no eviction, 0 out |
| `gravity_eighty_smoke_stays_stable` | live = built = 1,484; out of tank 0; max speed 8.40 m/s (recorded only) | no eviction, 0 out |
| `gravity_eighty_ratio_three_smoke_stays_stable` | live = built = 1,484; out of tank 0; max speed 8.40 m/s (recorded only) | no eviction, 0 out |

The session results match the Plan 01 `build_parts` spike exactly: fingers at step 114, overturn at step 156, and ratio-3 speeds of 4.29 and 2.36 m/s. The session's eviction pass never fired.

At gravity 80, max speed sits at 8.40 m/s, the 0.07 × 120 critical-velocity cap at r 0.035. That is above the gravity-10 `MAX_SPEED` bound, so it is recorded here and not asserted, as the plan specifies.

### Gravity 80 × ratio 3 layer heights (Research Pitfall 4, record only)

Particle-height quantiles from a session at gravity 80 and ratio 3. The light layer's thickness is its p01 to p99 span.

| Step | Light p01 / p99 (span) | Heavy p01 / p99 | Interface | Overturned |
|------|------------------------|-----------------|-----------|------------|
| 0 | 0.035 / 0.823 (0.788) | 0.665 / 1.452 | 0.770 | no |
| 36 (end of settle) | 0.020 / 0.837 (0.817) | 0.123 / 1.051 | 0.400 | no |
| 120 | 0.031 / 0.723 (0.692) | 0.005 / 0.310 | 0.241 | yes |
| 600 | 0.135 / 0.692 (0.557) | 0.004 / 0.264 | 0.246 | yes |
| 1,200 | 0.176 / 0.668 (0.492) | 0.005 / 0.250 | 0.238 | yes |

At gravity 80 the column squashes during the settle: the heavy top falls from 1.45 m to 1.05 m by step 36. The heavy layer then sinks within 2 s and packs into a layer about 0.25 m thick on the floor. The settled light layer on top is about 0.49 m thick (0.18 to 0.67 m), down from 0.79 m at load. Nothing leaves the tank. As D-18 and Phase 46 D-07a specify, no finger rule applies at gravity 80.

## TUNING

Unchanged since 49-01-SUMMARY (r 0.035, WATER, viscous 0.25, repulsive 1.0, pressure strength 2.0, damping 0.2, seed C, settle 36, ramp 24, 1,484 particles). After the shared-helper refactor, the Plan 01 spike guards still pass: `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor` reports 77 passed, 0 ignored.

## D-18 Bullet-to-Test Map

| D-18 bullet | Test(s) | File |
|-------------|---------|------|
| ≥ 3 fingers past the 20% line by about 4 s (ratio 2, g 10) | `default_ratio_grows_three_fingers_by_four_seconds` (session); spike guard `default_ratio_grows_three_fingers_by_step_240` | flow_tests.rs; spike_tests.rs |
| Layers overturn by about 10 s | `default_ratio_overturns_by_ten_seconds` (session); spike guard `default_ratio_overturns_by_step_600` | flow_tests.rs; spike_tests.rs |
| Same settings give the same end state at every ratio | `each_ratio_replays_to_the_same_end_state`, `ratio_three_differs_from_the_default_ratio`; spike guard `two_default_runs_are_bit_identical` | flow_tests.rs; spike_tests.rs |
| Extra weight follows tilted gravity; heavy centroid moves along the new gravity | `extra_weight_follows_tilted_gravity`, `extra_weight_leaves_the_light_layer_alone`, `zero_gravity_applies_no_extra_weight` (debug); `sideways_gravity_moves_the_heavy_layer_toward_it` (release) | force_lane_tests.rs; flow_tests.rs |
| No extra weight during the settle window | `settle_window_applies_no_extra_weight`, `settle_window_writes_no_force`, `extra_weight_starts_after_the_settle_window`; Plan 01 `ramp_is_zero_through_the_settle_window` | force_lane_tests.rs; hook_tests.rs |
| 60 s at ratio 3: no eviction, no failed step, bounded max speed | `ratio_three_runs_sixty_seconds_without_eviction` (session); spike guard `ratio_three_runs_sixty_seconds_bounded` | flow_tests.rs; spike_tests.rs |
| Gravity 2 and 80 smokes: no eviction, no failed step | `gravity_two_smoke_stays_stable`, `gravity_eighty_smoke_stays_stable` | flow_tests.rs |
| Gravity 80 × ratio 3 smoke | `gravity_eighty_ratio_three_smoke_stays_stable` | flow_tests.rs |
| Parser accepts each listed ratio, rejects unknown values; `apply_control` → `Recreated` | Plan 01: `weight_ratio_accepts_each_token_with_its_exact_extra_weight`, `weight_ratio_rejects_malformed_tokens`, `weight_ratio_default_is_two_and_matches_the_default_token` (tests.rs); `build_rejects_unknown_presets_and_tokens`, `presets_pick_the_default_or_the_listed_ratio`, `last_weight_ratio_preset_wins`, `hooks_recreate_for_each_listed_ratio`, `hooks_reject_unknown_control_values_and_names`, `session_recreates_for_a_listed_ratio`, `session_rejects_an_unlisted_ratio` (hook_tests.rs) | tests.rs; hook_tests.rs |

## Deviations from Plan

### File naming

**1. [Orchestrator note] `hooks_tests.rs` is named `force_lane_tests.rs`**
- **Issue:** Plan 01 already created `rayleigh_taylor/hook_tests.rs` (435 lines). A second file named `hooks_tests.rs` would be almost identical in name. Folding the six tests into `hook_tests.rs` would bring it near or over the 628-line limit.
- **Fix:** The tests live in `force_lane_tests.rs` (203 lines), in the same lib test binary. The plan's Task 1 acceptance greps were run against this file: `7.071_068` matches, `forces()` matches in test_support.rs, `to_bits` matches in test_support.rs, `1e-3` has no match, and `cfg_attr(debug_assertions, ignore` has 0 matches.
- **Commit:** `8a2ea1b14`

### Shared helpers

**2. [Plan-directed refactor] Spike stability recorder moved to test_support**
- The plan asked for shared runner pieces to move out of `spike_tests.rs`. `RunReport`'s four speed and containment fields are now one `stability: Stability` field. `record_stability`, `assert_bounded`, `STABILITY_SAMPLE_EVERY` and the early and late window constants moved to `test_support.rs`, and the spike's local `advance(parts)` became `test_support::fixed_step`. The rules, thresholds and sampling steps are unchanged, and all 10 spike guards still pass.
- **Commit:** `05e83b20f`

No other deviations. No hooks or thresholds changed.

## Known Stubs

None.

## Notes for Later Plans

- At gravity 80, max speed reaches the 8.4 m/s critical-velocity cap. Any future gravity-80 speed rule needs its own bound; it cannot reuse `MAX_SPEED`.
- `crates/liquidfun`, `web/` and `docs/benchmarks/scene-fingerprints.json` are untouched (`git diff --quiet HEAD~2 -- …` is clean).

## Self-Check: PASSED

- Files exist: `force_lane_tests.rs` (203 lines), `flow_tests.rs` (248), `test_support.rs` (453), `spike_tests.rs` (310), `rayleigh_taylor.rs` (353); every file is under 628 lines.
- Commits `8a2ea1b14` and `05e83b20f` are in `git log`.
- Debug: `cargo test -p liquidfun-wasm --all-features` passed 804 with 102 ignored, including the 9 flow tests, which show as ignored in debug. `rayleigh_taylor::force_lane_tests` passed 6 with 0 ignored.
- Release: `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor` passed 77 with 0 ignored. `cargo test -p liquidfun-wasm --all-features --release` passed 895.
- `cargo fmt --all`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo build -p liquidfun-wasm --all-targets --all-features` and `bun scripts/bright-builds-check.ts all` (0 findings) pass.
- `flow_tests.rs` has 9 `cfg_attr(debug_assertions, ignore` lines for 9 `#[test]`s. `apply_control("gravity", "80")` appears 2 times and `apply_control("weight-ratio"` appears once. `live_particle_count` matches in test_support.rs. The `eprintln!` and `println!` grep finds nothing; the temporary measurement prints were removed before commit.

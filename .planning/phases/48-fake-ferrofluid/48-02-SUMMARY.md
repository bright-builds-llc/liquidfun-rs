---
phase: 48-fake-ferrofluid
plan: "02"
subsystem: playground-scene-rust
tags: [scene, fake-ferrofluid, pointer, action, live-control, marker, gravity-frame, release-tests, d-23, readme-timing]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: "2026-10-15T06:21:54.284Z"
requires:
  - "48-01: FakeFerrofluidHooks, FIELD, geometry, surface-peak metric and the spike hold runner"
  - "46/47: pointer toggle pattern (cartesian_diver), parse_whole_rpm live control (archimedes_screw)"
provides:
  - "FakeFerrofluidHooks pointer seam: Down anywhere turns the magnet on at the clamped press point, Move drags while pressed, Up and Cancel (one arm) switch off and park at MAGNET_REST"
  - "MAGNET_ACTION \"magnet\" toggles in place; MAGNET_STRENGTH_CONTROL \"magnet-strength\" takes 0..=100 via parse_whole_rpm and returns ControlEffect::Live; every other name fails closed"
  - "geometry::clamp_magnet (frame inset by the bar half extents) and geometry::magnet_marker (gravity-frame bar: 8 segments on, 4 off, axis-aligned outline without a frame)"
  - "collect_segments = dish loops then the marker; collect_circles empty"
  - "18 release-only D-23 flow tests plus 22 debug control tests"
  - "test_support: HoldReport, HoldRecorder, assert_hold_is_bounded, HOLD_POSITIONS (7), session helpers advance/profile/first_step_where/run_session_hold"
affects: [48-03, 48-04, 48-05, 49]
tech-stack:
  added: []
  patterns:
    - "Intent-only seam hooks: pointer, action and control store magnet_on, magnet_position, pointer_held and strength_percent; on_advance reads them once per fixed step"
    - "Marker drawn from GravityFrame::from_gravity(world.gravity()) inside collect_segments, appended after static outlines so their indices never shift"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/control_tests.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/flow_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/spike_tests.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/tests.rs
key-decisions:
  - "Shipped the |E(h)| sideways-term fallback: with the odd E(h), a magnet pressed above the puddle (y 1.0) formed 1 peak and the above-puddle drag failed its precondition"
  - "WALL_TOP_Y 1.8 -> 3.0 m (FRAME_MAX_Y 2.4 -> 3.6): with |E(h)| a magnet held 1.3-1.5 m up lifts columns to 2.67 m; 2.5 m walls passed the written P4 positions but left those columns above the walls"
  - "P4 now also guards (0, 1.3) and (3.2, 1.5), the measured tallest-column positions"
  - "DEFAULT_STRENGTH_PERCENT stays 65; FIELD numeric values unchanged"
requirements-completed: []
requirements-advanced: [FERRO-01, FERRO-02, FERRO-03]
duration: 43min
completed: 2026-10-15
---

# Phase 48 Plan 02: Magnet Seam, Marker and D-23 Outcomes Summary

**The magnet now follows press and drag. It parks off under the dish on Up or Cancel, toggles with `magnet`, takes a live `magnet-strength` from 0 to 100, and is drawn as a gravity-oriented bar marker. 18 release session tests prove every D-23 outcome and the README timing. To get there I shipped the planned `|E(h)|` sideways-pull fallback and raised the dish walls from 1.8 m to 3.0 m.**

## Performance

- **Duration:** 43 min
- **Started:** 2026-10-15T05:39:15Z
- **Completed:** 2026-10-15T06:21:54Z
- **Tasks:** 2
- **Files modified:** 8 (2 created, 6 modified)

## Accomplishments

- **Seam (Task 1).** `apply_pointer` follows the research Pattern 2. Down sets `pointer_held` and `magnet_on` and moves the magnet to `clamp_magnet(p)`. Move while pressed follows the pointer. Up and Cancel share one arm and always switch the magnet off and park it at `MAGNET_REST`, whether or not a press was held. An unpressed Move is an `Ok` no-op.
- **Action and control.** `apply_action("magnet")` flips `magnet_on` without moving the magnet. `apply_control("magnet-strength", v)` takes `parse_whole_rpm(v, 100)`, stores the value and returns `ControlEffect::Live`. Every other name fails with `UnknownControl`, and no hook touches the world.
- **Marker.** `magnet_marker` builds the bar in the gravity frame. The outline is 0.8 m along `across` by 0.2 m along `up`. When on, it adds a pole divider and three 0.16 m ticks starting 0.04 m above the top face, at offsets -0.25, 0 and +0.25. That is 8 segments on and 4 off. Without a frame it draws an axis-aligned outline. `collect_segments` returns the 12 dish segments first, then the marker.
- **Tests.** 22 debug control tests cover the pointer arms, the clamp at (50, -50), session `InvalidPointer` with a bit-identical frame, the action toggle and allowlists, all 10 malformed strength tokens, the live/rebuild session results, world bits unchanged after every hook, and the marker counts, flush top face, quarter-turn and None fallback. They also check segment order.
- **Release flow tests.** 18 release-only tests in `flow_tests.rs` run through `SessionCore` with `"down"`, `"move"`, `"up"`, `"cancel"`, `magnet` and `magnet-strength`.

## Measured Outcomes (native release, gravity 10, 990 particles, final field and 3.0 m walls)

Peaks and flatness are sampled every 6 steps; the README run samples every 3 steps.

| Test | Measurement | Rule | Result |
|---|---|---|---|
| `load_forms_four_peaks_within_ninety_steps` | first >= 4 peaks at step **12** (0.2 s) | <= 90; D-04 about 60 | pass, under 60 |
| `default_strength_shows_five_to_nine_spikes` | **5** peaks at step 180 | 5..=9 | pass |
| `dragging_the_magnet_off_center_moves_the_crown_toward_it` (under the dish) | 5 -> 4 peaks, centroid -0.048 -> 1.120, shift **1.168 m** | shift >= 0.6, peaks >= 4 | pass |
| `dragging_the_magnet_inside_the_puddle_...` (press y 0.3) | 5 -> 4 peaks, centroid -0.048 -> 1.120, shift **1.168 m** | same | pass |
| `dragging_the_magnet_above_the_puddle_...` (press y 1.0) | 5 -> 4 peaks, centroid -0.048 -> 1.120, shift **1.168 m** | same | pass |
| `pointer_up_flattens_the_puddle` | flat at step **30** after Up | <= 90 | pass |
| `pointer_cancel_flattens_the_puddle` | flat at step **30** after Cancel | <= 90 | pass |
| `magnet_action_off_flattens_the_puddle` | flat at step **30** after the action | <= 90 | pass |
| `magnet_action_on_reforms_four_peaks_within_ninety_steps` | flat 30 steps after off; >= 4 peaks **18** steps (0.3 s) after on | <= 90; D-04 about 60 | pass, under 60 |
| `strength_zero_keeps_the_puddle_flat` | flat at 90 and 180; smoothed range 0.016 m; marker still on (dish + 8 segments) | flat both times | pass |
| `gravity_two_smoke_stays_stable` | 1,200 steps, every advance Ok, live 990 = built 990 | no eviction or failed step | pass |
| `gravity_eighty_smoke_stays_stable` | 1,200 steps, every advance Ok, live 990 = built 990 | same | pass |
| `readme_clip_shows_spikes_collapse_and_rerise` | **6** peaks at step 357; flat at step **393**; >= 4 peaks at step **495** | >= 4 @357; flat by 477; >= 4 by 597 | pass; Plan 03 keeps cue samples 120 and 160 |

Neither appearance step (12 on load, 18 after the action turns the magnet back on) exceeds D-04's 60 steps, so there is no D-04 deviation.

### Drag measurements by sideways variant

| Drag | `E(h)` (48-01 field, 1.8 m walls) | `|E(h)|` (shipped) |
|---|---|---|
| Under the dish | 5 -> 4, -0.048 -> 1.120, shift 1.168 | 5 -> 4, -0.048 -> 1.120, shift 1.168 |
| Press y 0.3 (inside) | 5 -> 4, -0.032 -> 1.120, shift 1.152 | 5 -> 4, -0.048 -> 1.120, shift 1.168 |
| Press y 1.0 (above) | **1 peak before (precondition FAIL)**, 0 after | 5 -> 4, -0.048 -> 1.120, shift 1.168 |

### 60 s holds at strength 100, pointer held (3,600 steps through the session)

Early and late are the 120..720 and 3,000..3,600 windows. KE h1 and h2 are the means over 120..1,860 and 1,860..3,600. Out, escaped and isolated are the maxima over samples every 30 steps. Top is the highest particle center over all steps. Every run ended with live = built = 990.

| Held at | Variant | Max speed all / early / late (m/s) | KE h1 / h2 | Out | Esc | Iso | Top (m) |
|---|---|---|---|---|---|---|---|
| under the dish (0, -0.35) | `E` and `|E|` (identical, all liquid above the magnet) | 1.924 / 0.604 / 0.603 | 2.496 / 2.499 | 0 | 0 | 0 | 0.885 |
| off-center (2.0, -0.35) | `E` and `|E|` (identical) | 2.008 / 0.609 / 0.604 | 2.295 / 2.121 | 0 | 0 | 0 | 0.890 |
| above the puddle (0, 1.2) | `E` | 1.879 / 0.613 / 0.610 | 1.483 / 1.398 | 0 | 0 | 0 | 0.674 |
| above the puddle (0, 1.2) | **`|E|`** | 2.052 / 1.347 / 0.603 | 5.624 / 3.551 | 0 | 0 | 0 | **2.159** |
| inside the puddle (0, 0.3) | `E` | 1.335 / 0.611 / 0.604 | 1.898 / 1.892 | 0 | 0 | 0 | 1.145 |
| inside the puddle (0, 0.3) | **`|E|`** | 1.337 / 0.933 / 0.989 | 32.078 / 31.671 | 0 | 0 | 0 | 1.498 |
| inside at the wall (2.7, 0.3) | `E` | 1.333 / 0.603 / 0.608 | 1.682 / 1.626 | 0 | 0 | 0 | 1.212 |
| inside at the wall (2.7, 0.3) | **`|E|`** | 1.336 / 0.925 / 0.913 | 18.953 / 18.589 | 0 | 0 | 0 | 1.397 |

Every shipped hold meets the P3 rules: max speed at most 2.06 (rule 4.0), late at most early + 0.06 (rule + 0.25), KE h2 below h1 in every case, and no isolated particles. The session holds reproduce the Plan 01 `build_parts` holds bit for bit.

### Tallest-column probe (`|E|`, strength 100, 3,600 steps; identical with 2.5 m and 3.0 m walls)

| Magnet | Top (m) | Out / Esc / Iso | Max speed |
|---|---|---|---|
| (0, 1.0) / (0, 1.1) / (0, 1.2) | 1.932 / 2.236 / 2.159 | 0 / 0 / 0 | <= 2.06 |
| (0, 1.3) / (0, 1.4) / (0, 1.5) | **2.667** / 2.404 / 2.664 | 0 / 0 / 0 | <= 2.07 |
| (0, 1.6) | 0.865 (surface out of reach) | 0 / 0 / 0 | 2.05 |
| (+-3.2, 1.0 / 1.2 / 1.3 / 1.4 / 1.5) | 2.133, 2.125 / 2.111, 2.035 / 2.454, 2.436 / 2.200, 2.545 / **2.666**, 2.302 | 0 / 0 / 0 | <= 2.14 |

The column top follows where the lift exceeds `G`: `L * S * bump(h / H) > 1` gives about `magnet + 0.606 H` (0.97 m at full strength). The worst case is a magnet held just low enough for the levitation band to reach the puddle surface.

## Final Values

- `DEFAULT_STRENGTH_PERCENT` **65** (unchanged), `MAX_STRENGTH_PERCENT` 100.
- `FIELD` (numbers unchanged): wavelength 0.65, envelope_half_width 2.6, reach 1.6, strength_scale 2.5, cap 1.5, drag_per_second 3.0. **Sideways term: `|E(h)|`** (`bump_integral(h, reach).abs()`); the lift and cap are unchanged.
- Particles: 990.
- **Geometry changed since 48-01-SUMMARY:** `WALL_TOP_Y` **3.0** (was 1.8), so **`FRAME_MAX_Y` 3.6** (was 2.4). `FRAME_MIN_X` -3.6, `FRAME_MAX_X` 3.6 and `FRAME_MIN_Y` -0.8 are unchanged, as are the dish width, floor and magnet rest.
- **For Plan 03:** `FAKE_FERROFLUID_VIEW_BOUNDS` = minX -3.6, minY -0.8, maxX 3.6, maxY **3.6**.
  - Portrait wall points: wall tops `(-3.3, 3.0)`, `(-3.0, 3.0)`, `(3.0, 3.0)`, `(3.3, 3.0)`; slab bottom corners `(-3.3, -0.25)`, `(3.3, -0.25)`; resting marker bottom corners `(-0.4, -0.45)`, `(0.4, -0.45)`.
  - Magnet clamp: x in [-3.2, 3.2], y in [-0.7, 3.5].

## Task Commits

1. **Task 1: pointer seam, magnet action, magnet-strength control and marker** — `53c65c4bc` (feat; tests written first, RED as a compile failure on the missing items, then GREEN)
1. **Task 2: release D-23 outcome tests, |E(h)| fallback, 3.0 m walls** — `443e24631` (test)

## Decisions Made

See the key-decisions frontmatter. The tuning used 3 of the 4 allowed iterations:

1. `|E(h)|` fallback. All three drags pass, but P4 fails at the (0, 1.2) hold (top 2.159).
1. Walls 2.5 m. Every written rule passes, but the probe found 2.67 m columns above the walls.
1. Walls 3.0 m. Everything passes with a 0.33 m margin over the worst measured column.

No threshold was weakened.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Workspace pedantic clippy findings in the new test code**
- **Found during:** Task 2
- **Issue:** `manual_is_multiple_of` in the shared hold recorder, and `struct_field_names` (every field ending in `_step`) on the README report.
- **Fix:** I used `step.is_multiple_of(..)` and renamed the fields to `crown_peaks`, `maybe_flat_at` and `maybe_rerise_at`.
- **Files modified:** fake_ferrofluid/{test_support,flow_tests}.rs
- **Commit:** 443e24631

**2. [Rule 2 - Missing critical guard] P4 strengthened with the measured tallest-column positions**
- **Found during:** Task 2, tuning iteration 2
- **Issue:** With `|E(h)|`, a magnet held 1.3-1.5 m up lifts columns to 2.67 m. The P4 positions alone allowed 2.5 m walls, which those columns overtop. The magnet clamp reaches x 3.2, outside the inner face, so an overtopping column could be drawn over a wall.
- **Fix:** `WALL_TOP_Y` is now 3.0, the Plan 01 rule applied to the worst measured column. `HOLD_POSITIONS` (the P4 set) now also includes `HIGH_ABOVE_THE_PUDDLE` (0, 1.3) and `HIGH_AT_THE_EDGE` (3.2, 1.5).
- **Files modified:** fake_ferrofluid/{geometry,test_support,spike_tests}.rs
- **Commit:** 443e24631

### Planned fallback applied

- The `<facts>` odd-`E(h)` fallback fired: `dragging_the_magnet_above_the_puddle_...` saw 1 peak before the drag. I changed only the sideways term. The lift and the cap are unchanged, and the curl test (`h > 0` samples) still passes. The field.rs `//!` now says the field is conservative above the magnet only. The Plan 01 spike suite and P4 were re-run, as were all Plan 02 holds.

### Notes

- Collapse tests press at `MAGNET_REST` at load, hold through `SETTLE_STEPS`, then release with Up, Cancel or the action. This exercises a real press-release with the default rest crown, per research Pitfall 5.
- `strength_zero_keeps_the_puddle_flat` checks that the magnet is still on through the frame's segment count, dish plus 8 marker segments, because the session hides the hooks.
- I measured with temporary `eprintln!` probes and an ignored probe test, then removed both before the commit. The grep for `eprintln!` and `println!` is empty.
- Out of scope: the pre-existing `unreachable expression` warning at `particle/solver/boundary/support.rs:98` in the release engine build. It is already listed in REQUIREMENTS.md.

## Issues Encountered

- **Visual risk for Plan 04 review:**
  - With `|E(h)|` the field is not conservative below the magnet. With the magnet held inside the puddle, the liquid keeps a steady slow churn: KE about 32 versus 1.9 with `E(h)`, max speed 1.34 m/s and no growth.
  - A magnet held 1.0-1.5 m up gathers liquid into columns that rise about 1 m above the magnet.
  - Both pass every stability rule. Check them by eye in the four render modes.
- **Look trade-off:** 3.0 m walls make the 6 m dish a deeper tray than D-15's "shallow dish". The landscape frame becomes 7.2 x 4.4 m instead of 7.2 x 3.2 m. Plan 03 sizes the view and portrait bounds from the values above.

## Known Stubs

None. Every seam is wired. The web labels ("Magnet strength", "Magnet") and the catalog entry are Plan 03's scope.

## Threat Model Coverage

- T-48-06: `parse_whole_rpm(value, 100)`; 10 malformed tokens are rejected and the strength is unchanged (`magnet_strength_rejects_malformed_tokens_and_keeps_the_strength`).
- T-48-07: `InvalidPointer` leaves the frame bit-identical; the (50, -50) clamp is tested.
- T-48-08: five 60 s full-strength held-pointer holds and the g 2/80 smokes pass.
- T-48-09: other actions and other scenes' controls fail closed (`other_actions_fail_closed_and_change_nothing`, `other_scene_controls_fail_closed`).

## Verification

- `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`: 76 passed, 0 failed, 0 ignored.
- `cargo test -p liquidfun-wasm --all-features --lib fake_ferrofluid::flow_tests`: 0 passed, 18 ignored. All 18 tests carry `cfg_attr(debug_assertions, ignore`.
- `cargo test -p liquidfun-wasm --all-features`: 741 passed, 81 ignored.
- `cargo fmt --all --check`, both clippy commands with `-D warnings` and `cargo build --all-targets --all-features` pass. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `git diff --quiet HEAD~2 -- crates/liquidfun web docs/benchmarks/scene-fingerprints.json` passes.
- File lengths: control_tests 455, flow_tests 471, tests 559, test_support 407, spike_tests 301, geometry 177, field 152, fake_ferrofluid.rs 280 (all <= 628).

## Next Phase Readiness

- Plan 03 can register the scene in the web catalog with `magnet-strength` (default 65, max 100, step 1) and the `magnet` action. It mirrors `FRAME_*` (-3.6, -0.8, 3.6, **3.6**) and uses the portrait wall points above.
- The README cues stay at samples 120 and 160. The native timing passes with margin: flat by step 393 against a 477 deadline, re-risen by 495 against 597.

## Self-Check: PASSED

//! Release-only D-06 spike for the Rayleigh–Taylor tuning, kept as
//! regression guards. The pass rules were written before measuring
//! (49-01-PLAN "D-06 pass rules", amended by 49-CONTEXT D-06a after the
//! first sweep passed only because the column collapsed while it settled;
//! 60 Hz steps; sample every 6 steps unless stated):
//!
//! - S0 no slump (D-06a): ratio 2, g 10: the fluid top (99th-percentile
//!   particle height) drops by at most 10% of the inner tank height between
//!   step 0 and the end of the settle window.
//! - Settled interface (D-06a): the mean heavy/light interface height at the
//!   end of the settle window. The 20% depth line is that height minus
//!   0.2 x the inner tank height. Fingers count only from the first step with
//!   extra weight, and the end-of-settle frame must show fewer than 3 fingers.
//! - S1 fingers: ratio 2, g 10: the first sample after the settle with
//!   `finger_count >= 3` is at step <= 240 (4.0 s).
//! - S2 overturn: ratio 2, g 10: the first sample with `is_overturned` is at
//!   step <= 600 (10.0 s).
//! - S3 slow ratio: ratio 1.2, g 10: the first overturned sample is at step
//!   <= 1,200 (20 s). Record ratio 1.5 and 3 finger and overturn steps too
//!   (record only).
//! - S4 stability: ratio 2 and ratio 3, g 10, 3,600 steps (60 s) each: every
//!   `on_advance` and `world.step` is Ok; `out_of_tank_count == 0` at every
//!   30th step; max speed <= `MAX_SPEED = 6.0` m/s at every 30th step (below
//!   the 9.6 m/s critical velocity at r 0.04 and 2 iterations; ratio 3 drives
//!   the heavy layer at 30 m/s^2); max speed over steps 3,000..3,600 <= max
//!   speed over steps 0..1,200 (non-growing).
//! - S5 determinism: two fresh `build_parts(WeightRatio::Two, &TUNING)` runs
//!   of 600 steps give bit-identical positions and velocities (`to_bits`).
//! - Rain veto (stricter, Research Pitfall 1): a configuration whose rain
//!   share at step 240, below the settled depth line, exceeds 0.5 is
//!   rejected as rain even if S1 passes.
//! - S6 visible start (SCENE-02): at step 60 (1.0 s), ratio 2, g 10, the
//!   deepest interior bin's 4th-deepest heavy particle lies at least
//!   `START_DEPTH_GAIN = 0.06` m (one stride at r 0.04) deeper below the
//!   settled interface than the same measure at step 0 below the step-0
//!   interface. Each depth is read against the interface of its own frame,
//!   so a uniform slump never counts as ripple growth (D-06a).
//!
//! The chosen radius is 0.035, where the critical velocity is 8.4 m/s, so the
//! 6.0 m/s speed bound stays below it.
//!
//! D-07 interpretation: a finger bin is past the 20% depth line when at
//! least 4 heavy particles in it lie below the line; the bin touching each
//! wall is skipped; a finger is a maximal run of adjacent past bins.
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor`.

use std::sync::OnceLock;

use liquidfun::math::Vec2;
use liquidfun::{NoDecisionHook, StepLimits};

use super::geometry::TANK_INNER_HEIGHT;
use super::test_support::{
    Layers, MAX_SPEED, depth_line, down_frame, finger_count, fluid_top, interface_height,
    is_overturned, max_speed, out_of_tank_count, rain_share, ripple_depth, split_layers,
    step_configuration,
};
use super::{RayleighTaylorParts, TUNING, Tuning, WeightRatio, build_parts};
use crate::scene::SceneHooks;

/// Finger and overturn samples are taken every this many steps (0.1 s).
const SAMPLE_EVERY: u32 = 6;
/// Speed and containment samples are taken every this many steps (S4).
const STABILITY_SAMPLE_EVERY: u32 = 30;
/// Largest settle drop of the fluid top, as a share of the tank height (S0).
const MAX_SETTLE_DROP_FRACTION: f32 = 0.1;
/// Latest first sample with at least 3 fingers at ratio 2 (S1, 4.0 s).
const FINGER_DEADLINE_STEP: u32 = 240;
/// Latest first overturned sample at ratio 2 (S2, 10.0 s).
const OVERTURN_DEADLINE_STEP: u32 = 600;
/// Latest first overturned sample at ratio 1.2 (S3, 20 s).
const SLOW_OVERTURN_DEADLINE_STEP: u32 = 1_200;
/// Fingers counted for S1; the settle frame must show fewer (D-06a).
const MIN_FINGERS: usize = 3;
/// Step of the rain veto sample.
const RAIN_SAMPLE_STEP: u32 = 240;
/// Largest rain share a passing configuration may show at step 240.
const MAX_RAIN_SHARE: f32 = 0.5;
/// Step of the S6 visible-start sample (1.0 s).
const START_SAMPLE_STEP: u32 = 60;
/// Least ripple-depth gain by step 60 (S6): one stride at r 0.04.
const START_DEPTH_GAIN: f32 = 0.06;
/// A 60 s run at 60 Hz (S4).
const LONG_RUN_STEPS: u32 = 3_600;
/// Last step of the early speed window (S4).
const EARLY_WINDOW_END: u32 = 1_200;
/// First step of the late speed window (S4).
const LATE_WINDOW_START: u32 = 3_000;
/// Steps of each determinism run (S5).
const DETERMINISM_STEPS: u32 = 600;

/// Measurements from one run from load at gravity 10.
#[derive(Debug, Clone, Copy)]
struct RunReport {
    fluid_top_at_0: f32,
    fluid_top_at_settle: f32,
    settled_interface: f32,
    fingers_at_settle: usize,
    maybe_three_fingers_step: Option<u32>,
    maybe_overturned_step: Option<u32>,
    maybe_rain_share_at_240: Option<f32>,
    ripple_depth_at_0: f32,
    maybe_ripple_depth_at_60: Option<f32>,
    max_speed_early: f32,
    maybe_max_speed_late: Option<f32>,
    max_speed_overall: f32,
    max_out_of_tank: usize,
}

/// One session-order fixed step: `on_advance`, then `world.step`.
fn advance(parts: &mut RayleighTaylorParts) {
    parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system)
        .expect("the extra weight applies");
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("the tank world steps");
}

fn layers(parts: &RayleighTaylorParts) -> Layers {
    split_layers(&parts.world, parts.particle_system, parts.hooks.heavy_group)
}

/// Builds `ratio` with `tuning` and steps through the settle window,
/// returning the parts and a report holding the step-0 and settle values.
fn settle(ratio: WeightRatio, tuning: &Tuning) -> (RayleighTaylorParts, RunReport) {
    let mut parts = build_parts(ratio, tuning).expect("rayleigh-taylor parts");
    let frame = down_frame();
    let start = layers(&parts);
    let start_interface = interface_height(&start, frame);
    for _step in 0..tuning.settle_steps {
        advance(&mut parts);
    }
    let settled = layers(&parts);
    let settled_interface = interface_height(&settled, frame);
    let report = RunReport {
        fluid_top_at_0: fluid_top(&start, frame),
        fluid_top_at_settle: fluid_top(&settled, frame),
        settled_interface,
        fingers_at_settle: finger_count(&settled, frame, depth_line(settled_interface)),
        maybe_three_fingers_step: None,
        maybe_overturned_step: None,
        maybe_rain_share_at_240: None,
        ripple_depth_at_0: ripple_depth(&start, frame, start_interface),
        maybe_ripple_depth_at_60: None,
        max_speed_early: 0.0,
        maybe_max_speed_late: None,
        max_speed_overall: 0.0,
        max_out_of_tank: 0,
    };
    (parts, report)
}

fn run(ratio: WeightRatio, tuning: &Tuning, steps: u32) -> RunReport {
    let (mut parts, mut report) = settle(ratio, tuning);
    let frame = down_frame();
    let line = depth_line(report.settled_interface);
    for step in u32::from(tuning.settle_steps) + 1..=steps {
        advance(&mut parts);
        if step % SAMPLE_EVERY != 0 {
            continue;
        }
        let sample = layers(&parts);
        if report.maybe_three_fingers_step.is_none()
            && finger_count(&sample, frame, line) >= MIN_FINGERS
        {
            report.maybe_three_fingers_step = Some(step);
        }
        if report.maybe_overturned_step.is_none() && is_overturned(&sample, frame) {
            report.maybe_overturned_step = Some(step);
        }
        if step == START_SAMPLE_STEP {
            report.maybe_ripple_depth_at_60 =
                Some(ripple_depth(&sample, frame, report.settled_interface));
        }
        if step == RAIN_SAMPLE_STEP {
            report.maybe_rain_share_at_240 = Some(rain_share(&sample, frame, tuning.radius, line));
        }
        if step % STABILITY_SAMPLE_EVERY == 0 {
            record_stability(&mut report, &parts, &sample, step, tuning.radius);
        }
    }
    report
}

fn record_stability(
    report: &mut RunReport,
    parts: &RayleighTaylorParts,
    sample: &Layers,
    step: u32,
    radius: f32,
) {
    let speed = max_speed(&sample.velocities);
    report.max_speed_overall = report.max_speed_overall.max(speed);
    if step <= EARLY_WINDOW_END {
        report.max_speed_early = report.max_speed_early.max(speed);
    }
    if step >= LATE_WINDOW_START {
        report.maybe_max_speed_late = Some(
            report
                .maybe_max_speed_late
                .map_or(speed, |late| late.max(speed)),
        );
    }
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    report.max_out_of_tank = report
        .max_out_of_tank
        .max(out_of_tank_count(view.positions(), radius));
}

/// The 60 s default-ratio run, shared by S0, S1, S2, S4, S6 and the rain veto.
fn default_run() -> RunReport {
    static REPORT: OnceLock<RunReport> = OnceLock::new();
    *REPORT.get_or_init(|| run(WeightRatio::Two, &TUNING, LONG_RUN_STEPS))
}

fn assert_bounded(report: &RunReport) {
    assert_eq!(report.max_out_of_tank, 0, "{report:?}");
    assert!(report.max_speed_overall <= MAX_SPEED, "{report:?}");
    let late = report.maybe_max_speed_late.expect("late window sampled");
    assert!(late <= report.max_speed_early, "{report:?}");
}

fn final_bits(parts: &RayleighTaylorParts) -> Vec<(u32, u32)> {
    let view = parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid");
    let bits = |vector: &Vec2| (vector.x.to_bits(), vector.y.to_bits());
    view.positions()
        .iter()
        .chain(view.velocities())
        .map(bits)
        .collect()
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn settle_keeps_the_fluid_top_within_ten_percent() {
    // Act
    let report = default_run();

    // Assert
    let drop = report.fluid_top_at_0 - report.fluid_top_at_settle;
    assert!(
        drop <= MAX_SETTLE_DROP_FRACTION * TANK_INNER_HEIGHT,
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn settle_frame_shows_fewer_than_three_fingers() {
    // Act
    let report = default_run();

    // Assert
    assert!(report.fingers_at_settle < MIN_FINGERS, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn default_ratio_grows_three_fingers_by_step_240() {
    // Act
    let report = default_run();

    // Assert
    let step = report.maybe_three_fingers_step.expect("three fingers form");
    assert!(step <= FINGER_DEADLINE_STEP, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn default_ratio_overturns_by_step_600() {
    // Act
    let report = default_run();

    // Assert
    let step = report.maybe_overturned_step.expect("the layers overturn");
    assert!(step <= OVERTURN_DEADLINE_STEP, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn ratio_one_point_two_overturns_by_step_1200() {
    // Act
    let report = run(
        WeightRatio::OnePointTwo,
        &TUNING,
        SLOW_OVERTURN_DEADLINE_STEP,
    );

    // Assert
    let step = report.maybe_overturned_step.expect("the layers overturn");
    assert!(step <= SLOW_OVERTURN_DEADLINE_STEP, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn default_ratio_runs_sixty_seconds_bounded() {
    // Act
    let report = default_run();

    // Assert
    assert_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn ratio_three_runs_sixty_seconds_bounded() {
    // Act
    let report = run(WeightRatio::Three, &TUNING, LONG_RUN_STEPS);

    // Assert
    assert_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn two_default_runs_are_bit_identical() {
    // Arrange
    let mut first = build_parts(WeightRatio::Two, &TUNING).expect("first parts");
    let mut second = build_parts(WeightRatio::Two, &TUNING).expect("second parts");

    // Act
    for _step in 0..DETERMINISM_STEPS {
        advance(&mut first);
        advance(&mut second);
    }

    // Assert
    assert_eq!(final_bits(&first), final_bits(&second));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn default_ratio_fingers_are_not_rain() {
    // Act
    let report = default_run();

    // Assert
    let share = report.maybe_rain_share_at_240.expect("step 240 sampled");
    assert!(share <= MAX_RAIN_SHARE, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor spike (D-06)")]
fn default_ratio_ripple_deepens_within_one_second() {
    // Act
    let report = default_run();

    // Assert
    let depth = report.maybe_ripple_depth_at_60.expect("step 60 sampled");
    assert!(
        depth - report.ripple_depth_at_0 >= START_DEPTH_GAIN,
        "{report:?}"
    );
}

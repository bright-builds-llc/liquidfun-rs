//! Release-only Rayleigh–Taylor outcome tests (D-18, RT-01, RT-02), driven
//! through the real session path where the outcome is a session property:
//! fingers by 4 s and overturn by 10 s at the default ratio, bit-identical
//! replays at every ratio, the ratio preset taking effect, the heavy layer
//! moving toward a sideways gravity, a 60 s ratio-3 run and gravity 2 and 80
//! smokes. Fingers are measured from the settled interface (D-06a) and are
//! not asserted at gravity 80, where the pressure cap squashes the column
//! (Phase 46 D-07a).
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor`.

use std::sync::OnceLock;

use liquidfun::math::Vec2;

use super::test_support::{
    StabilityReport, advance, assert_bounded, centroid, depth_line, down_frame, finger_count,
    fixed_step, heavy_group_of, interface_height, is_overturned, run_session_stability,
    session_bits, session_layers, split_layers,
};
use super::{RayleighTaylorParts, TUNING, WEIGHT_RATIO_TOKENS, WeightRatio, build_parts};
use crate::scene::SceneId;
use crate::session::SessionCore;

/// Finger and overturn samples are taken every this many steps (0.1 s).
const SAMPLE_EVERY: u32 = 6;
/// Latest first sample with at least 3 fingers at the default ratio (4 s).
const FINGER_DEADLINE_STEPS: u32 = 240;
/// Latest first overturned sample at the default ratio (10 s).
const OVERTURN_DEADLINE_STEPS: u32 = 600;
/// Fingers that must pass the depth line by the deadline.
const MIN_FINGERS: usize = 3;
/// Steps of each replay run (10 s).
const REPLAY_STEPS: u32 = 600;
/// Steps after which two ratios must already differ (2 s).
const RATIO_DIFFERENCE_STEPS: u32 = 120;
/// Steps the layers get to respond to a sideways gravity (2 s).
const TILT_WINDOW_STEPS: u32 = 120;
/// Least growth of the heavy-minus-light centroid along the new gravity.
const TILT_MIN_SEPARATION_GAIN: f32 = 0.2;
/// The 60 s run at 60 Hz.
const LONG_RUN_STEPS: u32 = 3_600;
/// The gravity stability smoke: 20 s.
const SMOKE_STEPS: u32 = 1_200;

/// First finger and overturn samples of a default session.
#[derive(Debug, Clone, Copy)]
struct OutcomeReport {
    maybe_three_fingers_step: Option<u32>,
    maybe_overturned_step: Option<u32>,
}

fn new_session() -> SessionCore {
    SessionCore::create(SceneId::RayleighTaylor).expect("rayleigh-taylor scene")
}

fn session_with(name: &str, value: &str) -> SessionCore {
    let mut session = new_session();
    let rebuilt = session.apply_control(name, value);
    assert_eq!(rebuilt, Ok(true), "{name}={value}");
    session
}

/// A default session run to the overturn deadline, sampled every 6 steps
/// from the first step with extra weight; shared by the finger and overturn
/// tests.
fn default_outcome() -> OutcomeReport {
    static REPORT: OnceLock<OutcomeReport> = OnceLock::new();
    *REPORT.get_or_init(|| {
        let mut session = new_session();
        let heavy = heavy_group_of(&session);
        let frame = down_frame();
        let settle = u32::from(TUNING.settle_steps);
        advance(&mut session, settle);
        let line = depth_line(interface_height(&session_layers(&session, heavy), frame));
        let mut report = OutcomeReport {
            maybe_three_fingers_step: None,
            maybe_overturned_step: None,
        };
        let mut step = settle;
        while step < OVERTURN_DEADLINE_STEPS {
            advance(&mut session, SAMPLE_EVERY);
            step += SAMPLE_EVERY;
            let sample = session_layers(&session, heavy);
            if report.maybe_three_fingers_step.is_none()
                && finger_count(&sample, frame, line) >= MIN_FINGERS
            {
                report.maybe_three_fingers_step = Some(step);
            }
            if report.maybe_overturned_step.is_none() && is_overturned(&sample, frame) {
                report.maybe_overturned_step = Some(step);
            }
        }
        report
    })
}

/// Heavy centroid minus light centroid, along `direction`.
fn separation(parts: &RayleighTaylorParts, direction: Vec2) -> f32 {
    let layers = split_layers(&parts.world, parts.particle_system, parts.hooks.heavy_group);
    (centroid(&layers.heavy) - centroid(&layers.light)).dot(direction)
}

fn assert_smoke_is_stable(report: &StabilityReport) {
    assert_eq!(report.live, report.built, "{report:?}");
    assert_eq!(report.stability.max_out_of_tank, 0, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn default_ratio_grows_three_fingers_by_four_seconds() {
    // Act
    let report = default_outcome();

    // Assert
    let step = report.maybe_three_fingers_step.expect("three fingers form");
    assert!(step <= FINGER_DEADLINE_STEPS, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn default_ratio_overturns_by_ten_seconds() {
    // Act
    let report = default_outcome();

    // Assert
    let step = report.maybe_overturned_step.expect("the layers overturn");
    assert!(step <= OVERTURN_DEADLINE_STEPS, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn each_ratio_replays_to_the_same_end_state() {
    for token in WEIGHT_RATIO_TOKENS {
        // Arrange
        let mut first = session_with("weight-ratio", token);
        let mut second = session_with("weight-ratio", token);

        // Act
        advance(&mut first, REPLAY_STEPS);
        advance(&mut second, REPLAY_STEPS);

        // Assert
        assert_eq!(
            session_bits(&first),
            session_bits(&second),
            "ratio {token} diverged"
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn ratio_three_differs_from_the_default_ratio() {
    // Arrange
    let mut default_ratio = new_session();
    let mut ratio_three = session_with("weight-ratio", "3");

    // Act
    advance(&mut default_ratio, RATIO_DIFFERENCE_STEPS);
    advance(&mut ratio_three, RATIO_DIFFERENCE_STEPS);

    // Assert
    assert_ne!(session_bits(&default_ratio), session_bits(&ratio_three));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn sideways_gravity_moves_the_heavy_layer_toward_it() {
    // Arrange
    let mut parts = build_parts(WeightRatio::Two, &TUNING).expect("rayleigh-taylor parts");
    for _step in 0..TUNING.settle_steps + TUNING.ramp_steps {
        fixed_step(&mut parts);
    }
    let sideways = Vec2::new(-10.0, 0.0);
    let toward_gravity = Vec2::new(-1.0, 0.0);
    let before = separation(&parts, toward_gravity);
    parts.world.set_gravity(sideways).expect("finite gravity");

    // Act
    for _step in 0..TILT_WINDOW_STEPS {
        fixed_step(&mut parts);
    }

    // Assert
    let after = separation(&parts, toward_gravity);
    assert!(
        after - before >= TILT_MIN_SEPARATION_GAIN,
        "{before} -> {after}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn ratio_three_runs_sixty_seconds_without_eviction() {
    // Arrange
    let mut session = session_with("weight-ratio", "3");

    // Act
    let report = run_session_stability(&mut session, LONG_RUN_STEPS);

    // Assert
    assert_eq!(report.live, report.built, "{report:?}");
    assert_bounded(&report.stability, &report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn gravity_two_smoke_stays_stable() {
    // Arrange
    let mut session = session_with("gravity", "2");

    // Act
    let report = run_session_stability(&mut session, SMOKE_STEPS);

    // Assert
    assert_smoke_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn gravity_eighty_smoke_stays_stable() {
    // Arrange
    let mut session = new_session();
    assert_eq!(session.apply_control("gravity", "80"), Ok(true));

    // Act
    let report = run_session_stability(&mut session, SMOKE_STEPS);

    // Assert
    assert_smoke_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only rayleigh-taylor flow (D-18)")]
fn gravity_eighty_ratio_three_smoke_stays_stable() {
    // Arrange
    let mut session = new_session();
    assert_eq!(session.apply_control("gravity", "80"), Ok(true));
    assert_eq!(session.apply_control("weight-ratio", "3"), Ok(true));

    // Act
    let report = run_session_stability(&mut session, SMOKE_STEPS);

    // Assert
    assert_smoke_is_stable(&report);
}

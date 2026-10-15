//! Release-only D-05 stability and appearance spike for the polynomial pull
//! (candidate (a)), kept as regression guards. The pass rules were written
//! before measuring (48-01-PLAN):
//!
//! - P1: from load at the default strength and gravity 10, at least 4 peaks
//!   by step 90, and 5 to 9 peaks at step 180 (D-04).
//! - P2: after 120 steps on, switching off flattens the surface by step 90.
//! - P2b: once flat, switching back on gives at least 4 peaks by step 90.
//! - P3: 60 s holds at full strength in five magnet positions: no failed
//!   step, nothing out of the dish or escaped, bounded and non-growing max
//!   speed, no kinetic-energy growth and at most 3 isolated particles.
//! - P4: every measured column top stays 0.3 m below the wall tops, including
//!   two positions where a raised magnet gathers the tallest columns (48-02).
//! - P5: a session run evicts nothing.
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`.

use liquidfun::math::Vec2;
use liquidfun::{NoDecisionHook, StepLimits};

use super::geometry::WALL_TOP_Y;
use super::test_support::{
    ABOVE_THE_PUDDLE, CROWN_STEP, HOLD_POSITIONS, HoldRecorder, HoldReport, INSIDE_THE_PUDDLE,
    INSIDE_THE_PUDDLE_AT_THE_WALL, MIN_PEAKS, OFF_CENTER, TARGET_PEAKS, UNDER_THE_DISH,
    assert_hold_is_bounded, step_configuration, surface_profile,
};
use super::{DEFAULT_STRENGTH_PERCENT, FerrofluidParts, MAX_STRENGTH_PERCENT, build_parts};
use crate::scene::{SceneHooks, SceneId};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore};

/// Appearance samples are taken every this many steps (0.1 s).
const APPEARANCE_SAMPLE_EVERY: u32 = 6;
/// Latest step at which the first 4 peaks or the flat surface may appear (1.5 s).
const APPEARANCE_STEPS: u32 = 90;
/// Steps the magnet stays on before the collapse run switches it off.
const ON_STEPS_BEFORE_OFF: u32 = 120;
/// Longest wait, after off or back on, that a report keeps sampling.
const PHASE_LIMIT_STEPS: u32 = 240;
/// A 60 s hold at 60 Hz.
const HOLD_STEPS: u32 = 3_600;
/// Clearance every column top keeps below the wall tops (Research Pitfall 2).
const WALL_MARGIN: f32 = 0.3;
/// Steps of the session eviction run (20 s).
const SESSION_STEPS: u32 = 1_200;

/// Measurements from the load run and the off/on run.
#[derive(Debug, Clone, Copy)]
struct AppearanceReport {
    maybe_first_four_peaks_step: Option<u32>,
    peaks_at_crown_step: usize,
    maybe_flat_step_after_off: Option<u32>,
    maybe_four_peaks_step_after_on: Option<u32>,
}

/// One session-order fixed step: `on_advance`, then `world.step`.
fn advance(parts: &mut FerrofluidParts) {
    parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system)
        .expect("the pull applies");
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("the ferrofluid world steps");
}

fn positions(parts: &FerrofluidParts) -> Vec<Vec2> {
    parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid")
        .positions()
        .to_vec()
}

fn velocities(parts: &FerrofluidParts) -> Vec<Vec2> {
    parts
        .world
        .particle_system_view(parts.particle_system)
        .expect("liquid")
        .velocities()
        .to_vec()
}

fn peak_count(parts: &FerrofluidParts) -> usize {
    surface_profile(&positions(parts)).peak_count()
}

fn is_flat(parts: &FerrofluidParts) -> bool {
    surface_profile(&positions(parts)).is_flat()
}

/// Steps until `done` holds at a sample, up to `PHASE_LIMIT_STEPS`; returns that step.
fn first_sample_step(
    parts: &mut FerrofluidParts,
    done: impl Fn(&FerrofluidParts) -> bool,
) -> Option<u32> {
    for step in 1..=PHASE_LIMIT_STEPS {
        advance(parts);
        if step % APPEARANCE_SAMPLE_EVERY == 0 && done(parts) {
            return Some(step);
        }
    }
    None
}

/// Runs a hold with the magnet fixed at `magnet` from load.
fn run_hold(magnet: Vec2, strength_percent: u16, steps: u32) -> HoldReport {
    let mut parts = build_parts().expect("ferrofluid parts");
    parts.hooks.magnet_position = magnet;
    parts.hooks.strength_percent = strength_percent;
    let mut recorder = HoldRecorder::new();
    for step in 1..=steps {
        advance(&mut parts);
        recorder.record(step, &positions(&parts), &velocities(&parts));
    }
    recorder.finish()
}

/// A full-strength 60 s hold (P3).
fn run_full_strength_hold(magnet: Vec2) -> HoldReport {
    run_hold(magnet, MAX_STRENGTH_PERCENT, HOLD_STEPS)
}

/// The load run (P1) and a separate off/on run (P2, P2b).
fn run_appearance(strength_percent: u16) -> AppearanceReport {
    let mut load = build_parts().expect("ferrofluid parts");
    load.hooks.strength_percent = strength_percent;
    let mut maybe_first_four_peaks_step = None;
    for step in 1..=CROWN_STEP {
        advance(&mut load);
        let sampled = step % APPEARANCE_SAMPLE_EVERY == 0;
        if sampled && maybe_first_four_peaks_step.is_none() && peak_count(&load) >= MIN_PEAKS {
            maybe_first_four_peaks_step = Some(step);
        }
    }
    let peaks_at_crown_step = peak_count(&load);

    let mut toggled = build_parts().expect("ferrofluid parts");
    toggled.hooks.strength_percent = strength_percent;
    for _ in 0..ON_STEPS_BEFORE_OFF {
        advance(&mut toggled);
    }
    toggled.hooks.magnet_on = false;
    let maybe_flat_step_after_off = first_sample_step(&mut toggled, is_flat);
    toggled.hooks.magnet_on = true;
    let maybe_four_peaks_step_after_on = maybe_flat_step_after_off
        .and_then(|_| first_sample_step(&mut toggled, |parts| peak_count(parts) >= MIN_PEAKS));

    AppearanceReport {
        maybe_first_four_peaks_step,
        peaks_at_crown_step,
        maybe_flat_step_after_off,
        maybe_four_peaks_step_after_on,
    }
}

fn within_appearance_window(maybe_step: Option<u32>) -> bool {
    maybe_step.is_some_and(|step| step <= APPEARANCE_STEPS)
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn default_strength_forms_four_peaks_within_ninety_steps() {
    // Act
    let report = run_appearance(DEFAULT_STRENGTH_PERCENT);

    // Assert
    assert!(
        within_appearance_window(report.maybe_first_four_peaks_step),
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn default_strength_shows_five_to_nine_peaks_at_step_180() {
    // Act
    let report = run_appearance(DEFAULT_STRENGTH_PERCENT);

    // Assert
    assert!(
        TARGET_PEAKS.contains(&report.peaks_at_crown_step),
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn switching_off_flattens_within_ninety_steps() {
    // Act
    let report = run_appearance(DEFAULT_STRENGTH_PERCENT);

    // Assert
    assert!(
        within_appearance_window(report.maybe_flat_step_after_off),
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn switching_back_on_reforms_four_peaks_within_ninety_steps() {
    // Act
    let report = run_appearance(DEFAULT_STRENGTH_PERCENT);

    // Assert
    assert!(
        within_appearance_window(report.maybe_four_peaks_step_after_on),
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn hold_under_the_dish_stays_bounded() {
    // Act
    let report = run_full_strength_hold(UNDER_THE_DISH);

    // Assert
    assert_hold_is_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn hold_above_the_puddle_stays_bounded() {
    // Act
    let report = run_full_strength_hold(ABOVE_THE_PUDDLE);

    // Assert
    assert_hold_is_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn hold_off_center_stays_bounded() {
    // Act
    let report = run_full_strength_hold(OFF_CENTER);

    // Assert
    assert_hold_is_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn hold_inside_the_puddle_stays_bounded() {
    // Act
    let report = run_full_strength_hold(INSIDE_THE_PUDDLE);

    // Assert
    assert_hold_is_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn hold_inside_the_puddle_at_the_wall_stays_bounded() {
    // Act
    let report = run_full_strength_hold(INSIDE_THE_PUDDLE_AT_THE_WALL);

    // Assert
    assert_hold_is_bounded(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn column_tops_stay_three_tenths_below_the_wall_tops() {
    // Act
    let tallest = HOLD_POSITIONS
        .map(|magnet| run_full_strength_hold(magnet).max_column_top)
        .into_iter()
        .fold(f32::MIN, f32::max);

    // Assert
    assert!(tallest + WALL_MARGIN <= WALL_TOP_Y, "{tallest}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid spike (D-05)")]
fn session_run_evicts_nothing() {
    // Arrange
    let mut session = SessionCore::create(SceneId::FakeFerrofluid).expect("session");

    // Act
    for _ in 0..SESSION_STEPS / MAX_ADVANCE_STEPS {
        session
            .advance(MAX_ADVANCE_STEPS)
            .expect("session advances");
    }

    // Assert
    assert_eq!(
        session.live_particle_count().expect("live count"),
        session.particle_count()
    );
}

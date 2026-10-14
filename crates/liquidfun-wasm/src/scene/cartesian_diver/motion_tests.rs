//! Release-only Cartesian Diver timing: sink and rise at gravity 2, 10 and 80 (D-07, D-07a).
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib cartesian_diver`.

use super::geometry::{
    BOTTOM_THIRD_Y, INTERIOR_HALF_WIDTH, INTERIOR_HEIGHT, PARTICLE_RADIUS, TOP_THIRD_Y,
    column_top_third_y, water_surface_y,
};
use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

/// Steps the water settles after construction or a gravity rebuild before a squeeze.
const SETTLE_STEPS: u32 = 60;
/// Extra squeezed steps after the diver reaches the bottom third, before release.
const HOLD_STEPS: u32 = 30;
/// A long press: five seconds squeezed on the floor before release.
const LONG_HOLD_STEPS: u32 = 300;
/// Resting steps after the settle before the floating diver is checked.
const REST_STEPS: u32 = 240;
/// D-07 windows in 60 Hz steps: 3 s sink and 4 s rise at gravity 10.
const SINK_LIMIT_TEN: u32 = 180;
const RISE_LIMIT_TEN: u32 = 240;
/// Gravity 2 scales the windows by sqrt(5): ceil(3 sqrt 5) s and ceil(4 sqrt 5) s.
const SINK_LIMIT_TWO: u32 = 420;
const RISE_LIMIT_TWO: u32 = 540;
/// Gravity 80 never gets a window tighter than gravity 10 (the D-07 sane cap).
const SINK_LIMIT_EIGHTY: u32 = 180;
const RISE_LIMIT_EIGHTY: u32 = 240;
/// Steps squeezed then released in the containment check.
const CONTAINMENT_PHASE_STEPS: u32 = 300;
/// README clip: 20 Hz samples, 3 steps each. The squeeze cue is sample 20 and
/// the release cue sample 110 in `web/scripts/readme-svg/plans.ts` (Plan 02).
const README_SQUEEZE_STEP: u32 = 60;
const README_RELEASE_STEP: u32 = 330;
/// The clip ends at step 600; the rise must finish a sample before that.
const README_RISE_DEADLINE_STEP: u32 = 597;

fn session_at_gravity(magnitude: &str) -> SessionCore {
    let mut session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene");
    if magnitude != "10" {
        let rebuilt = session
            .apply_control("gravity", magnitude)
            .expect("the gravity slider accepts this magnitude");
        assert!(rebuilt, "gravity rebuilds the session");
    }
    advance(&mut session, SETTLE_STEPS);
    session
}

fn advance(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(4);
        session.advance(chunk).expect("the diver scene should step");
        remaining -= chunk;
    }
}

fn diver_y(session: &SessionCore) -> f32 {
    let frame = ProofFrame::from(session.capture_frame().expect("frame"));
    frame.rigid_circles()[1]
}

fn particle_heights(session: &SessionCore) -> Vec<f32> {
    session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("system")
            .positions()
            .iter()
            .map(|position| position.y)
            .collect()
    })
}

/// First step count (1-based) at which `predicate` holds for the diver center y.
fn steps_until(
    session: &mut SessionCore,
    limit: u32,
    predicate: impl Fn(f32) -> bool,
) -> Option<u32> {
    for step in 1..=limit {
        session.advance(1).expect("the diver scene should step");
        if predicate(diver_y(session)) {
            return Some(step);
        }
    }
    None
}

fn toggle_squeeze(session: &mut SessionCore) {
    session
        .apply_action("squeeze")
        .expect("squeeze is the scene action");
}

/// Squeezes the settled diver and asserts it reaches the bottom third within `limit`.
fn squeeze_to_bottom_third(session: &mut SessionCore, gravity: &str, limit: u32) {
    toggle_squeeze(session);
    let maybe_sink = steps_until(session, limit, |y| y < BOTTOM_THIRD_Y);
    let y = diver_y(session);
    assert!(
        maybe_sink.is_some(),
        "squeezed diver at y {y} did not reach y < {BOTTOM_THIRD_Y} within {limit} steps at gravity {gravity}"
    );
}

/// Sinks, holds `hold_steps`, releases and asserts the rise to `target` within `limit`.
fn assert_rise_after_hold(
    gravity: &str,
    sink_limit: u32,
    hold_steps: u32,
    rise_limit: u32,
    target: impl Fn(&SessionCore) -> f32,
) {
    let mut session = session_at_gravity(gravity);
    squeeze_to_bottom_third(&mut session, gravity, sink_limit);
    advance(&mut session, hold_steps);
    let rise_target = target(&session);
    let start_y = diver_y(&session);
    assert!(
        start_y < rise_target,
        "diver at y {start_y} already above the rise target {rise_target} at gravity {gravity}"
    );

    toggle_squeeze(&mut session);
    let maybe_rise = steps_until(&mut session, rise_limit, |y| y >= rise_target);

    let y = diver_y(&session);
    assert!(
        maybe_rise.is_some(),
        "released diver at y {y} did not reach y >= {rise_target} within {rise_limit} steps at gravity {gravity}"
    );
}

/// The bottle's top third: the D-07 rise target at gravity 2 and 10.
fn bottle_top_third(_session: &SessionCore) -> f32 {
    TOP_THIRD_Y
}

/// The top third of the water column measured at release: the D-07a gravity-80 target.
fn water_column_top_third(session: &SessionCore) -> f32 {
    let surface = water_surface_y(&particle_heights(session)).expect("the bottle holds water");
    column_top_third_y(surface)
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn diver_floats_in_the_top_third_at_rest() {
    // Arrange
    let mut session = session_at_gravity("10");

    // Act
    advance(&mut session, REST_STEPS);

    // Assert
    let y = diver_y(&session);
    assert!(y >= TOP_THIRD_Y, "resting diver y {y}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn squeezed_diver_sinks_at_gravity_ten() {
    // Arrange
    let mut session = session_at_gravity("10");

    // Act / Assert
    squeeze_to_bottom_third(&mut session, "10", SINK_LIMIT_TEN);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn released_diver_rises_at_gravity_ten() {
    // Arrange / Act / Assert
    assert_rise_after_hold(
        "10",
        SINK_LIMIT_TEN,
        HOLD_STEPS,
        RISE_LIMIT_TEN,
        bottle_top_third,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn squeezed_diver_sinks_at_gravity_two() {
    // Arrange
    let mut session = session_at_gravity("2");

    // Act / Assert
    squeeze_to_bottom_third(&mut session, "2", SINK_LIMIT_TWO);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn released_diver_rises_at_gravity_two() {
    // Arrange / Act / Assert
    assert_rise_after_hold(
        "2",
        SINK_LIMIT_TWO,
        HOLD_STEPS,
        RISE_LIMIT_TWO,
        bottle_top_third,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn squeezed_diver_sinks_at_gravity_eighty() {
    // Arrange
    let mut session = session_at_gravity("80");

    // Act / Assert
    squeeze_to_bottom_third(&mut session, "80", SINK_LIMIT_EIGHTY);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn released_diver_rises_in_the_water_column_at_gravity_eighty() {
    // Arrange / Act / Assert
    assert_rise_after_hold(
        "80",
        SINK_LIMIT_EIGHTY,
        HOLD_STEPS,
        RISE_LIMIT_EIGHTY,
        water_column_top_third,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn released_diver_leaves_the_floor_after_a_long_hold_at_gravity_ten() {
    // Arrange / Act / Assert
    assert_rise_after_hold(
        "10",
        SINK_LIMIT_TEN,
        LONG_HOLD_STEPS,
        RISE_LIMIT_TEN,
        bottle_top_third,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn pointer_hold_sinks_and_cancel_releases() {
    // Arrange
    let mut session = session_at_gravity("10");

    // Act
    session
        .apply_pointer("down", 0.0, 2.8)
        .expect("a press inside the bottle squeezes");
    let maybe_sink = steps_until(&mut session, SINK_LIMIT_TEN, |y| y < BOTTOM_THIRD_Y);
    session
        .apply_pointer("cancel", 5.0, 9.0)
        .expect("a cancel anywhere releases");
    let maybe_rise = steps_until(&mut session, RISE_LIMIT_TEN, |y| y >= TOP_THIRD_Y);

    // Assert
    assert!(maybe_sink.is_some(), "pointer hold did not sink the diver");
    assert!(
        maybe_rise.is_some(),
        "pointer cancel did not raise the diver"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn readme_schedule_shows_a_full_sink_and_rise() {
    // Arrange
    let mut session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene");
    advance(&mut session, README_SQUEEZE_STEP);

    // Act
    toggle_squeeze(&mut session);
    let sink_window = README_RELEASE_STEP - README_SQUEEZE_STEP;
    let maybe_sink = steps_until(&mut session, sink_window, |y| y < BOTTOM_THIRD_Y);
    let reached = README_SQUEEZE_STEP + maybe_sink.unwrap_or(sink_window);
    advance(&mut session, README_RELEASE_STEP - reached);
    toggle_squeeze(&mut session);
    let rise_window = README_RISE_DEADLINE_STEP - README_RELEASE_STEP;
    let maybe_rise = steps_until(&mut session, rise_window, |y| y >= TOP_THIRD_Y);

    // Assert
    assert!(
        maybe_sink.is_some(),
        "no sink before step {README_RELEASE_STEP}"
    );
    assert!(
        maybe_rise.is_some(),
        "no rise by step {README_RISE_DEADLINE_STEP}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]
fn water_stays_in_the_bottle_at_gravity_eighty() {
    // Arrange
    let mut session = session_at_gravity("80");
    let built = session.particle_count();
    let x_limit = INTERIOR_HALF_WIDTH + PARTICLE_RADIUS;
    let y_range = -PARTICLE_RADIUS..=INTERIOR_HEIGHT + PARTICLE_RADIUS;

    // Act
    toggle_squeeze(&mut session);
    advance(&mut session, CONTAINMENT_PHASE_STEPS);
    toggle_squeeze(&mut session);
    advance(&mut session, CONTAINMENT_PHASE_STEPS);
    let points = session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("system")
            .positions()
            .to_vec()
    });

    // Assert
    assert_eq!(session.live_particle_count().expect("count"), built);
    for point in points {
        assert!(point.x.is_finite() && point.y.is_finite(), "{point:?}");
        assert!(
            point.x.abs() <= x_limit,
            "particle left the bottle at {point:?}"
        );
        assert!(
            y_range.contains(&point.y),
            "particle left the bottle at {point:?}"
        );
    }
}

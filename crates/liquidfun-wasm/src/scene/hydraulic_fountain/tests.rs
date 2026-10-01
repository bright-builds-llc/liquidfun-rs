use liquidfun::{BodyType, WorldObservationLimits};

use super::{
    ASCEND_SPEED, DESCEND_SPEED, DESCEND_WINDOW, FILL_BOTTOM_Y, GAP_HALF_WIDTH, HOLD_HIGH,
    HOLD_LOW, PRESSED_CENTER_Y, RAISED_BOTTOM_Y, RAISED_CENTER_Y, SIM_DT, WATER_DEPTH, build,
    plate_velocity,
};
use crate::ProofFrame;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const WATER_TOP_Y: f32 = FILL_BOTTOM_Y + WATER_DEPTH;
const JET_CLEARANCE_Y: f32 = RAISED_BOTTOM_Y;

#[test]
fn fresh_build_places_the_pool_under_raised_plates() {
    // Arrange
    let session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let positions = particle_positions(&session);
    let plate_ys = plate_center_ys(&session);

    // Assert
    assert!(positions.len() > 100, "the pool should start with water");
    assert!(
        positions
            .iter()
            .all(|position| position.y <= WATER_TOP_Y + 0.02),
        "the pool starts below the raised plates"
    );
    assert!(
        positions
            .iter()
            .any(|position| position.x < -GAP_HALF_WIDTH),
        "water starts under the left plate"
    );
    assert!(
        positions.iter().any(|position| position.x > GAP_HALF_WIDTH),
        "water starts under the right plate"
    );
    assert_eq!(plate_ys.len(), 2, "two plates hover over the pool");
    assert!(
        plate_ys
            .iter()
            .all(|center_y| center_y.to_bits() == RAISED_CENTER_Y.to_bits()),
        "both plates start raised, centers were {plate_ys:?}"
    );
}

#[test]
fn plates_are_kinematic() {
    // Arrange
    let session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let kinematic_count = session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the plates")
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Kinematic)
            .count()
    });

    // Assert
    assert_eq!(kinematic_count, 2, "the two plates are kinematic");
}

#[test]
fn motor_schedule_holds_then_snaps_down() {
    // Arrange
    let raised = RAISED_CENTER_Y;
    let during_hold = SIM_DT;
    let during_descent = HOLD_HIGH + SIM_DT;

    // Act
    let hold_speed = plate_velocity(during_hold, raised);
    let descend_speed = plate_velocity(during_descent, raised);
    let seated_speed = plate_velocity(during_descent, PRESSED_CENTER_Y);

    // Assert
    assert_eq!(hold_speed.to_bits(), 0.0_f32.to_bits());
    assert_eq!(descend_speed.to_bits(), (-DESCEND_SPEED).to_bits());
    assert_eq!(seated_speed.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn motor_schedule_holds_the_squeeze_then_lifts() {
    // Arrange
    let pressed = PRESSED_CENTER_Y;
    let during_hold_low = HOLD_HIGH + DESCEND_WINDOW + SIM_DT;
    let during_ascent = HOLD_HIGH + DESCEND_WINDOW + HOLD_LOW + SIM_DT;

    // Act
    let hold_speed = plate_velocity(during_hold_low, pressed);
    let lift_speed = plate_velocity(during_ascent, pressed);

    // Assert
    assert_eq!(hold_speed.to_bits(), 0.0_f32.to_bits());
    assert_eq!(lift_speed.to_bits(), ASCEND_SPEED.to_bits());
}

#[test]
fn controls_and_pointer_stay_watch_first() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let period = session.apply_control("period", "2");
    let aim = session.apply_action("aim");
    let pointer = session.apply_pointer("up", 0.0, 0.5);

    // Assert
    assert_eq!(period, Err(SessionError::UnknownControl));
    assert_eq!(aim, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
}

#[test]
fn pointer_up_leaves_the_live_count_unchanged() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let before = session
        .live_particle_count()
        .expect("the starting water should be countable");

    // Act
    session
        .apply_pointer("up", 0.0, 0.5)
        .expect("pointer up is a watch-first no-op");
    let after = session
        .live_particle_count()
        .expect("pointer up should leave the water in place");

    // Assert
    assert_eq!(after, before);
}

#[test]
fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
    // Arrange
    let period = [("period".to_owned(), "2".to_owned())];
    let gravity = [("gravity".to_owned(), "10".to_owned())];

    // Act
    let rejected = build(&period);
    let built = build_scene(SceneId::HydraulicFountain, &gravity);

    // Assert
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    let built = built.expect("gravity is stripped before the scene builder");
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn the_press_drives_a_jet_through_the_gap() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let start_count = session
        .live_particle_count()
        .expect("the pool should start with water");

    // Act
    advance_steps(&mut session, 16);
    let held_ys = plate_center_ys(&session);
    let mut best_crest = f32::MIN;
    let mut best_jet_count = 0_usize;
    let mut remaining = 120_u32;
    while remaining > 0 {
        let batch = remaining.min(4);
        session
            .advance(batch)
            .expect("the squeeze should keep stepping");
        remaining -= batch;
        let positions = particle_positions(&session);
        let crest = gap_crest(&positions);
        let jet_count = positions
            .iter()
            .filter(|position| {
                position.y > JET_CLEARANCE_Y && position.x.abs() < GAP_HALF_WIDTH + 0.02
            })
            .count();
        best_crest = best_crest.max(crest);
        best_jet_count = best_jet_count.max(jet_count);
    }
    let pressed_ys = plate_center_ys(&session);
    let end_count = session
        .live_particle_count()
        .expect("the squeezed water should still be live");
    let jet_count = best_jet_count;
    let crest = best_crest;

    // Assert
    assert!(
        held_ys
            .iter()
            .all(|center_y| (center_y - RAISED_CENTER_Y).abs() < 0.02),
        "the plates wait in the air before the snap, centers were {held_ys:?}"
    );
    assert!(
        pressed_ys
            .iter()
            .all(|center_y| (center_y - PRESSED_CENTER_Y).abs() < 0.04),
        "both plates reach the pool, centers were {pressed_ys:?}"
    );
    assert_eq!(end_count, start_count, "the squeeze keeps the same water");
    assert!(
        jet_count >= 8,
        "the gap should throw a jet above the raised plate line; crest {crest}, count {jet_count}"
    );
}

#[test]
fn rebuild_restores_the_raised_plates_and_pool() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    advance_steps(&mut session, 80);
    drop(session);

    // Act
    let rebuilt = SessionCore::create(SceneId::HydraulicFountain)
        .expect("a new session should restore the authored layout");
    let positions = particle_positions(&rebuilt);
    let plate_ys = plate_center_ys(&rebuilt);

    // Assert
    assert!(positions.len() > 100, "rebuild places the pool again");
    assert!(
        positions
            .iter()
            .all(|position| position.y <= WATER_TOP_Y + 0.02),
        "rebuild puts the water back in the pool"
    );
    assert!(
        plate_ys
            .iter()
            .all(|center_y| center_y.to_bits() == RAISED_CENTER_Y.to_bits()),
        "rebuild raises both plates"
    );
}

#[test]
fn captured_frame_draws_the_moving_plates() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let initial_bottom = lowest_plate_edge_y(&session);

    // Act
    advance_steps(&mut session, 52);
    let later_bottom = lowest_plate_edge_y(&session);

    // Assert
    assert!(
        (initial_bottom - RAISED_BOTTOM_Y).abs() < 0.02,
        "the first frame draws the raised plate bottoms at {initial_bottom}"
    );
    assert!(
        later_bottom < initial_bottom - 0.2,
        "the drawn plates move down toward the pool"
    );
}

fn gap_crest(positions: &[liquidfun::math::Vec2]) -> f32 {
    positions
        .iter()
        .filter(|position| position.x.abs() < GAP_HALF_WIDTH + 0.02)
        .map(|position| position.y)
        .fold(f32::MIN, f32::max)
}

fn particle_positions(session: &SessionCore) -> Vec<liquidfun::math::Vec2> {
    session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("the water system should be live")
            .positions()
            .to_vec()
    })
}

fn plate_center_ys(session: &SessionCore) -> Vec<f32> {
    session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the plates")
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Kinematic)
            .map(|body| body.snapshot().transform().position().y)
            .collect()
    })
}

fn lowest_plate_edge_y(session: &SessionCore) -> f32 {
    let segments = capture(session).rigid_segments();
    assert_eq!(
        segments.len(),
        44,
        "three wall segments and eight plate edges"
    );
    segments
        .chunks(4)
        .filter_map(|segment| {
            let start_x = segment[0];
            let start_y = segment[1];
            let end_x = segment[2];
            let end_y = segment[3];
            let horizontal = (start_y - end_y).abs() < 0.001;
            let on_one_plate = start_x.signum() == end_x.signum()
                && start_x.abs() > GAP_HALF_WIDTH
                && end_x.abs() > GAP_HALF_WIDTH;
            if horizontal && on_one_plate {
                Some(start_y.min(end_y))
            } else {
                None
            }
        })
        .fold(f32::MAX, f32::min)
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let batch = remaining.min(4);
        session
            .advance(batch)
            .expect("hydraulic fountain should keep stepping");
        remaining -= batch;
    }
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("hydraulic fountain should capture a frame"),
    )
}

use liquidfun::collision::PolygonShape;
use liquidfun::math::Vec2;
use liquidfun::{BodyType, WorldObservationLimits};

use super::{
    ASCEND_SPEED, CHEEK_HALF_HEIGHT, CYCLE, DESCEND_DURATION, GAP_HALF_WIDTH, GAP_TENTHS_DEFAULT,
    HOLD_HIGH, HOLD_LOW, PEAK_DESCEND_SPEED, PLATE_OUTER_X, POOL_TOP_Y, PRESSED_CENTER_Y,
    RAISED_CENTER_Y, RAISED_CHEEK_BOTTOM_Y, SIM_DT, SLAB_HALF_HEIGHT, SLOPE_ANGLE, WALL_OUTER_X,
    build, gap_half_from_tenths, plate_center_x, plate_velocity, scheduled_center_y,
    slab_half_width, slope_polygon,
};
use crate::ProofFrame;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const PARTICLE_COUNT: usize = 3_200;
const HOLD_HIGH_STEPS: u32 = 180;
const DESCEND_STEPS_BEFORE_SAMPLE: u32 = 60;
const JET_SAMPLE_STEPS: u32 = 80;
const PLATE_SEGMENT_FLOATS: usize = 108;

#[test]
fn fresh_build_places_3200_particles_under_raised_plates() {
    // Arrange
    let session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");

    // Act
    let positions = particle_positions(&session);
    let plate_ys = plate_center_ys(&session);

    // Assert
    assert_eq!(positions.len(), PARTICLE_COUNT);
    assert!(
        positions
            .iter()
            .all(|position| position.y <= POOL_TOP_Y + 0.02),
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
fn descent_eases_in_and_peaks_under_the_tunnel_limit() {
    // Arrange
    let early = HOLD_HIGH + SIM_DT;
    let late = HOLD_HIGH + DESCEND_DURATION - SIM_DT;

    // Act
    let early_speed = plate_velocity(early, scheduled_center_y(early - SIM_DT));
    let late_speed = plate_velocity(late, scheduled_center_y(late - SIM_DT));

    // Assert
    assert!(early_speed < 0.0, "the drop starts downward");
    assert!(
        late_speed < early_speed,
        "the drop accelerates, early {early_speed}, late {late_speed}"
    );
    assert!(
        early_speed.abs() < late_speed.abs() * 0.25,
        "the first drop step is much slower than the end"
    );
    assert!(
        late_speed.abs() <= PEAK_DESCEND_SPEED + 0.02,
        "the press peaks near {PEAK_DESCEND_SPEED}, got {late_speed}"
    );
}

#[test]
fn cycle_pauses_then_rises_slowly() {
    // Arrange
    let bottom = HOLD_HIGH + DESCEND_DURATION + HOLD_LOW * 0.5;
    let ascent = HOLD_HIGH + DESCEND_DURATION + HOLD_LOW + SIM_DT;
    let top = SIM_DT;
    let repeated_top = CYCLE + SIM_DT;

    // Act
    let bottom_speed = plate_velocity(bottom, PRESSED_CENTER_Y);
    let rise_speed = plate_velocity(ascent, scheduled_center_y(ascent - SIM_DT));
    let top_speed = plate_velocity(top, RAISED_CENTER_Y);
    let repeated_speed = plate_velocity(repeated_top, RAISED_CENTER_Y);

    // Assert
    assert_eq!(bottom_speed.to_bits(), 0.0_f32.to_bits());
    assert!(
        (rise_speed - ASCEND_SPEED).abs() < 1.0e-4,
        "the rise should be the slow linear speed, got {rise_speed}"
    );
    assert!(
        rise_speed < PEAK_DESCEND_SPEED * 0.5,
        "the rise stays slower than the press"
    );
    assert_eq!(top_speed.to_bits(), 0.0_f32.to_bits());
    assert_eq!(repeated_speed.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn platform_lids_slope_down_toward_the_gap() {
    // Arrange
    let left = slope_polygon(1.0, GAP_HALF_WIDTH).expect("left lid should build");
    let right = slope_polygon(-1.0, GAP_HALF_WIDTH).expect("right lid should build");

    // Act
    let left_angle = downhill_angle(&left, 1.0);
    let right_angle = downhill_angle(&right, -1.0);

    // Assert
    assert!(
        (left_angle - SLOPE_ANGLE).abs() < 1.0e-4,
        "left lid angle {left_angle}"
    );
    assert!(
        (right_angle - SLOPE_ANGLE).abs() < 1.0e-4,
        "right lid angle {right_angle}"
    );
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
fn gap_slider_moves_the_inner_edges_and_keeps_the_wall_seal() {
    // Arrange
    let mut session = SessionCore::create(SceneId::HydraulicFountain)
        .expect("hydraulic fountain should construct");
    let authored = gap_half_from_tenths(GAP_TENTHS_DEFAULT);

    // Act
    let too_narrow = session.apply_control("gap", "0.4");
    let padded = session.apply_control("gap", "0.50");
    let wide = session.apply_control("gap", "3.0");
    let wide_centers = plate_center_xs(&session);
    let wide_ys = plate_center_ys(&session);
    let narrow = session.apply_control("gap", "0.5");
    let narrow_centers = plate_center_xs(&session);
    advance_steps(&mut session, 30);
    let held_centers = plate_center_xs(&session);
    let count = session
        .live_particle_count()
        .expect("the gap slider should leave the water in place");

    // Assert
    assert_eq!(authored.to_bits(), GAP_HALF_WIDTH.to_bits());
    assert_eq!(too_narrow, Err(SessionError::UnknownControl));
    assert_eq!(padded, Err(SessionError::UnknownControl));
    assert!(wide.is_ok(), "3.0 cm is the widest opening");
    assert!(narrow.is_ok(), "0.5 cm is the narrowest opening");
    assert_plate_gap(&wide_centers, gap_half_from_tenths(30));
    assert_plate_gap(&narrow_centers, gap_half_from_tenths(5));
    assert_plate_gap(&held_centers, gap_half_from_tenths(5));
    assert!(
        wide_ys
            .iter()
            .all(|center_y| center_y.to_bits() == RAISED_CENTER_Y.to_bits()),
        "changing the gap keeps the plates at their current height"
    );
    assert_eq!(count, PARTICLE_COUNT);
}

#[test]
fn the_press_stops_just_above_the_floor() {
    // Arrange
    let cheek_bottom = PRESSED_CENTER_Y - CHEEK_HALF_HEIGHT;

    // Act
    let clearance = cheek_bottom - 0.0;

    // Assert
    assert!(
        clearance > 0.0,
        "the cheeks should stay above the floor, bottom {cheek_bottom}"
    );
    assert!(
        clearance < 0.02,
        "the cheeks should end close to the floor, clearance {clearance}"
    );
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
    assert_eq!(after, PARTICLE_COUNT);
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
    advance_steps(&mut session, HOLD_HIGH_STEPS);
    let held_ys = plate_center_ys(&session);
    advance_steps(&mut session, DESCEND_STEPS_BEFORE_SAMPLE);
    let mut best_jet = 0_usize;
    let mut best_side = 0_usize;
    let mut best_crest = f32::MIN;
    let mut remaining = JET_SAMPLE_STEPS;
    while remaining > 0 {
        let batch = remaining.min(4);
        session
            .advance(batch)
            .expect("the squeeze should keep stepping");
        remaining -= batch;
        let positions = particle_positions(&session);
        let plate_top = plate_center_ys(&session)
            .into_iter()
            .fold(0.0_f32, f32::max)
            + SLAB_HALF_HEIGHT;
        let jet = positions
            .iter()
            .filter(|position| position.y > plate_top + 0.05 && position.x.abs() < GAP_HALF_WIDTH)
            .count();
        // A 3 cm hole is only a little wider than one particle, so water also climbs
        // the lids. This count is particles that get past the outer face of the walls.
        let side = positions
            .iter()
            .filter(|position| position.y > plate_top + 0.05 && position.x.abs() > PLATE_OUTER_X)
            .count();
        if jet >= best_jet {
            best_jet = jet;
            best_side = side;
        }
        best_crest = best_crest.max(gap_crest(&positions));
    }
    let pressed_ys = plate_center_ys(&session);
    let end_count = session
        .live_particle_count()
        .expect("the squeezed water should still be live");
    // Assert
    assert!(
        held_ys
            .iter()
            .all(|center_y| (center_y - RAISED_CENTER_Y).abs() < 0.02),
        "the plates wait in the air, centers were {held_ys:?}"
    );
    assert!(
        pressed_ys
            .iter()
            .all(|center_y| (center_y - PRESSED_CENTER_Y).abs() < 0.04),
        "both plates reach the pool, centers were {pressed_ys:?}"
    );
    assert_eq!(end_count, start_count, "the squeeze keeps the same water");
    assert!(
        best_jet >= 8,
        "the gap should throw a jet above the lids; crest {best_crest}, jet {best_jet}, side {best_side}, plates {pressed_ys:?}, count {end_count}"
    );
    assert!(
        best_side * 2 <= best_jet,
        "water past the walls should stay smaller than the middle jet; jet {best_jet}, side {best_side}"
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
    assert_eq!(positions.len(), PARTICLE_COUNT);
    assert!(
        positions
            .iter()
            .all(|position| position.y <= POOL_TOP_Y + 0.02),
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
    advance_steps(&mut session, HOLD_HIGH_STEPS + 70);
    let later_bottom = lowest_plate_edge_y(&session);

    // Assert
    assert!(
        (initial_bottom - RAISED_CHEEK_BOTTOM_Y).abs() < 0.02,
        "the first frame draws the raised cheek bottoms at {initial_bottom}"
    );
    assert!(
        later_bottom < initial_bottom - 0.2,
        "the drawn plates accelerate down toward the pool"
    );
}

fn downhill_angle(polygon: &PolygonShape, toward_gap: f32) -> f32 {
    let (first, second) = highest_vertices(polygon);
    let (outer, gap) = if first.x * toward_gap < second.x * toward_gap {
        (first, second)
    } else {
        (second, first)
    };
    let run = (gap.x - outer.x) * toward_gap;
    let drop = outer.y - gap.y;
    drop.atan2(run)
}

fn highest_vertices(polygon: &PolygonShape) -> (Vec2, Vec2) {
    let mut vertices = polygon.vertices().iter();
    let first = vertices.next().copied().expect("a lid has vertices");
    let second = vertices.next().copied().expect("a lid has a second vertex");
    let (mut highest, mut next_highest) = if second.y > first.y {
        (second, first)
    } else {
        (first, second)
    };
    for vertex in vertices {
        if vertex.y > highest.y {
            next_highest = highest;
            highest = *vertex;
        } else if vertex.y > next_highest.y {
            next_highest = *vertex;
        }
    }
    (highest, next_highest)
}

fn gap_crest(positions: &[Vec2]) -> f32 {
    positions
        .iter()
        .filter(|position| position.x.abs() < GAP_HALF_WIDTH)
        .map(|position| position.y)
        .fold(f32::MIN, f32::max)
}

fn particle_positions(session: &SessionCore) -> Vec<Vec2> {
    session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("the water system should be live")
            .positions()
            .to_vec()
    })
}

fn assert_plate_gap(centers: &[f32], gap_half: f32) {
    assert_eq!(centers.len(), 2, "both plates stay in the tank");
    let mut ordered = centers.to_vec();
    ordered.sort_by(f32::total_cmp);
    let left = ordered[0];
    let right = ordered[1];
    let center = plate_center_x(gap_half);
    let slab_half = slab_half_width(gap_half);
    assert!(
        (left + center).abs() < 1.0e-4,
        "left plate center {left}, expected {}",
        -center
    );
    assert!(
        (right - center).abs() < 1.0e-4,
        "right plate center {right}, expected {center}"
    );
    assert!(
        (PLATE_OUTER_X - WALL_OUTER_X).abs() < 1.0e-4,
        "each plate reaches the outer face of the side wall"
    );
    assert!(
        (left - slab_half + PLATE_OUTER_X).abs() < 1.0e-4,
        "the left plate covers the side wall"
    );
    assert!(
        (right + slab_half - PLATE_OUTER_X).abs() < 1.0e-4,
        "the right plate covers the side wall"
    );
    assert!((left + slab_half + gap_half).abs() < 1.0e-4);
    assert!((right - slab_half - gap_half).abs() < 1.0e-4);
}

fn plate_center_xs(session: &SessionCore) -> Vec<f32> {
    session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the plates")
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Kinematic)
            .map(|body| body.snapshot().transform().position().x)
            .collect()
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
        PLATE_SEGMENT_FLOATS,
        "three wall segments plus the slab, cheek, and lid of each plate"
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

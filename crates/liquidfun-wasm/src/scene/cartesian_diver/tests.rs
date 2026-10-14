//! Debug-safe Cartesian Diver checks: construction, input model, density ramp and drawing.

use liquidfun::math::Vec2;

use super::geometry::{
    self, BOTTOM_THIRD_Y, DENT_DEPTH, DIVER_RADIUS, DIVER_START, HIT_REGION_MAX, HIT_REGION_MIN,
    INTERIOR_HALF_WIDTH, INTERIOR_HEIGHT, OUTLINE_BOTTOM_Y, OUTLINE_GAP, OUTLINE_HALF_WIDTH,
    OUTLINE_SEGMENT_COUNT, OUTLINE_TOP_Y, PARTICLE_RADIUS, SIDE_SEGMENTS, TOP_THIRD_Y,
    WALL_THICKNESS,
};
use super::{
    DiverParts, RAMP_STEPS, RELEASED_DENSITY, SQUEEZE_ACTION, SQUEEZED_DENSITY, build_parts,
};
use crate::ProofFrame;
use crate::scene::{PointerKind, SceneHooks, SceneId};
use crate::session::{SessionCore, SessionError};

const PARTICLE_BUDGET: usize = 3_000;
const INSIDE_POINT: (f32, f32) = (0.0, 2.8);

fn parts() -> DiverParts {
    build_parts().expect("the diver bottle should construct")
}

fn diver_mass(parts: &DiverParts) -> f32 {
    parts
        .world
        .body_snapshot(parts.hooks.diver)
        .expect("the diver body should exist")
        .mass()
}

fn advance_hooks(parts: &mut DiverParts, steps: u8) {
    for _ in 0..steps {
        parts
            .hooks
            .on_advance(&mut parts.world, parts.particle_system)
            .expect("the ramp should write the diver density");
    }
}

fn pointer(parts: &mut DiverParts, kind: PointerKind, x: f32, y: f32) {
    parts
        .hooks
        .apply_pointer(&mut parts.world, parts.particle_system, kind, x, y)
        .expect("every pointer kind is accepted");
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("the diver scene should capture a frame"),
    )
}

#[test]
fn build_fills_the_bottle_with_the_water_grid_within_the_budget() {
    // Arrange
    let expected = geometry::water_positions().len();

    // Act
    let session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene");

    // Assert
    assert_eq!(session.particle_count(), expected);
    assert!(expected <= PARTICLE_BUDGET, "{expected} particles");
    assert!(expected >= 1_500, "{expected} particles");
}

#[test]
fn water_grid_leaves_the_diver_start_clear() {
    // Arrange
    let clearance = DIVER_RADIUS + PARTICLE_RADIUS;

    // Act
    let positions = geometry::water_positions();

    // Assert
    assert!(
        positions
            .iter()
            .all(|point| (*point - DIVER_START).length() > clearance)
    );
}

#[test]
fn first_frame_draws_the_outline_and_two_circles() {
    // Arrange
    let session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene");

    // Act
    let frame = capture(&session);
    let circles = frame.rigid_circles();

    // Assert
    assert_eq!(frame.rigid_segments().len(), OUTLINE_SEGMENT_COUNT * 4);
    assert_eq!(circles.len(), 6);
    assert_eq!(circles[2].to_bits(), DIVER_RADIUS.to_bits());
}

#[test]
fn build_sets_the_authored_gravity() {
    // Arrange / Act
    let parts = parts();

    // Assert
    let gravity = parts.world.gravity();
    assert_eq!(gravity.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(gravity.y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn squeeze_action_flips_the_intent() {
    // Arrange
    let mut parts = parts();
    let mut observed = Vec::new();

    // Act
    for _ in 0..2 {
        parts
            .hooks
            .apply_action(&mut parts.world, parts.particle_system, SQUEEZE_ACTION)
            .expect("squeeze is the scene action");
        observed.push(parts.hooks.squeezed);
    }

    // Assert
    assert_eq!(observed, vec![true, false]);
}

#[test]
fn unknown_action_fails_closed() {
    // Arrange
    let mut parts = parts();

    // Act
    let result = parts
        .hooks
        .apply_action(&mut parts.world, parts.particle_system, "drop-body");

    // Assert
    assert_eq!(result, Err(SessionError::UnknownControl));
    assert!(!parts.hooks.squeezed);
}

#[test]
fn scene_controls_fail_closed() {
    // Arrange
    let mut session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene");

    // Act
    let result = session.apply_control("anything", "x");

    // Assert
    assert_eq!(result, Err(SessionError::UnknownControl));
}

#[test]
fn pointer_down_inside_the_bottle_squeezes() {
    // Arrange
    let mut parts = parts();

    // Act
    pointer(
        &mut parts,
        PointerKind::Down,
        INSIDE_POINT.0,
        INSIDE_POINT.1,
    );

    // Assert
    assert!(parts.hooks.squeezed);
}

#[test]
fn pointer_down_outside_the_bottle_is_ignored() {
    // Arrange
    let mut parts = parts();

    // Act
    pointer(&mut parts, PointerKind::Down, 2.5, 3.0);

    // Assert
    assert!(!parts.hooks.squeezed);
}

#[test]
fn pointer_move_is_ignored() {
    // Arrange
    let mut parts = parts();

    // Act
    pointer(
        &mut parts,
        PointerKind::Move,
        INSIDE_POINT.0,
        INSIDE_POINT.1,
    );

    // Assert
    assert!(!parts.hooks.squeezed);
}

#[test]
fn pointer_up_anywhere_releases() {
    // Arrange
    let mut parts = parts();
    pointer(
        &mut parts,
        PointerKind::Down,
        INSIDE_POINT.0,
        INSIDE_POINT.1,
    );

    // Act
    pointer(&mut parts, PointerKind::Up, 9.0, 9.0);

    // Assert
    assert!(!parts.hooks.squeezed);
}

#[test]
fn pointer_cancel_anywhere_releases() {
    // Arrange
    let mut parts = parts();
    pointer(
        &mut parts,
        PointerKind::Down,
        INSIDE_POINT.0,
        INSIDE_POINT.1,
    );

    // Act
    pointer(&mut parts, PointerKind::Cancel, -9.0, -9.0);

    // Assert
    assert!(!parts.hooks.squeezed);
}

#[test]
fn squeeze_action_alone_leaves_the_mass_unchanged() {
    // Arrange
    let mut parts = parts();
    let before = diver_mass(&parts);

    // Act
    parts
        .hooks
        .apply_action(&mut parts.world, parts.particle_system, SQUEEZE_ACTION)
        .expect("squeeze is the scene action");

    // Assert
    assert_eq!(diver_mass(&parts).to_bits(), before.to_bits());
    assert_eq!(parts.hooks.squeeze_steps, 0);
}

#[test]
fn ramp_reaches_the_squeezed_density_after_the_ramp_steps() {
    // Arrange
    let mut parts = parts();
    let initial = diver_mass(&parts);
    parts.hooks.squeezed = true;

    // Act
    advance_hooks(&mut parts, RAMP_STEPS);

    // Assert
    let ratio = diver_mass(&parts) / initial;
    let expected = SQUEEZED_DENSITY / RELEASED_DENSITY;
    assert_eq!(parts.hooks.squeeze().to_bits(), 1.0_f32.to_bits());
    assert!(
        ((ratio - expected) / expected).abs() < 1e-4,
        "mass ratio {ratio}, expected {expected}"
    );
}

#[test]
fn ramp_is_partway_before_the_last_step() {
    // Arrange
    let mut parts = parts();
    parts.hooks.squeezed = true;

    // Act
    advance_hooks(&mut parts, RAMP_STEPS - 1);

    // Assert
    let squeeze = parts.hooks.squeeze();
    assert!(squeeze > 0.0 && squeeze < 1.0, "squeeze {squeeze}");
}

#[test]
fn release_ramps_back_to_the_initial_mass() {
    // Arrange
    let mut parts = parts();
    let initial = diver_mass(&parts);
    parts.hooks.squeezed = true;
    advance_hooks(&mut parts, RAMP_STEPS);

    // Act
    parts.hooks.squeezed = false;
    advance_hooks(&mut parts, RAMP_STEPS);

    // Assert
    assert_eq!(parts.hooks.squeeze().to_bits(), 0.0_f32.to_bits());
    assert_eq!(diver_mass(&parts).to_bits(), initial.to_bits());
}

#[test]
fn idle_advance_leaves_the_mass_untouched() {
    // Arrange
    let mut parts = parts();
    let initial = diver_mass(&parts);

    // Act
    advance_hooks(&mut parts, 3);

    // Assert
    assert_eq!(parts.hooks.squeeze_steps, 0);
    assert_eq!(diver_mass(&parts).to_bits(), initial.to_bits());
}

fn side_x_values(squeeze: f32) -> Vec<f32> {
    geometry::outline_segments(squeeze)
        .iter()
        .flat_map(|segment| [segment.start.x, segment.end.x])
        .filter(|x| x.abs() > 0.5)
        .collect()
}

#[test]
fn outline_is_straight_at_rest() {
    // Arrange / Act
    let segments = geometry::outline_segments(0.0);

    // Assert
    for segment in &segments {
        let horizontal = segment.start.y.to_bits() == segment.end.y.to_bits();
        let vertical = segment.start.x.to_bits() == segment.end.x.to_bits();
        assert!(horizontal || vertical, "{segment:?}");
    }
    for x in side_x_values(0.0) {
        assert_eq!(x.abs().to_bits(), OUTLINE_HALF_WIDTH.to_bits());
    }
}

#[test]
fn outline_rectangle_sits_the_gap_outside_the_interior() {
    // Arrange / Act / Assert
    assert!((OUTLINE_HALF_WIDTH - 1.45).abs() < 1e-6);
    assert!((OUTLINE_BOTTOM_Y + 0.25).abs() < 1e-6);
    assert!((OUTLINE_TOP_Y - 5.85).abs() < 1e-6);
    assert!((OUTLINE_HALF_WIDTH - INTERIOR_HALF_WIDTH - OUTLINE_GAP).abs() < 1e-6);
    assert!((OUTLINE_TOP_Y - INTERIOR_HEIGHT - OUTLINE_GAP).abs() < 1e-6);
}

#[test]
fn outline_bows_inward_by_the_dent_depth_at_full_squeeze() {
    // Arrange
    let middle = SIDE_SEGMENTS / 2;

    // Act
    let segments = geometry::outline_segments(1.0);

    // Assert
    let right = segments[1 + middle].start;
    let left = segments[2 + SIDE_SEGMENTS + middle].start;
    assert!(
        (right.x - (OUTLINE_HALF_WIDTH - DENT_DEPTH)).abs() < 1e-6,
        "{right:?}"
    );
    assert!(
        (left.x - (-OUTLINE_HALF_WIDTH + DENT_DEPTH)).abs() < 1e-6,
        "{left:?}"
    );
    assert!((DENT_DEPTH - 0.25).abs() < 1e-6);
}

#[test]
fn outline_corners_stay_fixed_while_squeezed() {
    // Arrange
    let corners = [
        Vec2::new(-OUTLINE_HALF_WIDTH, OUTLINE_BOTTOM_Y),
        Vec2::new(OUTLINE_HALF_WIDTH, OUTLINE_BOTTOM_Y),
        Vec2::new(OUTLINE_HALF_WIDTH, OUTLINE_TOP_Y),
        Vec2::new(-OUTLINE_HALF_WIDTH, OUTLINE_TOP_Y),
    ];

    // Act
    let segments = geometry::outline_segments(1.0);

    // Assert
    let observed = [
        segments[0].start,
        segments[1].start,
        segments[SIDE_SEGMENTS].end,
        segments[2 + SIDE_SEGMENTS].start,
    ];
    for (point, corner) in observed.iter().zip(corners) {
        assert_eq!(point.x.to_bits(), corner.x.to_bits(), "{point:?}");
        assert_eq!(point.y.to_bits(), corner.y.to_bits(), "{point:?}");
    }
    let last = segments[OUTLINE_SEGMENT_COUNT - 1].end;
    assert_eq!(last.x.to_bits(), corners[0].x.to_bits());
    assert_eq!(last.y.to_bits(), corners[0].y.to_bits());
}

#[test]
fn outline_is_a_closed_loop_of_fourteen_segments_at_every_squeeze() {
    // Arrange
    let squeezes = [0.0, 0.25, 0.5, 1.0];

    // Act / Assert
    for squeeze in squeezes {
        let segments = geometry::outline_segments(squeeze);
        assert_eq!(segments.len(), 14);
        for pair in segments.windows(2) {
            assert_eq!(pair[0].end.x.to_bits(), pair[1].start.x.to_bits());
            assert_eq!(pair[0].end.y.to_bits(), pair[1].start.y.to_bits());
        }
    }
}

#[test]
fn bubble_shrinks_from_six_tenths_to_three_tenths_of_the_diver() {
    // Arrange
    let gravity = Vec2::new(0.0, -10.0);

    // Act
    let (_released_center, released) =
        geometry::bubble_circle(DIVER_START, DIVER_RADIUS, 0.0, gravity);
    let (_squeezed_center, squeezed) =
        geometry::bubble_circle(DIVER_START, DIVER_RADIUS, 1.0, gravity);

    // Assert
    assert!((released - 0.6 * DIVER_RADIUS).abs() < 1e-6, "{released}");
    assert!((squeezed - 0.3 * DIVER_RADIUS).abs() < 1e-6, "{squeezed}");
}

#[test]
fn bubble_rides_opposite_gravity() {
    // Arrange
    let down = Vec2::new(0.0, -10.0);
    let right = Vec2::new(10.0, 0.0);

    // Act
    let (above, _) = geometry::bubble_circle(DIVER_START, DIVER_RADIUS, 0.5, down);
    let (left, _) = geometry::bubble_circle(DIVER_START, DIVER_RADIUS, 0.5, right);

    // Assert
    assert!(above.y > DIVER_START.y, "{above:?}");
    assert!(left.x < DIVER_START.x, "{left:?}");
}

#[test]
fn bubble_rides_up_without_gravity() {
    // Arrange / Act
    let (center, _) = geometry::bubble_circle(DIVER_START, DIVER_RADIUS, 0.0, Vec2::ZERO);

    // Assert
    assert_eq!(center.x.to_bits(), DIVER_START.x.to_bits());
    assert!(center.y > DIVER_START.y, "{center:?}");
}

#[test]
fn bubble_stays_inside_the_diver() {
    // Arrange
    let gravities = [Vec2::new(0.0, -10.0), Vec2::new(10.0, 0.0), Vec2::ZERO];
    let squeezes = [0.0, 0.5, 1.0];

    // Act / Assert
    for gravity in gravities {
        for squeeze in squeezes {
            let (center, radius) =
                geometry::bubble_circle(DIVER_START, DIVER_RADIUS, squeeze, gravity);
            let reach = (center - DIVER_START).length() + radius;
            assert!(reach <= DIVER_RADIUS + 1e-6, "reach {reach} at {gravity:?}");
        }
    }
}

#[test]
fn hit_region_includes_its_edges() {
    // Arrange
    let edges = [Vec2::new(1.9, 6.3), Vec2::new(-1.9, -0.7)];

    // Act / Assert
    for point in edges {
        assert!(geometry::in_squeeze_region(point), "{point:?}");
    }
}

#[test]
fn hit_region_excludes_points_past_the_margin() {
    // Arrange
    let outside = [Vec2::new(1.95, 3.0), Vec2::new(0.0, -0.75)];

    // Act / Assert
    for point in outside {
        assert!(!geometry::in_squeeze_region(point), "{point:?}");
    }
}

#[test]
fn hit_region_is_the_outer_walls_plus_the_margin() {
    // Arrange
    let outer_half_width = INTERIOR_HALF_WIDTH + WALL_THICKNESS;
    let cap_top = INTERIOR_HEIGHT + WALL_THICKNESS;

    // Act / Assert
    assert!((HIT_REGION_MAX.x - outer_half_width - geometry::HIT_MARGIN).abs() < 1e-5);
    assert!((HIT_REGION_MIN.x + outer_half_width + geometry::HIT_MARGIN).abs() < 1e-5);
    assert!((HIT_REGION_MIN.y + WALL_THICKNESS + geometry::HIT_MARGIN).abs() < 1e-5);
    assert!((HIT_REGION_MAX.y - cap_top - geometry::HIT_MARGIN).abs() < 1e-5);
}

#[test]
fn thirds_split_the_interior_height() {
    // Arrange / Act / Assert
    assert!((BOTTOM_THIRD_Y - INTERIOR_HEIGHT / 3.0).abs() < 1e-6);
    assert!((TOP_THIRD_Y - INTERIOR_HEIGHT * 2.0 / 3.0).abs() < 1e-6);
    const { assert!(DIVER_START.y >= TOP_THIRD_Y) };
}

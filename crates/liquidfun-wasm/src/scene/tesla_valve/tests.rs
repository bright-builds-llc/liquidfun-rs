//! Tesla valve construction, live controls, and one-way throughput.

use liquidfun::collision::PolygonShape;
use liquidfun::math::Transform;
use liquidfun::math::Vec2;

use super::super::SceneId;
use super::geometry::{
    DRAIN_HALF_WIDTH, DRAIN_TOP_Y, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y,
    collision_polygons, mirror_y, outline_segments, valve_geometry,
};
use crate::session::SessionCore;

#[test]
fn create_draws_the_valve_inside_the_frame_lane() {
    for direction in ["1", "-1"] {
        // Arrange
        let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");

        // Act
        session
            .apply_control("flow-direction", direction)
            .expect("orientation should apply");
        session
            .capture_frame()
            .expect("drawing must fit the frame lane");

        // Assert
        assert_eq!(session.live_particle_count().expect("count"), 0);
        assert_eq!(session.rigid_shape_count(), 60);
    }
}

#[test]
fn wall_corners_stay_inside_the_catalog_frame() {
    // Arrange
    let forward = collision_polygons(true);
    let reverse = collision_polygons(false);

    // Act / Assert
    for polygons in [forward, reverse] {
        for (index, polygon) in polygons.iter().enumerate() {
            assert!(
                PolygonShape::new(polygon).is_ok(),
                "fixture {index} invalid: {polygon:?}"
            );
        }
        for corner in polygons.into_iter().flatten() {
            assert!(
                corner.x >= FRAME_MIN_X
                    && corner.x <= FRAME_MAX_X
                    && corner.y >= FRAME_MIN_Y
                    && corner.y <= FRAME_MAX_Y,
                "corner ({}, {}) left the frame",
                corner.x,
                corner.y
            );
        }
    }
}

#[test]
fn four_curved_heads_alternate_around_the_main_tube() {
    // Arrange
    let geometry = valve_geometry(true);

    // Act / Assert
    assert_eq!(geometry.splitters.len(), 4);
    for (index, island) in geometry.splitters.iter().enumerate() {
        let sign = if index.is_multiple_of(2) { 1.0 } else { -1.0 };
        assert!(island.iter().all(|point| point.x * sign > 0.0));
        assert!(island[5].y < island[0].y, "pointed tips face downstream");
        let outline = outline_segments(true);
        for edge in 0..island.len() {
            assert!(
                outline.iter().any(|segment| {
                    segment.start == island[edge]
                        && segment.end == island[(edge + 1) % island.len()]
                }),
                "splitter outlines must close over the collision polygon"
            );
        }
    }
    assert_eq!(outline_segments(true).len(), 60);
    assert_eq!(outline_segments(true).len(), outline_segments(false).len());
}

#[test]
fn splitter_interiors_are_solid_in_both_orientations() {
    // Arrange
    let forward = valve_geometry(true);
    let reverse = valve_geometry(false);

    // Act / Assert
    for (island, flipped) in forward.splitters.into_iter().zip(reverse.splitters) {
        let center = island
            .iter()
            .copied()
            .fold(Vec2::ZERO, |sum, point| sum + point)
            * (1.0 / 6.0);
        let polygon = PolygonShape::new(&island).expect("splitter should be convex");
        assert!(
            polygon
                .test_point(Transform::IDENTITY, center)
                .expect("valid point")
        );
        assert_eq!(island.map(mirror_y), flipped);
        let reversed = PolygonShape::new(&flipped).expect("reflected splitter should be convex");
        assert!(
            reversed
                .test_point(Transform::IDENTITY, mirror_y(center))
                .expect("valid point")
        );
        assert!(collision_polygons(true).contains(&island.to_vec()));
        assert!(collision_polygons(false).contains(&flipped.to_vec()));
    }
}

#[test]
fn flow_rate_zero_emits_nothing_and_rejects_off_step_values() {
    // Arrange
    let mut stopped =
        SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");
    let mut running =
        SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");

    // Act
    let stopped_control = stopped.apply_control("flow-rate", "0");
    let rejected = running.apply_control("flow-rate", "15");
    let unknown = running.apply_control("nozzle", "1");
    advance_steps(&mut stopped, 30);
    advance_steps(&mut running, 30);

    // Assert
    assert!(stopped_control.is_ok());
    assert!(rejected.is_err());
    assert!(unknown.is_err());
    assert_eq!(stopped.live_particle_count().expect("count"), 0);
    assert!(running.live_particle_count().expect("count") > 20);
}

#[test]
fn forward_reaches_the_drain_sooner_than_the_flipped_valve() {
    // Arrange
    let mut forward =
        SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");
    let mut reverse =
        SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");
    reverse
        .apply_control("flow-direction", "-1")
        .expect("reverse should flip the valve");

    // Act
    // Four alternating stages need a longer transit window than two heads.
    let maybe_forward_arrival = maybe_first_drain_step(&mut forward, true, 360);
    let maybe_reverse_arrival = maybe_first_drain_step(&mut reverse, false, 360);
    let forward_count = forward.live_particle_count().expect("forward count");
    let reverse_count = reverse.live_particle_count().expect("reverse count");
    let forward_positions = positions(&forward);
    let reverse_positions = positions(&reverse);

    // Assert
    let forward_arrival = maybe_forward_arrival.expect("forward flow should reach the drain");
    assert!(maybe_reverse_arrival.is_none_or(|reverse_arrival| forward_arrival < reverse_arrival));
    assert!(
        forward_positions
            .iter()
            .all(|position| !in_drain(*position)),
        "the drain should remove particles that fall through"
    );
    assert!(
        reverse_positions
            .iter()
            .all(|position| !in_drain(*position)),
        "the drain should remove particles that fall through the flipped valve"
    );
    assert!(
        forward_count + 80 < reverse_count,
        "forward held {forward_count} particles (min y {}) and reverse held {reverse_count} (min y {})",
        forward_positions
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min),
        reverse_positions
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min),
    );
    assert!(
        forward_count > 10,
        "forward should still show water in the valve"
    );
}

#[test]
fn shutting_off_the_source_drains_the_running_valve() {
    // Arrange
    let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    advance_contained(&mut session, true, 360);
    let before = session.live_particle_count().expect("count");

    // Act
    session
        .apply_control("flow-rate", "0")
        .expect("source should stop");
    advance_contained(&mut session, true, 240);
    let after = session.live_particle_count().expect("count");

    // Assert
    assert!(
        after + 80 < before,
        "stopped source held {before} then {after} particles"
    );
    assert_contained(&positions(&session), true);
}

#[test]
fn flipping_the_valve_keeps_a_nonzero_flow_rate() {
    // Arrange
    let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    session
        .apply_control("flow-rate", "360")
        .expect("rate should apply");

    // Act
    session
        .apply_control("flow-direction", "-1")
        .expect("valve should reverse");
    advance_steps(&mut session, 15);
    let reverse_count = session.live_particle_count().expect("count");
    session
        .apply_control("flow-direction", "1")
        .expect("valve should restore");
    advance_steps(&mut session, 15);

    // Assert
    assert_eq!(reverse_count, 90);
    assert_eq!(session.live_particle_count().expect("count"), 90);
}

#[test]
fn flipping_back_to_forward_keeps_the_chosen_flow_rate() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");
    session
        .apply_control("flow-rate", "0")
        .expect("flow rate should apply live");

    // Act
    session
        .apply_control("flow-direction", "-1")
        .expect("reverse should flip the valve");
    session
        .apply_control("flow-direction", "1")
        .expect("forward should flip the valve back");
    let rejected = session.apply_control("flow-direction", "0");
    advance_steps(&mut session, 20);

    // Assert
    assert!(rejected.is_err());
    assert_eq!(session.live_particle_count().expect("count"), 0);
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let batch = remaining.min(4);
        session.advance(batch).expect("Tesla Valve should step");
        remaining -= batch;
    }
}

fn maybe_first_drain_step(session: &mut SessionCore, forward: bool, steps: u32) -> Option<u32> {
    let boundary = outer_boundary(forward);
    let mut maybe_arrival = None;
    for elapsed in (4..=steps).step_by(4) {
        session.advance(4).expect("valve should step");
        assert_inside_boundary(&positions(session), &boundary);
        let count = session.live_particle_count().expect("count");
        if maybe_arrival.is_none() && count < (elapsed * 3) as usize {
            maybe_arrival = Some(elapsed);
        }
    }
    maybe_arrival
}

fn advance_contained(session: &mut SessionCore, forward: bool, steps: u32) {
    let boundary = outer_boundary(forward);
    for _ in (4..=steps).step_by(4) {
        session.advance(4).expect("valve should step");
        assert_inside_boundary(&positions(session), &boundary);
    }
}

fn positions(session: &SessionCore) -> Vec<Vec2> {
    session.read_particles(|world, system| {
        world
            .particle_system_view(system)
            .expect("the particle system should stay live")
            .positions()
            .to_vec()
    })
}

fn in_drain(position: Vec2) -> bool {
    position.y < DRAIN_TOP_Y && position.x.abs() < DRAIN_HALF_WIDTH
}

fn assert_contained(positions: &[Vec2], forward: bool) {
    assert_inside_boundary(positions, &outer_boundary(forward));
}

fn outer_boundary(forward: bool) -> Vec<Vec2> {
    let geometry = valve_geometry(forward);
    geometry.outer_paths[0]
        .iter()
        .copied()
        .chain(geometry.outer_paths[1].iter().copied().rev())
        .collect()
}

fn assert_inside_boundary(positions: &[Vec2], boundary: &[Vec2]) {
    for position in positions {
        assert!(position.x.is_finite() && position.y.is_finite());
        if position.y <= 3.02 {
            assert!(
                inside_boundary(*position, boundary),
                "particle escaped at {position:?}"
            );
        }
    }
}

fn inside_boundary(point: Vec2, boundary: &[Vec2]) -> bool {
    let mut inside = false;
    for index in 0..boundary.len() {
        let start = boundary[index];
        let end = boundary[(index + 1) % boundary.len()];
        if (start.y > point.y) != (end.y > point.y) {
            let crossing_x = start.x + (point.y - start.y) * (end.x - start.x) / (end.y - start.y);
            if point.x < crossing_x {
                inside = !inside;
            }
        }
    }
    inside
}

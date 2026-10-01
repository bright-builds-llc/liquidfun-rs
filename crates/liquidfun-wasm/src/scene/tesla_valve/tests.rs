//! Tesla valve construction, live controls, and one-way throughput.

use liquidfun::math::Vec2;

use super::super::SceneId;
use super::geometry::{
    DRAIN_HALF_WIDTH, DRAIN_TOP_Y, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y,
    collision_quads, outlines,
};
use crate::session::SessionCore;

#[test]
fn create_draws_the_valve_inside_the_frame_lane() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::TeslaValve).expect("Tesla Valve should construct");

    // Assert
    session
        .capture_frame()
        .expect("the valve drawing must fit the frame lane");
    assert_eq!(session.live_particle_count().expect("count"), 0);
    assert!(session.rigid_shape_count() <= 64);
    assert!(session.rigid_shape_count() >= 16);
}

#[test]
fn wall_corners_stay_inside_the_catalog_frame() {
    // Arrange
    let forward = collision_quads(true);
    let reverse = collision_quads(false);

    // Act / Assert
    for quads in [forward, reverse] {
        for corner in quads.into_iter().flatten() {
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
fn reverse_ramps_slope_away_from_the_forward_lip() {
    // Arrange
    let forward = collision_quads(true);
    let reverse = collision_quads(false);
    let forward_ramp = forward[4];
    let reverse_ramp = reverse[4];

    // Act
    let forward_closed_y = edge_y(&forward_ramp, 0, 3);
    let forward_lip_y = edge_y(&forward_ramp, 1, 2);
    let reverse_closed_y = edge_y(&reverse_ramp, 0, 3);
    let reverse_lip_y = edge_y(&reverse_ramp, 1, 2);

    // Assert
    assert!(
        forward_lip_y < forward_closed_y,
        "forward water should slide down toward the lip"
    );
    assert!(
        reverse_lip_y > reverse_closed_y,
        "the flipped tooth should climb toward the old lip"
    );
    assert_eq!(outlines(&forward).len(), outlines(&reverse).len());
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
    advance_steps(&mut forward, 180);
    advance_steps(&mut reverse, 180);
    let forward_count = forward.live_particle_count().expect("forward count");
    let reverse_count = reverse.live_particle_count().expect("reverse count");
    let forward_positions = positions(&forward);
    let reverse_positions = positions(&reverse);

    // Assert
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
        "forward held {forward_count} particles and reverse held {reverse_count}"
    );
    assert!(
        forward_count > 10,
        "forward should still show water in the valve"
    );
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

fn edge_y(quad: &[Vec2; 4], start: usize, end: usize) -> f32 {
    (quad[start].y + quad[end].y) * 0.5
}

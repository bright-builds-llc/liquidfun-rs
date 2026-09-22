//! Pinned LiquidFun Particles test: open basin, water circle, dynamic ball.
//!
//! Construction and factory wiring land in plan 26-01 Task 2. These tests are the
//! RED coverage that must fail until `SceneId::Particles` and `particles::build`
//! exist.

use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::{SessionCore, SessionError};

#[test]
fn create_builds_open_basin_water_and_ball() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::Particles)
        .expect("Particles should construct the pinned open basin");
    let frame = capture(&session);

    // Assert
    assert!(
        session.particle_count() > 0,
        "water group must create at least one particle"
    );
    assert!(
        session.rigid_shape_count() >= 4,
        "three basin segments plus one dynamic ball"
    );
    assert!(
        frame.rigid_circles().len() >= 3,
        "captured frame must export at least one rigid circle (x,y,r)"
    );
}

#[test]
fn eight_advances_keep_particles_alive() {
    // Arrange
    let mut session = SessionCore::create(SceneId::Particles)
        .expect("Particles should construct the pinned open basin");
    let before = session.particle_count();

    // Act
    for _ in 0..8 {
        session
            .advance(4)
            .expect("Particles advance must stay within the catch-up cap");
    }

    // Assert
    assert!(before > 0);
    assert!(
        session.particle_count() > 0,
        "particle count must not drop to zero after eight capped advances"
    );
}

#[test]
fn unknown_control_and_action_are_rejected() {
    // Arrange
    let mut session = SessionCore::create(SceneId::Particles)
        .expect("Particles should construct the pinned open basin");
    let before_count = session.particle_count();

    // Act
    let control = session.apply_control("water-amount", "medium");
    let action = session.apply_action("drop-obstacle");

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(session.particle_count(), before_count);
}

#[test]
fn pointer_is_a_noop_without_mutating_particle_count() {
    // Arrange
    let mut session = SessionCore::create(SceneId::Particles)
        .expect("Particles should construct the pinned open basin");
    let before_count = session.particle_count();

    // Act
    session
        .apply_pointer("down", 0.0, 4.0)
        .expect("watch-first pointer must succeed as a no-op");
    session
        .apply_pointer("move", 1.0, 3.0)
        .expect("watch-first pointer must succeed as a no-op");
    session
        .apply_pointer("up", 1.0, 3.0)
        .expect("watch-first pointer must succeed as a no-op");

    // Assert
    assert_eq!(session.particle_count(), before_count);
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("Particles frame capture should succeed"),
    )
}

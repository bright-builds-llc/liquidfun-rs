//! Pinned LiquidFun Liquid Timer test: tensile/viscous drain through shelves.
//!
//! Construction and factory wiring land in plan 26-02 Task 2. These tests are the
//! RED coverage that must fail until `SceneId::LiquidTimer` and `liquid_timer::build`
//! exist.

use liquidfun::ParticleFlags;

use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::{SessionCore, SessionError};

#[test]
fn create_builds_bowl_slab_and_drain_geometry() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::LiquidTimer)
        .expect("Liquid Timer should construct the pinned drain bowl");
    let frame = capture(&session);

    // Assert
    assert!(
        session.particle_count() > 0,
        "tensile/viscous slab must create at least one particle"
    );
    assert!(
        session.rigid_shape_count() >= 10,
        "collect_segments must export every shelf and column edge"
    );
    assert!(
        frame.rigid_segments().len() >= 40,
        "ten edges × four floats each must be drawable"
    );
}

#[test]
fn constructed_group_flags_include_tensile_and_viscous() {
    // Arrange / Act
    let super::BuiltScene {
        world,
        particle_system,
        ..
    } = super::build(&[]).expect("Liquid Timer should construct");
    let view = world
        .particle_system_view(particle_system)
        .expect("constructed system should stay live");
    let expected = ParticleFlags::TENSILE | ParticleFlags::VISCOUS;

    // Assert
    assert!(
        view.flags().iter().any(|flags| {
            flags.contains(ParticleFlags::TENSILE) && flags.contains(ParticleFlags::VISCOUS)
        }),
        "group particles should carry {expected:?}"
    );
}

#[test]
fn several_advances_keep_particles_alive() {
    // Arrange
    let mut session = SessionCore::create(SceneId::LiquidTimer)
        .expect("Liquid Timer should construct the pinned drain bowl");
    let before = session.particle_count();

    // Act
    for _ in 0..8 {
        session
            .advance(4)
            .expect("Liquid Timer advance must stay within the catch-up cap");
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
    let mut session = SessionCore::create(SceneId::LiquidTimer)
        .expect("Liquid Timer should construct the pinned drain bowl");
    let before_count = session.particle_count();

    // Act
    let control = session.apply_control("viscosity", "high");
    let action = session.apply_action("refill");

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(session.particle_count(), before_count);
}

#[test]
fn pointer_is_a_noop_without_mutating_particle_count() {
    // Arrange
    let mut session = SessionCore::create(SceneId::LiquidTimer)
        .expect("Liquid Timer should construct the pinned drain bowl");
    let before_count = session.particle_count();

    // Act
    session
        .apply_pointer("down", 0.0, 3.0)
        .expect("watch-first pointer must succeed as a no-op");
    session
        .apply_pointer("move", 0.5, 2.0)
        .expect("watch-first pointer must succeed as a no-op");
    session
        .apply_pointer("up", 0.5, 2.0)
        .expect("watch-first pointer must succeed as a no-op");

    // Assert
    assert_eq!(session.particle_count(), before_count);
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("Liquid Timer frame capture should succeed"),
    )
}

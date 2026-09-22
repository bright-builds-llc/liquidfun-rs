//! Pinned LiquidFun Surface Tension test: tensile color-mixing groups + falling ball.

use crate::session::SessionError;

use super::BuiltScene;

pub(crate) fn build(_presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    // RED stub: Task 2 replaces this with the pinned basin + groups + ball.
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use liquidfun::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_groups_and_ball() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "tensile color-mixing groups must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn constructed_group_flags_include_tensile_and_color_mixing() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Surface Tension should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::TENSILE | ParticleFlags::COLOR_MIXING;

        // Assert
        assert!(
            view.flags().iter().any(|flags| {
                flags.contains(ParticleFlags::TENSILE)
                    && flags.contains(ParticleFlags::COLOR_MIXING)
            }),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn several_advances_keep_particles_alive() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Surface Tension advance must stay within the catch-up cap");
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
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let before_count = session.particle_count();

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(session.particle_count(), before_count);
    }

    #[test]
    fn pointer_is_a_noop_without_mutating_particle_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
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
                .expect("Surface Tension frame capture should succeed"),
        )
    }
}

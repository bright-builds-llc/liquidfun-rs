//! Pinned `LiquidFun` Soup test: basin broth with floating solids (watch-first).

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    // Task 1 RED stub: soup_family + carve land in Task 2.
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_liquid_and_floating_solids() {
        // Arrange / Act
        let session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "water group must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn several_advances_keep_particles_alive() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Soup advance must stay within the catch-up cap");
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
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
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
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let before_count = session.particle_count();

        // Act
        session
            .apply_pointer("down", 0.0, 1.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("move", 0.5, 0.5)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("up", 0.5, 0.5)
            .expect("watch-first pointer must succeed as a no-op");

        // Assert
        assert_eq!(session.particle_count(), before_count);
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Soup frame capture should succeed"),
        )
    }
}

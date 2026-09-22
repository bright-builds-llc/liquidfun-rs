//! Pinned `LiquidFun` Soup Stirrer: soup basin plus paddle on a prismatic rail.

use super::{BuiltScene, SceneId};
use crate::session::SessionError;

/// Stub until Task 2 implements paddle, carve, prismatic toggle, and stir.
pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = (presets, SceneId::SoupStirrer);
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::{ControlEffect, PointerKind, SceneId};
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_soup_with_paddle_circle() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "water group must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 4,
            "captured frame must export soup solids plus the paddle circle"
        );
    }

    #[test]
    fn toggle_paddle_rail_twice_returns_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Soup Stirrer should construct over soup_family");
        assert_eq!(
            world.joint_count(),
            1,
            "rail must start attached after construction"
        );

        // Act
        let first = hooks
            .apply_control(&mut world, particle_system, "toggle-paddle-rail", "")
            .expect("first toggle must detach the rail");
        let joints_after_detach = world.joint_count();
        let second = hooks
            .apply_control(&mut world, particle_system, "toggle-paddle-rail", "")
            .expect("second toggle must restore the rail");
        let joints_after_reattach = world.joint_count();

        // Assert
        assert!(matches!(first, ControlEffect::Live));
        assert!(matches!(second, ControlEffect::Live));
        assert_eq!(joints_after_detach, 0);
        assert_eq!(joints_after_reattach, 1);
    }

    #[test]
    fn pointer_up_inside_soup_toggles_rail_outside_is_noop() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Soup Stirrer should construct over soup_family");
        assert_eq!(world.joint_count(), 1);

        // Act — outside AABB must not toggle
        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("out-of-soup pointer up must succeed as a no-op");
        assert_eq!(
            world.joint_count(),
            1,
            "pointer outside InSoup AABB must leave the rail attached"
        );

        // Act — inside AABB toggles (detaches)
        hooks
            .apply_pointer(&mut world, particle_system, PointerKind::Up, 0.0, 0.5)
            .expect("InSoup pointer up must toggle the rail");
        assert_eq!(world.joint_count(), 0);

        // Act — outside again must not reattach
        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("out-of-soup pointer up must succeed as a no-op");
        assert_eq!(
            world.joint_count(),
            0,
            "pointer outside InSoup AABB must leave rail state unchanged"
        );
    }

    #[test]
    fn remount_starts_with_rail_attached_so_first_toggle_detaches() {
        // Arrange — detach on a live session, then remount via create
        let mut session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");
        session
            .apply_control("toggle-paddle-rail", "")
            .expect("detach before remount");

        // Act
        session = SessionCore::create(SceneId::SoupStirrer)
            .expect("remount must reconstruct Soup Stirrer");
        let first = session
            .apply_control("toggle-paddle-rail", "")
            .expect("first toggle after remount must detach");

        // Assert — recreate starts attached (first toggle is a successful detach/live path)
        assert!(!first, "toggle is a live control, not a recreate");
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("fresh build must start with rail attached");
        assert_eq!(
            world.joint_count(),
            1,
            "construction must restore the paddle on the rail"
        );
        hooks
            .apply_control(&mut world, particle_system, "toggle-paddle-rail", "")
            .expect("first toggle must destroy the attached rail");
        assert_eq!(world.joint_count(), 0);
        let _ = session;
    }

    #[test]
    fn unknown_control_is_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Soup Stirrer frame capture should succeed"),
        )
    }
}

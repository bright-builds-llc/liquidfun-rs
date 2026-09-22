//! Pinned `LiquidFun` Elastic Particles test: spring/elastic SOLID clumps + spinning box.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    // RED stub: Task 2 implements the pinned basin + three soft SOLID groups + ball.
    Err(SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use liquidfun::particle::{ParticleFlags, ParticleGroupFlags};

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_groups_and_ball() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::ElasticParticles)
            .expect("Elastic Particles should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "soft clumps must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn constructed_groups_use_distinct_spring_and_elastic_with_solid() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Elastic Particles should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        let has_spring = view
            .flags()
            .iter()
            .any(|flags| flags.contains(ParticleFlags::SPRING));
        let has_elastic = view
            .flags()
            .iter()
            .any(|flags| flags.contains(ParticleFlags::ELASTIC));
        let has_combined = view.flags().iter().any(|flags| {
            flags.contains(ParticleFlags::SPRING) && flags.contains(ParticleFlags::ELASTIC)
        });

        let mut solid_group_count = 0usize;
        let mut seen = Vec::new();
        for maybe_group in view.group_ids() {
            let Some(group) = maybe_group else {
                continue;
            };
            if seen.contains(group) {
                continue;
            }
            seen.push(*group);
            let group_view = world
                .particle_group_view(*group)
                .expect("group should stay live");
            if group_view.flags().contains(ParticleGroupFlags::SOLID) {
                solid_group_count += 1;
            }
        }

        // Assert
        assert!(has_spring, "at least one particle should carry SPRING");
        assert!(has_elastic, "at least one particle should carry ELASTIC");
        assert!(
            !has_combined,
            "Elastic Particles must not collapse into Jelly Drop's ELASTIC|SPRING recipe"
        );
        assert!(
            solid_group_count >= 3,
            "all three soft clumps must carry ParticleGroupFlags::SOLID"
        );
    }

    #[test]
    fn blue_box_group_has_pinned_angle_and_spin() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Elastic Particles should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        let mut seen = Vec::new();
        let mut found_spinning_box = false;
        for maybe_group in view.group_ids() {
            let Some(group) = maybe_group else {
                continue;
            };
            if seen.contains(group) {
                continue;
            }
            seen.push(*group);
            let group_view = world
                .particle_group_view(*group)
                .expect("group should stay live");
            let angle = group_view.angle();
            let angular_velocity = group_view.angular_velocity();
            if (angle - (-0.5)).abs() < 1e-3 && (angular_velocity - 2.0).abs() < 1e-3 {
                found_spinning_box = true;
                break;
            }
        }

        // Assert
        assert!(
            found_spinning_box,
            "blue box group must use transform angle ≈ -0.5 and angular velocity 2.0"
        );
    }

    #[test]
    fn several_advances_do_not_panic() {
        // Arrange
        let mut session = SessionCore::create(SceneId::ElasticParticles)
            .expect("Elastic Particles should construct the pinned basin");

        // Act / Assert
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Elastic Particles advance must stay within the catch-up cap");
        }
        assert!(session.particle_count() > 0);
    }

    #[test]
    fn unknown_control_and_action_are_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::ElasticParticles)
            .expect("Elastic Particles should construct the pinned basin");
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
        let mut session = SessionCore::create(SceneId::ElasticParticles)
            .expect("Elastic Particles should construct the pinned basin");
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
                .expect("Elastic Particles frame capture should succeed"),
        )
    }
}

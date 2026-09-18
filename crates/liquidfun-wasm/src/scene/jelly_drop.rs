//! Unimplemented Jelly Drop scene. A later plan owns the persistent world.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneUnimplemented)
}

#[cfg(test)]
mod tests {
    use liquidfun::particle::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn create_jelly_drop_builds_a_bounded_elastic_group_on_two_bars() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let frame = capture(&session);

        // Assert
        assert!((8..=220).contains(&session.particle_count()));
        assert!(session.particle_count() <= 512);
        assert!(
            frame.rigid_segments().len() >= 8,
            "two rigid bars report at least eight segment floats"
        );
    }

    #[test]
    fn constructed_group_flags_include_elastic_and_spring() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Jelly Drop should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::ELASTIC | ParticleFlags::SPRING;

        // Assert
        assert!(
            view.flags()
                .iter()
                .any(|flags| flags.contains(ParticleFlags::ELASTIC)
                    && flags.contains(ParticleFlags::SPRING)),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn thirty_steps_keep_the_particle_count_constant() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let before = capture(&session).particle_count();

        // Act
        advance_steps(&mut session, 30);
        let after = capture(&session).particle_count();

        // Assert
        assert_eq!(after, before, "Jelly Drop must not emit extra particles");
        assert!((8..=220).contains(&after));
    }

    fn advance_steps(session: &mut SessionCore, steps: u32) {
        let mut remaining = steps;
        while remaining > 0 {
            let chunk = remaining.min(4);
            session
                .advance(chunk)
                .expect("bounded native steps should succeed");
            remaining -= chunk;
        }
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Jelly Drop should capture a frame"),
        )
    }
}

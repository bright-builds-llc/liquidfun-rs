//! Unimplemented Color Mixer scene. A later plan owns the persistent world.

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

    const TEAL: [u8; 4] = [57, 211, 199, 255];
    const RED: [u8; 4] = [248, 113, 113, 255];

    #[test]
    fn create_succeeds_with_two_distinct_mixing_colors() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::ColorMixer)
            .expect("Color Mixer should construct two mixing groups");
        let frame = capture(&session);
        let colors = frame.particle_colors();

        // Assert
        assert!((40..=220).contains(&session.particle_count()));
        assert!(
            colors_contain(colors.as_ref(), TEAL),
            "captured colors should include teal (57, 211, 199, 255)"
        );
        assert!(
            colors_contain(colors.as_ref(), RED),
            "captured colors should include destructive red (248, 113, 113, 255)"
        );
    }

    #[test]
    fn every_constructed_particle_carries_color_mixing() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Color Mixer should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        // Assert
        assert!(!view.flags().is_empty());
        assert!(
            view.flags()
                .iter()
                .all(|flags| flags.contains(ParticleFlags::COLOR_MIXING)),
            "every particle must carry COLOR_MIXING so contact mixing can run"
        );
    }

    fn colors_contain(colors: &[u8], expected: [u8; 4]) -> bool {
        colors.chunks_exact(4).any(|chunk| chunk == expected)
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Color Mixer should capture a frame"),
        )
    }
}

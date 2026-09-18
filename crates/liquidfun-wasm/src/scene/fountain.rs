//! Unimplemented Fountain scene. A later plan owns the persistent world.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneUnimplemented)
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::{ControlEffect, SceneId};
    use crate::session::SessionCore;

    #[test]
    fn create_fountain_succeeds() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::Fountain);

        // Assert
        assert!(
            session.is_ok(),
            "Fountain should construct a bounded native stream"
        );
    }

    #[test]
    fn default_medium_stream_plateaus_at_or_below_three_hundred_twenty() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Fountain)
            .expect("Fountain should construct a bounded native stream");

        // Act
        advance_steps(&mut session, 210);
        let mid_count = capture(&session).particle_count();
        advance_steps(&mut session, 30);
        let end_count = capture(&session).particle_count();

        // Assert
        assert!(end_count <= 320);
        assert!(
            end_count <= mid_count + 2,
            "count should plateau instead of climbing toward 512: {mid_count} -> {end_count}"
        );
    }

    #[test]
    fn emission_rate_off_does_not_increase_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Fountain)
            .expect("Fountain should construct a bounded native stream");
        let off = session
            .apply_control("emission-rate", "off")
            .expect("emission-rate=off should apply live");
        let before = capture(&session).particle_count();

        // Act
        advance_steps(&mut session, 60);
        let after = capture(&session).particle_count();

        // Assert
        assert!(!off, "emission-rate must return Live");
        assert_eq!(after, before, "off emission must not add particles");
    }

    #[test]
    fn fountain_controls_apply_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Fountain should construct");

        // Act
        let rate = hooks
            .apply_control(&mut world, particle_system, "emission-rate", "high")
            .expect("emission-rate should apply");
        let speed = hooks
            .apply_control(&mut world, particle_system, "launch-speed", "fast")
            .expect("launch-speed should apply");
        let aim = hooks
            .apply_control(&mut world, particle_system, "aim-angle", "left")
            .expect("aim-angle should apply");

        // Assert
        assert!(matches!(rate, ControlEffect::Live));
        assert!(matches!(speed, ControlEffect::Live));
        assert!(matches!(aim, ControlEffect::Live));
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
                .expect("Fountain should capture a frame"),
        )
    }
}

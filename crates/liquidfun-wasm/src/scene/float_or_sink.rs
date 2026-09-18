//! Unimplemented Float or Sink scene. A later plan owns the persistent world.

use super::BuiltScene;
use crate::session::SessionError;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    Err(SessionError::SceneUnimplemented)
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    const Y_SEPARATION_STEPS: u32 = 120;

    #[test]
    fn create_float_or_sink_builds_a_pool_without_dropped_circles() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");
        let frame = capture(&session);

        // Assert
        assert!((120..=220).contains(&session.particle_count()));
        assert!(session.particle_count() <= 384);
        assert!(frame.particle_count() >= 1);
        assert!(frame.rigid_segments().len() >= 4);
        assert!(frame.rigid_circles().is_empty());
    }

    #[test]
    fn particle_count_stays_at_or_below_the_frame_budget() {
        // Arrange
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");

        // Act
        advance_steps(&mut session, Y_SEPARATION_STEPS);
        let frame = capture(&session);

        // Assert
        assert!(session.particle_count() <= 384);
        assert!(frame.particle_count() <= 384);
    }

    #[test]
    fn cork_finishes_above_stone_after_the_same_native_steps() {
        // Arrange / Act
        let cork_y = dropped_body_y_after_steps("cork", Y_SEPARATION_STEPS);
        let stone_y = dropped_body_y_after_steps("stone", Y_SEPARATION_STEPS);

        // Assert
        assert!(
            cork_y > stone_y,
            "cork y {cork_y} should stay above stone y {stone_y} from native coupling"
        );
        assert!(
            cork_y > 1.0,
            "cork y {cork_y} should remain above the basin floor after contacting the pool"
        );
    }

    fn dropped_body_y_after_steps(preset: &str, steps: u32) -> f32 {
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");
        let recreated = session
            .apply_control("body", preset)
            .expect("body preset should apply live");
        assert!(!recreated, "body preset must not recreate the pool");
        session
            .apply_action("drop-body")
            .expect("drop-body should create one dynamic fixture");
        advance_steps(&mut session, steps);
        let circles = capture(&session).rigid_circles();
        assert_eq!(circles.len(), 3, "one dropped circle reports x, y, radius");
        circles[1]
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
                .expect("Float or Sink should capture a frame"),
        )
    }

    #[test]
    fn unknown_control_names_still_fail_closed() {
        // Arrange
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");

        // Act
        let result = session.apply_control("nope", "x");

        // Assert
        assert_eq!(result, Err(SessionError::UnknownControl));
    }
}

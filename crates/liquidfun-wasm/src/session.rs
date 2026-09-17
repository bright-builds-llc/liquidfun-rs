//! Native-testable ownership and stepping for one proof scene.

use crate::frame::FrameData;

pub(crate) const MAX_ADVANCE_STEPS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionError {
    SceneConstruction,
    StepCountOutOfRange,
    StepIndexExhausted,
    StepFailed,
    FrameCaptureFailed,
}

impl SessionError {
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::SceneConstruction => "Rust/WASM scene construction failed",
            Self::StepCountOutOfRange => "Rust/WASM step count must be within 1 through 4",
            Self::StepIndexExhausted => "Rust/WASM step index exhausted",
            Self::StepFailed => "Rust/WASM simulation step failed",
            Self::FrameCaptureFailed => "Rust/WASM frame capture failed",
        }
    }
}

pub(crate) struct SessionCore {
    step_index: u32,
}

impl SessionCore {
    pub(crate) fn new() -> Result<Self, SessionError> {
        Err(SessionError::SceneConstruction)
    }

    pub(crate) fn advance(&mut self, step_count: u32) -> Result<(), SessionError> {
        if !(1..=MAX_ADVANCE_STEPS).contains(&step_count) {
            return Err(SessionError::StepCountOutOfRange);
        }

        Err(SessionError::StepFailed)
    }

    pub(crate) fn capture_frame(&self) -> Result<FrameData, SessionError> {
        Err(SessionError::FrameCaptureFailed)
    }

    pub(crate) const fn step_index(&self) -> u32 {
        self.step_index
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;

    use super::*;

    fn new_session() -> SessionCore {
        SessionCore::new().expect("fixed proof scene should construct")
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("fixed proof scene should capture"),
        )
    }

    #[test]
    fn construction_creates_exact_bounded_colored_scene() {
        // Arrange
        let session = new_session();

        // Act
        let frame = capture(&session);

        // Assert
        assert_eq!(frame.step_index(), 0);
        assert_eq!(frame.particle_count(), 192);
        assert_eq!(frame.rigid_shape_count(), 4);
        assert_eq!(frame.rigid_segments().len(), 12);
        assert_eq!(frame.rigid_circles().len(), 3);
        assert_eq!(frame.particle_radii(), vec![0.2; 192].into_boxed_slice());
        assert_eq!(
            frame.particle_colors(),
            vec![57, 211, 199, 255]
                .repeat(192)
                .into_boxed_slice()
        );
    }

    #[test]
    fn advance_rejects_out_of_range_counts_without_effect() {
        // Arrange
        let rejected_counts = [0, MAX_ADVANCE_STEPS + 1];

        for rejected_count in rejected_counts {
            let mut session = new_session();
            let before = capture(&session).particle_positions();

            // Act
            let result = session.advance(rejected_count);

            // Assert
            assert_eq!(result, Err(SessionError::StepCountOutOfRange));
            assert_eq!(session.step_index(), 0);
            assert_eq!(capture(&session).particle_positions(), before);
        }
    }

    #[test]
    fn advance_increments_step_index_by_accepted_count() {
        // Arrange
        let mut session = new_session();

        // Act
        session
            .advance(MAX_ADVANCE_STEPS)
            .expect("bounded fixed steps should succeed");

        // Assert
        assert_eq!(session.step_index(), MAX_ADVANCE_STEPS);
    }

    #[test]
    fn captured_frame_lanes_are_aligned_finite_and_positive() {
        // Arrange
        let session = new_session();

        // Act
        let frame = capture(&session);

        // Assert
        assert_eq!(frame.particle_positions().len(), 192 * 2);
        assert_eq!(frame.particle_colors().len(), 192 * 4);
        assert_eq!(frame.particle_radii().len(), 192);
        assert!(
            frame
                .particle_positions()
                .iter()
                .all(|value| value.is_finite())
        );
        assert!(
            frame
                .particle_radii()
                .iter()
                .all(|radius| radius.is_finite() && *radius > 0.0)
        );
        assert!(frame.rigid_segments().iter().all(|value| value.is_finite()));
        assert!(frame.rigid_circles().iter().all(|value| value.is_finite()));
    }

    #[test]
    fn four_steps_move_particle_or_dynamic_circle_state() {
        // Arrange
        let mut session = new_session();
        let before = capture(&session);

        // Act
        session
            .advance(MAX_ADVANCE_STEPS)
            .expect("bounded fixed steps should succeed");
        let after = capture(&session);

        // Assert
        assert!(
            before.particle_positions() != after.particle_positions()
                || before.rigid_circles() != after.rigid_circles()
        );
    }

    #[test]
    fn step_index_overflow_is_rejected_before_world_effects() {
        // Arrange
        let mut session = new_session();
        session.step_index = u32::MAX;
        let before = capture(&session).particle_positions();

        // Act
        let result = session.advance(1);

        // Assert
        assert_eq!(result, Err(SessionError::StepIndexExhausted));
        assert_eq!(capture(&session).particle_positions(), before);
    }
}

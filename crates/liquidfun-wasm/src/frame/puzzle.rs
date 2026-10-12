//! Optional puzzle scalars on `ProofFrame` (D-17); undefined in JS for scene frames.

use wasm_bindgen::prelude::*;

use super::ProofFrame;

#[wasm_bindgen]
impl ProofFrame {
    /// Returns the counted particles strictly inside the cup, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalCount)]
    pub fn goal_count(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.goal_count)
    }

    /// Returns the count that must hold for the settle window, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalTarget)]
    pub fn goal_target(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.target_count)
    }

    /// Returns the authored count for 100% fill, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalFull)]
    pub fn goal_full(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.full_count)
    }

    /// Returns 0 while running, 1 once won and 2 once lost, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = outcome)]
    pub fn outcome(&self) -> Option<u8> {
        self.data.maybe_puzzle.map(|status| status.outcome.code())
    }

    /// Returns the latched outcome step, or undefined while running and for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = outcomeStep)]
    pub fn outcome_step(&self) -> Option<u32> {
        self.data
            .maybe_puzzle
            .and_then(|status| status.outcome.maybe_step())
    }

    /// Returns the level step limit, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = stepLimit)]
    pub fn step_limit(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.step_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FrameData, FrameDiagnostics, FrameError};
    use super::*;
    use crate::puzzle::goal::{Outcome, PuzzleStatus};

    const PARTICLES: usize = 4;

    fn frame_data_at(step_index: u32) -> FrameData {
        FrameData::new(
            step_index,
            vec![1.0; PARTICLES * 2],
            vec![255; PARTICLES * 4],
            vec![0.2; PARTICLES],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        )
        .expect("four-particle frame should pass validation")
    }

    const fn status(outcome: Outcome, observed_step: u32) -> PuzzleStatus {
        PuzzleStatus {
            goal_count: 2,
            target_count: 3,
            full_count: 4,
            outcome,
            observed_step,
            step_limit: 600,
        }
    }

    fn puzzle_frame(status: PuzzleStatus) -> ProofFrame {
        let mut data = frame_data_at(status.observed_step);
        data.set_puzzle_status(status)
            .expect("consistent status should attach");
        ProofFrame::from(data)
    }

    #[test]
    fn scene_frames_report_no_puzzle_scalars() {
        // Arrange
        let frame = ProofFrame::from(frame_data_at(5));

        // Act
        let scalars = (
            frame.goal_count(),
            frame.goal_target(),
            frame.goal_full(),
            frame.outcome(),
            frame.outcome_step(),
            frame.step_limit(),
        );

        // Assert
        assert_eq!(scalars, (None, None, None, None, None, None));
    }

    #[test]
    fn running_puzzle_frames_report_counts_and_no_outcome_step() {
        // Arrange
        let frame = puzzle_frame(status(Outcome::Running, 5));

        // Act
        let scalars = (
            frame.goal_count(),
            frame.goal_target(),
            frame.goal_full(),
            frame.outcome(),
            frame.outcome_step(),
            frame.step_limit(),
        );

        // Assert
        assert_eq!(
            scalars,
            (Some(2), Some(3), Some(4), Some(0), None, Some(600))
        );
    }

    #[test]
    fn won_frames_report_outcome_one_and_its_step() {
        // Arrange
        let frame = puzzle_frame(status(Outcome::Won { step: 5 }, 5));

        // Act
        let outcome = (frame.outcome(), frame.outcome_step());

        // Assert
        assert_eq!(outcome, (Some(1), Some(5)));
    }

    #[test]
    fn lost_frames_report_outcome_two_at_the_step_limit() {
        // Arrange
        let frame = puzzle_frame(status(Outcome::Lost { step: 600 }, 600));

        // Act
        let outcome = (frame.outcome(), frame.outcome_step());

        // Assert
        assert_eq!(outcome, (Some(2), Some(600)));
    }

    #[test]
    fn set_puzzle_status_rejects_inconsistent_status() {
        // Arrange
        let base = status(Outcome::Running, 5);
        let invalid = [
            PuzzleStatus {
                target_count: 5,
                ..base
            },
            PuzzleStatus {
                target_count: 0,
                ..base
            },
            PuzzleStatus {
                goal_count: 5,
                full_count: 6,
                ..base
            },
            PuzzleStatus {
                outcome: Outcome::Won { step: 601 },
                observed_step: 5,
                ..base
            },
            PuzzleStatus {
                outcome: Outcome::Won { step: 6 },
                ..base
            },
        ];

        // Act
        let results = invalid.map(|candidate| frame_data_at(5).set_puzzle_status(candidate));

        // Assert
        assert_eq!(results, [Err(FrameError::InvalidDiagnostic); 5]);
    }

    #[test]
    fn set_puzzle_status_rejects_a_step_counter_out_of_sync_with_the_frame() {
        // Arrange
        let mut data = frame_data_at(5);

        // Act
        let result = data.set_puzzle_status(status(Outcome::Running, 4));

        // Assert
        assert_eq!(result, Err(FrameError::InvalidDiagnostic));
    }
}

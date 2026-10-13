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

/// A running status: target 3, brim 4, full 4, nothing held yet.
const fn status(outcome: Outcome, observed_step: u32) -> PuzzleStatus {
    PuzzleStatus {
        goal_count: 2,
        target_count: 3,
        full_count: 4,
        outcome,
        observed_step,
        step_limit: 600,
        peak_held_count: 0,
        brim_count: 4,
        maybe_brim_step: None,
        maybe_decided_step: None,
    }
}

/// Won at `step` with the peak at target and no Brim, so still undecided.
const fn won_status(step: u32) -> PuzzleStatus {
    PuzzleStatus {
        peak_held_count: 3,
        ..status(Outcome::Won { step }, step)
    }
}

/// Lost at the step limit, decided there.
const fn lost_status() -> PuzzleStatus {
    PuzzleStatus {
        maybe_decided_step: Some(600),
        ..status(Outcome::Lost { step: 600 }, 600)
    }
}

/// The first-pour reference run as research measured it at its Brim step.
const fn brim_status() -> PuzzleStatus {
    PuzzleStatus {
        goal_count: 2,
        target_count: 540,
        full_count: 900,
        outcome: Outcome::Won { step: 201 },
        observed_step: 262,
        step_limit: 720,
        peak_held_count: 811,
        brim_count: 810,
        maybe_brim_step: Some(262),
        maybe_decided_step: Some(262),
    }
}

fn puzzle_frame(status: PuzzleStatus) -> ProofFrame {
    let mut data = frame_data_at(status.observed_step);
    data.set_puzzle_status(status)
        .expect("consistent status should attach");
    ProofFrame::from(data)
}

fn attach(status: PuzzleStatus) -> Result<(), FrameError> {
    frame_data_at(status.observed_step).set_puzzle_status(status)
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
fn scene_frames_report_no_star_scalars() {
    // Arrange
    let frame = ProofFrame::from(frame_data_at(5));

    // Act
    let scalars = (
        frame.peak_held_count(),
        frame.brim_count(),
        frame.brim_step(),
        frame.decided_step(),
    );

    // Assert
    assert_eq!(scalars, (None, None, None, None));
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
    let frame = puzzle_frame(won_status(5));

    // Act
    let outcome = (frame.outcome(), frame.outcome_step());

    // Assert
    assert_eq!(outcome, (Some(1), Some(5)));
}

#[test]
fn lost_frames_report_outcome_two_at_the_step_limit() {
    // Arrange
    let frame = puzzle_frame(lost_status());

    // Act
    let outcome = (frame.outcome(), frame.outcome_step(), frame.decided_step());

    // Assert
    assert_eq!(outcome, (Some(2), Some(600), Some(600)));
}

#[test]
fn brim_frames_report_peak_brim_and_decision_steps() {
    // Arrange
    let frame = puzzle_frame(brim_status());

    // Act
    let scalars = (
        frame.peak_held_count(),
        frame.brim_count(),
        frame.brim_step(),
        frame.decided_step(),
    );

    // Assert
    assert_eq!(scalars, (Some(811), Some(810), Some(262), Some(262)));
}

#[test]
fn set_puzzle_status_accepts_a_peak_above_the_particle_count() {
    // Arrange
    let status = brim_status();

    // Act
    let result = attach(status);

    // Assert
    assert_eq!(result, Ok(()));
    assert!(status.peak_held_count > u32::try_from(PARTICLES).expect("small count"));
}

#[test]
fn a_won_run_at_the_limit_without_brim_is_decided_at_the_limit() {
    // Arrange
    let status = PuzzleStatus {
        observed_step: 600,
        maybe_decided_step: Some(600),
        ..won_status(5)
    };

    // Act
    let frame = puzzle_frame(status);

    // Assert
    assert_eq!((frame.brim_step(), frame.decided_step()), (None, Some(600)));
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
fn set_puzzle_status_rejects_star_counts_inconsistent_with_the_peak() {
    // Arrange
    let running = status(Outcome::Running, 5);
    let won = won_status(5);
    let invalid = [
        (
            "brim <= target",
            PuzzleStatus {
                brim_count: 3,
                ..running
            },
        ),
        (
            "brim > full",
            PuzzleStatus {
                brim_count: 5,
                ..running
            },
        ),
        (
            "won below target",
            PuzzleStatus {
                peak_held_count: 2,
                ..won
            },
        ),
        (
            "running at target",
            PuzzleStatus {
                peak_held_count: 3,
                ..running
            },
        ),
        (
            "brim step below brim",
            PuzzleStatus {
                maybe_brim_step: Some(5),
                maybe_decided_step: Some(5),
                ..won
            },
        ),
        (
            "brim absent at brim",
            PuzzleStatus {
                peak_held_count: 4,
                ..won
            },
        ),
        (
            "brim step while running",
            PuzzleStatus {
                maybe_brim_step: Some(5),
                ..running
            },
        ),
    ];

    // Act
    let results = invalid.map(|(label, candidate)| (label, attach(candidate)));

    // Assert
    for (label, result) in results {
        assert_eq!(result, Err(FrameError::InvalidDiagnostic), "{label}");
    }
}

#[test]
fn set_puzzle_status_rejects_star_steps_out_of_order() {
    // Arrange
    let running = status(Outcome::Running, 5);
    let won = won_status(5);
    let brim = brim_status();
    let invalid = [
        (
            "brim step before the win",
            PuzzleStatus {
                maybe_brim_step: Some(200),
                maybe_decided_step: Some(200),
                ..brim
            },
        ),
        (
            "brim step after observed",
            PuzzleStatus {
                maybe_brim_step: Some(263),
                maybe_decided_step: Some(263),
                ..brim
            },
        ),
        (
            "decided off brim step",
            PuzzleStatus {
                maybe_decided_step: Some(263),
                ..brim
            },
        ),
        (
            "decided while running",
            PuzzleStatus {
                maybe_decided_step: Some(5),
                ..running
            },
        ),
        (
            "won undecided at limit",
            PuzzleStatus {
                observed_step: 600,
                ..won
            },
        ),
        (
            "lost undecided",
            PuzzleStatus {
                maybe_decided_step: None,
                ..lost_status()
            },
        ),
        (
            "lost before limit",
            PuzzleStatus {
                outcome: Outcome::Lost { step: 500 },
                maybe_decided_step: Some(500),
                ..lost_status()
            },
        ),
    ];

    // Act
    let results = invalid.map(|(label, candidate)| (label, attach(candidate)));

    // Assert
    for (label, result) in results {
        assert_eq!(result, Err(FrameError::InvalidDiagnostic), "{label}");
    }
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

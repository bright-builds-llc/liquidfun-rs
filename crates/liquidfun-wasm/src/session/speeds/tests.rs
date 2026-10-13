//! Speed lane tests (Phase 43 D-10, D-11).
//!
//! The read-only proofs compare two runs in the same build profile, never
//! recorded literals, because native debug diverges from release (Phase 37).

use liquidfun::math::Vec2;

use super::particle_speeds;
use crate::frame::ProofFrame;
use crate::puzzle::first_pour::REFERENCE_PLANK_POSE;
use crate::puzzle::tools::ToolKind;
use crate::session::{SessionCore, SessionError};

/// Steps compared by the read-only proofs.
const PROOF_STEPS: u32 = 120;

/// Dam Break steps taken before the lane is inspected.
const WARMUP_STEPS: u32 = 60;

/// Goal count, outcome, outcome step, peak held count and decided step.
type StatusRow = (
    Option<u32>,
    Option<u8>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
);

fn dam_break() -> SessionCore {
    crate::build_core("dam-break").expect("dam-break builds")
}

/// First Pour with the reference solution plank (`first-pour-solution.txt`).
fn first_pour_solution() -> SessionCore {
    crate::build_puzzle_core(
        "first-pour",
        &[ToolKind::Plank.code()],
        &REFERENCE_PLANK_POSE,
    )
    .expect("reference plank builds")
}

fn advance(core: &mut SessionCore, steps: u32) {
    for _ in 0..steps {
        core.advance(1).expect("advance(1) succeeds");
    }
}

fn capture_with_speeds(core: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        core.capture_frame_with_speeds(true)
            .expect("frames capture with speeds"),
    )
}

fn status_row(frame: &ProofFrame) -> StatusRow {
    (
        frame.goal_count(),
        frame.outcome(),
        frame.outcome_step(),
        frame.peak_held_count(),
        frame.decided_step(),
    )
}

/// Positions as bits plus the status row after each single step, capturing
/// with or without the speed lane.
fn state_trace(mut core: SessionCore, with_speeds: bool) -> Vec<(Vec<u32>, StatusRow)> {
    (0..PROOF_STEPS)
        .map(|_| {
            core.advance(1).expect("advance(1) succeeds");
            let frame_data = if with_speeds {
                core.capture_frame_with_speeds(true)
            } else {
                core.capture_frame()
            };
            let frame = ProofFrame::from(frame_data.expect("frames capture"));
            let bits = frame
                .particle_positions()
                .iter()
                .map(|value| value.to_bits())
                .collect();
            (bits, status_row(&frame))
        })
        .collect()
}

#[test]
fn particle_speeds_of_no_velocities_is_empty() {
    // Arrange / Act
    let speeds = particle_speeds(&[]);

    // Assert
    assert_eq!(speeds, Ok(Vec::new()));
}

#[test]
fn particle_speeds_are_velocity_lengths() {
    // Arrange
    let velocities = [Vec2::new(3.0, 4.0), Vec2::new(0.0, 0.0)];

    // Act
    let speeds = particle_speeds(&velocities);

    // Assert
    assert_eq!(speeds, Ok(vec![5.0, 0.0]));
}

#[test]
fn particle_speeds_reject_non_finite_velocities() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        // Arrange
        let velocities = [Vec2::new(1.0, 0.0), Vec2::new(bad, 0.0)];

        // Act
        let speeds = particle_speeds(&velocities);

        // Assert
        assert_eq!(speeds, Err(SessionError::FrameCaptureFailed), "{bad}");
    }
}

#[test]
fn speed_lane_has_one_finite_non_negative_speed_per_particle() {
    // Arrange
    let mut core = dam_break();
    advance(&mut core, WARMUP_STEPS);

    // Act
    let frame = capture_with_speeds(&core);

    // Assert
    let lane = frame.particle_speeds().expect("lane was requested");
    assert_eq!(lane.len(), frame.particle_count());
    assert!(lane.iter().all(|speed| speed.is_finite() && *speed >= 0.0));
}

#[test]
fn speed_lane_maximum_equals_max_speed_bit_for_bit() {
    // Arrange
    let mut core = dam_break();
    advance(&mut core, WARMUP_STEPS);

    // Act
    let frame = capture_with_speeds(&core);

    // Assert
    let lane = frame.particle_speeds().expect("lane was requested");
    let lane_max = lane.iter().copied().fold(0.0_f32, f32::max);
    assert!(frame.max_speed() > 0.0, "dam-break is moving after warmup");
    assert_eq!(lane_max.to_bits(), frame.max_speed().to_bits());
}

#[test]
fn captures_without_the_flag_have_no_speed_lane() {
    // Arrange
    let mut core = dam_break();
    advance(&mut core, WARMUP_STEPS);

    // Act
    let default_frame = ProofFrame::from(core.capture_frame().expect("frames capture"));
    let flag_off_frame = ProofFrame::from(
        core.capture_frame_with_speeds(false)
            .expect("frames capture"),
    );

    // Assert
    assert_eq!(default_frame.particle_speeds(), None);
    assert_eq!(flag_off_frame.particle_speeds(), None);
}

#[test]
fn speed_capture_leaves_scene_stepping_unchanged() {
    // Arrange
    let with_speeds_core = dam_break();
    let without_speeds_core = dam_break();

    // Act
    let with_speeds = state_trace(with_speeds_core, true);
    let without_speeds = state_trace(without_speeds_core, false);

    // Assert
    assert_eq!(with_speeds, without_speeds);
}

#[test]
fn speed_capture_leaves_puzzle_stepping_and_status_unchanged() {
    // Arrange
    let with_speeds_core = first_pour_solution();
    let without_speeds_core = first_pour_solution();

    // Act
    let with_speeds = state_trace(with_speeds_core, true);
    let without_speeds = state_trace(without_speeds_core, false);

    // Assert
    assert!(
        with_speeds.iter().all(|(_bits, row)| row.0.is_some()),
        "puzzle frames carry a status"
    );
    assert_eq!(with_speeds, without_speeds);
}

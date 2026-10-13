//! Sound aggregate tests (Phase 42 D-07, D-12).
//!
//! Determinism tests compare two runs within the same build profile, never
//! recorded literals, because native debug diverges from release (Phase 37).

use super::{SoundSignals, strongest_finite_impulse};
use crate::frame::{FrameData, FrameError, ProofFrame};
use crate::puzzle::LevelId;
use crate::puzzle::replay::fixture::{lanes, parse_fixture};
use crate::session::SessionCore;

/// Steps compared by the determinism and read-only proofs.
const PROOF_STEPS: u32 = 300;

/// Steps the probe keeps printing after the outcome latches.
const PROBE_TAIL_STEPS: u32 = 120;

/// Every checked-in fixture by file name.
const FIXTURES: [(&str, &str); 14] = [
    (
        "first-pour-solution.txt",
        include_str!("../fixtures/first-pour-solution.txt"),
    ),
    (
        "first-pour-empty.txt",
        include_str!("../fixtures/first-pour-empty.txt"),
    ),
    (
        "thick-syrup-solution.txt",
        include_str!("../fixtures/thick-syrup-solution.txt"),
    ),
    (
        "thick-syrup-empty.txt",
        include_str!("../fixtures/thick-syrup-empty.txt"),
    ),
    (
        "jelly-pad-solution.txt",
        include_str!("../fixtures/jelly-pad-solution.txt"),
    ),
    (
        "jelly-pad-empty.txt",
        include_str!("../fixtures/jelly-pad-empty.txt"),
    ),
    (
        "float-up-solution.txt",
        include_str!("../fixtures/float-up-solution.txt"),
    ),
    (
        "float-up-empty.txt",
        include_str!("../fixtures/float-up-empty.txt"),
    ),
    (
        "timed-gate-solution.txt",
        include_str!("../fixtures/timed-gate-solution.txt"),
    ),
    (
        "timed-gate-empty.txt",
        include_str!("../fixtures/timed-gate-empty.txt"),
    ),
    (
        "timed-gate-early.txt",
        include_str!("../fixtures/timed-gate-early.txt"),
    ),
    (
        "timed-gate-late.txt",
        include_str!("../fixtures/timed-gate-late.txt"),
    ),
    (
        "grand-finale-solution.txt",
        include_str!("../fixtures/grand-finale-solution.txt"),
    ),
    (
        "grand-finale-empty.txt",
        include_str!("../fixtures/grand-finale-empty.txt"),
    ),
];

/// The six reference-solution fixtures.
fn solution_fixtures() -> impl Iterator<Item = (&'static str, &'static str)> {
    FIXTURES
        .into_iter()
        .filter(|(file, _text)| file.ends_with("-solution.txt"))
}

fn fixture_text(file: &str) -> &'static str {
    FIXTURES
        .into_iter()
        .find(|(name, _text)| *name == file)
        .map(|(_name, text)| text)
        .expect("fixture is in the table")
}

/// A fresh puzzle session built from a fixture's level and placements.
fn fixture_core(text: &str) -> SessionCore {
    let fixture = parse_fixture(text).expect("checked-in fixture parses");
    let (kinds, poses) = lanes(&fixture);
    crate::build_puzzle_core(fixture.level.as_str(), &kinds, &poses)
        .expect("fixture level and placements build")
}

fn capture(core: &SessionCore) -> ProofFrame {
    ProofFrame::from(core.capture_frame().expect("puzzle frames capture"))
}

/// The sound scalars on a frame, with the impulse as raw bits.
type SoundRow = (Option<u32>, Option<u32>, Option<u32>);

fn sound_row(frame: &ProofFrame) -> SoundRow {
    (
        frame.splash_contacts(),
        frame.impact_count(),
        frame.impact_impulse().map(f32::to_bits),
    )
}

/// Goal count, outcome, outcome step, peak held count and decided step.
type StatusRow = (
    Option<u32>,
    Option<u8>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
);

fn status_row(frame: &ProofFrame) -> StatusRow {
    (
        frame.goal_count(),
        frame.outcome(),
        frame.outcome_step(),
        frame.peak_held_count(),
        frame.decided_step(),
    )
}

/// Sound scalars after each of the first `steps` single steps.
fn sound_trace(text: &str, steps: u32) -> Vec<SoundSignals> {
    let mut core = fixture_core(text);
    (0..steps)
        .map(|_| {
            core.advance(1).expect("advance(1) succeeds");
            signals_of(&capture(&core))
        })
        .collect()
}

fn signals_of(frame: &ProofFrame) -> SoundSignals {
    SoundSignals {
        splash_contacts: frame
            .splash_contacts()
            .expect("puzzle frames report splash"),
        impact_count: frame.impact_count().expect("puzzle frames report impacts"),
        impact_impulse: frame
            .impact_impulse()
            .expect("puzzle frames report an impulse"),
    }
}

/// Positions as bits plus the status row after each of `steps` single steps.
fn state_trace(mut core: SessionCore, steps: u32) -> Vec<(Vec<u32>, StatusRow)> {
    (0..steps)
        .map(|_| {
            core.advance(1).expect("advance(1) succeeds");
            let frame = capture(&core);
            let bits = frame
                .particle_positions()
                .iter()
                .map(|value| value.to_bits())
                .collect();
            (bits, status_row(&frame))
        })
        .collect()
}

/// A puzzle frame at step 0, which already carries a status and sound.
fn puzzle_frame_data() -> FrameData {
    fixture_core(fixture_text("first-pour-solution.txt"))
        .capture_frame()
        .expect("puzzle frames capture")
}

fn signals(splash_contacts: u32, impact_count: u32, impact_impulse: f32) -> SoundSignals {
    SoundSignals {
        splash_contacts,
        impact_count,
        impact_impulse,
    }
}

#[test]
fn puzzle_frame_reports_zero_sound_at_step_zero() {
    // Arrange
    let core = fixture_core(fixture_text("first-pour-solution.txt"));

    // Act
    let frame = capture(&core);

    // Assert
    assert_eq!(
        (
            frame.splash_contacts(),
            frame.impact_count(),
            frame.impact_impulse()
        ),
        (Some(0), Some(0), Some(0.0))
    );
}

#[test]
fn first_step_reports_spawn_contacts_as_a_silent_baseline() {
    for (file, text) in FIXTURES {
        // Arrange
        let steps = 1;

        // Act
        let trace = sound_trace(text, steps);

        // Assert
        assert_eq!(trace, vec![SoundSignals::default()], "{file} first step");
    }
}

#[test]
fn scene_frames_report_no_sound_before_and_after_a_step() {
    // Arrange
    let mut core = crate::build_core("dam-break").expect("dam-break builds");
    let before = sound_row(&capture(&core));

    // Act
    core.advance(1).expect("advance(1) succeeds");
    let after = sound_row(&capture(&core));

    // Assert
    assert_eq!((before, after), ((None, None, None), (None, None, None)));
}

#[test]
fn solution_sound_scalars_repeat_across_runs() {
    for (file, text) in solution_fixtures() {
        // Arrange
        let first = sound_trace(text, PROOF_STEPS);

        // Act
        let second = sound_trace(text, PROOF_STEPS);

        // Assert
        let bits = |trace: &[SoundSignals]| -> Vec<(u32, u32, u32)> {
            trace
                .iter()
                .map(|row| {
                    (
                        row.splash_contacts,
                        row.impact_count,
                        row.impact_impulse.to_bits(),
                    )
                })
                .collect()
        };
        assert_eq!(bits(&first), bits(&second), "{file} sound scalars");
    }
}

#[test]
fn removing_the_tally_leaves_positions_and_status_bit_identical() {
    for (file, text) in solution_fixtures() {
        // Arrange
        let with_tally = fixture_core(text);
        let without_tally = fixture_core(text).without_sound_tally();

        // Act
        let observed = state_trace(with_tally, PROOF_STEPS);
        let unobserved = state_trace(without_tally, PROOF_STEPS);

        // Assert
        assert_eq!(observed.len(), unobserved.len(), "{file} step count");
        for (step, (left, right)) in observed.iter().zip(&unobserved).enumerate() {
            assert!(left == right, "{file} diverged after step {}", step + 1);
        }
    }
}

#[test]
fn first_pour_solution_splashes() {
    // Arrange
    let text = fixture_text("first-pour-solution.txt");

    // Act
    let trace = sound_trace(text, PROOF_STEPS);

    // Assert
    assert!(trace.iter().any(|row| row.splash_contacts > 0));
}

#[test]
fn float_up_solution_reports_a_rigid_impact_with_impulse() {
    // Arrange
    let text = fixture_text("float-up-solution.txt");
    let info =
        crate::puzzle::PuzzleLevelInfo::from_spec(crate::puzzle::level_spec(LevelId::FloatUp));

    // Act
    let trace = sound_trace(text, info.step_limit() - 1);

    // Assert
    assert!(
        trace
            .iter()
            .any(|row| row.impact_count > 0 && row.impact_impulse > 0.0),
        "float-up reports no rigid impact with a solved impulse"
    );
}

#[test]
fn impulse_never_appears_without_a_counted_impact() {
    for (file, text) in solution_fixtures() {
        // Arrange / Act
        let trace = sound_trace(text, PROOF_STEPS);

        // Assert
        let maybe_bad = trace
            .iter()
            .position(|row| row.impact_impulse > 0.0 && row.impact_count == 0);
        assert_eq!(maybe_bad, None, "{file} impulse without an impact");
    }
}

#[test]
fn set_puzzle_sound_accepts_consistent_signals_on_a_puzzle_frame() {
    // Arrange
    let mut frame = puzzle_frame_data();

    // Act
    let result = frame.set_puzzle_sound(signals(3, 1, 0.25));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn set_puzzle_sound_rejects_a_frame_without_puzzle_status() {
    // Arrange
    let mut frame = crate::build_core("dam-break")
        .expect("dam-break builds")
        .capture_frame()
        .expect("scene frames capture");

    // Act
    let result = frame.set_puzzle_sound(signals(3, 1, 0.25));

    // Assert
    assert_eq!(result, Err(FrameError::InvalidDiagnostic));
}

#[test]
fn set_puzzle_sound_rejects_non_finite_or_negative_impulse() {
    for impulse in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.25] {
        // Arrange
        let mut frame = puzzle_frame_data();

        // Act
        let result = frame.set_puzzle_sound(signals(3, 1, impulse));

        // Assert
        assert_eq!(result, Err(FrameError::InvalidDiagnostic), "{impulse}");
    }
}

#[test]
fn set_puzzle_sound_rejects_impulse_without_an_impact() {
    // Arrange
    let mut frame = puzzle_frame_data();

    // Act
    let result = frame.set_puzzle_sound(signals(3, 0, 0.5));

    // Assert
    assert_eq!(result, Err(FrameError::InvalidDiagnostic));
}

#[test]
fn strongest_finite_impulse_drops_non_finite_impulses() {
    // Arrange
    let impulses = [0.25, f32::INFINITY, f32::NAN, 0.5, f32::NEG_INFINITY];

    // Act
    let strongest = strongest_finite_impulse(impulses);

    // Assert
    assert_eq!(strongest.to_bits(), 0.5_f32.to_bits());
}

#[test]
fn strongest_finite_impulse_is_zero_when_every_impulse_is_non_finite() {
    // Arrange
    let impulses = [f32::INFINITY, f32::NAN];

    // Act
    let strongest = strongest_finite_impulse(impulses);

    // Assert
    assert_eq!(strongest.to_bits(), 0.0_f32.to_bits());
}

/// Nearest-rank 90th percentile of `values`; 0 for none.
fn p90<T: Copy + Default + PartialOrd>(mut values: Vec<T>) -> T {
    values.sort_by(|left, right| {
        left.partial_cmp(right)
            .expect("probe values are comparable")
    });
    let rank = (values.len() * 9).div_ceil(10);
    rank.checked_sub(1)
        .and_then(|index| values.get(index).copied())
        .unwrap_or_default()
}

/// Prints one fixture's per-step rows and its summary line.
fn print_probe(file: &str, text: &str) {
    let mut core = fixture_core(text);
    let limit = core.level_info().expect("puzzle level info").step_limit();
    println!("# {file}");
    let mut rows = Vec::new();
    loop {
        core.advance(1).expect("advance(1) succeeds");
        let frame = capture(&core);
        let row = signals_of(&frame);
        let step = core.step_index();
        println!(
            "{step} {} {} {:.6} {:.4} {}",
            row.splash_contacts,
            row.impact_count,
            row.impact_impulse,
            frame.max_speed(),
            frame
                .goal_count()
                .expect("puzzle frames report a goal count")
        );
        rows.push(row);
        let tail_done = frame
            .outcome_step()
            .is_some_and(|latched| step >= latched + PROBE_TAIL_STEPS);
        if step >= limit || tail_done {
            break;
        }
    }
    let splashes = rows
        .iter()
        .map(|row| row.splash_contacts)
        .collect::<Vec<_>>();
    let impact_impulses = rows
        .iter()
        .filter(|row| row.impact_count > 0)
        .map(|row| row.impact_impulse)
        .collect::<Vec<_>>();
    println!(
        "## {file} splash_max={} splash_p90={} impact_steps={} impulse_max={:.6} impulse_p90={:.6}",
        splashes.iter().copied().max().unwrap_or(0),
        p90(splashes),
        impact_impulses.len(),
        impact_impulses.iter().copied().fold(0.0_f32, f32::max),
        p90(impact_impulses)
    );
}

#[test]
#[ignore = "prints per-step sound scalars for tuning"]
fn print_sound_probe() {
    println!(
        "# columns: step splash_contacts impact_count impact_impulse max_speed goal_count; splash_p90 over all steps, impulse_p90 over steps with impact_count > 0"
    );
    for (file, text) in FIXTURES {
        print_probe(file, text);
    }
}

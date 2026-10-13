//! Session-level puzzle tests: puzzle sources build through `SessionCore`,
//! reject scene-only input, and count steps with the session (D-10, D-14,
//! D-15, D-23). Phase 39 adds permuted-placement fingerprints (D-27) and the
//! peak held, Brim and decision steps of real runs (D-07 to D-10).

use liquidfun::math::Vec2;

use crate::frame::ProofFrame;
use crate::puzzle::first_pour::REFERENCE_PLANK_POSE;
use crate::puzzle::goal::{
    MARGIN_WIN_SLACK_STEPS, Outcome, PuzzleStatus, fill_margin_count, maybe_margin_win_step,
};
use crate::puzzle::tools::{Placements, ToolKind};
use crate::puzzle::{LevelId, level_spec};
use crate::scene_spot::frame_hash::frame_hash;
use crate::session::{SessionCore, SessionError};

fn first_pour(placements: Placements) -> SessionCore {
    SessionCore::create_puzzle(LevelId::FirstPour, placements).expect("first-pour session builds")
}

/// Advances one fixed step and returns the goal count after it.
fn step_and_count(core: &mut SessionCore) -> (Outcome, u32) {
    core.advance(1).expect("puzzle step advances");
    let status = core
        .maybe_puzzle_status()
        .expect("puzzle sessions report status");
    (status.outcome, status.goal_count)
}

/// Steps one fixed step at a time until the outcome latches. `counts[i]` is
/// the goal count after step `i + 1`.
fn run_until_latched(core: &mut SessionCore) -> (Outcome, Vec<u32>) {
    let step_limit = level_spec(LevelId::FirstPour).step_limit;
    let mut counts = Vec::new();
    loop {
        assert!(
            core.step_index() <= step_limit + 1,
            "outcome did not latch by step {}",
            core.step_index()
        );
        let (outcome, count) = step_and_count(core);
        counts.push(count);
        if outcome != Outcome::Running {
            return (outcome, counts);
        }
    }
}

fn gravity(core: &SessionCore) -> Vec2 {
    core.read_particles(|world, _system| world.gravity())
}

#[test]
fn create_puzzle_builds_first_pour_at_step_zero() {
    // Arrange
    let placements = Placements::empty();

    // Act
    let core = first_pour(placements);

    // Assert
    assert_eq!(core.step_index(), 0);
    assert!((1500..=3000).contains(&core.particle_count()));
    let status = core
        .maybe_puzzle_status()
        .expect("puzzle sessions report status");
    assert_eq!(status.outcome, Outcome::Running);
    assert_eq!(status.goal_count, 0);
    assert_eq!(status.observed_step, 0);
}

#[test]
fn puzzle_sessions_reject_controls_and_gravity_presets() {
    // Arrange
    let mut core = first_pour(Placements::empty());
    let particle_count = core.particle_count();

    // Act
    let gravity_preset = core.apply_control("gravity", "10");
    let scene_control = core.apply_control("water-amount", "small");

    // Assert
    assert_eq!(gravity_preset, Err(SessionError::UnknownControl));
    assert_eq!(scene_control, Err(SessionError::UnknownControl));
    assert_eq!(core.step_index(), 0);
    assert_eq!(core.particle_count(), particle_count);
}

#[test]
fn puzzle_sessions_reject_actions_and_pointer_input() {
    // Arrange
    let mut core = first_pour(Placements::empty());

    // Act
    let action = core.apply_action("drop");
    let pointer = core.apply_pointer("down", 0.0, 0.0);

    // Assert
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Err(SessionError::UnknownControl));
}

#[test]
fn puzzle_sessions_reject_live_gravity() {
    // Arrange
    let mut core = first_pour(Placements::empty());

    // Act
    let result = core.set_gravity(3.0, -9.0);

    // Assert
    assert_eq!(result, Err(SessionError::UnknownControl));
    assert_eq!(gravity(&core), Vec2::new(0.0, -10.0));
}

#[test]
fn restoring_authored_gravity_is_a_no_op_for_puzzles() {
    // Arrange
    let mut core = first_pour(Placements::empty());

    // Act
    let result = core.restore_authored_gravity();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(gravity(&core), Vec2::new(0.0, -10.0));
}

#[test]
fn puzzle_status_step_matches_session_step_index_across_batches() {
    // Arrange
    let mut core = first_pour(Placements::empty());

    // Act
    for _ in 0..3 {
        core.advance(4).expect("puzzle batch advances");
    }

    // Assert
    let status = core
        .maybe_puzzle_status()
        .expect("puzzle sessions report status");
    assert_eq!(core.step_index(), 12);
    assert_eq!(status.observed_step, 12);
}

#[test]
#[ignore = "phase 37 tuning probe; Plan 08 fixture replays cover this in the default suite"]
fn first_pour_without_placements_loses_at_the_step_limit() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let mut core = first_pour(Placements::empty());

    // Act
    let (outcome, counts) = run_until_latched(&mut core);

    // Assert
    assert_eq!(
        outcome,
        Outcome::Lost {
            step: spec.step_limit
        }
    );
    assert_eq!(core.step_index(), spec.step_limit);
    let max_count = counts.iter().copied().max().unwrap_or(0);
    let ceiling = spec.target_count - fill_margin_count(spec.full_count);
    assert!(
        max_count <= ceiling,
        "empty run peaked at {max_count}, above {ceiling}"
    );
}

#[test]
#[ignore = "phase 37 tuning probe; Plan 08 fixture replays cover this in the default suite"]
fn first_pour_reference_plank_wins_with_margin_and_stays_won() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let placements = Placements::parse(
        &[ToolKind::Plank.code()],
        &REFERENCE_PLANK_POSE,
        spec.tool_budget,
        spec.build_area,
    )
    .expect("reference plank pose parses");
    let mut core = first_pour(placements);
    let margin_deadline = spec.step_limit - MARGIN_WIN_SLACK_STEPS;

    // Act
    let (outcome, mut counts) = run_until_latched(&mut core);
    let latch_step = core.step_index();
    while core.step_index() < margin_deadline {
        counts.push(step_and_count(&mut core).1);
    }
    core.advance(4).expect("stepping continues after a win");
    core.advance(4).expect("stepping continues after a win");

    // Assert
    let Outcome::Won { step } = outcome else {
        panic!("reference plank did not win: {outcome:?}");
    };
    assert_eq!(step, latch_step);
    assert!(step <= margin_deadline, "win at step {step} is too late");
    let maybe_margin_step = maybe_margin_win_step(
        &counts,
        spec.target_count,
        spec.full_count,
        spec.settle_steps,
        spec.step_limit,
    );
    assert!(
        maybe_margin_step.is_some(),
        "the raised target {} never held for {} steps by step {margin_deadline}",
        spec.target_count + fill_margin_count(spec.full_count),
        spec.settle_steps
    );
    let status = core
        .maybe_puzzle_status()
        .expect("puzzle sessions report status");
    assert_eq!(status.outcome, Outcome::Won { step });
}

/// Builds first-pour through the wasm helper from raw placement lanes.
fn first_pour_from_lanes(kinds: &[u8], poses: &[i32]) -> SessionCore {
    crate::build_puzzle_core("first-pour", kinds, poses).expect("first-pour lanes build")
}

fn reference_core() -> SessionCore {
    first_pour_from_lanes(&[ToolKind::Plank.code()], &REFERENCE_PLANK_POSE)
}

/// Steps one fixed step at a time through `last_step`; `statuses[i]` is the
/// status after step `i + 1`.
fn statuses_through(core: &mut SessionCore, last_step: u32) -> Vec<PuzzleStatus> {
    (1..=last_step)
        .map(|_| {
            core.advance(1).expect("puzzle step advances");
            core.maybe_puzzle_status()
                .expect("puzzle sessions report status")
        })
        .collect()
}

/// Peak held count after `step` (1-based) of `statuses`.
fn peak_after(statuses: &[PuzzleStatus], step: u32) -> Option<u32> {
    let index = usize::try_from(step.checked_sub(1)?).ok()?;
    statuses.get(index).map(|status| status.peak_held_count)
}

fn is_record_mode() -> bool {
    std::env::var_os("LIQUIDFUN_PUZZLE_RECORD").is_some_and(|value| value == "1")
}

fn fingerprint_after(kinds: &[u8], poses: &[i32], steps: u32) -> u64 {
    let mut core = first_pour_from_lanes(kinds, poses);
    for _ in 0..steps {
        core.advance(1).expect("puzzle step advances");
    }
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    frame_hash(&frame)
}

#[test]
fn permuted_placements_give_identical_fingerprints() {
    // Arrange
    let kinds = [ToolKind::Plank.code(); 2];
    let in_order = [8, 64, 23, 30, 84, 0];
    let permuted = [30, 84, 0, 8, 64, 23];

    // Act
    let first = fingerprint_after(&kinds, &in_order, 120);
    let second = fingerprint_after(&kinds, &permuted, 120);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn reference_peak_clears_brim_with_margin() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let mut core = reference_core();
    let deadline = spec.step_limit - MARGIN_WIN_SLACK_STEPS;

    // Act
    let statuses = statuses_through(&mut core, spec.step_limit);

    // Assert
    let counts: Vec<u32> = statuses.iter().map(|status| status.goal_count).collect();
    let maybe_last = statuses.last().copied();
    let maybe_brim_step = maybe_last.and_then(|status| status.maybe_brim_step);
    let maybe_peak_at_deadline = peak_after(&statuses, deadline);
    if is_record_mode() {
        eprintln!(
            "first-pour reference peak held: @201 {:?} @262 {:?} @{deadline} {maybe_peak_at_deadline:?} @{} {:?}; brim {} latched at {maybe_brim_step:?}; decided {:?}",
            peak_after(&statuses, 201),
            peak_after(&statuses, 262),
            spec.step_limit,
            peak_after(&statuses, spec.step_limit),
            spec.brim_count,
            maybe_last.and_then(|status| status.maybe_decided_step),
        );
    }
    let maybe_margin_step = maybe_margin_win_step(
        &counts,
        spec.brim_count,
        spec.full_count,
        spec.settle_steps,
        spec.step_limit,
    );
    assert!(
        maybe_margin_step.is_some(),
        "peak held {maybe_peak_at_deadline:?} at step {deadline} (Brim at {maybe_brim_step:?}) does not clear brim {} by {}",
        spec.brim_count,
        fill_margin_count(spec.full_count)
    );
}

#[test]
fn reference_run_decides_at_the_brim_step() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let mut core = reference_core();

    // Act
    let maybe_decided = (1..=spec.step_limit).find_map(|_| {
        core.advance(1).expect("puzzle step advances");
        core.maybe_puzzle_status()
            .filter(|status| status.maybe_decided_step.is_some())
    });

    // Assert
    let status = maybe_decided.expect("the reference run is decided by the step limit");
    assert!(matches!(status.outcome, Outcome::Won { .. }));
    assert!(status.maybe_brim_step.is_some(), "no Brim: {status:?}");
    assert_eq!(status.maybe_decided_step, status.maybe_brim_step);
    assert_eq!(status.maybe_decided_step, Some(core.step_index()));
}

#[test]
fn empty_run_has_no_peak_and_decides_at_the_limit() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let mut core = first_pour(Placements::empty());

    // Act
    let statuses = statuses_through(&mut core, spec.step_limit);

    // Assert
    let last = statuses.last().copied().expect("the run stepped");
    assert_eq!(
        (
            last.peak_held_count,
            last.maybe_brim_step,
            last.maybe_decided_step,
            last.outcome
        ),
        (
            0,
            None,
            Some(spec.step_limit),
            Outcome::Lost {
                step: spec.step_limit
            }
        )
    );
}

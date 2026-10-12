//! Session-level puzzle tests: puzzle sources build through `SessionCore`,
//! reject scene-only input, and count steps with the session (D-10, D-14,
//! D-15, D-23).

use liquidfun::math::Vec2;

use crate::puzzle::LevelId;
use crate::puzzle::goal::Outcome;
use crate::puzzle::tools::Placements;
use crate::session::{SessionCore, SessionError};

fn first_pour(placements: Placements) -> SessionCore {
    SessionCore::create_puzzle(LevelId::FirstPour, placements).expect("first-pour session builds")
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

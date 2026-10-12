//! Boundary tests for `newPuzzle`, `levelInfo` and the frame puzzle scalars
//! (D-16, D-17, D-24). Native tests cannot build `JsError`, so they call the
//! helpers the wasm methods wrap.

use wasm_bindgen::JsError;

use crate::frame::ProofFrame;
use crate::puzzle::first_pour::REFERENCE_PLANK_POSE;
use crate::puzzle::tools::{GRID_METERS, PlacementError};
use crate::puzzle::{LevelId, PuzzleLevelInfo, level_spec};
use crate::session::{SessionCore, SessionError};
use crate::{ProofSession, build_core, build_puzzle_core};

fn frame(core: &SessionCore) -> ProofFrame {
    ProofFrame::from(core.capture_frame().expect("frame captures"))
}

/// The six frame puzzle scalars in getter order.
type PuzzleScalars = (
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<u8>,
    Option<u32>,
    Option<u32>,
);

fn puzzle_scalars(frame: &ProofFrame) -> PuzzleScalars {
    (
        frame.goal_count(),
        frame.goal_target(),
        frame.goal_full(),
        frame.outcome(),
        frame.outcome_step(),
        frame.step_limit(),
    )
}

fn assert_running_goal_contract(frame: &ProofFrame, info: &PuzzleLevelInfo, step: u32) {
    let particle_count = u32::try_from(frame.particle_count()).expect("particle count fits u32");
    let goal_count = frame
        .goal_count()
        .expect("puzzle frames report a goal count");
    assert!(goal_count <= particle_count);
    assert_eq!(frame.step_index(), step);
    assert_eq!(frame.goal_target(), Some(info.target_count()));
    assert_eq!(frame.goal_full(), Some(info.full_count()));
    assert_eq!(frame.outcome(), Some(0));
    assert_eq!(frame.outcome_step(), None);
    assert_eq!(frame.step_limit(), Some(info.step_limit()));
}

#[test]
fn first_pour_frames_and_level_info_report_the_goal_contract() {
    // Arrange
    let spec = level_spec(LevelId::FirstPour);
    let mut core = build_puzzle_core("first-pour", &[], &[]).expect("empty first-pour builds");

    // Act
    let info = core
        .level_info()
        .expect("puzzle sessions describe their level");
    let mut frames = vec![frame(&core)];
    for _ in 0..10 {
        core.advance(1).expect("puzzle step advances");
        frames.push(frame(&core));
    }

    // Assert
    let interior = spec.cup_interior;
    let expected_interior = [
        interior.min().x,
        interior.min().y,
        interior.max().x,
        interior.max().y,
    ]
    .map(f32::to_bits);
    assert_eq!(info.level_id(), "first-pour");
    assert_eq!(info.title(), "First Pour");
    assert_eq!(info.target_count(), spec.target_count);
    assert_eq!(info.full_count(), spec.full_count);
    assert_eq!(info.settle_steps(), spec.settle_steps);
    assert_eq!(info.step_limit(), spec.step_limit);
    assert_eq!(
        info.cup_interior()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        expected_interior
    );
    assert_eq!(*info.build_area(), spec.build_area.bounds().map(i32::from));
    assert_eq!(info.grid_meters().to_bits(), GRID_METERS.to_bits());
    assert_eq!(info.grid_meters().to_bits(), 0.05_f32.to_bits());
    assert_eq!(*info.tool_kinds(), [1]);
    assert_eq!(*info.tool_counts(), [2]);
    for (step, frame) in (0_u32..).zip(&frames) {
        assert_running_goal_contract(frame, &info, step);
    }
}

#[test]
fn reference_plank_lanes_build_through_the_wasm_helper() {
    // Arrange
    let empty = build_puzzle_core("first-pour", &[], &[]).expect("empty first-pour builds");

    // Act
    let planked = build_puzzle_core("first-pour", &[1], &REFERENCE_PLANK_POSE)
        .expect("reference plank builds");

    // Assert
    assert_eq!(
        frame(&planked).rigid_shape_count(),
        frame(&empty).rigid_shape_count() + 4
    );
}

#[test]
fn scene_frames_carry_no_puzzle_scalars() {
    // Arrange
    let core = build_core("dam-break").expect("Dam Break builds");

    // Act
    let scalars = puzzle_scalars(&frame(&core));

    // Assert
    assert_eq!(scalars, (None, None, None, None, None, None));
}

#[test]
fn level_info_on_a_scene_session_is_not_a_puzzle() {
    // Arrange
    let core = build_core("dam-break").expect("Dam Break builds");

    // Act
    let result = core.level_info();

    // Assert
    assert!(matches!(result, Err(SessionError::NotAPuzzle)));
}

#[test]
fn unknown_and_scene_ids_are_not_levels() {
    // Arrange
    let ids = ["dam-break", "First-Pour"];

    // Act
    let puzzle_results = ids.map(|id| build_puzzle_core(id, &[], &[]).err());
    let scene_result = build_core("first-pour").err();

    // Assert
    assert_eq!(puzzle_results, [Some(SessionError::UnknownLevel); 2]);
    assert_eq!(scene_result, Some(SessionError::UnknownScene));
}

#[test]
fn invalid_placement_lanes_fail_closed() {
    // Arrange
    let cases: [(&[u8], &[i32], PlacementError); 3] = [
        (
            &[1, 1, 1],
            &[8, 64, 23, 30, 84, 0, -20, 44, 0],
            PlacementError::ToolBudgetExceeded,
        ),
        (&[1], &[8, 64, 24], PlacementError::AngleOutOfRange),
        (&[1], &[8, 64], PlacementError::LaneLengthMismatch),
    ];

    for (kinds, poses, expected) in cases {
        // Act
        let result = build_puzzle_core("first-pour", kinds, poses).err();

        // Assert
        assert_eq!(result, Some(SessionError::InvalidPlacement(expected)));
    }
}

#[test]
fn puzzle_error_messages_are_bounded() {
    // Arrange
    let placement = PlacementError::OutsideBuildArea;

    // Act
    let messages = (
        SessionError::UnknownLevel.message(),
        SessionError::NotAPuzzle.message(),
        SessionError::InvalidPlacement(placement).message(),
    );

    // Assert
    assert_eq!(
        messages,
        (
            "Rust/WASM puzzle level id is not allowlisted",
            "Rust/WASM session is not a puzzle",
            placement.message(),
        )
    );
}

#[test]
fn overlap_error_messages_are_bounded_and_echo_no_input() {
    // Arrange
    let overlaps = [
        PlacementError::OverlapsWall,
        PlacementError::OverlapsCupWall,
        PlacementError::InsideCup,
        PlacementError::OverlapsWater,
        PlacementError::OverlapsTool,
        PlacementError::OverlapsNoPlace,
    ];

    // Act
    let messages = overlaps.map(|error| SessionError::InvalidPlacement(error).message());

    // Assert
    for (message, error) in messages.into_iter().zip(overlaps) {
        assert_eq!(message, error.message());
        assert!(
            message.starts_with("Rust/WASM puzzle placement "),
            "{message}"
        );
        assert!(message.len() <= 64, "{message}");
        assert!(
            !message.bytes().any(|byte| byte.is_ascii_digit()),
            "{message}"
        );
    }
}

#[test]
fn new_puzzle_and_level_info_exist_on_proof_session() {
    // Arrange
    type NewPuzzle = fn(String, &[u8], &[i32]) -> Result<ProofSession, JsError>;
    type LevelInfo = fn(&ProofSession) -> Result<PuzzleLevelInfo, JsError>;

    // Act
    let new_puzzle: NewPuzzle = ProofSession::new_puzzle;
    let level_info: LevelInfo = ProofSession::level_info;

    // Assert
    assert_eq!(
        core::mem::size_of_val(&new_puzzle),
        core::mem::size_of::<NewPuzzle>()
    );
    assert_eq!(
        core::mem::size_of_val(&level_info),
        core::mem::size_of::<LevelInfo>()
    );
}

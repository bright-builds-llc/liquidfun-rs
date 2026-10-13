use super::{
    Inspection, OUTLINE_STRIDE, PlacementStatus, inspect, inspect_level, validate_for_build,
};
use crate::build_puzzle_core;
use crate::puzzle::first_pour::{FIRST_POUR, REFERENCE_PLANK_POSE};
use crate::puzzle::spec::LevelSpec;
use crate::puzzle::tools::{GridRect, MAX_PLACEMENTS, PlacementError, pose_placement};
use crate::session::SessionError;

const NO_WALLS: LevelSpec = LevelSpec {
    walls: &[],
    ..FIRST_POUR
};
const NO_TOOLS: LevelSpec = LevelSpec {
    tool_budget: &[],
    ..FIRST_POUR
};
const NO_PLACE_ZONE: LevelSpec = LevelSpec {
    no_place: &[GridRect::new(0, 60, 20, 70)],
    ..FIRST_POUR
};

fn inspect_planks(spec: &LevelSpec, poses: &[i32]) -> Inspection {
    let kinds = vec![1_u8; poses.len() / 3];
    inspect(spec, &kinds, poses).expect("test lanes are well formed")
}

fn statuses_of(spec: &LevelSpec, poses: &[i32]) -> Vec<u8> {
    inspect_planks(spec, poses).statuses
}

fn codes(statuses: &[PlacementStatus]) -> Vec<u8> {
    statuses.iter().map(|status| status.code()).collect()
}

fn plank_outline_bits(pose: [i32; 3]) -> Vec<u32> {
    let placement = pose_placement(1, &pose).expect("test pose describes a plank");
    placement
        .vertices()
        .iter()
        .flat_map(|vertex| [vertex.x.to_bits(), vertex.y.to_bits()])
        .collect()
}

#[test]
fn the_reference_pose_is_valid() {
    // Arrange
    let poses = REFERENCE_PLANK_POSE;

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::Valid]));
}

#[test]
fn a_plank_through_the_reservoir_floor_overlaps_a_wall() {
    // Arrange
    let poses = [-20, 84, 6];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::OverlapsWall]));
}

#[test]
fn a_plank_against_the_backstop_overlaps_the_cup_wall() {
    // Arrange
    let poses = [40, 44, 0];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::OverlapsCupWall]));
}

#[test]
fn a_plank_hanging_into_the_cup_is_inside_the_cup() {
    // Arrange
    let poses = [38, 44, 6];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::InsideCup]));
}

#[test]
fn a_plank_through_the_water_overlaps_the_water() {
    // Arrange
    let poses = [-20, 84, 6];

    // Act
    let statuses = statuses_of(&NO_WALLS, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::OverlapsWater]));
}

#[test]
fn crossed_planks_both_overlap_a_tool() {
    // Arrange
    let poses = [8, 64, 23, 8, 64, 0];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(
        statuses,
        codes(&[PlacementStatus::OverlapsTool, PlacementStatus::OverlapsTool])
    );
}

#[test]
fn an_authored_zone_rejects_a_plank_as_no_place() {
    // Arrange
    let poses = [10, 64, 0];

    // Act
    let (plain, zoned) = (
        statuses_of(&FIRST_POUR, &poses),
        statuses_of(&NO_PLACE_ZONE, &poses),
    );

    // Assert
    assert_eq!(plain, codes(&[PlacementStatus::Valid]));
    assert_eq!(zoned, codes(&[PlacementStatus::OverlapsNoPlace]));
}

#[test]
fn a_centre_past_the_build_area_is_outside_it() {
    // Arrange
    let poses = [41, 64, 0];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::OutsideBuildArea]));
}

#[test]
fn the_third_plank_in_input_order_is_over_budget() {
    // Arrange
    let poses = [8, 64, 23, 30, 84, 0, -20, 44, 0];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(
        statuses,
        codes(&[
            PlacementStatus::Valid,
            PlacementStatus::Valid,
            PlacementStatus::OverBudget,
        ])
    );
}

#[test]
fn angle_index_24_is_out_of_range() {
    // Arrange
    let poses = [8, 64, 24];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::AngleOutOfRange]));
}

#[test]
fn kind_codes_zero_and_two_are_unknown() {
    // Arrange
    let poses = [8, 64, 23, 30, 84, 0];

    // Act
    let inspection = inspect(&FIRST_POUR, &[0, 2], &poses).expect("lanes are well formed");

    // Assert
    assert_eq!(
        inspection.statuses,
        codes(&[PlacementStatus::UnknownKind, PlacementStatus::UnknownKind])
    );
}

#[test]
fn a_kind_missing_from_the_budget_is_not_allowed() {
    // Arrange
    let poses = REFERENCE_PLANK_POSE;

    // Act
    let statuses = statuses_of(&NO_TOOLS, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::KindNotAllowed]));
}

#[test]
fn a_coordinate_beyond_i16_is_out_of_range() {
    // Arrange
    let poses = [40_000, 64, 0];

    // Act
    let statuses = statuses_of(&FIRST_POUR, &poses);

    // Assert
    assert_eq!(statuses, codes(&[PlacementStatus::PositionOutOfRange]));
}

#[test]
fn geometric_statuses_carry_the_baked_outline_bits() {
    // Arrange
    let poses = [[8, 64, 23], [-20, 84, 6], [41, 64, 0]];

    // Act
    let inspection = inspect_planks(&FIRST_POUR, poses.as_flattened());

    // Assert
    let expected: Vec<u32> = poses.into_iter().flat_map(plank_outline_bits).collect();
    let actual: Vec<u32> = inspection
        .outlines
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn poses_without_an_outline_emit_zeros() {
    // Arrange
    let kinds = [0, 1, 1];
    let poses = [8, 64, 23, 8, 64, 24, 40_000, 64, 0];

    // Act
    let inspection = inspect(&FIRST_POUR, &kinds, &poses).expect("lanes are well formed");

    // Assert
    assert_eq!(
        inspection.statuses,
        codes(&[
            PlacementStatus::UnknownKind,
            PlacementStatus::AngleOutOfRange,
            PlacementStatus::PositionOutOfRange,
        ])
    );
    assert_eq!(inspection.outlines, vec![0.0; 3 * OUTLINE_STRIDE]);
}

#[test]
fn a_disallowed_kind_without_an_outline_reports_the_decoding_reason() {
    // Arrange
    let kinds = [1, 1, 0, 1];
    let reference = REFERENCE_PLANK_POSE;
    let poses = [[8, 64, 24], [40_000, 64, 0], reference, reference];
    let non_geometric = [
        PlacementStatus::UnknownKind,
        PlacementStatus::AngleOutOfRange,
        PlacementStatus::PositionOutOfRange,
    ]
    .map(PlacementStatus::code);

    // Act
    let inspection =
        inspect(&NO_TOOLS, &kinds, poses.as_flattened()).expect("lanes are well formed");

    // Assert
    assert_eq!(
        inspection.statuses,
        codes(&[
            PlacementStatus::AngleOutOfRange,
            PlacementStatus::PositionOutOfRange,
            PlacementStatus::UnknownKind,
            PlacementStatus::KindNotAllowed,
        ])
    );
    for (status, outline) in inspection
        .statuses
        .iter()
        .zip(inspection.outlines.chunks_exact(OUTLINE_STRIDE))
    {
        let has_outline = outline.iter().any(|value| *value != 0.0);
        assert_eq!(
            has_outline,
            !non_geometric.contains(status),
            "status {status}"
        );
    }
}

#[test]
fn more_than_sixteen_placements_fail_the_request() {
    // Arrange
    let kinds = [1_u8; MAX_PLACEMENTS + 1];
    let poses = [0_i32; 3 * (MAX_PLACEMENTS + 1)];

    // Act
    let result = inspect(&FIRST_POUR, &kinds, &poses);

    // Assert
    assert_eq!(result, Err(PlacementError::TooManyPlacements));
}

#[test]
fn misaligned_pose_lanes_fail_the_request() {
    // Arrange
    let (kinds, poses) = ([1_u8], [8, 64]);

    // Act
    let result = inspect(&FIRST_POUR, &kinds, &poses);

    // Assert
    assert_eq!(result, Err(PlacementError::LaneLengthMismatch));
}

#[test]
fn build_validation_rejects_crossed_planks_as_a_tool_overlap() {
    // Arrange
    let poses = [8, 64, 23, 8, 64, 0];

    // Act
    let result = validate_for_build(&FIRST_POUR, &[1, 1], &poses);

    // Assert
    assert_eq!(result.err(), Some(PlacementError::OverlapsTool));
}

#[test]
fn first_pour_solution_fixture_placement_is_valid() {
    // Arrange
    let fixture = include_str!("../fixtures/first-pour-solution.txt");
    let pose: Vec<i32> = fixture
        .lines()
        .find_map(|line| line.strip_prefix("placement plank "))
        .expect("solution fixture records a plank")
        .split_whitespace()
        .map(|value| value.parse().expect("fixture pose values are integers"))
        .collect();

    // Act
    let statuses = statuses_of(&FIRST_POUR, &pose);

    // Assert
    assert_eq!(pose, REFERENCE_PLANK_POSE);
    assert_eq!(statuses, codes(&[PlacementStatus::Valid]));
}

#[test]
fn inspector_and_new_puzzle_agree() {
    // Arrange
    let cases: [(&[u8], &[i32]); 16] = [
        (&[], &[]),
        (&[1], &[8, 64, 23]),
        (&[1], &[30, 84, 0]),
        (&[1], &[-20, 44, 0]),
        (&[1], &[10, 64, 0]),
        (&[1], &[-20, 84, 6]),
        (&[1], &[40, 44, 0]),
        (&[1], &[38, 44, 6]),
        (&[1], &[41, 64, 0]),
        (&[1], &[8, 64, 24]),
        (&[0], &[8, 64, 23]),
        (&[2], &[8, 64, 23]),
        (&[1], &[40_000, 64, 0]),
        (&[1, 1], &[8, 64, 23, 30, 84, 0]),
        (&[1, 1], &[8, 64, 23, 8, 64, 0]),
        (&[1, 1, 1], &[8, 64, 23, 30, 84, 0, -20, 44, 0]),
    ];

    for (kinds, poses) in cases {
        // Act
        let inspection = inspect_level("first-pour", kinds, poses).expect("lanes are well formed");
        let built = build_puzzle_core("first-pour", kinds, poses);

        // Assert
        let all_valid = inspection
            .statuses
            .iter()
            .all(|status| *status == PlacementStatus::Valid.code());
        assert_eq!(all_valid, built.is_ok(), "{kinds:?} {poses:?}");
        if let Err(error) = built {
            assert!(
                matches!(error, SessionError::InvalidPlacement(_)),
                "{kinds:?} {poses:?}"
            );
        }
    }
}

#[test]
fn inspect_level_reports_reference_outline() {
    // Arrange
    let poses = REFERENCE_PLANK_POSE;

    // Act
    let inspection = inspect_level("first-pour", &[1], &poses).expect("first-pour inspects");

    // Assert
    let actual: Vec<u32> = inspection
        .outlines
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert_eq!(inspection.statuses, codes(&[PlacementStatus::Valid]));
    assert_eq!(actual, plank_outline_bits(REFERENCE_PLANK_POSE));
}

#[test]
fn inspect_level_rejects_unknown_levels() {
    // Arrange
    let ids = ["dam-break", "First-Pour", ""];

    // Act
    let results = ids.map(|id| inspect_level(id, &[], &[]).err());

    // Assert
    assert_eq!(results, [Some(SessionError::UnknownLevel); 3]);
}

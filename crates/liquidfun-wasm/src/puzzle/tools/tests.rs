use liquidfun::collision::PolygonShape;
use liquidfun::math::Vec2;

use super::{
    AngleIndex, GRID_CELLS_PER_METER, GRID_METERS, GridRect, Placement, PlacementError, Placements,
    ToolAllowance, ToolKind,
};

const BUILD_AREA: GridRect = GridRect::new(-20, 40, 40, 90);
const TWO_PLANKS: [ToolAllowance; 1] = [ToolAllowance {
    kind: ToolKind::Plank,
    count: 2,
}];

fn parse(kinds: &[u8], poses: &[i32]) -> Result<Placements, PlacementError> {
    Placements::parse(kinds, poses, &TWO_PLANKS, BUILD_AREA)
}

fn plank_at(x: i16, y: i16, angle: u8) -> Placement {
    Placement {
        kind: ToolKind::Plank,
        x,
        y,
        angle: AngleIndex::new(angle).expect("test angle index is below 24"),
    }
}

#[test]
fn empty_lanes_parse_to_no_placements() {
    // Arrange
    let kinds: [u8; 0] = [];
    let poses: [i32; 0] = [];

    // Act
    let parsed = parse(&kinds, &poses);

    // Assert
    assert_eq!(parsed, Ok(Placements::empty()));
}

#[test]
fn valid_plank_parses_with_its_grid_pose() {
    // Arrange
    let kinds = [ToolKind::Plank.code()];
    let poses = [12, 64, 23];

    // Act
    let parsed = parse(&kinds, &poses).expect("a valid plank parses");

    // Assert
    let [placement] = parsed.as_slice() else {
        panic!("expected exactly one placement");
    };
    assert_eq!(placement.kind(), ToolKind::Plank);
    assert_eq!(placement.grid(), (12, 64));
    assert_eq!(placement.angle().get(), 23);
}

#[test]
fn rejects_more_than_sixteen_placements() {
    // Arrange
    let kinds = [1_u8; 17];
    let poses = [0_i32; 17 * 3];

    // Act
    let parsed = parse(&kinds, &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::TooManyPlacements));
}

#[test]
fn rejects_a_pose_lane_that_is_not_three_per_kind() {
    // Arrange
    let kinds = [1_u8];
    let short_poses = [12, 64];
    let long_poses = [12, 64, 0, 5];

    // Act
    let short = parse(&kinds, &short_poses);
    let long = parse(&kinds, &long_poses);

    // Assert
    assert_eq!(short, Err(PlacementError::LaneLengthMismatch));
    assert_eq!(long, Err(PlacementError::LaneLengthMismatch));
}

#[test]
fn rejects_a_reserved_fourth_pose_value() {
    // Arrange
    let kinds = [1_u8];
    let poses_with_valve_time = [12, 64, 0, 120];

    // Act
    let parsed = parse(&kinds, &poses_with_valve_time);

    // Assert
    assert_eq!(parsed, Err(PlacementError::LaneLengthMismatch));
}

#[test]
fn rejects_an_unknown_tool_code() {
    // Arrange
    let poses = [12, 64, 0];

    // Act
    let zero = parse(&[0], &poses);
    let two = parse(&[2], &poses);

    // Assert
    assert_eq!(zero, Err(PlacementError::UnknownToolKind));
    assert_eq!(two, Err(PlacementError::UnknownToolKind));
}

#[test]
fn rejects_a_tool_outside_the_level_allowlist() {
    // Arrange
    let no_tools: [ToolAllowance; 0] = [];

    // Act
    let parsed = Placements::parse(&[1], &[12, 64, 0], &no_tools, BUILD_AREA);

    // Assert
    assert_eq!(parsed, Err(PlacementError::ToolNotAllowed));
}

#[test]
fn rejects_a_tool_beyond_its_budget() {
    // Arrange
    let kinds = [1_u8; 3];
    let poses = [0, 50, 0, 10, 50, 0, 20, 50, 0];

    // Act
    let parsed = parse(&kinds, &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::ToolBudgetExceeded));
}

#[test]
fn rejects_angle_indices_outside_zero_to_twenty_three() {
    // Arrange
    let angles = [24, -1, 300];

    for angle in angles {
        // Act
        let parsed = parse(&[1], &[12, 64, angle]);

        // Assert
        assert_eq!(
            parsed,
            Err(PlacementError::AngleOutOfRange),
            "angle {angle}"
        );
    }
}

#[test]
fn rejects_grid_coordinates_outside_i16() {
    // Arrange
    let poses = [40_000, 64, 0];

    // Act
    let parsed = parse(&[1], &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::PositionOutOfRange));
}

#[test]
fn rejects_a_position_outside_the_build_area() {
    // Arrange
    let outside = [[41, 64, 0], [12, 39, 0]];
    let inclusive_edges = [-20, 40, 0, 40, 90, 0];

    // Act
    let rejected = outside.map(|pose| parse(&[1], &pose));
    let accepted = parse(&[1, 1], &inclusive_edges);

    // Assert
    assert_eq!(
        rejected,
        [
            Err(PlacementError::OutsideBuildArea),
            Err(PlacementError::OutsideBuildArea)
        ]
    );
    assert!(accepted.is_ok(), "inclusive build-area edges are accepted");
}

#[test]
fn parse_orders_placements_canonically() {
    // Arrange
    let forward = [20, 50, 0, 10, 60, 3];
    let reversed = [10, 60, 3, 20, 50, 0];

    // Act
    let from_forward = parse(&[1, 1], &forward).expect("forward placements parse");
    let from_reversed = parse(&[1, 1], &reversed).expect("reversed placements parse");

    // Assert
    assert_eq!(from_forward, from_reversed);
    assert_eq!(from_forward.as_slice()[0].grid(), (10, 60));
}

#[test]
fn zero_angle_plank_vertices_are_axis_aligned() {
    // Arrange
    let placement = plank_at(0, 0, 0);

    // Act
    let vertices = placement.vertices();

    // Assert
    assert_eq!(
        vertices,
        [
            Vec2::new(-1.5, -0.125),
            Vec2::new(1.5, -0.125),
            Vec2::new(1.5, 0.125),
            Vec2::new(-1.5, 0.125),
        ]
    );
}

#[test]
fn quarter_turn_plank_vertices_swap_axes() {
    // Arrange
    let placement = plank_at(20, 40, 6);

    // Act
    let vertices = placement.vertices();

    // Assert
    assert_eq!(placement.center(), Vec2::new(1.0, 2.0));
    assert_eq!(
        vertices,
        [
            Vec2::new(1.125, 0.5),
            Vec2::new(1.125, 3.5),
            Vec2::new(0.875, 3.5),
            Vec2::new(0.875, 0.5),
        ]
    );
}

#[test]
fn plank_vertices_build_a_valid_polygon_for_every_angle() {
    // Arrange
    let placements = (0..24).map(|angle| plank_at(12, 64, angle));

    for placement in placements {
        // Act
        let polygon = PolygonShape::new(&placement.vertices());

        // Assert
        assert!(polygon.is_ok(), "angle {}", placement.angle().get());
    }
}

#[test]
fn placement_error_messages_are_bounded_and_distinct() {
    // Arrange
    let errors = [
        PlacementError::TooManyPlacements,
        PlacementError::LaneLengthMismatch,
        PlacementError::UnknownToolKind,
        PlacementError::ToolNotAllowed,
        PlacementError::ToolBudgetExceeded,
        PlacementError::AngleOutOfRange,
        PlacementError::PositionOutOfRange,
        PlacementError::OutsideBuildArea,
    ];

    // Act
    let messages = errors.map(PlacementError::message);

    // Assert
    for (index, message) in messages.iter().enumerate() {
        assert!(
            message.starts_with("Rust/WASM puzzle placement"),
            "{message}"
        );
        assert!(
            messages[index + 1..].iter().all(|other| other != message),
            "{message} is repeated"
        );
    }
}

#[test]
fn tool_kind_codes_and_tokens_round_trip() {
    // Arrange
    let kind = ToolKind::Plank;

    // Act
    let from_code = ToolKind::from_code(kind.code());
    let from_token = ToolKind::from_token(kind.token());

    // Assert
    assert_eq!(from_code, Some(kind));
    assert_eq!(from_token, Some(kind));
    assert_eq!(ToolKind::from_token("valve"), None);
}

#[test]
fn grid_rect_reports_its_inclusive_bounds() {
    // Arrange
    let area = BUILD_AREA;

    // Act
    let bounds = area.bounds();

    // Assert
    assert_eq!(bounds, [-20, 40, 40, 90]);
    assert!(area.contains(-20, 90));
    assert!(!area.contains(-21, 90));
}

#[test]
fn grid_cell_size_matches_cells_per_meter() {
    // Arrange
    let cells = GRID_CELLS_PER_METER;

    // Act
    let meters = GRID_METERS * cells;

    // Assert
    assert_eq!(meters.to_bits(), 1.0_f32.to_bits());
}

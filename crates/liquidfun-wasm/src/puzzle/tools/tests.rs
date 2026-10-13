use liquidfun::collision::PolygonShape;
use liquidfun::math::Vec2;

use super::{
    AngleIndex, GRID_CELLS_PER_METER, GRID_METERS, GridRect, Placement, PlacementError, Placements,
    ToolAllowance, ToolKind,
};
use crate::puzzle::first_pour::FIRST_POUR;
use crate::puzzle::spec::LevelSpec;

const BUILD_AREA: GridRect = GridRect::new(-20, 40, 40, 90);
const TWO_PLANKS_ONE_VALVE: [ToolAllowance; 2] = [
    ToolAllowance {
        kind: ToolKind::Plank,
        count: 2,
    },
    ToolAllowance {
        kind: ToolKind::Valve,
        count: 1,
    },
];
/// First-pour limits (step limit 720, so open ticks 1..=24) with a test budget.
const TOOLS_SPEC: LevelSpec = LevelSpec {
    tool_budget: &TWO_PLANKS_ONE_VALVE,
    build_area: BUILD_AREA,
    ..FIRST_POUR
};

fn parse(kinds: &[u8], poses: &[i32]) -> Result<Placements, PlacementError> {
    Placements::parse(kinds, poses, &TOOLS_SPEC)
}

fn tool_at(kind: ToolKind, x: i16, y: i16, angle: u8) -> Placement {
    Placement {
        kind,
        x,
        y,
        angle: AngleIndex::new(angle).expect("test angle index is below 24"),
        open_tick: 0,
    }
}

fn plank_at(x: i16, y: i16, angle: u8) -> Placement {
    tool_at(ToolKind::Plank, x, y, angle)
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
    let poses = [12, 64, 23, 0];

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
    let poses = [0_i32; 17 * 4];

    // Act
    let parsed = parse(&kinds, &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::TooManyPlacements));
}

#[test]
fn rejects_a_pose_lane_that_is_not_four_per_kind() {
    // Arrange
    let kinds = [1_u8];
    let short_poses = [12, 64];
    let long_poses = [12, 64, 0, 0, 5];

    // Act
    let short = parse(&kinds, &short_poses);
    let long = parse(&kinds, &long_poses);

    // Assert
    assert_eq!(short, Err(PlacementError::LaneLengthMismatch));
    assert_eq!(long, Err(PlacementError::LaneLengthMismatch));
}

#[test]
fn rejects_stride_three_lanes() {
    // Arrange
    let kinds = [1_u8];
    let stride_three_pose = [12, 64, 0];

    // Act
    let parsed = parse(&kinds, &stride_three_pose);

    // Assert
    assert_eq!(parsed, Err(PlacementError::LaneLengthMismatch));
}

#[test]
fn rejects_an_unknown_tool_code() {
    // Arrange
    let poses = [12, 64, 0, 0];

    // Act
    let zero = parse(&[0], &poses);
    let three = parse(&[3], &poses);

    // Assert
    assert_eq!(zero, Err(PlacementError::UnknownToolKind));
    assert_eq!(three, Err(PlacementError::UnknownToolKind));
}

#[test]
fn rejects_a_tool_outside_the_level_allowlist() {
    // Arrange
    let no_tools = LevelSpec {
        tool_budget: &[],
        ..TOOLS_SPEC
    };

    // Act
    let parsed = Placements::parse(&[1], &[12, 64, 0, 0], &no_tools);

    // Assert
    assert_eq!(parsed, Err(PlacementError::ToolNotAllowed));
}

#[test]
fn rejects_a_tool_beyond_its_budget() {
    // Arrange
    let kinds = [1_u8; 3];
    let poses = [0, 50, 0, 0, 10, 50, 0, 0, 20, 50, 0, 0];

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
        let parsed = parse(&[1], &[12, 64, angle, 0]);

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
    let poses = [40_000, 64, 0, 0];

    // Act
    let parsed = parse(&[1], &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::PositionOutOfRange));
}

#[test]
fn rejects_a_position_outside_the_build_area() {
    // Arrange
    let outside = [[41, 64, 0, 0], [12, 39, 0, 0]];
    let inclusive_edges = [-20, 40, 0, 0, 40, 90, 0, 0];

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
    let forward = [20, 50, 0, 0, 10, 60, 3, 0];
    let reversed = [10, 60, 3, 0, 20, 50, 0, 0];

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
        PlacementError::OverlapsWall,
        PlacementError::OverlapsCupWall,
        PlacementError::InsideCup,
        PlacementError::OverlapsWater,
        PlacementError::OverlapsTool,
        PlacementError::OverlapsNoPlace,
        PlacementError::ValveTimeOutOfRange,
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
    let kinds = [ToolKind::Plank, ToolKind::Valve];

    for kind in kinds {
        // Act
        let from_code = ToolKind::from_code(kind.code());
        let from_token = ToolKind::from_token(kind.token());

        // Assert
        assert_eq!(from_code, Some(kind));
        assert_eq!(from_token, Some(kind));
    }
}

#[test]
fn valve_has_wire_code_two_and_token_valve() {
    // Arrange
    let kind = ToolKind::Valve;

    // Act
    let (code, token) = (kind.code(), kind.token());

    // Assert
    assert_eq!(code, 2);
    assert_eq!(token, "valve");
    assert_eq!(ToolKind::from_code(2), Some(ToolKind::Valve));
    assert_eq!(ToolKind::from_token("valve"), Some(ToolKind::Valve));
}

#[test]
fn codes_zero_and_three_are_unknown() {
    // Arrange
    let codes = [0_u8, 3];

    // Act
    let kinds = codes.map(ToolKind::from_code);

    // Assert
    assert_eq!(kinds, [None, None]);
}

#[test]
fn a_valve_parses_with_its_open_tick() {
    // Arrange
    let poses = [12, 64, 0, 4];

    // Act
    let parsed = parse(&[ToolKind::Valve.code()], &poses).expect("a timed valve parses");

    // Assert
    let [placement] = parsed.as_slice() else {
        panic!("expected exactly one placement");
    };
    assert_eq!(placement.kind(), ToolKind::Valve);
    assert_eq!(placement.open_tick(), 4);
}

#[test]
fn valve_open_tick_accepts_one_through_the_step_limit_ticks() {
    // Arrange
    let ticks = [1, 24];

    for tick in ticks {
        // Act
        let parsed = parse(&[2], &[12, 64, 0, tick]);

        // Assert
        assert!(parsed.is_ok(), "tick {tick}");
    }
}

#[test]
fn valve_open_tick_outside_the_range_is_rejected() {
    // Arrange
    let ticks = [0, 25, -1, 300];

    for tick in ticks {
        // Act
        let parsed = parse(&[2], &[12, 64, 0, tick]);

        // Assert
        assert_eq!(
            parsed,
            Err(PlacementError::ValveTimeOutOfRange),
            "tick {tick}"
        );
    }
}

#[test]
fn a_plank_with_a_non_zero_open_tick_is_rejected() {
    // Arrange
    let poses = [12, 64, 0, 1];

    // Act
    let parsed = parse(&[1], &poses);

    // Assert
    assert_eq!(parsed, Err(PlacementError::ValveTimeOutOfRange));
}

#[test]
fn first_pour_max_open_tick_is_twenty_four() {
    // Arrange
    let spec = FIRST_POUR;

    // Act
    let max_open_tick = spec.max_open_tick();

    // Assert
    assert_eq!(max_open_tick, 24);
}

#[test]
fn canonical_order_puts_planks_before_valves() {
    // Arrange
    let kinds = [2, 1, 1];
    let poses = [-20, 40, 0, 4, 30, 80, 0, 0, 40, 90, 0, 0];

    // Act
    let parsed = parse(&kinds, &poses).expect("planks and a valve parse");

    // Assert
    let order: Vec<ToolKind> = parsed
        .as_slice()
        .iter()
        .map(|placement| placement.kind())
        .collect();
    assert_eq!(order, [ToolKind::Plank, ToolKind::Plank, ToolKind::Valve]);
}

#[test]
fn canonical_order_breaks_pose_ties_by_open_tick() {
    // Arrange
    let early = Placement {
        open_tick: 3,
        ..tool_at(ToolKind::Valve, 12, 64, 0)
    };
    let late = Placement {
        open_tick: 5,
        ..early
    };

    // Act
    let mut sorted = [late, early];
    sorted.sort_unstable();

    // Assert
    assert_eq!(sorted, [early, late]);
}

#[test]
fn zero_angle_valve_vertices_are_a_short_gate() {
    // Arrange
    let placement = tool_at(ToolKind::Valve, 0, 0, 0);

    // Act
    let vertices = placement.vertices();

    // Assert
    let bits: Vec<u32> = vertices
        .iter()
        .flat_map(|vertex| [vertex.x.to_bits(), vertex.y.to_bits()])
        .collect();
    let expected: Vec<u32> = [
        (-0.6_f32, -0.125_f32),
        (0.6, -0.125),
        (0.6, 0.125),
        (-0.6, 0.125),
    ]
    .iter()
    .flat_map(|(x, y)| [x.to_bits(), y.to_bits()])
    .collect();
    assert_eq!(bits, expected);
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

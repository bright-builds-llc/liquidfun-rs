//! Public time-of-impact contract and witness tests.

use liquidfun::collision::CollisionError;
use liquidfun::collision::shape::{ChainShape, CircleShape, EdgeShape, PolygonShape, Shape};
use liquidfun::collision::toi::{TimeOfImpactInput, TimeOfImpactState, time_of_impact};
use liquidfun::math::{Sweep, Vec2};

fn stationary_sweep(center: Vec2, angle: f32) -> Sweep {
    Sweep::new(Vec2::ZERO, center, center, angle, angle, 0.0).expect("test sweep should be valid")
}

fn circle(radius: f32) -> Shape {
    CircleShape::new(Vec2::ZERO, radius)
        .expect("test circle should be valid")
        .into()
}

fn moving_sweep(initial_center: Vec2, center: Vec2, initial_angle: f32, angle: f32) -> Sweep {
    Sweep::new(
        Vec2::ZERO,
        initial_center,
        center,
        initial_angle,
        angle,
        0.0,
    )
    .expect("test sweep should be valid")
}

fn run_toi(
    shape_a: &Shape,
    sweep_a: Sweep,
    shape_b: &Shape,
    sweep_b: Sweep,
) -> liquidfun::collision::toi::TimeOfImpactOutput {
    let child_a = shape_a.child_index(0).expect("first child should exist");
    let child_b = shape_b.child_index(0).expect("first child should exist");
    let input = TimeOfImpactInput::new(shape_a, child_a, sweep_a, shape_b, child_b, sweep_b, 1.0)
        .expect("TOI input should be valid");
    time_of_impact(&input).expect("finite TOI query should execute")
}

#[test]
fn toi_input_rejects_child_from_another_topology() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let foreign_child = liquidfun::collision::ChildIndex::new(1, 2)
        .expect("foreign topology should have a second child");
    let sweep = stationary_sweep(Vec2::ZERO, 0.0);

    // Act
    let result = TimeOfImpactInput::new(
        &shape_a,
        child_a,
        sweep,
        &shape_b,
        foreign_child,
        sweep,
        1.0,
    );

    // Assert
    assert!(matches!(
        result,
        Err(CollisionError::ChildIndexOutOfRange {
            requested: 1,
            child_count: 1,
        })
    ));
}

#[test]
fn toi_input_rejects_non_finite_t_max() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let sweep = stationary_sweep(Vec2::ZERO, 0.0);

    // Act
    let result =
        TimeOfImpactInput::new(&shape_a, child_a, sweep, &shape_b, child_b, sweep, f32::NAN);

    // Assert
    assert!(matches!(result, Err(CollisionError::NonFiniteValue)));
}

#[test]
fn toi_input_rejects_out_of_range_t_max() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let sweep = stationary_sweep(Vec2::ZERO, 0.0);

    // Act
    let result = TimeOfImpactInput::new(&shape_a, child_a, sweep, &shape_b, child_b, sweep, 1.25);

    // Assert
    assert!(matches!(result, Err(CollisionError::FractionOutOfRange)));
}

#[test]
fn toi_input_exposes_only_checked_t_max() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(2.0);
    let child_a = shape_a.child_index(0).expect("circle child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");

    // Act
    let input = TimeOfImpactInput::new(
        &shape_a,
        child_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        child_b,
        stationary_sweep(Vec2::new(4.0, 0.0), 0.0),
        0.75,
    )
    .expect("checked input should be valid");

    // Assert
    assert_eq!(input.t_max().to_bits(), 0.75_f32.to_bits());
}

#[test]
fn toi_testbed_witness_large_angle_normalization_does_not_mutate_callers() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(25.0, 5.0)
        .expect("test polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::box_shape(2.5, 2.5)
        .expect("test polygon should be valid")
        .into();
    let child_a = shape_a.child_index(0).expect("polygon child should exist");
    let child_b = shape_b.child_index(0).expect("polygon child should exist");
    let sweep_a = stationary_sweep(Vec2::new(24.0, -60.0), 2.95);
    let sweep_b = Sweep::new(
        Vec2::ZERO,
        Vec2::new(53.474_274, -50.252_514),
        Vec2::new(54.595_478, -51.083_473),
        513.366_76,
        513.627_8,
        0.0,
    )
    .expect("large-angle test sweep should be valid");
    let original_a = sweep_a;
    let original_b = sweep_b;
    let input = TimeOfImpactInput::new(&shape_a, child_a, sweep_a, &shape_b, child_b, sweep_b, 1.0)
        .expect("checked input should be valid");

    // Act
    let output = time_of_impact(&input).expect("finite testbed input should execute");

    // Assert
    assert_eq!(sweep_a, original_a);
    assert_eq!(sweep_b, original_b);
    assert!(matches!(
        output.state(),
        TimeOfImpactState::Overlapped
            | TimeOfImpactState::Touching
            | TimeOfImpactState::Separated
            | TimeOfImpactState::Failed
    ));
    assert!((0.0..=1.0).contains(&output.time()));
}

#[test]
fn toi_states_overlapped_for_coincident_circle_cores() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let sweep = stationary_sweep(Vec2::ZERO, 0.0);

    // Act
    let output = run_toi(&shape_a, sweep, &shape_b, sweep);

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Overlapped);
    assert_eq!(output.time().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn toi_states_touching_at_zero_inside_target_band() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let target = 2.0 - 3.0 * liquidfun::math::settings::LINEAR_SLOP;

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        stationary_sweep(Vec2::new(target, 0.0), 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert_eq!(output.time().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn toi_states_separated_for_stationary_distant_shapes() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        stationary_sweep(Vec2::new(5.0, 0.0), 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Separated);
    assert_eq!(output.time().to_bits(), 1.0_f32.to_bits());
}

#[test]
fn toi_states_translation_reaches_target_separation() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        moving_sweep(Vec2::new(5.0, 0.0), Vec2::ZERO, 0.0, 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.60..0.61).contains(&output.time()));
}

#[test]
fn toi_states_rotation_without_translation_reaches_target() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(2.0, 0.25)
        .expect("test polygon should be valid")
        .into();
    let shape_b = circle(0.5);

    // Act
    let output = run_toi(
        &shape_a,
        moving_sweep(Vec2::ZERO, Vec2::ZERO, 0.0, std::f32::consts::FRAC_PI_2),
        &shape_b,
        stationary_sweep(Vec2::new(0.0, 2.0), 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.0..1.0).contains(&output.time()));
}

#[test]
fn toi_states_combined_translation_and_rotation_stays_bounded() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(1.5, 0.5)
        .expect("test polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::box_shape(0.75, 0.25)
        .expect("test polygon should be valid")
        .into();

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        moving_sweep(
            Vec2::new(5.0, 0.5),
            Vec2::new(0.5, 0.0),
            -0.5,
            std::f32::consts::FRAC_PI_2,
        ),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.0..=1.0).contains(&output.time()));
}

#[test]
fn toi_states_near_tangent_translation_is_bounded() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        moving_sweep(Vec2::new(-4.0, 1.98), Vec2::new(4.0, 1.98), 0.0, 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.0..=1.0).contains(&output.time()));
}

#[test]
fn toi_states_edge_child_reaches_circle() {
    // Arrange
    let shape_a: Shape = EdgeShape::new(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0))
        .expect("test edge should be valid")
        .into();
    let shape_b = circle(1.0);

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        moving_sweep(Vec2::new(0.0, 3.0), Vec2::ZERO, 0.0, 0.0),
    );

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.0..1.0).contains(&output.time()));
}

#[test]
fn toi_states_chain_child_reaches_circle() {
    // Arrange
    let shape_a: Shape = ChainShape::open(
        &[
            Vec2::new(-3.0, 0.0),
            Vec2::new(-1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ],
        None,
        None,
    )
    .expect("test chain should be valid")
    .into();
    let shape_b = circle(1.0);
    let child_a = shape_a
        .child_index(1)
        .expect("second chain child should exist");
    let child_b = shape_b.child_index(0).expect("circle child should exist");
    let input = TimeOfImpactInput::new(
        &shape_a,
        child_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        child_b,
        moving_sweep(Vec2::new(0.0, 3.0), Vec2::ZERO, 0.0, 0.0),
        1.0,
    )
    .expect("TOI input should be valid");

    // Act
    let output = time_of_impact(&input).expect("finite TOI query should execute");

    // Assert
    assert_eq!(output.state(), TimeOfImpactState::Touching);
    assert!((0.0..1.0).contains(&output.time()));
}

#[test]
fn toi_states_symmetric_support_ties_remain_deterministic() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(1.0, 1.0)
        .expect("test polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::box_shape(1.0, 1.0)
        .expect("test polygon should be valid")
        .into();
    let sweep_a = stationary_sweep(Vec2::ZERO, 0.0);
    let sweep_b = moving_sweep(Vec2::new(0.0, 5.0), Vec2::ZERO, 0.0, 0.0);

    // Act
    let first = run_toi(&shape_a, sweep_a, &shape_b, sweep_b);
    let second = run_toi(&shape_a, sweep_a, &shape_b, sweep_b);

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.state(), TimeOfImpactState::Touching);
}

#[test]
fn toi_caps_successful_outputs_stay_inside_requested_interval() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let scenarios = [
        (Vec2::new(5.0, 0.0), Vec2::ZERO),
        (Vec2::new(-4.0, 1.98), Vec2::new(4.0, 1.98)),
        (Vec2::new(5.0, 5.0), Vec2::new(-5.0, -5.0)),
    ];

    // Act
    let outputs = scenarios.map(|(initial, final_center)| {
        run_toi(
            &shape_a,
            stationary_sweep(Vec2::ZERO, 0.0),
            &shape_b,
            moving_sweep(initial, final_center, 0.0, 0.0),
        )
    });

    // Assert
    assert!(
        outputs
            .iter()
            .all(|output| (0.0..=1.0).contains(&output.time()))
    );
}

#[test]
fn toi_caps_translation_time_respects_fixed_tolerance() {
    // Arrange
    let shape_a = circle(1.0);
    let shape_b = circle(1.0);
    let target = 2.0 - 3.0 * liquidfun::math::settings::LINEAR_SLOP;
    let tolerance = 0.25 * liquidfun::math::settings::LINEAR_SLOP;

    // Act
    let output = run_toi(
        &shape_a,
        stationary_sweep(Vec2::ZERO, 0.0),
        &shape_b,
        moving_sweep(Vec2::new(5.0, 0.0), Vec2::ZERO, 0.0, 0.0),
    );
    let separation = 5.0 * (1.0 - output.time());
    let before_separation = 5.0 * (1.0 - (output.time() - 0.001));
    let after_separation = 5.0 * (1.0 - (output.time() + 0.001));

    // Assert
    assert!((separation - target).abs() < tolerance);
    assert!(before_separation > target);
    assert!(after_separation < target);
}

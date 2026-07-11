//! Public time-of-impact contract and witness tests.

use liquidfun::collision::CollisionError;
use liquidfun::collision::shape::{CircleShape, PolygonShape, Shape};
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
fn toi_input_large_angle_normalization_does_not_mutate_callers() {
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

use proptest::prelude::*;

use crate::collision::{ChainShape, CircleShape, PolygonShape, Shape};
use crate::math::{Rotation, Transform, Vec2};

use super::{ParticleGroupSamplingError, SamplingLimits, plan_samples};
use crate::particle::group::{
    FilledParticleGroupShapes, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};

const DEFAULT_STRIDE: f32 = 0.5;
const MAX_WORK: usize = 4_096;

fn limits(maximum_samples: usize) -> SamplingLimits {
    SamplingLimits::new(MAX_WORK, maximum_samples)
}

fn recipe(source: ParticleGroupSource) -> ParticleGroupRecipe {
    ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
}

fn positions(plan: &super::SamplePlan) -> Vec<Vec2> {
    plan.samples()
        .iter()
        .map(|sample| sample.position())
        .collect()
}

#[test]
fn filled_union_deduplicates_overlapping_shapes() {
    // Arrange
    let circle = Shape::Circle(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"));
    let single = ParticleGroupSource::FilledShapes(
        FilledParticleGroupShapes::new(vec![circle.clone()]).expect("single shape should be valid"),
    );
    let overlapping = ParticleGroupSource::FilledShapes(
        FilledParticleGroupShapes::new(vec![circle.clone(), circle])
            .expect("overlapping shapes should be valid"),
    );
    let single_recipe = recipe(single)
        .with_stride(DEFAULT_STRIDE)
        .expect("stride should be valid");
    let overlapping_recipe = recipe(overlapping)
        .with_stride(DEFAULT_STRIDE)
        .expect("stride should be valid");

    // Act
    let expected =
        plan_samples(&single_recipe, DEFAULT_STRIDE, limits(64)).expect("shape should sample");
    let actual =
        plan_samples(&overlapping_recipe, DEFAULT_STRIDE, limits(64)).expect("union should sample");

    // Assert
    assert_eq!(positions(&actual), positions(&expected));
}

#[test]
fn filled_sampling_snaps_down_and_traverses_y_then_x_below_upper_bounds() {
    // Arrange
    let shape = Shape::Polygon(PolygonShape::box_shape(1.0, 1.0).expect("box should be valid"));
    let source = ParticleGroupSource::filled_shapes(vec![shape]).expect("source should be valid");
    let recipe = recipe(source)
        .with_stride(0.6)
        .expect("stride should be valid");

    // Act
    let plan = plan_samples(&recipe, DEFAULT_STRIDE, limits(32)).expect("box should sample");

    // Assert
    assert_eq!(
        positions(&plan),
        vec![
            Vec2::new(-0.6, -0.6),
            Vec2::new(0.0, -0.6),
            Vec2::new(0.6, -0.6),
            Vec2::new(-0.6, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.6, 0.0),
            Vec2::new(-0.6, 0.6),
            Vec2::new(0.0, 0.6),
            Vec2::new(0.6, 0.6),
        ]
    );
}

#[test]
fn filled_axis_start_uses_floor_before_multiplying_by_stride() {
    // Arrange
    let lower = -0.3;
    let upper = 0.7;
    let stride = 0.25;

    // Act
    let axis = super::preflight_axis(lower, upper, stride, MAX_WORK)
        .expect("bounded axis should preflight");

    // Assert
    assert_eq!(axis.start.to_bits(), (-0.5_f32).to_bits());
}

#[test]
fn chain_sampling_carries_position_across_children() {
    // Arrange
    let chain = ChainShape::open(
        &[
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
        ],
        None,
        None,
    )
    .expect("chain should be valid");
    let source = ParticleGroupSource::stroke_shape(Shape::Chain(chain))
        .expect("chain should be a stroke source");
    let recipe = recipe(source)
        .with_stride(0.6)
        .expect("stride should be valid");

    // Act
    let plan = plan_samples(&recipe, DEFAULT_STRIDE, limits(16))
        .expect("chain should sample with carried distance");

    // Assert
    assert_eq!(
        positions(&plan),
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.6, 0.0),
            Vec2::new(1.0, 0.200_000_05),
            Vec2::new(1.0, 0.800_000_1),
        ]
    );
    assert_ne!(
        positions(&plan),
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.6, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 0.6),
        ]
    );
}

#[test]
fn explicit_positions_preserve_input_order_and_transform_after_selection() {
    // Arrange
    let local = vec![
        Vec2::new(2.0, 1.0),
        Vec2::new(-3.0, 4.0),
        Vec2::new(0.5, -0.25),
    ];
    let source =
        ParticleGroupSource::positions(local.clone()).expect("positions should be a valid source");
    let transform = Transform::new(Vec2::new(10.0, -2.0), Rotation::from_angle(0.5));
    let recipe = recipe(source)
        .with_transform(transform)
        .expect("transform should be valid");

    // Act
    let plan =
        plan_samples(&recipe, DEFAULT_STRIDE, limits(local.len())).expect("positions sample");

    // Assert
    let expected = local
        .into_iter()
        .map(|position| transform.apply(position))
        .collect::<Vec<_>>();
    assert_eq!(positions(&plan), expected);
}

#[test]
fn initial_velocity_preserves_pinned_expression_grouping() {
    // Arrange
    let group_position = Vec2::new(4.0, -3.0);
    let transform = Transform::new(group_position, Rotation::IDENTITY);
    let linear_velocity = Vec2::new(1.25, -2.5);
    let angular_velocity = 3.5;
    let source = ParticleGroupSource::positions(vec![Vec2::new(-2.0, 5.0)])
        .expect("position should be valid");
    let recipe = recipe(source)
        .with_transform(transform)
        .expect("transform should be valid")
        .with_linear_velocity(linear_velocity)
        .expect("linear velocity should be valid")
        .with_angular_velocity(angular_velocity)
        .expect("angular velocity should be valid");
    let world_position = transform.apply(Vec2::new(-2.0, 5.0));
    let expected =
        linear_velocity + Vec2::scalar_cross(angular_velocity, world_position - group_position);

    // Act
    let plan = plan_samples(&recipe, DEFAULT_STRIDE, limits(1)).expect("position should sample");

    // Assert
    assert_eq!(
        plan.samples()[0].velocity().x.to_bits(),
        expected.x.to_bits()
    );
    assert_eq!(
        plan.samples()[0].velocity().y.to_bits(),
        expected.y.to_bits()
    );
}

#[test]
fn huge_finite_fill_is_rejected_before_materialization() {
    // Arrange
    let circle =
        Shape::Circle(CircleShape::new(Vec2::ZERO, 1.0e20).expect("finite circle should be valid"));
    let source = ParticleGroupSource::filled_shapes(vec![circle]).expect("source should be valid");
    let recipe = recipe(source)
        .with_stride(1.0e-10)
        .expect("tiny finite stride should be valid");

    // Act
    let result = plan_samples(&recipe, DEFAULT_STRIDE, SamplingLimits::new(128, 128));

    // Assert
    assert!(matches!(
        result,
        Err(ParticleGroupSamplingError::WorkLimitExceeded { limit: 128, .. })
    ));
}

#[test]
fn exact_sample_count_is_checked_against_effective_capacity() {
    // Arrange
    let source = ParticleGroupSource::positions(vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)])
        .expect("positions should be valid");
    let recipe = recipe(source);

    // Act
    let result = plan_samples(&recipe, DEFAULT_STRIDE, SamplingLimits::new(8, 1));

    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupSamplingError::CapacityExceeded {
            required: 2,
            limit: 1,
        })
    );
}

#[test]
fn non_finite_derived_position_is_rejected() {
    // Arrange
    let source = ParticleGroupSource::positions(vec![Vec2::new(f32::MAX, 0.0)])
        .expect("finite position should be valid");
    let transform = Transform::new(Vec2::new(f32::MAX, 0.0), Rotation::IDENTITY);
    let recipe = recipe(source)
        .with_transform(transform)
        .expect("finite transform should be valid");

    // Act
    let result = plan_samples(&recipe, DEFAULT_STRIDE, limits(1));

    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupSamplingError::NonFiniteDerivedPosition)
    );
}

#[test]
fn non_finite_derived_velocity_is_rejected() {
    // Arrange
    let source = ParticleGroupSource::positions(vec![Vec2::new(f32::MAX, 0.0)])
        .expect("finite position should be valid");
    let recipe = recipe(source)
        .with_angular_velocity(2.0)
        .expect("finite angular velocity should be valid");

    // Act
    let result = plan_samples(&recipe, DEFAULT_STRIDE, limits(1));

    // Assert
    assert_eq!(
        result,
        Err(ParticleGroupSamplingError::NonFiniteDerivedVelocity)
    );
}

proptest! {
    #[test]
    fn translating_a_filled_shape_translates_every_sample(
        translation_x in -10.0_f32..10.0,
        translation_y in -10.0_f32..10.0,
    ) {
        // Arrange
        let circle = Shape::Circle(
            CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"),
        );
        let source = ParticleGroupSource::filled_shapes(vec![circle])
            .expect("source should be valid");
        let local_recipe = recipe(source.clone()).with_stride(DEFAULT_STRIDE)
            .expect("stride should be valid");
        let translation = Vec2::new(translation_x, translation_y);
        let translated_recipe = recipe(source)
            .with_stride(DEFAULT_STRIDE)
            .expect("stride should be valid")
            .with_transform(Transform::new(translation, Rotation::IDENTITY))
            .expect("transform should be valid");

        // Act
        let local = plan_samples(&local_recipe, DEFAULT_STRIDE, limits(64))
            .expect("local shape should sample");
        let translated = plan_samples(&translated_recipe, DEFAULT_STRIDE, limits(64))
            .expect("translated shape should sample");

        // Assert
        let expected = positions(&local)
            .into_iter()
            .map(|position| position + translation)
            .collect::<Vec<_>>();
        prop_assert_eq!(positions(&translated), expected);
    }

    #[test]
    fn duplicating_a_filled_shape_is_union_idempotent(
        center_x in -2.0_f32..2.0,
        center_y in -2.0_f32..2.0,
        radius in 0.25_f32..2.0,
    ) {
        // Arrange
        let circle = Shape::Circle(
            CircleShape::new(Vec2::new(center_x, center_y), radius)
                .expect("circle should be valid"),
        );
        let single = ParticleGroupSource::filled_shapes(vec![circle.clone()])
            .expect("source should be valid");
        let duplicate = ParticleGroupSource::filled_shapes(vec![circle.clone(), circle])
            .expect("source should be valid");
        let single_recipe = recipe(single).with_stride(DEFAULT_STRIDE)
            .expect("stride should be valid");
        let duplicate_recipe = recipe(duplicate).with_stride(DEFAULT_STRIDE)
            .expect("stride should be valid");

        // Act
        let expected = plan_samples(&single_recipe, DEFAULT_STRIDE, limits(256))
            .expect("single shape should sample");
        let actual = plan_samples(&duplicate_recipe, DEFAULT_STRIDE, limits(256))
            .expect("duplicate union should sample");

        // Assert
        prop_assert_eq!(positions(&actual), positions(&expected));
    }
}

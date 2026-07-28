use super::*;
use crate::collision::ChildIndex;
use crate::collision::shape::{CircleShape, EdgeShape, PolygonShape, Shape};
use crate::math::Transform;
use crate::math::Vec2;

fn circle(center: Vec2) -> Shape {
    CircleShape::new(center, 1.0)
        .expect("circle should be valid")
        .into()
}

#[test]
fn cache_compatible_reuse_preserves_ordered_pairs() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let child = ChildIndex::new(0, 1).expect("child should exist");
    let proxy_a = DistanceProxy::new(&shape_a, child).expect("proxy should be valid");
    let proxy_b = DistanceProxy::new(&shape_b, child).expect("proxy should be valid");
    let pairs = [SupportIndexPair::new(0, 1)];
    let mut cache = DistanceCache::empty();
    cache
        .write(&proxy_a, &proxy_b, 0.0, &pairs)
        .expect("cache write should be valid");

    // Act
    let entries = cache
        .entries(&proxy_a, &proxy_b)
        .expect("same topology should be compatible");

    // Assert
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].index_a, 0);
    assert_eq!(entries[0].index_b, 1);
}

#[test]
fn cache_rejects_cross_topology_reuse_before_indexing() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(2.0, 0.0));
    let polygon: Shape = PolygonShape::box_shape(1.0, 1.0)
        .expect("polygon should be valid")
        .into();
    let child = ChildIndex::new(0, 1).expect("child should exist");
    let proxy_a = DistanceProxy::new(&shape_a, child).expect("proxy should be valid");
    let proxy_b = DistanceProxy::new(&shape_b, child).expect("proxy should be valid");
    let polygon_proxy = DistanceProxy::new(&polygon, child).expect("proxy should be valid");
    let mut cache = DistanceCache::empty();
    cache
        .write(&proxy_a, &proxy_b, 0.0, &[SupportIndexPair::new(0, 0)])
        .expect("cache write should be valid");

    // Act
    let result = cache.entries(&polygon_proxy, &proxy_b);

    // Assert
    assert_eq!(result, Err(CollisionError::IncompatibleDistanceCache));
}

#[test]
fn cache_ratio_boundaries_are_inclusive() {
    // Arrange
    let metric = 4.0;

    // Act
    let half_flushes = cache_metric_requires_flush(metric, 2.0);
    let double_flushes = cache_metric_requires_flush(metric, 8.0);
    let below_half_flushes =
        cache_metric_requires_flush(metric, f32::from_bits(2.0_f32.to_bits() - 1));
    let above_double_flushes =
        cache_metric_requires_flush(metric, f32::from_bits(8.0_f32.to_bits() + 1));

    // Assert
    assert!(!half_flushes);
    assert!(!double_flushes);
    assert!(below_half_flushes);
    assert!(above_double_flushes);
}

#[test]
fn cache_epsilon_flush_is_strict() {
    // Arrange
    let below = f32::from_bits(EPSILON.to_bits() - 1);

    // Act
    let below_flushes = cache_metric_requires_flush(EPSILON, below);
    let equal_flushes = cache_metric_requires_flush(EPSILON, EPSILON);

    // Assert
    assert!(below_flushes);
    assert!(!equal_flushes);
}

#[test]
fn gjk_identical_points_terminate_on_near_zero_direction() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::ZERO);
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child,
        Transform::IDENTITY,
        &shape_b,
        child,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(
        result.diagnostic_trace.termination,
        GjkTermination::NearZeroDirection
    );
    assert_eq!(result.iterations, 0);
}

#[test]
fn gjk_separated_points_terminate_on_duplicate_support() {
    // Arrange
    let shape_a = circle(Vec2::ZERO);
    let shape_b = circle(Vec2::new(4.0, 0.0));
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child,
        Transform::IDENTITY,
        &shape_b,
        child,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(
        result.diagnostic_trace.termination,
        GjkTermination::DuplicateSupport
    );
    assert_eq!(result.diagnostic_trace.steps.len(), 1);
}

#[test]
fn gjk_overlapping_polygons_terminate_with_triangle_simplex() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(1.0, 1.0)
        .expect("polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::oriented_box(1.0, 1.0, Vec2::new(0.25, 0.1), 0.2)
        .expect("polygon should be valid")
        .into();
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child,
        Transform::IDENTITY,
        &shape_b,
        child,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(
        result.diagnostic_trace.termination,
        GjkTermination::Triangle
    );
    assert_eq!(result.cache.snapshot().count(), 3);
}

#[test]
fn gjk_iteration_trace_is_bounded_by_pinned_cap() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(1.0, 2.0)
        .expect("polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::oriented_box(1.5, 0.5, Vec2::new(8.0, 3.0), 0.7)
        .expect("polygon should be valid")
        .into();
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child,
        Transform::IDENTITY,
        &shape_b,
        child,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(MAX_GJK_ITERATIONS, 20);
    assert!(result.iterations <= MAX_GJK_ITERATIONS);
    assert!(result.diagnostic_trace.steps.len() <= MAX_GJK_ITERATIONS);
}

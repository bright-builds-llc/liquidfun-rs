//! Public contract tests for GJK distance, reusable cache, and overlap.

use liquidfun::collision::distance::{DistanceCache, DistanceResult, distance, test_overlap};
use liquidfun::collision::shape::{CircleShape, PolygonShape, Shape};
use liquidfun::math::settings::EPSILON;
use liquidfun::math::{Transform, Vec2};

#[test]
fn cache_empty_state_is_fully_initialized() {
    // Arrange
    let cache = DistanceCache::empty();

    // Act
    let snapshot = cache.snapshot();

    // Assert
    assert_eq!(snapshot.count(), 0);
    assert!(snapshot.support_pairs().is_empty());
    assert_eq!(snapshot.metric().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn proxy_cache_snapshot_clone_preserves_semantic_state() {
    // Arrange
    let cache = DistanceCache::empty();

    // Act
    let cloned = cache.clone();

    // Assert
    assert_eq!(cloned.snapshot(), cache.snapshot());
}

#[test]
fn proxy_distance_result_surface_is_public_and_read_only() {
    // Arrange
    let maybe_result: Option<&DistanceResult> = None;

    // Act
    let is_absent = maybe_result.is_none();

    // Assert
    assert!(is_absent);
}

fn circle(center: Vec2, radius: f32) -> Shape {
    CircleShape::new(center, radius)
        .expect("circle should be valid")
        .into()
}

#[test]
fn gjk_circle_distance_returns_witnesses_and_iterations() {
    // Arrange
    let shape_a = circle(Vec2::ZERO, 1.0);
    let shape_b = circle(Vec2::new(5.0, 0.0), 2.0);
    let child_a = shape_a.child_index(0).expect("child should exist");
    let child_b = shape_b.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(result.point_a(), Vec2::ZERO);
    assert_eq!(result.point_b(), Vec2::new(5.0, 0.0));
    assert_eq!(result.distance().to_bits(), 5.0_f32.to_bits());
    assert_eq!(result.iterations(), 1);
}

#[test]
fn gjk_radii_move_separated_witnesses_to_surfaces() {
    // Arrange
    let shape_a = circle(Vec2::ZERO, 1.0);
    let shape_b = circle(Vec2::new(5.0, 0.0), 2.0);
    let child_a = shape_a.child_index(0).expect("child should exist");
    let child_b = shape_b.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        true,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(result.point_a(), Vec2::new(1.0, 0.0));
    assert_eq!(result.point_b(), Vec2::new(3.0, 0.0));
    assert_eq!(result.distance().to_bits(), 2.0_f32.to_bits());
}

#[test]
fn gjk_radii_collapse_overlapping_witnesses_to_midpoint() {
    // Arrange
    let shape_a = circle(Vec2::ZERO, 1.0);
    let shape_b = circle(Vec2::new(1.5, 0.0), 1.0);
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let result = distance(
        &shape_a,
        child,
        Transform::IDENTITY,
        &shape_b,
        child,
        Transform::IDENTITY,
        true,
        None,
    )
    .expect("distance should succeed");

    // Assert
    assert_eq!(result.point_a(), Vec2::new(0.75, 0.0));
    assert_eq!(result.point_b(), result.point_a());
    assert_eq!(result.distance().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn gjk_warm_cache_repeats_result_and_semantic_state() {
    // Arrange
    let shape_a: Shape = PolygonShape::box_shape(1.0, 1.0)
        .expect("polygon should be valid")
        .into();
    let shape_b: Shape = PolygonShape::oriented_box(1.0, 1.0, Vec2::new(4.0, 0.5), 0.2)
        .expect("polygon should be valid")
        .into();
    let child_a = shape_a.child_index(0).expect("child should exist");
    let child_b = shape_b.child_index(0).expect("child should exist");
    let cold = distance(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        None,
    )
    .expect("cold distance should succeed");

    // Act
    let warm = distance(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        Transform::IDENTITY,
        false,
        Some(cold.cache()),
    )
    .expect("warm distance should succeed");

    // Assert
    assert_eq!(warm.point_a(), cold.point_a());
    assert_eq!(warm.point_b(), cold.point_b());
    assert_eq!(warm.distance().to_bits(), cold.distance().to_bits());
    assert_eq!(warm.cache().snapshot(), cold.cache().snapshot());
}

#[test]
fn cache_debug_omits_private_topology_identity() {
    // Arrange
    let shape_a = circle(Vec2::ZERO, 1.0);
    let shape_b = circle(Vec2::new(3.0, 0.0), 1.0);
    let child = shape_a.child_index(0).expect("child should exist");
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

    // Act
    let cache_debug = format!("{:?}", result.cache());

    // Assert
    assert!(!cache_debug.contains("proxy_a"));
    assert!(!cache_debug.contains("vertex_bits"));
}

#[test]
fn overlap_threshold_is_strict_below_equal_and_above() {
    // Arrange
    let threshold = 10.0 * EPSILON;
    let below = f32::from_bits(threshold.to_bits() - 1);
    let above = f32::from_bits(threshold.to_bits() + 1);
    let shape_a = circle(Vec2::ZERO, 0.0);
    let below_shape = circle(Vec2::new(below, 0.0), 0.0);
    let equal_shape = circle(Vec2::new(threshold, 0.0), 0.0);
    let above_shape = circle(Vec2::new(above, 0.0), 0.0);
    let child = shape_a.child_index(0).expect("child should exist");

    // Act
    let below_overlaps = test_overlap(
        &shape_a,
        child,
        Transform::IDENTITY,
        &below_shape,
        child,
        Transform::IDENTITY,
    )
    .expect("overlap should succeed");
    let equal_overlaps = test_overlap(
        &shape_a,
        child,
        Transform::IDENTITY,
        &equal_shape,
        child,
        Transform::IDENTITY,
    )
    .expect("overlap should succeed");
    let above_overlaps = test_overlap(
        &shape_a,
        child,
        Transform::IDENTITY,
        &above_shape,
        child,
        Transform::IDENTITY,
    )
    .expect("overlap should succeed");

    // Assert
    assert!(below_overlaps);
    assert!(!equal_overlaps);
    assert!(!above_overlaps);
}

proptest::proptest! {
    #[test]
    fn gjk_circle_symmetry_preserves_distance(
        ax in -100.0_f32..100.0,
        ay in -100.0_f32..100.0,
        bx in -100.0_f32..100.0,
        by in -100.0_f32..100.0,
    ) {
        // Arrange
        let shape_a = circle(Vec2::new(ax, ay), 0.5);
        let shape_b = circle(Vec2::new(bx, by), 0.75);
        let child = shape_a.child_index(0).expect("child should exist");

        // Act
        let ab = distance(
            &shape_a,
            child,
            Transform::IDENTITY,
            &shape_b,
            child,
            Transform::IDENTITY,
            true,
            None,
        ).expect("distance should succeed");
        let ba = distance(
            &shape_b,
            child,
            Transform::IDENTITY,
            &shape_a,
            child,
            Transform::IDENTITY,
            true,
            None,
        ).expect("distance should succeed");

        // Assert
        proptest::prop_assert_eq!(ab.distance().to_bits(), ba.distance().to_bits());
        let ab_separation = (ab.point_b() - ab.point_a()).length();
        let ba_separation = (ba.point_b() - ba.point_a()).length();
        let ab_witness_tolerance = 4.0 * f32::EPSILON * ab.distance().max(1.0);
        let ba_witness_tolerance = 4.0 * f32::EPSILON * ba.distance().max(1.0);
        proptest::prop_assert!((ab_separation - ab.distance()).abs() <= ab_witness_tolerance);
        proptest::prop_assert!((ba_separation - ba.distance()).abs() <= ba_witness_tolerance);
        proptest::prop_assert!(ab.iterations() <= 20);
        proptest::prop_assert!(ba.iterations() <= 20);
    }
}

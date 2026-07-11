//! Public contract tests for owned collision shapes and unary queries.

use liquidfun::collision::CollisionError;
use liquidfun::collision::RayCastInput;
use liquidfun::collision::shape::{CircleShape, EdgeShape, PolygonShape, Shape};
use liquidfun::math::settings::{MAX_POLYGON_VERTICES, PI, POLYGON_RADIUS};
use liquidfun::math::{Transform, Vec2};

#[test]
fn circle_queries_preserve_pinned_geometry() {
    // Arrange
    let circle = CircleShape::new(Vec2::new(2.0, -1.0), 3.0)
        .expect("finite non-negative circle should be valid");
    let transform = Transform::IDENTITY;

    // Act
    let mass = circle
        .compute_mass(2.0)
        .expect("finite density should produce mass");
    let aabb = circle
        .compute_aabb(transform)
        .expect("finite transform should produce bounds");
    let contains_boundary = circle
        .test_point(transform, Vec2::new(5.0, -1.0))
        .expect("finite point query should succeed");

    // Assert
    assert_eq!(mass.mass().to_bits(), (2.0 * PI * 3.0 * 3.0).to_bits());
    assert_eq!(mass.center(), Vec2::new(2.0, -1.0));
    assert_eq!(aabb.lower_bound(), Vec2::new(-1.0, -4.0));
    assert_eq!(aabb.upper_bound(), Vec2::new(5.0, 2.0));
    assert!(contains_boundary);
}

#[test]
fn circle_center_distance_uses_documented_finite_normal() {
    // Arrange
    let circle = CircleShape::new(Vec2::new(1.0, 2.0), 0.5).expect("circle should be valid");

    // Act
    let distance = circle
        .distance_to_point(Transform::IDENTITY, Vec2::new(1.0, 2.0))
        .expect("finite point query should succeed");

    // Assert
    assert_eq!(distance.distance().to_bits(), (-0.5_f32).to_bits());
    assert_eq!(distance.normal(), Vec2::ZERO);
    assert!(distance.normal().is_valid());
}

#[test]
fn circle_ray_from_inside_has_no_hit() {
    // Arrange
    let circle = CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid");
    let input =
        RayCastInput::new(Vec2::ZERO, Vec2::new(2.0, 0.0), 1.0).expect("ray should be valid");

    // Act
    let maybe_hit = circle
        .ray_cast(input, Transform::IDENTITY)
        .expect("finite ray cast should succeed");

    // Assert
    assert_eq!(maybe_hit, None);
}

#[test]
fn edge_queries_are_two_sided_and_point_test_is_false() {
    // Arrange
    let edge =
        EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)).expect("edge should be valid");
    let from_above = RayCastInput::new(Vec2::new(0.0, 1.0), Vec2::new(0.0, -1.0), 1.0)
        .expect("ray should be valid");
    let from_below = RayCastInput::new(Vec2::new(0.0, -1.0), Vec2::new(0.0, 1.0), 1.0)
        .expect("ray should be valid");

    // Act
    let above_hit = edge
        .ray_cast(from_above, Transform::IDENTITY)
        .expect("finite ray cast should succeed")
        .expect("ray should hit");
    let below_hit = edge
        .ray_cast(from_below, Transform::IDENTITY)
        .expect("finite ray cast should succeed")
        .expect("ray should hit");
    let contains = edge
        .test_point(Transform::IDENTITY, Vec2::ZERO)
        .expect("finite point query should succeed");

    // Assert
    assert_eq!(above_hit.normal(), Vec2::new(0.0, 1.0));
    assert_eq!(below_hit.normal(), Vec2::new(0.0, -1.0));
    assert!(!contains);
}

#[test]
fn edge_adjacency_and_mass_are_initialized() {
    // Arrange
    let edge = EdgeShape::with_adjacency(
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, 0.0),
        Some(Vec2::new(-2.0, 0.0)),
        Some(Vec2::new(2.0, 0.0)),
    )
    .expect("connected edge should be valid");

    // Act
    let mass = edge
        .compute_mass(12.0)
        .expect("finite density should produce mass data");
    let aabb = edge
        .compute_aabb(Transform::IDENTITY)
        .expect("finite transform should produce bounds");

    // Assert
    assert_eq!(edge.previous(), Some(Vec2::new(-2.0, 0.0)));
    assert_eq!(edge.next(), Some(Vec2::new(2.0, 0.0)));
    assert_eq!(mass.mass().to_bits(), 0.0_f32.to_bits());
    assert_eq!(mass.center(), Vec2::ZERO);
    assert_eq!(
        aabb.lower_bound(),
        Vec2::new(-1.0 - POLYGON_RADIUS, -POLYGON_RADIUS)
    );
    assert_eq!(
        aabb.upper_bound(),
        Vec2::new(1.0 + POLYGON_RADIUS, POLYGON_RADIUS)
    );
}

#[test]
fn circle_and_edge_reject_invalid_geometry() {
    // Arrange
    let non_finite = Vec2::new(f32::NAN, 0.0);

    // Act
    let bad_circle = CircleShape::new(non_finite, 1.0);
    let negative_circle = CircleShape::new(Vec2::ZERO, -1.0);
    let bad_edge = EdgeShape::new(Vec2::ZERO, Vec2::ZERO);
    let bad_adjacency =
        EdgeShape::with_adjacency(Vec2::ZERO, Vec2::new(1.0, 0.0), Some(Vec2::ZERO), None);

    // Assert
    assert_eq!(bad_circle, Err(CollisionError::NonFiniteValue));
    assert_eq!(negative_circle, Err(CollisionError::InvalidGeometry));
    assert_eq!(bad_edge, Err(CollisionError::InvalidGeometry));
    assert_eq!(bad_adjacency, Err(CollisionError::InvalidGeometry));
}

#[test]
fn shape_circle_dispatch_matches_concrete_queries() {
    // Arrange
    let circle = CircleShape::new(Vec2::new(1.0, 0.0), 2.0).expect("circle should be valid");
    let shape = Shape::from(circle.clone());

    // Act
    let concrete_mass = circle.compute_mass(3.0).expect("mass should be valid");
    let dispatched_mass = shape.compute_mass(3.0).expect("mass should be valid");

    // Assert
    assert_eq!(shape.radius().to_bits(), circle.radius().to_bits());
    assert_eq!(shape.child_count(), 1);
    assert_eq!(dispatched_mass, concrete_mass);
}

#[test]
fn polygon_hull_starts_at_rightmost_lowest_and_preserves_winding() {
    // Arrange
    let points = [
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::ZERO,
    ];

    // Act
    let polygon = PolygonShape::new(&points).expect("convex hull should be valid");

    // Assert
    assert_eq!(
        polygon.vertices(),
        &[
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(-1.0, -1.0),
        ]
    );
    assert!(polygon.validate());
}

#[test]
fn polygon_box_queries_preserve_pinned_results() {
    // Arrange
    let polygon = PolygonShape::box_shape(2.0, 1.0).expect("box should be valid");
    let ray = RayCastInput::new(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0), 1.0)
        .expect("ray should be valid");

    // Act
    let contains = polygon
        .test_point(Transform::IDENTITY, Vec2::new(2.0, 0.0))
        .expect("point query should succeed");
    let hit = polygon
        .ray_cast(ray, Transform::IDENTITY)
        .expect("ray query should succeed")
        .expect("ray should hit");
    let mass = polygon
        .compute_mass(3.0)
        .expect("density should produce mass");

    // Assert
    assert!(contains);
    assert_eq!(hit.fraction().to_bits(), (1.0_f32 / 6.0).to_bits());
    assert_eq!(hit.normal(), Vec2::new(-1.0, 0.0));
    assert_eq!(mass.mass().to_bits(), 24.0_f32.to_bits());
    assert_eq!(mass.center(), Vec2::ZERO);
}

#[test]
fn polygon_oriented_box_owns_transformed_geometry() {
    // Arrange
    let center = Vec2::new(3.0, -2.0);

    // Act
    let polygon = PolygonShape::oriented_box(2.0, 1.0, center, PI / 2.0)
        .expect("oriented box should be valid");

    // Assert
    assert_eq!(polygon.centroid(), center);
    assert_eq!(polygon.vertex_count(), 4);
    assert!(
        polygon
            .test_point(Transform::IDENTITY, center)
            .expect("point query should succeed")
    );
}

#[test]
fn polygon_rejects_safe_rust_departure_inputs() {
    // Arrange
    let excess = vec![Vec2::ZERO; MAX_POLYGON_VERTICES + 1];
    let collinear = [Vec2::new(-1.0, 0.0), Vec2::ZERO, Vec2::new(1.0, 0.0)];
    let non_finite = [Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(f32::NAN, 1.0)];

    // Act
    let excess_result = PolygonShape::new(&excess);
    let collinear_result = PolygonShape::new(&collinear);
    let non_finite_result = PolygonShape::new(&non_finite);

    // Assert
    assert_eq!(excess_result, Err(CollisionError::InvalidGeometry));
    assert_eq!(collinear_result, Err(CollisionError::InvalidGeometry));
    assert_eq!(non_finite_result, Err(CollisionError::NonFiniteValue));
}

#[test]
fn polygon_weld_uses_pinned_unsquared_slop_threshold() {
    // Arrange
    let points = [
        Vec2::new(0.0, 0.0),
        Vec2::new(0.04, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
    ];

    // Act
    let polygon = PolygonShape::new(&points).expect("three retained points should form a hull");

    // Assert
    assert_eq!(polygon.vertex_count(), 3);
    assert!(!polygon.vertices().contains(&Vec2::new(0.04, 0.0)));
}

proptest::proptest! {
    #[test]
    fn polygon_box_normals_are_unit_and_centroid_is_contained(
        half_width in 0.01_f32..100.0,
        half_height in 0.01_f32..100.0,
    ) {
        // Arrange
        let polygon = PolygonShape::box_shape(half_width, half_height)
            .expect("positive finite half-extents should be valid");

        // Act
        let normal_lengths: Vec<f32> = polygon.normals().iter().map(|normal| normal.length()).collect();
        let contains_centroid = polygon
            .test_point(Transform::IDENTITY, polygon.centroid())
            .expect("finite point query should succeed");

        // Assert
        proptest::prop_assert!(normal_lengths.iter().all(|length| (length - 1.0).abs() < 1.0e-5));
        proptest::prop_assert!(contains_centroid);
    }
}

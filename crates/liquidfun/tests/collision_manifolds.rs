//! Public contract tests for semantic manifolds and shape-pair dispatch.

use liquidfun::collision::narrow::{
    collide_circles, collide_polygon_circle, collide_polygons, point_states, world_manifold,
};
use liquidfun::collision::shape::{CircleShape, PolygonShape};
use liquidfun::collision::{
    ContactFeatureId, FeatureKind, Manifold, ManifoldKind, ManifoldPoint, PointState,
};
use liquidfun::math::{Transform, Vec2};

fn feature(index_a: u8, index_b: u8) -> ContactFeatureId {
    ContactFeatureId::new(index_a, index_b, FeatureKind::Face, FeatureKind::Vertex)
}

fn point(index_a: u8, index_b: u8, x: f32) -> ManifoldPoint {
    ManifoldPoint::new(Vec2::new(x, 0.0), feature(index_a, index_b))
        .expect("point should be finite")
}

#[test]
fn point_state_preserves_order_and_multiplicity() {
    // Arrange
    let old = Manifold::face_a(
        Vec2::new(0.0, 1.0),
        Vec2::ZERO,
        &[point(0, 0, -1.0), point(0, 1, 1.0)],
    )
    .expect("old manifold should be valid");
    let new = Manifold::face_a(
        Vec2::new(0.0, 1.0),
        Vec2::ZERO,
        &[point(0, 1, 1.0), point(0, 2, 2.0)],
    )
    .expect("new manifold should be valid");

    // Act
    let states = point_states(&old, &new);

    // Assert
    assert_eq!(
        states.previous(),
        &[PointState::Removed, PointState::Persisted]
    );
    assert_eq!(
        states.current(),
        &[PointState::Persisted, PointState::Added]
    );
}

#[test]
fn clipping_result_capacity_is_fixed_by_manifold_contract() {
    // Arrange
    let points = [point(1, 0, -1.0), point(1, 1, 1.0)];

    // Act
    let manifold = Manifold::face_a(Vec2::new(0.0, 1.0), Vec2::ZERO, &points)
        .expect("two-point manifold should fit");

    // Assert
    assert_eq!(manifold.points(), points.as_slice());
}

#[test]
fn circle_circle_distinguishes_separation_tangency_and_overlap() {
    // Arrange
    let circle = CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid");
    let tangent_transform = Transform::from_position_angle(Vec2::new(2.0, 0.0), 0.0);
    let separated_transform = Transform::from_position_angle(Vec2::new(2.01, 0.0), 0.0);

    // Act
    let separated = collide_circles(&circle, Transform::IDENTITY, &circle, separated_transform)
        .expect("transforms should be finite");
    let tangent = collide_circles(&circle, Transform::IDENTITY, &circle, tangent_transform)
        .expect("transforms should be finite")
        .expect("tangent circles should touch");
    let overlap = collide_circles(&circle, Transform::IDENTITY, &circle, Transform::IDENTITY)
        .expect("transforms should be finite")
        .expect("coincident circles should touch");

    // Assert
    assert!(separated.is_none());
    assert_eq!(tangent.kind(), Some(ManifoldKind::Circles));
    assert_eq!(overlap.points().len(), 1);
    assert_eq!(overlap.points()[0].feature_id(), feature_vertex_vertex());
}

#[test]
fn circle_polygon_selects_inside_face_face_region_and_vertex_region() {
    // Arrange
    let polygon = PolygonShape::box_shape(1.0, 1.0).expect("box should be valid");
    let circle = CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid");
    let face_transform = Transform::from_position_angle(Vec2::new(0.0, -1.5), 0.0);
    let vertex_transform = Transform::from_position_angle(Vec2::new(1.35, -1.35), 0.0);

    // Act
    let inside =
        collide_polygon_circle(&polygon, Transform::IDENTITY, &circle, Transform::IDENTITY)
            .expect("transforms should be finite")
            .expect("inside circle should touch");
    let face = collide_polygon_circle(&polygon, Transform::IDENTITY, &circle, face_transform)
        .expect("transforms should be finite")
        .expect("face-region circle should touch");
    let vertex = collide_polygon_circle(&polygon, Transform::IDENTITY, &circle, vertex_transform)
        .expect("transforms should be finite")
        .expect("vertex-region circle should touch");

    // Assert
    assert_eq!(inside.local_normal(), Some(Vec2::new(0.0, -1.0)));
    assert_eq!(inside.local_point(), Some(Vec2::new(0.0, -1.0)));
    assert_eq!(face.local_normal(), Some(Vec2::new(0.0, -1.0)));
    assert_eq!(face.local_point(), Some(Vec2::new(0.0, -1.0)));
    let vertex_normal = vertex
        .local_normal()
        .expect("vertex manifold should have a normal");
    assert!(vertex_normal.x > 0.0);
    assert!(vertex_normal.y < 0.0);
    assert_eq!(vertex.local_point(), Some(Vec2::new(1.0, -1.0)));
}

#[test]
fn polygon_polygon_tie_keeps_face_a_and_source_point_order() {
    // Arrange
    let polygon = PolygonShape::box_shape(1.0, 1.0).expect("box should be valid");
    let transform_b = Transform::from_position_angle(Vec2::new(0.0, 1.5), 0.0);

    // Act
    let manifold = collide_polygons(&polygon, Transform::IDENTITY, &polygon, transform_b)
        .expect("transforms should be finite")
        .expect("boxes should overlap");

    // Assert
    assert_eq!(manifold.kind(), Some(ManifoldKind::FaceA));
    assert_eq!(manifold.local_normal(), Some(Vec2::new(0.0, 1.0)));
    assert_eq!(manifold.points().len(), 2);
    assert_eq!(manifold.points()[0].local_point(), Vec2::new(-1.0, -1.0));
    assert_eq!(manifold.points()[1].local_point(), Vec2::new(1.0, -1.0));
    assert_eq!(manifold.points()[0].feature_id().index_b(), 0);
    assert_eq!(manifold.points()[1].feature_id().index_b(), 1);
}

#[test]
fn polygon_polygon_can_select_face_b_and_swap_feature_orientation() {
    // Arrange
    let polygon_a = PolygonShape::box_shape(1.0, 1.0).expect("box should be valid");
    let polygon_b =
        PolygonShape::oriented_box(1.5, 0.4, Vec2::ZERO, liquidfun::math::settings::TAU / 16.0)
            .expect("oriented box should be valid");
    let transform_b = Transform::from_position_angle(Vec2::new(0.0, 1.1), 0.0);

    // Act
    let manifold = collide_polygons(&polygon_a, Transform::IDENTITY, &polygon_b, transform_b)
        .expect("transforms should be finite")
        .expect("boxes should overlap");

    // Assert
    assert_eq!(manifold.kind(), Some(ManifoldKind::FaceB));
    assert!(
        manifold
            .points()
            .iter()
            .all(|point| point.feature_id().kind_b() == FeatureKind::Face)
    );
}

#[test]
fn world_manifold_circle_fallback_normal_points_from_a_to_b() {
    // Arrange
    let circle = CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid");
    let manifold = collide_circles(&circle, Transform::IDENTITY, &circle, Transform::IDENTITY)
        .expect("transforms should be finite")
        .expect("coincident circles should touch");

    // Act
    let world = world_manifold(
        &manifold,
        Transform::IDENTITY,
        circle.radius(),
        Transform::IDENTITY,
        circle.radius(),
    )
    .expect("world manifold should remain finite")
    .expect("active manifold should convert");

    // Assert
    assert_eq!(world.normal(), Vec2::new(1.0, 0.0));
    assert_eq!(world.points()[0].point(), Vec2::ZERO);
    assert_close(world.points()[0].separation(), -2.0);
}

#[test]
fn world_manifold_face_b_flips_normal_and_keeps_active_values() {
    // Arrange
    let manifold = Manifold::face_b(
        Vec2::new(0.0, -1.0),
        Vec2::new(0.0, 1.0),
        &[ManifoldPoint::new(Vec2::new(0.0, 1.5), feature(0, 0)).expect("point should be finite")],
    )
    .expect("face manifold should be valid");

    // Act
    let world = world_manifold(
        &manifold,
        Transform::IDENTITY,
        0.1,
        Transform::IDENTITY,
        0.2,
    )
    .expect("world manifold should remain finite")
    .expect("active manifold should convert");

    // Assert
    assert_eq!(world.normal(), Vec2::new(0.0, 1.0));
    assert_close(world.points()[0].point().y, 1.2);
    assert_close(world.points()[0].separation(), -0.8);
}

fn feature_vertex_vertex() -> ContactFeatureId {
    ContactFeatureId::new(0, 0, FeatureKind::Vertex, FeatureKind::Vertex)
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-6,
        "{actual} != {expected}"
    );
}

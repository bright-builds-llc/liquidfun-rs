//! Public contract tests for semantic manifolds and shape-pair dispatch.

use liquidfun::collision::narrow::point_states;
use liquidfun::collision::{ContactFeatureId, FeatureKind, Manifold, ManifoldPoint, PointState};
use liquidfun::math::Vec2;

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

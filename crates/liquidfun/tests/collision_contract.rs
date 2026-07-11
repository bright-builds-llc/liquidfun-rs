//! Public contract tests for initialized collision-domain values.

use liquidfun::collision::{
    Aabb, ChildIndex, CollisionError, CollisionOutcome, ContactFeatureId, FeatureKind, Manifold,
    ManifoldKind, ManifoldPoint, MassData, PointState, RayCastHit, RayCastInput,
};
use liquidfun::math::Vec2;

#[test]
fn aabb_constructs_from_finite_ordered_bounds() {
    // Arrange
    let lower_bound = Vec2::new(-2.0, -1.0);
    let upper_bound = Vec2::new(4.0, 3.0);

    // Act
    let aabb = Aabb::new(lower_bound, upper_bound).expect("ordered finite bounds should be valid");

    // Assert
    assert_eq!(aabb.lower_bound(), lower_bound);
    assert_eq!(aabb.upper_bound(), upper_bound);
    assert_eq!(aabb.center(), Vec2::new(1.0, 1.0));
    assert_eq!(aabb.extents(), Vec2::new(3.0, 2.0));
}

#[test]
fn aabb_rejects_non_finite_bounds() {
    // Arrange
    let non_finite = Vec2::new(f32::INFINITY, 0.0);
    let upper = Vec2::new(1.0, 1.0);

    // Act
    let result = Aabb::new(non_finite, upper);

    // Assert
    assert_eq!(result, Err(CollisionError::NonFiniteValue));
}

#[test]
fn aabb_rejects_reversed_bounds() {
    // Arrange
    let lower_bound = Vec2::new(2.0, 0.0);
    let upper_bound = Vec2::new(1.0, 1.0);

    // Act
    let result = Aabb::new(lower_bound, upper_bound);

    // Assert
    assert_eq!(result, Err(CollisionError::InvalidBounds));
}

#[test]
fn ray_input_rejects_non_finite_fraction() {
    // Arrange
    let start = Vec2::ZERO;
    let end = Vec2::new(2.0, 0.0);

    // Act
    let result = RayCastInput::new(start, end, f32::NAN);

    // Assert
    assert_eq!(result, Err(CollisionError::NonFiniteValue));
}

#[test]
fn ray_input_rejects_non_finite_endpoint() {
    // Arrange
    let start = Vec2::ZERO;
    let end = Vec2::new(f32::NEG_INFINITY, 0.0);

    // Act
    let result = RayCastInput::new(start, end, 1.0);

    // Assert
    assert_eq!(result, Err(CollisionError::NonFiniteValue));
}

#[test]
fn ray_input_rejects_out_of_range_fraction() {
    // Arrange
    let start = Vec2::ZERO;
    let end = Vec2::new(2.0, 0.0);

    // Act
    let result = RayCastInput::new(start, end, 1.25);

    // Assert
    assert_eq!(result, Err(CollisionError::FractionOutOfRange));
}

#[test]
fn mass_data_rejects_negative_properties() {
    // Arrange
    let negative_mass = -1.0;

    // Act
    let result = MassData::new(negative_mass, Vec2::ZERO, 0.0);

    // Assert
    assert_eq!(result, Err(CollisionError::InvalidGeometry));
}

#[test]
fn mass_data_exposes_only_initialized_values() {
    // Arrange
    let center = Vec2::new(1.0, -2.0);

    // Act
    let mass_data = MassData::new(3.0, center, 4.5).expect("mass data should be valid");

    // Assert
    assert_eq!(mass_data.mass().to_bits(), 3.0_f32.to_bits());
    assert_eq!(mass_data.center(), center);
    assert_eq!(mass_data.rotational_inertia().to_bits(), 4.5_f32.to_bits());
}

#[test]
fn ray_hit_rejects_out_of_range_fraction() {
    // Arrange
    let normal = Vec2::new(0.0, 1.0);

    // Act
    let result = RayCastHit::new(normal, -0.25);

    // Assert
    assert_eq!(result, Err(CollisionError::FractionOutOfRange));
}

#[test]
fn ray_hit_exposes_only_initialized_values() {
    // Arrange
    let normal = Vec2::new(0.0, 1.0);

    // Act
    let hit = RayCastHit::new(normal, 0.25).expect("ray hit should be valid");

    // Assert
    assert_eq!(hit.normal(), normal);
    assert_eq!(hit.fraction().to_bits(), 0.25_f32.to_bits());
}

#[test]
fn child_index_checks_the_public_shape_child_range() {
    // Arrange
    let child_count = 2;

    // Act
    let valid = ChildIndex::new(1, child_count).expect("second child should exist");
    let invalid = ChildIndex::new(2, child_count);

    // Assert
    assert_eq!(valid.get(), 1);
    assert_eq!(
        invalid,
        Err(CollisionError::ChildIndexOutOfRange {
            requested: 2,
            child_count,
        })
    );
}

#[test]
fn contact_feature_identity_uses_four_semantic_fields() {
    // Arrange
    let expected = ContactFeatureId::new(1, 2, FeatureKind::Vertex, FeatureKind::Face);

    // Act
    let equal = ContactFeatureId::new(1, 2, FeatureKind::Vertex, FeatureKind::Face);
    let reversed_kind = ContactFeatureId::new(1, 2, FeatureKind::Face, FeatureKind::Vertex);

    // Assert
    assert_eq!(expected, equal);
    assert_ne!(expected, reversed_kind);
    assert_eq!(expected.index_a(), 1);
    assert_eq!(expected.index_b(), 2);
    assert_eq!(expected.kind_a(), FeatureKind::Vertex);
    assert_eq!(expected.kind_b(), FeatureKind::Face);
}

#[test]
fn active_manifold_points_preserve_input_order() {
    // Arrange
    let first = ManifoldPoint::new(
        Vec2::new(-1.0, 0.0),
        ContactFeatureId::new(0, 1, FeatureKind::Face, FeatureKind::Vertex),
    )
    .expect("first point should be finite");
    let second = ManifoldPoint::new(
        Vec2::new(1.0, 0.0),
        ContactFeatureId::new(0, 2, FeatureKind::Face, FeatureKind::Vertex),
    )
    .expect("second point should be finite");

    // Act
    let manifold = Manifold::face_a(Vec2::new(0.0, 1.0), Vec2::ZERO, &[first, second])
        .expect("two initialized points should fit");

    // Assert
    assert_eq!(manifold.kind(), Some(ManifoldKind::FaceA));
    assert_eq!(manifold.points(), &[first, second]);
    assert_eq!(manifold.local_normal(), Some(Vec2::new(0.0, 1.0)));
    assert_eq!(manifold.local_point(), Some(Vec2::ZERO));
}

#[test]
fn face_manifold_rejects_more_than_two_points() {
    // Arrange
    let feature = ContactFeatureId::new(0, 0, FeatureKind::Face, FeatureKind::Vertex);
    let point = ManifoldPoint::new(Vec2::ZERO, feature).expect("origin should be finite");
    let points = [point, point, point];

    // Act
    let result = Manifold::face_b(Vec2::new(0.0, 1.0), Vec2::ZERO, &points);

    // Assert
    assert_eq!(result, Err(CollisionError::InvalidGeometry));
}

#[test]
fn circle_manifold_omits_inactive_normal() {
    // Arrange
    let point = ManifoldPoint::new(
        Vec2::new(1.0, 0.0),
        ContactFeatureId::new(0, 0, FeatureKind::Vertex, FeatureKind::Vertex),
    )
    .expect("circle point should be finite");

    // Act
    let manifold = Manifold::circles(Vec2::ZERO, point).expect("circle manifold should be valid");

    // Assert
    assert_eq!(manifold.kind(), Some(ManifoldKind::Circles));
    assert_eq!(manifold.local_normal(), None);
    assert_eq!(manifold.points(), &[point]);
}

#[test]
fn empty_manifold_exposes_no_inactive_payload() {
    // Arrange and Act
    let manifold = Manifold::empty();

    // Assert
    assert_eq!(manifold.kind(), None);
    assert_eq!(manifold.local_normal(), None);
    assert_eq!(manifold.local_point(), None);
    assert!(manifold.points().is_empty());
}

#[test]
fn collision_outcomes_distinguish_unsupported_separated_and_touching() {
    // Arrange
    let point = ManifoldPoint::new(
        Vec2::ZERO,
        ContactFeatureId::new(0, 0, FeatureKind::Vertex, FeatureKind::Vertex),
    )
    .expect("origin should be finite");
    let manifold = Manifold::circles(Vec2::ZERO, point).expect("circle manifold should be valid");

    // Act
    let unsupported: CollisionOutcome<Manifold> = CollisionOutcome::Unsupported;
    let separated: CollisionOutcome<Manifold> = CollisionOutcome::Separated;
    let touching = CollisionOutcome::Touching(manifold);

    // Assert
    assert!(matches!(unsupported, CollisionOutcome::Unsupported));
    assert!(matches!(separated, CollisionOutcome::Separated));
    assert!(matches!(touching, CollisionOutcome::Touching(_)));
}

#[test]
fn point_state_names_every_semantic_transition() {
    // Arrange and Act
    let states = [
        PointState::Null,
        PointState::Added,
        PointState::Persisted,
        PointState::Removed,
    ];

    // Assert
    assert_eq!(states.len(), 4);
}

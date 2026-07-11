//! Public contract tests for semantic manifolds and shape-pair dispatch.

use liquidfun::collision::narrow::{
    PairOrientation, collide_circles, collide_edge_circle, collide_edge_polygon,
    collide_polygon_circle, collide_polygons, collide_shapes, point_states, world_manifold,
};
use liquidfun::collision::shape::{ChainShape, CircleShape, EdgeShape, PolygonShape, Shape};
use liquidfun::collision::{
    ChildIndex, CollisionOutcome, ContactFeatureId, FeatureKind, Manifold, ManifoldKind,
    ManifoldPoint, PointState,
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

#[test]
fn edge_circle_preserves_endpoint_face_and_adjacency_ownership() {
    // Arrange
    let isolated =
        EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)).expect("edge should be valid");
    let adjacent = EdgeShape::with_adjacency(
        Vec2::ZERO,
        Vec2::new(1.0, 0.0),
        Some(Vec2::new(-1.0, 0.0)),
        None,
    )
    .expect("adjacent edge should be valid");
    let next_adjacent = EdgeShape::with_adjacency(
        Vec2::new(-1.0, 0.0),
        Vec2::ZERO,
        None,
        Some(Vec2::new(1.0, 0.0)),
    )
    .expect("adjacent edge should be valid");
    let circle = CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid");
    let endpoint_transform = Transform::from_position_angle(Vec2::new(-1.4, 0.0), 0.0);
    let face_transform = Transform::from_position_angle(Vec2::new(0.0, 0.5), 0.0);
    let previous_owner_transform = Transform::from_position_angle(Vec2::new(-0.1, 0.0), 0.0);
    let next_owner_transform = Transform::from_position_angle(Vec2::new(0.1, 0.0), 0.0);

    // Act
    let endpoint = collide_edge_circle(&isolated, Transform::IDENTITY, &circle, endpoint_transform)
        .expect("transforms should be finite")
        .expect("endpoint should touch");
    let face = collide_edge_circle(&isolated, Transform::IDENTITY, &circle, face_transform)
        .expect("transforms should be finite")
        .expect("face should touch");
    let rejected = collide_edge_circle(
        &adjacent,
        Transform::IDENTITY,
        &circle,
        previous_owner_transform,
    )
    .expect("transforms should be finite");
    let next_rejected = collide_edge_circle(
        &next_adjacent,
        Transform::IDENTITY,
        &circle,
        next_owner_transform,
    )
    .expect("transforms should be finite");

    // Assert
    assert_eq!(endpoint.kind(), Some(ManifoldKind::Circles));
    assert_eq!(endpoint.points()[0].feature_id().index_a(), 0);
    assert_eq!(face.kind(), Some(ManifoldKind::FaceA));
    assert_eq!(face.local_normal(), Some(Vec2::new(0.0, 1.0)));
    assert!(rejected.is_none());
    assert!(next_rejected.is_none());
}

#[test]
fn edge_polygon_handles_front_and_back_with_ordered_points() {
    // Arrange
    let edge =
        EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)).expect("edge should be valid");
    let polygon = PolygonShape::box_shape(0.4, 0.4).expect("box should be valid");
    let above = Transform::from_position_angle(Vec2::new(0.0, 0.3), 0.0);
    let below = Transform::from_position_angle(Vec2::new(0.0, -0.3), 0.0);

    // Act
    let back = collide_edge_polygon(&edge, Transform::IDENTITY, &polygon, above)
        .expect("transforms should be finite")
        .expect("back polygon should touch");
    let front = collide_edge_polygon(&edge, Transform::IDENTITY, &polygon, below)
        .expect("transforms should be finite")
        .expect("front polygon should touch");

    // Assert
    assert_eq!(back.kind(), Some(ManifoldKind::FaceA));
    assert_eq!(back.local_normal(), Some(Vec2::new(0.0, 1.0)));
    assert_eq!(front.kind(), Some(ManifoldKind::FaceA));
    assert_eq!(front.local_normal(), Some(Vec2::new(0.0, -1.0)));
    assert_eq!(back.points().len(), 2);
    assert!(back.points()[0].local_point().x < back.points()[1].local_point().x);
}

#[test]
fn edge_polygon_classifies_convex_and_concave_adjacency() {
    // Arrange
    let convex = EdgeShape::with_adjacency(
        Vec2::ZERO,
        Vec2::new(2.0, 0.0),
        Some(Vec2::new(-1.0, 1.0)),
        Some(Vec2::new(3.0, 1.0)),
    )
    .expect("convex adjacency should be valid");
    let concave = EdgeShape::with_adjacency(
        Vec2::ZERO,
        Vec2::new(2.0, 0.0),
        Some(Vec2::new(-1.0, -1.0)),
        Some(Vec2::new(3.0, -1.0)),
    )
    .expect("concave adjacency should be valid");
    let polygon = PolygonShape::box_shape(0.4, 0.4).expect("box should be valid");
    let transform = Transform::from_position_angle(Vec2::new(1.0, -0.3), 0.0);

    // Act
    let convex_manifold = collide_edge_polygon(&convex, Transform::IDENTITY, &polygon, transform)
        .expect("transforms should be finite");
    let concave_manifold = collide_edge_polygon(&concave, Transform::IDENTITY, &polygon, transform)
        .expect("transforms should be finite");

    // Assert
    assert!(convex_manifold.is_some());
    assert!(concave_manifold.is_some());
}

#[test]
fn edge_feature_transition_changes_from_endpoint_to_face() {
    // Arrange
    let edge =
        EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)).expect("edge should be valid");
    let circle = CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid");
    let endpoint_transform = Transform::from_position_angle(Vec2::new(-1.4, 0.0), 0.0);
    let face_transform = Transform::from_position_angle(Vec2::new(-0.9, 0.1), 0.0);
    let endpoint = collide_edge_circle(&edge, Transform::IDENTITY, &circle, endpoint_transform)
        .expect("transforms should be finite")
        .expect("endpoint should touch");
    let face = collide_edge_circle(&edge, Transform::IDENTITY, &circle, face_transform)
        .expect("transforms should be finite")
        .expect("face should touch");

    // Act
    let states = point_states(&endpoint, &face);

    // Assert
    assert_eq!(states.previous()[0], PointState::Removed);
    assert_eq!(states.current()[0], PointState::Added);
}

#[test]
fn pair_registry_supports_exactly_seven_primary_families_and_reversals() {
    // Arrange
    let circle: Shape = CircleShape::new(Vec2::ZERO, 0.5)
        .expect("circle should be valid")
        .into();
    let polygon: Shape = PolygonShape::box_shape(0.5, 0.5)
        .expect("box should be valid")
        .into();
    let edge: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let chain: Shape = ChainShape::open(&[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)], None, None)
        .expect("chain should be valid")
        .into();

    // Act
    let symmetric = [
        collide_pair(&circle, &circle),
        collide_pair(&polygon, &polygon),
    ];
    let asymmetric = [
        collide_pair_both_orders(&polygon, &circle),
        collide_pair_both_orders(&edge, &circle),
        collide_pair_both_orders(&edge, &polygon),
        collide_pair_both_orders(&chain, &circle),
        collide_pair_both_orders(&chain, &polygon),
    ];
    let unsupported = [
        collide_pair(&edge, &edge),
        collide_pair(&edge, &chain),
        collide_pair(&chain, &edge),
        collide_pair(&chain, &chain),
    ];

    // Assert
    assert!(symmetric.into_iter().all(|outcome| matches!(
        outcome,
        CollisionOutcome::Touching(ref pair)
            if pair.orientation() == PairOrientation::Primary
    )));
    for (primary, reversed) in asymmetric {
        let CollisionOutcome::Touching(primary_pair) = primary else {
            panic!("primary registered pair should touch");
        };
        let CollisionOutcome::Touching(reversed_pair) = reversed else {
            panic!("reversed registered pair should touch");
        };
        assert_eq!(primary_pair.orientation(), PairOrientation::Primary);
        assert_eq!(reversed_pair.orientation(), PairOrientation::Reversed);
        assert_eq!(primary_pair.manifold(), reversed_pair.manifold());
    }
    assert!(
        unsupported
            .into_iter()
            .all(|outcome| matches!(outcome, CollisionOutcome::Unsupported))
    );
}

#[test]
fn pair_registry_distinguishes_supported_separation_and_invalid_chain_child() {
    // Arrange
    let edge: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
        .expect("edge should be valid")
        .into();
    let far_circle: Shape = CircleShape::new(Vec2::new(0.0, 10.0), 0.5)
        .expect("circle should be valid")
        .into();
    let chain: Shape = ChainShape::open(&[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)], None, None)
        .expect("chain should be valid")
        .into();
    let valid_child = ChildIndex::new(0, 1).expect("child should exist");
    let foreign_child = ChildIndex::new(1, 2).expect("foreign child should exist");

    // Act
    let separated = collide_shapes(
        &edge,
        valid_child,
        Transform::IDENTITY,
        &far_circle,
        valid_child,
        Transform::IDENTITY,
    )
    .expect("pair should be valid");
    let invalid = collide_shapes(
        &chain,
        foreign_child,
        Transform::IDENTITY,
        &far_circle,
        valid_child,
        Transform::IDENTITY,
    );

    // Assert
    assert!(matches!(separated, CollisionOutcome::Separated));
    assert!(invalid.is_err());
}

#[test]
fn pair_registry_chain_child_delegates_to_exact_edge_kernel() {
    // Arrange
    let chain = ChainShape::open(
        &[
            Vec2::new(-1.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 1.0),
        ],
        None,
        None,
    )
    .expect("chain should be valid");
    let child = chain.child_index(0).expect("child should exist");
    let edge = chain.child_edge(child).expect("child edge should be valid");
    let circle = CircleShape::new(Vec2::new(0.0, 0.2), 0.5).expect("circle should be valid");
    let chain_shape: Shape = chain.into();
    let circle_shape: Shape = circle.clone().into();
    let circle_child = circle_shape.child_index(0).expect("child should exist");
    let direct = collide_edge_circle(&edge, Transform::IDENTITY, &circle, Transform::IDENTITY)
        .expect("transforms should be finite")
        .expect("edge and circle should touch");

    // Act
    let delegated = collide_shapes(
        &chain_shape,
        child,
        Transform::IDENTITY,
        &circle_shape,
        circle_child,
        Transform::IDENTITY,
    )
    .expect("chain pair should be valid");

    // Assert
    let CollisionOutcome::Touching(pair) = delegated else {
        panic!("chain child should touch");
    };
    assert_eq!(pair.orientation(), PairOrientation::Primary);
    assert_eq!(pair.manifold(), &direct);
}

fn collide_pair(
    shape_a: &Shape,
    shape_b: &Shape,
) -> CollisionOutcome<liquidfun::collision::narrow::PairManifold> {
    let child_a = shape_a.child_index(0).expect("shape should have a child");
    let child_b = shape_b.child_index(0).expect("shape should have a child");
    collide_shapes(
        shape_a,
        child_a,
        Transform::IDENTITY,
        shape_b,
        child_b,
        Transform::IDENTITY,
    )
    .expect("pair should be valid")
}

fn collide_pair_both_orders(
    primary_a: &Shape,
    primary_b: &Shape,
) -> (
    CollisionOutcome<liquidfun::collision::narrow::PairManifold>,
    CollisionOutcome<liquidfun::collision::narrow::PairManifold>,
) {
    (
        collide_pair(primary_a, primary_b),
        collide_pair(primary_b, primary_a),
    )
}

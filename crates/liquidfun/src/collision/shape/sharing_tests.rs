//! Immutable shape ownership and public query equivalence.

use super::*;

#[test]
fn polygon_clone_reuses_immutable_backing_without_changing_queries() {
    // Arrange
    let mut original = PolygonShape::box_shape(2.0, 1.0).expect("box");
    let expected_mass = original.compute_mass(3.0).expect("mass");
    let expected_distance = original
        .distance_to_point(Transform::IDENTITY, Vec2::new(3.0, 0.0))
        .expect("point");

    // Act
    let clone = original.clone();

    // Assert
    assert!(
        std::ptr::eq(original.vertices().as_ptr(), clone.vertices().as_ptr()),
        "clone should share checked vertices"
    );
    assert!(
        std::ptr::eq(original.normals().as_ptr(), clone.normals().as_ptr()),
        "clone should share checked normals"
    );
    assert_eq!(clone, original);
    original = PolygonShape::box_shape(1.0, 1.0).expect("replacement");
    assert_ne!(clone, original);
    assert_eq!(clone.compute_mass(3.0).expect("mass"), expected_mass);
    assert_eq!(
        clone
            .distance_to_point(Transform::IDENTITY, Vec2::new(3.0, 0.0))
            .expect("point"),
        expected_distance
    );
    assert_eq!(
        clone,
        PolygonShape::box_shape(2.0, 1.0).expect("independent box")
    );
}

#[test]
fn chain_clone_keeps_owned_ghosts_and_children_after_original_is_dropped() {
    // Arrange
    let original = ChainShape::open(
        &[Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)],
        Some(Vec2::new(-1.0, 0.0)),
        Some(Vec2::new(1.0, 2.0)),
    )
    .expect("chain");
    let child = original.child_index(0).expect("child");
    let expected = original.child_edge(child).expect("edge");

    // Act
    let clone = original.clone();

    // Assert
    assert!(
        std::ptr::eq(original.vertices().as_ptr(), clone.vertices().as_ptr()),
        "clone should share checked points"
    );
    assert_eq!(clone, original);
    drop(original);
    assert_eq!(clone.child_edge(child).expect("retained edge"), expected);
    assert!(!clone.is_closed());
}

#[test]
fn shapes_keep_send_sync_and_public_const_queries() {
    // Arrange / Act / Assert
    fn require_send_sync<T: Send + Sync>() {}
    const fn centroid(shape: &PolygonShape) -> Vec2 {
        shape.centroid()
    }
    const fn closed(shape: &ChainShape) -> bool {
        shape.is_closed()
    }
    require_send_sync::<PolygonShape>();
    require_send_sync::<ChainShape>();
    let polygon = PolygonShape::box_shape(1.0, 1.0).expect("box");
    assert_eq!(centroid(&polygon), Vec2::ZERO);
    let chain =
        ChainShape::closed(&[Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)]).expect("loop");
    assert!(closed(&chain));
}

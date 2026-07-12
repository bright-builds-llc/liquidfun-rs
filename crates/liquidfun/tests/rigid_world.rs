//! Black-box checks for handle-oriented rigid world state.

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyDefError, BodyTransformError, BodyType, HandleError, ObjectSnapshot, World,
};

fn body_definition(body_type: BodyType) -> BodyDef {
    BodyDef::new(body_type, Vec2::new(3.5, -2.25), -0.75, false)
        .expect("finite body definition should be accepted")
}

#[test]
fn body_creation_preserves_each_checked_definition() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body_types = [BodyType::Static, BodyType::Kinematic, BodyType::Dynamic];

    // Act
    let snapshots = body_types.map(|body_type| {
        let body = world
            .create_body(&body_definition(body_type))
            .expect("body should fit");
        world.body_snapshot(body).expect("body should remain live")
    });

    // Assert
    assert_eq!(
        snapshots.map(liquidfun::BodySnapshot::body_type),
        body_types
    );
    for snapshot in snapshots {
        assert_eq!(snapshot.position(), Vec2::new(3.5, -2.25));
        assert_eq!(snapshot.angle().to_bits(), (-0.75_f32).to_bits());
        assert!(!snapshot.is_active());
    }
}

#[test]
fn body_mutations_update_only_the_requested_semantic_state() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static))
        .expect("body should fit");

    // Act
    world
        .set_body_type(body, BodyType::Dynamic)
        .expect("body should remain live");
    world
        .set_body_transform(body, Vec2::new(-0.0, 4.25), -0.0)
        .expect("finite transform should be accepted");
    world
        .set_body_active(body, true)
        .expect("body should remain live");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.body_type(), BodyType::Dynamic);
    assert_eq!(snapshot.position().x.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(snapshot.position().y.to_bits(), 4.25_f32.to_bits());
    assert_eq!(snapshot.angle().to_bits(), (-0.0_f32).to_bits());
    assert!(snapshot.is_active());
}

#[test]
fn body_transform_rejection_preserves_prior_state() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Kinematic))
        .expect("body should fit");
    let before = world.body_snapshot(body).expect("body should remain live");

    // Act
    let result = world.set_body_transform(body, Vec2::new(f32::INFINITY, 7.0), 1.25);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(BodyTransformError::InvalidTransform(
            BodyDefError::NonFinitePositionX
        ))
    );
    assert_eq!(after, before);
}

#[test]
fn body_operations_reject_cross_world_and_stale_handles_without_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let survivor = world
        .create_body(&body_definition(BodyType::Dynamic))
        .expect("body should fit");
    let stale = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let replacement = world
        .create_body(&BodyDef::default())
        .expect("reused slot should fit");
    let mut other = World::new().expect("world key should remain available");
    let foreign = other
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let before = world
        .body_snapshot(survivor)
        .expect("survivor should remain live");

    // Act
    let stale_result = world.set_body_active(stale, true);
    let foreign_result = world.set_body_type(foreign, BodyType::Static);
    let after = world
        .body_snapshot(survivor)
        .expect("survivor should remain live");

    // Assert
    assert_eq!(stale_result, Err(HandleError::StaleOrDestroyed));
    assert_eq!(foreign_result, Err(HandleError::WrongWorld));
    assert_ne!(stale, replacement);
    assert_eq!(after, before);
}

#[test]
fn body_destruction_retains_semantic_state_before_immediate_invalidation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = body_definition(BodyType::Dynamic);
    let body = world.create_body(&definition).expect("body should fit");

    // Act
    let records = world.destroy_body(body).expect("body should be live");
    let lookup = world.body_snapshot(body);

    // Assert
    assert_eq!(lookup, Err(HandleError::StaleOrDestroyed));
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body { state, fixtures, joints })
            if *state == definition.snapshot() && fixtures.is_empty() && joints.is_empty()
    ));
}

//! Black-box checks for handle-oriented rigid world state.

use liquidfun::collision::FilterData;
use liquidfun::collision::shape::{CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyActivationError, BodyDef, BodyDefError, BodyTransformError, BodyType, BodyTypeChangeError,
    CreateObjectError, DestroyedId, FixtureDef, HandleError, ObjectSnapshot, World,
};

fn body_definition(body_type: BodyType) -> BodyDef {
    BodyDef::new(body_type, Vec2::new(3.5, -2.25), -0.75, false)
        .expect("finite body definition should be accepted")
}

fn fixture_definition() -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::new(-0.0, 1.25), 0.75).expect("circle should be valid"));
    FixtureDef::new(
        shape,
        f32::from_bits(0x3f80_0001),
        -0.0,
        0.5,
        true,
        FilterData::new(0x0004, 0x00f0, -3),
    )
    .expect("fixture definition should be valid")
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
    assert_eq!(
        stale_result,
        Err(BodyActivationError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(
        foreign_result,
        Err(BodyTypeChangeError::InvalidHandle(HandleError::WrongWorld))
    );
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

#[test]
fn fixture_creation_clones_definition_and_exposes_semantic_owner_state() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let definition = fixture_definition();
    let expected = definition.snapshot();

    // Act
    let fixture = world
        .create_fixture(body, &definition)
        .expect("fixture should fit");
    drop(definition);
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.body(), body);
    assert_eq!(snapshot.shape(), expected.shape());
    assert_eq!(snapshot.density().to_bits(), expected.density().to_bits());
    assert_eq!(snapshot.friction().to_bits(), expected.friction().to_bits());
    assert_eq!(
        snapshot.restitution().to_bits(),
        expected.restitution().to_bits()
    );
    assert_eq!(snapshot.is_sensor(), expected.is_sensor());
    assert_eq!(snapshot.filter_data(), expected.filter_data());
}

#[test]
fn fixture_creation_rejects_invalid_body_without_partial_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let local = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let stale = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let mut other = World::new().expect("world key should remain available");
    let foreign = other
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let definition = fixture_definition();

    // Act
    let stale_result = world.create_fixture(stale, &definition);
    let foreign_result = world.create_fixture(foreign, &definition);
    let records = world
        .destroy_body(local)
        .expect("local body should be live");

    // Assert
    assert_eq!(
        stale_result,
        Err(CreateObjectError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(
        foreign_result,
        Err(CreateObjectError::InvalidHandle(HandleError::WrongWorld))
    );
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].destroyed(), DestroyedId::Body(local));
}

#[test]
fn fixture_snapshot_rejects_stale_and_cross_world_handles() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let stale = world
        .create_fixture(body, &fixture_definition())
        .expect("fixture should fit");
    world
        .destroy_fixture(stale)
        .expect("fixture should be live");
    let replacement = world
        .create_fixture(body, &fixture_definition())
        .expect("reused slot should fit");
    let mut other = World::new().expect("world key should remain available");
    let other_body = other
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let foreign = other
        .create_fixture(other_body, &fixture_definition())
        .expect("fixture should fit");

    // Act
    let stale_result = world.fixture_snapshot(stale);
    let foreign_result = world.fixture_snapshot(foreign);

    // Assert
    assert_eq!(stale_result, Err(HandleError::StaleOrDestroyed));
    assert_eq!(foreign_result, Err(HandleError::WrongWorld));
    assert_ne!(stale, replacement);
    assert!(world.fixture_snapshot(replacement).is_ok());
}

#[test]
fn fixture_cascade_is_newest_first_and_retains_owned_semantic_state() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let first_definition = fixture_definition();
    let second_definition = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 2.0).expect("circle should be valid")),
        2.0,
        0.25,
        0.75,
        false,
        FilterData::default(),
    )
    .expect("fixture definition should be valid");
    let first = world
        .create_fixture(body, &first_definition)
        .expect("fixture should fit");
    let second = world
        .create_fixture(body, &second_definition)
        .expect("fixture should fit");

    // Act
    let records = world.destroy_body(body).expect("body should be live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::Fixture(second),
            DestroyedId::Fixture(first),
            DestroyedId::Body(body),
        ]
    );
    assert!(matches!(
        records.first().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Fixture {
            body: snapshot_body,
            state,
        }) if *snapshot_body == body
            && state.body() == body
            && state.shape() == second_definition.shape()
    ));
}

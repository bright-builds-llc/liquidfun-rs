//! Black-box checks for fixture broad-phase lifecycle and deferred side effects.

use liquidfun::collision::FilterData;
use liquidfun::collision::shape::{ChainShape, CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyActivationError, BodyDef, BodyTransformError, BodyType, FixtureBoundsError, FixtureDef,
    World,
};

fn body_definition(body_type: BodyType, active: bool) -> BodyDef {
    BodyDef::new(body_type, Vec2::ZERO, 0.0, active)
        .expect("finite body definition should be accepted")
}

fn circle_fixture() -> FixtureDef {
    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"));
    FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition should be valid")
}

fn far_circle_fixture() -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::new(f32::MAX, 0.0), 1.0).expect("circle should be valid"),
    );
    FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition should be valid")
}

#[test]
fn proxy_active_fixture_creation_tracks_each_shape_child() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("body should fit");

    // Act
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.broad_phase_entry_count(), 1);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_inactive_creation_and_activation_transitions_are_deferred() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, false))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    let initial = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, true)
        .expect("activation should create entries");
    let active = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, false)
        .expect("deactivation should remove entries");
    let inactive = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, true)
        .expect("reactivation should recreate entries");
    let reactivated = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(initial.broad_phase_entry_count(), 0);
    assert_eq!(active.broad_phase_entry_count(), 1);
    assert_eq!(inactive.broad_phase_entry_count(), 0);
    assert_eq!(reactivated.broad_phase_entry_count(), 1);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_chain_fixture_owns_one_entry_per_child() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("body should fit");
    let vertices = [
        Vec2::new(-2.0, 0.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 0.0),
    ];
    let chain = ChainShape::open(&vertices, None, None).expect("chain should be valid");
    let expected_children = chain.child_count();
    let definition = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture definition should be valid");

    // Act
    let fixture = world
        .create_fixture(body, &definition)
        .expect("fixture should fit");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.broad_phase_entry_count(), expected_children);
    assert_eq!(world.broad_phase_entry_count(), expected_children);
}

#[test]
fn proxy_transform_synchronizes_entries_without_creating_contacts() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    world
        .set_body_transform(body, Vec2::new(4.0, -3.0), 0.25)
        .expect("finite transform should synchronize entries");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.position(), Vec2::new(4.0, -3.0));
    assert_eq!(snapshot.angle().to_bits(), 0.25_f32.to_bits());
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_transform_overflow_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &far_circle_fixture())
        .expect("fixture should fit");
    let before = world.body_snapshot(body).expect("body should remain live");

    // Act
    let result = world.set_body_transform(body, Vec2::new(f32::MAX, 0.0), 0.0);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(BodyTransformError::InvalidFixtureBounds(
            FixtureBoundsError::NonFiniteDerivedBounds
        ))
    );
    assert_eq!(after, before);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_activation_overflow_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::new(f32::MAX, 0.0), 0.0, false)
        .expect("finite body definition should be accepted");
    let body = world.create_body(&definition).expect("body should fit");
    world
        .create_fixture(body, &far_circle_fixture())
        .expect("inactive fixture should not need entries");

    // Act
    let result = world.set_body_active(body, true);
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(BodyActivationError::InvalidFixtureBounds(
            FixtureBoundsError::NonFiniteDerivedBounds
        ))
    );
    assert!(!snapshot.is_active());
    assert_eq!(world.broad_phase_entry_count(), 0);
}

#[test]
fn proxy_type_change_preserves_entries_for_step_time_reconsideration() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    world
        .set_body_type(body, BodyType::Static)
        .expect("body should remain live");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.body_type(), BodyType::Static);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

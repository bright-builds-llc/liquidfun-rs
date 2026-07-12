//! Automatic rigid-contact lifecycle integration evidence.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, ContactTransitionKind, FixtureDef, FixtureId, StepHook, StepLimits,
    World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("test body definition should be valid")
}

fn fixture_definition(sensor: bool, friction: f32, restitution: f32) -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
    FixtureDef::new(
        shape,
        1.0,
        friction,
        restitution,
        sensor,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid")
}

fn touching_world(sensor: bool) -> (World, BodyId, FixtureId, FixtureId) {
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0)))
        .expect("dynamic body should fit");
    let static_fixture = world
        .create_fixture(static_body, &fixture_definition(sensor, 0.25, 0.1))
        .expect("static fixture should fit");
    let dynamic_fixture = world
        .create_fixture(dynamic_body, &fixture_definition(false, 1.0, 0.8))
        .expect("dynamic fixture should fit");
    (world, dynamic_body, static_fixture, dynamic_fixture)
}

#[test]
fn lifecycle_creates_persists_and_suppresses_duplicate_pairs() {
    // Arrange
    let (mut world, _dynamic_body, static_fixture, _dynamic_fixture) = touching_world(false);
    let mut hook = NoopHook;

    // Act
    let first = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("first automatic contact step should succeed");
    world
        .set_fixture_filter(static_fixture, FilterData::default())
        .expect("touching the fixture should succeed");
    let second = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("persistent automatic contact step should succeed");

    // Assert
    assert_eq!(world.contact_count(), 1);
    assert_eq!(first.contact_transitions().len(), 1);
    assert_eq!(
        first.contact_transitions()[0].kind(),
        ContactTransitionKind::Begin
    );
    assert_eq!(second.contact_transitions().len(), 1);
    assert_eq!(
        second.contact_transitions()[0].kind(),
        ContactTransitionKind::Persist
    );
    let first_contact = first.contact_transitions()[0].contact();
    let second_contact = second.contact_transitions()[0].contact();
    assert!(!first_contact.is_sensor());
    assert!(first_contact.maybe_manifold().is_some());
    assert_eq!(first_contact.points().len(), 1);
    assert_eq!(
        first_contact.points()[0].feature_id(),
        second_contact.points()[0].feature_id()
    );
    assert_eq!(first_contact.friction().to_bits(), 0.5_f32.to_bits());
    assert_eq!(first_contact.restitution().to_bits(), 0.8_f32.to_bits());
}

#[test]
fn lifecycle_material_mix_remains_authoritative_until_recreation() {
    // Arrange
    let (mut world, dynamic_body, static_fixture, _dynamic_fixture) = touching_world(false);
    let mut hook = NoopHook;
    let created = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("contact creation should succeed");

    // Act
    world
        .set_fixture_friction(static_fixture, 4.0)
        .expect("friction edit should be valid");
    world
        .set_fixture_restitution(static_fixture, 1.2)
        .expect("restitution edit should be valid");
    let persisted = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("existing contact should persist");
    world
        .set_body_active(dynamic_body, false)
        .expect("deactivation should succeed");
    world
        .set_body_active(dynamic_body, true)
        .expect("activation should succeed");
    let recreated = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("recreated contact should succeed on the next step");

    // Assert
    let created_contact = created.contact_transitions()[0].contact();
    let persisted_contact = persisted.contact_transitions()[0].contact();
    assert_eq!(
        persisted_contact.friction().to_bits(),
        created_contact.friction().to_bits()
    );
    assert_eq!(
        persisted_contact.restitution().to_bits(),
        created_contact.restitution().to_bits()
    );
    assert_eq!(
        recreated
            .contact_transitions()
            .iter()
            .map(liquidfun::ContactTransition::kind)
            .collect::<Vec<_>>(),
        vec![ContactTransitionKind::End, ContactTransitionKind::Begin]
    );
    let recreated_contact = recreated.contact_transitions()[1].contact();
    assert_eq!(recreated_contact.friction().to_bits(), 2.0_f32.to_bits());
    assert_eq!(recreated_contact.restitution().to_bits(), 1.2_f32.to_bits());
}

#[test]
fn sensor_lifecycle_uses_overlap_without_a_manifold() {
    // Arrange
    let (mut world, dynamic_body, _static_fixture, _dynamic_fixture) = touching_world(true);
    let mut hook = NoopHook;

    // Act
    let began = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("sensor begin should succeed");
    let persisted = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("sensor persistence should succeed");
    world
        .set_body_active(dynamic_body, false)
        .expect("sensor body deactivation should succeed");
    let ended = world
        .step(&[], &mut hook, StepLimits::default())
        .expect("sensor end evidence should succeed");

    // Assert
    assert_eq!(
        began.contact_transitions()[0].kind(),
        ContactTransitionKind::Begin
    );
    assert_eq!(
        persisted.contact_transitions()[0].kind(),
        ContactTransitionKind::Persist
    );
    assert_eq!(
        ended.contact_transitions()[0].kind(),
        ContactTransitionKind::End
    );
    for report in [&began, &persisted, &ended] {
        let contact = report.contact_transitions()[0].contact();
        assert!(contact.is_sensor());
        assert!(contact.maybe_manifold().is_none());
        assert!(contact.points().is_empty());
    }
}

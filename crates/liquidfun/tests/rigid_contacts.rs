//! Automatic rigid-contact lifecycle integration evidence.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, ContactTransitionKind, DestroyedId, FixtureDef, FixtureId,
    StepConfiguration, StepHook, StepLifecycleEvent, StepLimits, World, WorldCommand,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn phase6_step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test configuration should be valid")
}

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
fn non_dynamic_static_kinematic_overlap_is_rejected() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let kinematic_body = world
        .create_body(&body_definition(BodyType::Kinematic, Vec2::new(1.5, 0.0)))
        .expect("kinematic body should fit");
    world
        .create_fixture(static_body, &fixture_definition(false, 0.25, 0.1))
        .expect("static fixture should fit");
    world
        .create_fixture(kinematic_body, &fixture_definition(false, 1.0, 0.8))
        .expect("kinematic fixture should fit");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("non-dynamic overlap step should succeed");

    // Assert
    assert_eq!(world.contact_count(), 0);
    assert!(report.contact_transitions().is_empty());
    assert!(report.events().is_empty());
    assert!(report.contact_solves().is_empty());
}

#[test]
fn non_dynamic_kinematic_kinematic_overlap_is_rejected() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let first_body = world
        .create_body(&body_definition(BodyType::Kinematic, Vec2::ZERO))
        .expect("first kinematic body should fit");
    let second_body = world
        .create_body(&body_definition(BodyType::Kinematic, Vec2::new(1.5, 0.0)))
        .expect("second kinematic body should fit");
    world
        .create_fixture(first_body, &fixture_definition(false, 0.25, 0.1))
        .expect("first kinematic fixture should fit");
    world
        .create_fixture(second_body, &fixture_definition(false, 1.0, 0.8))
        .expect("second kinematic fixture should fit");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("non-dynamic overlap step should succeed");

    // Assert
    assert_eq!(world.contact_count(), 0);
    assert!(report.contact_transitions().is_empty());
    assert!(report.events().is_empty());
    assert!(report.contact_solves().is_empty());
}

#[test]
fn lifecycle_creates_persists_and_suppresses_duplicate_pairs() {
    // Arrange
    let (mut world, _dynamic_body, static_fixture, _dynamic_fixture) = touching_world(false);
    let mut hook = NoopHook;

    // Act
    let first = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("first automatic contact step should succeed");
    world
        .set_fixture_filter(static_fixture, FilterData::default())
        .expect("touching the fixture should succeed");
    let second = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
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
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("contact creation should succeed");

    // Act
    world
        .set_fixture_friction(static_fixture, 4.0)
        .expect("friction edit should be valid");
    world
        .set_fixture_restitution(static_fixture, 1.2)
        .expect("restitution edit should be valid");
    let persisted = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("existing contact should persist");
    world
        .set_body_active(dynamic_body, false)
        .expect("deactivation should succeed");
    world
        .set_body_active(dynamic_body, true)
        .expect("activation should succeed");
    let recreated = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
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
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("sensor begin should succeed");
    let persisted = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("sensor persistence should succeed");
    world
        .set_body_active(dynamic_body, false)
        .expect("sensor body deactivation should succeed");
    let ended = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
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

#[test]
fn filtering_is_deferred_and_reconsideration_recreates_the_contact() {
    // Arrange
    let (mut world, _dynamic_body, static_fixture, _dynamic_fixture) = touching_world(false);
    let mut hook = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("initial contact should begin");

    // Act
    world
        .set_fixture_filter(static_fixture, FilterData::new(0x0001, 0x0000, 0))
        .expect("filter edit should succeed");
    let count_before_update = world.contact_count();
    let rejected = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("deferred refilter should succeed");
    world
        .set_fixture_filter(static_fixture, FilterData::default())
        .expect("filter restoration should succeed");
    let reconsidered = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("touched pair should be reconsidered");

    // Assert
    assert_eq!(count_before_update, 1);
    assert_eq!(rejected.contact_transitions().len(), 1);
    assert_eq!(
        rejected.contact_transitions()[0].kind(),
        ContactTransitionKind::End
    );
    assert_eq!(reconsidered.contact_transitions().len(), 1);
    assert_eq!(
        reconsidered.contact_transitions()[0].kind(),
        ContactTransitionKind::Begin
    );
    assert_eq!(world.contact_count(), 1);
}

#[test]
fn destruction_deactivation_emits_end_and_activation_waits_for_step() {
    // Arrange
    let (mut world, dynamic_body, _static_fixture, _dynamic_fixture) = touching_world(false);
    let mut hook = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("initial contact should begin");

    // Act
    world
        .set_body_active(dynamic_body, false)
        .expect("deactivation should succeed");
    let after_deactivation = world.contact_count();
    world
        .set_body_active(dynamic_body, true)
        .expect("activation should succeed");
    let before_step = world.contact_count();
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("activation discovery step should succeed");

    // Assert
    assert_eq!(after_deactivation, 0);
    assert_eq!(before_step, 0);
    assert_eq!(
        report
            .contact_transitions()
            .iter()
            .map(liquidfun::ContactTransition::kind)
            .collect::<Vec<_>>(),
        vec![ContactTransitionKind::End, ContactTransitionKind::Begin]
    );
    assert_eq!(world.contact_count(), 1);
}

struct OneCommandHook {
    maybe_command: Option<WorldCommand>,
}

impl StepHook for OneCommandHook {
    fn command(&mut self, _contact: liquidfun::ContactView<'_>) -> Option<WorldCommand> {
        self.maybe_command.take()
    }
}

#[test]
fn destruction_fixture_uses_end_before_invalidation() {
    // Arrange
    let (mut world, _dynamic_body, static_fixture, _dynamic_fixture) = touching_world(false);
    let mut noop = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut noop,
            StepLimits::default(),
        )
        .expect("initial contact should begin");
    let mut hook = OneCommandHook {
        maybe_command: Some(WorldCommand::DestroyFixture(static_fixture)),
    };

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("fixture destruction command should succeed");

    // Assert
    assert!(matches!(
        report.lifecycle(),
        [
            StepLifecycleEvent::Contact(persist),
            StepLifecycleEvent::Hook(_),
            StepLifecycleEvent::Solve(_),
            StepLifecycleEvent::Command(_),
            StepLifecycleEvent::Contact(end),
            StepLifecycleEvent::Destruction(record),
        ] if persist.kind() == ContactTransitionKind::Persist
            && end.kind() == ContactTransitionKind::End
            && record.destroyed() == DestroyedId::Fixture(static_fixture)
    ));
    assert!(!world.contains_fixture(static_fixture));
    assert_eq!(world.contact_count(), 0);
}

#[test]
fn destruction_body_cascade_orders_end_before_fixture_before_body() {
    // Arrange
    let (mut world, dynamic_body, _static_fixture, dynamic_fixture) = touching_world(false);
    let mut noop = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut noop,
            StepLimits::default(),
        )
        .expect("initial contact should begin");
    let mut hook = OneCommandHook {
        maybe_command: Some(WorldCommand::DestroyBody(dynamic_body)),
    };

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("body destruction command should succeed");

    // Assert
    assert!(matches!(
        report.lifecycle(),
        [
            StepLifecycleEvent::Contact(persist),
            StepLifecycleEvent::Hook(_),
            StepLifecycleEvent::Solve(_),
            StepLifecycleEvent::Command(_),
            StepLifecycleEvent::Contact(end),
            StepLifecycleEvent::Destruction(fixture),
            StepLifecycleEvent::Destruction(body),
        ] if persist.kind() == ContactTransitionKind::Persist
            && end.kind() == ContactTransitionKind::End
            && fixture.destroyed() == DestroyedId::Fixture(dynamic_fixture)
            && body.destroyed() == DestroyedId::Body(dynamic_body)
    ));
    assert!(!world.contains_body(dynamic_body));
    assert!(!world.contains_fixture(dynamic_fixture));
    assert_eq!(world.contact_count(), 0);
}

#[test]
fn filtering_preserves_duplicate_event_multiplicity_in_manager_order() {
    // Arrange
    let (mut world, _dynamic_body, static_fixture, dynamic_fixture) = touching_world(false);
    let static_body = world
        .fixture_snapshot(static_fixture)
        .expect("static fixture should remain live")
        .body();
    world
        .set_fixture_sensor(static_fixture, true)
        .expect("first static fixture should become a sensor");
    let second_static_fixture = world
        .create_fixture(static_body, &fixture_definition(true, 0.25, 0.1))
        .expect("second static fixture should fit");
    let mut hook = NoopHook;
    let began = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("both contacts should begin");

    // Act
    world
        .set_fixture_filter(dynamic_fixture, FilterData::new(0x0001, 0x0000, 0))
        .expect("shared fixture refilter should succeed");
    let ended = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("both contacts should end");

    // Assert
    assert_eq!(began.contact_transitions().len(), 2);
    assert_eq!(ended.contact_transitions().len(), 2);
    assert_eq!(
        began
            .contact_transitions()
            .iter()
            .map(|transition| transition.contact().fixtures())
            .collect::<Vec<_>>(),
        ended
            .contact_transitions()
            .iter()
            .map(|transition| transition.contact().fixtures())
            .collect::<Vec<_>>()
    );
    assert!(began.contact_transitions().iter().all(|transition| {
        let fixtures = transition.contact().fixtures();
        transition.kind() == ContactTransitionKind::Begin
            && fixtures.contains(&dynamic_fixture)
            && fixtures
                .iter()
                .any(|fixture| [static_fixture, second_static_fixture].contains(fixture))
    }));
    assert!(
        ended
            .contact_transitions()
            .iter()
            .all(|transition| transition.kind() == ContactTransitionKind::End)
    );
}

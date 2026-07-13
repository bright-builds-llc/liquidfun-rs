//! Bounded Phase 6 rigid-contact solver evidence.

use liquidfun::collision::{CircleShape, FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, ContactView, FixtureDef, FixtureId, StepConfiguration, StepHook,
    StepLimits, StepPhase, World, WorldCommand,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn phase6_step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test configuration should be valid")
}

fn discrete_world() -> World {
    let mut world = World::new().expect("test world key should remain available");
    world
        .set_continuous_physics_enabled(false)
        .expect("test configuration should remain mutable");
    world
}

fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("test body definition should be valid")
}

fn circle_fixture(sensor: bool, friction: f32, restitution: f32) -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle shape should be valid"));
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

fn box_fixture() -> FixtureDef {
    let shape = Shape::from(
        PolygonShape::new(&[
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ])
        .expect("test polygon shape should be valid"),
    );
    FixtureDef::new(shape, 1.0, 0.5, 0.25, false, FilterData::default())
        .expect("test fixture definition should be valid")
}

fn circle_contact_world(
    first_type: BodyType,
    second_type: BodyType,
    sensor: bool,
) -> (World, BodyId, BodyId, FixtureId, FixtureId) {
    let mut world = discrete_world();
    let first_body = world
        .create_body(&body_definition(first_type, Vec2::ZERO))
        .expect("first body should fit");
    let second_body = world
        .create_body(&body_definition(second_type, Vec2::new(1.5, 0.0)))
        .expect("second body should fit");
    let first_fixture = world
        .create_fixture(first_body, &circle_fixture(sensor, 0.25, 0.1))
        .expect("first fixture should fit");
    let second_fixture = world
        .create_fixture(second_body, &circle_fixture(false, 1.0, 0.8))
        .expect("second fixture should fit");
    (
        world,
        first_body,
        second_body,
        first_fixture,
        second_fixture,
    )
}

#[test]
fn cold_contact_solves_with_finite_zero_impulses() {
    // Arrange
    let (mut world, _, _, _, _) = circle_contact_world(BodyType::Static, BodyType::Dynamic, false);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("one supported contact should solve");

    // Assert
    assert_eq!(report.contact_solves().len(), 1);
    let solved = report.contact_solves()[0].contact();
    assert_eq!(solved.points().len(), 1);
    assert_eq!(solved.points()[0].normal_impulse().to_bits(), 0);
    assert_eq!(solved.points()[0].tangent_impulse().to_bits(), 0);
    assert!(solved.points()[0].normal_impulse().is_finite());
    assert!(solved.points()[0].tangent_impulse().is_finite());
}

#[test]
fn contact_step_commits_source_ordered_position_correction() {
    // Arrange
    let mut world = discrete_world();
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(1.0, 0.0)))
        .expect("dynamic body should fit");
    world
        .create_fixture(static_body, &circle_fixture(false, 0.5, 0.125))
        .expect("static fixture should fit");
    world
        .create_fixture(dynamic_body, &circle_fixture(false, 0.25, 0.5))
        .expect("dynamic fixture should fit");
    let mut hook = NoopHook;

    // Act
    world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("one supported contact should solve");
    let snapshot = world
        .body_snapshot(dynamic_body)
        .expect("dynamic body should remain live");

    // Assert
    assert_eq!(snapshot.position().x.to_bits(), 0x3fbe_26d4);
    assert_eq!(snapshot.position().y.to_bits(), 0);
    assert_eq!(snapshot.angle().to_bits(), 0);
}

#[test]
fn persistent_feature_reuses_warm_start_lanes_in_source_order() {
    // Arrange
    let (mut world, _, _, _, _) = circle_contact_world(BodyType::Static, BodyType::Dynamic, false);
    let mut hook = NoopHook;
    let first = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("cold solve should succeed");

    // Act
    let second = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("warm solve should succeed");

    // Assert
    let first_point = first.contact_solves()[0].contact().points()[0];
    let second_point = second.contact_solves()[0].contact().points()[0];
    assert_eq!(first_point.feature_id(), second_point.feature_id());
    assert_eq!(
        first_point.normal_impulse().to_bits(),
        second_point.normal_impulse().to_bits()
    );
    assert_eq!(
        first_point.tangent_impulse().to_bits(),
        second_point.tangent_impulse().to_bits()
    );
}

#[test]
fn recreated_contact_starts_with_cold_impulse_lanes() {
    // Arrange
    let (mut world, _, dynamic_body, _, _) =
        circle_contact_world(BodyType::Static, BodyType::Dynamic, false);
    let mut hook = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("initial contact should solve");
    world
        .set_body_active(dynamic_body, false)
        .expect("deactivation should succeed");
    world
        .set_body_active(dynamic_body, true)
        .expect("reactivation should succeed");

    // Act
    let recreated = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("recreated contact should solve");

    // Assert
    let point = recreated.contact_solves()[0].contact().points()[0];
    assert_eq!(point.normal_impulse().to_bits(), 0);
    assert_eq!(point.tangent_impulse().to_bits(), 0);
}

#[test]
fn sensor_contact_bypasses_constraint_creation() {
    // Arrange
    let (mut world, _, _, _, _) = circle_contact_world(BodyType::Static, BodyType::Dynamic, true);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("sensor lifecycle should succeed");

    // Assert
    assert!(report.contact_solves().is_empty());
    assert!(report.contact_transitions()[0].contact().is_sensor());
    assert!(
        report.contact_transitions()[0]
            .contact()
            .points()
            .is_empty()
    );
}

#[test]
fn two_point_contact_uses_fixed_capacity_and_preserves_material() {
    // Arrange
    let mut world = discrete_world();
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(0.0, 1.5)))
        .expect("dynamic body should fit");
    world
        .create_fixture(static_body, &box_fixture())
        .expect("static fixture should fit");
    world
        .create_fixture(dynamic_body, &box_fixture())
        .expect("dynamic fixture should fit");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("two-point contact should solve");

    // Assert
    let solved = report.contact_solves()[0].contact();
    assert_eq!(solved.points().len(), 2);
    assert_eq!(solved.friction().to_bits(), 0.5_f32.to_bits());
    assert_eq!(solved.restitution().to_bits(), 0.25_f32.to_bits());
    assert!(solved.points().iter().all(|point| {
        point.normal_impulse().is_finite() && point.tangent_impulse().is_finite()
    }));
}

#[test]
fn dynamic_pair_solves_as_one_source_ordered_island() {
    // Arrange
    let (mut world, first_body, second_body, _, _) =
        circle_contact_world(BodyType::Dynamic, BodyType::Dynamic, false);
    let first_before = world
        .body_snapshot(first_body)
        .expect("first body should be live");
    let second_before = world
        .body_snapshot(second_body)
        .expect("second body should be live");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("dynamic/dynamic contact should solve in Phase 7");
    let first_after = world
        .body_snapshot(first_body)
        .expect("first body should remain live");
    let second_after = world
        .body_snapshot(second_body)
        .expect("second body should remain live");

    // Assert
    assert_eq!(world.contact_count(), 1);
    assert_ne!(first_before.position(), first_after.position());
    assert_ne!(second_before.position(), second_after.position());
    assert_eq!(report.contact_transitions().len(), 1);
    assert_eq!(report.contact_solves().len(), 1);
    assert!(
        report.contact_solves()[0]
            .contact()
            .points()
            .iter()
            .all(|point| {
                point.normal_impulse().to_bits() == 0 && point.tangent_impulse().to_bits() == 0
            })
    );
}

struct DestroyAfterSolveHook {
    target: Option<BodyId>,
    pre_solve_calls: usize,
}

impl StepHook for DestroyAfterSolveHook {
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> liquidfun::PreSolveDirective {
        self.pre_solve_calls += 1;
        liquidfun::PreSolveDirective::Enable
    }

    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        self.target.take().map(WorldCommand::DestroyBody)
    }
}

#[test]
fn step_order_discovers_hooks_solves_unlocks_and_applies_commands() {
    // Arrange
    let (mut world, _, _, _, _) = circle_contact_world(BodyType::Static, BodyType::Dynamic, false);
    let command_body = world
        .create_body(&BodyDef::default())
        .expect("command body should fit");
    let mut hook = DestroyAfterSolveHook {
        target: Some(command_body),
        pre_solve_calls: 0,
    };

    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("supported automatic step should succeed");

    // Assert
    assert_eq!(
        report.phases(),
        &[
            StepPhase::FindPairs,
            StepPhase::UpdateContacts,
            StepPhase::Hook,
            StepPhase::Solve,
            StepPhase::Unlock,
            StepPhase::ApplyCommands,
        ]
    );
    assert_eq!(hook.pre_solve_calls, 1);
    assert_eq!(report.events().len(), 1);
    assert_eq!(report.contact_solves().len(), 1);
    assert_eq!(report.command_applications().len(), 1);
    assert!(!world.contains_body(command_body));
    assert!(!world.is_locked());
}

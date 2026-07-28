use crate::collision::{CircleShape, FilterData};
use crate::{BodyDef, BodyType, FixtureDef, StepConfiguration, StepHook, StepLimits, World};

use super::*;

struct NoopHook;

impl StepHook for NoopHook {}

fn phase6_step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test configuration should be valid")
}

fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("test body should be valid")
}

fn fixture_definition() -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
    FixtureDef::new(shape, 1.0, 0.5, 0.25, false, FilterData::default())
        .expect("test fixture should be valid")
}

#[test]
fn multi_contact_island_solves_all_manager_occurrences() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let first_dynamic = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0)))
        .expect("first dynamic body should fit");
    let static_fixture = world
        .create_fixture(static_body, &fixture_definition())
        .expect("static fixture should fit");
    let first_dynamic_fixture = world
        .create_fixture(first_dynamic, &fixture_definition())
        .expect("first dynamic fixture should fit");
    let mut hook = NoopHook;
    world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("one supported contact should solve");
    world.seed_first_contact_impulses_for_test(2.0, -0.5);
    world.set_body_solver_velocity_for_test(first_dynamic, Vec2::new(3.0, -4.0), 0.75);
    let second_dynamic = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(-1.5, 0.0)))
        .expect("second dynamic body should fit");
    world
        .create_fixture(second_dynamic, &fixture_definition())
        .expect("second dynamic fixture should fit");
    world.set_body_solver_velocity_for_test(second_dynamic, Vec2::new(-2.0, 5.0), -0.25);
    // Act
    let report = world
        .step(
            phase6_step_configuration(),
            &mut hook,
            StepLimits::default(),
        )
        .expect("multi-contact topology should solve as one island");

    // Assert
    assert_eq!(report.contact_solves().len(), 2);
    assert!(
        report
            .contact_solves()
            .iter()
            .any(|solve| solve.contact().fixtures() == [static_fixture, first_dynamic_fixture])
    );
    for body in [first_dynamic, second_dynamic] {
        let (linear, angular) = world.body_solver_velocity_for_test(body);
        assert!(linear.is_valid());
        assert!(angular.is_finite());
    }
}

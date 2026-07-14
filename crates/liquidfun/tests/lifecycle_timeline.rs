//! Source-ordered owned lifecycle evidence for one rigid-world step.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, CollisionDecisionHook, CollisionDirective, ContactView, FixtureDef,
    FixturePairView, LifecycleEvent, NoDecisionHook, StepConfiguration, StepError, StepHook,
    StepLimits, World, WorldCommand,
};

fn configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test step should be valid")
}

fn two_sensor_world() -> World {
    let mut world = World::new().expect("world key should remain available");
    let static_body = world
        .create_body(&BodyDef::default())
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit");
    let sensor = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        0.0,
        0.2,
        0.0,
        true,
        FilterData::default(),
    )
    .expect("sensor should be valid");
    let solid = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("solid fixture should be valid");
    for _ in 0..2 {
        world
            .create_fixture(static_body, &sensor)
            .expect("sensor should fit");
    }
    world
        .create_fixture(dynamic_body, &solid)
        .expect("solid fixture should fit");
    world
}

fn touching_world() -> World {
    let mut world = World::new().expect("world key should remain available");
    let static_body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("static body should be valid"),
        )
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture should be valid");
    world
        .create_fixture(static_body, &fixture)
        .expect("static fixture should fit");
    world
        .create_fixture(dynamic_body, &fixture)
        .expect("dynamic fixture should fit");
    world
}

#[derive(Default)]
struct RecordingFilter;

impl CollisionDecisionHook for RecordingFilter {
    fn should_collide(&mut self, _pair: FixturePairView<'_>) -> CollisionDirective {
        CollisionDirective::Collide
    }
}

#[test]
fn filter_transition_hook_and_discrete_solve_follow_effect_order() {
    // Arrange
    let mut world = touching_world();
    let mut hook = RecordingFilter;

    // Act
    let report = world
        .step(configuration(), &mut hook, StepLimits::default())
        .expect("touching world should step");

    // Assert
    assert!(
        matches!(
            report.lifecycle(),
            [
                LifecycleEvent::Filter(filter),
                LifecycleEvent::Contact(begin),
                LifecycleEvent::Hook(_),
                LifecycleEvent::Solve(_),
                LifecycleEvent::Contact(persist),
                LifecycleEvent::Hook(_),
                LifecycleEvent::ContinuousSolve(_),
            ] if filter.decision() == CollisionDirective::Collide
                && begin.kind() == liquidfun::ContactTransitionKind::Begin
                && persist.kind() == liquidfun::ContactTransitionKind::Persist
        ),
        "{:#?}",
        report.lifecycle()
    );
    assert_eq!(report.events().len(), 2);
    assert_eq!(report.contact_transitions().len(), 2);
    assert_eq!(report.contact_solves().len(), 1);
    assert_eq!(report.continuous_contact_solves().len(), 1);
}

#[test]
fn convenience_views_are_stable_projections_of_the_timeline() {
    // Arrange
    let mut world = touching_world();

    // Act
    let report = world
        .step(configuration(), &mut NoDecisionHook, StepLimits::default())
        .expect("touching world should step");
    let projected_contacts = report
        .lifecycle()
        .iter()
        .filter(|event| matches!(event, LifecycleEvent::Contact(_)))
        .count();
    let projected_hooks = report
        .lifecycle()
        .iter()
        .filter(|event| matches!(event, LifecycleEvent::Hook(_)))
        .count();
    let projected_solves = report
        .lifecycle()
        .iter()
        .filter(|event| matches!(event, LifecycleEvent::Solve(_)))
        .count();

    // Assert
    assert_eq!(report.contact_transitions().len(), projected_contacts);
    assert_eq!(report.events().len(), projected_hooks);
    assert_eq!(report.contact_solves().len(), projected_solves);
}

struct CommandQueue {
    commands: VecDeque<WorldCommand>,
}

impl StepHook for CommandQueue {
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        self.commands.pop_front()
    }
}

#[test]
fn recoverable_invalid_command_does_not_hide_later_application() {
    // Arrange
    let mut world = two_sensor_world();
    let stale = world
        .create_body(&BodyDef::default())
        .expect("stale body should fit");
    world
        .destroy_body(stale)
        .expect("stale body should be live");
    let live = world
        .create_body(&BodyDef::default())
        .expect("live body should fit");
    let mut hook = CommandQueue {
        commands: [
            WorldCommand::DestroyBody(stale),
            WorldCommand::DestroyBody(live),
        ]
        .into(),
    };

    // Act
    let report = world
        .step(configuration(), &mut hook, StepLimits::default())
        .expect("recoverable command failure should not stop the queue");
    let commands = report
        .lifecycle()
        .iter()
        .filter_map(|event| match event {
            LifecycleEvent::Command(application) => Some(application),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].command(), WorldCommand::DestroyBody(stale));
    assert!(commands[0].result().is_err());
    assert_eq!(commands[1].command(), WorldCommand::DestroyBody(live));
    assert!(commands[1].result().is_ok());
    assert!(!world.contains_body(live));
}

struct PanickingHook;

impl StepHook for PanickingHook {
    fn command(&mut self, contact: ContactView<'_>) -> Option<WorldCommand> {
        Some(WorldCommand::DestroyBody(contact.bodies()[0]))
    }

    fn observe(&mut self, _contact: ContactView<'_>) {
        panic!("intentional lifecycle hook panic");
    }
}

#[test]
fn hook_panic_restores_lock_discards_commands_and_poisons_world() {
    // Arrange
    let mut world = touching_world();

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _result = world.step(configuration(), &mut PanickingHook, StepLimits::default());
    }));
    let repeated = world.step(configuration(), &mut NoDecisionHook, StepLimits::default());

    // Assert
    assert!(panic.is_err());
    assert!(!world.is_locked());
    assert!(world.is_poisoned());
    assert_eq!(repeated, Err(StepError::Poisoned));
    assert_eq!(world.contact_count(), 1);
}

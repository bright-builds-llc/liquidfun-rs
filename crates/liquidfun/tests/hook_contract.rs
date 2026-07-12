//! Black-box evidence for manager-owned restricted hooks and deferred mutation.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    AggregateMassError, BodyDef, BodyType, CollisionDirective, CommandError, ContactView,
    FixtureDef, HandleError, PreSolveDirective, StepError, StepHook, StepLimits, World,
    WorldCommand,
};

fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("test body definition should be valid")
}

fn fixture_definition(sensor: bool) -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("test circle shape should be valid"));
    FixtureDef::new(shape, 1.0, 0.2, 0.0, sensor, FilterData::default())
        .expect("test fixture definition should be valid")
}

fn touching_world(sensor: bool) -> World {
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(0.75, 0.0)))
        .expect("dynamic body should fit");
    world
        .create_fixture(static_body, &fixture_definition(sensor))
        .expect("static fixture should fit");
    world
        .create_fixture(dynamic_body, &fixture_definition(false))
        .expect("dynamic fixture should fit");
    world
}

fn two_sensor_occurrence_world() -> World {
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, Vec2::ZERO))
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(0.75, 0.0)))
        .expect("dynamic body should fit");
    for _ in 0..2 {
        world
            .create_fixture(static_body, &fixture_definition(true))
            .expect("sensor fixture should fit");
    }
    world
        .create_fixture(dynamic_body, &fixture_definition(false))
        .expect("dynamic fixture should fit");
    world
}

#[derive(Default)]
struct RecordingHook {
    fixtures: Vec<[liquidfun::FixtureId; 2]>,
    point_counts: Vec<usize>,
}

impl StepHook for RecordingHook {
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Disable
    }

    fn observe(&mut self, contact: ContactView<'_>) {
        self.fixtures.push(contact.fixtures());
        self.point_counts.push(contact.points().len());
        assert!(contact.is_touching());
        assert!(contact.maybe_manifold().is_some());
        assert!(contact.friction().is_finite());
        assert!(contact.restitution().is_finite());
    }
}

#[test]
fn contact_view_owns_semantic_state_without_durable_identity() {
    // Arrange
    let mut world = touching_world(false);
    let mut hook = RecordingHook::default();

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("supported contact hooks should succeed");

    // Assert
    assert_eq!(report.events().len(), 1);
    assert_eq!(hook.fixtures.len(), 1);
    assert_eq!(hook.point_counts, vec![1]);
    assert_eq!(report.events()[0].collision(), CollisionDirective::Collide);
    assert_eq!(
        report.events()[0].maybe_pre_solve(),
        Some(PreSolveDirective::Disable)
    );
    assert!(report.contact_solves().is_empty());
}

#[derive(Default)]
struct SensorHook {
    pre_solve_calls: usize,
    observe_calls: usize,
}

impl StepHook for SensorHook {
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        self.pre_solve_calls += 1;
        PreSolveDirective::Enable
    }

    fn observe(&mut self, contact: ContactView<'_>) {
        self.observe_calls += 1;
        assert!(contact.is_sensor());
        assert!(contact.maybe_manifold().is_none());
        assert!(contact.points().is_empty());
    }
}

#[test]
fn sensor_occurrence_skips_pre_solve_and_constraint_creation() {
    // Arrange
    let mut world = touching_world(true);
    let mut hook = SensorHook::default();

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("sensor hook should succeed");

    // Assert
    assert_eq!(hook.pre_solve_calls, 0);
    assert_eq!(hook.observe_calls, 1);
    assert_eq!(report.events()[0].maybe_pre_solve(), None);
    assert!(report.contact_solves().is_empty());
}

#[test]
fn manager_occurrence_multiplicity_is_not_deduplicated() {
    // Arrange
    let mut world = two_sensor_occurrence_world();
    let mut hook = SensorHook::default();

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("two sensor occurrences should remain bounded");

    // Assert
    assert_eq!(world.contact_count(), 2);
    assert_eq!(hook.observe_calls, 2);
    assert_eq!(report.events().len(), 2);
    assert_eq!(report.contact_transitions().len(), 2);
}

struct CommandHook {
    commands: VecDeque<WorldCommand>,
}

impl StepHook for CommandHook {
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        self.commands.pop_front()
    }
}

#[test]
fn deferred_commands_apply_after_unlock_in_occurrence_order() {
    // Arrange
    let mut world = two_sensor_occurrence_world();
    let first = world
        .create_body(&BodyDef::default())
        .expect("first command body should fit");
    let second = world
        .create_body(&BodyDef::default())
        .expect("second command body should fit");
    let mut hook = CommandHook {
        commands: [
            WorldCommand::DestroyBody(first),
            WorldCommand::DestroyBody(second),
        ]
        .into(),
    };

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("queued commands should apply after unlock");

    // Assert
    assert!(!world.is_locked());
    assert_eq!(report.command_applications().len(), 2);
    assert_eq!(
        report.command_applications()[0].command(),
        WorldCommand::DestroyBody(first)
    );
    assert_eq!(
        report.command_applications()[1].command(),
        WorldCommand::DestroyBody(second)
    );
    assert!(report.command_applications()[0].result().is_ok());
    assert!(report.command_applications()[1].result().is_ok());
}

#[test]
fn stale_command_does_not_hide_later_success() {
    // Arrange
    let mut world = two_sensor_occurrence_world();
    let stale = world
        .create_body(&BodyDef::default())
        .expect("stale command body should fit");
    world.destroy_body(stale).expect("body should be live");
    let live = world
        .create_body(&BodyDef::default())
        .expect("live command body should fit");
    let mut hook = CommandHook {
        commands: [
            WorldCommand::DestroyBody(stale),
            WorldCommand::DestroyBody(live),
        ]
        .into(),
    };

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("stale commands are recoverable per occurrence");

    // Assert
    assert_eq!(
        report.command_applications()[0].result(),
        Err(CommandError::InvalidHandle(HandleError::StaleOrDestroyed))
    );
    assert!(report.command_applications()[1].result().is_ok());
    assert!(!world.contains_body(live));
}

#[test]
fn aggregate_mass_command_error_does_not_hide_later_success() {
    // Arrange
    let mut world = two_sensor_occurrence_world();
    let target_body = world
        .create_body(&body_definition(BodyType::Dynamic, Vec2::new(100.0, 0.0)))
        .expect("target body should fit");
    let high_density_fixture = || {
        let shape = Shape::from(
            CircleShape::new(Vec2::ZERO, 1.0).expect("high-density circle should be valid"),
        );
        FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
            .expect("high-density fixture definition should be valid")
    };
    let first = world
        .create_fixture(target_body, &high_density_fixture())
        .expect("first fixture should fit");
    let second = world
        .create_fixture(target_body, &high_density_fixture())
        .expect("second fixture should fit");
    let target = world
        .create_fixture(target_body, &high_density_fixture())
        .expect("target fixture should fit");
    for fixture in [first, second, target] {
        world
            .set_fixture_density(fixture, f32::MAX / 4.0)
            .expect("individual fixture mass should remain finite");
    }
    let live = world
        .create_body(&BodyDef::default())
        .expect("later command body should fit");
    let mut hook = CommandHook {
        commands: [
            WorldCommand::DestroyFixture(target),
            WorldCommand::DestroyBody(live),
        ]
        .into(),
    };

    // Act
    let report = world
        .step(&mut hook, StepLimits::default())
        .expect("aggregate command rejection should be recoverable");

    // Assert
    assert_eq!(
        report.command_applications()[0].result(),
        Err(CommandError::InvalidAggregateMass(
            AggregateMassError::NonFiniteMass
        ))
    );
    assert!(world.contains_fixture(target));
    assert!(report.command_applications()[1].result().is_ok());
    assert!(!world.contains_body(live));
}

#[test]
fn event_overflow_discards_queued_commands() {
    // Arrange
    let mut world = two_sensor_occurrence_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("command body should fit");
    let mut hook = CommandHook {
        commands: [WorldCommand::DestroyBody(body)].into(),
    };
    let limits = StepLimits::new(1, 1).expect("limits should be below hard maxima");

    // Act
    let result = world.step(&mut hook, limits);

    // Assert
    assert_eq!(
        result,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 1,
        })
    );
    assert!(world.contains_body(body));
    assert!(!world.is_locked());
}

struct PanickingHook;

impl StepHook for PanickingHook {
    fn observe(&mut self, _contact: ContactView<'_>) {
        panic!("intentional consumer hook panic");
    }
}

#[test]
fn hook_panic_restores_lock_discards_commands_and_poisons_world() {
    // Arrange
    let mut world = touching_world(false);
    let body = world
        .create_body(&BodyDef::default())
        .expect("survivor body should fit");
    let mut hook = PanickingHook;

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _result = world.step(&mut hook, StepLimits::default());
    }));

    // Assert
    assert!(panic.is_err());
    assert!(!world.is_locked());
    assert!(world.is_poisoned());
    assert!(world.contains_body(body));
    assert_eq!(world.destroy_body(body), Err(HandleError::WorldPoisoned));
    assert_eq!(
        world.step(&mut hook, StepLimits::default()),
        Err(StepError::Poisoned)
    );
}

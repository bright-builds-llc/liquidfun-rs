//! Retry evidence for transactional ordinary hook-limit failures.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, CollisionDirective, CommandError, ContactView, FixtureDef, NoDecisionHook,
    PreSolveDirective, StepCompletion, StepConfiguration, StepError, StepHook, StepLifecycleEvent,
    StepLimits, StepPhase, StepReport, World, WorldCommand,
};

fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed step should be valid")
}

fn two_sensor_world() -> World {
    let mut world = World::new().expect("world key should remain available");
    let static_body = world
        .create_body(&BodyDef::default())
        .expect("static body should fit");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body definition should be valid"),
        )
        .expect("dynamic body should fit");
    let shape = || Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid"));
    for _ in 0..2 {
        let definition = FixtureDef::new(shape(), 1.0, 0.2, 0.0, true, FilterData::default())
            .expect("sensor fixture should be valid");
        world
            .create_fixture(static_body, &definition)
            .expect("sensor fixture should fit");
    }
    let definition = FixtureDef::new(shape(), 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("dynamic fixture should be valid");
    world
        .create_fixture(dynamic_body, &definition)
        .expect("dynamic fixture should fit");
    world
}

#[derive(Debug, PartialEq)]
struct StepSemantics {
    contact_count: usize,
    completion: StepCompletion,
    time_step_ratio_bits: u32,
    phases: Vec<StepPhase>,
    lifecycle: Vec<LifecycleSemantics>,
    transitions: Vec<(liquidfun::ContactTransitionKind, bool, bool, usize)>,
    events: Vec<(
        CollisionDirective,
        Option<PreSolveDirective>,
        bool,
        bool,
        usize,
    )>,
    solves: Vec<(bool, bool, usize)>,
    command_results: Vec<Result<usize, CommandError>>,
}

#[derive(Debug, PartialEq)]
enum LifecycleSemantics {
    Filter(CollisionDirective),
    Contact(liquidfun::ContactTransitionKind),
    ContactDestruction(liquidfun::ContactTransitionKind),
    Hook,
    Solve,
    ContinuousSolve,
    JointGoodbye,
    FixtureGoodbye,
    Command,
    Destruction,
    Unknown,
}

fn step_semantics(world: &World, report: &StepReport) -> StepSemantics {
    StepSemantics {
        contact_count: world.contact_count(),
        completion: report.completion(),
        time_step_ratio_bits: report.time_step_ratio().to_bits(),
        phases: report.phases().to_vec(),
        lifecycle: report
            .lifecycle()
            .iter()
            .map(|event| match event {
                StepLifecycleEvent::Filter(event) => LifecycleSemantics::Filter(event.decision()),
                StepLifecycleEvent::Contact(value) => LifecycleSemantics::Contact(value.kind()),
                StepLifecycleEvent::ContactDestruction(value) => {
                    LifecycleSemantics::ContactDestruction(value.kind())
                }
                StepLifecycleEvent::Hook(_) => LifecycleSemantics::Hook,
                StepLifecycleEvent::Solve(_) => LifecycleSemantics::Solve,
                StepLifecycleEvent::ContinuousSolve(_) => LifecycleSemantics::ContinuousSolve,
                StepLifecycleEvent::JointGoodbye(_) => LifecycleSemantics::JointGoodbye,
                StepLifecycleEvent::FixtureGoodbye(_) => LifecycleSemantics::FixtureGoodbye,
                StepLifecycleEvent::Command(_) => LifecycleSemantics::Command,
                StepLifecycleEvent::Destruction(_) => LifecycleSemantics::Destruction,
                _ => LifecycleSemantics::Unknown,
            })
            .collect(),
        transitions: report
            .contact_transitions()
            .iter()
            .map(|value| {
                let contact = value.contact();
                (
                    value.kind(),
                    contact.is_touching(),
                    contact.is_sensor(),
                    contact.points().len(),
                )
            })
            .collect(),
        events: report
            .events()
            .iter()
            .map(|value| {
                let contact = value.contact();
                (
                    value.collision(),
                    value.maybe_pre_solve(),
                    contact.is_touching(),
                    contact.is_sensor(),
                    contact.points().len(),
                )
            })
            .collect(),
        solves: report
            .contact_solves()
            .iter()
            .map(|value| {
                let contact = value.contact();
                (
                    contact.is_touching(),
                    contact.is_sensor(),
                    contact.points().len(),
                )
            })
            .collect(),
        command_results: report
            .command_applications()
            .iter()
            .map(|value| value.result().map(<[_]>::len))
            .collect(),
    }
}

#[test]
fn event_capacity_failure_retry_matches_clean_one_shot_world() {
    for max_events in [0, 1] {
        // Arrange
        let mut retry_world = two_sensor_world();
        let mut clean_world = two_sensor_world();
        let limits = StepLimits::new(max_events, 64).expect("event limit should be valid");
        #[cfg(feature = "differential-internals")]
        let before_failure = retry_world.world_diagnostics();

        // Act
        let failure = retry_world.step(step_configuration(), &mut NoDecisionHook, limits);

        // Assert
        assert_eq!(
            failure,
            Err(StepError::LimitExceeded {
                resource: "event",
                limit: max_events
            })
        );
        #[cfg(feature = "differential-internals")]
        assert_eq!(retry_world.world_diagnostics(), before_failure);
        assert_eq!(retry_world.contact_count(), 0);
        assert!(!retry_world.is_locked());

        // Act
        let retry = retry_world
            .step(
                step_configuration(),
                &mut NoDecisionHook,
                StepLimits::default(),
            )
            .expect("retry should restore pair work");
        let clean = clean_world
            .step(
                step_configuration(),
                &mut NoDecisionHook,
                StepLimits::default(),
            )
            .expect("clean step should complete");

        // Assert
        assert_eq!(
            step_semantics(&retry_world, &retry),
            step_semantics(&clean_world, &clean)
        );
    }
}

#[derive(Clone, Copy)]
struct FixedCommandHook(WorldCommand);

impl StepHook for FixedCommandHook {
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        Some(self.0)
    }
}

#[test]
fn command_capacity_failure_retry_matches_clean_one_shot_world() {
    for max_commands in [0, 1] {
        // Arrange
        let mut retry_world = two_sensor_world();
        let retry_target = retry_world
            .create_body(&BodyDef::default())
            .expect("target should fit");
        let mut retry_hook = FixedCommandHook(WorldCommand::DestroyBody(retry_target));
        let mut clean_world = two_sensor_world();
        let clean_target = clean_world
            .create_body(&BodyDef::default())
            .expect("target should fit");
        let mut clean_hook = FixedCommandHook(WorldCommand::DestroyBody(clean_target));
        let limits = StepLimits::new(256, max_commands).expect("command limit should be valid");
        #[cfg(feature = "differential-internals")]
        let before_failure = retry_world.world_diagnostics();

        // Act
        let failure = retry_world.step(step_configuration(), &mut retry_hook, limits);

        // Assert
        assert_eq!(
            failure,
            Err(StepError::LimitExceeded {
                resource: "command",
                limit: max_commands
            })
        );
        #[cfg(feature = "differential-internals")]
        assert_eq!(retry_world.world_diagnostics(), before_failure);
        assert_eq!(retry_world.contact_count(), 0);
        assert!(retry_world.contains_body(retry_target));
        assert!(!retry_world.is_locked());

        // Act
        let retry = retry_world
            .step(step_configuration(), &mut retry_hook, StepLimits::default())
            .expect("retry should restore contact work");
        let clean = clean_world
            .step(step_configuration(), &mut clean_hook, StepLimits::default())
            .expect("clean step should complete");

        // Assert
        assert_eq!(
            step_semantics(&retry_world, &retry),
            step_semantics(&clean_world, &clean)
        );
        assert!(!retry_world.contains_body(retry_target));
        assert!(!clean_world.contains_body(clean_target));
    }
}

//! Source-timing evidence for borrowed collision and pre-solve decisions.

use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, CollisionDecisionHook, CollisionDirective, ContactControlError, FixtureDef,
    FixtureId, FixturePairView, NoDecisionHook, PreSolveDirective, PreSolveView, StepConfiguration,
    StepError, StepLimits, World, WorldCommand,
};

fn configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed step should be valid")
}

fn touching_world(sensor: bool) -> (World, FixtureId) {
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
    let fixture = |is_sensor| {
        FixtureDef::new(
            Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
            1.0,
            0.2,
            0.0,
            is_sensor,
            FilterData::default(),
        )
        .expect("fixture should be valid")
    };
    let static_fixture = world
        .create_fixture(static_body, &fixture(sensor))
        .expect("static fixture should fit");
    world
        .create_fixture(dynamic_body, &fixture(false))
        .expect("dynamic fixture should fit");
    (world, static_fixture)
}

#[derive(Default)]
struct CountingFilter {
    calls: usize,
    directive: Option<CollisionDirective>,
}

impl CollisionDecisionHook for CountingFilter {
    fn should_collide(&mut self, pair: FixturePairView<'_>) -> CollisionDirective {
        self.calls += 1;
        assert_ne!(pair.fixtures()[0], pair.fixtures()[1]);
        assert_ne!(pair.bodies()[0], pair.bodies()[1]);
        self.directive.unwrap_or(CollisionDirective::Collide)
    }
}

#[test]
fn filter_runs_at_admission_and_flagged_refilter_before_contact_effects() {
    // Arrange
    let (mut world, fixture) = touching_world(false);
    let mut admitting = CountingFilter::default();

    // Act
    world
        .step(configuration(), &mut admitting, StepLimits::default())
        .expect("admitted pair should step");
    world
        .set_fixture_filter(fixture, FilterData::new(0x0002, u16::MAX, 0))
        .expect("filter mutation should be valid");
    let mut rejecting = CountingFilter {
        calls: 0,
        directive: Some(CollisionDirective::Ignore),
    };
    world
        .step(configuration(), &mut rejecting, StepLimits::default())
        .expect("refilter rejection should remain coherent");

    // Assert
    assert_eq!(admitting.calls, 1);
    assert_eq!(rejecting.calls, 1);
    assert_eq!(world.contact_count(), 0);
}

#[derive(Default)]
struct PreSolveRecorder {
    calls: usize,
    previous_point_counts: Vec<usize>,
    current_point_counts: Vec<usize>,
    directive: PreSolveDirective,
}

impl CollisionDecisionHook for PreSolveRecorder {
    fn pre_solve(&mut self, contact: PreSolveView<'_>) -> PreSolveDirective {
        self.calls += 1;
        self.previous_point_counts.push(
            contact
                .maybe_previous_manifold()
                .map_or(0, |manifold| manifold.points().len()),
        );
        self.current_point_counts
            .push(contact.current_manifold().points().len());
        self.directive
    }
}

#[test]
fn pre_solve_receives_current_and_previous_manifolds_and_disable_resets() {
    // Arrange
    let (mut world, _fixture) = touching_world(false);
    let mut disabling = PreSolveRecorder {
        directive: PreSolveDirective::Disable,
        ..PreSolveRecorder::default()
    };

    // Act
    let first = world
        .step(configuration(), &mut disabling, StepLimits::default())
        .expect("disabled update should remain coherent");
    let second = world
        .step(configuration(), &mut NoDecisionHook, StepLimits::default())
        .expect("next update should re-enable the contact");

    // Assert
    assert_eq!(disabling.calls, 1);
    assert_eq!(disabling.previous_point_counts, vec![0]);
    assert_eq!(disabling.current_point_counts, vec![1]);
    assert!(first.contact_solves().is_empty());
    assert_eq!(second.contact_solves().len(), 1);
}

#[test]
fn sensor_skips_pre_solve() {
    // Arrange
    let (mut world, _fixture) = touching_world(true);
    let mut hook = PreSolveRecorder::default();

    // Act
    let report = world
        .step(configuration(), &mut hook, StepLimits::default())
        .expect("sensor update should remain coherent");

    // Assert
    assert_eq!(hook.calls, 0);
    assert!(report.contact_solves().is_empty());
}

#[test]
fn material_controls_validate_before_the_hook_can_return_them() {
    // Arrange
    let valid = PreSolveDirective::Enable
        .with_friction(0.75)
        .and_then(|directive| directive.with_restitution(0.25))
        .and_then(|directive| directive.with_tangent_speed(-1.5));

    // Act
    let invalid_friction = PreSolveDirective::Enable.with_friction(f32::NAN);
    let invalid_restitution = PreSolveDirective::Enable.with_restitution(-1.0);
    let invalid_tangent = PreSolveDirective::Enable.with_tangent_speed(f32::INFINITY);

    // Assert
    assert!(valid.is_ok());
    assert_eq!(invalid_friction, Err(ContactControlError::NonFinite));
    assert_eq!(invalid_restitution, Err(ContactControlError::Negative));
    assert_eq!(invalid_tangent, Err(ContactControlError::NonFinite));
}

struct MaterialHook;

impl CollisionDecisionHook for MaterialHook {
    fn pre_solve(&mut self, _contact: PreSolveView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
            .with_friction(0.75)
            .and_then(|directive| directive.with_restitution(0.25))
            .and_then(|directive| directive.with_tangent_speed(-1.5))
            .expect("fixed source-supported controls should be valid")
    }
}

#[test]
fn validated_material_controls_reach_the_same_updates_solver_state() {
    // Arrange
    let (mut world, _fixture) = touching_world(false);
    let mut hook = MaterialHook;

    // Act
    let report = world
        .step(configuration(), &mut hook, StepLimits::default())
        .expect("material-controlled contact should solve");
    let solved = report.contact_solves()[0].contact();

    // Assert
    assert_eq!(solved.friction().to_bits(), 0.75_f32.to_bits());
    assert_eq!(solved.restitution().to_bits(), 0.25_f32.to_bits());
    assert_eq!(solved.tangent_speed().to_bits(), (-1.5_f32).to_bits());
}

fn moving_circle(world: &mut World, position: Vec2, velocity: Vec2) {
    let body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, position, 0.0, true)
                .expect("moving body should be valid")
                .with_linear_velocity(velocity)
                .expect("moving velocity should be valid")
                .with_bullet(true),
        )
        .expect("moving body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("moving fixture should be valid");
    world
        .create_fixture(body, &fixture)
        .expect("moving fixture should fit");
}

fn static_circle(world: &mut World) {
    let body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("static body should be valid"),
        )
        .expect("static body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("circle should be valid")),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("static fixture should be valid");
    world
        .create_fixture(body, &fixture)
        .expect("static fixture should fit");
}

#[test]
fn continuous_refreshes_remain_eligible_for_pre_solve_per_occurrence() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    moving_circle(&mut world, Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0));
    static_circle(&mut world);
    static_circle(&mut world);
    let configuration = StepConfiguration::new(1.0, 8, 3).expect("swept step should be valid");
    let mut hook = PreSolveRecorder::default();

    // Act
    let report = world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("bounded continuous occurrences should solve");

    // Assert
    assert!(hook.calls >= 2);
    assert_eq!(hook.calls, report.events().len());
    assert!(
        report
            .events()
            .iter()
            .all(|event| event.maybe_pre_solve().is_some())
    );
}

struct PanickingFilter;

impl CollisionDecisionHook for PanickingFilter {
    fn should_collide(&mut self, _pair: FixturePairView<'_>) -> CollisionDirective {
        panic!("intentional admission panic");
    }
}

#[test]
fn decision_panic_restores_lock_and_poisons_without_admitting_contact() {
    // Arrange
    let (mut world, _fixture) = touching_world(false);
    let mut hook = PanickingFilter;

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _result = world.step(configuration(), &mut hook, StepLimits::default());
    }));

    // Assert
    assert!(panic.is_err());
    assert!(!world.is_locked());
    assert!(world.is_poisoned());
    assert_eq!(world.contact_count(), 0);
    assert_eq!(
        world.step(configuration(), &mut hook, StepLimits::default()),
        Err(StepError::Poisoned)
    );
}

struct QueuingThenPanickingHook {
    body: liquidfun::BodyId,
    observations: usize,
}

impl CollisionDecisionHook for QueuingThenPanickingHook {
    fn observe(&mut self, _contact: liquidfun::ContactView<'_>) {
        self.observations += 1;
        assert!(self.observations < 2, "intentional later occurrence panic");
    }

    fn command(&mut self, _contact: liquidfun::ContactView<'_>) -> Option<WorldCommand> {
        Some(WorldCommand::DestroyBody(self.body))
    }
}

#[test]
fn later_hook_panic_discards_command_queued_by_an_earlier_occurrence() {
    // Arrange
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
    let sensor = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        1.0,
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
            .expect("sensor fixture should fit");
    }
    world
        .create_fixture(dynamic_body, &solid)
        .expect("dynamic fixture should fit");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("queued command body should fit");
    let mut hook = QueuingThenPanickingHook {
        body: survivor,
        observations: 0,
    };

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _result = world.step(configuration(), &mut hook, StepLimits::default());
    }));

    // Assert
    assert!(panic.is_err());
    assert!(!world.is_locked());
    assert!(world.is_poisoned());
    assert!(world.contains_body(survivor));
}

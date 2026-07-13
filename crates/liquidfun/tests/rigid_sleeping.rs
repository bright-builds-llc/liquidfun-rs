//! Per-island sleeping and source-specific waking witnesses.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::math::settings::{ANGULAR_SLEEP_TOLERANCE, LINEAR_SLEEP_TOLERANCE, TIME_TO_SLEEP};
#[cfg(feature = "differential-internals")]
use liquidfun::rigid_differential::RigidStepFailureInjection;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, StepConfiguration, StepHook, StepLimits, WakePolicy,
    World,
};

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

fn body_definition(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .expect("test body definition should be valid")
}

fn create_body(world: &mut World, definition: &BodyDef) -> BodyId {
    world.create_body(definition).expect("test body should fit")
}

fn attach_circle(world: &mut World, body: BodyId) {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
    let fixture = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture should be valid");
    world
        .create_fixture(body, &fixture)
        .expect("test fixture should fit");
}

fn step(world: &mut World, time_step: f32, position_iterations: u32) {
    let configuration = StepConfiguration::new(time_step, 8, position_iterations)
        .expect("test step configuration should be valid");
    world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("test world should step successfully");
}

fn isolated_body_is_awake_after(time_step: f32) -> bool {
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let body = create_body(&mut world, &body_definition(Vec2::ZERO));

    step(&mut world, time_step, 3);

    world
        .body_snapshot(body)
        .expect("body should remain live")
        .is_awake()
}

#[test]
fn thresholds_linear_velocity_use_strict_squared_comparison() {
    // Arrange
    let below = f32::from_bits(LINEAR_SLEEP_TOLERANCE.to_bits() - 1);
    let equal = LINEAR_SLEEP_TOLERANCE;
    let above = f32::from_bits(LINEAR_SLEEP_TOLERANCE.to_bits() + 1);
    let mut awake = Vec::new();

    // Act
    for velocity in [below, equal, above] {
        let mut world = World::new().expect("world key should remain available");
        world
            .set_gravity(Vec2::ZERO)
            .expect("zero gravity should be valid");
        let definition = body_definition(Vec2::ZERO)
            .with_linear_velocity(Vec2::new(velocity, 0.0))
            .expect("test velocity should be valid");
        let body = create_body(&mut world, &definition);
        step(&mut world, TIME_TO_SLEEP, 3);
        awake.push(
            world
                .body_snapshot(body)
                .expect("body should remain live")
                .is_awake(),
        );
    }

    // Assert
    assert_eq!(awake, [false, false, true]);
}

#[test]
fn thresholds_angular_velocity_use_strict_squared_comparison() {
    // Arrange
    let below = f32::from_bits(ANGULAR_SLEEP_TOLERANCE.to_bits() - 1);
    let equal = ANGULAR_SLEEP_TOLERANCE;
    let above = f32::from_bits(ANGULAR_SLEEP_TOLERANCE.to_bits() + 1);
    let mut awake = Vec::new();

    // Act
    for velocity in [below, equal, above] {
        let mut world = World::new().expect("world key should remain available");
        world
            .set_gravity(Vec2::ZERO)
            .expect("zero gravity should be valid");
        let definition = body_definition(Vec2::ZERO)
            .with_angular_velocity(velocity)
            .expect("test angular velocity should be valid");
        let body = create_body(&mut world, &definition);
        step(&mut world, TIME_TO_SLEEP, 3);
        awake.push(
            world
                .body_snapshot(body)
                .expect("body should remain live")
                .is_awake(),
        );
    }

    // Assert
    assert_eq!(awake, [false, false, true]);
}

#[test]
fn thresholds_duration_uses_inclusive_time_to_sleep_boundary() {
    // Arrange
    let before = f32::from_bits(TIME_TO_SLEEP.to_bits() - 1);
    let equal = TIME_TO_SLEEP;
    let after = f32::from_bits(TIME_TO_SLEEP.to_bits() + 1);

    // Act
    let awake = [
        isolated_body_is_awake_after(before),
        isolated_body_is_awake_after(equal),
        isolated_body_is_awake_after(after),
    ];

    // Assert
    assert_eq!(awake, [true, false, false]);
}

#[test]
fn thresholds_unconverged_contact_positions_prevent_sleep() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let first = create_body(&mut world, &body_definition(Vec2::ZERO));
    let second = create_body(&mut world, &body_definition(Vec2::ZERO));
    attach_circle(&mut world, first);
    attach_circle(&mut world, second);

    // Act
    step(&mut world, TIME_TO_SLEEP, 1);

    // Assert
    assert!(
        world
            .body_snapshot(first)
            .expect("first body should remain live")
            .is_awake()
    );
    assert!(
        world
            .body_snapshot(second)
            .expect("second body should remain live")
            .is_awake()
    );
}

#[test]
fn thresholds_mixed_allowed_sleep_blocks_then_transitions_whole_island() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let first = create_body(&mut world, &body_definition(Vec2::ZERO));
    let blocked_definition = body_definition(Vec2::new(2.0, 0.0)).with_sleeping_allowed(false);
    let blocked = create_body(&mut world, &blocked_definition);
    attach_circle(&mut world, first);
    attach_circle(&mut world, blocked);

    // Act
    step(&mut world, TIME_TO_SLEEP, 3);
    let first_while_blocked = world
        .body_snapshot(first)
        .expect("first body should remain live");
    let blocked_while_blocked = world
        .body_snapshot(blocked)
        .expect("blocked body should remain live");
    world
        .set_body_sleeping_allowed(blocked, true)
        .expect("body should remain live");
    step(&mut world, TIME_TO_SLEEP, 3);
    let first_after = world
        .body_snapshot(first)
        .expect("first body should remain live");
    let blocked_after = world
        .body_snapshot(blocked)
        .expect("blocked body should remain live");

    // Assert
    assert!(first_while_blocked.is_awake());
    assert!(blocked_while_blocked.is_awake());
    assert!(!first_after.is_awake());
    assert!(!blocked_after.is_awake());
}

#[test]
fn thresholds_sleep_primitive_clears_forces_when_automatic_clearing_is_disabled() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    world
        .set_automatic_force_clearing_enabled(false)
        .expect("world should accept force-clearing policy");
    let body = create_body(&mut world, &body_definition(Vec2::ZERO));
    world
        .apply_body_force_to_center(body, Vec2::new(0.001, 0.0), WakePolicy::Wake)
        .expect("small force should be valid");

    // Act
    step(&mut world, TIME_TO_SLEEP, 3);
    world
        .set_body_awake(body, true)
        .expect("body should remain live");
    step(&mut world, 0.25, 3);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(after.linear_velocity(), Vec2::ZERO);
}

#[cfg(feature = "differential-internals")]
#[test]
fn thresholds_sleep_candidates_roll_back_with_late_island_failure() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let body = create_body(&mut world, &body_definition(Vec2::ZERO));
    let limits = StepLimits::default()
        .with_rigid_failure_injection(RigidStepFailureInjection::LateIsland { solved_islands: 1 });
    let configuration = StepConfiguration::new(TIME_TO_SLEEP, 8, 3)
        .expect("test step configuration should be valid");

    // Act
    let failed = world.step(configuration, &mut NoopHook, limits);
    let after_failure = world.body_snapshot(body).expect("body should remain live");
    step(&mut world, TIME_TO_SLEEP, 3);
    let after_success = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert!(failed.is_err());
    assert!(after_failure.is_awake());
    assert!(!after_success.is_awake());
}

#[test]
fn wake_sources_type_and_sensor_markers_wake_before_island_seeding() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let type_body = create_body(
        &mut world,
        &body_definition(Vec2::new(-4.0, 0.0)).with_awake(false),
    );
    let sensor_body = create_body(
        &mut world,
        &body_definition(Vec2::new(4.0, 0.0)).with_awake(false),
    );
    let sensor_shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
    let sensor_fixture = world
        .create_fixture(
            sensor_body,
            &FixtureDef::new(sensor_shape, 1.0, 0.2, 0.0, false, FilterData::default())
                .expect("test fixture should be valid"),
        )
        .expect("test fixture should fit");

    // Act
    world
        .set_body_type(type_body, BodyType::Kinematic)
        .expect("body type change should be valid");
    world
        .set_fixture_sensor(sensor_fixture, true)
        .expect("sensor change should be valid");
    step(&mut world, 0.1, 3);
    let type_after = world
        .body_snapshot(type_body)
        .expect("type body should remain live");
    let sensor_after = world
        .body_snapshot(sensor_body)
        .expect("sensor body should remain live");

    // Assert
    assert!(type_after.is_awake());
    assert!(sensor_after.is_awake());
}

#[test]
fn wake_sources_solid_contact_creation_wakes_both_bodies() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let first = create_body(&mut world, &body_definition(Vec2::ZERO).with_awake(false));
    let second = create_body(
        &mut world,
        &body_definition(Vec2::new(2.0, 0.0)).with_awake(false),
    );
    attach_circle(&mut world, first);
    attach_circle(&mut world, second);

    // Act
    step(&mut world, 0.1, 3);

    // Assert
    assert!(
        world
            .body_snapshot(first)
            .expect("first body should remain live")
            .is_awake()
    );
    assert!(
        world
            .body_snapshot(second)
            .expect("second body should remain live")
            .is_awake()
    );
}

#[test]
fn wake_sources_solid_contact_transition_wakes_both_bodies() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let first = create_body(&mut world, &body_definition(Vec2::ZERO));
    let second = create_body(&mut world, &body_definition(Vec2::new(2.0, 0.0)));
    attach_circle(&mut world, first);
    attach_circle(&mut world, second);
    step(&mut world, TIME_TO_SLEEP, 3);
    assert!(
        !world
            .body_snapshot(first)
            .expect("first body should remain live")
            .is_awake()
    );
    assert!(
        !world
            .body_snapshot(second)
            .expect("second body should remain live")
            .is_awake()
    );

    // Act
    world
        .set_body_transform(second, Vec2::new(2.05, 0.0), 0.0)
        .expect("separating transform should be valid");
    step(&mut world, 0.1, 3);

    // Assert
    assert!(
        world
            .body_snapshot(first)
            .expect("first body should remain live")
            .is_awake()
    );
    assert!(
        world
            .body_snapshot(second)
            .expect("second body should remain live")
            .is_awake()
    );
}

#[test]
fn wake_sources_sensor_contact_creation_preserves_sleep() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity should be valid");
    let first = create_body(&mut world, &body_definition(Vec2::ZERO).with_awake(false));
    let second = create_body(
        &mut world,
        &body_definition(Vec2::new(2.0, 0.0)).with_awake(false),
    );
    attach_circle(&mut world, first);
    let sensor_shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid"));
    world
        .create_fixture(
            second,
            &FixtureDef::new(sensor_shape, 1.0, 0.2, 0.0, true, FilterData::default())
                .expect("test sensor fixture should be valid"),
        )
        .expect("test sensor fixture should fit");

    // Act
    step(&mut world, 0.1, 3);

    // Assert
    assert!(
        !world
            .body_snapshot(first)
            .expect("first body should remain live")
            .is_awake()
    );
    assert!(
        !world
            .body_snapshot(second)
            .expect("second body should remain live")
            .is_awake()
    );
}

#[test]
fn wake_sources_activation_and_passive_controls_do_not_invent_waking() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, false)
        .expect("inactive body definition should be valid")
        .with_awake(false);
    let body = create_body(&mut world, &definition);

    // Act
    world
        .set_body_active(body, true)
        .expect("activation should be valid");
    world
        .set_body_linear_damping(body, 0.5)
        .expect("linear damping should be valid");
    world
        .set_body_angular_damping(body, 0.25)
        .expect("angular damping should be valid");
    world
        .set_body_gravity_scale(body, -1.0)
        .expect("gravity scale should be valid");
    world
        .set_body_bullet(body, true)
        .expect("bullet state should be valid");
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .expect("world gravity should be valid");
    world
        .set_body_linear_velocity(body, Vec2::ZERO)
        .expect("zero velocity should be valid");
    world
        .apply_body_force_to_center(body, Vec2::new(1.0, 0.0), WakePolicy::PreserveSleep)
        .expect("preserve-sleep force should be a successful no-op");
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert!(!after.is_awake());
}

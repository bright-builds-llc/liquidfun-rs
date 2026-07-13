//! Black-box checks for checked rigid-world and timestep configuration.

use liquidfun::math::Vec2;
use liquidfun::{
    StepCompletion, StepConfiguration, StepConfigurationError, StepHook, StepLimits, World,
    WorldConfigurationError,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn configuration(time_step: f32) -> StepConfiguration {
    StepConfiguration::new(time_step, 8, 3).expect("test step configuration should be valid")
}

#[test]
fn configuration_defaults_match_the_pinned_world() {
    // Arrange
    let world = World::new().expect("world key should remain available");

    // Act
    let gravity = world.gravity();

    // Assert
    assert_eq!(gravity, Vec2::ZERO);
    assert!(world.is_warm_starting_enabled());
    assert!(world.is_continuous_physics_enabled());
    assert!(!world.is_sub_stepping_enabled());
    assert!(world.is_automatic_force_clearing_enabled());
    assert_eq!(StepCompletion::default(), StepCompletion::Complete);
}

#[test]
fn configuration_accepts_iteration_maxima_and_rejects_the_next_values() {
    // Arrange
    let maximum_velocity = StepConfiguration::MAX_VELOCITY_ITERATIONS;
    let maximum_position = StepConfiguration::MAX_POSITION_ITERATIONS;

    // Act
    let accepted = StepConfiguration::new(1.0 / 60.0, maximum_velocity, maximum_position);
    let velocity_overflow =
        StepConfiguration::new(1.0 / 60.0, maximum_velocity + 1, maximum_position);
    let position_overflow =
        StepConfiguration::new(1.0 / 60.0, maximum_velocity, maximum_position + 1);

    // Assert
    let accepted = accepted.expect("reviewed maxima should be accepted");
    assert_eq!(accepted.velocity_iterations(), maximum_velocity);
    assert_eq!(accepted.position_iterations(), maximum_position);
    assert_eq!(
        velocity_overflow,
        Err(StepConfigurationError::VelocityIterationsOutOfRange {
            requested: maximum_velocity + 1,
            maximum: maximum_velocity,
        })
    );
    assert_eq!(
        position_overflow,
        Err(StepConfigurationError::PositionIterationsOutOfRange {
            requested: maximum_position + 1,
            maximum: maximum_position,
        })
    );
}

#[test]
fn configuration_rejects_zero_iterations() {
    // Arrange
    let time_step = 1.0 / 60.0;

    // Act
    let zero_velocity = StepConfiguration::new(time_step, 0, 3);
    let zero_position = StepConfiguration::new(time_step, 8, 0);

    // Assert
    assert_eq!(
        zero_velocity,
        Err(StepConfigurationError::VelocityIterationsOutOfRange {
            requested: 0,
            maximum: StepConfiguration::MAX_VELOCITY_ITERATIONS,
        })
    );
    assert_eq!(
        zero_position,
        Err(StepConfigurationError::PositionIterationsOutOfRange {
            requested: 0,
            maximum: StepConfiguration::MAX_POSITION_ITERATIONS,
        })
    );
}

#[test]
fn configuration_accepts_zero_timestep_and_rejects_invalid_values() {
    // Arrange
    let velocity_iterations = 8;
    let position_iterations = 3;

    // Act
    let zero = StepConfiguration::new(0.0, velocity_iterations, position_iterations);
    let negative = StepConfiguration::new(-0.25, velocity_iterations, position_iterations);
    let non_finite = StepConfiguration::new(f32::NAN, velocity_iterations, position_iterations);

    // Assert
    assert_eq!(
        zero.expect("zero timestep should be accepted")
            .time_step()
            .to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(negative, Err(StepConfigurationError::NegativeTimeStep));
    assert_eq!(non_finite, Err(StepConfigurationError::NonFiniteTimeStep));
}

#[test]
fn configuration_gravity_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::new(2.5, -9.0))
        .expect("finite gravity should be accepted");

    // Act
    let x_result = world.set_gravity(Vec2::new(f32::NAN, 1.0));
    let y_result = world.set_gravity(Vec2::new(1.0, f32::INFINITY));

    // Assert
    assert_eq!(x_result, Err(WorldConfigurationError::NonFiniteGravityX));
    assert_eq!(y_result, Err(WorldConfigurationError::NonFiniteGravityY));
    assert_eq!(world.gravity(), Vec2::new(2.5, -9.0));
}

#[test]
fn configuration_boolean_controls_round_trip_without_changing_gravity() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let gravity_before = world.gravity();

    // Act
    world
        .set_warm_starting_enabled(false)
        .expect("world configuration should be mutable");
    world
        .set_continuous_physics_enabled(false)
        .expect("world configuration should be mutable");
    world
        .set_sub_stepping_enabled(true)
        .expect("world configuration should be mutable");
    world
        .set_automatic_force_clearing_enabled(false)
        .expect("world configuration should be mutable");

    // Assert
    assert!(!world.is_warm_starting_enabled());
    assert!(!world.is_continuous_physics_enabled());
    assert!(world.is_sub_stepping_enabled());
    assert!(!world.is_automatic_force_clearing_enabled());
    assert_eq!(world.gravity(), gravity_before);
}

#[test]
fn configuration_zero_step_retains_prior_inverse_timestep_for_exact_ratio() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let mut hook = NoopHook;

    // Act
    let first = world
        .step(configuration(0.5), &mut hook, StepLimits::default())
        .expect("positive empty-world step should succeed");
    let zero = world
        .step(configuration(0.0), &mut hook, StepLimits::default())
        .expect("zero empty-world step should succeed");
    let next = world
        .step(configuration(0.25), &mut hook, StepLimits::default())
        .expect("positive empty-world step should succeed");

    // Assert
    assert_eq!(first.time_step_ratio().to_bits(), 0.0_f32.to_bits());
    assert_eq!(zero.time_step_ratio().to_bits(), 0.0_f32.to_bits());
    assert_eq!(next.time_step_ratio().to_bits(), 0.5_f32.to_bits());
    assert_eq!(first.completion(), StepCompletion::Complete);
    assert_eq!(zero.completion(), StepCompletion::Complete);
    assert_eq!(next.completion(), StepCompletion::Complete);
}

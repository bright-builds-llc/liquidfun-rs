//! Public semantic witnesses for the complete particle solver transaction.

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{
    NoDecisionHook, StepConfiguration, StepConfigurationError, StepError, StepLimits, World,
};

fn configuration(iterations: u32) -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("base step configuration is valid")
        .with_particle_iterations(iterations)
        .expect("particle iteration count is valid")
}

#[test]
fn empty_particle_world_has_no_solver_effect() {
    // Arrange
    let mut world = World::new().expect("world key remains available");

    // Act
    let result = world.step(configuration(2), &mut NoDecisionHook, StepLimits::default());

    // Assert
    assert!(result.is_ok());
    assert_eq!(world.particle_system_ids().count(), 0);
}

#[test]
fn paused_nonempty_system_preserves_particle_state() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("paused system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.0, 2.0))
                .expect("position is finite")
                .with_velocity(Vec2::new(3.0, 4.0))
                .expect("velocity is finite"),
        )
        .expect("particle fits")
        .created_particle();
    let before = world.particle_snapshot(particle).expect("particle is live");

    // Act
    world
        .step(configuration(2), &mut NoDecisionHook, StepLimits::default())
        .expect("paused step succeeds");

    // Assert
    assert_eq!(world.particle_snapshot(particle), Ok(before));
}

#[test]
fn valid_one_and_two_iteration_steps_integrate_the_same_horizon() {
    // Arrange
    let mut one = World::new().expect("first world key remains available");
    let mut two = World::new().expect("second world key remains available");
    one.set_gravity(Vec2::ZERO).expect("zero gravity is valid");
    two.set_gravity(Vec2::ZERO).expect("zero gravity is valid");
    let system_one = one
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("first system fits");
    let system_two = two
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("second system fits");
    let definition = ParticleDef::default()
        .with_velocity(Vec2::new(6.0, 0.0))
        .expect("velocity is finite");
    let particle_one = one
        .create_particle_with_def(system_one, None, &definition)
        .expect("first particle fits")
        .created_particle();
    let particle_two = two
        .create_particle_with_def(system_two, None, &definition)
        .expect("second particle fits")
        .created_particle();

    // Act
    one.step(configuration(1), &mut NoDecisionHook, StepLimits::default())
        .expect("one-iteration step succeeds");
    two.step(configuration(2), &mut NoDecisionHook, StepLimits::default())
        .expect("two-iteration step succeeds");

    // Assert
    let one = one
        .particle_snapshot(particle_one)
        .expect("particle is live");
    let two = two
        .particle_snapshot(particle_two)
        .expect("particle is live");
    let one_step_position = 6.0_f32 * (1.0_f32 / 60.0_f32);
    let two_step_position = (6.0_f32 * (1.0_f32 / 120.0_f32)) * 2.0_f32;
    assert_eq!(one.position().x.to_bits(), one_step_position.to_bits());
    assert_eq!(two.position().x.to_bits(), two_step_position.to_bits());
    assert_eq!(one.velocity(), two.velocity());
}

#[test]
fn zero_particle_iterations_returns_the_existing_typed_error() {
    // Arrange
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("base step configuration is valid");

    // Act
    let error = configuration
        .with_particle_iterations(0)
        .expect_err("zero iterations must be rejected");

    // Assert
    assert_eq!(
        error,
        StepConfigurationError::ParticleIterationsOutOfRange {
            requested: 0,
            maximum: 1_024,
        }
    );
}

#[test]
fn late_contact_journal_failure_rolls_back_particle_state() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("system fits");
    let flags = ParticleFlags::PARTICLE_CONTACT_LISTENER;
    let first = world
        .create_particle_with_def(system, None, &ParticleDef::default().with_flags(flags))
        .expect("first particle fits")
        .created_particle();
    let second = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(0.5, 0.0))
                .expect("position is finite")
                .with_flags(flags),
        )
        .expect("second particle fits")
        .created_particle();
    let before = [
        world.particle_snapshot(first).expect("first is live"),
        world.particle_snapshot(second).expect("second is live"),
    ];

    // Act
    let result = world.step(
        configuration(2),
        &mut NoDecisionHook,
        StepLimits::new(0, 0).expect("zero journal capacity is valid"),
    );

    // Assert
    assert!(matches!(result, Err(StepError::LimitExceeded { .. })));
    assert_eq!(world.particle_snapshot(first), Ok(before[0]));
    assert_eq!(world.particle_snapshot(second), Ok(before[1]));
}

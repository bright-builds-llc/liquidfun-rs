//! Transactional particle lifecycle integration with the World step journal.

use liquidfun::particle::{ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{
    DestroyedId, HandleError, NoDecisionHook, StepConfiguration, StepError, StepLifecycleEvent,
    StepLimits, World,
};

fn step_configuration(time_step: f32) -> StepConfiguration {
    StepConfiguration::new(time_step, 8, 3).expect("test step configuration is valid")
}

fn finite_particle(flags: ParticleFlags) -> ParticleDef {
    ParticleDef::default()
        .with_flags(flags)
        .with_lifetime(1.0)
        .expect("test lifetime is finite")
}

#[test]
fn world_step_compacts_expired_particles_even_when_the_system_is_paused() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("granularity is positive")
        .with_paused(true);
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &finite_particle(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("particle fits");
    let mut hook = NoDecisionHook;

    // Act
    let report = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("paused maintenance succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(particle),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(report.destructions().len(), 1);
    assert!(matches!(
        report.lifecycle(),
        [StepLifecycleEvent::ParticleDestruction(record)]
            if record.destroyed() == DestroyedId::Particle(particle)
    ));
}

#[test]
fn requested_particle_destructions_follow_newest_system_first_order() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("granularity is positive");
    let older_system = world
        .create_particle_system_with_def(&definition)
        .expect("older system fits");
    let older = world
        .create_particle_with_def(
            older_system,
            None,
            &finite_particle(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("older particle fits");
    let newer_system = world
        .create_particle_system_with_def(&definition)
        .expect("newer system fits");
    let newer = world
        .create_particle_with_def(
            newer_system,
            None,
            &finite_particle(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("newer particle fits");
    let mut hook = NoDecisionHook;

    // Act
    let report = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("particle maintenance succeeds");
    let destroyed = report
        .destructions()
        .iter()
        .map(liquidfun::DestructionRecord::destroyed)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        destroyed,
        vec![DestroyedId::Particle(newer), DestroyedId::Particle(older)]
    );
}

#[test]
fn unrequested_particle_destruction_compacts_without_fabricating_a_callback() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("granularity is positive");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(system, None, &finite_particle(ParticleFlags::WATER))
        .expect("particle fits");
    let mut hook = NoDecisionHook;

    // Act
    let report = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("particle maintenance succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(particle),
        Err(HandleError::StaleOrDestroyed)
    );
    assert!(report.lifecycle().is_empty());
    assert!(report.destructions().is_empty());
}

#[test]
fn particle_event_limit_failure_restores_lifetime_and_storage_for_retry() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("granularity is positive");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &finite_particle(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("particle fits");
    let mut hook = NoDecisionHook;
    let zero_events = StepLimits::new(0, StepLimits::default().max_commands())
        .expect("zero is a valid event limit");

    // Act
    let rejected = world.step(step_configuration(1.0), &mut hook, zero_events);
    let after_rejection = world.particle_snapshot(particle);
    let retry = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("retry succeeds from the exact pre-call state");

    // Assert
    assert_eq!(
        rejected,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 0,
        })
    );
    assert!(after_rejection.is_ok());
    assert_eq!(retry.destructions().len(), 1);
    assert_eq!(
        retry.destructions()[0].destroyed(),
        DestroyedId::Particle(particle)
    );
}

#[test]
fn particle_system_teardown_uses_the_shared_lifecycle_vocabulary() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system fits");
    let requested = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_flags(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("particle fits");

    // Act
    let report = world
        .destroy_particle_system(system)
        .expect("system is live");

    // Assert
    assert_eq!(report.len(), 2);
    assert!(matches!(
        report.lifecycle(),
        [
            StepLifecycleEvent::ParticleDestruction(particle),
            StepLifecycleEvent::Destruction(root),
        ] if particle.destroyed() == DestroyedId::Particle(requested)
            && root.destroyed() == DestroyedId::ParticleSystem(system)
    ));
}

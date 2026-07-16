//! Transactional particle lifecycle integration with the World step journal.

use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleCapacity, ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{
    BodyDef, BodyType, DestroyedId, FixtureDef, HandleError, NoDecisionHook, StepConfiguration,
    StepError, StepHook, StepLifecycleEvent, StepLimits, World,
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
        .expect("particle fits")
        .created_particle();
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
        .expect("older particle fits")
        .created_particle();
    let newer_system = world
        .create_particle_system_with_def(&definition)
        .expect("newer system fits");
    let newer = world
        .create_particle_with_def(
            newer_system,
            None,
            &finite_particle(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("newer particle fits")
        .created_particle();
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
        .expect("particle fits")
        .created_particle();
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
        .expect("particle fits")
        .created_particle();
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
        .expect("particle fits")
        .created_particle();

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

#[test]
fn paused_step_compacts_an_explicit_zombie_and_journals_only_requested_occurrences() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("particle system fits");
    let requested = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_flags(ParticleFlags::WATER | ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("requested particle fits")
        .created_particle();
    let unrequested = world
        .create_particle(system, None)
        .expect("unrequested particle fits")
        .created_particle();
    world
        .mark_particle_for_destruction(requested)
        .expect("requested particle becomes pending");
    world
        .mark_particle_for_destruction(unrequested)
        .expect("unrequested particle becomes pending");
    let mut hook = NoDecisionHook;

    // Act
    let report = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("paused zombie maintenance succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(requested),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world.particle_snapshot(unrequested),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(report.destructions().len(), 1);
    assert_eq!(
        report.destructions()[0].destroyed(),
        DestroyedId::Particle(requested)
    );
}

#[test]
fn direct_compaction_returns_the_same_source_timed_listener_projection() {
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
        .expect("requested particle fits")
        .created_particle();
    let unrequested = world
        .create_particle(system, None)
        .expect("unrequested particle fits")
        .created_particle();
    world
        .mark_particle_for_destruction(requested)
        .expect("requested particle becomes pending");
    world
        .mark_particle_for_destruction(unrequested)
        .expect("unrequested particle becomes pending");

    // Act
    let report = world
        .compact_pending_particles(system)
        .expect("direct compaction succeeds");

    // Assert
    assert_eq!(report.len(), 2);
    assert!(matches!(
        report.lifecycle(),
        [StepLifecycleEvent::ParticleDestruction(record)]
            if record.destroyed() == DestroyedId::Particle(requested)
    ));
}

#[test]
fn maximum_count_creation_compacts_immediately_and_preserves_the_replacement() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_capacity(ParticleCapacity::growable(1).expect("capacity is representable"))
        .expect("capacity is valid")
        .with_maximum_count(1)
        .expect("maximum matches capacity")
        .with_destruction_by_age(true);
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let evicted = world
        .create_particle(system, None)
        .expect("first particle fits")
        .created_particle();

    // Act
    let replacement = world
        .create_particle(system, None)
        .expect("oldest particle is compacted before replacement creation")
        .created_particle();

    // Assert
    assert_eq!(
        world.particle_snapshot(evicted),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world
            .particle_snapshot(replacement)
            .expect("replacement remains live")
            .id(),
        replacement
    );
}

#[test]
fn rigid_contact_effects_precede_particle_destruction_in_the_shared_journal() {
    // Arrange
    let mut world = touching_world();
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
        .expect("particle fits")
        .created_particle();
    let mut hook = NoDecisionHook;

    // Act
    let report = world
        .step(step_configuration(1.0), &mut hook, StepLimits::default())
        .expect("mixed step succeeds");
    let contact_index = report
        .lifecycle()
        .iter()
        .position(|event| matches!(event, StepLifecycleEvent::Contact(_)))
        .expect("rigid contact begins");
    let particle_index = report
        .lifecycle()
        .iter()
        .position(|event| matches!(event, StepLifecycleEvent::ParticleDestruction(record) if record.destroyed() == DestroyedId::Particle(particle)))
        .expect("particle listener occurrence is journaled");

    // Assert
    assert!(contact_index < particle_index);
}

struct PanickingHook;

impl StepHook for PanickingHook {
    fn observe(&mut self, _contact: liquidfun::ContactView<'_>) {
        panic!("intentional particle lifecycle poison witness");
    }
}

#[test]
fn hook_panic_restores_the_lock_discards_particle_maintenance_and_poisons_access() {
    // Arrange
    let mut world = touching_world();
    let system = world
        .create_particle_system()
        .expect("particle system fits");
    let particle = world
        .create_particle(system, None)
        .expect("particle fits")
        .created_particle();
    world
        .mark_particle_for_destruction(particle)
        .expect("particle becomes pending");
    let mut hook = PanickingHook;

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _report = world.step(step_configuration(0.0), &mut hook, StepLimits::default());
    }));
    let nested = world.step(
        step_configuration(0.0),
        &mut NoDecisionHook,
        StepLimits::default(),
    );

    // Assert
    assert!(panic.is_err());
    assert!(world.is_poisoned());
    assert_eq!(nested, Err(StepError::Poisoned));
    assert_eq!(
        world.particle_snapshot(particle),
        Err(HandleError::WorldPoisoned)
    );
}

fn touching_world() -> World {
    let mut world = World::new().expect("world key remains available");
    world
        .set_continuous_physics_enabled(false)
        .expect("world configuration remains mutable");
    let static_body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("static body definition is valid"),
        )
        .expect("static body fits");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(1.5, 0.0), 0.0, true)
                .expect("dynamic body definition is valid"),
        )
        .expect("dynamic body fits");
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle geometry is valid"));
    let fixture = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition is valid");
    world
        .create_fixture(static_body, &fixture)
        .expect("static fixture fits");
    world
        .create_fixture(dynamic_body, &fixture)
        .expect("dynamic fixture fits");
    world
}

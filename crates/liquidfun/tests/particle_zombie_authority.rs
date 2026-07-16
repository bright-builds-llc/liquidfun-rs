//! Public regressions for storage-authoritative zombie lifecycle transitions.

use liquidfun::particle::{ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{DestroyedId, HandleError, NoDecisionHook, StepConfiguration, StepLimits, World};

fn positive_step() -> StepConfiguration {
    StepConfiguration::new(1.0, 8, 3).expect("test step configuration is valid")
}

#[test]
fn public_destruction_mark_sets_zombie_and_preserves_listener_flags_atomically() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default().with_flags(ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("particle fits")
        .created_particle();

    // Act
    let pending = world
        .mark_particle_for_destruction(particle)
        .expect("live particle becomes pending");
    let repeated = world.mark_particle_for_destruction(particle);
    let flags = world
        .particle_system_view(system)
        .expect("system remains live")
        .flags()[0];
    let report = world
        .compact_pending_particles(system)
        .expect("pending particle compacts");

    // Assert
    assert!(pending.flags().contains(ParticleFlags::ZOMBIE));
    assert!(
        pending
            .flags()
            .contains(ParticleFlags::DESTRUCTION_LISTENER)
    );
    assert!(flags.contains(ParticleFlags::ZOMBIE));
    assert!(flags.contains(ParticleFlags::DESTRUCTION_LISTENER));
    assert_eq!(repeated, Err(HandleError::PendingDelete));
    assert_eq!(report.len(), 1);
    assert_eq!(report[0].destroyed(), DestroyedId::Particle(particle));
}

#[test]
fn particle_created_with_zombie_compacts_on_next_fresh_positive_step() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("paused system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default().with_flags(ParticleFlags::ZOMBIE),
        )
        .expect("zombie particle fits")
        .created_particle();

    // Act
    let report = world
        .step(positive_step(), &mut NoDecisionHook, StepLimits::default())
        .expect("fresh lifecycle pass succeeds");

    // Assert
    assert_eq!(
        world.particle_snapshot(particle),
        Err(HandleError::StaleOrDestroyed)
    );
    assert!(report.lifecycle().is_empty());
}

#[test]
fn zombie_listener_occurrences_follow_ascending_old_rows_exactly_once() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system fits");
    let flags = ParticleFlags::ZOMBIE | ParticleFlags::DESTRUCTION_LISTENER;
    let flagged = world
        .create_particle_with_def(system, None, &ParticleDef::default().with_flags(flags))
        .expect("flagged zombie fits")
        .created_particle();
    let survivor = world
        .create_particle(system, None)
        .expect("survivor fits")
        .created_particle();
    let explicitly_marked = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default().with_flags(ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("explicit particle fits")
        .created_particle();
    world
        .mark_particle_for_destruction(explicitly_marked)
        .expect("explicit particle becomes pending");
    let pending_flags = world
        .particle_system_view(system)
        .expect("system remains live")
        .flags()
        .to_vec();

    // Act
    let report = world
        .step(positive_step(), &mut NoDecisionHook, StepLimits::default())
        .expect("fresh lifecycle pass succeeds");
    let destroyed = report
        .destructions()
        .iter()
        .map(liquidfun::DestructionRecord::destroyed)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        destroyed,
        vec![
            DestroyedId::Particle(flagged),
            DestroyedId::Particle(explicitly_marked),
        ]
    );
    assert!(pending_flags[0].contains(flags));
    assert!(pending_flags[2].contains(flags));
    assert_eq!(
        world.particle_snapshot(flagged),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world.particle_snapshot(explicitly_marked),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world
            .particle_snapshot(survivor)
            .expect("survivor remains live")
            .id(),
        survivor
    );
}

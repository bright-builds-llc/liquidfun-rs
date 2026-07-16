//! Synchronous capacity-eviction receipt regressions.

use liquidfun::particle::{ParticleDef, ParticleFlags, ParticleSystemDef};
use liquidfun::{HandleError, NoDecisionHook, StepConfiguration, StepLimits, World};

fn maximum_one_system(world: &mut World) -> liquidfun::ParticleSystemId {
    let definition = ParticleSystemDef::default()
        .with_maximum_count(1)
        .expect("one particle is a valid maximum")
        .with_destruction_by_age(true);
    world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits")
}

#[test]
fn requested_capacity_eviction_is_returned_synchronously_exactly_once() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = maximum_one_system(&mut world);
    let first_receipt = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default().with_flags(ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("first particle fits");
    let victim = first_receipt.created_particle();
    assert!(first_receipt.destruction_occurrences().is_empty());

    // Act
    let replacement_receipt = world
        .create_particle(system, None)
        .expect("replacement evicts the oldest particle");
    let replacement = replacement_receipt.created_particle();
    let first_inspection = replacement_receipt.destruction_occurrences().to_vec();
    let second_inspection = replacement_receipt.destruction_occurrences().to_vec();
    let next_step = world
        .step(
            StepConfiguration::new(1.0, 8, 3).expect("step configuration is valid"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("next eligible step succeeds");

    // Assert
    assert_eq!(first_inspection.len(), 1);
    assert_eq!(first_inspection[0].particle(), victim);
    assert_eq!(second_inspection, first_inspection);
    assert_eq!(
        world.particle_snapshot(victim),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world
            .particle_snapshot(replacement)
            .expect("replacement remains live")
            .id(),
        replacement
    );
    assert!(next_step.lifecycle().is_empty());
}

#[test]
fn unrequested_capacity_eviction_returns_no_occurrence() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = maximum_one_system(&mut world);
    let victim = world
        .create_particle(system, None)
        .expect("first particle fits")
        .created_particle();

    // Act
    let replacement_receipt = world
        .create_particle(system, None)
        .expect("replacement evicts the oldest particle");

    // Assert
    assert!(replacement_receipt.destruction_occurrences().is_empty());
    assert_eq!(
        world.particle_snapshot(victim),
        Err(HandleError::StaleOrDestroyed)
    );
    assert!(
        world
            .particle_snapshot(replacement_receipt.created_particle())
            .is_ok()
    );
}

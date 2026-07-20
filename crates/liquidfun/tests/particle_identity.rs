//! Black-box consumer evidence for stable particle identities and cleanup.

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource};
use liquidfun::{
    AssociationMap, DestroyedId, HandleError, ObjectSnapshot, ParticleGroupId, ParticleId,
    ParticleSystemId, World,
};

fn test_world() -> World {
    World::new().expect("test world key should remain available")
}

fn create_test_group(world: &mut World, system: ParticleSystemId) -> (ParticleGroupId, ParticleId) {
    let source =
        ParticleGroupSource::positions(vec![Vec2::ZERO]).expect("one finite position is valid");
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New);
    let group = world
        .create_particle_group(system, &recipe)
        .expect("particle group should fit");
    let particle = world
        .particle_group_view(group)
        .expect("particle group remains live")
        .member_ids()[0];
    (group, particle)
}

#[test]
fn particle_identity_survives_supported_group_removal() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let (group, particle) = create_test_group(&mut world, system);

    // Act
    let record = world
        .destroy_particle_group(group)
        .expect("particle group should be live");

    // Assert
    assert!(world.contains_particle(particle));
    assert_eq!(record.destroyed(), DestroyedId::ParticleGroup(group));
    assert!(matches!(
        record.snapshot(),
        ObjectSnapshot::ParticleGroup { particles, .. } if particles == &[particle]
    ));
}

#[test]
fn particle_system_cascade_invalidates_stable_ids_in_occurrence_order() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let (group, first) = create_test_group(&mut world, system);
    let second = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();

    // Act
    let records = world
        .destroy_particle_system(system)
        .expect("particle system should be live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::ParticleGroup(group),
            DestroyedId::Particle(first),
            DestroyedId::Particle(second),
            DestroyedId::ParticleSystem(system),
        ]
    );
    assert_eq!(
        world.destroy_particle(first),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world.destroy_particle(second),
        Err(HandleError::StaleOrDestroyed)
    );
}

#[test]
fn particle_associations_cleanup_by_stable_identity() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();
    let second = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();
    let mut labels = AssociationMap::<ParticleId, _>::new();
    labels.insert(first, "first");
    labels.insert(second, "second");
    let first_record = world
        .destroy_particle(first)
        .expect("particle should be live");

    // Act
    let removed = labels.cleanup_record(&first_record);

    // Assert
    assert_eq!(removed, Some("first"));
    assert_eq!(labels.get(&second), Some(&"second"));
}

use super::*;
use crate::particle::{
    ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource, ParticleSystemDef,
};

fn group_fixture(world: &mut World) -> (ParticleSystemId, ParticleGroupId) {
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("particle system fits");
    let source =
        ParticleGroupSource::positions(vec![Vec2::ZERO]).expect("one finite position is valid");
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New);
    let group = world
        .create_particle_group(system, &recipe)
        .expect("particle group fits");
    (system, group)
}

#[test]
fn locked_group_destruction_is_a_no_effect_error() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let (system, group) = group_fixture(&mut world);
    let before = world
        .particle_system_statistics(system)
        .expect("system remains live");
    world.step_state.set_locked_for_test(true);

    // Act
    let result = world.destroy_particle_group_particles(group, true);
    world.step_state.set_locked_for_test(false);

    // Assert
    assert_eq!(result, Err(CreateObjectError::WorldLocked));
    assert_eq!(
        world
            .particle_system_statistics(system)
            .expect("system remains unchanged"),
        before
    );
}

#[test]
fn poisoned_group_destruction_is_a_no_effect_error() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let (system, group) = group_fixture(&mut world);
    let before = world
        .particle_system_statistics(system)
        .expect("system remains live");
    world.step_state.set_poisoned_for_test(true);

    // Act
    let result = world.destroy_particle_group_particles(group, true);
    world.step_state.set_poisoned_for_test(false);

    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidHandle(HandleError::WorldPoisoned))
    );
    assert_eq!(
        world
            .particle_system_statistics(system)
            .expect("system remains unchanged"),
        before
    );
}

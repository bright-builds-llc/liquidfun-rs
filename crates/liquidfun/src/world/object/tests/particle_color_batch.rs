use super::*;
use crate::particle::{ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource};
use crate::{HandleError, ParticleColor, ParticleEditError};

const SPARK_RED: ParticleColor = ParticleColor::new(248, 113, 113, 255);
const FADED: ParticleColor = ParticleColor::new(10, 20, 30, 255);

fn colored_burst(color: ParticleColor) -> (World, ParticleSystemId, ParticleGroupId) {
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let source =
        ParticleGroupSource::positions(vec![Vec2::ZERO, Vec2::new(1.5, 0.0), Vec2::new(0.75, 1.3)])
            .expect("finite positions are valid");
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New).with_color(color);
    let group = world
        .create_particle_group(system, &recipe)
        .expect("particle group should fit");
    (world, system, group)
}

fn member_ids(world: &World, group: ParticleGroupId) -> Vec<ParticleId> {
    world
        .particle_group_view(group)
        .expect("particle group remains live")
        .member_ids()
        .to_vec()
}

fn member_colors(
    world: &World,
    system: ParticleSystemId,
    members: &[ParticleId],
) -> Option<Vec<ParticleColor>> {
    let view = world
        .particle_system_view(system)
        .expect("particle system remains live");
    let colors = view.maybe_colors()?;
    let copied = view
        .particle_ids()
        .iter()
        .zip(colors.iter().copied())
        .filter(|(id, _)| members.contains(id))
        .map(|(_, color)| color)
        .collect::<Vec<_>>();
    Some(copied)
}

#[test]
fn set_particle_colors_replaces_every_member_and_keeps_count() {
    // Arrange
    let (mut world, system, group) = colored_burst(SPARK_RED);
    let members = member_ids(&world, group);
    let count_before = world
        .particle_system_view(system)
        .expect("particle system remains live")
        .particle_ids()
        .len();
    let before = member_colors(&world, system, &members).expect("non-zero group color allocates");
    assert!(before.iter().all(|color| *color == SPARK_RED));

    // Act
    let result = world.set_particle_colors(&members, FADED);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        world
            .particle_system_view(system)
            .expect("particle system remains live")
            .particle_ids()
            .len(),
        count_before
    );
    let colors = member_colors(&world, system, &members).expect("color lane remains allocated");
    assert_eq!(colors.len(), members.len());
    assert!(colors.iter().all(|color| *color == FADED));
    assert!(colors.iter().all(|color| color.components()[3] == 255));
}

#[test]
fn set_particle_colors_empty_slice_leaves_colors_unchanged() {
    // Arrange
    let (mut world, system, group) = colored_burst(SPARK_RED);
    let members = member_ids(&world, group);
    let before = member_colors(&world, system, &members).expect("non-zero group color allocates");

    // Act
    let result = world.set_particle_colors(&[], FADED);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(member_colors(&world, system, &members), Some(before));
}

#[test]
fn set_particle_colors_zero_group_returns_missing_color_lane() {
    // Arrange
    let (mut world, system, group) = colored_burst(ParticleColor::ZERO);
    let members = member_ids(&world, group);
    assert!(
        world
            .particle_system_view(system)
            .expect("particle system remains live")
            .maybe_colors()
            .is_none()
    );

    // Act
    let result = world.set_particle_colors(&members, FADED);

    // Assert
    assert_eq!(result, Err(ParticleEditError::MissingColorLane));
    assert!(
        world
            .particle_system_view(system)
            .expect("particle system remains live")
            .maybe_colors()
            .is_none()
    );
}

#[test]
fn set_particle_colors_stale_id_leaves_stored_colors_unchanged() {
    // Arrange
    let (mut world, system, group) = colored_burst(SPARK_RED);
    let members = member_ids(&world, group);
    let stale = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();
    world
        .destroy_particle(stale)
        .expect("created particle is live");
    let mut ids = members.clone();
    ids.push(stale);
    let before = member_colors(&world, system, &members).expect("non-zero group color allocates");

    // Act
    let result = world.set_particle_colors(&ids, FADED);

    // Assert
    assert_eq!(
        result,
        Err(ParticleEditError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(member_colors(&world, system, &members), Some(before));
}

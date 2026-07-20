//! Black-box coverage for borrow-scoped particle views and editors.

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource};
use liquidfun::{
    AssociationMap, ParticleColor, ParticleDef, ParticleEditError, ParticleFlags, ParticleId, World,
};

fn particle_definition(
    position: Vec2,
    velocity: Vec2,
    color: ParticleColor,
    flags: ParticleFlags,
) -> ParticleDef {
    ParticleDef::default()
        .with_position(position)
        .expect("test position is finite")
        .with_velocity(velocity)
        .expect("test velocity is finite")
        .with_color(color)
        .with_flags(flags)
}

fn assert_root_view_reachable(_view: &liquidfun::ParticleSystemView<'_>) {}

#[test]
fn aggregate_view_exposes_semantic_lanes_and_associations() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first_definition = particle_definition(
        Vec2::new(1.0, 2.0),
        Vec2::new(3.0, 4.0),
        ParticleColor::ZERO,
        ParticleFlags::WALL,
    );
    let second_definition = particle_definition(
        Vec2::new(5.0, 6.0),
        Vec2::new(7.0, 8.0),
        ParticleColor::new(10, 20, 30, 40),
        ParticleFlags::VISCOUS,
    );
    let first_source = ParticleGroupSource::positions(vec![first_definition.position()])
        .expect("one finite position is valid");
    let first_recipe = ParticleGroupRecipe::new(first_source, ParticleGroupDestination::New)
        .with_particle_flags(first_definition.flags())
        .with_linear_velocity(first_definition.velocity())
        .expect("velocity is finite")
        .with_color(first_definition.color());
    let group = world
        .create_particle_group(system, &first_recipe)
        .expect("particle group should fit");
    let first = world
        .particle_group_view(group)
        .expect("group remains live")
        .member_ids()[0];
    let second = world
        .create_particle_with_def(system, Some(group), &second_definition)
        .expect("second particle should fit")
        .created_particle();
    let mut associations = AssociationMap::<ParticleId, _>::new();
    associations.insert(second, "second");

    // Act
    let view = world
        .particle_system_view(system)
        .expect("particle system should be live");
    assert_root_view_reachable(&view);
    let viewed_associations = view
        .user_associations(&associations)
        .map(|(id, maybe_value)| (id, maybe_value.copied()))
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(view.particle_ids(), &[first, second]);
    assert_eq!(
        view.positions(),
        &[Vec2::new(1.0, 2.0), Vec2::new(5.0, 6.0)]
    );
    assert_eq!(
        view.velocities(),
        &[Vec2::new(3.0, 4.0), Vec2::new(7.0, 8.0)]
    );
    assert_eq!(view.flags(), &[ParticleFlags::WALL, ParticleFlags::VISCOUS]);
    assert_eq!(view.group_ids(), &[Some(group), Some(group)]);
    assert_eq!(view.weights(), &[0.0, 0.0]);
    assert_eq!(
        view.maybe_colors(),
        Some([ParticleColor::ZERO, ParticleColor::new(10, 20, 30, 40)].as_slice())
    );
    assert_eq!(
        viewed_associations,
        vec![(first, None), (second, Some("second"))]
    );
    assert_eq!(view.particle_contacts().len(), 0);
    assert_eq!(view.body_contacts().len(), 0);
    assert_eq!(view.pairs().len(), 0);
    assert_eq!(view.triads().len(), 0);
    assert!(view.maybe_expiration_order().is_none());
}

#[test]
fn aggregate_view_preserves_stable_semantics_after_compaction() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(1.0, 0.0),
                Vec2::ZERO,
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("first particle should fit")
        .created_particle();
    let removed = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(2.0, 0.0),
                Vec2::ZERO,
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("middle particle should fit")
        .created_particle();
    let last = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(3.0, 0.0),
                Vec2::ZERO,
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("last particle should fit")
        .created_particle();
    world
        .mark_particle_for_destruction(removed)
        .expect("middle particle should be live");
    world
        .compact_pending_particles(system)
        .expect("pending particle should compact");

    // Act
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");

    // Assert
    assert_eq!(view.particle_ids(), &[first, last]);
    assert_eq!(
        view.positions(),
        &[Vec2::new(1.0, 0.0), Vec2::new(3.0, 0.0)]
    );
    assert!(view.maybe_colors().is_none());
}

#[test]
fn scoped_editor_commits_validated_kinematics_and_returns_coherent_view() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(1.0, 2.0),
                Vec2::new(3.0, 4.0),
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("particle should fit")
        .created_particle();

    // Act
    let returned = world
        .edit_particle(particle, |editor| {
            editor.set_position(Vec2::new(5.0, 6.0))?;
            editor.set_velocity(Vec2::new(7.0, 8.0))?;
            Ok("edited")
        })
        .expect("finite edit should commit");
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");

    // Assert
    assert_eq!(returned, "edited");
    assert_eq!(view.positions(), &[Vec2::new(5.0, 6.0)]);
    assert_eq!(view.velocities(), &[Vec2::new(7.0, 8.0)]);
    assert_eq!(view.particle_contacts().len(), 0);
    assert_eq!(view.body_contacts().len(), 0);
    assert_eq!(view.pairs().len(), 0);
    assert_eq!(view.triads().len(), 0);
}

#[test]
fn rejected_editor_candidate_leaves_particle_state_unchanged() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(1.0, 2.0),
                Vec2::new(3.0, 4.0),
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("particle should fit")
        .created_particle();

    // Act
    let result = world.edit_particle(particle, |editor| {
        editor.set_position(Vec2::new(f32::NAN, 6.0))?;
        Ok(())
    });
    let snapshot = world
        .particle_snapshot(particle)
        .expect("rejected edit preserves the particle");

    // Assert
    assert_eq!(result, Err(ParticleEditError::NonFinitePositionX));
    assert_eq!(snapshot.position(), Vec2::new(1.0, 2.0));
    assert_eq!(snapshot.velocity(), Vec2::new(3.0, 4.0));
}

#[test]
fn panicking_editor_closure_leaves_particle_state_unchanged() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &particle_definition(
                Vec2::new(1.0, 2.0),
                Vec2::new(3.0, 4.0),
                ParticleColor::ZERO,
                ParticleFlags::WATER,
            ),
        )
        .expect("particle should fit")
        .created_particle();

    // Act
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _result: Result<(), ParticleEditError> = world.edit_particle(particle, |editor| {
            editor.set_position(Vec2::new(5.0, 6.0))?;
            panic!("intentional editor panic");
        });
    }));
    let snapshot = world
        .particle_snapshot(particle)
        .expect("panicking edit preserves the particle");

    // Assert
    assert!(panic.is_err());
    assert_eq!(snapshot.position(), Vec2::new(1.0, 2.0));
    assert_eq!(snapshot.velocity(), Vec2::new(3.0, 4.0));
}

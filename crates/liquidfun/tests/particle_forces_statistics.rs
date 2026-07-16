//! Black-box particle force, impulse, and statistics regressions.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::{Vec2, settings};
use liquidfun::{
    BodyDef, FixtureDef, HandleError, NoDecisionHook, ParticleCapacity, ParticleDef, ParticleFlags,
    ParticleForceError, ParticleSystemDef, StepConfiguration, StepLimits, World,
};

fn particles(
    world: &mut World,
    system: liquidfun::ParticleSystemId,
    count: usize,
) -> Vec<liquidfun::ParticleId> {
    (0..count)
        .map(|_| {
            world
                .create_particle_with_def(system, None, &ParticleDef::default())
                .expect("particle fits")
                .created_particle()
        })
        .collect()
}

fn particle_state(world: &World, system: liquidfun::ParticleSystemId) -> (Vec<Vec2>, Vec<Vec2>) {
    let view = world
        .particle_system_view(system)
        .expect("system remains live");
    (view.forces().to_vec(), view.velocities().to_vec())
}

fn circle_fixture(world: &mut World, body: liquidfun::BodyId) {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test fixture geometry is valid"));
    world
        .create_fixture(
            body,
            &FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
                .expect("test fixture is valid"),
        )
        .expect("fixture fits");
}

fn colliding_particles_world(
    definition: &ParticleSystemDef,
) -> (
    World,
    liquidfun::ParticleSystemId,
    [liquidfun::ParticleId; 2],
) {
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(definition)
        .expect("particle system fits");
    let first = ParticleDef::default()
        .with_position(Vec2::new(-0.5, 0.0))
        .expect("position is finite")
        .with_velocity(Vec2::new(1.0, 0.0))
        .expect("velocity is finite");
    let second = ParticleDef::default()
        .with_position(Vec2::new(0.5, 0.0))
        .expect("position is finite")
        .with_velocity(Vec2::new(-1.0, 0.0))
        .expect("velocity is finite");
    let first = world
        .create_particle_with_def(system, None, &first)
        .expect("first particle fits")
        .created_particle();
    let second = world
        .create_particle_with_def(system, None, &second)
        .expect("second particle fits")
        .created_particle();
    (world, system, [first, second])
}

fn refresh_contacts(world: &mut World) {
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step is valid"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("contact refresh succeeds");
}

fn expected_collision_energy(
    world: &World,
    system: liquidfun::ParticleSystemId,
    definition: ParticleSystemDef,
) -> f32 {
    let view = world
        .particle_system_view(system)
        .expect("system remains live");
    let contact = view
        .particle_contacts()
        .next()
        .expect("contact refresh produced one contact");
    let [first_id, second_id] = contact.particles();
    let first_index = view
        .particle_ids()
        .iter()
        .position(|particle| *particle == first_id)
        .expect("first contact particle remains present");
    let second_index = view
        .particle_ids()
        .iter()
        .position(|particle| *particle == second_id)
        .expect("second contact particle remains present");
    let relative_velocity = view.velocities()[second_index] - view.velocities()[first_index];
    let normal_velocity = relative_velocity.dot(contact.normal());
    let diameter = 2.0 * definition.radius();
    let stride = settings::PARTICLE_STRIDE * diameter;
    let particle_mass = definition.density() * stride * stride;
    let sum_velocity_squared = normal_velocity * normal_velocity;
    0.5 * particle_mass * sum_velocity_squared
}

#[test]
fn forces_range_distributes_one_total_force_in_source_order() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particles = particles(&mut world, system, 3);

    // Act
    world
        .apply_particle_force_range(system, &particles, Vec2::new(12.0, -6.0))
        .expect("finite movable range accepts force");

    // Assert
    assert_eq!(
        world
            .particle_system_view(system)
            .expect("system remains live")
            .forces(),
        &[Vec2::new(4.0, -2.0); 3]
    );
}

#[test]
fn forces_singleton_matches_one_particle_range() {
    // Arrange
    let mut singleton_world = World::new().expect("world key remains available");
    let singleton_system = singleton_world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let singleton = particles(&mut singleton_world, singleton_system, 1)[0];
    let mut range_world = World::new().expect("world key remains available");
    let range_system = range_world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let range = particles(&mut range_world, range_system, 1);

    // Act
    singleton_world
        .apply_particle_force(singleton, Vec2::new(5.0, 7.0))
        .expect("finite movable particle accepts force");
    range_world
        .apply_particle_force_range(range_system, &range, Vec2::new(5.0, 7.0))
        .expect("singleton range accepts force");

    // Assert
    assert_eq!(
        singleton_world
            .particle_system_view(singleton_system)
            .expect("system remains live")
            .forces(),
        range_world
            .particle_system_view(range_system)
            .expect("system remains live")
            .forces()
    );
}

#[test]
fn forces_and_impulses_use_source_mass_and_range_scaling() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_density(2.0)
        .expect("test density is valid")
        .with_radius(0.5)
        .expect("test radius is valid");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particles = particles(&mut world, system, 3);
    let impulse = Vec2::new(13.5, -6.75);

    // Act
    world
        .apply_particle_linear_impulse_range(system, &particles, impulse)
        .expect("finite movable range accepts impulse");

    // Assert
    let diameter = 2.0 * definition.radius();
    let stride = settings::PARTICLE_STRIDE * diameter;
    let particle_mass = definition.density() * stride * stride;
    let total_mass = 3.0 * particle_mass;
    let expected = impulse / total_mass;
    assert_eq!(
        world
            .particle_system_view(system)
            .expect("system remains live")
            .velocities(),
        &[expected; 3]
    );
}

#[test]
fn forces_empty_and_noncontiguous_ranges_have_no_effect() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let particles = particles(&mut world, system, 3);
    let before = particle_state(&world, system);

    // Act
    let empty = world.apply_particle_force_range(system, &[], Vec2::new(1.0, 2.0));
    let skipped = world.apply_particle_linear_impulse_range(
        system,
        &[particles[0], particles[2]],
        Vec2::new(1.0, 2.0),
    );

    // Assert
    assert_eq!(empty, Err(ParticleForceError::EmptyRange));
    assert_eq!(skipped, Err(ParticleForceError::NonContiguousRange));
    assert_eq!(particle_state(&world, system), before);
}

#[test]
fn forces_validate_every_handle_before_mutation() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let source_particles = particles(&mut world, system, 2);
    let other_system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("second particle system fits");
    let other_particle = particles(&mut world, other_system, 1)[0];
    let mut foreign_world = World::new().expect("second world key remains available");
    let foreign_system = foreign_world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("foreign particle system fits");
    let foreign_particle = particles(&mut foreign_world, foreign_system, 1)[0];
    let before = particle_state(&world, system);

    // Act
    let cross_system = world.apply_particle_force_range(
        system,
        &[source_particles[0], other_particle],
        Vec2::new(3.0, 4.0),
    );
    let cross_world = world.apply_particle_force_range(
        system,
        &[source_particles[0], foreign_particle],
        Vec2::new(3.0, 4.0),
    );

    // Assert
    assert_eq!(
        cross_system,
        Err(ParticleForceError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert_eq!(
        cross_world,
        Err(ParticleForceError::InvalidHandle(HandleError::WrongWorld))
    );
    assert_eq!(particle_state(&world, system), before);
}

#[test]
fn forces_pending_and_stale_ids_have_no_effect() {
    // Arrange
    let mut pending_world = World::new().expect("world key remains available");
    let pending_system = pending_world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let pending_particles = particles(&mut pending_world, pending_system, 2);
    pending_world
        .mark_particle_for_destruction(pending_particles[1])
        .expect("particle becomes pending");
    let pending_before = particle_state(&pending_world, pending_system);
    let mut stale_world = World::new().expect("second world key remains available");
    let stale_system = stale_world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let stale_particles = particles(&mut stale_world, stale_system, 2);
    stale_world
        .destroy_particle(stale_particles[1])
        .expect("particle destruction succeeds");
    let stale_before = particle_state(&stale_world, stale_system);

    // Act
    let pending = pending_world.apply_particle_force_range(
        pending_system,
        &pending_particles,
        Vec2::new(3.0, 4.0),
    );
    let stale = stale_world.apply_particle_force(stale_particles[1], Vec2::new(3.0, 4.0));

    // Assert
    assert_eq!(
        pending,
        Err(ParticleForceError::InvalidHandle(
            HandleError::PendingDelete
        ))
    );
    assert_eq!(
        stale,
        Err(ParticleForceError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(
        particle_state(&pending_world, pending_system),
        pending_before
    );
    assert_eq!(particle_state(&stale_world, stale_system), stale_before);
}

#[test]
fn forces_wall_and_non_finite_inputs_have_no_effect() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system fits");
    let movable = world
        .create_particle_with_def(system, None, &ParticleDef::default())
        .expect("movable particle fits")
        .created_particle();
    let wall = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default().with_flags(ParticleFlags::WALL),
        )
        .expect("wall particle fits")
        .created_particle();
    let before = particle_state(&world, system);

    // Act
    let wall_error =
        world.apply_particle_force_range(system, &[movable, wall], Vec2::new(3.0, 4.0));
    let non_finite_force = world.apply_particle_force(movable, Vec2::new(f32::NAN, 1.0));
    let non_finite_impulse =
        world.apply_particle_linear_impulse(movable, Vec2::new(1.0, f32::INFINITY));

    // Assert
    assert_eq!(wall_error, Err(ParticleForceError::WallParticle));
    assert_eq!(non_finite_force, Err(ParticleForceError::NonFiniteX));
    assert_eq!(non_finite_impulse, Err(ParticleForceError::NonFiniteY));
    assert_eq!(particle_state(&world, system), before);
}

#[test]
fn forces_invalid_derived_mass_has_no_effect() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_radius(f32::MAX)
        .expect("finite positive radius is accepted at definition time");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system fits");
    let particle = particles(&mut world, system, 1)[0];
    let before = particle_state(&world, system);

    // Act
    let result = world.apply_particle_linear_impulse(particle, Vec2::new(1.0, 0.0));

    // Assert
    assert_eq!(result, Err(ParticleForceError::InvalidDistribution));
    assert_eq!(particle_state(&world, system), before);
}

#[test]
fn statistics_follow_contact_refresh_and_explicit_capacity() {
    // Arrange
    let definition = ParticleSystemDef::default()
        .with_capacity(ParticleCapacity::fixed(4).expect("test capacity is valid"))
        .expect("capacity and maximum are compatible")
        .with_maximum_count(3)
        .expect("test maximum is valid");
    let (mut world, system, _particles) = colliding_particles_world(&definition);

    // Act
    let before = world
        .particle_system_statistics(system)
        .expect("system remains live");
    refresh_contacts(&mut world);
    let contacted = world
        .particle_system_statistics(system)
        .expect("system remains live");
    let world_contacted = world.particle_world_statistics();
    let expected_collision_energy = expected_collision_energy(&world, system, definition);

    // Assert
    assert_eq!(before.particle_count(), 2);
    assert_eq!(before.particle_contact_count(), 0);
    assert_eq!(before.declared_capacity(), 4);
    assert_eq!(before.effective_capacity(), 3);
    assert_eq!(before.configured_maximum(), Some(3));
    assert_eq!(contacted.particle_contact_count(), 1);
    assert_eq!(contacted.body_contact_count(), 0);
    assert_eq!(
        contacted.collision_energy().to_bits(),
        expected_collision_energy.to_bits()
    );
    assert_eq!(contacted.stuck_candidates(), &[]);
    assert_eq!(world_contacted.system_count(), 1);
    assert_eq!(world_contacted.particle_count(), 2);
    assert_eq!(world_contacted.particle_contact_count(), 1);
    assert_eq!(
        world_contacted.collision_energy().to_bits(),
        contacted.collision_energy().to_bits()
    );
}

#[test]
fn statistics_follow_pause_compaction_and_teardown() {
    // Arrange
    let definition = ParticleSystemDef::default();
    let (mut world, system, [first, second]) = colliding_particles_world(&definition);
    refresh_contacts(&mut world);

    // Act
    world
        .set_particle_system_paused(system, true)
        .expect("pause succeeds");
    world
        .mark_particle_for_destruction(second)
        .expect("particle becomes pending");
    let pending = world
        .particle_system_statistics(system)
        .expect("system remains live");
    world
        .compact_pending_particles(system)
        .expect("compaction succeeds");
    let compacted = world
        .particle_system_statistics(system)
        .expect("system remains live");
    world
        .destroy_particle_system(system)
        .expect("teardown succeeds");
    let destroyed = world.particle_system_statistics(system);

    // Assert
    assert!(pending.is_paused());
    assert_eq!(pending.particle_count(), 2);
    assert_eq!(pending.pending_particle_count(), 1);
    assert_eq!(compacted.particle_count(), 1);
    assert_eq!(compacted.pending_particle_count(), 0);
    assert_eq!(compacted.particle_contact_count(), 0);
    assert_eq!(compacted.stuck_candidates(), &[]);
    assert_eq!(world.particle_world_statistics().system_count(), 0);
    assert_eq!(destroyed, Err(HandleError::StaleOrDestroyed));
    assert_eq!(first, compacted.particle_ids()[0]);
}

#[test]
fn statistics_expose_stuck_candidates_as_stable_ids() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("static body fits");
    circle_fixture(&mut world, body);
    circle_fixture(&mut world, body);
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_stuck_threshold(1))
        .expect("particle system fits");
    let particle = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(1.5, 0.0))
                .expect("position is finite"),
        )
        .expect("particle fits")
        .created_particle();
    let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step is valid");

    // Act
    world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("first contact refresh succeeds");
    world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("second contact refresh succeeds");
    let statistics = world
        .particle_system_statistics(system)
        .expect("system remains live");

    // Assert
    assert_eq!(statistics.body_contact_count(), 2);
    assert_eq!(statistics.stuck_candidates(), &[particle]);
    assert_eq!(world.particle_world_statistics().stuck_candidate_count(), 1);
}

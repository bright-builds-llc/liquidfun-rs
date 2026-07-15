//! Black-box evidence for authoritative world-owned particle systems.

use std::any::TypeId;

use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleColor, ParticleDef, ParticleFlags, ParticleSnapshot, ParticleSystemDef,
    ParticleSystemSnapshot,
};
use liquidfun::{HandleError, ParticleId, World};

fn test_world() -> World {
    World::new().expect("test world key should remain available")
}

#[test]
fn public_particle_object_types_are_reachable_from_module_and_root() {
    // Arrange / Act / Assert
    assert_eq!(
        TypeId::of::<ParticleSnapshot>(),
        TypeId::of::<liquidfun::ParticleSnapshot>()
    );
    assert_eq!(
        TypeId::of::<ParticleSystemSnapshot>(),
        TypeId::of::<liquidfun::ParticleSystemSnapshot>()
    );
}

#[test]
fn system_configuration_and_particle_state_share_one_public_lifecycle() {
    // Arrange
    let mut world = test_world();
    let system_definition = ParticleSystemDef::default()
        .with_density(2.0)
        .expect("density is valid")
        .with_paused(true);
    let particle_definition = ParticleDef::default()
        .with_position(Vec2::new(3.0, 4.0))
        .expect("position is valid")
        .with_velocity(Vec2::new(-2.0, 1.0))
        .expect("velocity is valid")
        .with_color(ParticleColor::new(1, 2, 3, 4))
        .with_flags(ParticleFlags::WALL | ParticleFlags::VISCOUS);

    // Act
    let system = world
        .create_particle_system_with_def(&system_definition)
        .expect("particle system should fit");
    let particle = world
        .create_particle_with_def(system, None, &particle_definition)
        .expect("particle should fit");
    world
        .set_particle_system_paused(system, false)
        .expect("system should remain live");
    let system_snapshot = world
        .particle_system_snapshot(system)
        .expect("system should remain live");
    let particle_snapshot = world
        .particle_snapshot(particle)
        .expect("particle should remain live");

    // Assert
    assert_eq!(
        system_snapshot.definition().density().to_bits(),
        2.0_f32.to_bits()
    );
    assert!(!system_snapshot.is_paused());
    assert_eq!(system_snapshot.particle_count(), 1);
    assert_eq!(particle_snapshot.id(), particle);
    assert_eq!(particle_snapshot.system(), system);
    assert_eq!(particle_snapshot.position(), Vec2::new(3.0, 4.0));
    assert_eq!(particle_snapshot.velocity(), Vec2::new(-2.0, 1.0));
    assert_eq!(
        particle_snapshot.flags(),
        ParticleFlags::WALL | ParticleFlags::VISCOUS
    );
    assert_eq!(particle_snapshot.color(), ParticleColor::new(1, 2, 3, 4));
}

#[test]
fn particle_system_enumeration_is_newest_first() {
    // Arrange
    let mut world = test_world();
    let first = world
        .create_particle_system()
        .expect("first particle system should fit");
    let second = world
        .create_particle_system()
        .expect("second particle system should fit");
    let third = world
        .create_particle_system()
        .expect("third particle system should fit");

    // Act
    let systems = world.particle_system_ids().collect::<Vec<_>>();

    // Assert
    assert_eq!(systems, vec![third, second, first]);
}

#[test]
fn pending_and_stale_particle_states_are_distinct() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle(system, None)
        .expect("particle should fit");

    // Act
    let marked = world
        .mark_particle_for_destruction(particle)
        .expect("live particle should become pending");
    let pending = world.particle_snapshot(particle);
    let records = world
        .compact_pending_particles(system)
        .expect("pending compaction should succeed");
    let stale = world.particle_snapshot(particle);

    // Assert
    assert_eq!(marked.id(), particle);
    assert_eq!(pending, Err(HandleError::PendingDelete));
    assert_eq!(records.len(), 1);
    assert_eq!(stale, Err(HandleError::StaleOrDestroyed));
    assert!(!world.contains_particle(particle));
}

fn _assert_particle_id_is_send_sync(_: ParticleId) {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ParticleId>();
}

//! Black-box evidence for authoritative world-owned particle systems.

use std::any::TypeId;

use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleCapacity, ParticleColor, ParticleDef, ParticleFlags, ParticleSnapshot,
    ParticleSystemDef, ParticleSystemSnapshot,
};
use liquidfun::{
    ArenaInsertError, AssociationMap, BodyDef, CreateObjectError, DestroyedId, HandleError,
    ObjectSnapshot, ParticleId, World,
};

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

#[test]
fn particle_snapshot_distinguishes_wrong_world_and_wrong_system() {
    // Arrange
    let mut first_world = test_world();
    let second_world = test_world();
    let owner = first_world
        .create_particle_system()
        .expect("owner system should fit");
    let other_system = first_world
        .create_particle_system()
        .expect("other system should fit");
    let particle = first_world
        .create_particle(owner, None)
        .expect("particle should fit");

    // Act
    let wrong_system = first_world.particle_snapshot_in_system(other_system, particle);
    let wrong_world = second_world.particle_snapshot(particle);

    // Assert
    assert_eq!(wrong_system, Err(HandleError::WrongParticleSystem));
    assert_eq!(wrong_world, Err(HandleError::WrongWorld));
}

#[test]
fn survivor_identity_and_state_remain_stable_after_pending_compaction() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let removed = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(-3.0, 2.0))
                .expect("position is valid"),
        )
        .expect("particle should fit");
    let survivor = world
        .create_particle_with_def(
            system,
            None,
            &ParticleDef::default()
                .with_position(Vec2::new(7.0, 5.0))
                .expect("position is valid")
                .with_velocity(Vec2::new(2.0, -1.0))
                .expect("velocity is valid"),
        )
        .expect("particle should fit");
    let before = world
        .particle_snapshot(survivor)
        .expect("survivor should be live");
    world
        .mark_particle_for_destruction(removed)
        .expect("particle should become pending");

    // Act
    let records = world
        .compact_pending_particles(system)
        .expect("compaction should succeed");
    let after = world
        .particle_snapshot(survivor)
        .expect("survivor should remain live");

    // Assert
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].destroyed(), DestroyedId::Particle(removed));
    assert_eq!(after, before);
    assert_eq!(after.id(), survivor);
}

#[test]
fn destroyed_slot_reuse_does_not_resurrect_stale_particle_identity() {
    // Arrange
    let mut world = test_world();
    let capacity = ParticleCapacity::fixed(1).expect("capacity is valid");
    let definition = ParticleSystemDef::default()
        .with_capacity(capacity)
        .expect("capacity matches the system maximum");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system should fit");
    let stale = world
        .create_particle(system, None)
        .expect("particle should fit");
    world
        .destroy_particle(stale)
        .expect("particle should be live");

    // Act
    let replacement = world
        .create_particle(system, None)
        .expect("vacated slot should be reusable");

    // Assert
    assert_ne!(replacement, stale);
    assert_eq!(
        world.particle_snapshot(stale),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world
            .particle_snapshot(replacement)
            .expect("replacement should be live")
            .id(),
        replacement
    );
}

#[test]
fn rejected_capacity_growth_preserves_diagnostic_identity_sequence() {
    // Arrange
    let mut world = test_world();
    let capacity = ParticleCapacity::fixed(1).expect("capacity is valid");
    let definition = ParticleSystemDef::default()
        .with_capacity(capacity)
        .expect("capacity matches the system maximum");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system should fit");
    let particle = world
        .create_particle(system, None)
        .expect("first particle should fit");

    // Act
    let rejected = world.create_particle(system, None);
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit after rejected particle");
    let particle_record = world
        .destroy_particle(particle)
        .expect("particle should be live");
    let body_records = world.destroy_body(body).expect("body should be live");

    // Assert
    assert_eq!(
        rejected,
        Err(CreateObjectError::Arena(
            ArenaInsertError::CapacityExceeded { limit: 1 }
        ))
    );
    assert_eq!(body_records.len(), 1);
    assert_eq!(
        body_records[0].diagnostic_id(),
        particle_record.diagnostic_id() + 1
    );
}

#[test]
fn pending_particle_association_cleanup_waits_for_compaction_record() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle(system, None)
        .expect("particle should fit");
    let mut labels = AssociationMap::<ParticleId, _>::new();
    labels.insert(particle, "pending");

    // Act
    world
        .mark_particle_for_destruction(particle)
        .expect("particle should become pending");
    let before_compaction = labels.get(&particle).copied();
    let records = world
        .compact_pending_particles(system)
        .expect("compaction should succeed");
    let removed = labels.cleanup(&records);

    // Assert
    assert_eq!(before_compaction, Some("pending"));
    assert_eq!(removed, vec!["pending"]);
    assert!(labels.is_empty());
}

#[test]
fn system_teardown_captures_authoritative_membership_and_preserves_other_systems() {
    // Arrange
    let mut world = test_world();
    let survivor_system = world
        .create_particle_system()
        .expect("survivor system should fit");
    let survivor = world
        .create_particle(survivor_system, None)
        .expect("survivor particle should fit");
    let removed_system = world
        .create_particle_system()
        .expect("removed system should fit");
    let group = world
        .create_particle_group(removed_system)
        .expect("particle group should fit");
    let grouped = world
        .create_particle(removed_system, Some(group))
        .expect("grouped particle should fit");
    let ungrouped = world
        .create_particle(removed_system, None)
        .expect("ungrouped particle should fit");
    let pending = world
        .create_particle(removed_system, None)
        .expect("pending particle should fit");
    world
        .mark_particle_for_destruction(pending)
        .expect("particle should become pending");

    // Act
    let records = world
        .destroy_particle_system(removed_system)
        .expect("system should be live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::ParticleGroup(group),
            DestroyedId::Particle(grouped),
            DestroyedId::Particle(ungrouped),
            DestroyedId::Particle(pending),
            DestroyedId::ParticleSystem(removed_system),
        ]
    );
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleSystem { groups, particles })
            if groups == &[group] && particles == &[grouped, ungrouped, pending]
    ));
    assert_eq!(
        world.particle_system_ids().collect::<Vec<_>>(),
        vec![survivor_system]
    );
    assert_eq!(
        world
            .particle_snapshot(survivor)
            .expect("other-system particle should survive")
            .system(),
        survivor_system
    );
    assert_eq!(
        world.particle_snapshot(grouped),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(
        world.particle_snapshot(pending),
        Err(HandleError::StaleOrDestroyed)
    );
}

#[test]
fn group_teardown_clears_particle_membership_before_system_teardown() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let group = world
        .create_particle_group(system)
        .expect("particle group should fit");
    let particle = world
        .create_particle(system, Some(group))
        .expect("particle should fit");

    // Act
    let group_record = world
        .destroy_particle_group(group)
        .expect("group should be live");
    let particle_after = world
        .particle_snapshot(particle)
        .expect("particle should survive group destruction");
    let system_records = world
        .destroy_particle_system(system)
        .expect("system should be live");

    // Assert
    assert!(matches!(
        group_record.snapshot(),
        ObjectSnapshot::ParticleGroup {
            system: snapshot_system,
            particles,
        } if *snapshot_system == system && particles == &[particle]
    ));
    assert_eq!(particle_after.maybe_group(), None);
    assert!(matches!(
        system_records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleSystem { groups, particles })
            if groups.is_empty() && particles == &[particle]
    ));
    assert!(matches!(
        system_records.first().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Particle {
            system: snapshot_system,
            maybe_group,
        }) if *snapshot_system == system && maybe_group.is_none()
    ));
}

fn _assert_particle_id_is_send_sync(_: ParticleId) {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ParticleId>();
}

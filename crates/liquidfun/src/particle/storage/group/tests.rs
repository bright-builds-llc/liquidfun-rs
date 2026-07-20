use crate::identity::{HandleIdentity, Identity, WorldKey};
use crate::math::Rotation;

use super::*;
use crate::particle::storage::lanes::{ParticleContact, ParticlePair};
use crate::particle::storage::{ParticleIndex, ParticleInput};

fn identities() -> (ParticleSystemId, ParticleGroupId) {
    let world = WorldKey::fresh().expect("test world key remains available");
    (
        ParticleSystemId::from_identity(Identity::new(world, 0, 0)),
        ParticleGroupId::from_identity(Identity::new(world, 1, 0)),
    )
}

fn limits() -> VoronoiLimits {
    VoronoiLimits::new(64, 4_096, 16_384, 2_000_000, 8_192)
}

fn grouped_storage(
    groups_and_values: &[(ParticleGroupId, f32)],
    capacity: usize,
) -> ParticleStorage {
    let system = ParticleSystemId::from_identity(Identity::new(
        groups_and_values[0].0.identity().world(),
        0,
        0,
    ));
    let mut storage =
        ParticleStorage::new(system.identity().world(), system, 0, capacity, capacity)
            .expect("test storage contract is valid");
    for (group, value) in groups_and_values {
        storage
            .create(ParticleInput {
                position: Vec2::new(*value, 0.0),
                velocity: Vec2::new(0.0, *value),
                flags: ParticleFlags::WATER,
                maybe_group: Some(*group),
                maybe_color: None,
                maybe_user_association: None,
                maybe_expiration_time: None,
            })
            .expect("grouped fixture particle fits");
    }
    storage
}

#[test]
fn cache_invalidation_clears_only_the_source_timestamp() {
    // Arrange
    let mut cache = GroupStatisticsCache {
        maybe_source_timestamp: Some(7),
        mass: 2.0,
        center: Vec2::new(1.0, 2.0),
        linear_velocity: Vec2::new(3.0, 4.0),
        inertia: 5.0,
        angular_velocity: 6.0,
    };

    // Act
    cache.invalidate();

    // Assert
    assert_eq!(cache.maybe_source_timestamp, None);
    assert_eq!(cache.mass.to_bits(), 2.0_f32.to_bits());
    assert_eq!(cache.center, Vec2::new(1.0, 2.0));
}

#[test]
fn empty_transition_resets_every_aggregate_to_positive_zero() {
    // Arrange
    let (system, group) = identities();
    let mut record = GroupRecord::new(group, system, 2..4);
    record.statistics = GroupStatisticsCache {
        maybe_source_timestamp: Some(9),
        mass: 2.0,
        center: Vec2::new(1.0, 2.0),
        linear_velocity: Vec2::new(3.0, 4.0),
        inertia: 5.0,
        angular_velocity: 6.0,
    };

    // Act
    record.retain_empty_after_member_removal();

    // Assert
    assert_eq!(record.range(), 0..0);
    assert_eq!(record.statistics, GroupStatisticsCache::INVALIDATED_ZERO);
    assert!(
        record
            .internal_flags
            .contains(InternalGroupFlags::WILL_BE_DESTROYED)
    );
    assert_eq!(record.validate(system, 4), Ok(()));
}

#[test]
fn non_finite_transform_or_statistics_are_rejected() {
    // Arrange
    let (system, group) = identities();
    let mut invalid_transform = GroupRecord::new(group, system, 0..1);
    invalid_transform.transform = Transform::new(Vec2::ZERO, Rotation::from_angle(f32::NAN));
    let mut invalid_cache = GroupRecord::new(group, system, 0..1);
    invalid_cache.statistics.mass = f32::INFINITY;

    // Act
    let transform_result = invalid_transform.validate(system, 1);
    let cache_result = invalid_cache.validate(system, 1);

    // Assert
    assert_eq!(
        transform_result,
        Err(ParticleStorageError::InvalidGroupRange)
    );
    assert_eq!(cache_result, Err(ParticleStorageError::InvalidGroupRange));
}

#[test]
fn unknown_internal_state_cannot_enter_the_authoritative_table() {
    // Arrange
    let (system, group) = identities();
    let mut record = GroupRecord::new(group, system, 0..1);
    record.internal_flags = InternalGroupFlags(0b0000_0100);

    // Act
    let result = record.validate(system, 1);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidGroupRange));
}

#[test]
fn storage_exposes_individual_public_flags_in_stable_record_order() {
    // Arrange
    let (system, first_group) = identities();
    let second_group =
        ParticleGroupId::from_identity(Identity::new(system.identity().world(), 2, 0));
    let mut storage = ParticleStorage::new(system.identity().world(), system, 0, 4, 4)
        .expect("test storage contract is valid");
    for group in [first_group, second_group] {
        storage
            .create(ParticleInput {
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                flags: ParticleFlags::WATER,
                maybe_group: Some(group),
                maybe_color: None,
                maybe_user_association: None,
                maybe_expiration_time: None,
            })
            .expect("source-ordered grouped particle fits");
    }
    storage.group_records[0].flags = ParticleGroupFlags::RIGID;
    storage.group_records[1].flags = ParticleGroupFlags::SOLID;

    // Act
    let first_scan = storage.group_flags().collect::<Vec<_>>();
    let second_scan = storage.group_flags().collect::<Vec<_>>();

    // Assert
    assert_eq!(
        first_scan,
        vec![ParticleGroupFlags::RIGID, ParticleGroupFlags::SOLID]
    );
    assert_eq!(second_scan, first_scan);
}

#[test]
fn solid_transition_schedules_depth_and_allocates_the_private_lane() {
    // Arrange
    let (system, group) = identities();
    let mut storage = grouped_storage(&[(group, 0.0)], 4);
    assert_eq!(storage.system(), system);

    // Act
    storage
        .set_group_flags_internal(group, ParticleGroupFlags::SOLID)
        .expect("solid transition should preflight");

    // Assert
    assert!(
        storage.group_records[0]
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
    );
    assert_eq!(storage.maybe_depths(), Some(&[0.0][..]));
}

#[test]
fn group_flag_change_invalidates_the_statistics_timestamp() {
    // Arrange
    let (_system, group) = identities();
    let mut storage = grouped_storage(&[(group, 0.0)], 4);
    storage
        .update_group_statistics(group, 1.0, 5)
        .expect("fixture statistics should compute");

    // Act
    storage
        .set_group_flags_internal(group, ParticleGroupFlags::RIGID)
        .expect("finite group flags should commit");

    // Assert
    assert_eq!(
        storage.group_records[0].statistics.maybe_source_timestamp,
        None
    );
}

#[test]
fn particle_flag_change_invalidates_only_its_group_statistics() {
    // Arrange
    let (system, first_group) = identities();
    let second_group =
        ParticleGroupId::from_identity(Identity::new(system.identity().world(), 2, 0));
    let mut storage = grouped_storage(&[(first_group, 0.0), (second_group, 1.0)], 4);
    for group in [first_group, second_group] {
        storage
            .update_group_statistics(group, 1.0, 8)
            .expect("fixture statistics should compute");
    }
    let first_particle = storage.particle_ids()[0];

    // Act
    storage
        .set_particle_flags_internal(first_particle, ParticleFlags::VISCOUS)
        .expect("particle flag change should commit");

    // Assert
    assert_eq!(
        storage.group_records[0].statistics.maybe_source_timestamp,
        None
    );
    assert_eq!(
        storage.group_records[1].statistics.maybe_source_timestamp,
        Some(8)
    );
}

#[test]
fn multi_group_depth_uses_contact_order_and_clears_schedule_after_commit() {
    // Arrange
    let (system, first_group) = identities();
    let second_group =
        ParticleGroupId::from_identity(Identity::new(system.identity().world(), 2, 0));
    let mut storage = grouped_storage(
        &[
            (first_group, 0.0),
            (first_group, 1.0),
            (first_group, 2.0),
            (second_group, 3.0),
            (second_group, 4.0),
        ],
        8,
    );
    for record in &mut storage.group_records {
        record.flags = ParticleGroupFlags::SOLID;
        record
            .internal_flags
            .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    }
    storage
        .solver_state
        .refresh_group_flags(&storage.group_records);
    storage.particle_contacts = vec![
        ParticleContact {
            indices: [ParticleIndex(0), ParticleIndex(1)],
            flags: ParticleFlags::WATER,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        },
        ParticleContact {
            indices: [ParticleIndex(1), ParticleIndex(2)],
            flags: ParticleFlags::WATER,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        },
        ParticleContact {
            indices: [ParticleIndex(3), ParticleIndex(4)],
            flags: ParticleFlags::WATER,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        },
        ParticleContact {
            indices: [ParticleIndex(2), ParticleIndex(3)],
            flags: ParticleFlags::WATER,
            weight: 1.0,
            normal: Vec2::new(1.0, 0.0),
        },
    ];

    // Act
    storage
        .compute_solid_depth(2.0)
        .expect("finite scheduled depth should compute");

    // Assert
    assert_eq!(storage.maybe_depths(), Some(&[0.0, 1.0, 0.0, 0.0, 0.0][..]));
    assert!(storage.group_records.iter().all(|record| {
        !record
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
    }));
}

#[test]
fn failed_depth_update_preserves_lane_and_schedule_exactly() {
    // Arrange
    let (_system, group) = identities();
    let mut storage = grouped_storage(&[(group, 0.0), (group, 1.0)], 4);
    storage.group_records[0].flags = ParticleGroupFlags::SOLID;
    storage.group_records[0]
        .internal_flags
        .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    storage
        .solver_state
        .refresh_group_flags(&storage.group_records);
    let before = storage.clone();

    // Act
    let result = storage.compute_solid_depth(f32::NAN);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidLaneBundle));
    assert!(storage == before);
}

#[test]
fn reactive_commit_keeps_first_duplicate_and_clears_flags_only_on_success() {
    // Arrange
    let (_system, group) = identities();
    let mut storage = grouped_storage(&[(group, 0.0), (group, 1.0), (group, 2.0)], 8);
    storage.flags[0] = ParticleFlags::SPRING | ParticleFlags::REACTIVE;
    storage.flags[1] = ParticleFlags::SPRING;
    storage.flags[2] = ParticleFlags::SPRING;
    storage.solver_state.refresh_particle_flags(&storage.flags);
    storage.particle_contacts = vec![
        ParticleContact {
            indices: [ParticleIndex(1), ParticleIndex(2)],
            flags: ParticleFlags::SPRING,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        },
        ParticleContact {
            indices: [ParticleIndex(0), ParticleIndex(1)],
            flags: ParticleFlags::SPRING,
            weight: 0.5,
            normal: Vec2::new(1.0, 0.0),
        },
    ];
    storage.pairs = vec![ParticlePair {
        indices: [ParticleIndex(0), ParticleIndex(1)],
        flags: ParticleFlags::SPRING,
        strength: 1.0,
        distance: 99.0,
    }];

    // Act
    storage
        .regenerate_reactive_topology(1.0, limits())
        .expect("reactive topology should commit atomically");

    // Assert
    assert_eq!(storage.pairs.len(), 1);
    assert_eq!(storage.pairs[0].distance.to_bits(), 99.0_f32.to_bits());
    assert!(
        storage
            .flags
            .iter()
            .all(|flags| !flags.contains(ParticleFlags::REACTIVE))
    );
}

#[test]
fn failed_reactive_generation_preserves_topology_flags_and_cache() {
    // Arrange
    let (_system, group) = identities();
    let mut storage = grouped_storage(&[(group, 0.0), (group, 0.0)], 4);
    storage.flags[0] = ParticleFlags::SPRING | ParticleFlags::REACTIVE;
    storage.flags[1] = ParticleFlags::SPRING;
    storage.solver_state.refresh_particle_flags(&storage.flags);
    storage.particle_contacts = vec![ParticleContact {
        indices: [ParticleIndex(0), ParticleIndex(1)],
        flags: ParticleFlags::SPRING,
        weight: 1.0,
        normal: Vec2::new(1.0, 0.0),
    }];
    storage.group_records[0].statistics.maybe_source_timestamp = Some(3);
    let before = storage.clone();

    // Act
    let result = storage.regenerate_reactive_topology(1.0, limits());

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidLaneBundle));
    assert!(storage == before);
}

#[test]
fn statistics_follow_source_order_and_cache_by_timestamp() {
    // Arrange
    let (_system, group) = identities();
    let mut storage = grouped_storage(&[(group, 1.0), (group, 3.0)], 4);
    let first_particle = storage.particle_ids()[0];

    // Act
    let first = storage
        .update_group_statistics(group, 2.0, 7)
        .expect("finite statistics should compute");
    storage.positions[0] = Vec2::new(99.0, 0.0);
    let cached = storage
        .update_group_statistics(group, 2.0, 7)
        .expect("same timestamp should reuse cache");
    storage
        .set_position(first_particle, Vec2::new(5.0, 0.0))
        .expect("position edit should invalidate cache");
    let refreshed = storage
        .update_group_statistics(group, 2.0, 7)
        .expect("invalidated same-timestamp cache should recompute");

    // Assert
    assert_eq!(first.mass.to_bits(), 4.0_f32.to_bits());
    assert_eq!(first.center, Vec2::new(2.0, 0.0));
    assert_eq!(first.linear_velocity, Vec2::new(0.0, 2.0));
    assert_eq!(first.inertia.to_bits(), 4.0_f32.to_bits());
    assert_eq!(first.angular_velocity.to_bits(), 1.0_f32.to_bits());
    assert_eq!(cached, first);
    assert_eq!(refreshed.center, Vec2::new(4.0, 0.0));
}

#[test]
fn empty_statistics_are_exact_zero_and_rigid_state_retains_transform() {
    // Arrange
    let (system, group) = identities();
    let mut storage = ParticleStorage::new(system.identity().world(), system, 0, 2, 2)
        .expect("test storage contract is valid");
    let mut record = GroupRecord::new(group, system, 0..0);
    record.flags = ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY;
    record.transform = Transform::new(Vec2::new(3.0, 4.0), Rotation::from_angle(0.25));
    storage.group_records.push(record);
    storage
        .solver_state
        .refresh_group_flags(&storage.group_records);

    // Act
    let state = storage
        .rigid_group_state(group, 1.0, 11)
        .expect("retained empty rigid state should remain finite");

    // Assert
    assert_eq!(state.range, 0..0);
    assert_eq!(state.transform, record.transform);
    assert_eq!(
        state.statistics,
        GroupStatisticsCache {
            maybe_source_timestamp: Some(11),
            ..GroupStatisticsCache::INVALIDATED_ZERO
        }
    );
}

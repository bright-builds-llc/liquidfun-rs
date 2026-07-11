use super::*;

fn input(value: i32) -> ParticleInput {
    ParticleInput {
        position: [value, -value],
        velocity: [value + 1, value + 2],
        flags: u32::try_from(value).expect("test values are non-negative"),
        group: 0,
        maybe_color: Some([u8::try_from(value).expect("test values fit u8"); 4]),
        maybe_lifetime: Some(u32::try_from(value).expect("test values fit u32") + 10),
    }
}

fn system(world: WorldKey, slot: usize) -> ParticleSystemId {
    ParticleSystemId::from_identity(Identity::new(world, slot, 0))
}

fn storage(world: WorldKey, system_slot: usize, identity_base: usize) -> ParticleStorage {
    ParticleStorage::new(world, system(world, system_slot), identity_base, 4, 4)
        .expect("test storage contract is valid")
}

#[test]
fn stable_id_survives_group_rotation() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut storage = storage(world, 0, 0);
    let first = storage.create(input(1)).expect("first particle fits");
    let second = storage.create(input(2)).expect("second particle fits");
    let third = storage.create(input(3)).expect("third particle fits");

    // Act
    storage.rotate_rows(0, 1, 3).expect("rotation is valid");

    // Assert
    assert_eq!(storage.input(first), Ok(input(1)));
    assert_eq!(storage.input(second), Ok(input(2)));
    assert_eq!(storage.input(third), Ok(input(3)));
}

#[test]
fn cross_system_id_is_rejected_before_dense_lookup() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut first = storage(world, 0, 0);
    let second = storage(world, 1, 4);
    let id = first.create(input(1)).expect("particle fits");

    // Act
    let result = second.input(id);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::WrongParticleSystem));
}

#[test]
fn pending_delete_rejects_mutation_but_preserves_snapshot() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut storage = storage(world, 0, 0);
    let id = storage.create(input(7)).expect("particle fits");

    // Act
    let snapshot = storage
        .mark_delete(id)
        .expect("live particle can be marked");
    let mutation = storage.set_position(id, [99, 99]);

    // Assert
    assert_eq!(
        snapshot,
        ParticleSnapshot {
            id,
            input: input(7)
        }
    );
    assert_eq!(mutation, Err(ParticleStorageError::PendingDelete));
}

#[test]
fn compacted_id_is_stale_and_snapshot_remains_owned() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut storage = storage(world, 0, 0);
    let id = storage.create(input(4)).expect("particle fits");
    storage
        .mark_delete(id)
        .expect("live particle can be marked");

    // Act
    let destroyed = storage.compact_pending().expect("compaction is valid");

    // Assert
    assert_eq!(
        destroyed,
        vec![ParticleSnapshot {
            id,
            input: input(4)
        }]
    );
    assert_eq!(
        storage.input(id),
        Err(ParticleStorageError::StaleOrDestroyed)
    );
}

#[test]
fn declared_capacity_does_not_grow_implicitly() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut storage = ParticleStorage::new(world, system(world, 0), 0, 4, 1)
        .expect("test storage contract is valid");
    storage.create(input(1)).expect("declared row fits");

    // Act
    let result = storage.create(input(2));

    // Assert
    assert_eq!(
        result,
        Err(ParticleStorageError::CapacityExceeded { limit: 1 })
    );
    assert!(storage.dense_to_id.capacity() >= storage.declared_capacity);
}

#[test]
fn retired_identity_reports_exhaustion_without_resurrection() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key remains available");
    let mut storage = ParticleStorage::new(world, system(world, 0), 0, 1, 1)
        .expect("test storage contract is valid");
    storage.identities.push(IdentityEntry {
        generation: u64::MAX,
        state: IdentityState::Vacant,
    });
    storage.free_identity_slots.push(0);
    let id = storage
        .create(input(1))
        .expect("maximum generation can be live once");
    storage
        .mark_delete(id)
        .expect("live particle can be marked");
    storage.compact_pending().expect("compaction is valid");

    // Act
    let result = storage.create(input(2));

    // Assert
    assert_eq!(result, Err(ParticleStorageError::IdentityExhausted));
    assert_eq!(
        storage.input(id),
        Err(ParticleStorageError::StaleOrDestroyed)
    );
}

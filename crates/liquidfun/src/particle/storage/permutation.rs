use super::*;

fn input(value: i32, group: u16, optional: bool) -> ParticleInput {
    ParticleInput {
        position: [value, -value],
        velocity: [value + 10, value + 20],
        flags: u32::try_from(value).expect("test values are non-negative"),
        group,
        maybe_color: optional
            .then_some([u8::try_from(value).expect("test values fit in a color component"); 4]),
        maybe_lifetime: optional
            .then_some(u32::try_from(value).expect("test values are non-negative") + 100),
    }
}

fn storage() -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, 8, 8).expect("test storage contract is valid")
}

fn input_with_optional_defaults(value: i32, group: u16) -> ParticleInput {
    let mut value = input(value, group, false);
    value.maybe_color = Some([0; 4]);
    value.maybe_lifetime = Some(0);
    value
}

fn populated_storage() -> (ParticleStorage, [ParticleId; 4]) {
    let mut storage = storage();
    let ids = [
        storage.create(input(1, 0, true)).expect("particle fits"),
        storage.create(input(2, 0, false)).expect("particle fits"),
        storage.create(input(3, 1, true)).expect("particle fits"),
        storage.create(input(4, 1, false)).expect("particle fits"),
    ];
    storage.contacts = vec![[ParticleIndex(0), ParticleIndex(2)]];
    storage.pairs = vec![[ParticleIndex(1), ParticleIndex(3)]];
    storage.triads = vec![[ParticleIndex(0), ParticleIndex(1), ParticleIndex(3)]];
    (storage, ids)
}

#[test]
fn one_transaction_remaps_all_lanes_indices_and_group_ranges() {
    // Arrange
    let (mut storage, ids) = populated_storage();

    // Act
    storage
        .rotate_rows(0, 2, 4)
        .expect("whole-group rotation is valid");

    // Assert
    assert_eq!(storage.input(ids[0]), Ok(input(1, 0, true)));
    assert_eq!(
        storage.input(ids[1]),
        Ok(input_with_optional_defaults(2, 0))
    );
    assert_eq!(storage.input(ids[2]), Ok(input(3, 1, true)));
    assert_eq!(
        storage.input(ids[3]),
        Ok(input_with_optional_defaults(4, 1))
    );
    assert_eq!(
        storage.proxies,
        vec![
            ParticleIndex(2),
            ParticleIndex(3),
            ParticleIndex(0),
            ParticleIndex(1)
        ]
    );
    assert_eq!(storage.contacts, vec![[ParticleIndex(2), ParticleIndex(0)]]);
    assert_eq!(storage.pairs, vec![[ParticleIndex(3), ParticleIndex(1)]]);
    assert_eq!(
        storage.triads,
        vec![[ParticleIndex(2), ParticleIndex(3), ParticleIndex(1)]]
    );
    assert_eq!(
        storage.lifetime_order,
        vec![
            ParticleIndex(2),
            ParticleIndex(3),
            ParticleIndex(0),
            ParticleIndex(1)
        ]
    );
    assert_eq!(
        storage.group_ranges,
        vec![
            GroupRange {
                group: 1,
                start: ParticleIndex(0),
                end: ParticleIndex(2)
            },
            GroupRange {
                group: 0,
                start: ParticleIndex(2),
                end: ParticleIndex(4)
            },
        ]
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

#[test]
fn invalid_duplicate_permutation_leaves_state_unchanged() {
    // Arrange
    let (mut storage, _ids) = populated_storage();
    let before = storage.clone();

    // Act
    let result = storage.apply_permutation(&[Some(0), Some(0), Some(1), Some(2)]);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidPermutation));
    assert!(storage == before);
}

#[test]
fn out_of_range_derived_reference_leaves_state_unchanged() {
    // Arrange
    let (mut storage, _ids) = populated_storage();
    storage.contacts.push([ParticleIndex(0), ParticleIndex(99)]);
    let before = storage.clone();

    // Act
    let result = storage.rotate_rows(0, 2, 4);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidDerivedReference));
    assert!(storage == before);
}

#[test]
fn mismatched_optional_lane_leaves_state_unchanged() {
    // Arrange
    let (mut storage, _ids) = populated_storage();
    storage
        .maybe_colors
        .as_mut()
        .expect("fixture enables the optional color lane")
        .pop();
    let before = storage.clone();

    // Act
    let result = storage.rotate_rows(0, 2, 4);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::LaneLengthMismatch));
    assert!(storage == before);
}

#[test]
fn compaction_drops_removed_references_and_remaps_survivors() {
    // Arrange
    let (mut storage, ids) = populated_storage();
    storage.mark_delete(ids[0]).expect("particle is live");
    storage.mark_delete(ids[3]).expect("particle is live");

    // Act
    let destroyed = storage.compact_pending().expect("compaction is valid");

    // Assert
    assert_eq!(destroyed.len(), 2);
    assert!(storage.contacts.is_empty());
    assert!(storage.pairs.is_empty());
    assert!(storage.triads.is_empty());
    assert_eq!(storage.proxies, vec![ParticleIndex(0), ParticleIndex(1)]);
    assert_eq!(
        storage.lifetime_order,
        vec![ParticleIndex(0), ParticleIndex(1)]
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

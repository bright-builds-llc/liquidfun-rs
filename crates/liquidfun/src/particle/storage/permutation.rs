use super::*;

fn input(value: i16, maybe_group: Option<ParticleGroupId>, optional: bool) -> ParticleInput {
    let scalar = f32::from(value);
    let component = u8::try_from(value).expect("test values fit in a color component");
    ParticleInput {
        position: Vec2::new(scalar, -scalar),
        velocity: Vec2::new(scalar + 10.0, scalar + 20.0),
        flags: ParticleFlags::from_bits_retain(
            u32::try_from(value).expect("test values are non-negative"),
        ),
        maybe_group,
        maybe_color: optional.then_some(ParticleColor::new(
            component, component, component, component,
        )),
        maybe_user_association: optional.then_some(UserAssociationKey::new(
            u64::try_from(value).expect("test values are non-negative"),
        )),
        maybe_expiration_time: optional.then_some(i32::from(value) + 100),
    }
}

fn storage() -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    ParticleStorage::new(world, system, 0, 8, 8).expect("test storage contract is valid")
}

fn input_with_optional_defaults(value: i16, maybe_group: Option<ParticleGroupId>) -> ParticleInput {
    let mut value = input(value, maybe_group, false);
    value.maybe_color = Some(ParticleColor::ZERO);
    value.maybe_expiration_time = Some(0);
    value
}

fn populated_storage() -> (ParticleStorage, [ParticleId; 4]) {
    let mut storage = storage();
    let first_group = ParticleGroupId::from_identity(Identity::new(storage.world, 10, 0));
    let second_group = ParticleGroupId::from_identity(Identity::new(storage.world, 11, 0));
    let ids = [
        storage
            .create(input(1, Some(first_group), true))
            .expect("particle fits"),
        storage
            .create(input(2, Some(first_group), false))
            .expect("particle fits"),
        storage
            .create(input(3, Some(second_group), true))
            .expect("particle fits"),
        storage
            .create(input(4, Some(second_group), false))
            .expect("particle fits"),
    ];
    storage.particle_contacts = vec![ParticleContact {
        indices: [ParticleIndex(0), ParticleIndex(2)],
        flags: ParticleFlags::WATER,
        weight: 0.5,
        normal: Vec2::new(1.0, 0.0),
    }];
    storage.pairs = vec![ParticlePair {
        indices: [ParticleIndex(1), ParticleIndex(3)],
        flags: ParticleFlags::SPRING,
        strength: 0.75,
        distance: 2.0,
    }];
    storage.triads = vec![ParticleTriad {
        indices: [ParticleIndex(0), ParticleIndex(1), ParticleIndex(3)],
        flags: ParticleFlags::ELASTIC,
        strength: 0.25,
    }];
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
    let first_group = storage.groups[2];
    let second_group = storage.groups[0];
    assert_eq!(storage.input(ids[0]), Ok(input(1, first_group, true)));
    assert_eq!(
        storage.input(ids[1]),
        Ok(input_with_optional_defaults(2, first_group))
    );
    assert_eq!(storage.input(ids[2]), Ok(input(3, second_group, true)));
    assert_eq!(
        storage.input(ids[3]),
        Ok(input_with_optional_defaults(4, second_group))
    );
    assert_eq!(
        storage
            .proxies
            .iter()
            .map(|proxy| proxy.index)
            .collect::<Vec<_>>(),
        vec![
            ParticleIndex(2),
            ParticleIndex(3),
            ParticleIndex(0),
            ParticleIndex(1)
        ]
    );
    assert_eq!(
        storage.particle_contacts[0].indices,
        [ParticleIndex(2), ParticleIndex(0)]
    );
    assert_eq!(
        storage.pairs[0].indices,
        [ParticleIndex(3), ParticleIndex(1)]
    );
    assert_eq!(
        storage.triads[0].indices,
        [ParticleIndex(2), ParticleIndex(3), ParticleIndex(1)]
    );
    assert_eq!(
        storage.maybe_expiration_order,
        Some(vec![
            ParticleIndex(2),
            ParticleIndex(3),
            ParticleIndex(0),
            ParticleIndex(1)
        ])
    );
    assert_eq!(
        storage.group_ranges,
        vec![
            GroupRange {
                maybe_group: second_group,
                start: ParticleIndex(0),
                end: ParticleIndex(2)
            },
            GroupRange {
                maybe_group: first_group,
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
    storage.particle_contacts.push(ParticleContact {
        indices: [ParticleIndex(0), ParticleIndex(99)],
        flags: ParticleFlags::WATER,
        weight: 0.0,
        normal: Vec2::ZERO,
    });
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
    assert!(storage.particle_contacts.is_empty());
    assert!(storage.pairs.is_empty());
    assert!(storage.triads.is_empty());
    assert_eq!(
        storage
            .proxies
            .iter()
            .map(|proxy| proxy.index)
            .collect::<Vec<_>>(),
        vec![ParticleIndex(0), ParticleIndex(1)]
    );
    assert_eq!(
        storage.maybe_expiration_order,
        Some(vec![ParticleIndex(0), ParticleIndex(1)])
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

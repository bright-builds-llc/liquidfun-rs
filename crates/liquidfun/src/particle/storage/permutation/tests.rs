use super::*;

use crate::identity::{BodyId, FixtureId, HandleIdentity, Identity, ParticleSystemId, WorldKey};

use super::super::ParticleInput;

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
    storage.weights = vec![0.5, 0.0, 0.5, 0.0];
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
        pa: Vec2::new(1.0, -2.0),
        pb: Vec2::new(-3.0, 4.0),
        pc: Vec2::new(5.0, -6.0),
        ka: -7.0,
        kb: 8.0,
        kc: -9.0,
        s: -10.0,
    }];
    (storage, ids)
}

#[test]
fn one_transaction_remaps_all_lanes_indices_and_group_records() {
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
    assert_eq!(storage.triads[0].pa, Vec2::new(1.0, -2.0));
    assert_eq!(storage.triads[0].pb, Vec2::new(-3.0, 4.0));
    assert_eq!(storage.triads[0].pc, Vec2::new(5.0, -6.0));
    assert_eq!(
        (
            storage.triads[0].ka,
            storage.triads[0].kb,
            storage.triads[0].kc,
            storage.triads[0].s
        ),
        (-7.0, 8.0, -9.0, -10.0)
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
        storage
            .group_records
            .iter()
            .map(|record| (Some(record.id), record.first, record.last))
            .collect::<Vec<_>>(),
        vec![(second_group, 0, 2), (first_group, 2, 4),]
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

#[test]
fn invalid_duplicate_permutation_leaves_contacts_and_weights_unchanged() {
    // Arrange
    let (mut storage, _ids) = populated_storage();
    let before = storage.clone();

    // Act
    let result =
        apply_preserving_historical_order(&mut storage, &[Some(0), Some(0), Some(1), Some(2)]);

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

#[test]
fn incomplete_mapping_cannot_remove_a_live_row() {
    // Arrange
    let (mut storage, _ids) = populated_storage();
    let before = storage.clone();

    // Act
    let result =
        apply_preserving_historical_order(&mut storage, &[Some(0), None, Some(1), Some(2)]);

    // Assert
    assert_eq!(result, Err(ParticleStorageError::InvalidPermutation));
    assert!(storage == before);
}

#[test]
fn rotation_recomputes_weights_from_remapped_contacts() {
    // Arrange
    let (mut storage, ids) = populated_storage();
    let body = BodyId::from_identity(Identity::new(storage.world, 20, 0));
    let fixture = FixtureId::from_identity(Identity::new(storage.world, 21, 0));
    storage.weights = vec![0.5, 0.75, 0.5, 0.25];
    storage.forces = vec![
        Vec2::new(1.0, 10.0),
        Vec2::new(2.0, 20.0),
        Vec2::new(3.0, 30.0),
        Vec2::new(4.0, 40.0),
    ];
    storage.maybe_stuck = Some(StuckLanes {
        last_body_contact_steps: vec![10, 20, 30, 40],
        body_contact_counts: vec![1, 2, 3, 4],
        consecutive_contact_steps: vec![5, 6, 7, 8],
        candidates: vec![ParticleIndex(1)],
    });
    storage.body_contacts = vec![
        ParticleBodyContact {
            index: ParticleIndex(1),
            body,
            fixture,
            weight: 0.75,
            normal: Vec2::new(0.0, 1.0),
            mass: 2.5,
        },
        ParticleBodyContact {
            index: ParticleIndex(3),
            body,
            fixture,
            weight: 0.25,
            normal: Vec2::new(0.0, -1.0),
            mass: 1.5,
        },
    ];
    let before_particle_contact = storage.particle_contacts[0]
        .indices
        .map(|index| storage.dense_to_id[index.0]);
    let before_body_contacts = storage
        .body_contacts
        .iter()
        .map(|contact| (storage.dense_to_id[contact.index.0], contact.weight))
        .collect::<Vec<_>>();

    // Act
    storage
        .rotate_rows(0, 2, 4)
        .expect("complete production permutation is valid");

    // Assert
    assert_eq!(storage.input(ids[0]), Ok(input(1, storage.groups[2], true)));
    assert_eq!(
        storage.forces,
        vec![
            Vec2::new(3.0, 30.0),
            Vec2::new(4.0, 40.0),
            Vec2::new(1.0, 10.0),
            Vec2::new(2.0, 20.0),
        ]
    );
    assert_eq!(storage.weights, vec![0.5, 0.25, 0.5, 0.75]);
    assert_eq!(
        storage.particle_contacts[0]
            .indices
            .map(|index| storage.dense_to_id[index.0]),
        before_particle_contact
    );
    assert_eq!(
        storage
            .body_contacts
            .iter()
            .map(|contact| (storage.dense_to_id[contact.index.0], contact.weight))
            .collect::<Vec<_>>(),
        before_body_contacts
    );
    let stuck = storage
        .maybe_stuck
        .expect("stuck lanes remain allocated after permutation");
    assert_eq!(stuck.last_body_contact_steps, vec![30, 40, 10, 20]);
    assert_eq!(stuck.body_contact_counts, vec![3, 4, 1, 2]);
    assert_eq!(stuck.consecutive_contact_steps, vec![7, 8, 5, 6]);
    assert!(stuck.candidates.is_empty());
    assert_eq!(storage.body_contacts[0].index, ParticleIndex(3));
    assert_eq!(storage.body_contacts[0].body, body);
    assert_eq!(storage.body_contacts[0].fixture, fixture);
    assert_eq!(storage.body_contacts[1].index, ParticleIndex(1));
}

#[test]
fn compaction_recomputes_weights_after_removing_a_contacted_middle_row() {
    // Arrange
    let (mut storage, ids) = populated_storage();
    let body = BodyId::from_identity(Identity::new(storage.world, 20, 0));
    let fixture = FixtureId::from_identity(Identity::new(storage.world, 21, 0));
    storage.body_contacts = vec![ParticleBodyContact {
        index: ParticleIndex(1),
        body,
        fixture,
        weight: 0.75,
        normal: Vec2::new(0.0, 1.0),
        mass: 2.5,
    }];
    storage.weights = vec![0.5, 0.75, 0.5, 0.0];
    let before_particle_contact = storage.particle_contacts[0]
        .indices
        .map(|index| storage.dense_to_id[index.0]);
    storage
        .mark_delete(ids[1])
        .expect("middle particle is live");

    // Act
    storage
        .compact_pending()
        .expect("contacted middle particle compacts");

    // Assert
    assert_eq!(storage.weights, vec![0.5, 0.5, 0.0]);
    assert!(storage.body_contacts.is_empty());
    assert_eq!(
        storage.particle_contacts[0]
            .indices
            .map(|index| storage.dense_to_id[index.0]),
        before_particle_contact
    );
    assert_eq!(storage.check_invariants(), Ok(()));
}

#[test]
fn production_storage_has_one_permutation_authority() {
    // Arrange
    let storage_source = include_str!("../../storage.rs");
    let lifecycle_source = include_str!("../lifecycle.rs");
    let lanes_source = include_str!("../lanes.rs");

    // Act / Assert
    assert!(!storage_source.contains("fn apply_permutation"));
    assert!(!lifecycle_source.contains("fn apply_permutation"));
    assert!(lifecycle_source.contains("mutation::MutationCandidate"));
    for forbidden in [
        ".rotate_left(",
        ".rotate_right(",
        ".retain(",
        ".swap_remove(",
    ] {
        assert!(!storage_source.contains(forbidden));
        assert!(!lifecycle_source.contains(forbidden));
        assert!(!lanes_source.contains(forbidden));
    }
}

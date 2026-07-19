use proptest::prelude::*;

use crate::identity::{BodyId, FixtureId};

use super::super::*;
use super::{input, ordinary_storage};

const ROW_COUNT: usize = 6;

fn populated_storage() -> (ParticleStorage, [ParticleId; ROW_COUNT]) {
    let mut storage = ordinary_storage(ROW_COUNT);
    let first_group = ParticleGroupId::from_identity(Identity::new(storage.world, 20, 0));
    let second_group = ParticleGroupId::from_identity(Identity::new(storage.world, 21, 0));
    let ids = std::array::from_fn(|old| {
        let mut particle = input(
            i16::try_from(old + 1).expect("row value fits"),
            old % 2 == 1,
        );
        particle.maybe_group = Some(if old < ROW_COUNT / 2 {
            first_group
        } else {
            second_group
        });
        storage.create(particle).expect("fixture particle fits")
    });

    let body = BodyId::from_identity(Identity::new(storage.world, 30, 0));
    let fixture = FixtureId::from_identity(Identity::new(storage.world, 31, 0));
    let row_count = u16::try_from(ROW_COUNT).expect("fixture row count fits in u16");
    storage.weights = vec![0.25, 0.25, 0.75, 0.5, 0.75, 0.0];
    storage.forces = (1..=row_count)
        .map(|value| {
            let scalar = f32::from(value);
            Vec2::new(scalar, -scalar)
        })
        .collect();
    storage.maybe_stuck = Some(StuckLanes {
        last_body_contact_steps: (10..16).collect(),
        body_contact_counts: (20..26).collect(),
        consecutive_contact_steps: (30..36).collect(),
        candidates: vec![ParticleIndex(0), ParticleIndex(4)],
    });
    storage.particle_contacts = vec![
        ParticleContact {
            indices: [ParticleIndex(0), ParticleIndex(1)],
            flags: ParticleFlags::WATER,
            weight: 0.25,
            normal: Vec2::new(1.0, 0.0),
        },
        ParticleContact {
            indices: [ParticleIndex(2), ParticleIndex(4)],
            flags: ParticleFlags::VISCOUS,
            weight: 0.75,
            normal: Vec2::new(0.0, 1.0),
        },
    ];
    storage.body_contacts = vec![ParticleBodyContact {
        index: ParticleIndex(3),
        body,
        fixture,
        weight: 0.5,
        normal: Vec2::new(-1.0, 0.0),
        mass: 2.0,
    }];
    storage.pairs = vec![ParticlePair {
        indices: [ParticleIndex(1), ParticleIndex(2)],
        flags: ParticleFlags::SPRING,
        strength: 0.5,
        distance: 3.0,
    }];
    storage.triads = vec![ParticleTriad {
        indices: [ParticleIndex(0), ParticleIndex(3), ParticleIndex(5)],
        flags: ParticleFlags::ELASTIC,
        strength: 0.75,
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

fn destination_mapping(keys: [u8; ROW_COUNT], removed: [bool; ROW_COUNT]) -> Vec<Option<usize>> {
    let mut survivor_order = Vec::new();
    for group in 0..2 {
        let mut group_rows: Vec<_> = (0..ROW_COUNT)
            .filter(|old| *old / (ROW_COUNT / 2) == group && !removed[*old])
            .collect();
        group_rows.sort_by_key(|old| (keys[*old], *old));
        survivor_order.extend(group_rows);
    }

    let mut old_to_new = vec![None; ROW_COUNT];
    for (new, old) in survivor_order.into_iter().enumerate() {
        old_to_new[old] = Some(new);
    }
    old_to_new
}

fn mapped_ids<const N: usize>(
    storage: &ParticleStorage,
    indices: [ParticleIndex; N],
) -> [ParticleId; N] {
    indices.map(|index| storage.dense_to_id[index.0])
}

fn survives<const N: usize>(ids: [ParticleId; N], removed_ids: &[ParticleId]) -> bool {
    ids.iter().all(|id| !removed_ids.contains(id))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn total_permutation_matches_stable_semantic_snapshot(
        keys in any::<[u8; ROW_COUNT]>(),
        removed in any::<[bool; ROW_COUNT]>(),
    ) {
        // Arrange
        let (mut storage, ids) = populated_storage();
        let semantic_inputs: Vec<_> = ids
            .iter()
            .copied()
            .map(|id| (id, storage.input(id).expect("fixture id is live")))
            .collect();
        let semantic_forces: Vec<_> = ids
            .iter()
            .copied()
            .zip(storage.forces.iter().copied())
            .collect();
        let contact_ids: Vec<_> = storage
            .particle_contacts
            .iter()
            .map(|contact| mapped_ids(&storage, contact.indices))
            .collect();
        let particle_contact_weights: Vec<_> = storage
            .particle_contacts
            .iter()
            .map(|contact| (mapped_ids(&storage, contact.indices), contact.weight))
            .collect();
        let body_contact_ids: Vec<_> = storage
            .body_contacts
            .iter()
            .map(|contact| storage.dense_to_id[contact.index.0])
            .collect();
        let body_contact_weights: Vec<_> = storage
            .body_contacts
            .iter()
            .map(|contact| (storage.dense_to_id[contact.index.0], contact.weight))
            .collect();
        let pair_ids: Vec<_> = storage
            .pairs
            .iter()
            .map(|pair| mapped_ids(&storage, pair.indices))
            .collect();
        let triad_ids: Vec<_> = storage
            .triads
            .iter()
            .map(|triad| mapped_ids(&storage, triad.indices))
            .collect();
        let old_to_new = destination_mapping(keys, removed);
        let removed_ids: Vec<_> = ids
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(old, id)| removed[old].then_some(id))
            .collect();
        for id in &removed_ids {
            storage.mark_delete(*id).expect("selected fixture id is live");
        }

        // Act
        let destroyed = super::super::permutation::apply_permutation(&mut storage, &old_to_new)
            .map_err(|error| TestCaseError::fail(format!("valid mapping failed: {error:?}")))?;

        // Assert
        let mut expected_order: Vec<_> = (0..ROW_COUNT)
            .filter_map(|old| old_to_new[old].map(|new| (new, ids[old])))
            .collect();
        expected_order.sort_by_key(|(new, _)| *new);
        let expected_ids: Vec<_> = expected_order.iter().map(|(_, id)| *id).collect();
        prop_assert_eq!(&storage.dense_to_id, &expected_ids);
        for (id, expected_input) in semantic_inputs {
            let expected = if removed_ids.contains(&id) {
                Err(ParticleStorageError::StaleOrDestroyed)
            } else {
                Ok(expected_input)
            };
            prop_assert_eq!(storage.input(id), expected);
        }
        let expected_forces: Vec<_> = expected_ids
            .iter()
            .map(|id| {
                semantic_forces
                    .iter()
                    .find_map(|(candidate, force)| (candidate == id).then_some(*force))
                    .expect("survivor force exists")
            })
            .collect();
        prop_assert_eq!(&storage.forces, &expected_forces);
        let mut expected_weights = vec![0.0; expected_ids.len()];
        for (id, weight) in body_contact_weights {
            if let Some(index) = expected_ids.iter().position(|candidate| *candidate == id) {
                expected_weights[index] += weight;
            }
        }
        for (contact, weight) in particle_contact_weights {
            if survives(contact, &removed_ids) {
                for id in contact {
                    let index = expected_ids
                        .iter()
                        .position(|candidate| *candidate == id)
                        .expect("retained contact particle survives");
                    expected_weights[index] += weight;
                }
            }
        }
        prop_assert_eq!(&storage.weights, &expected_weights);
        prop_assert_eq!(
            storage
                .particle_contacts
                .iter()
                .map(|contact| mapped_ids(&storage, contact.indices))
                .collect::<Vec<_>>(),
            contact_ids
                .into_iter()
                .filter(|record| survives(*record, &removed_ids))
                .collect::<Vec<_>>()
        );
        prop_assert_eq!(
            storage
                .body_contacts
                .iter()
                .map(|contact| storage.dense_to_id[contact.index.0])
                .collect::<Vec<_>>(),
            body_contact_ids
                .into_iter()
                .filter(|id| !removed_ids.contains(id))
                .collect::<Vec<_>>()
        );
        prop_assert_eq!(
            storage
                .pairs
                .iter()
                .map(|pair| mapped_ids(&storage, pair.indices))
                .collect::<Vec<_>>(),
            pair_ids
                .into_iter()
                .filter(|record| survives(*record, &removed_ids))
                .collect::<Vec<_>>()
        );
        prop_assert_eq!(
            storage
                .triads
                .iter()
                .map(|triad| mapped_ids(&storage, triad.indices))
                .collect::<Vec<_>>(),
            triad_ids
                .into_iter()
                .filter(|record| survives(*record, &removed_ids))
                .collect::<Vec<_>>()
        );
        prop_assert_eq!(
            storage
                .maybe_expiration_order
                .as_ref()
                .expect("optional fixture allocated expiration order")
                .iter()
                .map(|index| storage.dense_to_id[index.0])
                .collect::<Vec<_>>(),
            ids.into_iter()
                .filter(|id| !removed_ids.contains(id))
                .collect::<Vec<_>>()
        );
        let stuck = storage
            .maybe_stuck
            .as_ref()
            .expect("fixture allocates stuck lanes");
        prop_assert_eq!(stuck.candidates.as_slice(), &[]);
        prop_assert_eq!(storage.check_invariants(), Ok(()));
        prop_assert_eq!(
            destroyed.iter().map(|snapshot| snapshot.id).collect::<Vec<_>>(),
            removed_ids
        );
    }

    #[test]
    fn invalid_mappings_are_atomic(kind in any::<u8>()) {
        // Arrange
        let (mut storage, _ids) = populated_storage();
        let before = storage.clone();
        let mapping = match kind % 4 {
            0 => vec![Some(0); ROW_COUNT - 1],
            1 => vec![Some(0), Some(0), Some(1), Some(2), Some(3), Some(4)],
            2 => vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(ROW_COUNT)],
            _ => vec![Some(0), None, Some(1), Some(2), Some(3), Some(4)],
        };

        // Act
        let result = super::super::permutation::apply_permutation(&mut storage, &mapping);

        // Assert
        prop_assert_eq!(result, Err(ParticleStorageError::InvalidPermutation));
        prop_assert!(storage == before);
    }

    #[test]
    fn terminal_generations_retire_without_resurrection(start_at_max in any::<bool>()) {
        // Arrange
        let mut storage = ordinary_storage(1);
        let initial = storage.create(input(1, false)).expect("identity fits");
        storage.mark_delete(initial).expect("identity is live");
        storage.compact_pending().expect("initial deletion compacts");
        storage.identities[0].generation = if start_at_max { u64::MAX } else { u64::MAX - 1 };

        // Act
        let terminal = storage.create(input(2, true)).expect("terminal generation is reusable");
        storage.mark_delete(terminal).expect("terminal identity is live");
        storage.compact_pending().expect("terminal deletion compacts");
        if !start_at_max {
            let maximum = storage.create(input(3, false)).expect("maximum generation is reusable");
            storage.mark_delete(maximum).expect("maximum identity is live");
            storage.compact_pending().expect("maximum deletion compacts");
        }

        // Assert
        prop_assert_eq!(storage.create(input(4, false)), Err(ParticleStorageError::IdentityExhausted));
        prop_assert_eq!(storage.retired_identity_slots, 1);
        prop_assert_eq!(storage.check_invariants(), Ok(()));
    }
}

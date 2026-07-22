use proptest::prelude::*;

use crate::identity::{
    HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::{Rotation, Transform, Vec2};
use crate::particle::storage::group::{GroupRecord, InternalGroupFlags};
use crate::particle::storage::lanes::{
    ParticleContact, ParticlePair, ParticleTriad, UserAssociationKey,
};
use crate::particle::storage::{
    ParticleIndex, ParticleInput, ParticleStorage, ParticleStorageError,
};
use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::SplitPlanError;

struct Fixture {
    storage: ParticleStorage,
    source: ParticleGroupId,
    other: ParticleGroupId,
    new_a: ParticleGroupId,
    new_b: ParticleGroupId,
    ids: [ParticleId; 8],
}

fn particle(value: f32, group: ParticleGroupId, flags: ParticleFlags) -> ParticleInput {
    ParticleInput {
        position: Vec2::new(value, -value),
        velocity: Vec2::new(-value, value),
        flags,
        maybe_group: Some(group),
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
    }
}

fn contact(a: usize, b: usize) -> ParticleContact {
    ParticleContact {
        indices: [ParticleIndex(a), ParticleIndex(b)],
        flags: ParticleFlags::SPRING,
        weight: 0.5,
        normal: Vec2::new(1.0, 0.0),
    }
}

fn pair(indices: [usize; 2], rest: f32) -> ParticlePair {
    ParticlePair {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::SPRING,
        strength: rest + 0.25,
        distance: rest,
    }
}

fn triad(indices: [usize; 3], rest: f32) -> ParticleTriad {
    ParticleTriad {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::ELASTIC,
        strength: rest + 0.5,
        pa: Vec2::new(rest + 1.0, rest + 2.0),
        pb: Vec2::new(rest + 3.0, rest + 4.0),
        pc: Vec2::new(rest + 5.0, rest + 6.0),
        ka: rest + 7.0,
        kb: rest + 8.0,
        kc: rest + 9.0,
        s: rest + 10.0,
    }
}

fn fixture() -> Fixture {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let [source, other, new_a, new_b] =
        [20, 21, 22, 23].map(|slot| ParticleGroupId::from_identity(Identity::new(world, slot, 0)));
    let mut storage =
        ParticleStorage::new(world, system, 0, 16, 16).expect("test storage is valid");
    let mut ids = Vec::new();
    for index in 0..8 {
        let group = if index < 6 { source } else { other };
        let flags = if index == 5 {
            ParticleFlags::SPRING | ParticleFlags::ELASTIC | ParticleFlags::ZOMBIE
        } else {
            ParticleFlags::SPRING | ParticleFlags::ELASTIC
        };
        ids.push(
            storage
                .create(particle(
                    f32::from(u8::try_from(index).expect("fixture index fits in u8")),
                    group,
                    flags,
                ))
                .expect("fixture particle fits"),
        );
    }
    let ids = ids.try_into().expect("fixture has eight particles");
    storage.particle_contacts = vec![contact(0, 2), contact(1, 3), contact(0, 6), contact(3, 7)];
    storage.pairs = vec![pair([0, 1], 10.0), pair([1, 3], 20.0)];
    storage.triads = vec![triad([0, 1, 4], 30.0)];
    let source_record = storage
        .group_records
        .iter_mut()
        .find(|record| record.id == source)
        .expect("source record exists");
    source_record.flags = ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY;
    source_record
        .internal_flags
        .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    source_record.strength = 0.375;
    source_record.transform = Transform::new(Vec2::new(1.25, -2.5), Rotation::from_angle(0.25));
    source_record.maybe_user_association = Some(UserAssociationKey::new(91));
    storage
        .solver_state
        .refresh_group_flags(&storage.group_records);
    Fixture {
        storage,
        source,
        other,
        new_a,
        new_b,
        ids,
    }
}

fn record(storage: &ParticleStorage, group: ParticleGroupId) -> GroupRecord {
    storage
        .group_records
        .iter()
        .copied()
        .find(|record| record.id == group)
        .expect("group record exists")
}

fn pair_rest_bits(storage: &ParticleStorage) -> Vec<(u32, u32)> {
    storage
        .pairs
        .iter()
        .map(|pair| (pair.strength.to_bits(), pair.distance.to_bits()))
        .collect()
}

fn triad_rest_bits(storage: &ParticleStorage) -> Vec<[u32; 11]> {
    storage
        .triads
        .iter()
        .map(|triad| {
            [
                triad.strength.to_bits(),
                triad.pa.x.to_bits(),
                triad.pa.y.to_bits(),
                triad.pb.x.to_bits(),
                triad.pb.y.to_bits(),
                triad.pc.x.to_bits(),
                triad.pc.y.to_bits(),
                triad.ka.to_bits(),
                triad.kb.to_bits(),
                triad.kc.to_bits(),
                triad.s.to_bits(),
            ]
        })
        .collect()
}

#[test]
fn split_preserves_stable_ids_and_retargets_historical_topology_exactly() {
    // Arrange
    let mut fixture = fixture();
    let pair_bits = pair_rest_bits(&fixture.storage);
    let triad_bits = triad_rest_bits(&fixture.storage);

    // Act
    let plan = fixture
        .storage
        .plan_split(fixture.source, &[fixture.new_a, fixture.new_b])
        .expect("split candidate validates");
    assert_eq!(
        plan.result_groups(),
        &[fixture.source, fixture.new_a, fixture.new_b]
    );
    plan.commit(&mut fixture.storage);

    // Assert
    assert_eq!(
        fixture.storage.particle_ids(),
        &[
            fixture.ids[0],
            fixture.ids[2],
            fixture.ids[5],
            fixture.ids[6],
            fixture.ids[7],
            fixture.ids[1],
            fixture.ids[3],
            fixture.ids[4],
        ]
    );
    let expected_memberships = [
        fixture.source,
        fixture.new_a,
        fixture.source,
        fixture.new_a,
        fixture.new_b,
        fixture.source,
        fixture.other,
        fixture.other,
    ];
    for (id, expected_group) in fixture.ids.into_iter().zip(expected_memberships) {
        assert_eq!(
            fixture.storage.input(id).map(|input| input.maybe_group),
            Ok(Some(expected_group))
        );
    }
    assert_eq!(pair_rest_bits(&fixture.storage), pair_bits);
    assert_eq!(triad_rest_bits(&fixture.storage), triad_bits);
    assert_eq!(
        fixture
            .storage
            .pairs
            .iter()
            .map(|pair| pair.indices.map(|index| index.0))
            .collect::<Vec<_>>(),
        vec![[0, 5], [5, 6]]
    );
    assert_eq!(
        fixture.storage.triads[0].indices.map(|index| index.0),
        [0, 5, 7]
    );
}

#[test]
fn split_created_metadata_matches_the_pinned_probe_verbatim() {
    // Arrange
    let mut fixture = fixture();
    let witness = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.json"
    ));
    let provenance = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.provenance.json"
    ));

    // Act
    fixture
        .storage
        .plan_split(fixture.source, &[fixture.new_a, fixture.new_b])
        .expect("split candidate validates")
        .commit(&mut fixture.storage);
    let created = record(&fixture.storage, fixture.new_a);

    // Assert
    assert!(witness.contains("\"id\": \"split_created_metadata\""));
    assert!(witness.contains("\"raw_group_flags\": 21"));
    assert!(witness.contains("\"strength_bits\": \"0x3f800000\""));
    assert!(witness.contains("\"user_data_preserved\": true"));
    assert!(provenance.contains(
        "\"witness_sha256\": \"90d212d3380fe9aa645ca9d972e39b962db9f912853850a9deb5943be2395278\""
    ));
    assert_eq!(
        created.flags,
        ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY
    );
    assert!(
        created
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
    );
    assert_eq!(created.strength.to_bits(), 1.0_f32.to_bits());
    assert_eq!(created.transform, Transform::IDENTITY);
    assert_eq!(
        created.maybe_user_association,
        Some(UserAssociationKey::new(91))
    );
    assert_eq!(created.statistics.maybe_source_timestamp, None);
    assert_eq!(created.statistics.mass.to_bits(), 0);
    assert_eq!(created.statistics.center, Vec2::ZERO);
    assert_eq!(created.statistics.linear_velocity, Vec2::ZERO);
    assert_eq!(created.statistics.inertia.to_bits(), 0);
    assert_eq!(created.statistics.angular_velocity.to_bits(), 0);
    assert_eq!(record(&fixture.storage, fixture.source).range(), 0..3);
    assert_eq!(record(&fixture.storage, fixture.other).range(), 3..5);
    assert_eq!(created.range(), 5..7);
    assert_eq!(record(&fixture.storage, fixture.new_b).range(), 7..8);
}

#[test]
fn invalid_group_identity_candidates_leave_storage_unchanged() {
    // Arrange
    let fixture = fixture();
    let before = fixture.storage.clone();
    let wrong_world = WorldKey::fresh().expect("test world key remains available");
    let wrong_group = ParticleGroupId::from_identity(Identity::new(wrong_world, 22, 0));

    // Act
    let count_result = fixture.storage.plan_split(fixture.source, &[fixture.new_a]);
    let duplicate_result = fixture
        .storage
        .plan_split(fixture.source, &[fixture.new_a, fixture.new_a]);
    let wrong_world_result = fixture
        .storage
        .plan_split(fixture.source, &[fixture.new_a, wrong_group]);

    // Assert
    assert_eq!(
        count_result.map(|_| ()),
        Err(SplitPlanError::GroupIdentityCount {
            required: 2,
            provided: 1,
        })
    );
    assert_eq!(
        duplicate_result.map(|_| ()),
        Err(SplitPlanError::Storage(
            ParticleStorageError::InvalidGroupRange
        ))
    );
    assert_eq!(
        wrong_world_result.map(|_| ()),
        Err(SplitPlanError::Storage(
            ParticleStorageError::InvalidGroupRange
        ))
    );
    assert!(fixture.storage == before);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn bounded_split_candidates_preserve_ids_rest_bits_and_rollback(
        member_count in 2_usize..12,
        adjacent_contacts in prop::collection::vec(any::<bool>(), 1..12),
    ) {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let source = ParticleGroupId::from_identity(Identity::new(world, 20, 0));
        let other = ParticleGroupId::from_identity(Identity::new(world, 21, 0));
        let mut storage = ParticleStorage::new(world, system, 0, 32, 32)
            .expect("bounded storage is valid");
        let mut ids = Vec::new();
        for index in 0..member_count {
            ids.push(
                storage
                    .create(particle(
                        f32::from(u8::try_from(index).expect("bounded index fits in u8")),
                        source,
                        ParticleFlags::SPRING,
                    ))
                    .expect("bounded particle fits"),
            );
        }
        ids.push(
            storage
                .create(particle(100.0, other, ParticleFlags::SPRING))
                .expect("other particle fits"),
        );
        for index in 0..member_count.saturating_sub(1) {
            if adjacent_contacts[index % adjacent_contacts.len()] {
                storage.particle_contacts.push(contact(index, index + 1));
            }
        }
        storage.pairs = (0..member_count.saturating_sub(1))
            .map(|index| {
                let value = f32::from(
                    u8::try_from(index).expect("bounded pair index fits in u8"),
                );
                pair([index, index + 1], value + 1.0)
            })
            .collect();
        let original_ids = storage.particle_ids().to_vec();
        let original_pair_bits = pair_rest_bits(&storage);
        let component_count = storage
            .split_group_count(source)
            .expect("bounded connectivity validates");
        let new_groups = (0..component_count - 1)
            .map(|ordinal| {
                ParticleGroupId::from_identity(Identity::new(world, 100 + ordinal, 0))
            })
            .collect::<Vec<_>>();
        let before_invalid = storage.clone();

        // Act
        let invalid_result = storage.plan_split(source, &[]);
        let invalid_left_storage_unchanged = storage == before_invalid;
        let plan = storage
            .plan_split(source, &new_groups)
            .expect("exact identity count validates");
        plan.commit(&mut storage);

        // Assert
        if component_count == 1 {
            prop_assert!(invalid_result.is_ok());
        } else {
            prop_assert_eq!(
                invalid_result.map(|_| ()),
                Err(SplitPlanError::GroupIdentityCount {
                    required: component_count - 1,
                    provided: 0,
                })
            );
        }
        prop_assert!(invalid_left_storage_unchanged);
        let mut sorted_after = storage.particle_ids().to_vec();
        sorted_after.sort_by_key(|id| id.identity().slot());
        let mut sorted_before = original_ids;
        sorted_before.sort_by_key(|id| id.identity().slot());
        prop_assert_eq!(sorted_after, sorted_before);
        prop_assert_eq!(pair_rest_bits(&storage), original_pair_bits);
        prop_assert_eq!(storage.check_invariants(), Ok(()));
    }
}

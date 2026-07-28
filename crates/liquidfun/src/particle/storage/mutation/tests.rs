use crate::identity::{HandleIdentity, Identity, ParticleGroupId, ParticleSystemId, WorldKey};
use crate::math::Vec2;
use crate::particle::ParticleFlags;

use super::*;
use crate::particle::storage::{ParticleIndex, ParticleInput};

fn storage() -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let group = ParticleGroupId::from_identity(Identity::new(world, 1, 0));
    let mut storage =
        ParticleStorage::new(world, system, 0, 8, 8).expect("storage contract is valid");
    for value in [0.0, 1.0, 2.0, 3.0] {
        storage
            .create(ParticleInput {
                position: Vec2::new(value, -value),
                velocity: Vec2::ZERO,
                flags: ParticleFlags::SPRING | ParticleFlags::ELASTIC,
                maybe_group: Some(group),
                maybe_color: None,
                maybe_user_association: None,
                maybe_expiration_time: None,
            })
            .expect("fixture particle fits");
    }
    storage.pairs = vec![pair([0, 1], 10.0), pair([2, 3], 20.0), pair([1, 2], 30.0)];
    storage.triads = vec![triad([0, 1, 2], 10.0), triad([1, 2, 3], 20.0)];
    storage
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
fn candidate_kind_is_closed_over_all_seven_operations() {
    // Arrange
    let storage = storage();
    let identity = (0..storage.len()).map(Some).collect::<Vec<_>>();
    let candidates = [
        MutationCandidate::prepare_create_group(&storage, Vec::new(), Vec::new())
            .expect("create candidate validates"),
        MutationCandidate::prepare_join_groups(&storage, &identity, Vec::new(), Vec::new())
            .expect("join candidate validates"),
        MutationCandidate::prepare_split_group(&storage, &identity)
            .expect("split candidate validates"),
        MutationCandidate::prepare_zombie_compaction(&storage, &identity)
            .expect("compaction candidate validates"),
        MutationCandidate::prepare_reactive_regeneration(&storage, Vec::new(), Vec::new())
            .expect("reactive candidate validates"),
        MutationCandidate::prepare_group_flag_change(&storage).expect("flag candidate validates"),
        MutationCandidate::prepare_ordinary_rotation(&storage, 0, 2, 4)
            .expect("rotation candidate validates"),
    ];

    // Act
    let kinds = candidates.map(|candidate| candidate.kind());

    // Assert
    assert_eq!(
        kinds,
        [
            MutationCandidateKind::CreateGroup,
            MutationCandidateKind::JoinGroups,
            MutationCandidateKind::SplitGroup,
            MutationCandidateKind::ZombieCompaction,
            MutationCandidateKind::ReactiveRegeneration,
            MutationCandidateKind::GroupFlagChange,
            MutationCandidateKind::OrdinaryRotation,
        ]
    );
}

#[test]
fn ordinary_rotation_preserves_topology_order_and_every_rest_bit() {
    // Arrange
    let mut storage = storage();
    let pair_bits = pair_rest_bits(&storage);
    let triad_bits = triad_rest_bits(&storage);

    // Act
    let candidate = MutationCandidate::prepare_ordinary_rotation(&storage, 0, 2, 4)
        .expect("rotation candidate validates");
    assert_eq!(
        candidate.payload().topology_mode,
        TopologyRemapMode::PreserveHistoricalOrder
    );
    candidate.commit(&mut storage);

    // Assert
    assert_eq!(pair_rest_bits(&storage), pair_bits);
    assert_eq!(triad_rest_bits(&storage), triad_bits);
    assert_eq!(
        storage
            .pairs
            .iter()
            .map(|pair| pair.indices.map(|index| index.0))
            .collect::<Vec<_>>(),
        vec![[2, 3], [0, 1], [3, 0]]
    );
}

#[test]
fn split_retarget_preserves_topology_order_and_every_rest_bit() {
    // Arrange
    let mut storage = storage();
    let mapping = [Some(1), Some(0), Some(3), Some(2)];
    let pair_bits = pair_rest_bits(&storage);
    let triad_bits = triad_rest_bits(&storage);

    // Act
    let candidate = MutationCandidate::prepare_split_group(&storage, &mapping)
        .expect("split candidate validates");
    candidate.commit(&mut storage);

    // Assert
    assert_eq!(pair_rest_bits(&storage), pair_bits);
    assert_eq!(triad_rest_bits(&storage), triad_bits);
    assert_eq!(
        storage
            .triads
            .iter()
            .map(|triad| triad.indices.map(|index| index.0))
            .collect::<Vec<_>>(),
        vec![[1, 0, 3], [0, 3, 2]]
    );
}

#[test]
fn append_policy_stable_sorts_and_keeps_the_first_duplicate() {
    // Arrange
    let mut storage = storage();
    let existing_duplicate_bits = storage.pairs[0].distance.to_bits();
    let appended = vec![pair([3, 0], 40.0), pair([0, 1], 99.0)];

    // Act
    let candidate =
        MutationCandidate::prepare_reactive_regeneration(&storage, appended, Vec::new())
            .expect("reactive candidate validates");
    assert_eq!(
        candidate.payload().topology_mode,
        TopologyRemapMode::AppendStableSortFirstDuplicate
    );
    candidate.commit(&mut storage);

    // Assert
    assert_eq!(
        storage
            .pairs
            .iter()
            .map(|pair| pair.indices.map(|index| index.0))
            .collect::<Vec<_>>(),
        vec![[0, 1], [1, 2], [2, 3], [3, 0]]
    );
    assert_eq!(storage.pairs[0].distance.to_bits(), existing_duplicate_bits);
}

#[test]
fn invalid_rotation_and_non_finite_append_leave_storage_unchanged() {
    // Arrange
    let storage = storage();
    let before = storage.clone();
    let invalid_pair = ParticlePair {
        distance: f32::NAN,
        ..pair([0, 1], 1.0)
    };

    // Act
    let range_result = MutationCandidate::prepare_ordinary_rotation(&storage, 0, 5, storage.len());
    let topology_result = MutationCandidate::prepare_join_groups(
        &storage,
        &[Some(0), Some(1), Some(2), Some(3)],
        vec![invalid_pair],
        Vec::new(),
    );

    // Assert
    assert!(matches!(
        range_result,
        Err(ParticleStorageError::InvalidPermutation)
    ));
    assert!(matches!(
        topology_result,
        Err(ParticleStorageError::InvalidLaneBundle)
    ));
    assert!(storage == before);
}

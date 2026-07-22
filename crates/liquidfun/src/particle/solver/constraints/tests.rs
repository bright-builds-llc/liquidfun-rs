use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
use crate::math::{Rotation, Vec2};
use crate::particle::storage::lanes::{ParticlePair, ParticleTriad};
use crate::particle::storage::{ParticleIndex, ParticleInput, ParticleStorage};
use crate::particle::{ParticleFlags, ParticleSystemDef};

use super::*;
use crate::particle::solver::PassId;
use crate::particle::solver::manifest::PASS_GRAPH;

fn pair(indices: [usize; 2], flags: ParticleFlags, distance: f32) -> ParticlePair {
    ParticlePair {
        indices: indices.map(ParticleIndex),
        flags,
        strength: 1.0,
        distance,
    }
}

fn triad(indices: [usize; 3], offsets: [Vec2; 3]) -> ParticleTriad {
    ParticleTriad {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::ELASTIC,
        strength: 1.0,
        pa: offsets[0],
        pb: offsets[1],
        pc: offsets[2],
        ka: 1.0,
        kb: 2.0,
        kc: 3.0,
        s: 4.0,
    }
}

fn bits(values: &[Vec2]) -> Vec<[u32; 2]> {
    values
        .iter()
        .map(|value| [value.x.to_bits(), value.y.to_bits()])
        .collect()
}

fn storage(velocities: &[Vec2]) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let mut storage = ParticleStorage::new(
        world,
        system,
        0,
        velocities.len().max(1),
        velocities.len().max(1),
    )
    .expect("test storage is valid");
    for (index, velocity) in velocities.iter().copied().enumerate() {
        let position_x =
            f32::from(u16::try_from(index).expect("small test particle index fits u16"));
        storage
            .create(ParticleInput {
                position: Vec2::new(position_x, 0.0),
                velocity,
                flags: ParticleFlags::WATER,
                maybe_group: None,
                maybe_color: None,
                maybe_user_association: None,
                maybe_expiration_time: None,
            })
            .expect("test particle fits");
    }
    storage
}

#[test]
fn manifest_admits_each_constraint_kernel_exactly_once_in_order() {
    // Arrange
    let expected = [PassId::Elastic, PassId::Spring, PassId::LimitVelocity];

    // Act
    let observed = PASS_GRAPH
        .iter()
        .filter(|descriptor| expected.contains(&descriptor.id))
        .map(|descriptor| descriptor.id)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(observed, expected);
}

#[test]
fn empty_and_inactive_topology_are_exact_controls() {
    // Arrange
    let positions = [Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)];
    let velocities = [Vec2::new(0.25, -0.5); 3];
    let inactive_pair = pair([0, 1], ParticleFlags::BARRIER, 1.0);
    let mut inactive_triad = triad(
        [0, 1, 2],
        [
            Vec2::new(-1.0 / 3.0, -1.0 / 3.0),
            Vec2::new(2.0 / 3.0, -1.0 / 3.0),
            Vec2::new(-1.0 / 3.0, 2.0 / 3.0),
        ],
    );
    inactive_triad.flags = ParticleFlags::SPRING;

    // Act
    let empty = spring_candidate(&positions, &velocities, &[], 0.25, 0.5, 2.0)
        .expect("empty topology is valid");
    let inactive = spring_candidate(&positions, &velocities, &[inactive_pair], 0.25, 0.5, 2.0)
        .expect("inactive pair is valid");
    let inactive_elastic =
        elastic_candidate(&positions, &velocities, &[inactive_triad], 0.25, 0.5, 2.0)
            .expect("inactive triad is valid");

    // Assert
    assert_eq!(empty, velocities);
    assert_eq!(inactive, velocities);
    assert_eq!(inactive_elastic, velocities);
}

#[test]
fn spring_uses_stored_rest_distance_and_default_quarter_strength() {
    // Arrange
    let positions = [Vec2::ZERO, Vec2::new(2.0, 0.0)];
    let velocities = [Vec2::ZERO; 2];
    let pair = pair([0, 1], ParticleFlags::SPRING, 1.0);

    // Act
    let result = spring_candidate(&positions, &velocities, &[pair], 0.25, 0.5, 2.0)
        .expect("spring candidate remains finite");

    // Assert
    assert_eq!(result, [Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0)]);
}

#[test]
fn elastic_uses_stored_offsets_and_default_quarter_strength() {
    // Arrange
    let positions = [Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0), Vec2::ZERO];
    let velocities = [Vec2::ZERO; 3];
    let record = triad(
        [0, 1, 2],
        [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::ZERO],
    );

    // Act
    let result = elastic_candidate(&positions, &velocities, &[record], 0.25, 0.5, 2.0)
        .expect("elastic candidate remains finite");

    // Assert
    assert_eq!(result[0].x.to_bits(), 0x3f00_3778);
    assert_eq!(result[1].x.to_bits(), 0xbf00_3778);
    assert_eq!(result[2], Vec2::ZERO);
}

#[test]
fn spring_interaction_observes_source_record_order() {
    // Arrange
    let positions = [
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(4.0, 0.0),
    ];
    let velocities = [Vec2::ZERO; 3];
    let first = pair([0, 1], ParticleFlags::SPRING, 1.0);
    let second = pair([1, 2], ParticleFlags::SPRING, 1.0);

    // Act
    let forward = spring_candidate(&positions, &velocities, &[first, second], 0.25, 0.5, 2.0)
        .expect("forward order remains finite");
    let reverse = spring_candidate(&positions, &velocities, &[second, first], 0.25, 0.5, 2.0)
        .expect("reverse order remains finite");

    // Assert
    assert_ne!(bits(&forward), bits(&reverse));
}

#[test]
fn zero_length_spring_is_probe_backed_typed_error_but_barrier_is_noop() {
    // Arrange
    let positions = [Vec2::ZERO, Vec2::ZERO];
    let velocities = [Vec2::ZERO; 2];
    let spring = pair([0, 1], ParticleFlags::SPRING, 0.0);
    let collapsed_prediction = pair([0, 1], ParticleFlags::SPRING, 1.0);
    let barrier = pair([0, 1], ParticleFlags::BARRIER, 0.0);
    let witness = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.json"
    ));
    let provenance = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.provenance.json"
    ));

    // Act
    let spring_result = spring_candidate(&positions, &velocities, &[spring], 0.25, 0.5, 2.0);
    let collapsed_result = spring_candidate(
        &positions,
        &velocities,
        &[collapsed_prediction],
        0.25,
        0.5,
        2.0,
    );
    let barrier_result = spring_candidate(&positions, &velocities, &[barrier], 0.25, 0.5, 2.0)
        .expect("barrier-only pair is inactive in S19");

    // Assert
    assert!(witness.contains("\"decision\": \"typed_error\",\n      \"id\": \"zero_length_pair\""));
    assert!(witness.contains("\"typed_invariant\": \"zero_length_pair_distance\""));
    assert!(
        witness.contains(
            "\"decision\": \"preserve_source_behavior\",\n      \"id\": \"barrier_pair\""
        )
    );
    assert!(provenance.contains(
        "\"witness_sha256\": \"90d212d3380fe9aa645ca9d972e39b962db9f912853850a9deb5943be2395278\""
    ));
    assert_eq!(
        spring_result,
        Err(ConstraintSolverError::ZeroLengthPairDistance)
    );
    assert_eq!(
        collapsed_result,
        Err(ConstraintSolverError::ZeroLengthPairDistance)
    );
    assert_eq!(barrier_result, velocities);
}

#[test]
fn degenerate_triad_preserves_probe_backed_finite_zero_behavior() {
    // Arrange
    let positions = [Vec2::ZERO; 3];
    let velocities = [Vec2::ZERO; 3];
    let degenerate = triad([0, 1, 2], [Vec2::ZERO; 3]);
    let witness = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.json"
    ));

    // Act
    let result = elastic_candidate(&positions, &velocities, &[degenerate], 0.25, 0.5, 2.0)
        .expect("probe-backed degenerate triad remains finite");

    // Assert
    assert!(witness.contains(
        "\"decision\": \"preserve_source_behavior\",\n      \"id\": \"degenerate_triad\""
    ));
    assert_eq!(result, velocities);
    assert!(result.iter().all(|velocity| velocity.is_valid()));
}

#[test]
fn elastic_validates_every_stored_triad_coefficient_before_solving() {
    // Arrange
    let positions = [Vec2::ZERO; 3];
    let velocities = [Vec2::ZERO; 3];
    let valid = triad([0, 1, 2], [Vec2::ZERO; 3]);
    let invalid = [
        ParticleTriad {
            pa: Vec2::new(f32::NAN, 0.0),
            ..valid
        },
        ParticleTriad {
            pb: Vec2::new(0.0, f32::INFINITY),
            ..valid
        },
        ParticleTriad {
            pc: Vec2::new(f32::NEG_INFINITY, 0.0),
            ..valid
        },
        ParticleTriad {
            ka: f32::NAN,
            ..valid
        },
        ParticleTriad {
            kb: f32::INFINITY,
            ..valid
        },
        ParticleTriad {
            kc: f32::NEG_INFINITY,
            ..valid
        },
        ParticleTriad {
            s: f32::NAN,
            ..valid
        },
    ];

    // Act
    let results =
        invalid.map(|record| elastic_candidate(&positions, &velocities, &[record], 0.25, 0.5, 2.0));

    // Assert
    assert!(results.iter().all(|result| {
        *result
            == Err(ConstraintSolverError::Storage(
                ParticleStorageError::InvalidLaneBundle,
            ))
    }));
}

#[test]
fn elastic_uses_stored_offsets_under_translation_and_rotation() {
    // Arrange
    let rest = [
        Vec2::new(-1.0, -1.0 / 3.0),
        Vec2::new(1.0, -1.0 / 3.0),
        Vec2::new(0.0, 2.0 / 3.0),
    ];
    let rotation = Rotation::from_angle(0.375);
    let translation = Vec2::new(7.0, -4.0);
    let positions = rest.map(|point| rotation.apply(point) + translation);
    let velocities = [Vec2::ZERO; 3];
    let record = triad([0, 1, 2], rest);

    // Act
    let result = elastic_candidate(&positions, &velocities, &[record], 0.25, 0.5, 2.0)
        .expect("transformed rest state remains finite");

    // Assert
    assert!(result.iter().all(|velocity| {
        velocity.is_valid() && velocity.length_squared() <= 16.0 * f32::EPSILON
    }));
}

#[test]
fn retargeted_topology_preserves_join_split_results_without_rebuilding_rest() {
    // Arrange
    let positions = [
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(4.0, 0.0),
    ];
    let velocities = [Vec2::ZERO; 3];
    let original = [
        pair([0, 1], ParticleFlags::SPRING, 1.0),
        pair([1, 2], ParticleFlags::SPRING, 1.0),
    ];
    let permutation = [2, 0, 1];
    let permuted_positions = permutation.map(|old| positions[old]);
    let permuted_velocities = permutation.map(|old| velocities[old]);
    let retargeted = [
        pair([1, 2], ParticleFlags::SPRING, 1.0),
        pair([2, 0], ParticleFlags::SPRING, 1.0),
    ];

    // Act
    let original_result = spring_candidate(&positions, &velocities, &original, 0.25, 0.5, 2.0)
        .expect("original topology remains finite");
    let permuted_result = spring_candidate(
        &permuted_positions,
        &permuted_velocities,
        &retargeted,
        0.25,
        0.5,
        2.0,
    )
    .expect("retargeted topology remains finite");

    // Assert
    for (new, old) in permutation.into_iter().enumerate() {
        assert_eq!(permuted_result[new], original_result[old]);
    }
    assert_eq!(
        retargeted.map(|record| record.distance.to_bits()),
        original.map(|record| record.distance.to_bits())
    );
}

#[test]
fn limit_velocity_preserves_exact_threshold_and_clamps_only_above_it() {
    // Arrange
    let below = f32::from_bits(4.0_f32.to_bits() - 1);
    let above = f32::from_bits(4.0_f32.to_bits() + 1);
    let velocities = [
        Vec2::new(below, 0.0),
        Vec2::new(4.0, 0.0),
        Vec2::new(above, 0.0),
    ];

    // Act
    let result = limit_velocity_candidate(&velocities, 2.0, 2.0)
        .expect("finite critical threshold is valid");

    // Assert
    assert_eq!(result[0].x.to_bits(), below.to_bits());
    assert_eq!(result[1].x.to_bits(), 4.0_f32.to_bits());
    assert_eq!(result[2].x.to_bits(), 4.0_f32.to_bits());
}

#[test]
fn storage_shell_commits_one_finite_velocity_candidate() {
    // Arrange
    let mut storage = storage(&[Vec2::new(8.0, 0.0), Vec2::new(0.0, -1.0)]);
    let definition = ParticleSystemDef::default();
    let ids = storage.particle_ids().to_vec();

    // Act
    limit_velocity(&mut storage, definition, 1.0).expect("velocity limit succeeds");

    // Assert
    assert_eq!(storage.particle_ids(), ids);
    assert_eq!(
        storage.velocities(),
        &[Vec2::new(2.0, 0.0), Vec2::new(0.0, -1.0)]
    );
}

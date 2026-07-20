use crate::identity::{
    HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::Vec2;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache, InternalGroupFlags};
use crate::particle::storage::lanes::{ParticleContact, ParticlePair, ParticleTriad};
use crate::particle::storage::{
    ParticleIndex, ParticleInput, ParticleStorage, ParticleStorageError,
};
use crate::particle::topology::VoronoiLimits;
use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::{JoinPlanError, JoinTopologyParameters};

const DIAMETER: f32 = 1.0;

struct Fixture {
    storage: ParticleStorage,
    group_a: ParticleGroupId,
    group_b: ParticleGroupId,
    group_c: ParticleGroupId,
    group_d: ParticleGroupId,
    ids: [ParticleId; 8],
}

fn topology_parameters() -> JoinTopologyParameters {
    JoinTopologyParameters::new(
        DIAMETER,
        VoronoiLimits::new(64, 4_096, 16_384, 2_000_000, 8_192),
    )
}

fn particle(value: f32, group: ParticleGroupId, flags: ParticleFlags) -> ParticleInput {
    ParticleInput {
        position: Vec2::new(value, 0.0),
        velocity: Vec2::new(value, -value),
        flags,
        maybe_group: Some(group),
        maybe_color: None,
        maybe_user_association: None,
        maybe_expiration_time: None,
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

fn contact(indices: [usize; 2]) -> ParticleContact {
    ParticleContact {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::SPRING,
        weight: 0.5,
        normal: Vec2::new(1.0, 0.0),
    }
}

fn fixture(flags: ParticleFlags) -> Fixture {
    let world = WorldKey::fresh().expect("test world key remains available");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let [group_a, group_b, group_c, group_d] =
        [10, 11, 12, 13].map(|slot| ParticleGroupId::from_identity(Identity::new(world, slot, 0)));
    let mut storage =
        ParticleStorage::new(world, system, 0, 16, 16).expect("test storage is valid");
    let inputs = [
        particle(0.0, group_a, flags),
        particle(1.0, group_a, flags),
        particle(2.0, group_a, flags),
        particle(3.0, group_c, flags),
        particle(4.0, group_b, flags),
        particle(5.0, group_b, flags),
        particle(6.0, group_b, flags),
        particle(7.0, group_d, flags),
    ];
    let mut ids = Vec::new();
    for input in inputs {
        ids.push(storage.create(input).expect("fixture particle fits"));
    }
    let ids: [ParticleId; 8] = ids.try_into().expect("fixture has eight particles");
    storage.particle_contacts = vec![
        contact([0, 4]),
        contact([1, 5]),
        contact([0, 1]),
        contact([4, 5]),
    ];
    storage.weights = vec![1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
    storage.pairs = vec![pair([0, 1], 10.0), pair([4, 5], 20.0)];
    storage.triads = vec![triad([0, 1, 2], 30.0), triad([4, 5, 6], 40.0)];
    Fixture {
        storage,
        group_a,
        group_b,
        group_c,
        group_d,
        ids,
    }
}

fn record_mut(storage: &mut ParticleStorage, group: ParticleGroupId) -> &mut GroupRecord {
    storage
        .group_records
        .iter_mut()
        .find(|record| record.id == group)
        .expect("fixture group has a record")
}

#[test]
fn join_rotates_source_groups_and_preserves_every_identity() {
    // Arrange
    let mut fixture = fixture(ParticleFlags::SPRING);
    let original_inputs = fixture
        .ids
        .map(|id| fixture.storage.input(id).expect("fixture identity is live"));

    // Act
    let plan = fixture
        .storage
        .plan_join(fixture.group_a, fixture.group_b, topology_parameters())
        .expect("join candidate validates");
    plan.commit(&mut fixture.storage);

    // Assert
    assert_eq!(
        fixture.storage.particle_ids(),
        &[
            fixture.ids[3],
            fixture.ids[7],
            fixture.ids[0],
            fixture.ids[1],
            fixture.ids[2],
            fixture.ids[4],
            fixture.ids[5],
            fixture.ids[6],
        ]
    );
    for (id, input) in fixture.ids.into_iter().zip(original_inputs) {
        let mut expected = input;
        if expected.maybe_group == Some(fixture.group_b) {
            expected.maybe_group = Some(fixture.group_a);
        }
        assert_eq!(fixture.storage.input(id), Ok(expected));
    }
    assert_eq!(fixture.storage.groups()[0], Some(fixture.group_c));
    assert_eq!(fixture.storage.groups()[1], Some(fixture.group_d));
    assert_eq!(
        fixture
            .storage
            .plan_join(fixture.group_a, fixture.group_b, topology_parameters())
            .map(|_| ()),
        Err(JoinPlanError::Storage(
            ParticleStorageError::StaleOrDestroyed
        ))
    );
}

#[test]
fn join_preserves_historical_topology_bytes_and_appends_cross_pairs_only() {
    // Arrange
    let mut fixture = fixture(ParticleFlags::SPRING);
    let historical_pair_bits = fixture
        .storage
        .pairs
        .iter()
        .map(|pair| (pair.strength.to_bits(), pair.distance.to_bits()))
        .collect::<Vec<_>>();
    let historical_triad_bits = fixture
        .storage
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
        .collect::<Vec<_>>();

    // Act
    fixture
        .storage
        .plan_join(fixture.group_a, fixture.group_b, topology_parameters())
        .expect("join candidate validates")
        .commit(&mut fixture.storage);

    // Assert
    assert_eq!(
        fixture.storage.pairs[..2]
            .iter()
            .map(|pair| (pair.strength.to_bits(), pair.distance.to_bits()))
            .collect::<Vec<_>>(),
        historical_pair_bits
    );
    assert_eq!(
        fixture.storage.triads[..2]
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
            .collect::<Vec<_>>(),
        historical_triad_bits
    );
    assert_eq!(
        fixture.storage.pairs[2..]
            .iter()
            .map(|pair| pair.indices.map(|index| index.0))
            .collect::<Vec<_>>(),
        vec![[2, 5], [3, 6]]
    );
    assert_eq!(fixture.storage.triads.len(), 2);
}

#[test]
fn join_unions_flags_and_invalidates_the_surviving_group_cache() {
    // Arrange
    let mut fixture = fixture(ParticleFlags::SPRING);
    let cached = GroupStatisticsCache {
        maybe_source_timestamp: Some(7),
        mass: 2.0,
        center: Vec2::new(1.0, 2.0),
        linear_velocity: Vec2::new(3.0, 4.0),
        inertia: 5.0,
        angular_velocity: 6.0,
    };
    let group_a = record_mut(&mut fixture.storage, fixture.group_a);
    group_a.flags = ParticleGroupFlags::RIGID;
    group_a.statistics = cached;
    let group_b = record_mut(&mut fixture.storage, fixture.group_b);
    group_b.flags = ParticleGroupFlags::SOLID;
    group_b
        .internal_flags
        .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
    fixture
        .storage
        .solver_state
        .refresh_group_flags(&fixture.storage.group_records);

    // Act
    fixture
        .storage
        .plan_join(fixture.group_a, fixture.group_b, topology_parameters())
        .expect("join candidate validates")
        .commit(&mut fixture.storage);

    // Assert
    let surviving = fixture
        .storage
        .group_records
        .iter()
        .find(|record| record.id == fixture.group_a)
        .expect("group A survives");
    assert_eq!(
        surviving.flags,
        ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID
    );
    assert!(
        surviving
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH)
    );
    assert_eq!(surviving.statistics.maybe_source_timestamp, None);
    assert!(
        fixture
            .storage
            .group_records
            .iter()
            .all(|record| record.id != fixture.group_b)
    );
}

#[test]
fn join_accepts_empty_a_or_b_without_losing_the_nonempty_members() {
    // Arrange
    let mut a_empty = fixture(ParticleFlags::SPRING);
    make_group_empty(&mut a_empty.storage, a_empty.group_a);
    let mut b_empty = fixture(ParticleFlags::SPRING);
    make_group_empty(&mut b_empty.storage, b_empty.group_b);

    // Act
    a_empty
        .storage
        .plan_join(a_empty.group_a, a_empty.group_b, topology_parameters())
        .expect("empty A can join")
        .commit(&mut a_empty.storage);
    b_empty
        .storage
        .plan_join(b_empty.group_a, b_empty.group_b, topology_parameters())
        .expect("empty B can join")
        .commit(&mut b_empty.storage);

    // Assert
    assert_eq!(
        a_empty
            .storage
            .groups()
            .iter()
            .filter(|maybe_group| **maybe_group == Some(a_empty.group_a))
            .count(),
        3
    );
    assert_eq!(
        b_empty
            .storage
            .groups()
            .iter()
            .filter(|maybe_group| **maybe_group == Some(b_empty.group_a))
            .count(),
        3
    );
}

fn make_group_empty(storage: &mut ParticleStorage, group: ParticleGroupId) {
    storage.groups.iter_mut().for_each(|maybe_group| {
        if *maybe_group == Some(group) {
            *maybe_group = None;
        }
    });
    let record = record_mut(storage, group);
    record.flags |= ParticleGroupFlags::CAN_BE_EMPTY;
    record.set_range(0..0);
    storage
        .group_records
        .sort_by_key(|record| usize::from(record.first == record.last));
    storage
        .solver_state
        .refresh_group_flags(&storage.group_records);
}

#[test]
fn aliased_stale_and_wrong_system_handles_fail_without_effects() {
    // Arrange
    let fixture = fixture(ParticleFlags::SPRING);
    let stale = ParticleGroupId::from_identity(Identity::new(fixture.storage.world, 99, 4));
    let mut wrong_system_storage = fixture.storage.clone();
    record_mut(&mut wrong_system_storage, fixture.group_b).system =
        ParticleSystemId::from_identity(Identity::new(fixture.storage.world, 8, 0));
    let before = fixture.storage.clone();
    let wrong_before = wrong_system_storage.clone();

    // Act
    let aliased =
        fixture
            .storage
            .plan_join(fixture.group_a, fixture.group_a, topology_parameters());
    let stale_result = fixture
        .storage
        .plan_join(fixture.group_a, stale, topology_parameters());
    let wrong_system =
        wrong_system_storage.plan_join(fixture.group_a, fixture.group_b, topology_parameters());

    // Assert
    assert_eq!(
        aliased.map(|_| ()),
        Err(JoinPlanError::Storage(
            ParticleStorageError::InvalidGroupRange
        ))
    );
    assert_eq!(
        stale_result.map(|_| ()),
        Err(JoinPlanError::Storage(
            ParticleStorageError::StaleOrDestroyed
        ))
    );
    assert_eq!(
        wrong_system.map(|_| ()),
        Err(JoinPlanError::Storage(
            ParticleStorageError::WrongParticleSystem
        ))
    );
    assert!(fixture.storage == before);
    assert!(wrong_system_storage == wrong_before);
}

#[test]
fn bounded_topology_failure_leaves_the_complete_storage_snapshot_unchanged() {
    // Arrange
    let fixture = fixture(ParticleFlags::SPRING | ParticleFlags::ELASTIC);
    let before = fixture.storage.clone();
    let constrained = JoinTopologyParameters::new(DIAMETER, VoronoiLimits::new(1, 1, 1, 1, 1));

    // Act
    let result = fixture
        .storage
        .plan_join(fixture.group_a, fixture.group_b, constrained);

    // Assert
    assert!(matches!(result, Err(JoinPlanError::Constraints(_))));
    assert!(fixture.storage == before);
}

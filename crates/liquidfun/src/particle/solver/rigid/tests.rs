use crate::identity::{
    BodyId, HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::{Rotation, Transform, Vec2};
use crate::particle::ParticleGroupFlags;
use crate::particle::solver::PassId;
use crate::particle::solver::manifest::PASS_GRAPH;
use crate::particle::storage::ParticleIndex;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache};
use crate::particle::storage::lanes::ParticleContact;

use super::*;

struct Fixture {
    owner: ParticleSystemId,
    group_ids: [ParticleGroupId; 2],
    particle_ids: Vec<ParticleId>,
}

impl Fixture {
    fn new(particle_count: usize) -> Self {
        let world = WorldKey::fresh().expect("test world key remains available");
        let owner = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let group_ids =
            [1, 2].map(|slot| ParticleGroupId::from_identity(Identity::new(world, slot, 0)));
        let particle_ids = (0..particle_count)
            .map(|slot| {
                ParticleId::from_identity(Identity::new_particle(world, slot, 0, owner.identity()))
            })
            .collect();
        Self {
            owner,
            group_ids,
            particle_ids,
        }
    }

    fn group(
        &self,
        ordinal: usize,
        range: std::ops::Range<usize>,
        flags: ParticleGroupFlags,
    ) -> GroupRecord {
        let mut group = GroupRecord::new(self.group_ids[ordinal], self.owner, range);
        group.flags = flags;
        group
    }
}

fn contact(indices: [usize; 2], normal: Vec2, weight: f32) -> ParticleContact {
    ParticleContact {
        indices: indices.map(ParticleIndex),
        flags: crate::ParticleFlags::WATER,
        weight,
        normal,
    }
}

fn bits(values: &[Vec2]) -> Vec<[u32; 2]> {
    values
        .iter()
        .map(|value| [value.x.to_bits(), value.y.to_bits()])
        .collect()
}

#[test]
fn manifest_places_rigid_damping_before_rigid_projection() {
    // Arrange
    let expected = [PassId::RigidDamping, PassId::Rigid];

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
fn non_rigid_group_is_an_exact_control() {
    // Arrange
    let fixture = Fixture::new(2);
    let group = fixture.group(0, 0..2, ParticleGroupFlags::SOLID);
    let positions = [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)];
    let velocities = [Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)];
    let memberships = [Some(fixture.group_ids[0]); 2];

    // Act
    let result = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &positions,
        &velocities,
        &memberships,
        &[group],
        &[],
        &[],
        1.0,
        0.2,
        4,
        0,
    )
    .expect("non-rigid control is valid");

    // Assert
    assert_eq!(result.velocities, velocities);
    assert_eq!(result.groups, [group]);
    assert!(result.body_impulses.is_empty());
}

#[test]
fn rigid_statistics_and_projection_preserve_identity_order_and_association() {
    // Arrange
    let fixture = Fixture::new(2);
    let mut group = fixture.group(0, 0..2, ParticleGroupFlags::RIGID);
    group.maybe_user_association =
        Some(crate::particle::storage::lanes::UserAssociationKey::new(77));
    let positions = [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)];
    let velocities = [Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0)];
    let memberships = [Some(fixture.group_ids[0]); 2];

    // Act
    let damped = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &positions,
        &velocities,
        &memberships,
        &[group],
        &[],
        &[],
        2.0,
        0.2,
        9,
        0,
    )
    .expect("rigid statistics remain finite");
    let projected = rigid_projection_candidate(
        fixture.owner,
        &damped.particle_ids,
        &positions,
        &damped.velocities,
        &memberships,
        &damped.groups,
        0.5,
        2.0,
    )
    .expect("rigid projection remains finite");

    // Assert
    assert_eq!(projected.particle_ids, fixture.particle_ids);
    assert_eq!(projected.groups[0].id, group.id);
    assert_eq!(
        projected.groups[0].maybe_user_association,
        group.maybe_user_association
    );
    assert_eq!(projected.groups[0].range(), 0..2);
    assert_eq!(
        damped.groups[0].statistics.mass.to_bits(),
        4.0_f32.to_bits()
    );
    assert_eq!(damped.groups[0].statistics.center, Vec2::ZERO);
    assert_eq!(
        damped.groups[0].statistics.linear_velocity,
        Vec2::new(1.0, 0.0)
    );
    assert_eq!(
        damped.groups[0].statistics.inertia.to_bits(),
        4.0_f32.to_bits()
    );
    assert_eq!(
        damped.groups[0].statistics.angular_velocity.to_bits(),
        1.0_f32.to_bits()
    );
    assert_eq!(
        projected.groups[0].transform.rotation(),
        Rotation::from_angle(0.5)
    );
}

#[test]
fn translated_and_rotated_projection_uses_source_transform_order() {
    // Arrange
    let fixture = Fixture::new(2);
    let mut group = fixture.group(0, 0..2, ParticleGroupFlags::RIGID);
    group.transform =
        Transform::from_position_angle(Vec2::new(2.0, -1.0), crate::math::settings::TAU / 8.0);
    group.statistics = GroupStatisticsCache {
        maybe_source_timestamp: Some(3),
        mass: 2.0,
        center: Vec2::new(1.0, 2.0),
        linear_velocity: Vec2::new(3.0, -2.0),
        inertia: 4.0,
        angular_velocity: 0.5,
    };
    let positions = [Vec2::new(0.0, 2.0), Vec2::new(2.0, 2.0)];
    let velocities = [Vec2::ZERO; 2];
    let memberships = [Some(fixture.group_ids[0]); 2];

    // Act
    let projected = rigid_projection_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &positions,
        &velocities,
        &memberships,
        &[group],
        0.25,
        4.0,
    )
    .expect("translated rotation remains finite");
    let rotation = Rotation::from_angle(0.125);
    let translation = group.statistics.center + 0.25 * group.statistics.linear_velocity
        - rotation.apply(group.statistics.center);
    let expected = Transform::new(translation, rotation).compose(group.transform);

    // Assert
    assert_eq!(projected.groups[0].transform, expected);
    assert_ne!(bits(&projected.velocities), bits(&velocities));
}

#[test]
fn two_rigid_groups_exchange_damping_in_source_contact_order() {
    // Arrange
    let fixture = Fixture::new(2);
    let groups = [
        fixture.group(0, 0..1, ParticleGroupFlags::RIGID),
        fixture.group(1, 1..2, ParticleGroupFlags::RIGID),
    ];
    let positions = [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)];
    let velocities = [Vec2::new(2.0, 0.0), Vec2::new(-2.0, 0.0)];
    let memberships = [Some(fixture.group_ids[0]), Some(fixture.group_ids[1])];

    // Act
    let result = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &positions,
        &velocities,
        &memberships,
        &groups,
        &[contact([0, 1], Vec2::new(1.0, 0.0), 1.0)],
        &[],
        1.0,
        0.5,
        1,
        0,
    )
    .expect("two-group damping is finite");

    // Assert
    assert_eq!(
        result.groups[0].statistics.linear_velocity,
        Vec2::new(1.0, 0.0)
    );
    assert_eq!(
        result.groups[1].statistics.linear_velocity,
        Vec2::new(-1.0, 0.0)
    );
}

#[test]
fn rigid_body_contact_emits_candidate_without_mutating_identity() {
    // Arrange
    let fixture = Fixture::new(1);
    let group = fixture.group(0, 0..1, ParticleGroupFlags::RIGID);
    let positions = [Vec2::ZERO];
    let velocities = [Vec2::new(2.0, 0.0)];
    let memberships = [Some(fixture.group_ids[0])];
    let body = BodyId::from_identity(Identity::new(fixture.owner.identity().world(), 9, 0));
    let body_contact = RigidBodyContact {
        particle: 0,
        body,
        weight: 1.0,
        normal: Vec2::new(1.0, 0.0),
        body_mass: 1.0,
        body_inertia: 0.0,
        body_center: Vec2::ZERO,
        body_linear_velocity: Vec2::ZERO,
        body_angular_velocity: 0.0,
    };

    // Act
    let result = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &positions,
        &velocities,
        &memberships,
        &[group],
        &[],
        &[body_contact],
        1.0,
        0.5,
        1,
        1,
    )
    .expect("body coupling emits one candidate");

    // Assert
    assert_eq!(result.particle_ids, fixture.particle_ids);
    assert_eq!(result.body_impulses.len(), 1);
    assert_eq!(result.body_impulses[0].body, body);
    assert_eq!(result.body_impulses[0].impulse, Vec2::new(0.5, 0.0));
    assert_eq!(
        result.groups[0].statistics.linear_velocity,
        Vec2::new(1.5, 0.0)
    );
}

#[test]
fn body_candidate_limit_fails_without_partial_result() {
    // Arrange
    let fixture = Fixture::new(1);
    let group = fixture.group(0, 0..1, ParticleGroupFlags::RIGID);
    let body = BodyId::from_identity(Identity::new(fixture.owner.identity().world(), 9, 0));
    let contact = RigidBodyContact {
        particle: 0,
        body,
        weight: 1.0,
        normal: Vec2::new(1.0, 0.0),
        body_mass: 1.0,
        body_inertia: 0.0,
        body_center: Vec2::ZERO,
        body_linear_velocity: Vec2::ZERO,
        body_angular_velocity: 0.0,
    };

    // Act
    let result = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &[Vec2::ZERO],
        &[Vec2::new(2.0, 0.0)],
        &[Some(fixture.group_ids[0])],
        &[group],
        &[],
        &[contact],
        1.0,
        0.5,
        1,
        0,
    );

    // Assert
    assert_eq!(
        result,
        Err(RigidSolverError::ResourceLimit {
            resource: "rigid body impulse candidates",
            limit: 0,
        })
    );
}

#[test]
fn empty_and_one_particle_probe_classifications_are_preserved() {
    // Arrange
    let empty_fixture = Fixture::new(0);
    let mut empty = empty_fixture.group(
        0,
        0..0,
        ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY,
    );
    empty.statistics = GroupStatisticsCache::INVALIDATED_ZERO;
    let one_fixture = Fixture::new(1);
    let one = one_fixture.group(0, 0..1, ParticleGroupFlags::RIGID);
    let witness = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.json"
    ));
    let provenance = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/particle/testdata/group-topology-witnesses.provenance.json"
    ));

    // Act
    let empty_result = rigid_damping_candidate(
        empty_fixture.owner,
        &[],
        &[],
        &[],
        &[],
        &[empty],
        &[],
        &[],
        0.5625,
        0.2,
        8,
        0,
    )
    .expect("retained empty rigid group is finite");
    let one_damped = rigid_damping_candidate(
        one_fixture.owner,
        &one_fixture.particle_ids,
        &[Vec2::new(2.0, -3.0)],
        &[Vec2::new(7.5, 3.75)],
        &[Some(one_fixture.group_ids[0])],
        &[one],
        &[],
        &[],
        0.5625,
        0.2,
        9,
        0,
    )
    .expect("one-particle rigid statistics are finite");
    let one_projected = rigid_projection_candidate(
        one_fixture.owner,
        &one_damped.particle_ids,
        &[Vec2::new(2.0, -3.0)],
        &one_damped.velocities,
        &[Some(one_fixture.group_ids[0])],
        &one_damped.groups,
        1.0 / 60.0,
        60.0,
    )
    .expect("one-particle projection is finite");

    // Assert
    assert!(witness.contains(
        "\"decision\": \"preserve_source_behavior\",\n      \"id\": \"rigid_group_empty\""
    ));
    assert!(witness.contains(
        "\"decision\": \"preserve_source_behavior\",\n      \"id\": \"rigid_group_one_particle\""
    ));
    assert!(provenance.contains(
        "\"witness_sha256\": \"90d212d3380fe9aa645ca9d972e39b962db9f912853850a9deb5943be2395278\""
    ));
    assert_eq!(empty_result.groups[0].statistics.mass.to_bits(), 0);
    assert_eq!(empty_result.groups[0].statistics.center, Vec2::ZERO);
    assert_eq!(
        empty_result.groups[0].statistics.linear_velocity,
        Vec2::ZERO
    );
    assert_eq!(empty_result.groups[0].statistics.inertia.to_bits(), 0);
    assert_eq!(
        empty_result.groups[0].statistics.angular_velocity.to_bits(),
        0
    );
    assert_eq!(empty_result.groups[0].transform, Transform::IDENTITY);
    assert_eq!(one_damped.groups[0].statistics.mass.to_bits(), 0x3f10_0000);
    assert_eq!(one_damped.groups[0].statistics.center, Vec2::new(2.0, -3.0));
    assert_eq!(one_damped.groups[0].statistics.inertia.to_bits(), 0);
    assert_eq!(
        one_damped.groups[0].statistics.angular_velocity.to_bits(),
        0
    );
    assert_eq!(
        bits(&one_projected.velocities),
        vec![[0x40f0_0000, 0x4070_0000]]
    );
    assert_eq!(
        one_projected.groups[0].transform.position(),
        Vec2::new(0.125, 0.0625)
    );
}

#[test]
fn stale_group_membership_and_cross_system_particle_are_rejected() {
    // Arrange
    let fixture = Fixture::new(1);
    let group = fixture.group(0, 0..1, ParticleGroupFlags::RIGID);
    let other = Fixture::new(1);

    // Act
    let stale_membership = rigid_damping_candidate(
        fixture.owner,
        &fixture.particle_ids,
        &[Vec2::ZERO],
        &[Vec2::ZERO],
        &[Some(fixture.group_ids[1])],
        &[group],
        &[],
        &[],
        1.0,
        0.2,
        1,
        0,
    );
    let cross_system = rigid_damping_candidate(
        fixture.owner,
        &other.particle_ids,
        &[Vec2::ZERO],
        &[Vec2::ZERO],
        &[Some(fixture.group_ids[0])],
        &[group],
        &[],
        &[],
        1.0,
        0.2,
        1,
        0,
    );

    // Assert
    assert_eq!(stale_membership, Err(RigidSolverError::InvalidInput));
    assert_eq!(cross_system, Err(RigidSolverError::InvalidInput));
}

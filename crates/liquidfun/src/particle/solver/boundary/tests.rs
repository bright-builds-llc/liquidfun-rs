use crate::ParticleFlags;
use crate::identity::{
    BodyId, HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::{Rotation, Transform, Vec2};
use crate::particle::ParticleGroupFlags;
use crate::particle::solver::PassId;
use crate::particle::solver::manifest::PASS_GRAPH;
use crate::particle::storage::ParticleIndex;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache};
use crate::particle::storage::lanes::ParticlePair;

use super::*;

struct Fixture {
    owner: ParticleSystemId,
    group_ids: [ParticleGroupId; 2],
    particle_ids: Vec<ParticleId>,
    body: BodyId,
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
        let body = BodyId::from_identity(Identity::new(world, 8, 0));
        Self {
            owner,
            group_ids,
            particle_ids,
            body,
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

    #[allow(
        clippy::too_many_arguments,
        reason = "the fixture exposes every aligned candidate lane to each focused witness"
    )]
    fn candidate(
        &self,
        positions: &[Vec2],
        velocities: &[Vec2],
        flags: &[ParticleFlags],
        memberships: &[Option<ParticleGroupId>],
        groups: &[GroupRecord],
        effect_limit: usize,
    ) -> BoundaryCandidate {
        BoundaryCandidate::new(
            self.owner,
            &self.particle_ids,
            positions,
            velocities,
            &vec![Vec2::ZERO; positions.len()],
            flags,
            memberships,
            groups,
            false,
            effect_limit,
        )
        .expect("test boundary candidate is valid")
    }
}

fn pair(indices: [usize; 2], flags: ParticleFlags) -> ParticlePair {
    ParticlePair {
        indices: indices.map(ParticleIndex),
        flags,
        strength: 1.0,
        distance: 2.0,
    }
}

fn pass_barrier(candidate: &BoundaryCandidate) -> BoundaryCandidate {
    barrier_candidate(candidate, &[], 1.0, 1.0, 1.0, 0).expect("empty barrier pass is valid")
}

fn pass_collision(candidate: &BoundaryCandidate) -> BoundaryCandidate {
    collision_candidate(candidate, &[], 0, 1.0, 1.0, 1.0, 0).expect("empty collision pass is valid")
}

fn bits(values: &[Vec2]) -> Vec<[u32; 2]> {
    values
        .iter()
        .map(|value| [value.x.to_bits(), value.y.to_bits()])
        .collect()
}

#[test]
fn manifest_admits_s22_through_s26_exactly_once_in_order() {
    // Arrange
    let expected = [
        PassId::Barrier,
        PassId::Collision,
        PassId::Rigid,
        PassId::Wall,
        PassId::Integrate,
    ];

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
fn barrier_activation_stops_crossing_particle_and_preserves_follow_up_force() {
    // Arrange
    let fixture = Fixture::new(3);
    let group = fixture.group(0, 0..2, ParticleGroupFlags::empty());
    let source = fixture.candidate(
        &[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::ZERO],
        &[Vec2::ZERO, Vec2::ZERO, Vec2::new(0.0, 1.0)],
        &[
            ParticleFlags::BARRIER,
            ParticleFlags::BARRIER,
            ParticleFlags::WATER,
        ],
        &[Some(fixture.group_ids[0]), Some(fixture.group_ids[0]), None],
        &[group],
        1,
    );

    // Act
    let result = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0,
        1.0,
        3,
    )
    .expect("crossing is handled");

    // Assert
    assert_eq!(result.velocities[2], Vec2::ZERO);
    assert_eq!(result.forces[2], Vec2::new(0.0, 1.0));
    assert!(result.has_pending_force);
    assert_eq!(result.effects.len(), 1);
    assert_eq!(result.effects[0].pass, BoundaryPass::Barrier);
    assert_eq!(result.effects[0].maybe_body, None);
}

#[test]
fn barrier_wall_endpoints_are_zeroed_before_crossing_scan() {
    // Arrange
    let fixture = Fixture::new(2);
    let source = fixture.candidate(
        &[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)],
        &[Vec2::new(4.0, 2.0), Vec2::new(-3.0, 7.0)],
        &[
            ParticleFlags::BARRIER | ParticleFlags::WALL,
            ParticleFlags::BARRIER | ParticleFlags::WALL,
        ],
        &[None, None],
        &[],
        0,
    );

    // Act
    let result = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0,
        1.0,
        2,
    )
    .expect("barrier-wall endpoints are valid");

    // Assert
    assert_eq!(result.velocities, [Vec2::ZERO; 2]);
    assert_eq!(result.forces, [Vec2::ZERO; 2]);
}

#[test]
fn collapsed_barrier_pair_preserves_probe_backed_finite_noop() {
    // Arrange
    let fixture = Fixture::new(3);
    let group = fixture.group(0, 0..2, ParticleGroupFlags::empty());
    let source = fixture.candidate(
        &[Vec2::ZERO, Vec2::ZERO, Vec2::new(0.25, 0.0)],
        &[Vec2::ZERO; 3],
        &[
            ParticleFlags::BARRIER,
            ParticleFlags::BARRIER,
            ParticleFlags::WATER,
        ],
        &[Some(fixture.group_ids[0]), Some(fixture.group_ids[0]), None],
        &[group],
        1,
    );
    let witness = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reference/artifacts/phase10/group-topology-witnesses.json"
    ));

    // Act
    let result = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0 / 60.0,
        60.0,
        3,
    )
    .expect("collapsed barrier remains a finite no-op");

    // Assert
    assert!(
        witness.contains(
            "\"decision\": \"preserve_source_behavior\",\n      \"id\": \"barrier_pair\""
        )
    );
    assert_eq!(result.velocities, source.velocities);
    assert_eq!(result.forces, source.forces);
    assert!(!result.has_pending_force);
    assert!(result.effects.is_empty());
}

#[test]
fn collision_start_uses_previous_transform_only_on_first_iteration() {
    // Arrange
    let position = Vec2::new(0.5, -0.25);
    let previous = Transform::from_position_angle(Vec2::new(-1.0, 0.0), 0.0);
    let current = Transform::from_position_angle(Vec2::new(2.0, 0.0), 0.0);

    // Act
    let first =
        collision_start_from_previous_transform(position, previous, current, Vec2::ZERO, false, 0)
            .expect("first-iteration transform is finite");
    let later =
        collision_start_from_previous_transform(position, previous, current, Vec2::ZERO, false, 1)
            .expect("later iteration is finite");

    // Assert
    assert_eq!(first, Vec2::new(3.5, -0.25));
    assert_eq!(later, position);
    assert_ne!(first, later);
}

#[test]
fn circle_collision_start_rotates_about_local_center() {
    // Arrange
    let position = Vec2::new(2.0, 0.0);
    let previous = Transform::IDENTITY;
    let current = Transform::new(
        Vec2::ZERO,
        Rotation::from_angle(crate::math::settings::TAU / 4.0),
    );

    // Act
    let circle = collision_start_from_previous_transform(
        position,
        previous,
        current,
        Vec2::new(1.0, 0.0),
        true,
        0,
    )
    .expect("circle correction remains finite");
    let non_circle = collision_start_from_previous_transform(
        position,
        previous,
        current,
        Vec2::new(1.0, 0.0),
        false,
        0,
    )
    .expect("ordinary correction remains finite");

    // Assert
    assert_ne!(circle, non_circle);
    assert_eq!(
        [circle.x.to_bits(), circle.y.to_bits()],
        [0x3f7f_ffff, 1.0_f32.to_bits()]
    );
}

#[test]
fn collision_applies_filtered_hit_in_stable_order_and_records_force() {
    // Arrange
    let fixture = Fixture::new(1);
    let source = pass_barrier(&fixture.candidate(
        &[Vec2::ZERO],
        &[Vec2::new(2.0, 0.0)],
        &[ParticleFlags::WATER],
        &[None],
        &[],
        1,
    ));
    let hit = FilteredCollisionHit {
        particle: 0,
        body: fixture.body,
        previous_transform: Transform::IDENTITY,
        current_transform: Transform::IDENTITY,
        body_local_center: Vec2::ZERO,
        is_circle: false,
        fraction: 0.5,
        normal: Vec2::new(0.0, 1.0),
    };

    // Act
    let result = collision_candidate(&source, &[hit], 0, 1.0, 1.0, 1.0, 1)
        .expect("filtered hit produces a candidate");

    // Assert
    assert_eq!(
        bits(&result.velocities),
        vec![[1.0_f32.to_bits(), 0.005_f32.to_bits()]]
    );
    assert_eq!(
        bits(&result.forces),
        vec![[1.0_f32.to_bits(), (-0.005_f32).to_bits()]]
    );
    assert_eq!(result.effects[0].maybe_body, Some(fixture.body));
    assert_eq!(result.effects[0].pass, BoundaryPass::Collision);
}

#[test]
fn stationary_collision_control_with_no_filtered_hits_is_exact() {
    // Arrange
    let fixture = Fixture::new(1);
    let source = pass_barrier(&fixture.candidate(
        &[Vec2::new(3.0, -2.0)],
        &[Vec2::ZERO],
        &[ParticleFlags::WATER],
        &[None],
        &[],
        0,
    ));

    // Act
    let result = collision_candidate(&source, &[], 0, 1.0, 1.0, 1.0, 0)
        .expect("empty filtered query is valid");

    // Assert
    assert_eq!(result.positions, source.positions);
    assert_eq!(result.velocities, source.velocities);
    assert_eq!(result.forces, source.forces);
    assert!(result.effects.is_empty());
}

#[test]
fn wall_targets_only_wall_particles_after_rigid_projection() {
    // Arrange
    let fixture = Fixture::new(2);
    let source = fixture.candidate(
        &[Vec2::ZERO; 2],
        &[Vec2::new(5.0, -1.0), Vec2::new(2.0, 3.0)],
        &[ParticleFlags::WALL, ParticleFlags::WATER],
        &[None; 2],
        &[],
        0,
    );
    let collided = pass_collision(&pass_barrier(&source));
    let rigid = mark_rigid_projection(&collided).expect("rigid pass marker is ordered");

    // Act
    let result = wall_candidate(&rigid).expect("wall pass is ordered");

    // Assert
    assert_eq!(result.velocities[0], Vec2::ZERO);
    assert_eq!(result.velocities[1], Vec2::new(2.0, 3.0));
}

#[test]
fn integration_occurs_exactly_once_and_only_after_wall() {
    // Arrange
    let fixture = Fixture::new(1);
    let source = fixture.candidate(
        &[Vec2::new(1.0, 2.0)],
        &[Vec2::new(3.0, -4.0)],
        &[ParticleFlags::WATER],
        &[None],
        &[],
        0,
    );
    let barrier = pass_barrier(&source);
    let collision = pass_collision(&barrier);
    let rigid = mark_rigid_projection(&collision).expect("rigid marker follows collision");
    let wall = wall_candidate(&rigid).expect("wall follows rigid");

    // Act
    let integrated = integrate_candidate(&wall, 0.5).expect("integration follows wall");
    let repeated = integrate_candidate(&integrated, 0.5);
    let early = integrate_candidate(&source, 0.5);

    // Assert
    assert_eq!(integrated.positions, [Vec2::new(2.5, 0.0)]);
    assert_eq!(
        integrated.pass_trace,
        [
            BoundaryPass::Barrier,
            BoundaryPass::Collision,
            BoundaryPass::Rigid,
            BoundaryPass::Wall,
            BoundaryPass::Integrate,
        ]
    );
    assert_eq!(
        repeated,
        Err(BoundarySolverError::ReorderedPass {
            expected: BoundaryStage::AfterWall,
            actual: BoundaryStage::Integrated,
        })
    );
    assert_eq!(
        early,
        Err(BoundarySolverError::ReorderedPass {
            expected: BoundaryStage::AfterWall,
            actual: BoundaryStage::AfterRigidDamping,
        })
    );
}

#[test]
fn mixed_rigid_barrier_interaction_preserves_ids_memberships_and_order() {
    // Arrange
    let fixture = Fixture::new(3);
    let endpoint_group = fixture.group(0, 0..2, ParticleGroupFlags::empty());
    let mut rigid_group = fixture.group(1, 2..3, ParticleGroupFlags::RIGID);
    rigid_group.statistics = GroupStatisticsCache {
        maybe_source_timestamp: Some(1),
        mass: 1.0,
        center: Vec2::ZERO,
        linear_velocity: Vec2::new(0.0, 1.0),
        inertia: 0.0,
        angular_velocity: 0.0,
    };
    let memberships = [
        Some(fixture.group_ids[0]),
        Some(fixture.group_ids[0]),
        Some(fixture.group_ids[1]),
    ];
    let source = fixture.candidate(
        &[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::ZERO],
        &[Vec2::ZERO, Vec2::ZERO, Vec2::new(0.0, 1.0)],
        &[
            ParticleFlags::BARRIER,
            ParticleFlags::BARRIER,
            ParticleFlags::WALL,
        ],
        &memberships,
        &[endpoint_group, rigid_group],
        1,
    );

    // Act
    let barrier = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0,
        1.0,
        3,
    )
    .expect("rigid crossing is handled");
    let collision = pass_collision(&barrier);
    let rigid = mark_rigid_projection(&collision).expect("rigid marker is ordered");
    let wall = wall_candidate(&rigid).expect("wall remains last before integration");

    // Assert
    assert_eq!(wall.particle_ids, source.particle_ids);
    assert_eq!(wall.memberships, source.memberships);
    assert_eq!(
        wall.groups.iter().map(|group| group.id).collect::<Vec<_>>(),
        source
            .groups
            .iter()
            .map(|group| group.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(wall.groups[1].statistics.linear_velocity, Vec2::ZERO);
    assert_eq!(wall.velocities[2], Vec2::ZERO);
}

#[test]
fn resource_limits_are_typed_and_leave_source_candidate_unchanged() {
    // Arrange
    let fixture = Fixture::new(3);
    let group = fixture.group(0, 0..2, ParticleGroupFlags::empty());
    let source = fixture.candidate(
        &[Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::ZERO],
        &[Vec2::ZERO, Vec2::ZERO, Vec2::new(0.0, 1.0)],
        &[
            ParticleFlags::BARRIER,
            ParticleFlags::BARRIER,
            ParticleFlags::WATER,
        ],
        &[Some(fixture.group_ids[0]), Some(fixture.group_ids[0]), None],
        &[group],
        0,
    );
    let snapshot = source.clone();

    // Act
    let scan_overflow = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0,
        1.0,
        2,
    );
    let journal_overflow = barrier_candidate(
        &source,
        &[pair([0, 1], ParticleFlags::BARRIER)],
        1.0,
        1.0,
        1.0,
        3,
    );

    // Assert
    assert_eq!(
        scan_overflow,
        Err(BoundarySolverError::ResourceLimit {
            resource: "barrier particle scans",
            limit: 2,
        })
    );
    assert_eq!(
        journal_overflow,
        Err(BoundarySolverError::ResourceLimit {
            resource: "boundary effect journal",
            limit: 0,
        })
    );
    assert_eq!(source, snapshot);
}

#[test]
fn filtered_hit_limit_rejects_before_any_candidate_effect() {
    // Arrange
    let fixture = Fixture::new(1);
    let source = pass_barrier(&fixture.candidate(
        &[Vec2::ZERO],
        &[Vec2::new(1.0, 0.0)],
        &[ParticleFlags::WATER],
        &[None],
        &[],
        1,
    ));
    let hit = FilteredCollisionHit {
        particle: 0,
        body: fixture.body,
        previous_transform: Transform::IDENTITY,
        current_transform: Transform::IDENTITY,
        body_local_center: Vec2::ZERO,
        is_circle: false,
        fraction: 0.5,
        normal: Vec2::new(1.0, 0.0),
    };
    let snapshot = source.clone();

    // Act
    let result = collision_candidate(&source, &[hit], 0, 1.0, 1.0, 1.0, 0);

    // Assert
    assert_eq!(
        result,
        Err(BoundarySolverError::ResourceLimit {
            resource: "filtered collision hits",
            limit: 0,
        })
    );
    assert_eq!(source, snapshot);
}

#[test]
fn deterministic_repeats_match_exact_candidate_bits() {
    // Arrange
    let fixture = Fixture::new(1);
    let source = fixture.candidate(
        &[Vec2::new(1.0, -2.0)],
        &[Vec2::new(0.25, 0.5)],
        &[ParticleFlags::WATER],
        &[None],
        &[],
        0,
    );
    let run = || {
        let barrier = pass_barrier(&source);
        let collision = pass_collision(&barrier);
        let rigid = mark_rigid_projection(&collision).expect("rigid marker is ordered");
        let wall = wall_candidate(&rigid).expect("wall is ordered");
        integrate_candidate(&wall, 0.125).expect("integration is ordered")
    };
    let expected = run();

    // Act
    let repeated = (0..128).all(|_| run() == expected);

    // Assert
    assert!(repeated);
    assert_eq!(
        bits(&expected.positions),
        vec![[1.03125_f32.to_bits(), (-1.9375_f32).to_bits(),]]
    );
}

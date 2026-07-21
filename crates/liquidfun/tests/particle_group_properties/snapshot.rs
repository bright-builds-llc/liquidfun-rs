use std::collections::HashSet;

use liquidfun::math::Vec2;
use liquidfun::{HandleError, ParticleGroupId, ParticleId};

use super::model::Model;
use super::{MAX_GROUPS, MAX_PARTICLES};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleState {
    position: [u32; 2],
    velocity: [u32; 2],
    flags: u32,
    maybe_group: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GroupState {
    Live {
        flags: u32,
        members: Vec<usize>,
        depths: Option<Vec<u32>>,
    },
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticSnapshot {
    particles: Vec<ParticleState>,
    groups: Vec<GroupState>,
    contacts: Vec<[usize; 2]>,
    pairs: Vec<[usize; 2]>,
    triads: Vec<[usize; 3]>,
}

pub(super) fn semantic_snapshot(model: &Model) -> SemanticSnapshot {
    let view = model
        .world
        .particle_system_view(model.system)
        .expect("model system remains live");
    let ids = view.particle_ids();
    let particle_ordinal = |particle: ParticleId| {
        ids.iter()
            .position(|candidate| *candidate == particle)
            .expect("derived record references a current particle")
    };
    let group_ordinal = |group: ParticleGroupId| {
        model
            .known_groups
            .iter()
            .position(|candidate| *candidate == group)
            .expect("membership references a known group")
    };
    let particles = view
        .positions()
        .iter()
        .copied()
        .zip(view.velocities().iter().copied())
        .zip(view.flags().iter().copied())
        .zip(view.group_ids().iter().copied())
        .map(
            |(((position, velocity), flags), maybe_group)| ParticleState {
                position: bits(position),
                velocity: bits(velocity),
                flags: flags.bits(),
                maybe_group: maybe_group.map(group_ordinal),
            },
        )
        .collect();
    let groups = model
        .known_groups
        .iter()
        .copied()
        .map(|group| match model.world.particle_group_view(group) {
            Ok(group_view) => GroupState::Live {
                flags: group_view.flags().bits(),
                members: group_view
                    .member_ids()
                    .iter()
                    .copied()
                    .map(particle_ordinal)
                    .collect(),
                depths: group_view
                    .maybe_depths()
                    .map(|depths| depths.iter().map(|depth| depth.to_bits()).collect()),
            },
            Err(HandleError::StaleOrDestroyed | HandleError::PendingDelete) => GroupState::Stale,
            Err(error) => panic!("known local group has unexpected error: {error}"),
        })
        .collect();
    SemanticSnapshot {
        particles,
        groups,
        contacts: view
            .particle_contacts()
            .map(|contact| contact.particles().map(particle_ordinal))
            .collect(),
        pairs: view
            .pairs()
            .map(|pair| pair.particles().map(particle_ordinal))
            .collect(),
        triads: view
            .triads()
            .map(|triad| triad.particles().map(particle_ordinal))
            .collect(),
    }
}

pub(super) fn assert_invariants(model: &Model) {
    let view = model
        .world
        .particle_system_view(model.system)
        .expect("model system remains live");
    let ids = view.particle_ids();
    assert!(ids.len() <= MAX_PARTICLES);
    assert_eq!(ids.iter().copied().collect::<HashSet<_>>().len(), ids.len());
    assert_eq!(view.positions().len(), ids.len());
    assert_eq!(view.velocities().len(), ids.len());
    assert_eq!(view.flags().len(), ids.len());
    assert_eq!(view.group_ids().len(), ids.len());
    assert!(view.positions().iter().all(|position| position.is_valid()));
    assert!(view.velocities().iter().all(|velocity| velocity.is_valid()));

    let live = model.live_groups();
    assert_eq!(
        model
            .known_groups
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len(),
        model.known_groups.len()
    );
    assert!(live.len() <= MAX_GROUPS);
    assert_eq!(
        live.iter().copied().collect::<HashSet<_>>().len(),
        live.len()
    );
    assert_eq!(
        model
            .world
            .particle_system_statistics(model.system)
            .expect("model system remains live")
            .group_count(),
        live.len()
    );
    for group in live {
        let group_view = model
            .world
            .particle_group_view(group)
            .expect("live group remains inspectable");
        let members = group_view.member_ids();
        if let Some(first) = members.first() {
            let start = ids
                .iter()
                .position(|particle| particle == first)
                .expect("group member appears in system order");
            assert_eq!(&ids[start..start + members.len()], members);
            assert!(
                view.group_ids()[start..start + members.len()]
                    .iter()
                    .all(|maybe_group| *maybe_group == Some(group))
            );
        }
        assert!(group_view.center().is_valid());
        assert!(group_view.linear_velocity().is_valid());
        assert!(group_view.angular_velocity().is_finite());
        assert!(group_view.mass().is_finite());
        assert!(group_view.inertia().is_finite());
        if let Some(depths) = group_view.maybe_depths() {
            assert_eq!(depths.len(), members.len());
            assert!(depths.iter().all(|depth| depth.is_finite()));
        }
    }
    let current = ids.iter().copied().collect::<HashSet<_>>();
    for contact in view.particle_contacts() {
        assert!(
            contact
                .particles()
                .iter()
                .all(|particle| current.contains(particle))
        );
        assert!(contact.weight().is_finite());
        assert!(contact.normal().is_valid());
    }
    for pair in view.pairs() {
        assert!(
            pair.particles()
                .iter()
                .all(|particle| current.contains(particle))
        );
        assert!(pair.strength().is_finite());
        assert!(pair.distance().is_finite());
    }
    for triad in view.triads() {
        assert!(
            triad
                .particles()
                .iter()
                .all(|particle| current.contains(particle))
        );
        assert!(triad.strength().is_finite());
        assert!(triad.pa().is_valid() && triad.pb().is_valid() && triad.pc().is_valid());
        assert!(triad.ka().is_finite());
        assert!(triad.kb().is_finite());
        assert!(triad.kc().is_finite());
        assert!(triad.s().is_finite());
    }
}

fn bits(vector: Vec2) -> [u32; 2] {
    [vector.x.to_bits(), vector.y.to_bits()]
}

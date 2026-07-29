use std::collections::HashSet;

use liquidfun::math::Vec2;
use liquidfun::{
    HandleError, LifecycleEvent, ParticleGroupId, ParticleId, ParticleSystemSnapshot,
    ParticleSystemStatistics, ParticleSystemView, ParticleWorldStatistics,
};

use super::model::Model;
use super::{MAX_GROUPS, MAX_PARTICLES};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleState {
    position: [u32; 2],
    velocity: [u32; 2],
    flags: u32,
    maybe_group: Option<usize>,
    weight: u32,
    force: [u32; 2],
    color: [u8; 4],
    maybe_snapshot: Option<ParticleSnapshotState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleSnapshotState {
    position: [u32; 2],
    velocity: [u32; 2],
    flags: u32,
    maybe_group: Option<usize>,
    color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GroupState {
    Live {
        flags: u32,
        position: [u32; 2],
        angle: u32,
        center: [u32; 2],
        linear_velocity: [u32; 2],
        angular_velocity: u32,
        mass: u32,
        inertia: u32,
        members: Vec<usize>,
        depths: Option<Vec<u32>>,
    },
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleContactState {
    particles: [usize; 2],
    flags: u32,
    weight: u32,
    normal: [u32; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BodyContactState {
    particle: usize,
    body: String,
    fixture: String,
    weight: u32,
    normal: [u32; 2],
    mass: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PairState {
    particles: [usize; 2],
    flags: u32,
    strength: u32,
    distance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriadState {
    particles: [usize; 3],
    flags: u32,
    strength: u32,
    pa: [u32; 2],
    pb: [u32; 2],
    pc: [u32; 2],
    ka: u32,
    kb: u32,
    kc: u32,
    s: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SystemStatisticsState {
    particle_ids: Vec<usize>,
    pending_particle_count: usize,
    group_count: usize,
    particle_contact_count: usize,
    body_contact_count: usize,
    stuck_candidates: Vec<usize>,
    collision_energy: u32,
    paused: bool,
    declared_capacity: usize,
    effective_capacity: usize,
    configured_maximum: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorldStatisticsState {
    system_count: usize,
    particle_count: usize,
    pending_particle_count: usize,
    group_count: usize,
    particle_contact_count: usize,
    body_contact_count: usize,
    stuck_candidate_count: usize,
    collision_energy: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LifecycleKind {
    Filter,
    Contact,
    ContactDestruction,
    Hook,
    Solve,
    ContinuousSolve,
    JointGoodbye,
    FixtureGoodbye,
    ParticleDestruction,
    ParticleBodyContact,
    ParticleContact,
    Command,
    Destruction,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticSnapshot {
    particles: Vec<ParticleState>,
    groups: Vec<GroupState>,
    contacts: Vec<ParticleContactState>,
    body_contacts: Vec<BodyContactState>,
    pairs: Vec<PairState>,
    triads: Vec<TriadState>,
    maybe_colors: Option<Vec<[u8; 4]>>,
    maybe_expiration_order: Option<Vec<usize>>,
    system: ParticleSystemState,
    system_statistics: SystemStatisticsState,
    world_statistics: WorldStatisticsState,
    lifecycle: Vec<LifecycleKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleSystemState {
    particle_count: usize,
    pending_particle_count: usize,
    paused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RollbackSnapshot {
    semantic: SemanticSnapshot,
    particle_ids: Vec<ParticleId>,
    groups: Vec<(ParticleGroupId, Vec<ParticleId>)>,
}

pub(super) fn semantic_snapshot(model: &Model) -> SemanticSnapshot {
    let view = model
        .world
        .particle_system_view(model.system)
        .expect("model system remains live");
    let ids = view.particle_ids();
    let system = model
        .world
        .particle_system_snapshot(model.system)
        .expect("model system remains live");
    let system_statistics = model
        .world
        .particle_system_statistics(model.system)
        .expect("model system remains live");
    let world_statistics = model.world.particle_world_statistics();
    SemanticSnapshot {
        particles: snapshot_particles(model, &view),
        groups: snapshot_groups(model, ids),
        contacts: view
            .particle_contacts()
            .map(|contact| ParticleContactState {
                particles: contact
                    .particles()
                    .map(|particle| particle_ordinal(ids, particle)),
                flags: contact.flags().bits(),
                weight: contact.weight().to_bits(),
                normal: bits(contact.normal()),
            })
            .collect(),
        body_contacts: view
            .body_contacts()
            .map(|contact| BodyContactState {
                particle: particle_ordinal(ids, contact.particle()),
                body: format!("{:?}", contact.body()),
                fixture: format!("{:?}", contact.fixture()),
                weight: contact.weight().to_bits(),
                normal: bits(contact.normal()),
                mass: contact.mass().to_bits(),
            })
            .collect(),
        pairs: view
            .pairs()
            .map(|pair| PairState {
                particles: pair
                    .particles()
                    .map(|particle| particle_ordinal(ids, particle)),
                flags: pair.flags().bits(),
                strength: pair.strength().to_bits(),
                distance: pair.distance().to_bits(),
            })
            .collect(),
        triads: view
            .triads()
            .map(|triad| TriadState {
                particles: triad
                    .particles()
                    .map(|particle| particle_ordinal(ids, particle)),
                flags: triad.flags().bits(),
                strength: triad.strength().to_bits(),
                pa: bits(triad.pa()),
                pb: bits(triad.pb()),
                pc: bits(triad.pc()),
                ka: triad.ka().to_bits(),
                kb: triad.kb().to_bits(),
                kc: triad.kc().to_bits(),
                s: triad.s().to_bits(),
            })
            .collect(),
        maybe_colors: view.maybe_colors().map(|colors| {
            colors
                .iter()
                .map(|color| color.components())
                .collect::<Vec<_>>()
        }),
        maybe_expiration_order: view.maybe_expiration_order().map(|order| {
            order
                .map(|particle| particle_ordinal(ids, particle))
                .collect::<Vec<_>>()
        }),
        system: particle_system_state(system),
        system_statistics: system_statistics_state(&system_statistics, ids),
        world_statistics: world_statistics_state(world_statistics),
        lifecycle: model.lifecycle.clone(),
    }
}

fn snapshot_particles(model: &Model, view: &ParticleSystemView<'_>) -> Vec<ParticleState> {
    let ids = view.particle_ids();
    view.positions()
        .iter()
        .copied()
        .zip(view.velocities().iter().copied())
        .zip(view.flags().iter().copied())
        .zip(view.group_ids().iter().copied())
        .zip(view.weights().iter().copied())
        .zip(view.forces().iter().copied())
        .enumerate()
        .map(
            |(index, (((((position, velocity), flags), maybe_group), weight), force))| {
                let id = ids[index];
                let maybe_snapshot = match model.world.particle_snapshot(id) {
                    Ok(snapshot) => Some(ParticleSnapshotState {
                        position: bits(snapshot.position()),
                        velocity: bits(snapshot.velocity()),
                        flags: snapshot.flags().bits(),
                        maybe_group: snapshot
                            .maybe_group()
                            .map(|group| group_ordinal(model, group)),
                        color: snapshot.color().components(),
                    }),
                    Err(HandleError::PendingDelete) => None,
                    Err(error) => panic!("view identity has unexpected snapshot error: {error}"),
                };
                ParticleState {
                    position: bits(position),
                    velocity: bits(velocity),
                    flags: flags.bits(),
                    maybe_group: maybe_group.map(|group| group_ordinal(model, group)),
                    weight: weight.to_bits(),
                    force: bits(force),
                    color: view
                        .maybe_colors()
                        .map_or([0, 0, 0, 0], |colors| colors[index].components()),
                    maybe_snapshot,
                }
            },
        )
        .collect()
}

fn snapshot_groups(model: &Model, particle_ids: &[ParticleId]) -> Vec<GroupState> {
    model
        .known_groups
        .iter()
        .copied()
        .map(|group| match model.world.particle_group_view(group) {
            Ok(group_view) => GroupState::Live {
                flags: group_view.flags().bits(),
                position: bits(group_view.position()),
                angle: group_view.angle().to_bits(),
                center: bits(group_view.center()),
                linear_velocity: bits(group_view.linear_velocity()),
                angular_velocity: group_view.angular_velocity().to_bits(),
                mass: group_view.mass().to_bits(),
                inertia: group_view.inertia().to_bits(),
                members: group_view
                    .member_ids()
                    .iter()
                    .copied()
                    .map(|particle| particle_ordinal(particle_ids, particle))
                    .collect(),
                depths: group_view
                    .maybe_depths()
                    .map(|depths| depths.iter().map(|depth| depth.to_bits()).collect()),
            },
            Err(HandleError::StaleOrDestroyed | HandleError::PendingDelete) => GroupState::Stale,
            Err(error) => panic!("known local group has unexpected error: {error}"),
        })
        .collect()
}

fn particle_ordinal(ids: &[ParticleId], particle: ParticleId) -> usize {
    ids.iter()
        .position(|candidate| *candidate == particle)
        .expect("derived record references a current particle")
}

fn group_ordinal(model: &Model, group: ParticleGroupId) -> usize {
    model
        .known_groups
        .iter()
        .position(|candidate| *candidate == group)
        .expect("membership references a known group")
}

pub(super) fn rollback_snapshot(model: &Model) -> RollbackSnapshot {
    let view = model
        .world
        .particle_system_view(model.system)
        .expect("model system remains live");
    RollbackSnapshot {
        semantic: semantic_snapshot(model),
        particle_ids: view.particle_ids().to_vec(),
        groups: model
            .known_groups
            .iter()
            .copied()
            .filter_map(|group| {
                model
                    .world
                    .particle_group_view(group)
                    .ok()
                    .map(|view| (group, view.member_ids().to_vec()))
            })
            .collect(),
    }
}

pub(super) const fn lifecycle_kind(event: &LifecycleEvent) -> LifecycleKind {
    match event {
        LifecycleEvent::Filter(_) => LifecycleKind::Filter,
        LifecycleEvent::Contact(_) => LifecycleKind::Contact,
        LifecycleEvent::ContactDestruction(_) => LifecycleKind::ContactDestruction,
        LifecycleEvent::Hook(_) => LifecycleKind::Hook,
        LifecycleEvent::Solve(_) => LifecycleKind::Solve,
        LifecycleEvent::ContinuousSolve(_) => LifecycleKind::ContinuousSolve,
        LifecycleEvent::JointGoodbye(_) => LifecycleKind::JointGoodbye,
        LifecycleEvent::FixtureGoodbye(_) => LifecycleKind::FixtureGoodbye,
        LifecycleEvent::ParticleDestruction(_) => LifecycleKind::ParticleDestruction,
        LifecycleEvent::ParticleBodyContact(_) => LifecycleKind::ParticleBodyContact,
        LifecycleEvent::ParticleContact(_) => LifecycleKind::ParticleContact,
        LifecycleEvent::Command(_) => LifecycleKind::Command,
        LifecycleEvent::Destruction(_) => LifecycleKind::Destruction,
        _ => LifecycleKind::Other,
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

fn particle_system_state(snapshot: ParticleSystemSnapshot) -> ParticleSystemState {
    ParticleSystemState {
        particle_count: snapshot.particle_count(),
        pending_particle_count: snapshot.pending_particle_count(),
        paused: snapshot.is_paused(),
    }
}

fn system_statistics_state(
    statistics: &ParticleSystemStatistics,
    particle_ids: &[ParticleId],
) -> SystemStatisticsState {
    SystemStatisticsState {
        particle_ids: statistics
            .particle_ids()
            .iter()
            .copied()
            .map(|particle| particle_ordinal(particle_ids, particle))
            .collect(),
        pending_particle_count: statistics.pending_particle_count(),
        group_count: statistics.group_count(),
        particle_contact_count: statistics.particle_contact_count(),
        body_contact_count: statistics.body_contact_count(),
        stuck_candidates: statistics
            .stuck_candidates()
            .iter()
            .copied()
            .map(|particle| particle_ordinal(particle_ids, particle))
            .collect(),
        collision_energy: statistics.collision_energy().to_bits(),
        paused: statistics.is_paused(),
        declared_capacity: statistics.declared_capacity(),
        effective_capacity: statistics.effective_capacity(),
        configured_maximum: statistics.configured_maximum(),
    }
}

fn world_statistics_state(statistics: ParticleWorldStatistics) -> WorldStatisticsState {
    WorldStatisticsState {
        system_count: statistics.system_count(),
        particle_count: statistics.particle_count(),
        pending_particle_count: statistics.pending_particle_count(),
        group_count: statistics.group_count(),
        particle_contact_count: statistics.particle_contact_count(),
        body_contact_count: statistics.body_contact_count(),
        stuck_candidate_count: statistics.stuck_candidate_count(),
        collision_energy: statistics.collision_energy().to_bits(),
    }
}

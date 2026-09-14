#![allow(
    dead_code,
    reason = "the bounded storage spike is executable architecture evidence for later particle work"
)]

use crate::identity::{
    HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::Vec2;
use crate::particle::group::ParticleGroupViewState;
use crate::particle::{
    ParticleBodyContact as SemanticBodyContact, ParticleBufferBundle, ParticleBufferLanes,
    ParticleBufferMode, ParticleColor, ParticleContact as SemanticParticleContact, ParticleFlags,
    ParticleGroupFlags, ParticleGroupView,
};
use std::ops::Range;

use group::GroupRecord;
use lanes::{
    OwnedLaneBundle, ParticleBodyContact, ParticleContact, ParticlePair, ParticleProxy,
    ParticleTriad, StuckLanes, UserAssociationKey,
};
use solver_state::{AggregateGroupFlags, SolverState};
use validation::{
    rebuild_group_records_for_system, validate_groups, validate_reference_sets, validate_references,
};

mod creation;
pub(in crate::particle) mod group;
mod lane_inventory;
pub(in crate::particle) mod lanes;
mod lifecycle;
mod mutation;
pub(in crate::particle) mod permutation;
mod runtime;
mod solver_state;
mod validation;

pub(crate) use mutation::{GroupPlan, GroupPlanError, GroupPlanInput, SplitPlanError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleInput {
    pub(crate) position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) flags: ParticleFlags,
    pub(crate) maybe_group: Option<ParticleGroupId>,
    pub(crate) maybe_color: Option<ParticleColor>,
    pub(crate) maybe_user_association: Option<UserAssociationKey>,
    pub(crate) maybe_expiration_time: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleSnapshot {
    pub(crate) id: ParticleId,
    pub(crate) diagnostic_id: u64,
    pub(crate) input: ParticleInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParticleStorageError {
    WrongWorld,
    WrongParticleSystem,
    StaleOrDestroyed,
    PendingDelete,
    CapacityExceeded { limit: usize },
    IdentityExhausted,
    InvalidPermutation,
    LaneLengthMismatch,
    InvalidDerivedReference,
    InvalidGroupRange,
    InvalidLaneBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) struct ParticleIndex(pub(in crate::particle) usize);

#[derive(Debug, Clone, Copy, PartialEq)]
enum IdentityState {
    Live(ParticleIndex),
    PendingDelete {
        dense: ParticleIndex,
        snapshot: ParticleSnapshot,
    },
    Vacant,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct IdentityEntry {
    generation: u64,
    diagnostic_id: Option<u64>,
    state: IdentityState,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ParticleStorage {
    world: WorldKey,
    system: ParticleSystemId,
    identity_slot_base: usize,
    identity_capacity: usize,
    declared_capacity: usize,
    identities: Vec<IdentityEntry>,
    free_identity_slots: Vec<usize>,
    retired_identity_slots: usize,
    dense_to_id: Vec<ParticleId>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    groups: Vec<Option<ParticleGroupId>>,
    weights: Vec<f32>,
    forces: Vec<Vec2>,
    maybe_colors: Option<Vec<ParticleColor>>,
    maybe_user_associations: Option<Vec<Option<UserAssociationKey>>>,
    maybe_stuck: Option<StuckLanes>,
    maybe_expiration_times: Option<Vec<i32>>,
    maybe_expiration_order: Option<Vec<ParticleIndex>>,
    proxies: Vec<ParticleProxy>,
    particle_contacts: Vec<ParticleContact>,
    body_contacts: Vec<ParticleBodyContact>,
    pairs: Vec<ParticlePair>,
    triads: Vec<ParticleTriad>,
    group_records: Vec<GroupRecord>,
    solver_state: SolverState,
}

struct CreateCandidate {
    input: ParticleInput,
    diagnostic_id: u64,
    id: ParticleId,
    local_slot: usize,
    generation: u64,
    append_identity: bool,
    dense: ParticleIndex,
    group_records: Vec<GroupRecord>,
    solver_state: SolverState,
}

fn push_optional<T: Clone>(
    lane: &mut Option<Vec<T>>,
    maybe_value: Option<T>,
    default: T,
    previous_len: usize,
) {
    match (lane.as_mut(), maybe_value) {
        (Some(values), Some(value)) => values.push(value),
        (Some(values), None) => values.push(default),
        (None, Some(value)) => {
            let mut values = vec![default; previous_len];
            values.push(value);
            *lane = Some(values);
        }
        (None, None) => {}
    }
}

fn push_optional_stuck(maybe_lanes: &mut Option<StuckLanes>) {
    let Some(lanes) = maybe_lanes else {
        return;
    };
    lanes.last_body_contact_steps.push(0);
    lanes.body_contact_counts.push(0);
    lanes.consecutive_contact_steps.push(0);
}

fn push_expiration_order(
    maybe_order: &mut Option<Vec<ParticleIndex>>,
    enable: bool,
    dense: ParticleIndex,
) {
    match (maybe_order.as_mut(), enable) {
        (Some(order), _) => order.push(dense),
        (None, true) => {
            *maybe_order = Some((0..=dense.0).map(ParticleIndex).collect());
        }
        (None, false) => {}
    }
}

#[cfg(test)]
pub(crate) mod identity;

#[cfg(test)]
pub(crate) mod properties;

#[cfg(test)]
mod editor_tests;

#[cfg(test)]
std::thread_local! {
    static DIAGNOSTIC_RECORDS: std::cell::RefCell<Option<Vec<String>>> = const {
        std::cell::RefCell::new(None)
    };
}

#[cfg(test)]
pub(crate) fn diagnostic_enable() {
    DIAGNOSTIC_RECORDS.with(|records| *records.borrow_mut() = Some(Vec::new()));
}

#[cfg(test)]
pub(crate) fn diagnostic_disable() {
    DIAGNOSTIC_RECORDS.with(|records| *records.borrow_mut() = None);
}

#[cfg(test)]
pub(crate) fn diagnostic_record(message: std::fmt::Arguments<'_>) {
    use std::io::Write;

    DIAGNOSTIC_RECORDS.with(|records| {
        let mut records = records.borrow_mut();
        let Some(records) = records.as_mut() else {
            return;
        };
        assert!(records.len() < 2_048, "bounded diagnostic record budget");
        let line = format!("PHASE14 {message}");
        writeln!(std::io::stderr().lock(), "{line}").expect("diagnostic stderr is writable");
        records.push(line);
    });
}

#[cfg(test)]
impl ParticleStorage {
    pub(crate) fn diagnostic_observe(&self, stage: &str) {
        if !DIAGNOSTIC_RECORDS.with(|records| records.borrow().is_some()) {
            return;
        }
        let positions = self
            .positions
            .iter()
            .take(32)
            .map(|v| [v.x.to_bits(), v.y.to_bits()])
            .collect::<Vec<_>>();
        let velocities = self
            .velocities
            .iter()
            .take(32)
            .map(|v| [v.x.to_bits(), v.y.to_bits()])
            .collect::<Vec<_>>();
        let ids = self
            .dense_to_id
            .iter()
            .take(32)
            .map(|id| (id.identity().slot(), id.identity().generation()))
            .collect::<Vec<_>>();
        let lengths = [
            self.dense_to_id.len(),
            self.positions.len(),
            self.velocities.len(),
            self.flags.len(),
            self.groups.len(),
            self.weights.len(),
            self.forces.len(),
        ];
        let optional_lengths = [
            self.maybe_colors.as_ref().map(Vec::len),
            self.maybe_user_associations.as_ref().map(Vec::len),
            self.maybe_expiration_times.as_ref().map(Vec::len),
            self.maybe_expiration_order.as_ref().map(Vec::len),
        ];
        diagnostic_record(format_args!(
            "stage={stage} count={} declared_capacity={} invariant={:?} lengths={lengths:?} optional_lengths={optional_lengths:?} ids={ids:?} position_bits={positions:?} velocity_bits={velocities:?}",
            self.len(),
            self.declared_capacity,
            self.check_invariants()
        ));
        diagnostic_record(format_args!(
            "stage={stage} solver_validation={:?} solver={:?} group_validation={:?} groups={:?} identities={:?} free={:?} retired={}",
            self.solver_state
                .validate(self.len(), &self.flags, &self.group_records),
            self.solver_state,
            validate_groups(self.system, &self.groups, &self.group_records),
            self.group_records,
            self.identities,
            self.free_identity_slots,
            self.retired_identity_slots
        ));
        diagnostic_record(format_args!(
            "stage={stage} contacts={:?} pairs={:?} triads={:?} expiration_order={:?}",
            self.particle_contacts, self.pairs, self.triads, self.maybe_expiration_order
        ));
    }

    pub(super) fn diagnostic_topology(&self, pairs: &[ParticlePair], triads: &[ParticleTriad]) {
        if !DIAGNOSTIC_RECORDS.with(|records| records.borrow().is_some()) {
            return;
        }
        for pair in pairs.iter().take(496) {
            diagnostic_record(format_args!(
                "generated.pair={pair:?} strength_bits={} distance_bits={} validation={:?}",
                pair.strength.to_bits(),
                pair.distance.to_bits(),
                pair.validate(self.len())
            ));
        }
        for triad in triads.iter().take(496) {
            diagnostic_record(format_args!(
                "generated.triad={triad:?} validation={:?}",
                triad.validate(self.len())
            ));
        }
    }
}

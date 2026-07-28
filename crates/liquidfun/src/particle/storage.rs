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

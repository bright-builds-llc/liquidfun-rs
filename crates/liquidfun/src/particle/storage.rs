#![allow(
    dead_code,
    reason = "the bounded storage spike is executable architecture evidence for later particle work"
)]

use crate::identity::{
    HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::Vec2;
use crate::particle::{ParticleColor, ParticleFlags};

use lanes::{
    GroupRange, OwnedLaneBundle, ParticleBodyContact, ParticleContact, ParticlePair, ParticleProxy,
    ParticleTriad, StuckLanes, UserAssociationKey,
};
use validation::{build_group_ranges, validate_reference_sets, validate_references};

mod lane_inventory;
mod lanes;
mod permutation;
mod validation;

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
struct ParticleIndex(usize);

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
    group_ranges: Vec<GroupRange>,
}

struct CreateCandidate {
    input: ParticleInput,
    id: ParticleId,
    local_slot: usize,
    generation: u64,
    append_identity: bool,
    dense: ParticleIndex,
    group_ranges: Vec<GroupRange>,
}

impl ParticleStorage {
    pub(crate) fn new(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        Self::from_owned_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            OwnedLaneBundle::with_capacity(declared_capacity, false),
        )
    }

    pub(crate) fn from_owned_lanes(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        lanes: OwnedLaneBundle,
    ) -> Result<Self, ParticleStorageError> {
        if system.identity().world() != world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity_slot_base.checked_add(identity_capacity).is_none() {
            return Err(ParticleStorageError::IdentityExhausted);
        }
        lanes.validate_empty(declared_capacity)?;

        Ok(Self {
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            identities: Vec::new(),
            free_identity_slots: Vec::new(),
            retired_identity_slots: 0,
            dense_to_id: Vec::with_capacity(declared_capacity),
            positions: lanes.positions,
            velocities: lanes.velocities,
            flags: lanes.flags,
            groups: lanes.groups,
            weights: lanes.weights,
            forces: lanes.forces,
            maybe_colors: lanes.maybe_colors,
            maybe_user_associations: lanes.maybe_user_associations,
            maybe_stuck: lanes.maybe_stuck,
            maybe_expiration_times: lanes.maybe_expiration_times,
            maybe_expiration_order: lanes.maybe_expiration_order,
            proxies: Vec::with_capacity(declared_capacity),
            particle_contacts: Vec::new(),
            body_contacts: Vec::new(),
            pairs: Vec::new(),
            triads: Vec::new(),
            group_ranges: Vec::new(),
        })
    }

    pub(crate) fn positions(&self) -> &[Vec2] {
        &self.positions
    }

    pub(crate) fn into_owned_lanes(self) -> OwnedLaneBundle {
        OwnedLaneBundle {
            positions: self.positions,
            velocities: self.velocities,
            flags: self.flags,
            groups: self.groups,
            weights: self.weights,
            forces: self.forces,
            maybe_colors: self.maybe_colors,
            maybe_user_associations: self.maybe_user_associations,
            maybe_stuck: self.maybe_stuck,
            maybe_expiration_times: self.maybe_expiration_times,
            maybe_expiration_order: self.maybe_expiration_order,
        }
    }

    pub(crate) fn create(
        &mut self,
        input: ParticleInput,
    ) -> Result<ParticleId, ParticleStorageError> {
        let candidate = self.prepare_create(input)?;
        Ok(self.commit_create(candidate))
    }

    fn prepare_create(
        &self,
        input: ParticleInput,
    ) -> Result<CreateCandidate, ParticleStorageError> {
        if self.dense_to_id.len() >= self.declared_capacity {
            return Err(ParticleStorageError::CapacityExceeded {
                limit: self.declared_capacity,
            });
        }
        self.validate_appended_group(input.maybe_group)?;
        let (local_slot, generation, append_identity) = self.identity_slot_candidate()?;
        let particle_slot = self
            .identity_slot_base
            .checked_add(local_slot)
            .ok_or(ParticleStorageError::IdentityExhausted)?;
        let id = ParticleId::from_identity(Identity::new_particle(
            self.world,
            particle_slot,
            generation,
            self.system.identity(),
        ));
        let dense = ParticleIndex(self.dense_to_id.len());
        let mut groups = self.groups.clone();
        groups.push(input.maybe_group);
        let group_ranges = build_group_ranges(&groups)?;
        Ok(CreateCandidate {
            input,
            id,
            local_slot,
            generation,
            append_identity,
            dense,
            group_ranges,
        })
    }

    fn commit_create(&mut self, candidate: CreateCandidate) -> ParticleId {
        if candidate.append_identity {
            self.identities.push(IdentityEntry {
                generation: candidate.generation,
                state: IdentityState::Vacant,
            });
        } else {
            let reused = self
                .free_identity_slots
                .pop()
                .expect("prepared reused identity remains available until commit");
            debug_assert_eq!(reused, candidate.local_slot);
        }
        self.identities[candidate.local_slot].state = IdentityState::Live(candidate.dense);
        self.push_row(candidate.id, candidate.input);
        self.group_ranges = candidate.group_ranges;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        candidate.id
    }

    fn identity_slot_candidate(&self) -> Result<(usize, u64, bool), ParticleStorageError> {
        if let Some(local_slot) = self.free_identity_slots.last().copied() {
            let entry = self
                .identities
                .get(local_slot)
                .expect("free identity slots always refer to existing entries");
            debug_assert_eq!(entry.state, IdentityState::Vacant);
            return Ok((local_slot, entry.generation, false));
        }
        if self.identities.len() >= self.identity_capacity {
            if self.retired_identity_slots > 0 {
                return Err(ParticleStorageError::IdentityExhausted);
            }
            return Err(ParticleStorageError::CapacityExceeded {
                limit: self.identity_capacity,
            });
        }

        let local_slot = self.identities.len();
        Ok((local_slot, 0, true))
    }

    fn push_row(&mut self, id: ParticleId, input: ParticleInput) {
        let previous_len = self.dense_to_id.len();
        self.dense_to_id.push(id);
        self.positions.push(input.position);
        self.velocities.push(input.velocity);
        self.flags.push(input.flags);
        self.groups.push(input.maybe_group);
        self.weights.push(0.0);
        self.forces.push(Vec2::ZERO);
        let dense = ParticleIndex(previous_len);
        self.proxies.push(ParticleProxy::new(dense));
        push_optional(
            &mut self.maybe_colors,
            input.maybe_color,
            ParticleColor::ZERO,
            previous_len,
        );
        push_optional(
            &mut self.maybe_user_associations,
            input.maybe_user_association.map(Some),
            None,
            previous_len,
        );
        push_optional_stuck(&mut self.maybe_stuck);
        push_optional(
            &mut self.maybe_expiration_times,
            input.maybe_expiration_time,
            0,
            previous_len,
        );
        push_expiration_order(
            &mut self.maybe_expiration_order,
            input.maybe_expiration_time.is_some(),
            dense,
        );
    }

    pub(crate) fn input(&self, id: ParticleId) -> Result<ParticleInput, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        Ok(self.input_at(dense))
    }

    pub(crate) fn set_position(
        &mut self,
        id: ParticleId,
        position: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.positions[dense.0] = position;
        Ok(())
    }

    pub(crate) fn mark_delete(
        &mut self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let snapshot = ParticleSnapshot {
            id,
            input: self.input_at(dense),
        };
        let local_slot = self.local_slot(id)?;
        self.identities[local_slot].state = IdentityState::PendingDelete { dense, snapshot };
        Ok(snapshot)
    }

    pub(crate) fn compact_pending(
        &mut self,
    ) -> Result<Vec<ParticleSnapshot>, ParticleStorageError> {
        let old_to_new: Vec<Option<usize>> = self
            .dense_to_id
            .iter()
            .scan(0_usize, |next, id| {
                let local_slot = self
                    .local_slot(*id)
                    .expect("dense identities are always locally scoped");
                let keep = matches!(self.identities[local_slot].state, IdentityState::Live(_));
                let mapped = keep.then(|| {
                    let destination = *next;
                    *next += 1;
                    destination
                });
                Some(mapped)
            })
            .collect();
        permutation::apply_permutation(self, &old_to_new)
    }

    pub(crate) fn rotate_rows(
        &mut self,
        start: usize,
        middle: usize,
        end: usize,
    ) -> Result<(), ParticleStorageError> {
        if start > middle || middle > end || end > self.dense_to_id.len() {
            return Err(ParticleStorageError::InvalidPermutation);
        }
        let mut old_to_new: Vec<_> = (0..self.dense_to_id.len()).map(Some).collect();
        for (old, destination) in old_to_new.iter_mut().enumerate().take(middle).skip(start) {
            *destination = Some(old + end - middle);
        }
        for (old, destination) in old_to_new.iter_mut().enumerate().take(end).skip(middle) {
            *destination = Some(old + start - middle);
        }
        permutation::apply_permutation(self, &old_to_new).map(|_destroyed| ())
    }

    fn resolve_live(&self, id: ParticleId) -> Result<ParticleIndex, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::Live(dense) => Ok(dense),
            IdentityState::PendingDelete { .. } => Err(ParticleStorageError::PendingDelete),
            IdentityState::Vacant | IdentityState::Retired => {
                Err(ParticleStorageError::StaleOrDestroyed)
            }
        }
    }

    fn local_slot(&self, id: ParticleId) -> Result<usize, ParticleStorageError> {
        let identity = id.identity();
        if identity.world() != self.world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity.maybe_particle_system() != Some(self.system.identity().scope()) {
            return Err(ParticleStorageError::WrongParticleSystem);
        }
        let Some(local_slot) = identity.slot().checked_sub(self.identity_slot_base) else {
            return Err(ParticleStorageError::WrongParticleSystem);
        };
        if local_slot >= self.identity_capacity {
            return Err(ParticleStorageError::WrongParticleSystem);
        }
        Ok(local_slot)
    }

    fn input_at(&self, dense: ParticleIndex) -> ParticleInput {
        ParticleInput {
            position: self.positions[dense.0],
            velocity: self.velocities[dense.0],
            flags: self.flags[dense.0],
            maybe_group: self.groups[dense.0],
            maybe_color: self.maybe_colors.as_ref().map(|lane| lane[dense.0]),
            maybe_user_association: self
                .maybe_user_associations
                .as_ref()
                .and_then(|lane| lane[dense.0]),
            maybe_expiration_time: self
                .maybe_expiration_times
                .as_ref()
                .map(|lane| lane[dense.0]),
        }
    }

    fn validate_appended_group(
        &self,
        maybe_group: Option<ParticleGroupId>,
    ) -> Result<(), ParticleStorageError> {
        let Some(group) = maybe_group else {
            return Ok(());
        };
        let Some(last) = self.groups.last() else {
            return Ok(());
        };
        if *last == Some(group) || !self.groups.contains(&Some(group)) {
            return Ok(());
        }
        Err(ParticleStorageError::InvalidGroupRange)
    }

    fn check_invariants(&self) -> Result<(), ParticleStorageError> {
        let count = self.dense_to_id.len();
        self.check_lane_lengths(count)?;
        self.check_identity_map()?;
        self.check_derived_references(count)?;
        if self.group_ranges != build_group_ranges(&self.groups)? {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        Ok(())
    }

    fn check_lane_lengths(&self, count: usize) -> Result<(), ParticleStorageError> {
        if self.positions.len() != count
            || self.velocities.len() != count
            || self.flags.len() != count
            || self.groups.len() != count
            || self.weights.len() != count
            || self.forces.len() != count
            || self
                .maybe_colors
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self
                .maybe_user_associations
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self.maybe_stuck.as_ref().is_some_and(|lanes| {
                lanes.last_body_contact_steps.len() != count
                    || lanes.body_contact_counts.len() != count
                    || lanes.consecutive_contact_steps.len() != count
            })
            || self
                .maybe_expiration_times
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self
                .maybe_expiration_order
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
        {
            return Err(ParticleStorageError::LaneLengthMismatch);
        }
        Ok(())
    }

    fn check_identity_map(&self) -> Result<(), ParticleStorageError> {
        for (dense, id) in self.dense_to_id.iter().copied().enumerate() {
            let local_slot = self.local_slot(id)?;
            let entry = self
                .identities
                .get(local_slot)
                .ok_or(ParticleStorageError::StaleOrDestroyed)?;
            if entry.generation != id.identity().generation()
                || !matches!(
                    entry.state,
                    IdentityState::Live(ParticleIndex(index))
                        | IdentityState::PendingDelete {
                            dense: ParticleIndex(index),
                            ..
                        } if index == dense
                )
            {
                return Err(ParticleStorageError::StaleOrDestroyed);
            }
            if self.dense_to_id[..dense].contains(&id) {
                return Err(ParticleStorageError::StaleOrDestroyed);
            }
        }
        Ok(())
    }

    fn check_derived_references(&self, count: usize) -> Result<(), ParticleStorageError> {
        validate_references(
            &self
                .proxies
                .iter()
                .map(|proxy| proxy.index)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .particle_contacts
                .iter()
                .map(|contact| contact.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_references(
            &self
                .body_contacts
                .iter()
                .map(|contact| contact.index)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .pairs
                .iter()
                .map(|pair| pair.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .triads
                .iter()
                .map(|triad| triad.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        if let Some(order) = &self.maybe_expiration_order {
            validate_references(order, count)?;
        }
        if let Some(stuck) = &self.maybe_stuck {
            validate_references(&stuck.candidates, count)?;
        }
        Ok(())
    }
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

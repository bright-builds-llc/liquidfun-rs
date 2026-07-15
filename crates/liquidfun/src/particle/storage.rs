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

mod lane_inventory;
mod lanes;

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

struct PermutationCandidate {
    identities: Vec<IdentityEntry>,
    freed_slots: Vec<usize>,
    dense_to_id: Vec<Option<ParticleId>>,
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
    destroyed: Vec<ParticleSnapshot>,
}

struct DerivedPermutation {
    proxies: Vec<ParticleProxy>,
    particle_contacts: Vec<ParticleContact>,
    body_contacts: Vec<ParticleBodyContact>,
    pairs: Vec<ParticlePair>,
    triads: Vec<ParticleTriad>,
    maybe_expiration_order: Option<Vec<ParticleIndex>>,
}

struct RowPermutationCandidate {
    identities: Vec<IdentityEntry>,
    freed_slots: Vec<usize>,
    dense_to_id: Vec<Option<ParticleId>>,
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
    destroyed: Vec<ParticleSnapshot>,
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
        self.apply_permutation(&old_to_new)
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
        self.apply_permutation(&old_to_new).map(|_destroyed| ())
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

    fn apply_permutation(
        &mut self,
        old_to_new: &[Option<usize>],
    ) -> Result<Vec<ParticleSnapshot>, ParticleStorageError> {
        self.check_invariants()?;
        let old_count = self.dense_to_id.len();
        let new_count = validate_basic_permutation(old_to_new, old_count)?;
        let derived = self.remap_derived(old_to_new)?;
        let mut rows = self.empty_row_permutation(new_count);

        for (old, maybe_new) in old_to_new.iter().copied().enumerate() {
            let id = self.dense_to_id[old];
            let local_slot = self.local_slot(id)?;
            let entry = &mut rows.identities[local_slot];
            if let Some(new) = maybe_new {
                rows.dense_to_id[new] = Some(id);
                rows.positions[new] = self.positions[old];
                rows.velocities[new] = self.velocities[old];
                rows.flags[new] = self.flags[old];
                rows.groups[new] = self.groups[old];
                rows.forces[new] = self.forces[old];
                copy_optional(
                    self.maybe_colors.as_deref(),
                    rows.maybe_colors.as_deref_mut(),
                    old,
                    new,
                );
                copy_optional(
                    self.maybe_user_associations.as_deref(),
                    rows.maybe_user_associations.as_deref_mut(),
                    old,
                    new,
                );
                copy_stuck(
                    self.maybe_stuck.as_ref(),
                    rows.maybe_stuck.as_mut(),
                    old,
                    new,
                );
                copy_optional(
                    self.maybe_expiration_times.as_deref(),
                    rows.maybe_expiration_times.as_deref_mut(),
                    old,
                    new,
                );
                entry.state = match entry.state {
                    IdentityState::Live(_) => IdentityState::Live(ParticleIndex(new)),
                    IdentityState::PendingDelete { snapshot, .. } => IdentityState::PendingDelete {
                        dense: ParticleIndex(new),
                        snapshot,
                    },
                    IdentityState::Vacant | IdentityState::Retired => {
                        return Err(ParticleStorageError::InvalidPermutation);
                    }
                };
                continue;
            }

            let IdentityState::PendingDelete { snapshot, .. } = entry.state else {
                return Err(ParticleStorageError::InvalidPermutation);
            };
            rows.destroyed.push(snapshot);
            let Some(next_generation) = entry.generation.checked_add(1) else {
                entry.state = IdentityState::Retired;
                continue;
            };
            entry.generation = next_generation;
            entry.state = IdentityState::Vacant;
            rows.freed_slots.push(local_slot);
        }

        let group_ranges = build_group_ranges(&rows.groups)?;
        let candidate = PermutationCandidate {
            identities: rows.identities,
            freed_slots: rows.freed_slots,
            dense_to_id: rows.dense_to_id,
            positions: rows.positions,
            velocities: rows.velocities,
            flags: rows.flags,
            groups: rows.groups,
            weights: rows.weights,
            forces: rows.forces,
            maybe_colors: rows.maybe_colors,
            maybe_user_associations: rows.maybe_user_associations,
            maybe_stuck: rows.maybe_stuck,
            maybe_expiration_times: rows.maybe_expiration_times,
            maybe_expiration_order: derived.maybe_expiration_order,
            proxies: derived.proxies,
            particle_contacts: derived.particle_contacts,
            body_contacts: derived.body_contacts,
            pairs: derived.pairs,
            triads: derived.triads,
            group_ranges,
            destroyed: rows.destroyed,
        };
        Ok(self.commit_permutation(candidate))
    }

    fn empty_row_permutation(&self, new_count: usize) -> RowPermutationCandidate {
        RowPermutationCandidate {
            identities: self.identities.clone(),
            freed_slots: Vec::new(),
            dense_to_id: vec![None; new_count],
            positions: vec![Vec2::ZERO; new_count],
            velocities: vec![Vec2::ZERO; new_count],
            flags: vec![ParticleFlags::WATER; new_count],
            groups: vec![None; new_count],
            weights: vec![0.0; new_count],
            forces: vec![Vec2::ZERO; new_count],
            maybe_colors: self
                .maybe_colors
                .as_ref()
                .map(|_| vec![ParticleColor::ZERO; new_count]),
            maybe_user_associations: self
                .maybe_user_associations
                .as_ref()
                .map(|_| vec![None; new_count]),
            maybe_stuck: self.maybe_stuck.as_ref().map(|_| StuckLanes {
                last_body_contact_steps: vec![0; new_count],
                body_contact_counts: vec![0; new_count],
                consecutive_contact_steps: vec![0; new_count],
                candidates: Vec::new(),
            }),
            maybe_expiration_times: self
                .maybe_expiration_times
                .as_ref()
                .map(|_| vec![0; new_count]),
            destroyed: Vec::new(),
        }
    }

    fn remap_derived(
        &self,
        old_to_new: &[Option<usize>],
    ) -> Result<DerivedPermutation, ParticleStorageError> {
        Ok(DerivedPermutation {
            proxies: remap_proxies(&self.proxies, old_to_new)?,
            particle_contacts: remap_particle_contacts(&self.particle_contacts, old_to_new)?,
            body_contacts: remap_body_contacts(&self.body_contacts, old_to_new)?,
            pairs: remap_pairs(&self.pairs, old_to_new)?,
            triads: remap_triads(&self.triads, old_to_new)?,
            maybe_expiration_order: self
                .maybe_expiration_order
                .as_ref()
                .map(|order| remap_references(order, old_to_new))
                .transpose()?,
        })
    }

    fn commit_permutation(&mut self, candidate: PermutationCandidate) -> Vec<ParticleSnapshot> {
        self.identities = candidate.identities;
        self.free_identity_slots.extend(candidate.freed_slots);
        self.retired_identity_slots = self
            .identities
            .iter()
            .filter(|entry| entry.state == IdentityState::Retired)
            .count();
        self.dense_to_id = candidate
            .dense_to_id
            .into_iter()
            .map(|maybe_id| maybe_id.expect("validated permutations fill every destination"))
            .collect();
        self.positions = candidate.positions;
        self.velocities = candidate.velocities;
        self.flags = candidate.flags;
        self.groups = candidate.groups;
        self.weights = candidate.weights;
        self.forces = candidate.forces;
        self.maybe_colors = candidate.maybe_colors;
        self.maybe_user_associations = candidate.maybe_user_associations;
        self.maybe_stuck = candidate.maybe_stuck;
        self.maybe_expiration_times = candidate.maybe_expiration_times;
        self.maybe_expiration_order = candidate.maybe_expiration_order;
        self.proxies = candidate.proxies;
        self.particle_contacts = candidate.particle_contacts;
        self.body_contacts = candidate.body_contacts;
        self.pairs = candidate.pairs;
        self.triads = candidate.triads;
        self.group_ranges = candidate.group_ranges;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        candidate.destroyed
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

fn copy_optional<T: Copy>(
    source: Option<&[T]>,
    destination: Option<&mut [T]>,
    old: usize,
    new: usize,
) {
    if let (Some(source), Some(destination)) = (source, destination) {
        destination[new] = source[old];
    }
}

fn copy_stuck(
    source: Option<&StuckLanes>,
    destination: Option<&mut StuckLanes>,
    old: usize,
    new: usize,
) {
    let (Some(source), Some(destination)) = (source, destination) else {
        return;
    };
    destination.last_body_contact_steps[new] = source.last_body_contact_steps[old];
    destination.body_contact_counts[new] = source.body_contact_counts[old];
    destination.consecutive_contact_steps[new] = source.consecutive_contact_steps[old];
}

fn validate_basic_permutation(
    old_to_new: &[Option<usize>],
    old_count: usize,
) -> Result<usize, ParticleStorageError> {
    if old_to_new.len() != old_count {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    let new_count = old_to_new.iter().filter(|value| value.is_some()).count();
    let mut seen = vec![false; new_count];
    for destination in old_to_new.iter().flatten().copied() {
        let Some(was_seen) = seen.get_mut(destination) else {
            return Err(ParticleStorageError::InvalidPermutation);
        };
        if *was_seen {
            return Err(ParticleStorageError::InvalidPermutation);
        }
        *was_seen = true;
    }
    if seen.iter().any(|was_seen| !was_seen) {
        return Err(ParticleStorageError::InvalidPermutation);
    }
    Ok(new_count)
}

fn validate_references(
    references: &[ParticleIndex],
    count: usize,
) -> Result<(), ParticleStorageError> {
    if references.iter().any(|index| index.0 >= count) {
        return Err(ParticleStorageError::InvalidDerivedReference);
    }
    Ok(())
}

fn validate_reference_sets<const N: usize>(
    references: &[[ParticleIndex; N]],
    count: usize,
) -> Result<(), ParticleStorageError> {
    if references.iter().flatten().any(|index| index.0 >= count) {
        return Err(ParticleStorageError::InvalidDerivedReference);
    }
    Ok(())
}

fn remap_references(
    references: &[ParticleIndex],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleIndex>, ParticleStorageError> {
    references
        .iter()
        .filter_map(|index| match old_to_new.get(index.0) {
            Some(Some(destination)) => Some(Ok(ParticleIndex(*destination))),
            Some(None) => None,
            None => Some(Err(ParticleStorageError::InvalidDerivedReference)),
        })
        .collect()
}

fn remap_reference_sets<const N: usize>(
    references: &[[ParticleIndex; N]],
    old_to_new: &[Option<usize>],
) -> Result<Vec<[ParticleIndex; N]>, ParticleStorageError> {
    let mut remapped = Vec::with_capacity(references.len());
    for reference in references {
        let mut mapped = [ParticleIndex(0); N];
        let mut removed = false;
        for (destination, old) in mapped.iter_mut().zip(reference) {
            match old_to_new.get(old.0) {
                Some(Some(new)) => *destination = ParticleIndex(*new),
                Some(None) => removed = true,
                None => return Err(ParticleStorageError::InvalidDerivedReference),
            }
        }
        if !removed {
            remapped.push(mapped);
        }
    }
    Ok(remapped)
}

fn remap_proxies(
    proxies: &[ParticleProxy],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleProxy>, ParticleStorageError> {
    proxies
        .iter()
        .filter_map(|proxy| match remap_index(proxy.index, old_to_new) {
            Ok(Some(index)) => Some(Ok(ParticleProxy { index, ..*proxy })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn remap_particle_contacts(
    contacts: &[ParticleContact],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleContact>, ParticleStorageError> {
    remap_records(
        contacts,
        old_to_new,
        |contact| contact.indices,
        |contact, indices| ParticleContact { indices, ..contact },
    )
}

fn remap_body_contacts(
    contacts: &[ParticleBodyContact],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleBodyContact>, ParticleStorageError> {
    contacts
        .iter()
        .filter_map(|contact| match remap_index(contact.index, old_to_new) {
            Ok(Some(index)) => Some(Ok(ParticleBodyContact { index, ..*contact })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn remap_pairs(
    pairs: &[ParticlePair],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticlePair>, ParticleStorageError> {
    remap_records(
        pairs,
        old_to_new,
        |pair| pair.indices,
        |pair, indices| ParticlePair { indices, ..pair },
    )
}

fn remap_triads(
    triads: &[ParticleTriad],
    old_to_new: &[Option<usize>],
) -> Result<Vec<ParticleTriad>, ParticleStorageError> {
    remap_records(
        triads,
        old_to_new,
        |triad| triad.indices,
        |triad, indices| ParticleTriad { indices, ..triad },
    )
}

fn remap_records<T: Copy, const N: usize>(
    records: &[T],
    old_to_new: &[Option<usize>],
    indices: impl Fn(T) -> [ParticleIndex; N],
    rebuild: impl Fn(T, [ParticleIndex; N]) -> T,
) -> Result<Vec<T>, ParticleStorageError> {
    let mut remapped = Vec::with_capacity(records.len());
    for record in records.iter().copied() {
        let old_indices = indices(record);
        let mut new_indices = [ParticleIndex(0); N];
        let mut removed = false;
        for (destination, old) in new_indices.iter_mut().zip(old_indices) {
            match remap_index(old, old_to_new)? {
                Some(new) => *destination = new,
                None => removed = true,
            }
        }
        if !removed {
            remapped.push(rebuild(record, new_indices));
        }
    }
    Ok(remapped)
}

fn remap_index(
    old: ParticleIndex,
    old_to_new: &[Option<usize>],
) -> Result<Option<ParticleIndex>, ParticleStorageError> {
    old_to_new
        .get(old.0)
        .copied()
        .map(|maybe_new| maybe_new.map(ParticleIndex))
        .ok_or(ParticleStorageError::InvalidDerivedReference)
}

fn build_group_ranges(
    groups: &[Option<ParticleGroupId>],
) -> Result<Vec<GroupRange>, ParticleStorageError> {
    let mut ranges: Vec<GroupRange> = Vec::new();
    for (dense, maybe_group) in groups.iter().copied().enumerate() {
        let Some(group) = maybe_group else {
            continue;
        };
        if let Some(last) = ranges.last_mut()
            && last.maybe_group == Some(group)
            && last.end.0 == dense
        {
            last.end = ParticleIndex(dense + 1);
            continue;
        }
        if ranges.iter().any(|range| range.maybe_group == Some(group)) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        ranges.push(GroupRange {
            maybe_group: Some(group),
            start: ParticleIndex(dense),
            end: ParticleIndex(dense + 1),
        });
    }
    Ok(ranges)
}

#[cfg(test)]
pub(crate) mod identity;

#[cfg(test)]
pub(crate) mod permutation;

#[cfg(test)]
pub(crate) mod properties;

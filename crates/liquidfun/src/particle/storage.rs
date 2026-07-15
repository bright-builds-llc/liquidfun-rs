#![allow(
    dead_code,
    reason = "the bounded storage spike is executable architecture evidence for later particle work"
)]

use crate::identity::{
    HandleIdentity, Identity, ParticleGroupId, ParticleId, ParticleSystemId, WorldKey,
};
use crate::math::Vec2;
use crate::particle::{
    ParticleBodyContact as SemanticBodyContact, ParticleBufferBundle, ParticleBufferLanes,
    ParticleBufferMode, ParticleColor, ParticleContact as SemanticParticleContact, ParticleFlags,
};
use std::ops::Range;

use lanes::{
    GroupRange, OwnedLaneBundle, ParticleBodyContact, ParticleContact, ParticlePair, ParticleProxy,
    ParticleTriad, StuckLanes, UserAssociationKey,
};
use validation::{build_group_ranges, validate_reference_sets, validate_references};

mod lane_inventory;
pub(in crate::particle) mod lanes;
pub(in crate::particle) mod permutation;
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
    group_ranges: Vec<GroupRange>,
}

struct CreateCandidate {
    input: ParticleInput,
    diagnostic_id: u64,
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
        Self::with_initial_capacity(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            declared_capacity,
        )
    }

    pub(crate) fn with_initial_capacity(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        initial_capacity: usize,
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        if initial_capacity > declared_capacity {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let lanes = OwnedLaneBundle::with_capacity(initial_capacity, false);
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            initial_capacity,
        )
    }

    pub(crate) fn from_buffer_bundle(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        bundle: ParticleBufferBundle,
    ) -> Self {
        let (mode, supplied) = bundle.into_parts();
        let initial_capacity = mode.declared_count();
        let lanes = OwnedLaneBundle {
            positions: supplied.positions,
            velocities: supplied.velocities,
            flags: supplied.flags,
            groups: Vec::with_capacity(initial_capacity),
            weights: Vec::with_capacity(initial_capacity),
            forces: Vec::with_capacity(initial_capacity),
            maybe_colors: supplied.maybe_colors,
            maybe_user_associations: None,
            maybe_stuck: None,
            maybe_expiration_times: None,
            maybe_expiration_order: None,
        };
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            initial_capacity,
        )
        .expect("validated owned lanes and world-scoped system form valid storage")
    }

    fn from_validated_lanes(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        lanes: OwnedLaneBundle,
        minimum_lane_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        if system.identity().world() != world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity_slot_base.checked_add(identity_capacity).is_none() {
            return Err(ParticleStorageError::IdentityExhausted);
        }
        lanes.validate_empty(minimum_lane_capacity)?;

        Ok(Self {
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            identities: Vec::new(),
            free_identity_slots: Vec::new(),
            retired_identity_slots: 0,
            dense_to_id: Vec::with_capacity(minimum_lane_capacity),
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
            proxies: Vec::with_capacity(minimum_lane_capacity),
            particle_contacts: Vec::new(),
            body_contacts: Vec::new(),
            pairs: Vec::new(),
            triads: Vec::new(),
            group_ranges: Vec::new(),
        })
    }

    pub(crate) fn system(&self) -> ParticleSystemId {
        self.system
    }

    pub(crate) fn len(&self) -> usize {
        self.dense_to_id.len()
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.identities
            .iter()
            .filter(|entry| matches!(entry.state, IdentityState::PendingDelete { .. }))
            .count()
    }

    pub(crate) fn snapshots(&self) -> Vec<ParticleSnapshot> {
        self.dense_to_id
            .iter()
            .copied()
            .map(|id| self.snapshot_for_teardown(id))
            .collect()
    }

    pub(crate) fn particle_ids(&self) -> &[ParticleId] {
        &self.dense_to_id
    }

    pub(crate) fn clear_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<Vec<ParticleId>, ParticleStorageError> {
        self.check_invariants()?;
        let particles = self
            .dense_to_id
            .iter()
            .copied()
            .zip(self.groups.iter().copied())
            .filter_map(|(id, maybe_group)| (maybe_group == Some(group)).then_some(id))
            .collect::<Vec<_>>();
        let mut groups = self.groups.clone();
        for maybe_group in &mut groups {
            if *maybe_group == Some(group) {
                *maybe_group = None;
            }
        }
        let ranges = build_group_ranges(&groups)?;
        self.groups = groups;
        self.group_ranges = ranges;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(particles)
    }

    pub(crate) fn snapshot(
        &self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let local_slot = self.local_slot(id)?;
        Ok(ParticleSnapshot {
            id,
            diagnostic_id: self.identities[local_slot]
                .diagnostic_id
                .expect("live particles always retain a diagnostic identity"),
            input: self.input_at(dense),
        })
    }

    fn snapshot_for_teardown(&self, id: ParticleId) -> ParticleSnapshot {
        let local_slot = self
            .local_slot(id)
            .expect("dense identities remain scoped to their owning storage");
        let entry = &self.identities[local_slot];
        match entry.state {
            IdentityState::Live(dense) => ParticleSnapshot {
                id,
                diagnostic_id: entry
                    .diagnostic_id
                    .expect("live particles always retain a diagnostic identity"),
                input: self.input_at(dense),
            },
            IdentityState::PendingDelete { snapshot, .. } => snapshot,
            IdentityState::Vacant | IdentityState::Retired => {
                unreachable!("dense rows cannot refer to vacant or retired identities")
            }
        }
    }

    pub(crate) fn create_with_diagnostic(
        &mut self,
        input: ParticleInput,
        diagnostic_id: u64,
    ) -> Result<ParticleId, ParticleStorageError> {
        let candidate = self.prepare_create(input, diagnostic_id)?;
        Ok(self.commit_create(candidate))
    }

    pub(crate) fn validate_create(&self, input: ParticleInput) -> Result<(), ParticleStorageError> {
        self.prepare_create(input, 0).map(|_candidate| ())
    }

    pub(crate) fn create(
        &mut self,
        input: ParticleInput,
    ) -> Result<ParticleId, ParticleStorageError> {
        self.create_with_diagnostic(input, 0)
    }

    fn prepare_create(
        &self,
        input: ParticleInput,
        diagnostic_id: u64,
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
            diagnostic_id,
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
                diagnostic_id: None,
                state: IdentityState::Vacant,
            });
        } else {
            let reused = self
                .free_identity_slots
                .pop()
                .expect("prepared reused identity remains available until commit");
            debug_assert_eq!(reused, candidate.local_slot);
        }
        self.identities[candidate.local_slot].diagnostic_id = Some(candidate.diagnostic_id);
        self.identities[candidate.local_slot].state = IdentityState::Live(candidate.dense);
        self.push_row(candidate.id, candidate.input);
        self.group_ranges = candidate.group_ranges;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        candidate.id
    }

    /*
        Self::from_owned_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            OwnedLaneBundle::with_capacity(declared_capacity, false),
        )
    */

    pub(crate) fn from_owned_lanes(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        lanes: OwnedLaneBundle,
    ) -> Result<Self, ParticleStorageError> {
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            declared_capacity,
        )
    }

    pub(crate) fn positions(&self) -> &[Vec2] {
        &self.positions
    }

    pub(in crate::particle) fn velocities(&self) -> &[Vec2] {
        &self.velocities
    }

    pub(in crate::particle) fn flags(&self) -> &[ParticleFlags] {
        &self.flags
    }

    pub(in crate::particle) fn groups(&self) -> &[Option<ParticleGroupId>] {
        &self.groups
    }

    pub(in crate::particle) fn weights(&self) -> &[f32] {
        &self.weights
    }

    pub(in crate::particle) fn forces(&self) -> &[Vec2] {
        &self.forces
    }

    pub(in crate::particle) fn maybe_colors(&self) -> Option<&[ParticleColor]> {
        self.maybe_colors.as_deref()
    }

    pub(in crate::particle) fn particle_contacts(&self) -> &[ParticleContact] {
        &self.particle_contacts
    }

    pub(crate) fn semantic_particle_contacts(&self) -> Vec<SemanticParticleContact> {
        self.particle_contacts
            .iter()
            .map(|contact| {
                SemanticParticleContact::new_internal(
                    contact.indices.map(|index| self.particle_id_at(index)),
                    contact.flags,
                    contact.weight,
                    contact.normal,
                )
            })
            .collect()
    }

    pub(crate) fn replace_particle_contacts(
        &mut self,
        contacts: &[SemanticParticleContact],
    ) -> Result<(), ParticleStorageError> {
        let particle_contacts = contacts
            .iter()
            .map(|contact| {
                let [first, second] = contact.particles();
                Ok(ParticleContact {
                    indices: [self.resolve_live(first)?, self.resolve_live(second)?],
                    flags: contact.flags(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                })
            })
            .collect::<Result<Vec<_>, ParticleStorageError>>()?;
        self.particle_contacts = particle_contacts;
        self.recompute_weights();
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    pub(in crate::particle) fn body_contacts(&self) -> &[ParticleBodyContact] {
        &self.body_contacts
    }

    pub(crate) fn semantic_body_contacts(&self) -> Vec<SemanticBodyContact> {
        self.body_contacts
            .iter()
            .map(|contact| {
                SemanticBodyContact::new_internal(
                    self.particle_id_at(contact.index),
                    contact.body,
                    contact.fixture,
                    contact.weight,
                    contact.normal,
                    contact.mass,
                )
            })
            .collect()
    }

    pub(crate) fn replace_body_contacts(
        &mut self,
        contacts: &[SemanticBodyContact],
    ) -> Result<(), ParticleStorageError> {
        let body_contacts = contacts
            .iter()
            .map(|contact| {
                Ok(ParticleBodyContact {
                    index: self.resolve_live(contact.particle())?,
                    body: contact.body(),
                    fixture: contact.fixture(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                    mass: contact.mass(),
                })
            })
            .collect::<Result<Vec<_>, ParticleStorageError>>()?;
        self.body_contacts = body_contacts;
        self.recompute_weights();
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    pub(crate) fn update_stuck_candidates(&mut self, timestamp: u32, threshold: u32) {
        if threshold == 0 {
            return;
        }
        let particle_count = self.len();
        if self.maybe_stuck.is_none() {
            self.maybe_stuck = Some(StuckLanes {
                last_body_contact_steps: vec![0; particle_count],
                body_contact_counts: vec![0; particle_count],
                consecutive_contact_steps: vec![0; particle_count],
                candidates: Vec::new(),
            });
        }
        let stuck = self
            .maybe_stuck
            .as_mut()
            .expect("stuck lanes were allocated before update");
        stuck.body_contact_counts.fill(0);
        stuck.candidates.clear();
        for row in 0..particle_count {
            if timestamp > stuck.last_body_contact_steps[row].saturating_add(1) {
                stuck.consecutive_contact_steps[row] = 0;
            }
        }
        for contact in &self.body_contacts {
            let row = contact.index.0;
            stuck.body_contact_counts[row] += 1;
            if stuck.body_contact_counts[row] == 2 {
                stuck.consecutive_contact_steps[row] += 1;
                if stuck.consecutive_contact_steps[row] > threshold {
                    stuck.candidates.push(contact.index);
                }
            }
            stuck.last_body_contact_steps[row] = timestamp;
        }
    }

    pub(crate) fn particle_velocity(
        &self,
        particle: ParticleId,
    ) -> Result<Vec2, ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        Ok(self.velocities[index.0])
    }

    pub(crate) fn particle_weight(
        &self,
        particle: ParticleId,
    ) -> Result<f32, ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        Ok(self.weights[index.0])
    }

    pub(crate) fn set_particle_velocity_internal(
        &mut self,
        particle: ParticleId,
        velocity: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        self.velocities[index.0] = velocity;
        Ok(())
    }

    pub(in crate::particle) fn resolve_contiguous_live_range(
        &self,
        particles: &[ParticleId],
    ) -> Result<Range<usize>, ParticleStorageError> {
        let indices = particles
            .iter()
            .copied()
            .map(|particle| self.resolve_live(particle))
            .collect::<Result<Vec<_>, _>>()?;
        let Some(first) = indices.first().copied() else {
            return Err(ParticleStorageError::InvalidGroupRange);
        };
        if indices.windows(2).any(|pair| pair[1].0 != pair[0].0 + 1) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        Ok(first.0..first.0 + indices.len())
    }

    pub(in crate::particle) fn range_contains_wall(&self, range: Range<usize>) -> bool {
        self.flags[range]
            .iter()
            .any(|flags| flags.contains(ParticleFlags::WALL))
    }

    pub(in crate::particle) fn force_range(&self, range: Range<usize>) -> &[Vec2] {
        &self.forces[range]
    }

    pub(in crate::particle) fn velocity_range(&self, range: Range<usize>) -> &[Vec2] {
        &self.velocities[range]
    }

    pub(in crate::particle) fn replace_force_range(
        &mut self,
        range: Range<usize>,
        forces: &[Vec2],
    ) {
        self.forces[range].copy_from_slice(forces);
    }

    pub(in crate::particle) fn replace_velocity_range(
        &mut self,
        range: Range<usize>,
        velocities: &[Vec2],
    ) {
        self.velocities[range].copy_from_slice(velocities);
    }

    pub(in crate::particle) fn stuck_candidates(
        &self,
    ) -> impl ExactSizeIterator<Item = ParticleId> + '_ {
        self.maybe_stuck
            .as_ref()
            .map_or(&[] as &[ParticleIndex], |stuck| stuck.candidates.as_slice())
            .iter()
            .copied()
            .map(|index| self.particle_id_at(index))
    }

    fn recompute_weights(&mut self) {
        self.weights.fill(0.0);
        for contact in &self.body_contacts {
            self.weights[contact.index.0] += contact.weight;
        }
        for contact in &self.particle_contacts {
            for index in contact.indices {
                self.weights[index.0] += contact.weight;
            }
        }
    }

    pub(in crate::particle) fn pairs(&self) -> &[ParticlePair] {
        &self.pairs
    }

    pub(in crate::particle) fn triads(&self) -> &[ParticleTriad] {
        &self.triads
    }

    pub(in crate::particle) fn maybe_expiration_order(&self) -> Option<&[ParticleIndex]> {
        self.maybe_expiration_order.as_deref()
    }

    pub(in crate::particle) fn lifetime_tracking_enabled(&self) -> bool {
        self.maybe_expiration_times.is_some()
    }

    pub(in crate::particle) fn enable_lifetime_tracking(&mut self) {
        if self.maybe_expiration_times.is_none() {
            self.maybe_expiration_times = Some(vec![0; self.len()]);
            self.maybe_expiration_order = Some((0..self.len()).map(ParticleIndex).collect());
        }
    }

    pub(in crate::particle) fn expiration_time(
        &self,
        id: ParticleId,
    ) -> Result<i32, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.maybe_expiration_times
            .as_ref()
            .map(|times| times[dense.0])
            .ok_or(ParticleStorageError::InvalidLaneBundle)
    }

    pub(in crate::particle) fn set_expiration_time(
        &mut self,
        id: ParticleId,
        expiration: i32,
    ) -> Result<bool, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.enable_lifetime_tracking();
        let times = self
            .maybe_expiration_times
            .as_mut()
            .expect("lifetime tracking was enabled before mutation");
        let changed = times[dense.0] != expiration;
        times[dense.0] = expiration;
        Ok(changed)
    }

    pub(in crate::particle) fn expiration_entries(&self) -> Vec<(ParticleId, i32)> {
        let Some(times) = self.maybe_expiration_times.as_ref() else {
            return Vec::new();
        };
        let order = self
            .maybe_expiration_order
            .as_ref()
            .expect("expiration times and order are allocated together");
        order
            .iter()
            .map(|index| (self.particle_id_at(*index), times[index.0]))
            .collect()
    }

    pub(in crate::particle) fn replace_expiration_order(
        &mut self,
        ordered_ids: &[ParticleId],
    ) -> Result<(), ParticleStorageError> {
        let order = ordered_ids
            .iter()
            .map(|id| self.resolve_present(*id))
            .collect::<Result<Vec<_>, _>>()?;
        if order.len() != self.len() {
            return Err(ParticleStorageError::InvalidDerivedReference);
        }
        self.maybe_expiration_order = Some(order);
        Ok(())
    }

    pub(in crate::particle) fn is_pending(
        &self,
        id: ParticleId,
    ) -> Result<bool, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        Ok(matches!(entry.state, IdentityState::PendingDelete { .. }))
    }

    pub(in crate::particle) fn pending_snapshot(
        &self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::PendingDelete { snapshot, .. } => Ok(snapshot),
            IdentityState::Live(_) | IdentityState::Vacant | IdentityState::Retired => {
                Err(ParticleStorageError::InvalidPermutation)
            }
        }
    }

    pub(in crate::particle) fn particle_id_at(&self, index: ParticleIndex) -> ParticleId {
        self.dense_to_id[index.0]
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

    pub(crate) fn into_buffer_bundle(self, mode: ParticleBufferMode) -> ParticleBufferBundle {
        ParticleBufferBundle::from_storage(
            mode,
            ParticleBufferLanes::new(
                self.positions,
                self.velocities,
                self.flags,
                self.maybe_colors,
            ),
        )
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

    pub(crate) fn commit_kinematic_edit(
        &mut self,
        id: ParticleId,
        position: Vec2,
        velocity: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let position_changed = self.positions[dense.0] != position;
        self.positions[dense.0] = position;
        self.velocities[dense.0] = velocity;
        if position_changed {
            self.repair_spatial_state();
        }
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    fn repair_spatial_state(&mut self) {
        self.weights.fill(0.0);
        if let Some(stuck) = &mut self.maybe_stuck {
            stuck.last_body_contact_steps.fill(0);
            stuck.body_contact_counts.fill(0);
            stuck.consecutive_contact_steps.fill(0);
            stuck.candidates.clear();
        }
        self.proxies = (0..self.len())
            .map(|index| ParticleProxy::new(ParticleIndex(index)))
            .collect();
        self.particle_contacts.clear();
        self.body_contacts.clear();
        self.pairs.clear();
        self.triads.clear();
    }

    pub(crate) fn mark_delete(
        &mut self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let local_slot = self.local_slot(id)?;
        let snapshot = ParticleSnapshot {
            id,
            diagnostic_id: self.identities[local_slot]
                .diagnostic_id
                .expect("live particles always retain a diagnostic identity"),
            input: self.input_at(dense),
        };
        self.identities[local_slot].state = IdentityState::PendingDelete { dense, snapshot };
        Ok(snapshot)
    }

    pub(in crate::particle) fn mark_delete_for_lifecycle(
        &mut self,
        id: ParticleId,
        request_listener: bool,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.flags[dense.0].insert(ParticleFlags::ZOMBIE);
        if request_listener {
            self.flags[dense.0].insert(ParticleFlags::DESTRUCTION_LISTENER);
        }
        self.mark_delete(id)
    }

    pub(crate) fn compact_pending(
        &mut self,
    ) -> Result<Vec<ParticleSnapshot>, ParticleStorageError> {
        crate::particle::lifetime::compact_pending_with_occurrences(self)
            .map(|outcome| outcome.destroyed)
    }

    pub(crate) fn compact_particle(
        &mut self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        if !matches!(entry.state, IdentityState::PendingDelete { .. }) {
            return Err(ParticleStorageError::InvalidPermutation);
        }

        let mut next = 0;
        let old_to_new = self
            .dense_to_id
            .iter()
            .map(|candidate| {
                if *candidate == id {
                    return None;
                }
                let destination = next;
                next += 1;
                Some(destination)
            })
            .collect::<Vec<_>>();
        let mut destroyed = permutation::apply_permutation(self, &old_to_new)?;
        let Some(snapshot) = destroyed.pop() else {
            return Err(ParticleStorageError::InvalidPermutation);
        };
        debug_assert!(destroyed.is_empty());
        Ok(snapshot)
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

    fn resolve_present(&self, id: ParticleId) -> Result<ParticleIndex, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::Live(dense) | IdentityState::PendingDelete { dense, .. } => Ok(dense),
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

#[cfg(test)]
mod editor_tests;

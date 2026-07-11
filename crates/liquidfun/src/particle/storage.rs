#![allow(
    dead_code,
    reason = "the bounded storage spike is executable architecture evidence for later particle work"
)]

use crate::identity::{HandleIdentity, Identity, ParticleId, ParticleSystemId, WorldKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParticleInput {
    pub(crate) position: [i32; 2],
    pub(crate) velocity: [i32; 2],
    pub(crate) flags: u32,
    pub(crate) group: u16,
    pub(crate) maybe_color: Option<[u8; 4]>,
    pub(crate) maybe_lifetime: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityState {
    Live(ParticleIndex),
    PendingDelete {
        dense: ParticleIndex,
        snapshot: ParticleSnapshot,
    },
    Vacant,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IdentityEntry {
    generation: u64,
    state: IdentityState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GroupRange {
    group: u16,
    start: ParticleIndex,
    end: ParticleIndex,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OwnedLaneBundle {
    pub(crate) positions: Vec<[i32; 2]>,
    pub(crate) velocities: Vec<[i32; 2]>,
    pub(crate) flags: Vec<u32>,
    pub(crate) groups: Vec<u16>,
    pub(crate) maybe_colors: Option<Vec<[u8; 4]>>,
    pub(crate) maybe_lifetimes: Option<Vec<u32>>,
}

impl OwnedLaneBundle {
    pub(crate) fn with_capacity(capacity: usize, optional: bool) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            flags: Vec::with_capacity(capacity),
            groups: Vec::with_capacity(capacity),
            maybe_colors: optional.then(|| Vec::with_capacity(capacity)),
            maybe_lifetimes: optional.then(|| Vec::with_capacity(capacity)),
        }
    }

    fn validate_empty(&self, declared_capacity: usize) -> Result<(), ParticleStorageError> {
        if !self.positions.is_empty()
            || !self.velocities.is_empty()
            || !self.flags.is_empty()
            || !self.groups.is_empty()
            || self
                .maybe_colors
                .as_ref()
                .is_some_and(|lane| !lane.is_empty())
            || self
                .maybe_lifetimes
                .as_ref()
                .is_some_and(|lane| !lane.is_empty())
            || self.positions.capacity() < declared_capacity
            || self.velocities.capacity() < declared_capacity
            || self.flags.capacity() < declared_capacity
            || self.groups.capacity() < declared_capacity
            || self
                .maybe_colors
                .as_ref()
                .is_some_and(|lane| lane.capacity() < declared_capacity)
            || self
                .maybe_lifetimes
                .as_ref()
                .is_some_and(|lane| lane.capacity() < declared_capacity)
        {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
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
    positions: Vec<[i32; 2]>,
    velocities: Vec<[i32; 2]>,
    flags: Vec<u32>,
    groups: Vec<u16>,
    maybe_colors: Option<Vec<[u8; 4]>>,
    maybe_lifetimes: Option<Vec<u32>>,
    proxies: Vec<ParticleIndex>,
    contacts: Vec<[ParticleIndex; 2]>,
    pairs: Vec<[ParticleIndex; 2]>,
    triads: Vec<[ParticleIndex; 3]>,
    lifetime_order: Vec<ParticleIndex>,
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
            maybe_colors: lanes.maybe_colors,
            maybe_lifetimes: lanes.maybe_lifetimes,
            proxies: Vec::with_capacity(declared_capacity),
            contacts: Vec::new(),
            pairs: Vec::new(),
            triads: Vec::new(),
            lifetime_order: Vec::with_capacity(declared_capacity),
            group_ranges: Vec::new(),
        })
    }

    pub(crate) fn positions(&self) -> &[[i32; 2]] {
        &self.positions
    }

    pub(crate) fn into_owned_lanes(self) -> OwnedLaneBundle {
        OwnedLaneBundle {
            positions: self.positions,
            velocities: self.velocities,
            flags: self.flags,
            groups: self.groups,
            maybe_colors: self.maybe_colors,
            maybe_lifetimes: self.maybe_lifetimes,
        }
    }

    pub(crate) fn create(
        &mut self,
        input: ParticleInput,
    ) -> Result<ParticleId, ParticleStorageError> {
        if self.dense_to_id.len() >= self.declared_capacity {
            return Err(ParticleStorageError::CapacityExceeded {
                limit: self.declared_capacity,
            });
        }
        self.validate_appended_group(input.group)?;

        let (local_slot, generation) = self.allocate_identity_slot()?;
        let particle_slot = self
            .identity_slot_base
            .checked_add(local_slot)
            .ok_or(ParticleStorageError::IdentityExhausted)?;
        let id = ParticleId::from_identity(Identity::new(self.world, particle_slot, generation));
        let dense = ParticleIndex(self.dense_to_id.len());
        self.identities[local_slot].state = IdentityState::Live(dense);
        self.push_row(id, input);
        self.group_ranges = build_group_ranges(&self.groups)?;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(id)
    }

    fn allocate_identity_slot(&mut self) -> Result<(usize, u64), ParticleStorageError> {
        if let Some(local_slot) = self.free_identity_slots.pop() {
            let entry = self
                .identities
                .get(local_slot)
                .expect("free identity slots always refer to existing entries");
            debug_assert_eq!(entry.state, IdentityState::Vacant);
            return Ok((local_slot, entry.generation));
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
        self.identities.push(IdentityEntry {
            generation: 0,
            state: IdentityState::Vacant,
        });
        Ok((local_slot, 0))
    }

    fn push_row(&mut self, id: ParticleId, input: ParticleInput) {
        let previous_len = self.dense_to_id.len();
        self.dense_to_id.push(id);
        self.positions.push(input.position);
        self.velocities.push(input.velocity);
        self.flags.push(input.flags);
        self.groups.push(input.group);
        let dense = ParticleIndex(previous_len);
        self.proxies.push(dense);
        self.lifetime_order.push(dense);
        push_optional(
            &mut self.maybe_colors,
            input.maybe_color,
            [0; 4],
            previous_len,
        );
        push_optional(
            &mut self.maybe_lifetimes,
            input.maybe_lifetime,
            0,
            previous_len,
        );
    }

    pub(crate) fn input(&self, id: ParticleId) -> Result<ParticleInput, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        Ok(self.input_at(dense))
    }

    pub(crate) fn set_position(
        &mut self,
        id: ParticleId,
        position: [i32; 2],
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
            group: self.groups[dense.0],
            maybe_color: self.maybe_colors.as_ref().map(|lane| lane[dense.0]),
            maybe_lifetime: self.maybe_lifetimes.as_ref().map(|lane| lane[dense.0]),
        }
    }

    fn validate_appended_group(&self, group: u16) -> Result<(), ParticleStorageError> {
        let Some(last) = self.groups.last() else {
            return Ok(());
        };
        if *last == group || !self.groups.contains(&group) {
            return Ok(());
        }
        Err(ParticleStorageError::InvalidGroupRange)
    }

    fn check_invariants(&self) -> Result<(), ParticleStorageError> {
        let count = self.dense_to_id.len();
        if self.positions.len() != count
            || self.velocities.len() != count
            || self.flags.len() != count
            || self.groups.len() != count
            || self
                .maybe_colors
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self
                .maybe_lifetimes
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
        {
            return Err(ParticleStorageError::LaneLengthMismatch);
        }

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

        validate_references(&self.proxies, count)?;
        validate_reference_sets(&self.contacts, count)?;
        validate_reference_sets(&self.pairs, count)?;
        validate_reference_sets(&self.triads, count)?;
        validate_references(&self.lifetime_order, count)?;
        if self.group_ranges != build_group_ranges(&self.groups)? {
            return Err(ParticleStorageError::InvalidGroupRange);
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
        let proxies = remap_references(&self.proxies, old_to_new)?;
        let contacts = remap_reference_sets(&self.contacts, old_to_new)?;
        let pairs = remap_reference_sets(&self.pairs, old_to_new)?;
        let triads = remap_reference_sets(&self.triads, old_to_new)?;
        let lifetime_order = remap_references(&self.lifetime_order, old_to_new)?;
        let mut dense_to_id = vec![None; new_count];
        let mut positions = vec![[0; 2]; new_count];
        let mut velocities = vec![[0; 2]; new_count];
        let mut flags = vec![0; new_count];
        let mut groups = vec![0; new_count];
        let mut maybe_colors = self.maybe_colors.as_ref().map(|_| vec![[0; 4]; new_count]);
        let mut maybe_lifetimes = self.maybe_lifetimes.as_ref().map(|_| vec![0; new_count]);
        let mut identities = self.identities.clone();
        let mut destroyed = Vec::new();
        let mut freed_slots = Vec::new();

        for (old, maybe_new) in old_to_new.iter().copied().enumerate() {
            let id = self.dense_to_id[old];
            let local_slot = self.local_slot(id)?;
            let entry = &mut identities[local_slot];
            if let Some(new) = maybe_new {
                dense_to_id[new] = Some(id);
                positions[new] = self.positions[old];
                velocities[new] = self.velocities[old];
                flags[new] = self.flags[old];
                groups[new] = self.groups[old];
                copy_optional(
                    self.maybe_colors.as_deref(),
                    maybe_colors.as_deref_mut(),
                    old,
                    new,
                );
                copy_optional(
                    self.maybe_lifetimes.as_deref(),
                    maybe_lifetimes.as_deref_mut(),
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
            destroyed.push(snapshot);
            let Some(next_generation) = entry.generation.checked_add(1) else {
                entry.state = IdentityState::Retired;
                continue;
            };
            entry.generation = next_generation;
            entry.state = IdentityState::Vacant;
            freed_slots.push(local_slot);
        }

        let group_ranges = build_group_ranges(&groups)?;

        self.identities = identities;
        self.free_identity_slots.extend(freed_slots);
        self.retired_identity_slots = self
            .identities
            .iter()
            .filter(|entry| entry.state == IdentityState::Retired)
            .count();
        self.dense_to_id = dense_to_id
            .into_iter()
            .map(|maybe_id| maybe_id.expect("validated permutations fill every destination"))
            .collect();
        self.positions = positions;
        self.velocities = velocities;
        self.flags = flags;
        self.groups = groups;
        self.maybe_colors = maybe_colors;
        self.maybe_lifetimes = maybe_lifetimes;
        self.proxies = proxies;
        self.contacts = contacts;
        self.pairs = pairs;
        self.triads = triads;
        self.lifetime_order = lifetime_order;
        self.group_ranges = group_ranges;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(destroyed)
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

fn build_group_ranges(groups: &[u16]) -> Result<Vec<GroupRange>, ParticleStorageError> {
    let mut ranges: Vec<GroupRange> = Vec::new();
    for (dense, group) in groups.iter().copied().enumerate() {
        if let Some(last) = ranges.last_mut()
            && last.group == group
        {
            last.end = ParticleIndex(dense + 1);
            continue;
        }
        if ranges.iter().any(|range| range.group == group) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        ranges.push(GroupRange {
            group,
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

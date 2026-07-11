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
        if system.identity().world() != world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity_slot_base.checked_add(identity_capacity).is_none() {
            return Err(ParticleStorageError::IdentityExhausted);
        }

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
            positions: Vec::with_capacity(declared_capacity),
            velocities: Vec::with_capacity(declared_capacity),
            flags: Vec::with_capacity(declared_capacity),
            groups: Vec::with_capacity(declared_capacity),
            maybe_colors: None,
            maybe_lifetimes: None,
            proxies: Vec::with_capacity(declared_capacity),
            contacts: Vec::new(),
            pairs: Vec::new(),
            triads: Vec::new(),
            lifetime_order: Vec::with_capacity(declared_capacity),
            group_ranges: Vec::new(),
        })
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
pub(crate) mod identity {
    use super::*;

    fn input(value: i32) -> ParticleInput {
        ParticleInput {
            position: [value, -value],
            velocity: [value + 1, value + 2],
            flags: u32::try_from(value).expect("test values are non-negative"),
            group: 0,
            maybe_color: Some([u8::try_from(value).expect("test values fit u8"); 4]),
            maybe_lifetime: Some(u32::try_from(value).expect("test values fit u32") + 10),
        }
    }

    fn system(world: WorldKey, slot: usize) -> ParticleSystemId {
        ParticleSystemId::from_identity(Identity::new(world, slot, 0))
    }

    fn storage(world: WorldKey, system_slot: usize, identity_base: usize) -> ParticleStorage {
        ParticleStorage::new(world, system(world, system_slot), identity_base, 4, 4)
            .expect("test storage contract is valid")
    }

    #[test]
    fn stable_id_survives_group_rotation() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut storage = storage(world, 0, 0);
        let first = storage.create(input(1)).expect("first particle fits");
        let second = storage.create(input(2)).expect("second particle fits");
        let third = storage.create(input(3)).expect("third particle fits");

        // Act
        storage.rotate_rows(0, 1, 3).expect("rotation is valid");

        // Assert
        assert_eq!(storage.input(first), Ok(input(1)));
        assert_eq!(storage.input(second), Ok(input(2)));
        assert_eq!(storage.input(third), Ok(input(3)));
    }

    #[test]
    fn cross_system_id_is_rejected_before_dense_lookup() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut first = storage(world, 0, 0);
        let second = storage(world, 1, 4);
        let id = first.create(input(1)).expect("particle fits");

        // Act
        let result = second.input(id);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::WrongParticleSystem));
    }

    #[test]
    fn pending_delete_rejects_mutation_but_preserves_snapshot() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut storage = storage(world, 0, 0);
        let id = storage.create(input(7)).expect("particle fits");

        // Act
        let snapshot = storage
            .mark_delete(id)
            .expect("live particle can be marked");
        let mutation = storage.set_position(id, [99, 99]);

        // Assert
        assert_eq!(
            snapshot,
            ParticleSnapshot {
                id,
                input: input(7)
            }
        );
        assert_eq!(mutation, Err(ParticleStorageError::PendingDelete));
    }

    #[test]
    fn compacted_id_is_stale_and_snapshot_remains_owned() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut storage = storage(world, 0, 0);
        let id = storage.create(input(4)).expect("particle fits");
        storage
            .mark_delete(id)
            .expect("live particle can be marked");

        // Act
        let destroyed = storage.compact_pending().expect("compaction is valid");

        // Assert
        assert_eq!(
            destroyed,
            vec![ParticleSnapshot {
                id,
                input: input(4)
            }]
        );
        assert_eq!(
            storage.input(id),
            Err(ParticleStorageError::StaleOrDestroyed)
        );
    }

    #[test]
    fn declared_capacity_does_not_grow_implicitly() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut storage = ParticleStorage::new(world, system(world, 0), 0, 4, 1)
            .expect("test storage contract is valid");
        storage.create(input(1)).expect("declared row fits");

        // Act
        let result = storage.create(input(2));

        // Assert
        assert_eq!(
            result,
            Err(ParticleStorageError::CapacityExceeded { limit: 1 })
        );
        assert!(storage.dense_to_id.capacity() >= storage.declared_capacity);
    }

    #[test]
    fn retired_identity_reports_exhaustion_without_resurrection() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let mut storage = ParticleStorage::new(world, system(world, 0), 0, 1, 1)
            .expect("test storage contract is valid");
        storage.identities.push(IdentityEntry {
            generation: u64::MAX,
            state: IdentityState::Vacant,
        });
        storage.free_identity_slots.push(0);
        let id = storage
            .create(input(1))
            .expect("maximum generation can be live once");
        storage
            .mark_delete(id)
            .expect("live particle can be marked");
        storage.compact_pending().expect("compaction is valid");

        // Act
        let result = storage.create(input(2));

        // Assert
        assert_eq!(result, Err(ParticleStorageError::IdentityExhausted));
        assert_eq!(
            storage.input(id),
            Err(ParticleStorageError::StaleOrDestroyed)
        );
    }
}

#[cfg(test)]
pub(crate) mod permutation {
    use super::*;

    fn input(value: i32, group: u16, optional: bool) -> ParticleInput {
        ParticleInput {
            position: [value, -value],
            velocity: [value + 10, value + 20],
            flags: u32::try_from(value).expect("test values are non-negative"),
            group,
            maybe_color: optional
                .then_some([u8::try_from(value).expect("test values fit in a color component"); 4]),
            maybe_lifetime: optional
                .then_some(u32::try_from(value).expect("test values are non-negative") + 100),
        }
    }

    fn storage() -> ParticleStorage {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        ParticleStorage::new(world, system, 0, 8, 8).expect("test storage contract is valid")
    }

    fn input_with_optional_defaults(value: i32, group: u16) -> ParticleInput {
        let mut value = input(value, group, false);
        value.maybe_color = Some([0; 4]);
        value.maybe_lifetime = Some(0);
        value
    }

    fn populated_storage() -> (ParticleStorage, [ParticleId; 4]) {
        let mut storage = storage();
        let ids = [
            storage.create(input(1, 0, true)).expect("particle fits"),
            storage.create(input(2, 0, false)).expect("particle fits"),
            storage.create(input(3, 1, true)).expect("particle fits"),
            storage.create(input(4, 1, false)).expect("particle fits"),
        ];
        storage.contacts = vec![[ParticleIndex(0), ParticleIndex(2)]];
        storage.pairs = vec![[ParticleIndex(1), ParticleIndex(3)]];
        storage.triads = vec![[ParticleIndex(0), ParticleIndex(1), ParticleIndex(3)]];
        (storage, ids)
    }

    #[test]
    fn one_transaction_remaps_all_lanes_indices_and_group_ranges() {
        // Arrange
        let (mut storage, ids) = populated_storage();

        // Act
        storage
            .rotate_rows(0, 2, 4)
            .expect("whole-group rotation is valid");

        // Assert
        assert_eq!(storage.input(ids[0]), Ok(input(1, 0, true)));
        assert_eq!(
            storage.input(ids[1]),
            Ok(input_with_optional_defaults(2, 0))
        );
        assert_eq!(storage.input(ids[2]), Ok(input(3, 1, true)));
        assert_eq!(
            storage.input(ids[3]),
            Ok(input_with_optional_defaults(4, 1))
        );
        assert_eq!(
            storage.proxies,
            vec![
                ParticleIndex(2),
                ParticleIndex(3),
                ParticleIndex(0),
                ParticleIndex(1)
            ]
        );
        assert_eq!(storage.contacts, vec![[ParticleIndex(2), ParticleIndex(0)]]);
        assert_eq!(storage.pairs, vec![[ParticleIndex(3), ParticleIndex(1)]]);
        assert_eq!(
            storage.triads,
            vec![[ParticleIndex(2), ParticleIndex(3), ParticleIndex(1)]]
        );
        assert_eq!(
            storage.lifetime_order,
            vec![
                ParticleIndex(2),
                ParticleIndex(3),
                ParticleIndex(0),
                ParticleIndex(1)
            ]
        );
        assert_eq!(
            storage.group_ranges,
            vec![
                GroupRange {
                    group: 1,
                    start: ParticleIndex(0),
                    end: ParticleIndex(2)
                },
                GroupRange {
                    group: 0,
                    start: ParticleIndex(2),
                    end: ParticleIndex(4)
                },
            ]
        );
        assert_eq!(storage.check_invariants(), Ok(()));
    }

    #[test]
    fn invalid_duplicate_permutation_leaves_state_unchanged() {
        // Arrange
        let (mut storage, _ids) = populated_storage();
        let before = storage.clone();

        // Act
        let result = storage.apply_permutation(&[Some(0), Some(0), Some(1), Some(2)]);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::InvalidPermutation));
        assert!(storage == before);
    }

    #[test]
    fn out_of_range_derived_reference_leaves_state_unchanged() {
        // Arrange
        let (mut storage, _ids) = populated_storage();
        storage.contacts.push([ParticleIndex(0), ParticleIndex(99)]);
        let before = storage.clone();

        // Act
        let result = storage.rotate_rows(0, 2, 4);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::InvalidDerivedReference));
        assert!(storage == before);
    }

    #[test]
    fn mismatched_optional_lane_leaves_state_unchanged() {
        // Arrange
        let (mut storage, _ids) = populated_storage();
        storage
            .maybe_colors
            .as_mut()
            .expect("fixture enables the optional color lane")
            .pop();
        let before = storage.clone();

        // Act
        let result = storage.rotate_rows(0, 2, 4);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::LaneLengthMismatch));
        assert!(storage == before);
    }

    #[test]
    fn compaction_drops_removed_references_and_remaps_survivors() {
        // Arrange
        let (mut storage, ids) = populated_storage();
        storage.mark_delete(ids[0]).expect("particle is live");
        storage.mark_delete(ids[3]).expect("particle is live");

        // Act
        let destroyed = storage.compact_pending().expect("compaction is valid");

        // Assert
        assert_eq!(destroyed.len(), 2);
        assert!(storage.contacts.is_empty());
        assert!(storage.pairs.is_empty());
        assert!(storage.triads.is_empty());
        assert_eq!(storage.proxies, vec![ParticleIndex(0), ParticleIndex(1)]);
        assert_eq!(
            storage.lifetime_order,
            vec![ParticleIndex(0), ParticleIndex(1)]
        );
        assert_eq!(storage.check_invariants(), Ok(()));
    }
}

//! Incremental oldest-particle index. Rank-zero eviction is logarithmic.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};

use crate::ParticleId;

const POSITION_GAP: u64 = 1 << 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Placement {
    expiration: i32,
    position: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EvictionIndex {
    by_particle: HashMap<ParticleId, Placement>,
    finite: BTreeMap<(i32, Reverse<u64>), ParticleId>,
    infinite: BTreeMap<(Reverse<i32>, u64), ParticleId>,
    next_position: u64,
}

impl EvictionIndex {
    pub(super) fn new() -> Self {
        Self {
            by_particle: HashMap::new(),
            finite: BTreeMap::new(),
            infinite: BTreeMap::new(),
            next_position: POSITION_GAP,
        }
    }

    pub(super) fn from_ordered_entries(entries: &[(ParticleId, i32)]) -> Self {
        let mut index = Self::new();
        for (particle, expiration) in entries {
            index.insert_new(*particle, *expiration);
        }
        index
    }

    pub(super) fn len(&self) -> usize {
        self.by_particle.len()
    }

    /// Updates expiration without moving the particle in the current sequence.
    ///
    /// The lifetime order is stable-sorted lazily. An expiration change keeps
    /// the particle where it already sits until that sort runs.
    pub(super) fn upsert(&mut self, particle: ParticleId, expiration: i32) {
        let Some(existing) = self.by_particle.get(&particle).copied() else {
            self.insert_new(particle, expiration);
            return;
        };
        if existing.expiration == expiration {
            return;
        }
        self.remove_placement(particle, existing);
        self.insert_at(particle, expiration, existing.position);
    }

    pub(super) fn remove(&mut self, particle: ParticleId) -> bool {
        let Some(existing) = self.by_particle.get(&particle).copied() else {
            return false;
        };
        self.remove_placement(particle, existing);
        true
    }

    /// Oldest finite particle, then oldest infinite particle.
    pub(super) fn oldest(&self, rank: usize) -> Option<ParticleId> {
        if rank < self.finite.len() {
            return self.finite.values().nth(rank).copied();
        }
        let infinite_rank = rank - self.finite.len();
        self.infinite.values().nth(infinite_rank).copied()
    }

    /// Storage order: infinite particles, then finite particles from latest expiration.
    pub(super) fn storage_order(&self) -> Vec<ParticleId> {
        let mut ordered = Vec::with_capacity(self.by_particle.len());
        ordered.extend(self.infinite.values().copied());
        ordered.extend(self.finite.values().rev().copied());
        ordered
    }

    /// Assigns fresh positions in the current storage order.
    ///
    /// Call this after the storage vector is replaced with that order so the
    /// next lazy sort uses the same sequence.
    pub(super) fn resequence_to_storage_order(&mut self) {
        let ordered = self.storage_order();
        let expirations = ordered
            .iter()
            .filter_map(|particle| {
                self.by_particle
                    .get(particle)
                    .map(|placement| (*particle, placement.expiration))
            })
            .collect::<Vec<_>>();
        *self = Self::from_ordered_entries(&expirations);
    }

    fn insert_new(&mut self, particle: ParticleId, expiration: i32) {
        let position = self.next_position;
        self.next_position = self.next_position.saturating_add(POSITION_GAP);
        self.insert_at(particle, expiration, position);
    }

    fn insert_at(&mut self, particle: ParticleId, expiration: i32, position: u64) {
        self.next_position = self
            .next_position
            .max(position.saturating_add(POSITION_GAP));
        let placement = Placement {
            expiration,
            position,
        };
        self.by_particle.insert(particle, placement);
        self.insert_key(particle, placement);
    }

    fn insert_key(&mut self, particle: ParticleId, placement: Placement) {
        if placement.expiration > 0 {
            self.finite.insert(
                (placement.expiration, Reverse(placement.position)),
                particle,
            );
            return;
        }
        self.infinite.insert(
            (Reverse(placement.expiration), placement.position),
            particle,
        );
    }

    fn remove_placement(&mut self, particle: ParticleId, placement: Placement) {
        self.by_particle.remove(&particle);
        if placement.expiration > 0 {
            self.finite
                .remove(&(placement.expiration, Reverse(placement.position)));
            return;
        }
        self.infinite
            .remove(&(Reverse(placement.expiration), placement.position));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};

    fn particle(slot: usize) -> ParticleId {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        ParticleId::from_identity(Identity::new_particle(world, slot, 1, system.identity()))
    }

    #[test]
    fn equal_expirations_evict_the_newest_particle_first() {
        // Arrange
        let mut index = EvictionIndex::new();
        let first = particle(0);
        let second = particle(1);
        let third = particle(2);
        index.upsert(first, 4);
        index.upsert(second, 4);
        index.upsert(third, 4);

        // Act
        let oldest = index.oldest(0);
        let next = index.oldest(1);

        // Assert
        assert_eq!(oldest, Some(third));
        assert_eq!(next, Some(second));
    }

    #[test]
    fn earlier_expiration_outranks_a_newer_particle() {
        // Arrange
        let mut index = EvictionIndex::new();
        let early = particle(0);
        let later = particle(1);
        index.upsert(early, 2);
        index.upsert(later, 9);

        // Act
        let oldest = index.oldest(0);

        // Assert
        assert_eq!(oldest, Some(early));
    }

    #[test]
    fn moving_onto_an_expiration_keeps_the_sorted_sequence() {
        // Arrange
        let mut index = EvictionIndex::new();
        let first = particle(0);
        let moved = particle(1);
        let third = particle(2);
        index.upsert(first, 10);
        index.upsert(moved, 1);
        index.upsert(third, 10);
        index.resequence_to_storage_order();

        // Act
        index.upsert(moved, 10);

        // Assert
        assert_eq!(index.oldest(0), Some(moved));
        assert_eq!(index.storage_order(), vec![first, third, moved]);
    }
}

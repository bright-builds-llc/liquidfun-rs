//! Incremental oldest-particle index. Rank-zero eviction is logarithmic.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};

use crate::ParticleId;

const POSITION_GAP: u64 = 1 << 16;

/// Keyed lookup by engine-issued particle identity; never iterated.
type ParticleIdMap<V> = HashMap<ParticleId, V, BuildHasherDefault<ParticleIdHasher>>;

/// Deterministic multiply-rotate hasher for small integer identity fields.
///
/// Keys are engine-issued identities, not caller-chosen data, so the standard
/// hasher's flooding resistance buys nothing here. `finish` applies the
/// `SplitMix64` finalizer so both the bucket bits and the control bits are
/// well mixed.
#[derive(Debug, Clone, Copy, Default)]
struct ParticleIdHasher {
    state: u64,
}

impl ParticleIdHasher {
    const MULTIPLIER: u64 = 0x517c_c1b7_2722_0a95;

    fn add(&mut self, value: u64) {
        self.state = (self.state.rotate_left(5) ^ value).wrapping_mul(Self::MULTIPLIER);
    }
}

impl Hasher for ParticleIdHasher {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.add(u64::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.add(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.add(value);
    }

    fn write_usize(&mut self, value: usize) {
        self.add(value as u64);
    }

    fn write_isize(&mut self, value: isize) {
        self.add(value.cast_unsigned() as u64);
    }

    fn finish(&self) -> u64 {
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Placement {
    expiration: i32,
    position: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EvictionIndex {
    by_particle: ParticleIdMap<Placement>,
    finite: BTreeMap<(i32, Reverse<u64>), ParticleId>,
    infinite: BTreeMap<(Reverse<i32>, u64), ParticleId>,
    next_position: u64,
}

impl EvictionIndex {
    pub(super) fn new() -> Self {
        Self::with_capacity(0)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            by_particle: ParticleIdMap::with_capacity_and_hasher(
                capacity,
                BuildHasherDefault::default(),
            ),
            finite: BTreeMap::new(),
            infinite: BTreeMap::new(),
            next_position: POSITION_GAP,
        }
    }

    pub(super) fn from_ordered_entries(entries: &[(ParticleId, i32)]) -> Self {
        let mut index = Self::with_capacity(entries.len());
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
    ///
    /// Equal to rebuilding with `from_ordered_entries(storage order)`: the
    /// `i`-th particle in storage order gets position `POSITION_GAP * (i + 1)`.
    /// Positions rise along storage order, so neither map's key order changes
    /// and both are rebuilt in bulk from already sorted keys.
    pub(super) fn resequence_to_storage_order(&mut self) {
        let infinite = std::mem::take(&mut self.infinite);
        let finite = std::mem::take(&mut self.finite);
        let mut position = POSITION_GAP;
        let mut infinite_keys = Vec::with_capacity(infinite.len());
        for ((expiration, _old_position), particle) in infinite {
            if !self.set_position(particle, position) {
                continue;
            }
            infinite_keys.push(((expiration, position), particle));
            position = position.saturating_add(POSITION_GAP);
        }
        let mut finite_keys = Vec::with_capacity(finite.len());
        for ((expiration, _old_position), particle) in finite.into_iter().rev() {
            if !self.set_position(particle, position) {
                continue;
            }
            finite_keys.push(((expiration, Reverse(position)), particle));
            position = position.saturating_add(POSITION_GAP);
        }
        finite_keys.reverse();
        self.infinite = infinite_keys.into_iter().collect();
        self.finite = finite_keys.into_iter().collect();
        self.next_position = position;
    }

    /// Moves a placed particle; like the rebuild, an unplaced key is skipped.
    fn set_position(&mut self, particle: ParticleId, position: u64) -> bool {
        let Some(placement) = self.by_particle.get_mut(&particle) else {
            return false;
        };
        placement.position = position;
        true
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

    /// The original body of `resequence_to_storage_order`: a full rebuild.
    fn resequence_by_rebuild(index: &mut EvictionIndex) {
        let ordered = index.storage_order();
        let expirations = ordered
            .iter()
            .filter_map(|particle| {
                index
                    .by_particle
                    .get(particle)
                    .map(|placement| (*particle, placement.expiration))
            })
            .collect::<Vec<_>>();
        *index = EvictionIndex::from_ordered_entries(&expirations);
    }

    fn particles(count: usize) -> Vec<ParticleId> {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        (0..count)
            .map(|slot| {
                ParticleId::from_identity(Identity::new_particle(world, slot, 1, system.identity()))
            })
            .collect()
    }

    fn assert_same_order(index: &EvictionIndex, reference: &EvictionIndex) {
        assert_eq!(index.storage_order(), reference.storage_order());
        for rank in 0..=reference.len() {
            assert_eq!(index.oldest(rank), reference.oldest(rank));
        }
    }

    #[test]
    fn resequence_matches_rebuild_from_entries() {
        // Arrange
        let pool = particles(300);
        let mut index = EvictionIndex::new();
        let mut reference = EvictionIndex::new();
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut resequences = 0;

        // Act and Assert
        for step in 1..=2_000 {
            let roll = next();
            let particle = pool[usize::try_from(roll % 300).expect("pool index fits usize")];
            let expiration = i32::try_from((roll >> 32) % 48).expect("expiration fits i32") - 8;
            if (roll >> 16) % 5 == 0 {
                assert_eq!(index.remove(particle), reference.remove(particle));
            } else {
                index.upsert(particle, expiration);
                reference.upsert(particle, expiration);
            }
            if step % 40 == 0 {
                index.resequence_to_storage_order();
                resequence_by_rebuild(&mut reference);
                resequences += 1;
                assert_eq!(index, reference);
                assert_same_order(&index, &reference);
            }
            assert_eq!(index.storage_order(), reference.storage_order());
            assert_eq!(index.oldest(0), reference.oldest(0));
        }
        assert_eq!(resequences, 50);
        assert!(index.len() > 100);
    }

    #[test]
    fn index_equality_still_compares_contents() {
        // Arrange
        let pool = particles(3);
        let entries = [(pool[0], 5), (pool[1], 0), (pool[2], -1)];
        let first = EvictionIndex::from_ordered_entries(&entries);
        let mut second = EvictionIndex::from_ordered_entries(&entries);
        let equal_before = first == second;

        // Act
        second.upsert(particles(1)[0], 3);

        // Assert
        assert!(equal_before);
        assert_ne!(first, second);
    }
}

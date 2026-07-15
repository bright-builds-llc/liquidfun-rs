//! Source-compatible particle lifetime quantization and stable-ID ordering.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use crate::ParticleId;

use super::ParticleSystemDef;
use super::storage::{ParticleSnapshot, ParticleStorage, ParticleStorageError};

const FIXED_POINT_SCALE: f32 = 4_294_967_296.0;

/// A checked failure while advancing or quantizing particle lifetime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleLifetimeError {
    /// A timestep is not finite.
    NonFiniteTimeStep,
    /// A timestep is negative.
    NegativeTimeStep,
    /// The 32.32 elapsed-time accumulator cannot represent the result.
    ElapsedTimeOverflow,
    /// A lifetime is not finite.
    NonFiniteLifetime,
    /// A quantized expiration value cannot be represented as an i32.
    ExpirationOutOfRange,
    /// The same stable particle identity was inserted twice.
    DuplicateParticle,
}

impl fmt::Display for ParticleLifetimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteTimeStep => "particle lifetime timestep must be finite",
            Self::NegativeTimeStep => "particle lifetime timestep must be non-negative",
            Self::ElapsedTimeOverflow => "particle lifetime elapsed-time accumulator overflowed",
            Self::NonFiniteLifetime => "particle lifetime must be finite",
            Self::ExpirationOutOfRange => {
                "particle lifetime expiration must fit the pinned signed 32-bit range"
            }
            Self::DuplicateParticle => {
                "particle lifetime ordering already contains the stable identity"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ParticleLifetimeError {}

/// The pinned 32.32 elapsed-time accumulator for one particle system.
///
/// Integer units correspond to the system definition's lifetime granularity.
/// Fractional ticks are retained across calls, while the observable elapsed
/// value and lifetime inputs truncate toward zero exactly like the pinned
/// source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLifetimeClock {
    elapsed_fixed: i64,
    granularity: f32,
}

impl ParticleLifetimeClock {
    /// Creates a zeroed clock from an already checked particle-system definition.
    #[must_use]
    pub const fn from_system_definition(definition: ParticleSystemDef) -> Self {
        Self {
            elapsed_fixed: 0,
            granularity: definition.lifetime_granularity(),
        }
    }

    /// Returns elapsed time in whole lifetime-granularity units.
    #[must_use]
    pub fn quantized_time_elapsed(self) -> i32 {
        let elapsed = self.elapsed_fixed >> 32;
        i32::try_from(elapsed).unwrap_or_else(|_| {
            if elapsed.is_negative() {
                i32::MIN
            } else {
                i32::MAX
            }
        })
    }

    /// Advances the 32.32 clock and returns its whole-unit component.
    ///
    /// # Errors
    ///
    /// Invalid inputs and overflow leave the clock unchanged.
    pub fn advance(&mut self, timestep: f32) -> Result<i32, ParticleLifetimeError> {
        if !timestep.is_finite() {
            return Err(ParticleLifetimeError::NonFiniteTimeStep);
        }
        if timestep < 0.0 {
            return Err(ParticleLifetimeError::NegativeTimeStep);
        }
        let delta = fixed_delta(timestep, self.granularity)?;
        let elapsed_fixed = self
            .elapsed_fixed
            .checked_add(delta)
            .ok_or(ParticleLifetimeError::ElapsedTimeOverflow)?;
        let elapsed = elapsed_fixed >> 32;
        let quantized =
            i32::try_from(elapsed).map_err(|_| ParticleLifetimeError::ElapsedTimeOverflow)?;
        self.elapsed_fixed = elapsed_fixed;
        Ok(quantized)
    }

    /// Quantizes a relative lifetime into the pinned absolute expiration value.
    ///
    /// Positive values expire relative to the current clock. Values at or
    /// below zero remain non-positive and identify infinite-lifetime age.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite or out-of-range values.
    pub fn expiration_time(self, lifetime: f32) -> Result<i32, ParticleLifetimeError> {
        if !lifetime.is_finite() {
            return Err(ParticleLifetimeError::NonFiniteLifetime);
        }
        let quantized = quantized_lifetime(lifetime, self.granularity)?;
        if quantized <= 0 {
            return Ok(quantized);
        }
        self.quantized_time_elapsed()
            .checked_add(quantized)
            .ok_or(ParticleLifetimeError::ExpirationOutOfRange)
    }

    /// Creates an empty stable-identity expiration ordering.
    #[must_use]
    pub const fn ordering(self) -> ParticleLifetimeOrder {
        ParticleLifetimeOrder {
            entries: Vec::new(),
        }
    }

    /// Returns the creation-time expiration for a finite or infinite particle.
    ///
    /// When tracking is already enabled, a non-positive lifetime records the
    /// negated elapsed tick so infinite particles retain source-compatible age.
    ///
    /// # Errors
    ///
    /// Returns the same checked quantization errors as `expiration_time`.
    pub fn creation_expiration_time(
        self,
        lifetime: f32,
        tracking_enabled: bool,
    ) -> Result<Option<i32>, ParticleLifetimeError> {
        if lifetime > 0.0 {
            return self.expiration_time(lifetime).map(Some);
        }
        if !lifetime.is_finite() {
            return Err(ParticleLifetimeError::NonFiniteLifetime);
        }
        Ok(tracking_enabled.then(|| -self.quantized_time_elapsed()))
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the pinned C++ conversion truncates the positive 32.32 delta toward zero"
)]
fn fixed_delta(timestep: f32, granularity: f32) -> Result<i64, ParticleLifetimeError> {
    let scaled = (timestep / granularity) * FIXED_POINT_SCALE;
    if !scaled.is_finite() || scaled >= 9_223_372_036_854_775_808.0_f32 {
        return Err(ParticleLifetimeError::ElapsedTimeOverflow);
    }
    Ok(scaled.trunc() as i64)
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the pinned C++ conversion truncates checked relative lifetimes toward zero"
)]
fn quantized_lifetime(lifetime: f32, granularity: f32) -> Result<i32, ParticleLifetimeError> {
    let quantized = (lifetime / granularity).trunc();
    if !quantized.is_finite() || !(-2_147_483_648.0_f32..2_147_483_648.0_f32).contains(&quantized) {
        return Err(ParticleLifetimeError::ExpirationOutOfRange);
    }
    Ok(quantized as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpirationEntry {
    particle: ParticleId,
    expiration: i32,
    insertion_order: usize,
}

/// Stable particle identities ordered by pinned finite/infinite lifetime rules.
///
/// Equal finite expirations use the canonical pinned-oracle witness: the most
/// recently inserted identity is selected first by oldest-particle eviction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleLifetimeOrder {
    entries: Vec<ExpirationEntry>,
}

impl ParticleLifetimeOrder {
    /// Inserts one stable identity and quantized expiration value.
    ///
    /// # Errors
    ///
    /// Duplicate identities are rejected without changing the ordering.
    pub fn set_expiration(
        &mut self,
        particle: ParticleId,
        expiration: i32,
    ) -> Result<(), ParticleLifetimeError> {
        if self.entries.iter().any(|entry| entry.particle == particle) {
            return Err(ParticleLifetimeError::DuplicateParticle);
        }
        self.entries.push(ExpirationEntry {
            particle,
            expiration,
            insertion_order: self.entries.len(),
        });
        Ok(())
    }

    /// Returns the ranked oldest identity, preferring every finite lifetime.
    #[must_use]
    pub fn oldest_particle(&self, rank: usize) -> Option<ParticleId> {
        let mut finite = self
            .entries
            .iter()
            .filter(|entry| entry.expiration > 0)
            .copied()
            .collect::<Vec<_>>();
        finite.sort_by(compare_finite_oldest);
        if let Some(entry) = finite.get(rank) {
            return Some(entry.particle);
        }

        let infinite_rank = rank.checked_sub(finite.len())?;
        let mut infinite = self
            .entries
            .iter()
            .filter(|entry| entry.expiration <= 0)
            .copied()
            .collect::<Vec<_>>();
        infinite.sort_by(compare_infinite_oldest);
        infinite.get(infinite_rank).map(|entry| entry.particle)
    }
}

fn compare_finite_oldest(left: &ExpirationEntry, right: &ExpirationEntry) -> Ordering {
    left.expiration
        .cmp(&right.expiration)
        .then_with(|| right.insertion_order.cmp(&left.insertion_order))
}

fn compare_infinite_oldest(left: &ExpirationEntry, right: &ExpirationEntry) -> Ordering {
    right
        .expiration
        .cmp(&left.expiration)
        .then_with(|| left.insertion_order.cmp(&right.insertion_order))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "the lifecycle kernel is integrated into World stepping by the next Phase 9 plan"
)]
pub(crate) enum ParticleLifecycleError {
    Lifetime(ParticleLifetimeError),
    Storage(ParticleStorageError),
    CapacityExceeded { limit: usize },
    OldestRankOutOfRange,
}

impl From<ParticleLifetimeError> for ParticleLifecycleError {
    fn from(error: ParticleLifetimeError) -> Self {
        Self::Lifetime(error)
    }
}

impl From<ParticleStorageError> for ParticleLifecycleError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

/// Pure clock and ordering state applied transactionally to authoritative storage.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    dead_code,
    reason = "the lifecycle kernel is integrated into World stepping by the next Phase 9 plan"
)]
pub(crate) struct ParticleLifetimeState {
    clock: ParticleLifetimeClock,
    expiration_order_dirty: bool,
    destroy_by_age: bool,
    maybe_maximum_count: Option<usize>,
}

#[allow(
    dead_code,
    reason = "the lifecycle kernel is integrated into World stepping by the next Phase 9 plan"
)]
impl ParticleLifetimeState {
    pub(crate) const fn new(definition: ParticleSystemDef) -> Self {
        Self {
            clock: ParticleLifetimeClock::from_system_definition(definition),
            expiration_order_dirty: false,
            destroy_by_age: definition.destroys_by_age(),
            maybe_maximum_count: definition.maximum_count(),
        }
    }

    pub(crate) fn initialize_created_particle(
        &mut self,
        storage: &mut ParticleStorage,
        particle: ParticleId,
        lifetime: f32,
    ) -> Result<(), ParticleLifecycleError> {
        let maybe_expiration = self
            .clock
            .creation_expiration_time(lifetime, storage.lifetime_tracking_enabled())?;
        let Some(expiration) = maybe_expiration else {
            return Ok(());
        };
        self.set_expiration(storage, particle, expiration)
    }

    pub(crate) fn set_particle_lifetime(
        &mut self,
        storage: &mut ParticleStorage,
        particle: ParticleId,
        lifetime: f32,
    ) -> Result<(), ParticleLifecycleError> {
        let expiration = self.clock.expiration_time(lifetime)?;
        self.set_expiration(storage, particle, expiration)
    }

    fn set_expiration(
        &mut self,
        storage: &mut ParticleStorage,
        particle: ParticleId,
        expiration: i32,
    ) -> Result<(), ParticleLifecycleError> {
        self.expiration_order_dirty |= storage.set_expiration_time(particle, expiration)?;
        Ok(())
    }

    pub(crate) fn solve_lifetimes(
        &mut self,
        storage: &mut ParticleStorage,
        timestep: f32,
    ) -> Result<Vec<ParticleSnapshot>, ParticleLifecycleError> {
        let elapsed = self.clock.advance(timestep)?;
        self.sort_if_dirty(storage)?;
        let mut marked = Vec::new();
        for (particle, expiration) in storage.expiration_entries().into_iter().rev() {
            if elapsed < expiration || expiration <= 0 {
                break;
            }
            if storage.is_pending(particle)? {
                continue;
            }
            marked.push(storage.mark_delete_for_lifecycle(particle, false)?);
        }
        Ok(marked)
    }

    pub(crate) fn destroy_oldest_particle(
        &mut self,
        storage: &mut ParticleStorage,
        rank: usize,
        request_listener: bool,
    ) -> Result<ParticleSnapshot, ParticleLifecycleError> {
        self.sort_if_dirty(storage)?;
        let mut ordering = self.clock.ordering();
        for (particle, expiration) in storage.expiration_entries() {
            ordering.set_expiration(particle, expiration)?;
        }
        let particle = ordering
            .oldest_particle(rank)
            .ok_or(ParticleLifecycleError::OldestRankOutOfRange)?;
        storage
            .mark_delete_for_lifecycle(particle, request_listener)
            .map_err(Into::into)
    }

    pub(crate) fn prepare_capacity_for_creation(
        &mut self,
        storage: &mut ParticleStorage,
    ) -> Result<Option<ParticleSnapshot>, ParticleLifecycleError> {
        let Some(maximum) = self.maybe_maximum_count else {
            return Ok(None);
        };
        if storage.len() < maximum {
            return Ok(None);
        }
        if !self.destroy_by_age {
            return Err(ParticleLifecycleError::CapacityExceeded { limit: maximum });
        }
        let evicted = self.destroy_oldest_particle(storage, 0, false)?;
        let compacted = storage.compact_particle(evicted.id)?;
        Ok(Some(compacted))
    }

    fn sort_if_dirty(
        &mut self,
        storage: &mut ParticleStorage,
    ) -> Result<(), ParticleLifecycleError> {
        if !self.expiration_order_dirty {
            return Ok(());
        }
        let mut entries = storage.expiration_entries();
        entries.sort_by(|left, right| compare_source_order(left.1, right.1));
        storage.replace_expiration_order(
            &entries
                .into_iter()
                .map(|(particle, _expiration)| particle)
                .collect::<Vec<_>>(),
        )?;
        self.expiration_order_dirty = false;
        Ok(())
    }
}

#[allow(
    dead_code,
    reason = "the lifecycle kernel is integrated into World stepping by the next Phase 9 plan"
)]
fn compare_source_order(left: i32, right: i32) -> Ordering {
    let left_infinite = left <= 0;
    let right_infinite = right <= 0;
    if left_infinite == right_infinite {
        return right.cmp(&left);
    }
    if left_infinite {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
    use crate::math::Vec2;
    use crate::particle::ParticleFlags;
    use crate::particle::storage::{ParticleInput, ParticleStorage};

    use super::*;

    fn system_definition(maximum: usize) -> ParticleSystemDef {
        ParticleSystemDef::default()
            .with_lifetime_granularity(1.0)
            .expect("test granularity is positive")
            .with_maximum_count(maximum)
            .expect("test maximum fits")
    }

    fn storage(capacity: usize) -> ParticleStorage {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        ParticleStorage::new(world, system, 0, capacity, capacity).expect("test storage is valid")
    }

    fn input() -> ParticleInput {
        ParticleInput {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            flags: ParticleFlags::WATER,
            maybe_group: None,
            maybe_color: None,
            maybe_user_association: None,
            maybe_expiration_time: None,
        }
    }

    #[test]
    fn solve_marks_only_expired_finite_particles() {
        // Arrange
        let definition = system_definition(3);
        let mut storage = storage(3);
        let first = storage.create(input()).expect("first particle fits");
        let second = storage.create(input()).expect("second particle fits");
        let third = storage.create(input()).expect("third particle fits");
        let mut state = ParticleLifetimeState::new(definition);
        state
            .initialize_created_particle(&mut storage, first, 2.0)
            .expect("finite lifetime is valid");
        state
            .initialize_created_particle(&mut storage, second, 0.0)
            .expect("infinite lifetime is valid");
        state
            .initialize_created_particle(&mut storage, third, 4.0)
            .expect("finite lifetime is valid");

        // Act
        let marked = state
            .solve_lifetimes(&mut storage, 2.0)
            .expect("clock and storage remain valid");

        // Assert
        assert_eq!(
            marked
                .into_iter()
                .map(|snapshot| snapshot.id)
                .collect::<Vec<_>>(),
            vec![first]
        );
        assert_eq!(
            storage.input(first),
            Err(ParticleStorageError::PendingDelete)
        );
        assert!(storage.input(second).is_ok());
        assert!(storage.input(third).is_ok());
    }

    #[test]
    fn full_capacity_evicts_canonical_tie_without_listener_request() {
        // Arrange
        let definition = system_definition(2);
        let mut storage = storage(2);
        let first = storage.create(input()).expect("first particle fits");
        let second = storage.create(input()).expect("second particle fits");
        let mut state = ParticleLifetimeState::new(definition);
        state
            .initialize_created_particle(&mut storage, first, 2.5)
            .expect("finite lifetime is valid");
        state
            .initialize_created_particle(&mut storage, second, 2.5)
            .expect("finite lifetime is valid");

        // Act
        let evicted = state
            .prepare_capacity_for_creation(&mut storage)
            .expect("destroy-by-age frees capacity")
            .expect("full capacity evicts one particle");

        // Assert
        assert_eq!(evicted.id, second);
        assert!(evicted.input.flags.contains(ParticleFlags::ZOMBIE));
        assert!(
            !evicted
                .input
                .flags
                .contains(ParticleFlags::DESTRUCTION_LISTENER)
        );
        assert_eq!(storage.particle_ids(), &[first]);
        assert_eq!(
            storage.input(second),
            Err(ParticleStorageError::StaleOrDestroyed)
        );
    }
}

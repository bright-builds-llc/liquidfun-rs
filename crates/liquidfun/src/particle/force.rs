//! Transactional preparation and source-ordered particle force application.

use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::HandleError;
use crate::math::{Vec2, settings};
use crate::particle::storage::{ParticleStorage, ParticleStorageError};
use crate::{ParticleId, ParticleSystemDef};

/// A no-effect failure while applying a particle force or linear impulse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleForceError {
    /// A system or particle identity is foreign, stale, or pending deletion.
    InvalidHandle(HandleError),
    /// A contiguous-range operation received no particle identities.
    EmptyRange,
    /// Stable identities were not contiguous in current source order.
    NonContiguousRange,
    /// The vector's x-coordinate is not finite.
    NonFiniteX,
    /// The vector's y-coordinate is not finite.
    NonFiniteY,
    /// At least one selected particle is an immovable wall particle.
    WallParticle,
    /// Particle mass or the derived distribution scale is not finite and positive.
    InvalidDistribution,
}

impl fmt::Display for ParticleForceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid particle handle: {error}"),
            Self::EmptyRange => formatter.write_str("particle range must not be empty"),
            Self::NonContiguousRange => {
                formatter.write_str("particle identities must be contiguous in source order")
            }
            Self::NonFiniteX => formatter.write_str("particle force or impulse x must be finite"),
            Self::NonFiniteY => formatter.write_str("particle force or impulse y must be finite"),
            Self::WallParticle => {
                formatter.write_str("particle force or impulse cannot target a wall particle")
            }
            Self::InvalidDistribution => formatter.write_str(
                "particle mass or force/impulse distribution is outside the finite range",
            ),
        }
    }
}

impl Error for ParticleForceError {}

impl From<HandleError> for ParticleForceError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedParticleForce {
    range: Range<usize>,
    forces: Vec<Vec2>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedParticleImpulse {
    range: Range<usize>,
    velocities: Vec<Vec2>,
}

pub(crate) fn prepare_force(
    storage: &ParticleStorage,
    particles: &[ParticleId],
    force: Vec2,
) -> Result<PreparedParticleForce, ParticleForceError> {
    validate_vector(force)?;
    let range = prepare_range(storage, particles)?;
    let count = count_as_source_float(range.len())?;
    let distributed_force = force / count;
    if !distributed_force.is_valid() {
        return Err(ParticleForceError::InvalidDistribution);
    }
    let forces = storage
        .force_range(range.clone())
        .iter()
        .copied()
        .map(|accumulated| accumulated + distributed_force)
        .collect::<Vec<_>>();
    validate_candidates(&forces)?;
    Ok(PreparedParticleForce { range, forces })
}

pub(crate) fn prepare_impulse(
    storage: &ParticleStorage,
    definition: ParticleSystemDef,
    particles: &[ParticleId],
    impulse: Vec2,
) -> Result<PreparedParticleImpulse, ParticleForceError> {
    validate_vector(impulse)?;
    let range = prepare_range(storage, particles)?;
    let count = count_as_source_float(range.len())?;
    let diameter = 2.0 * definition.radius();
    let stride = settings::PARTICLE_STRIDE * diameter;
    let particle_mass = definition.density() * stride * stride;
    let total_mass = count * particle_mass;
    if !particle_mass.is_finite()
        || particle_mass <= 0.0
        || !total_mass.is_finite()
        || total_mass <= 0.0
    {
        return Err(ParticleForceError::InvalidDistribution);
    }
    let velocity_delta = impulse / total_mass;
    if !velocity_delta.is_valid() {
        return Err(ParticleForceError::InvalidDistribution);
    }
    let velocities = storage
        .velocity_range(range.clone())
        .iter()
        .copied()
        .map(|velocity| velocity + velocity_delta)
        .collect::<Vec<_>>();
    validate_candidates(&velocities)?;
    Ok(PreparedParticleImpulse { range, velocities })
}

pub(crate) fn apply_force(storage: &mut ParticleStorage, prepared: PreparedParticleForce) {
    storage.replace_force_range(prepared.range, &prepared.forces);
}

pub(crate) fn apply_impulse(storage: &mut ParticleStorage, prepared: PreparedParticleImpulse) {
    storage.replace_velocity_range(prepared.range, &prepared.velocities);
}

fn prepare_range(
    storage: &ParticleStorage,
    particles: &[ParticleId],
) -> Result<Range<usize>, ParticleForceError> {
    if particles.is_empty() {
        return Err(ParticleForceError::EmptyRange);
    }
    let range = storage
        .resolve_contiguous_live_range(particles)
        .map_err(force_storage_error)?;
    if storage.range_contains_wall(range.clone()) {
        return Err(ParticleForceError::WallParticle);
    }
    Ok(range)
}

fn validate_vector(value: Vec2) -> Result<(), ParticleForceError> {
    if !value.x.is_finite() {
        return Err(ParticleForceError::NonFiniteX);
    }
    if !value.y.is_finite() {
        return Err(ParticleForceError::NonFiniteY);
    }
    Ok(())
}

#[allow(
    clippy::cast_precision_loss,
    reason = "particle counts are bounded to int32 and the pinned source casts that count to float32"
)]
fn count_as_source_float(count: usize) -> Result<f32, ParticleForceError> {
    let count = count as f32;
    if !count.is_finite() || count <= 0.0 {
        return Err(ParticleForceError::InvalidDistribution);
    }
    Ok(count)
}

fn validate_candidates(candidates: &[Vec2]) -> Result<(), ParticleForceError> {
    if candidates.iter().any(|candidate| !candidate.is_valid()) {
        return Err(ParticleForceError::InvalidDistribution);
    }
    Ok(())
}

fn force_storage_error(error: ParticleStorageError) -> ParticleForceError {
    match error {
        ParticleStorageError::WrongWorld => {
            ParticleForceError::InvalidHandle(HandleError::WrongWorld)
        }
        ParticleStorageError::WrongParticleSystem => {
            ParticleForceError::InvalidHandle(HandleError::WrongParticleSystem)
        }
        ParticleStorageError::StaleOrDestroyed => {
            ParticleForceError::InvalidHandle(HandleError::StaleOrDestroyed)
        }
        ParticleStorageError::PendingDelete => {
            ParticleForceError::InvalidHandle(HandleError::PendingDelete)
        }
        ParticleStorageError::InvalidGroupRange => ParticleForceError::NonContiguousRange,
        ParticleStorageError::CapacityExceeded { .. }
        | ParticleStorageError::IdentityExhausted
        | ParticleStorageError::InvalidPermutation
        | ParticleStorageError::LaneLengthMismatch
        | ParticleStorageError::InvalidDerivedReference
        | ParticleStorageError::InvalidLaneBundle => {
            unreachable!("force preparation cannot violate authoritative storage invariants")
        }
    }
}

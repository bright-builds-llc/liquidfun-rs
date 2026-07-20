use crate::identity::{HandleIdentity, ParticleGroupId, ParticleSystemId};
use crate::math::Vec2;
use crate::particle::storage::group::GroupRecord;
use crate::{ParticleFlags, ParticleId};

use super::{BoundaryCandidate, BoundarySolverError};

#[allow(
    clippy::too_many_arguments,
    reason = "the boundary validates every aligned authoritative lane together"
)]
pub(super) fn validate_source_lanes(
    owner: ParticleSystemId,
    particle_ids: &[ParticleId],
    positions: &[Vec2],
    velocities: &[Vec2],
    forces: &[Vec2],
    flags: &[ParticleFlags],
    memberships: &[Option<ParticleGroupId>],
    groups: &[GroupRecord],
) -> Result<(), BoundarySolverError> {
    let count = particle_ids.len();
    if positions.len() != count
        || velocities.len() != count
        || forces.len() != count
        || flags.len() != count
        || memberships.len() != count
        || positions.iter().any(|value| !value.is_valid())
        || velocities.iter().any(|value| !value.is_valid())
        || forces.iter().any(|value| !value.is_valid())
        || particle_ids.iter().any(|particle| {
            particle.identity().maybe_particle_system() != Some(owner.identity().scope())
        })
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    for group in groups {
        group
            .validate(owner, count)
            .map_err(|_error| BoundarySolverError::InvalidInput)?;
        if memberships[group.range()]
            .iter()
            .any(|maybe_group| *maybe_group != Some(group.id))
        {
            return Err(BoundarySolverError::InvalidInput);
        }
    }
    for (index, maybe_group) in memberships.iter().copied().enumerate() {
        let Some(group_id) = maybe_group else {
            continue;
        };
        if !groups
            .iter()
            .any(|group| group.id == group_id && group.range().contains(&index))
        {
            return Err(BoundarySolverError::InvalidInput);
        }
    }
    Ok(())
}

pub(super) fn copy_slice<T: Copy>(
    source: &[T],
    resource_name: &'static str,
) -> Result<Vec<T>, BoundarySolverError> {
    let mut candidate = Vec::new();
    candidate
        .try_reserve_exact(source.len())
        .map_err(|_error| resource(resource_name, source.len()))?;
    candidate.extend_from_slice(source);
    Ok(candidate)
}

pub(super) fn validate_candidate(candidate: &BoundaryCandidate) -> Result<(), BoundarySolverError> {
    if candidate
        .positions
        .iter()
        .chain(&candidate.velocities)
        .chain(&candidate.forces)
        .any(|value| !value.is_valid())
        || candidate.groups.iter().any(|group| {
            !group.statistics.mass.is_finite()
                || !group.statistics.center.is_valid()
                || !group.statistics.linear_velocity.is_valid()
                || !group.statistics.inertia.is_finite()
                || !group.statistics.angular_velocity.is_finite()
        })
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    Ok(())
}

pub(super) const fn resource(resource: &'static str, limit: usize) -> BoundarySolverError {
    BoundarySolverError::ResourceLimit { resource, limit }
}

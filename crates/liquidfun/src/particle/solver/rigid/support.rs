use crate::ParticleId;
use crate::identity::{HandleIdentity, ParticleGroupId, ParticleSystemId};
use crate::math::Vec2;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache};

use super::{RigidBodyContact, RigidCandidate, RigidSolverError};

pub(super) fn copy_candidate(
    particle_ids: &[ParticleId],
    velocities: &[Vec2],
    groups: &[GroupRecord],
    body_impulse_limit: usize,
) -> Result<RigidCandidate, RigidSolverError> {
    let mut copied_ids = Vec::new();
    copied_ids
        .try_reserve_exact(particle_ids.len())
        .map_err(|_error| resource("rigid particle identities", particle_ids.len()))?;
    copied_ids.extend_from_slice(particle_ids);
    let mut copied_velocities = Vec::new();
    copied_velocities
        .try_reserve_exact(velocities.len())
        .map_err(|_error| resource("rigid velocity candidates", velocities.len()))?;
    copied_velocities.extend_from_slice(velocities);
    let mut copied_groups = Vec::new();
    copied_groups
        .try_reserve_exact(groups.len())
        .map_err(|_error| resource("rigid group candidates", groups.len()))?;
    copied_groups.extend_from_slice(groups);
    Ok(RigidCandidate {
        particle_ids: copied_ids,
        velocities: copied_velocities,
        groups: copied_groups,
        body_impulses: Vec::with_capacity(body_impulse_limit),
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the shared rigid boundary validates every aligned source lane"
)]
pub(super) fn validate_inputs(
    owner: ParticleSystemId,
    particle_ids: &[ParticleId],
    positions: &[Vec2],
    velocities: &[Vec2],
    memberships: &[Option<ParticleGroupId>],
    groups: &[GroupRecord],
    particle_mass: f32,
) -> Result<(), RigidSolverError> {
    let count = particle_ids.len();
    if positions.len() != count
        || velocities.len() != count
        || memberships.len() != count
        || !particle_mass.is_finite()
        || particle_mass <= 0.0
        || positions.iter().any(|position| !position.is_valid())
        || velocities.iter().any(|velocity| !velocity.is_valid())
        || particle_ids.iter().any(|particle| {
            particle.identity().maybe_particle_system() != Some(owner.identity().scope())
        })
    {
        return Err(RigidSolverError::InvalidInput);
    }
    for group in groups {
        group
            .validate(owner, count)
            .map_err(|_error| RigidSolverError::InvalidInput)?;
        if memberships[group.range()]
            .iter()
            .any(|maybe_group| *maybe_group != Some(group.id))
        {
            return Err(RigidSolverError::InvalidInput);
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
            return Err(RigidSolverError::InvalidInput);
        }
    }
    Ok(())
}

pub(super) fn validate_body_contact(
    contact: RigidBodyContact,
    particle_count: usize,
) -> Result<(), RigidSolverError> {
    if contact.particle >= particle_count
        || !contact.weight.is_finite()
        || contact.weight < 0.0
        || !contact.normal.is_valid()
        || !contact.body_mass.is_finite()
        || contact.body_mass < 0.0
        || !contact.body_inertia.is_finite()
        || contact.body_inertia < 0.0
        || !contact.body_center.is_valid()
        || !contact.body_linear_velocity.is_valid()
        || !contact.body_angular_velocity.is_finite()
    {
        return Err(RigidSolverError::InvalidInput);
    }
    Ok(())
}

pub(super) fn validate_candidate(candidate: &RigidCandidate) -> Result<(), RigidSolverError> {
    if candidate
        .velocities
        .iter()
        .any(|velocity| !velocity.is_valid())
        || candidate
            .groups
            .iter()
            .any(|group| !statistics_are_finite(group.statistics))
        || candidate
            .body_impulses
            .iter()
            .any(|impulse| !impulse.impulse.is_valid() || !impulse.point.is_valid())
    {
        return Err(RigidSolverError::InvalidInput);
    }
    Ok(())
}

pub(super) fn statistics_are_finite(statistics: GroupStatisticsCache) -> bool {
    statistics.mass.is_finite()
        && statistics.center.is_valid()
        && statistics.linear_velocity.is_valid()
        && statistics.inertia.is_finite()
        && statistics.angular_velocity.is_finite()
}

pub(super) const fn resource(resource: &'static str, limit: usize) -> RigidSolverError {
    RigidSolverError::ResourceLimit { resource, limit }
}

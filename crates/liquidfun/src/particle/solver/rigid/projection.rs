use crate::ParticleId;
use crate::identity::{ParticleGroupId, ParticleSystemId};
use crate::math::{Rotation, Transform, Vec2};
use crate::particle::ParticleGroupFlags;
use crate::particle::storage::group::GroupRecord;

use super::support::{copy_candidate, validate_candidate, validate_inputs};
use super::{RigidCandidate, RigidSolverError};

#[allow(
    clippy::too_many_arguments,
    reason = "the pure S24 candidate keeps every source lane explicit"
)]
pub(crate) fn rigid_projection_candidate(
    owner: ParticleSystemId,
    particle_ids: &[ParticleId],
    positions: &[Vec2],
    velocities: &[Vec2],
    memberships: &[Option<ParticleGroupId>],
    groups: &[GroupRecord],
    time_step: f32,
    inverse_time_step: f32,
) -> Result<RigidCandidate, RigidSolverError> {
    validate_inputs(
        owner,
        particle_ids,
        positions,
        velocities,
        memberships,
        groups,
        1.0,
    )?;
    if !time_step.is_finite()
        || time_step < 0.0
        || !inverse_time_step.is_finite()
        || inverse_time_step < 0.0
    {
        return Err(RigidSolverError::InvalidInput);
    }
    let mut candidate = copy_candidate(particle_ids, velocities, groups, 0)?;
    for group in &mut candidate.groups {
        if !group.flags.contains(ParticleGroupFlags::RIGID) || group.first == group.last {
            continue;
        }
        let statistics = group.statistics;
        let rotation = Rotation::from_angle(time_step * statistics.angular_velocity);
        let translation = statistics.center + time_step * statistics.linear_velocity
            - rotation.apply(statistics.center);
        let delta = Transform::new(translation, rotation);
        group.transform = delta.compose(group.transform);
        for (position, velocity) in positions[group.range()]
            .iter()
            .copied()
            .zip(&mut candidate.velocities[group.range()])
        {
            let rotated_velocity = Vec2::new(
                inverse_time_step
                    * ((rotation.cosine() - 1.0) * position.x - rotation.sine() * position.y),
                inverse_time_step
                    * (rotation.sine() * position.x + (rotation.cosine() - 1.0) * position.y),
            );
            *velocity = inverse_time_step * translation + rotated_velocity;
        }
    }
    validate_candidate(&candidate)?;
    Ok(candidate)
}

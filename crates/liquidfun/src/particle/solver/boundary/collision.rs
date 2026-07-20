use crate::ParticleFlags;
use crate::math::{Transform, Vec2, settings};

use super::support::{resource, validate_candidate};
use super::{
    BoundaryCandidate, BoundaryPass, BoundarySolverError, BoundaryStage, FilteredCollisionHit,
};

pub(in crate::particle::solver) fn collision_start_from_previous_transform(
    particle_position: Vec2,
    previous_transform: Transform,
    current_transform: Transform,
    body_local_center: Vec2,
    is_circle: bool,
    particle_iteration: u32,
) -> Result<Vec2, BoundarySolverError> {
    if !particle_position.is_valid()
        || !body_local_center.is_valid()
        || !transform_is_finite(previous_transform)
        || !transform_is_finite(current_transform)
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    if particle_iteration != 0 {
        return Ok(particle_position);
    }
    let mut local_start = previous_transform.inverse_apply(particle_position);
    if is_circle {
        local_start -= body_local_center;
        local_start = previous_transform.rotation().apply(local_start);
        local_start = current_transform.rotation().inverse_apply(local_start);
        local_start += body_local_center;
    }
    Ok(current_transform.apply(local_start))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the filtered collision candidate keeps source timing and bounds explicit"
)]
pub(in crate::particle::solver) fn collision_candidate(
    source: &BoundaryCandidate,
    hits: &[FilteredCollisionHit],
    particle_iteration: u32,
    particle_mass: f32,
    time_step: f32,
    inverse_time_step: f32,
    hit_limit: usize,
) -> Result<BoundaryCandidate, BoundarySolverError> {
    if hits.len() > hit_limit {
        return Err(resource("filtered collision hits", hit_limit));
    }
    if !particle_mass.is_finite()
        || particle_mass <= 0.0
        || !time_step.is_finite()
        || time_step < 0.0
        || !inverse_time_step.is_finite()
        || inverse_time_step < 0.0
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    let mut candidate = source.begin_pass(BoundaryStage::AfterBarrier)?;
    for hit in hits {
        validate_hit(*hit, candidate.positions.len())?;
        let particle = hit.particle;
        let position = candidate.positions[particle];
        let prior_velocity = candidate.velocities[particle];
        let start = collision_start_from_previous_transform(
            position,
            hit.previous_transform,
            hit.current_transform,
            hit.body_local_center,
            hit.is_circle,
            particle_iteration,
        )?;
        let end = position + time_step * prior_velocity;
        let impact_position =
            (1.0 - hit.fraction) * start + hit.fraction * end + settings::LINEAR_SLOP * hit.normal;
        let velocity = inverse_time_step * (impact_position - position);
        let force = inverse_time_step * particle_mass * (prior_velocity - velocity);
        candidate.velocities[particle] = velocity;
        candidate.forces[particle] += force;
        candidate.has_pending_force = true;
        candidate.record_effect(BoundaryPass::Collision, particle, Some(hit.body))?;
    }
    candidate.stage = BoundaryStage::AfterCollision;
    candidate.pass_trace.push(BoundaryPass::Collision);
    validate_candidate(&candidate)?;
    Ok(candidate)
}

pub(in crate::particle::solver) fn mark_rigid_projection(
    source: &BoundaryCandidate,
) -> Result<BoundaryCandidate, BoundarySolverError> {
    let mut candidate = source.begin_pass(BoundaryStage::AfterCollision)?;
    candidate.stage = BoundaryStage::AfterRigidProjection;
    candidate.pass_trace.push(BoundaryPass::Rigid);
    Ok(candidate)
}

pub(in crate::particle::solver) fn wall_candidate(
    source: &BoundaryCandidate,
) -> Result<BoundaryCandidate, BoundarySolverError> {
    let mut candidate = source.begin_pass(BoundaryStage::AfterRigidProjection)?;
    for (flags, velocity) in candidate
        .flags
        .iter()
        .copied()
        .zip(&mut candidate.velocities)
    {
        if flags.contains(ParticleFlags::WALL) {
            *velocity = Vec2::ZERO;
        }
    }
    candidate.stage = BoundaryStage::AfterWall;
    candidate.pass_trace.push(BoundaryPass::Wall);
    validate_candidate(&candidate)?;
    Ok(candidate)
}

pub(in crate::particle::solver) fn integrate_candidate(
    source: &BoundaryCandidate,
    time_step: f32,
) -> Result<BoundaryCandidate, BoundarySolverError> {
    if !time_step.is_finite() || time_step < 0.0 {
        return Err(BoundarySolverError::InvalidInput);
    }
    let mut candidate = source.begin_pass(BoundaryStage::AfterWall)?;
    for (position, velocity) in candidate
        .positions
        .iter_mut()
        .zip(candidate.velocities.iter().copied())
    {
        *position += time_step * velocity;
    }
    candidate.stage = BoundaryStage::Integrated;
    candidate.pass_trace.push(BoundaryPass::Integrate);
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn validate_hit(
    hit: FilteredCollisionHit,
    particle_count: usize,
) -> Result<(), BoundarySolverError> {
    if hit.particle >= particle_count
        || !hit.body_local_center.is_valid()
        || !hit.fraction.is_finite()
        || hit.fraction < 0.0
        || hit.fraction > 1.0
        || !hit.normal.is_valid()
        || !transform_is_finite(hit.previous_transform)
        || !transform_is_finite(hit.current_transform)
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    Ok(())
}

fn transform_is_finite(transform: Transform) -> bool {
    transform.position().is_valid()
        && transform.rotation().sine().is_finite()
        && transform.rotation().cosine().is_finite()
}

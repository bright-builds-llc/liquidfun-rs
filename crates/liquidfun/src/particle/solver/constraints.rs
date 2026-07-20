//! Exact S18-S20 particle topology and velocity-limit kernels.

use crate::math::{Vec2, inverse_sqrt};
use crate::particle::ParticleFlags;
use crate::particle::definition::ParticleSystemDef;
use crate::particle::storage::lanes::{ParticlePair, ParticleTriad};
use crate::particle::storage::{ParticleStorage, ParticleStorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConstraintSolverError {
    Storage(ParticleStorageError),
    ZeroLengthPairDistance,
}

impl From<ParticleStorageError> for ConstraintSolverError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

/// S18 `Elastic`: enforce complete stored triad rest state in source order.
pub(super) fn elastic(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    time_step: f32,
    inverse_time_step: f32,
) -> Result<(), ConstraintSolverError> {
    let stable_ids = stable_particle_ids(storage)?;
    let velocities = elastic_candidate(
        storage.positions(),
        storage.velocities(),
        storage.triads(),
        definition.elastic_strength(),
        time_step,
        inverse_time_step,
    )?;
    validate_stable_ids(storage, &stable_ids)?;
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

/// S19 `Spring`: enforce stored pair rest distances in source order.
pub(super) fn spring(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    time_step: f32,
    inverse_time_step: f32,
) -> Result<(), ConstraintSolverError> {
    let stable_ids = stable_particle_ids(storage)?;
    let velocities = spring_candidate(
        storage.positions(),
        storage.velocities(),
        storage.pairs(),
        definition.spring_strength(),
        time_step,
        inverse_time_step,
    )?;
    validate_stable_ids(storage, &stable_ids)?;
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

/// S20 `LimitVelocity`: clamp only speeds strictly above the critical threshold.
pub(super) fn limit_velocity(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ConstraintSolverError> {
    let stable_ids = stable_particle_ids(storage)?;
    let velocities = limit_velocity_candidate(
        storage.velocities(),
        2.0 * definition.radius(),
        inverse_time_step,
    )?;
    validate_stable_ids(storage, &stable_ids)?;
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

fn elastic_candidate(
    positions: &[Vec2],
    velocities: &[Vec2],
    triads: &[ParticleTriad],
    elastic_strength: f32,
    time_step: f32,
    inverse_time_step: f32,
) -> Result<Vec<Vec2>, ConstraintSolverError> {
    validate_step_and_lanes(positions, velocities, time_step, inverse_time_step)?;
    validate_triads(triads, positions.len())?;
    let mut candidate = copy_velocities(velocities)?;
    let step_strength = inverse_time_step * elastic_strength;
    if !step_strength.is_finite() {
        return Err(invalid_lane());
    }

    for triad in triads {
        if !triad.flags.intersects(ParticleFlags::ELASTIC) {
            continue;
        }
        solve_elastic_triad(positions, &mut candidate, *triad, step_strength, time_step);
    }
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn solve_elastic_triad(
    positions: &[Vec2],
    velocities: &mut [Vec2],
    triad: ParticleTriad,
    step_strength: f32,
    time_step: f32,
) {
    let [a, b, c] = triad.indices.map(|index| index.0);
    let mut pa = positions[a];
    let mut pb = positions[b];
    let mut pc = positions[c];
    let mut va = velocities[a];
    let mut vb = velocities[b];
    let mut vc = velocities[c];
    pa += time_step * va;
    pb += time_step * vb;
    pc += time_step * vc;
    let midpoint = (1.0_f32 / 3.0) * (pa + pb + pc);
    pa -= midpoint;
    pb -= midpoint;
    pc -= midpoint;

    let mut sine = triad.pa.cross(pa) + triad.pb.cross(pb) + triad.pc.cross(pc);
    let mut cosine = triad.pa.dot(pa) + triad.pb.dot(pb) + triad.pc.dot(pc);
    let rotation_squared = sine * sine + cosine * cosine;
    let inverse_rotation = inverse_sqrt(rotation_squared);
    sine *= inverse_rotation;
    cosine *= inverse_rotation;
    let strength = step_strength * triad.strength;
    va += strength * (rotate(cosine, sine, triad.pa) - pa);
    vb += strength * (rotate(cosine, sine, triad.pb) - pb);
    vc += strength * (rotate(cosine, sine, triad.pc) - pc);
    velocities[a] = va;
    velocities[b] = vb;
    velocities[c] = vc;
}

fn spring_candidate(
    positions: &[Vec2],
    velocities: &[Vec2],
    pairs: &[ParticlePair],
    spring_strength: f32,
    time_step: f32,
    inverse_time_step: f32,
) -> Result<Vec<Vec2>, ConstraintSolverError> {
    validate_step_and_lanes(positions, velocities, time_step, inverse_time_step)?;
    validate_pairs(pairs, positions.len())?;
    let mut candidate = copy_velocities(velocities)?;
    let step_strength = inverse_time_step * spring_strength;
    if !step_strength.is_finite() {
        return Err(invalid_lane());
    }

    for pair in pairs {
        if !pair.flags.intersects(ParticleFlags::SPRING) {
            continue;
        }
        let [a, b] = pair.indices.map(|index| index.0);
        let pa = positions[a] + time_step * candidate[a];
        let pb = positions[b] + time_step * candidate[b];
        let displacement = pb - pa;
        let current_distance = displacement.length();
        if current_distance == 0.0 {
            return Err(ConstraintSolverError::ZeroLengthPairDistance);
        }
        let strength = step_strength * pair.strength;
        let impulse =
            strength * (pair.distance - current_distance) / current_distance * displacement;
        candidate[a] -= impulse;
        candidate[b] += impulse;
    }
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn limit_velocity_candidate(
    velocities: &[Vec2],
    particle_diameter: f32,
    inverse_time_step: f32,
) -> Result<Vec<Vec2>, ConstraintSolverError> {
    if !particle_diameter.is_finite()
        || particle_diameter <= 0.0
        || !inverse_time_step.is_finite()
        || inverse_time_step < 0.0
    {
        return Err(invalid_lane());
    }
    let critical_velocity = particle_diameter * inverse_time_step;
    let critical_velocity_squared = critical_velocity * critical_velocity;
    if !critical_velocity_squared.is_finite() {
        return Err(invalid_lane());
    }
    let mut candidate = copy_velocities(velocities)?;
    for velocity in &mut candidate {
        let speed_squared = velocity.dot(*velocity);
        if speed_squared > critical_velocity_squared {
            *velocity *= (critical_velocity_squared / speed_squared).sqrt();
        }
    }
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn validate_step_and_lanes(
    positions: &[Vec2],
    velocities: &[Vec2],
    time_step: f32,
    inverse_time_step: f32,
) -> Result<(), ConstraintSolverError> {
    if positions.len() != velocities.len()
        || positions.iter().any(|position| !position.is_valid())
        || velocities.iter().any(|velocity| !velocity.is_valid())
        || !time_step.is_finite()
        || time_step < 0.0
        || !inverse_time_step.is_finite()
        || inverse_time_step < 0.0
    {
        return Err(invalid_lane());
    }
    Ok(())
}

fn validate_pairs(
    pairs: &[ParticlePair],
    particle_count: usize,
) -> Result<(), ConstraintSolverError> {
    for pair in pairs {
        pair.validate(particle_count)?;
        if pair.indices[0] == pair.indices[1] {
            return Err(invalid_lane());
        }
        if pair.flags.intersects(ParticleFlags::SPRING) && pair.distance == 0.0 {
            return Err(ConstraintSolverError::ZeroLengthPairDistance);
        }
    }
    Ok(())
}

fn validate_triads(
    triads: &[ParticleTriad],
    particle_count: usize,
) -> Result<(), ConstraintSolverError> {
    for triad in triads {
        triad.validate(particle_count)?;
        let [a, b, c] = triad.indices;
        if a == b || b == c || c == a {
            return Err(invalid_lane());
        }
    }
    Ok(())
}

fn stable_particle_ids(
    storage: &ParticleStorage,
) -> Result<Vec<crate::ParticleId>, ConstraintSolverError> {
    let mut ids = Vec::new();
    ids.try_reserve_exact(storage.len())
        .map_err(|_error| invalid_lane())?;
    ids.extend_from_slice(storage.particle_ids());
    Ok(ids)
}

fn validate_stable_ids(
    storage: &ParticleStorage,
    stable_ids: &[crate::ParticleId],
) -> Result<(), ConstraintSolverError> {
    if storage.particle_ids() != stable_ids {
        return Err(invalid_lane());
    }
    Ok(())
}

fn copy_velocities(velocities: &[Vec2]) -> Result<Vec<Vec2>, ConstraintSolverError> {
    let mut candidate = Vec::new();
    candidate
        .try_reserve_exact(velocities.len())
        .map_err(|_error| invalid_lane())?;
    candidate.extend_from_slice(velocities);
    Ok(candidate)
}

fn validate_candidate(candidate: &[Vec2]) -> Result<(), ConstraintSolverError> {
    if candidate.iter().any(|velocity| !velocity.is_valid()) {
        return Err(invalid_lane());
    }
    Ok(())
}

fn rotate(cosine: f32, sine: f32, vector: Vec2) -> Vec2 {
    Vec2::new(
        cosine * vector.x - sine * vector.y,
        sine * vector.x + cosine * vector.y,
    )
}

const fn invalid_lane() -> ConstraintSolverError {
    ConstraintSolverError::Storage(ParticleStorageError::InvalidLaneBundle)
}

#[cfg(test)]
mod tests;

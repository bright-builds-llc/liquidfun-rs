use crate::ParticleFlags;
use crate::identity::ParticleGroupId;
use crate::math::Vec2;
use crate::particle::ParticleGroupFlags;
use crate::particle::storage::group::GroupRecord;
use crate::particle::storage::lanes::ParticlePair;

use super::support::{resource, validate_candidate};
use super::{BoundaryCandidate, BoundaryPass, BoundarySolverError, BoundaryStage};

const BARRIER_COLLISION_TIME: f32 = 2.5;

pub(crate) fn barrier_candidate(
    source: &BoundaryCandidate,
    pairs: &[ParticlePair],
    particle_mass: f32,
    time_step: f32,
    inverse_time_step: f32,
    scan_limit: usize,
) -> Result<BoundaryCandidate, BoundarySolverError> {
    if !particle_mass.is_finite()
        || particle_mass <= 0.0
        || !time_step.is_finite()
        || time_step < 0.0
        || !inverse_time_step.is_finite()
        || inverse_time_step < 0.0
    {
        return Err(BoundarySolverError::InvalidInput);
    }
    let barrier_pairs = pairs
        .iter()
        .filter(|pair| pair.flags.intersects(ParticleFlags::BARRIER))
        .count();
    let required_scans = barrier_pairs
        .checked_mul(source.positions.len())
        .ok_or_else(|| resource("barrier particle scans", scan_limit))?;
    if required_scans > scan_limit {
        return Err(resource("barrier particle scans", scan_limit));
    }
    for pair in pairs {
        pair.validate(source.positions.len())
            .map_err(|_error| BoundarySolverError::InvalidInput)?;
    }

    let mut candidate = source.begin_pass(BoundaryStage::AfterRigidDamping)?;
    for (flags, velocity) in candidate
        .flags
        .iter()
        .copied()
        .zip(&mut candidate.velocities)
    {
        if flags.contains(ParticleFlags::BARRIER | ParticleFlags::WALL) {
            *velocity = Vec2::ZERO;
        }
    }
    for pair in pairs {
        if !pair.flags.intersects(ParticleFlags::BARRIER) {
            continue;
        }
        solve_pair(
            &mut candidate,
            *pair,
            particle_mass,
            time_step,
            inverse_time_step,
        )?;
    }
    candidate.stage = BoundaryStage::AfterBarrier;
    candidate.pass_trace.push(BoundaryPass::Barrier);
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn solve_pair(
    candidate: &mut BoundaryCandidate,
    pair: ParticlePair,
    particle_mass: f32,
    time_step: f32,
    inverse_time_step: f32,
) -> Result<(), BoundarySolverError> {
    let [a, b] = pair.indices.map(|index| index.0);
    let pa = candidate.positions[a];
    let pb = candidate.positions[b];
    let group_a = candidate.memberships[a];
    let group_b = candidate.memberships[b];
    let va = velocity_at(candidate, group_a, a, pa);
    let vb = velocity_at(candidate, group_b, b, pb);
    let pba = pb - pa;
    let vba = vb - va;
    let lower = Vec2::new(pa.x.min(pb.x), pa.y.min(pb.y));
    let upper = Vec2::new(pa.x.max(pb.x), pa.y.max(pb.y));

    for c in 0..candidate.positions.len() {
        let pc = candidate.positions[c];
        if pc.x < lower.x || pc.x > upper.x || pc.y < lower.y || pc.y > upper.y {
            continue;
        }
        let group_c = candidate.memberships[c];
        if group_a == group_c || group_b == group_c {
            continue;
        }
        let vc = velocity_at(candidate, group_c, c, pc);
        let Some(s) = crossing_fraction(
            pba,
            vba,
            pc - pa,
            vc - va,
            BARRIER_COLLISION_TIME * time_step,
        ) else {
            continue;
        };
        let delta_velocity = va + s * vba - vc;
        let force = particle_mass * delta_velocity;
        if let Some(group_index) = rigid_group_index(&candidate.groups, group_c) {
            let statistics = &mut candidate.groups[group_index].statistics;
            if statistics.mass > 0.0 {
                statistics.linear_velocity += (1.0 / statistics.mass) * force;
            }
            if statistics.inertia > 0.0 {
                statistics.angular_velocity +=
                    (pc - statistics.center).cross(force) / statistics.inertia;
            }
        } else {
            candidate.velocities[c] += delta_velocity;
        }
        candidate.forces[c] += -inverse_time_step * force;
        candidate.has_pending_force = true;
        candidate.record_effect(BoundaryPass::Barrier, c, None)?;
    }
    Ok(())
}

fn crossing_fraction(pba: Vec2, vba: Vec2, pca: Vec2, vca: Vec2, maximum_time: f32) -> Option<f32> {
    let e2 = vba.cross(vca);
    let e1 = pba.cross(vca) - pca.cross(vba);
    let e0 = pba.cross(pca);
    if e2 == 0.0 {
        if e1 == 0.0 {
            return None;
        }
        return accepted_fraction(-e0 / e1, pba, vba, pca, vca, maximum_time);
    }
    let determinant = e1 * e1 - 4.0 * e0 * e2;
    if determinant < 0.0 {
        return None;
    }
    let root = determinant.sqrt();
    let mut first = (-e1 - root) / (2.0 * e2);
    let mut second = (-e1 + root) / (2.0 * e2);
    if first > second {
        std::mem::swap(&mut first, &mut second);
    }
    accepted_fraction(first, pba, vba, pca, vca, maximum_time)
        .or_else(|| accepted_fraction(second, pba, vba, pca, vca, maximum_time))
}

fn accepted_fraction(
    time: f32,
    pba: Vec2,
    vba: Vec2,
    pca: Vec2,
    vca: Vec2,
    maximum_time: f32,
) -> Option<f32> {
    if !(time >= 0.0 && time < maximum_time) {
        return None;
    }
    let line = pba + time * vba;
    let point = pca + time * vca;
    let denominator = line.dot(line);
    if denominator == 0.0 {
        return None;
    }
    let fraction = line.dot(point) / denominator;
    (0.0..=1.0).contains(&fraction).then_some(fraction)
}

fn velocity_at(
    candidate: &BoundaryCandidate,
    maybe_group: Option<ParticleGroupId>,
    particle: usize,
    point: Vec2,
) -> Vec2 {
    let Some(group_index) = rigid_group_index(&candidate.groups, maybe_group) else {
        return candidate.velocities[particle];
    };
    let statistics = candidate.groups[group_index].statistics;
    statistics.linear_velocity
        + Vec2::scalar_cross(statistics.angular_velocity, point - statistics.center)
}

fn rigid_group_index(
    groups: &[GroupRecord],
    maybe_group: Option<ParticleGroupId>,
) -> Option<usize> {
    let group_id = maybe_group?;
    groups
        .iter()
        .position(|group| group.id == group_id && group.flags.contains(ParticleGroupFlags::RIGID))
}

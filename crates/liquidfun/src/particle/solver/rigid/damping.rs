use crate::ParticleId;
use crate::identity::{ParticleGroupId, ParticleSystemId};
use crate::math::{Vec2, min};
use crate::particle::ParticleGroupFlags;
use crate::particle::storage::group::{GroupRecord, GroupStatisticsCache};
use crate::particle::storage::lanes::ParticleContact;

use super::support::{
    copy_candidate, resource, statistics_are_finite, validate_body_contact, validate_candidate,
    validate_inputs,
};
use super::{BodyImpulseCandidate, RigidBodyContact, RigidCandidate, RigidSolverError};

#[allow(
    clippy::too_many_arguments,
    reason = "the pure S21 candidate keeps every source lane and bound explicit"
)]
pub(in crate::particle::solver) fn rigid_damping_candidate(
    owner: ParticleSystemId,
    particle_ids: &[ParticleId],
    positions: &[Vec2],
    velocities: &[Vec2],
    memberships: &[Option<ParticleGroupId>],
    groups: &[GroupRecord],
    particle_contacts: &[ParticleContact],
    body_contacts: &[RigidBodyContact],
    particle_mass: f32,
    damping: f32,
    timestamp: u32,
    body_impulse_limit: usize,
) -> Result<RigidCandidate, RigidSolverError> {
    validate_inputs(
        owner,
        particle_ids,
        positions,
        velocities,
        memberships,
        groups,
        particle_mass,
    )?;
    if !damping.is_finite() || damping < 0.0 {
        return Err(RigidSolverError::InvalidInput);
    }

    let mut candidate = copy_candidate(particle_ids, velocities, groups, body_impulse_limit)?;
    refresh_rigid_statistics(
        &mut candidate.groups,
        positions,
        &candidate.velocities,
        particle_mass,
        timestamp,
    )?;
    solve_body_contacts(
        positions,
        memberships,
        body_contacts,
        damping,
        &mut candidate,
        body_impulse_limit,
    )?;
    solve_particle_contacts(
        positions,
        memberships,
        particle_contacts,
        particle_mass,
        damping,
        &mut candidate,
    )?;
    validate_candidate(&candidate)?;
    Ok(candidate)
}

fn refresh_rigid_statistics(
    groups: &mut [GroupRecord],
    positions: &[Vec2],
    velocities: &[Vec2],
    particle_mass: f32,
    timestamp: u32,
) -> Result<(), RigidSolverError> {
    for group in groups {
        if !group.flags.contains(ParticleGroupFlags::RIGID)
            || group.statistics.maybe_source_timestamp == Some(timestamp)
        {
            continue;
        }
        let mut statistics = GroupStatisticsCache {
            maybe_source_timestamp: Some(timestamp),
            ..GroupStatisticsCache::INVALIDATED_ZERO
        };
        for (position, velocity) in positions[group.range()]
            .iter()
            .copied()
            .zip(velocities[group.range()].iter().copied())
        {
            statistics.mass += particle_mass;
            statistics.center += particle_mass * position;
            statistics.linear_velocity += particle_mass * velocity;
        }
        if statistics.mass > 0.0 {
            statistics.center *= 1.0 / statistics.mass;
            statistics.linear_velocity *= 1.0 / statistics.mass;
        }
        for (position, velocity) in positions[group.range()]
            .iter()
            .copied()
            .zip(velocities[group.range()].iter().copied())
        {
            let relative_position = position - statistics.center;
            statistics.inertia += particle_mass * relative_position.length_squared();
            statistics.angular_velocity +=
                particle_mass * relative_position.cross(velocity - statistics.linear_velocity);
        }
        if statistics.inertia > 0.0 {
            statistics.angular_velocity *= 1.0 / statistics.inertia;
        }
        if !statistics_are_finite(statistics) {
            return Err(RigidSolverError::InvalidInput);
        }
        group.statistics = statistics;
    }
    Ok(())
}

fn solve_body_contacts(
    positions: &[Vec2],
    memberships: &[Option<ParticleGroupId>],
    contacts: &[RigidBodyContact],
    damping: f32,
    candidate: &mut RigidCandidate,
    body_impulse_limit: usize,
) -> Result<(), RigidSolverError> {
    for contact in contacts {
        validate_body_contact(*contact, positions.len())?;
        let Some(group_id) = memberships[contact.particle] else {
            continue;
        };
        let Some(group) = candidate
            .groups
            .iter_mut()
            .find(|group| group.id == group_id && group.flags.contains(ParticleGroupFlags::RIGID))
        else {
            continue;
        };
        let point = positions[contact.particle];
        let group_velocity = velocity_at(
            group.statistics.linear_velocity,
            group.statistics.angular_velocity,
            group.statistics.center,
            point,
        );
        let body_velocity = velocity_at(
            contact.body_linear_velocity,
            contact.body_angular_velocity,
            contact.body_center,
            point,
        );
        let normal_velocity = (body_velocity - group_velocity).dot(contact.normal);
        if normal_velocity >= 0.0 {
            continue;
        }
        let group_parameter = damping_parameter(
            group.statistics.mass,
            group.statistics.inertia,
            group.statistics.center,
            point,
            contact.normal,
        );
        let body_parameter = damping_parameter(
            contact.body_mass,
            contact.body_inertia,
            contact.body_center,
            point,
            contact.normal,
        );
        let impulse = damping
            * min(contact.weight, 1.0)
            * damping_impulse(group_parameter, body_parameter, normal_velocity);
        apply_group_damping(group, group_parameter, impulse, contact.normal);
        if candidate.body_impulses.len() == body_impulse_limit {
            return Err(resource(
                "rigid body impulse candidates",
                body_impulse_limit,
            ));
        }
        candidate.body_impulses.push(BodyImpulseCandidate {
            body: contact.body,
            impulse: -impulse * contact.normal,
            point,
        });
    }
    Ok(())
}

fn solve_particle_contacts(
    positions: &[Vec2],
    memberships: &[Option<ParticleGroupId>],
    contacts: &[ParticleContact],
    particle_mass: f32,
    damping: f32,
    candidate: &mut RigidCandidate,
) -> Result<(), RigidSolverError> {
    for contact in contacts {
        let [a, b] = contact.indices.map(|index| index.0);
        if a >= positions.len()
            || b >= positions.len()
            || a == b
            || !contact.weight.is_finite()
            || !contact.normal.is_valid()
        {
            return Err(RigidSolverError::InvalidInput);
        }
        let group_a = memberships[a];
        let group_b = memberships[b];
        if group_a == group_b {
            continue;
        }
        let rigid_a = rigid_group_index(&candidate.groups, group_a);
        let rigid_b = rigid_group_index(&candidate.groups, group_b);
        if rigid_a.is_none() && rigid_b.is_none() {
            continue;
        }
        let point = 0.5 * (positions[a] + positions[b]);
        let velocity_a = particle_or_group_velocity(candidate, rigid_a, a, point);
        let velocity_b = particle_or_group_velocity(candidate, rigid_b, b, point);
        let normal_velocity = (velocity_b - velocity_a).dot(contact.normal);
        if normal_velocity >= 0.0 {
            continue;
        }
        let parameter_a =
            particle_or_group_parameter(candidate, rigid_a, particle_mass, point, contact.normal);
        let parameter_b =
            particle_or_group_parameter(candidate, rigid_b, particle_mass, point, contact.normal);
        let impulse =
            damping * contact.weight * damping_impulse(parameter_a, parameter_b, normal_velocity);
        apply_particle_or_group(candidate, rigid_a, a, parameter_a, impulse, contact.normal);
        apply_particle_or_group(candidate, rigid_b, b, parameter_b, -impulse, contact.normal);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct DampingParameter {
    inverse_mass: f32,
    inverse_inertia: f32,
    tangent_distance: f32,
}

fn damping_parameter(
    mass: f32,
    inertia: f32,
    center: Vec2,
    point: Vec2,
    normal: Vec2,
) -> DampingParameter {
    DampingParameter {
        inverse_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
        inverse_inertia: if inertia > 0.0 { 1.0 / inertia } else { 0.0 },
        tangent_distance: (point - center).cross(normal),
    }
}

fn damping_impulse(a: DampingParameter, b: DampingParameter, normal_velocity: f32) -> f32 {
    let inverse_mass = a.inverse_mass
        + a.inverse_inertia * a.tangent_distance * a.tangent_distance
        + b.inverse_mass
        + b.inverse_inertia * b.tangent_distance * b.tangent_distance;
    if inverse_mass > 0.0 {
        normal_velocity / inverse_mass
    } else {
        0.0
    }
}

fn particle_or_group_parameter(
    candidate: &RigidCandidate,
    maybe_group_index: Option<usize>,
    particle_mass: f32,
    point: Vec2,
    normal: Vec2,
) -> DampingParameter {
    let Some(group_index) = maybe_group_index else {
        return damping_parameter(particle_mass, 0.0, point, point, normal);
    };
    let statistics = candidate.groups[group_index].statistics;
    damping_parameter(
        statistics.mass,
        statistics.inertia,
        statistics.center,
        point,
        normal,
    )
}

fn particle_or_group_velocity(
    candidate: &RigidCandidate,
    maybe_group_index: Option<usize>,
    particle: usize,
    point: Vec2,
) -> Vec2 {
    let Some(group_index) = maybe_group_index else {
        return candidate.velocities[particle];
    };
    let statistics = candidate.groups[group_index].statistics;
    velocity_at(
        statistics.linear_velocity,
        statistics.angular_velocity,
        statistics.center,
        point,
    )
}

fn apply_particle_or_group(
    candidate: &mut RigidCandidate,
    maybe_group_index: Option<usize>,
    particle: usize,
    parameter: DampingParameter,
    impulse: f32,
    normal: Vec2,
) {
    let Some(group_index) = maybe_group_index else {
        candidate.velocities[particle] += impulse * parameter.inverse_mass * normal;
        return;
    };
    apply_group_damping(
        &mut candidate.groups[group_index],
        parameter,
        impulse,
        normal,
    );
}

fn apply_group_damping(
    group: &mut GroupRecord,
    parameter: DampingParameter,
    impulse: f32,
    normal: Vec2,
) {
    group.statistics.linear_velocity += impulse * parameter.inverse_mass * normal;
    group.statistics.angular_velocity +=
        impulse * parameter.tangent_distance * parameter.inverse_inertia;
}

fn velocity_at(linear: Vec2, angular: f32, center: Vec2, point: Vec2) -> Vec2 {
    linear + Vec2::scalar_cross(angular, point - center)
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

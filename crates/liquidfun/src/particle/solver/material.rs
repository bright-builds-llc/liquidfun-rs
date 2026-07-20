//! Exact S07-S12 particle material kernels.

use crate::identity::BodyId;
use crate::math::{Vec2, min, settings};
use crate::particle::definition::ParticleSystemDef;
use crate::particle::storage::{ParticleStorage, ParticleStorageError};
use crate::particle::{ParticleColor, ParticleFlags, ParticleGroupFlags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialSolverError {
    Storage(ParticleStorageError),
    MissingBody(BodyId),
}

impl From<ParticleStorageError> for MaterialSolverError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

pub(crate) trait MaterialBodyCoupling {
    fn contains_body(&self, body: BodyId) -> bool;
    fn velocity_at(&self, body: BodyId, point: Vec2) -> Vec2;
    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, point: Vec2);
}

/// S07 `Viscous`: transfer relative velocity in body-contact then particle-contact order.
pub(crate) fn viscous<B: MaterialBodyCoupling>(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    bodies: &mut B,
) -> Result<(), MaterialSolverError> {
    for contact in storage.body_contacts() {
        if !bodies.contains_body(contact.body) {
            return Err(MaterialSolverError::MissingBody(contact.body));
        }
    }

    let strength = definition.viscous_strength();
    let inverse_mass = particle_inverse_mass(definition);
    let positions = storage.positions();
    let flags = storage.flags();
    let body_contacts = storage.body_contacts();
    let particle_contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in body_contacts {
        let particle = contact.index.0;
        if flags[particle].contains(ParticleFlags::VISCOUS) {
            let position = positions[particle];
            let relative_velocity =
                bodies.velocity_at(contact.body, position) - velocities[particle];
            let impulse = strength * contact.mass * contact.weight * relative_velocity;
            velocities[particle] += inverse_mass * impulse;
            bodies.apply_linear_impulse(contact.body, -impulse, position);
        }
    }
    for contact in particle_contacts {
        if contact.flags.intersects(ParticleFlags::VISCOUS) {
            let [a, b] = contact.indices;
            let relative_velocity = velocities[b.0] - velocities[a.0];
            let impulse = strength * contact.weight * relative_velocity;
            velocities[a.0] += impulse;
            velocities[b.0] -= impulse;
        }
    }

    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

/// S08 `Repulsive`: repel flagged contacts whose particles have distinct memberships.
pub(crate) fn repulsive(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ParticleStorageError> {
    let strength =
        definition.repulsive_strength() * critical_velocity(definition, inverse_time_step);
    let groups = storage.groups();
    let contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in contacts {
        if contact.flags.intersects(ParticleFlags::REPULSIVE) {
            let [a, b] = contact.indices;
            if groups[a.0] != groups[b.0] {
                let impulse = strength * contact.weight * contact.normal;
                velocities[a.0] -= impulse;
                velocities[b.0] += impulse;
            }
        }
    }

    storage.replace_solver_velocities(velocities)
}

/// S09 `Powder`: scatter flagged contacts above the pinned stride threshold.
pub(crate) fn powder(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ParticleStorageError> {
    let strength = definition.powder_strength() * critical_velocity(definition, inverse_time_step);
    let minimum_weight = 1.0 - settings::PARTICLE_STRIDE;
    let contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in contacts {
        if contact.flags.intersects(ParticleFlags::POWDER) && contact.weight > minimum_weight {
            let [a, b] = contact.indices;
            let impulse = strength * (contact.weight - minimum_weight) * contact.normal;
            velocities[a.0] -= impulse;
            velocities[b.0] += impulse;
        }
    }

    storage.replace_solver_velocities(velocities)
}

/// S10 `Tensile`: accumulate weighted normals before applying surface tension.
pub(crate) fn tensile(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ParticleStorageError> {
    if !storage
        .flags()
        .iter()
        .any(|flags| flags.contains(ParticleFlags::TENSILE))
    {
        return Ok(());
    }

    storage.ensure_tensile_accumulations()?;
    let contacts = storage.particle_contacts();
    let mut accumulations = vec![Vec2::ZERO; storage.len()];
    for contact in contacts {
        if contact.flags.intersects(ParticleFlags::TENSILE) {
            let [a, b] = contact.indices;
            let weighted_normal = (1.0 - contact.weight) * contact.weight * contact.normal;
            accumulations[a.0] -= weighted_normal;
            accumulations[b.0] += weighted_normal;
        }
    }
    storage.replace_tensile_accumulations(accumulations)?;

    let critical_velocity = critical_velocity(definition, inverse_time_step);
    let pressure_strength = definition.surface_tension_pressure_strength() * critical_velocity;
    let normal_strength = definition.surface_tension_normal_strength() * critical_velocity;
    let maximum_velocity_variation = settings::MAX_PARTICLE_FORCE * critical_velocity;
    let accumulations = storage
        .maybe_tensile_accumulations()
        .expect("tensile gate allocated aligned scratch");
    let weights = storage.weights();
    let contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in contacts {
        if contact.flags.intersects(ParticleFlags::TENSILE) {
            let [a, b] = contact.indices;
            let combined_weight = weights[a.0] + weights[b.0];
            let normal_difference = accumulations[b.0] - accumulations[a.0];
            let normal_force = min(
                pressure_strength * (combined_weight - 2.0)
                    + normal_strength * normal_difference.dot(contact.normal),
                maximum_velocity_variation,
            ) * contact.weight;
            let impulse = normal_force * contact.normal;
            velocities[a.0] -= impulse;
            velocities[b.0] += impulse;
        }
    }

    storage.replace_solver_velocities(velocities)
}

/// S11 `Solid`: eject cross-group contacts using the authoritative depth lane.
pub(crate) fn solid(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ParticleStorageError> {
    if !storage
        .group_flags()
        .any(|flags| flags.contains(ParticleGroupFlags::SOLID))
    {
        return Ok(());
    }

    let depths = storage
        .maybe_depths()
        .ok_or(ParticleStorageError::InvalidLaneBundle)?;
    let strength = inverse_time_step * definition.ejection_strength();
    let groups = storage.groups();
    let contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in contacts {
        let [a, b] = contact.indices;
        if groups[a.0] != groups[b.0] {
            let depth = depths[a.0] + depths[b.0];
            let impulse = strength * depth * contact.weight * contact.normal;
            velocities[a.0] -= impulse;
            velocities[b.0] += impulse;
        }
    }

    storage.replace_solver_velocities(velocities)
}

/// S12 `ColorMixing`: mix channels only when both particles carry the flag.
pub(crate) fn color_mixing(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
) -> Result<(), ParticleStorageError> {
    if !storage
        .flags()
        .iter()
        .any(|flags| flags.contains(ParticleFlags::COLOR_MIXING))
    {
        return Ok(());
    }

    #[allow(clippy::cast_possible_truncation)] // Match upstream's explicit float-to-int conversion.
    let color_mixing_128 = (128.0 * definition.color_mixing_strength()) as i32;
    if color_mixing_128 == 0 {
        return Ok(());
    }
    let Some(colors) = storage.maybe_colors() else {
        return Ok(());
    };
    let flags = storage.flags();
    let contacts = storage.particle_contacts();
    let mut colors = colors.to_vec();

    for contact in contacts {
        let [a, b] = contact.indices;
        if flags[a.0].contains(ParticleFlags::COLOR_MIXING)
            && flags[b.0].contains(ParticleFlags::COLOR_MIXING)
        {
            let [red_a, green_a, blue_a, alpha_a] = colors[a.0].components();
            let [red_b, green_b, blue_b, alpha_b] = colors[b.0].components();
            let (red_a, red_b) = mix_channel(red_a, red_b, color_mixing_128);
            let (green_a, green_b) = mix_channel(green_a, green_b, color_mixing_128);
            let (blue_a, blue_b) = mix_channel(blue_a, blue_b, color_mixing_128);
            let (alpha_a, alpha_b) = mix_channel(alpha_a, alpha_b, color_mixing_128);
            colors[a.0] = ParticleColor::new(red_a, green_a, blue_a, alpha_a);
            colors[b.0] = ParticleColor::new(red_b, green_b, blue_b, alpha_b);
        }
    }

    storage.replace_solver_colors(colors)
}

fn mix_channel(first: u8, second: u8, strength: i32) -> (u8, u8) {
    let difference = i64::from(second) - i64::from(first);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // Preserve upstream uint8 wrap.
    let delta = ((i64::from(strength) * difference) >> 8) as u8;
    (first.wrapping_add(delta), second.wrapping_sub(delta))
}

fn critical_velocity(definition: ParticleSystemDef, inverse_time_step: f32) -> f32 {
    (2.0 * definition.radius()) * inverse_time_step
}

fn particle_inverse_mass(definition: ParticleSystemDef) -> f32 {
    let inverse_diameter = 1.0 / (2.0 * definition.radius());
    let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
    (1.0 / definition.density()) * inverse_stride * inverse_stride
}

#[cfg(test)]
#[path = "material/tests.rs"]
mod tests;

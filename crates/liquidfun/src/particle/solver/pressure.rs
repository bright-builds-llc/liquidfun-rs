//! Exact S14-S17 particle pressure and damping kernels.

use crate::identity::BodyId;
use crate::math::{Vec2, max, min, settings};
use crate::particle::ParticleFlags;
use crate::particle::definition::ParticleSystemDef;
use crate::particle::storage::{ParticleStorage, ParticleStorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PressureSolverError {
    Storage(ParticleStorageError),
    MissingBody(BodyId),
}

impl From<ParticleStorageError> for PressureSolverError {
    fn from(error: ParticleStorageError) -> Self {
        Self::Storage(error)
    }
}

pub(crate) trait BodyCoupling {
    fn contains_body(&self, body: BodyId) -> bool;
    fn velocity_at(&self, body: BodyId, point: Vec2) -> Vec2;
    fn apply_linear_impulse(&mut self, body: BodyId, impulse: Vec2, point: Vec2);
}

pub(crate) fn static_pressure(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
) -> Result<(), ParticleStorageError> {
    storage.ensure_static_pressures()?;
    let critical_pressure = critical_pressure(definition, inverse_time_step);
    let pressure_per_weight = definition.static_pressure_strength() * critical_pressure;
    let max_pressure = settings::MAX_PARTICLE_PRESSURE * critical_pressure;
    let relaxation = definition.static_pressure_relaxation();
    let weights = storage.weights();
    let flags = storage.flags();
    let contacts = storage.particle_contacts();
    let mut pressures = storage
        .maybe_static_pressures()
        .expect("static-pressure gate allocated aligned scratch")
        .to_vec();
    let mut accumulation = vec![0.0; storage.len()];

    for _ in 0..definition.static_pressure_iterations() {
        accumulation.fill(0.0);
        for contact in contacts {
            if contact.flags.intersects(ParticleFlags::STATIC_PRESSURE) {
                let [a, b] = contact.indices;
                let weight = contact.weight;
                accumulation[a.0] += weight * pressures[b.0];
                accumulation[b.0] += weight * pressures[a.0];
            }
        }
        for index in 0..storage.len() {
            let weight = weights[index];
            if flags[index].contains(ParticleFlags::STATIC_PRESSURE) {
                let weighted_pressure = accumulation[index];
                let pressure = (weighted_pressure
                    + pressure_per_weight * (weight - settings::MIN_PARTICLE_WEIGHT))
                    / (weight + relaxation);
                pressures[index] = max(0.0, min(pressure, max_pressure));
            } else {
                pressures[index] = 0.0;
            }
        }
    }

    storage.replace_static_pressures(pressures)
}

pub(crate) fn pressure<B: BodyCoupling>(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    time_step: f32,
    inverse_time_step: f32,
    bodies: &mut B,
) -> Result<(), PressureSolverError> {
    validate_contact_bodies(storage, bodies)?;
    let critical_pressure = critical_pressure(definition, inverse_time_step);
    let pressure_per_weight = definition.pressure_strength() * critical_pressure;
    let max_pressure = settings::MAX_PARTICLE_PRESSURE * critical_pressure;
    let mut accumulation = Vec::with_capacity(storage.len());
    for weight in storage.weights().iter().copied() {
        let pressure = pressure_per_weight * max(0.0, weight - settings::MIN_PARTICLE_WEIGHT);
        accumulation.push(min(pressure, max_pressure));
    }
    if storage
        .flags()
        .iter()
        .copied()
        .any(|flags| flags.intersects(ParticleFlags::POWDER | ParticleFlags::TENSILE))
    {
        for (index, flags) in storage.flags().iter().copied().enumerate() {
            if flags.intersects(ParticleFlags::POWDER | ParticleFlags::TENSILE) {
                accumulation[index] = 0.0;
            }
        }
    }
    if storage
        .flags()
        .iter()
        .copied()
        .any(|flags| flags.contains(ParticleFlags::STATIC_PRESSURE))
    {
        let static_pressures = storage
            .maybe_static_pressures()
            .ok_or(ParticleStorageError::InvalidLaneBundle)?;
        for (index, flags) in storage.flags().iter().copied().enumerate() {
            if flags.contains(ParticleFlags::STATIC_PRESSURE) {
                accumulation[index] += static_pressures[index];
            }
        }
    }

    let diameter = particle_diameter(definition);
    let velocity_per_pressure = time_step / (definition.density() * diameter);
    let inverse_mass = particle_inverse_mass(definition);
    let positions = storage.positions();
    let body_contacts = storage.body_contacts();
    let particle_contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();
    for contact in body_contacts {
        let particle = contact.index.0;
        let pressure = accumulation[particle] + pressure_per_weight * contact.weight;
        let impulse =
            velocity_per_pressure * contact.weight * contact.mass * pressure * contact.normal;
        velocities[particle] -= inverse_mass * impulse;
        bodies.apply_linear_impulse(contact.body, impulse, positions[particle]);
    }
    for contact in particle_contacts {
        let [a, b] = contact.indices;
        let pressure = accumulation[a.0] + accumulation[b.0];
        let impulse = velocity_per_pressure * contact.weight * pressure * contact.normal;
        velocities[a.0] -= impulse;
        velocities[b.0] += impulse;
    }
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

pub(crate) fn damping<B: BodyCoupling>(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    inverse_time_step: f32,
    bodies: &mut B,
) -> Result<(), PressureSolverError> {
    validate_contact_bodies(storage, bodies)?;
    let linear_damping = definition.damping();
    let quadratic_damping = 1.0 / critical_velocity(definition, inverse_time_step);
    let inverse_mass = particle_inverse_mass(definition);
    let positions = storage.positions();
    let body_contacts = storage.body_contacts();
    let particle_contacts = storage.particle_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in body_contacts {
        let particle = contact.index.0;
        let position = positions[particle];
        let relative_velocity = bodies.velocity_at(contact.body, position) - velocities[particle];
        let normal_velocity = relative_velocity.dot(contact.normal);
        if normal_velocity < 0.0 {
            let damping = max(
                linear_damping * contact.weight,
                min(-quadratic_damping * normal_velocity, 0.5),
            );
            let impulse = damping * contact.mass * normal_velocity * contact.normal;
            velocities[particle] += inverse_mass * impulse;
            bodies.apply_linear_impulse(contact.body, -impulse, position);
        }
    }
    for contact in particle_contacts {
        let [a, b] = contact.indices;
        let relative_velocity = velocities[b.0] - velocities[a.0];
        let normal_velocity = relative_velocity.dot(contact.normal);
        if normal_velocity < 0.0 {
            let damping = max(
                linear_damping * contact.weight,
                min(-quadratic_damping * normal_velocity, 0.5),
            );
            let impulse = damping * normal_velocity * contact.normal;
            velocities[a.0] += impulse;
            velocities[b.0] -= impulse;
        }
    }
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

pub(crate) fn extra_damping<B: BodyCoupling>(
    storage: &mut ParticleStorage,
    definition: ParticleSystemDef,
    bodies: &mut B,
) -> Result<(), PressureSolverError> {
    validate_contact_bodies(storage, bodies)?;
    let inverse_mass = particle_inverse_mass(definition);
    let flags = storage.flags();
    let positions = storage.positions();
    let contacts = storage.body_contacts();
    let mut velocities = storage.velocities().to_vec();

    for contact in contacts {
        let particle = contact.index.0;
        if flags[particle].contains(ParticleFlags::STATIC_PRESSURE) {
            let position = positions[particle];
            let relative_velocity =
                bodies.velocity_at(contact.body, position) - velocities[particle];
            let normal_velocity = relative_velocity.dot(contact.normal);
            if normal_velocity < 0.0 {
                let impulse = 0.5 * contact.mass * normal_velocity * contact.normal;
                velocities[particle] += inverse_mass * impulse;
                bodies.apply_linear_impulse(contact.body, -impulse, position);
            }
        }
    }
    storage.replace_solver_velocities(velocities)?;
    Ok(())
}

fn validate_contact_bodies<B: BodyCoupling>(
    storage: &ParticleStorage,
    bodies: &B,
) -> Result<(), PressureSolverError> {
    for contact in storage.body_contacts() {
        if !bodies.contains_body(contact.body) {
            return Err(PressureSolverError::MissingBody(contact.body));
        }
    }
    Ok(())
}

fn critical_velocity(definition: ParticleSystemDef, inverse_time_step: f32) -> f32 {
    particle_diameter(definition) * inverse_time_step
}

fn critical_pressure(definition: ParticleSystemDef, inverse_time_step: f32) -> f32 {
    let velocity = critical_velocity(definition, inverse_time_step);
    definition.density() * (velocity * velocity)
}

fn particle_diameter(definition: ParticleSystemDef) -> f32 {
    2.0 * definition.radius()
}

fn particle_inverse_mass(definition: ParticleSystemDef) -> f32 {
    let inverse_diameter = 1.0 / particle_diameter(definition);
    let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
    let inverse_density = 1.0 / definition.density();
    inverse_density * inverse_stride * inverse_stride
}

#[cfg(test)]
#[path = "pressure/tests.rs"]
mod tests;

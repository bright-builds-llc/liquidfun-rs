//! Converts per-particle accelerations into one fixed step of particle force.
//!
//! The engine consumes accumulated particle force once per world step, in the
//! first particle substep (`dt / iterations`), so `F = m * a * iterations` adds
//! exactly `a * dt` of velocity per step. The particle mass is
//! `density * (PARTICLE_STRIDE * 2r)^2`, read from the live system definition.
//!
//! The helper is scene-agnostic: callers supply the acceleration, built from
//! [`GravityFrame`] so every scripted pull follows the live gravity vector.

use liquidfun::math::Vec2;
use liquidfun::math::settings::PARTICLE_STRIDE;
use liquidfun::{ParticleGroupId, ParticleId, ParticleSystemId, World};

use crate::session::SessionError;

/// Shortest gravity vector that still defines a direction.
const MIN_GRAVITY_MAGNITUDE: f32 = 1e-4;

/// Gravity magnitude and the field axes built from `world.gravity()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GravityFrame {
    pub(crate) magnitude: f32,
    /// Unit vector opposite to gravity.
    pub(crate) up: Vec2,
    /// `up` turned a quarter turn clockwise: `(up.y, -up.x)`.
    pub(crate) across: Vec2,
}

impl GravityFrame {
    /// The frame for `gravity`, or `None` when gravity is non-finite or too
    /// small to define a direction.
    pub(crate) fn from_gravity(gravity: Vec2) -> Option<Self> {
        if !gravity.x.is_finite() || !gravity.y.is_finite() {
            return None;
        }
        let magnitude = gravity.length();
        if magnitude <= MIN_GRAVITY_MAGNITUDE {
            return None;
        }
        let up = gravity * (-1.0 / magnitude);
        Some(Self {
            magnitude,
            up,
            across: Vec2::new(up.y, -up.x),
        })
    }
}

/// Force per unit acceleration for one particle: mass times particle iterations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ForceScale(f32);

impl ForceScale {
    /// Reads radius and density from the live system definition.
    ///
    /// # Errors
    ///
    /// `SceneConstruction` when the system is stale, the iteration count does
    /// not fit, or the product is not finite and positive.
    pub(crate) fn for_system(
        world: &World,
        system: ParticleSystemId,
        particle_iterations: u32,
    ) -> Result<Self, SessionError> {
        let definition = world
            .particle_system_snapshot(system)
            .map_err(|_error| SessionError::SceneConstruction)?
            .definition();
        let iterations = f32::from(
            u16::try_from(particle_iterations).map_err(|_error| SessionError::SceneConstruction)?,
        );
        let stride = PARTICLE_STRIDE * 2.0 * definition.radius();
        let value = definition.density() * stride * stride * iterations;
        if !value.is_finite() || value <= 0.0 {
            return Err(SessionError::SceneConstruction);
        }
        Ok(Self(value))
    }

    /// The per-particle force that adds `acceleration * dt` in one step.
    pub(crate) fn force(self, acceleration: Vec2) -> Vec2 {
        acceleration * self.0
    }
}

/// Reusable copies of the particle lanes, so a per-step caller reuses these
/// buffers after the first step. Each force application still allocates in
/// the engine, once per affected particle.
#[derive(Debug, Default)]
pub(crate) struct ParticleScratch {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    ids: Vec<ParticleId>,
}

/// Applies `acceleration(position, velocity)` to every particle of `system`
/// for one fixed step. Zero results are skipped.
///
/// # Errors
///
/// `StepFailed` when the system view or a force application fails. Forces
/// already applied to earlier particles in this call are not rolled back.
pub(crate) fn apply_accelerations(
    world: &mut World,
    system: ParticleSystemId,
    scale: ForceScale,
    scratch: &mut ParticleScratch,
    mut acceleration: impl FnMut(Vec2, Vec2) -> Vec2,
) -> Result<(), SessionError> {
    {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::StepFailed)?;
        scratch.positions.clear();
        scratch.positions.extend_from_slice(view.positions());
        scratch.velocities.clear();
        scratch.velocities.extend_from_slice(view.velocities());
        scratch.ids.clear();
        scratch.ids.extend_from_slice(view.particle_ids());
    }
    let lanes = scratch
        .positions
        .iter()
        .zip(&scratch.velocities)
        .zip(&scratch.ids);
    for ((position, velocity), id) in lanes {
        let particle_acceleration = acceleration(*position, *velocity);
        if particle_acceleration == Vec2::ZERO {
            continue;
        }
        world
            .apply_particle_force_range(
                system,
                std::slice::from_ref(id),
                scale.force(particle_acceleration),
            )
            .map_err(|_error| SessionError::StepFailed)?;
    }
    Ok(())
}

/// Adds one acceleration to every member of `group` with a single range call.
///
/// The range call divides its total over the members, so the total is the
/// per-particle force times the member count.
///
/// # Errors
///
/// `StepFailed` when the group view or the force application fails, or when
/// the group has more than `u16::MAX` (65,535) members, the most whose count
/// converts to `f32` without a lossy cast.
pub(crate) fn apply_group_acceleration(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
    scale: ForceScale,
    acceleration: Vec2,
) -> Result<(), SessionError> {
    let members = world
        .particle_group_view(group)
        .map_err(|_error| SessionError::StepFailed)?
        .member_ids()
        .to_vec();
    if members.is_empty() {
        return Ok(());
    }
    let count = f32::from(u16::try_from(members.len()).map_err(|_error| SessionError::StepFailed)?);
    world
        .apply_particle_force_range(system, &members, scale.force(acceleration) * count)
        .map_err(|_error| SessionError::StepFailed)
}

#[cfg(test)]
mod tests;

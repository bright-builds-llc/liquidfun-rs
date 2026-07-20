//! Exact S21 rigid damping and S24 rigid projection candidates.

mod damping;
mod projection;
mod support;

use crate::ParticleId;
use crate::identity::BodyId;
use crate::math::Vec2;
use crate::particle::storage::group::GroupRecord;

#[allow(
    unused_imports,
    reason = "Plan 10-22 consumes the closed S21 and S24 kernel surface"
)]
pub(crate) use damping::rigid_damping_candidate;
#[allow(
    unused_imports,
    reason = "Plan 10-22 consumes the closed S21 and S24 kernel surface"
)]
pub(crate) use projection::rigid_projection_candidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RigidSolverError {
    InvalidInput,
    ResourceLimit {
        resource: &'static str,
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RigidBodyContact {
    pub(crate) particle: usize,
    pub(crate) body: BodyId,
    pub(crate) weight: f32,
    pub(crate) normal: Vec2,
    pub(crate) body_mass: f32,
    pub(crate) body_inertia: f32,
    pub(crate) body_center: Vec2,
    pub(crate) body_linear_velocity: Vec2,
    pub(crate) body_angular_velocity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BodyImpulseCandidate {
    pub(crate) body: BodyId,
    pub(crate) impulse: Vec2,
    pub(crate) point: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RigidCandidate {
    pub(crate) particle_ids: Vec<ParticleId>,
    pub(crate) velocities: Vec<Vec2>,
    pub(crate) groups: Vec<GroupRecord>,
    pub(crate) body_impulses: Vec<BodyImpulseCandidate>,
}

#[cfg(test)]
mod tests;

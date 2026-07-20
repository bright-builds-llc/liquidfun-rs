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
pub(super) use damping::rigid_damping_candidate;
#[allow(
    unused_imports,
    reason = "Plan 10-22 consumes the closed S21 and S24 kernel surface"
)]
pub(super) use projection::rigid_projection_candidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RigidSolverError {
    InvalidInput,
    ResourceLimit {
        resource: &'static str,
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RigidBodyContact {
    pub(super) particle: usize,
    pub(super) body: BodyId,
    pub(super) weight: f32,
    pub(super) normal: Vec2,
    pub(super) body_mass: f32,
    pub(super) body_inertia: f32,
    pub(super) body_center: Vec2,
    pub(super) body_linear_velocity: Vec2,
    pub(super) body_angular_velocity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BodyImpulseCandidate {
    pub(super) body: BodyId,
    pub(super) impulse: Vec2,
    pub(super) point: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RigidCandidate {
    pub(super) particle_ids: Vec<ParticleId>,
    pub(super) velocities: Vec<Vec2>,
    pub(super) groups: Vec<GroupRecord>,
    pub(super) body_impulses: Vec<BodyImpulseCandidate>,
}

#[cfg(test)]
mod tests;

//! Exact S22-S23 and S25-S26 transactional solver-tail candidates.

mod barrier;
mod collision;
mod support;

use crate::identity::{BodyId, ParticleGroupId, ParticleSystemId};
use crate::math::{Transform, Vec2};
use crate::particle::storage::group::GroupRecord;
use crate::{ParticleFlags, ParticleId};

#[allow(
    unused_imports,
    reason = "Plan 10-22 consumes the closed S22 boundary kernel surface"
)]
pub(crate) use barrier::barrier_candidate;
#[allow(
    unused_imports,
    reason = "Plan 10-22 consumes the closed S23/S25/S26 boundary kernel surface"
)]
pub(crate) use collision::{
    collision_candidate, collision_start_from_previous_transform, integrate_candidate,
    mark_rigid_projection, wall_candidate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundarySolverError {
    InvalidInput,
    ReorderedPass {
        expected: BoundaryStage,
        actual: BoundaryStage,
    },
    ResourceLimit {
        resource: &'static str,
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundaryStage {
    AfterRigidDamping,
    AfterBarrier,
    AfterCollision,
    AfterRigidProjection,
    AfterWall,
    Integrated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundaryPass {
    Barrier,
    Collision,
    Rigid,
    Wall,
    Integrate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BoundaryEffect {
    pub(super) pass: BoundaryPass,
    pub(super) particle: ParticleId,
    pub(super) maybe_body: Option<BodyId>,
}

/// One fixture ray hit already admitted by the existing world query and filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FilteredCollisionHit {
    pub(crate) particle: usize,
    pub(crate) body: BodyId,
    pub(crate) previous_transform: Transform,
    pub(crate) current_transform: Transform,
    pub(crate) body_local_center: Vec2,
    pub(crate) is_circle: bool,
    pub(crate) fraction: f32,
    pub(crate) normal: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundaryCandidate {
    pub(crate) owner: ParticleSystemId,
    pub(crate) particle_ids: Vec<ParticleId>,
    pub(crate) positions: Vec<Vec2>,
    pub(crate) velocities: Vec<Vec2>,
    pub(crate) forces: Vec<Vec2>,
    pub(crate) flags: Vec<ParticleFlags>,
    pub(crate) memberships: Vec<Option<ParticleGroupId>>,
    pub(crate) groups: Vec<GroupRecord>,
    pub(crate) stage: BoundaryStage,
    pub(crate) has_pending_force: bool,
    pub(crate) pass_trace: Vec<BoundaryPass>,
    pub(crate) effects: Vec<BoundaryEffect>,
    effect_limit: usize,
}

impl BoundaryCandidate {
    #[allow(
        clippy::too_many_arguments,
        reason = "the solver-tail candidate validates every aligned authoritative lane"
    )]
    pub(crate) fn new(
        owner: ParticleSystemId,
        particle_ids: &[ParticleId],
        positions: &[Vec2],
        velocities: &[Vec2],
        forces: &[Vec2],
        flags: &[ParticleFlags],
        memberships: &[Option<ParticleGroupId>],
        groups: &[GroupRecord],
        has_pending_force: bool,
        effect_limit: usize,
    ) -> Result<Self, BoundarySolverError> {
        support::validate_source_lanes(
            owner,
            particle_ids,
            positions,
            velocities,
            forces,
            flags,
            memberships,
            groups,
        )?;
        Ok(Self {
            owner,
            particle_ids: support::copy_slice(particle_ids, "boundary particle identities")?,
            positions: support::copy_slice(positions, "boundary position candidates")?,
            velocities: support::copy_slice(velocities, "boundary velocity candidates")?,
            forces: support::copy_slice(forces, "boundary force candidates")?,
            flags: support::copy_slice(flags, "boundary flag candidates")?,
            memberships: support::copy_slice(memberships, "boundary membership candidates")?,
            groups: support::copy_slice(groups, "boundary group candidates")?,
            stage: BoundaryStage::AfterRigidDamping,
            has_pending_force,
            pass_trace: Vec::with_capacity(5),
            effects: Vec::with_capacity(effect_limit),
            effect_limit,
        })
    }

    fn begin_pass(&self, expected: BoundaryStage) -> Result<Self, BoundarySolverError> {
        if self.stage != expected {
            return Err(BoundarySolverError::ReorderedPass {
                expected,
                actual: self.stage,
            });
        }
        Ok(self.clone())
    }

    fn record_effect(
        &mut self,
        pass: BoundaryPass,
        particle: usize,
        maybe_body: Option<BodyId>,
    ) -> Result<(), BoundarySolverError> {
        if particle >= self.particle_ids.len() {
            return Err(BoundarySolverError::InvalidInput);
        }
        if self.effects.len() == self.effect_limit {
            return Err(support::resource(
                "boundary effect journal",
                self.effect_limit,
            ));
        }
        self.effects.push(BoundaryEffect {
            pass,
            particle: self.particle_ids[particle],
            maybe_body,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests;

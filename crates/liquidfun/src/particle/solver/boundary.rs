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

#[derive(Default)]
pub(crate) struct BoundaryBuffers {
    particle_ids: Vec<ParticleId>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    forces: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    memberships: Vec<Option<ParticleGroupId>>,
    groups: Vec<GroupRecord>,
    pass_trace: Vec<BoundaryPass>,
    effects: Vec<BoundaryEffect>,
}

impl BoundaryBuffers {
    pub(crate) fn clear(&mut self) {
        self.particle_ids.clear();
        self.positions.clear();
        self.velocities.clear();
        self.forces.clear();
        self.flags.clear();
        self.memberships.clear();
        self.groups.clear();
        self.pass_trace.clear();
        self.effects.clear();
    }
}

impl BoundaryCandidate {
    #[allow(
        clippy::too_many_arguments,
        reason = "the solver-tail candidate validates every aligned authoritative lane"
    )]
    pub(crate) fn new_with_buffers(
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
        buffers: &mut BoundaryBuffers,
    ) -> Result<Self, BoundarySolverError> {
        #[cfg(debug_assertions)]
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
        support::copy_into(
            particle_ids,
            &mut buffers.particle_ids,
            "boundary particle identities",
        )?;
        support::copy_into(
            positions,
            &mut buffers.positions,
            "boundary position candidates",
        )?;
        support::copy_into(
            velocities,
            &mut buffers.velocities,
            "boundary velocity candidates",
        )?;
        support::copy_into(forces, &mut buffers.forces, "boundary force candidates")?;
        support::copy_into(flags, &mut buffers.flags, "boundary flag candidates")?;
        support::copy_into(
            memberships,
            &mut buffers.memberships,
            "boundary membership candidates",
        )?;
        support::copy_into(groups, &mut buffers.groups, "boundary group candidates")?;
        buffers.pass_trace.clear();
        buffers.pass_trace.reserve(5);
        buffers.effects.clear();
        buffers.effects.reserve(effect_limit);
        Ok(Self {
            owner,
            particle_ids: std::mem::take(&mut buffers.particle_ids),
            positions: std::mem::take(&mut buffers.positions),
            velocities: std::mem::take(&mut buffers.velocities),
            forces: std::mem::take(&mut buffers.forces),
            flags: std::mem::take(&mut buffers.flags),
            memberships: std::mem::take(&mut buffers.memberships),
            groups: std::mem::take(&mut buffers.groups),
            stage: BoundaryStage::AfterRigidDamping,
            has_pending_force,
            pass_trace: std::mem::take(&mut buffers.pass_trace),
            effects: std::mem::take(&mut buffers.effects),
            effect_limit,
        })
    }

    pub(crate) fn recycle(self, buffers: &mut BoundaryBuffers) {
        *buffers = BoundaryBuffers {
            particle_ids: self.particle_ids,
            positions: self.positions,
            velocities: self.velocities,
            forces: self.forces,
            flags: self.flags,
            memberships: self.memberships,
            groups: self.groups,
            pass_trace: self.pass_trace,
            effects: self.effects,
        };
        buffers.clear();
    }

    #[cfg(test)]
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
        Self::new_with_buffers(
            owner,
            particle_ids,
            positions,
            velocities,
            forces,
            flags,
            memberships,
            groups,
            has_pending_force,
            effect_limit,
            &mut BoundaryBuffers::default(),
        )
    }

    fn begin_pass(&mut self, expected: BoundaryStage) -> Result<(), BoundarySolverError> {
        if self.stage != expected {
            return Err(BoundarySolverError::ReorderedPass {
                expected,
                actual: self.stage,
            });
        }
        Ok(())
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

#[cfg(test)]
mod buffer_tests;

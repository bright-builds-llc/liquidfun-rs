//! Boundary-tail execution kept as one staged candidate.

use crate::math::settings;
use crate::particle::solver::boundary::{
    BoundaryCandidate, barrier_candidate, collision_candidate, integrate_candidate,
    mark_rigid_projection, wall_candidate,
};
use crate::particle::solver::rigid::rigid_projection_candidate;
use crate::{CollisionDecisionHook, StepError};

use super::{SystemPassExecutor, boundary_error, rigid_error};

impl<H: CollisionDecisionHook> SystemPassExecutor<'_, '_, H> {
    pub(super) fn begin_boundary(&mut self) -> Result<(), StepError> {
        let (time_step, inverse_time_step) = self.substep();
        let record = self.record();
        let definition = record.definition;
        let diameter = 2.0 * definition.radius();
        let particle_mass = definition.density() * (settings::PARTICLE_STRIDE * diameter).powi(2);
        let source = BoundaryCandidate::new(
            self.system,
            record.storage.particle_ids(),
            record.storage.positions(),
            record.storage.velocities(),
            record.storage.forces(),
            record.storage.flags(),
            record.storage.groups(),
            record.storage.group_records(),
            record.storage.has_pending_system_force(),
            record.storage.len(),
        )
        .map_err(boundary_error)?;
        let candidate = barrier_candidate(
            &source,
            record.storage.pairs(),
            particle_mass,
            time_step,
            inverse_time_step,
            record
                .storage
                .len()
                .saturating_mul(record.storage.pairs().len()),
        )
        .map_err(boundary_error)?;
        self.maybe_boundary = Some(candidate);
        Ok(())
    }

    pub(super) fn run_collision(&mut self, iteration: u32) -> Result<(), StepError> {
        if self.maybe_boundary.is_none() {
            self.begin_boundary()?;
        }
        let (time_step, inverse_time_step) = self.substep();
        let record = self.record();
        let diameter = 2.0 * record.definition.radius();
        let particle_mass =
            record.definition.density() * (settings::PARTICLE_STRIDE * diameter).powi(2);
        let source = self
            .maybe_boundary
            .as_ref()
            .ok_or(StepError::ParticleLifecycleInvariant)?;
        let hits = self.world.filtered_collision_hits(
            source,
            self.bodies,
            time_step,
            iteration,
            self.hook_run,
        )?;
        let candidate = collision_candidate(
            source,
            &hits,
            iteration,
            particle_mass,
            time_step,
            inverse_time_step,
            hits.len(),
        )
        .map_err(boundary_error)?;
        self.maybe_boundary = Some(candidate);
        Ok(())
    }

    pub(super) fn run_rigid_projection(&mut self) -> Result<(), StepError> {
        let (time_step, inverse_time_step) = self.substep();
        let source = self
            .maybe_boundary
            .as_ref()
            .ok_or(StepError::ParticleLifecycleInvariant)?;
        let projected = rigid_projection_candidate(
            self.system,
            &source.particle_ids,
            &source.positions,
            &source.velocities,
            &source.memberships,
            &source.groups,
            time_step,
            inverse_time_step,
        )
        .map_err(rigid_error)?;
        let mut boundary = source.clone();
        boundary.velocities = projected.velocities;
        boundary.groups = projected.groups;
        self.maybe_boundary = Some(mark_rigid_projection(&boundary).map_err(boundary_error)?);
        Ok(())
    }

    pub(super) fn run_wall(&mut self) -> Result<(), StepError> {
        self.ensure_rigid_boundary_stage()?;
        let source = self
            .maybe_boundary
            .as_ref()
            .ok_or(StepError::ParticleLifecycleInvariant)?;
        self.maybe_boundary = Some(wall_candidate(source).map_err(boundary_error)?);
        Ok(())
    }

    pub(super) fn run_integrate(&mut self) -> Result<(), StepError> {
        let (time_step, _) = self.substep();
        self.ensure_rigid_boundary_stage()?;
        let source = self
            .maybe_boundary
            .take()
            .ok_or(StepError::ParticleLifecycleInvariant)?;
        let source = match source.stage {
            crate::particle::solver::boundary::BoundaryStage::AfterRigidProjection => {
                wall_candidate(&source).map_err(boundary_error)?
            }
            crate::particle::solver::boundary::BoundaryStage::AfterWall => source,
            _ => return Err(StepError::ParticleLifecycleInvariant),
        };
        let candidate = integrate_candidate(&source, time_step).map_err(boundary_error)?;
        self.commit_boundary(candidate)
    }

    pub(super) fn ensure_rigid_boundary_stage(&mut self) -> Result<(), StepError> {
        let source = self
            .maybe_boundary
            .as_ref()
            .ok_or(StepError::ParticleLifecycleInvariant)?;
        if source.stage == crate::particle::solver::boundary::BoundaryStage::AfterCollision {
            self.maybe_boundary = Some(mark_rigid_projection(source).map_err(boundary_error)?);
        }
        Ok(())
    }

    pub(super) fn commit_boundary(
        &mut self,
        candidate: BoundaryCandidate,
    ) -> Result<(), StepError> {
        self.record_mut()
            .storage
            .replace_solver_candidate(
                &candidate.particle_ids,
                candidate.positions,
                candidate.velocities,
                candidate.forces,
                candidate.groups,
                candidate.has_pending_force,
            )
            .map_err(|_error| StepError::ParticleLifecycleInvariant)
    }
}

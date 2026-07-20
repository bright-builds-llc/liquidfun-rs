//! Manifest pass execution for one particle system.

mod boundary_runtime;

use crate::arena::Arena;
use crate::math::settings;
use crate::particle::solver::boundary::BoundaryCandidate;
use crate::particle::solver::constraints;
use crate::particle::solver::material;
use crate::particle::solver::preparation;
use crate::particle::solver::pressure;
use crate::particle::solver::rigid::{RigidBodyContact, rigid_damping_candidate};
use crate::particle::solver::{ParticlePassExecutor, PassGate, PassId};
use crate::particle::{ParticleGroupFlags, VoronoiLimits};
use crate::{
    BodyId, CollisionDecisionHook, ParticleSystemId, StepConfiguration, StepError, WakePolicy,
    World,
};

use super::super::object::{Body, ParticleSystem};
use super::super::step::ContactHookRun;
use super::body_coupling::CandidateBodyCoupling;

pub(super) struct SystemPassExecutor<'a, 'hook, H> {
    world: &'a World,
    system: ParticleSystemId,
    configuration: StepConfiguration,
    systems: &'a mut Arena<ParticleSystem, ParticleSystemId>,
    bodies: &'a mut Arena<Body, BodyId>,
    hook_run: &'a mut ContactHookRun<'hook, H>,
    maybe_boundary: Option<BoundaryCandidate>,
}

impl<'a, 'hook, H: CollisionDecisionHook> SystemPassExecutor<'a, 'hook, H> {
    pub(super) fn new(
        world: &'a World,
        system: ParticleSystemId,
        configuration: StepConfiguration,
        systems: &'a mut Arena<ParticleSystem, ParticleSystemId>,
        bodies: &'a mut Arena<Body, BodyId>,
        hook_run: &'a mut ContactHookRun<'hook, H>,
    ) -> Self {
        Self {
            world,
            system,
            configuration,
            systems,
            bodies,
            hook_run,
            maybe_boundary: None,
        }
    }

    fn substep(&self) -> (f32, f32) {
        #[allow(
            clippy::cast_precision_loss,
            reason = "the checked particle iteration maximum is exactly representable as f32"
        )]
        let scale = self.configuration.particle_iterations() as f32;
        let time_step = self.configuration.time_step() / scale;
        let inverse_time_step = if self.configuration.time_step() > 0.0 {
            (1.0 / self.configuration.time_step()) * scale
        } else {
            0.0
        };
        (time_step, inverse_time_step)
    }

    fn record(&self) -> &ParticleSystem {
        self.systems
            .get(self.system)
            .expect("particle-system order contains only live systems")
    }

    fn record_mut(&mut self) -> &mut ParticleSystem {
        self.systems
            .get_mut(self.system)
            .expect("particle-system order contains only live systems")
    }

    fn with_body_coupling<T>(
        &mut self,
        solve: impl FnOnce(&mut ParticleSystem, &mut CandidateBodyCoupling<'_>) -> Result<T, StepError>,
    ) -> Result<T, StepError> {
        let system = self
            .systems
            .get_mut(self.system)
            .expect("particle-system order contains only live systems");
        let mut bodies = CandidateBodyCoupling::new(self.bodies);
        let value = solve(system, &mut bodies)?;
        bodies.finish()?;
        Ok(value)
    }

    fn rigid_body_contacts(&self) -> Result<Vec<RigidBodyContact>, StepError> {
        let storage = &self.record().storage;
        storage
            .semantic_body_contacts()
            .into_iter()
            .map(|contact| {
                let particle = storage
                    .particle_ids()
                    .iter()
                    .position(|candidate| *candidate == contact.particle())
                    .ok_or(StepError::ParticleLifecycleInvariant)?;
                let body = self
                    .bodies
                    .get(contact.body())
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
                let snapshot = body.state.snapshot();
                Ok(RigidBodyContact {
                    particle,
                    body: contact.body(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                    body_mass: snapshot.mass(),
                    body_inertia: snapshot.rotational_inertia(),
                    body_center: body.state.sweep().center(),
                    body_linear_velocity: body.state.solver_linear(),
                    body_angular_velocity: body.state.solver_angular(),
                })
            })
            .collect()
    }

    fn run_rigid_damping(&mut self) -> Result<(), StepError> {
        let contacts = self.rigid_body_contacts()?;
        let record = self.record();
        let definition = record.definition;
        let diameter = 2.0 * definition.radius();
        let particle_mass = definition.density() * (settings::PARTICLE_STRIDE * diameter).powi(2);
        let candidate = rigid_damping_candidate(
            self.system,
            record.storage.particle_ids(),
            record.storage.positions(),
            record.storage.velocities(),
            record.storage.groups(),
            record.storage.group_records(),
            record.storage.particle_contacts(),
            &contacts,
            particle_mass,
            definition.damping(),
            record.timestamp,
            contacts.len(),
        )
        .map_err(rigid_error)?;
        for impulse in &candidate.body_impulses {
            let body = self
                .bodies
                .get_mut(impulse.body)
                .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
            body.state = body
                .state
                .candidate_apply_linear_impulse(impulse.impulse, impulse.point, WakePolicy::Wake)
                .map_err(StepError::ParticleCoupling)?;
        }
        let record = self.record_mut();
        record
            .storage
            .replace_solver_candidate(
                &candidate.particle_ids,
                record.storage.positions().to_vec(),
                candidate.velocities,
                record.storage.forces().to_vec(),
                candidate.groups,
                record.storage.has_pending_system_force(),
            )
            .map_err(|_error| StepError::ParticleLifecycleInvariant)
    }
}
impl<H: CollisionDecisionHook> ParticlePassExecutor for SystemPassExecutor<'_, '_, H> {
    type Error = StepError;

    fn is_empty(&self) -> bool {
        self.record().storage.len() == 0
    }

    fn is_paused(&self) -> bool {
        self.record().definition.is_paused()
    }

    fn admits(&mut self, gate: PassGate) -> bool {
        match gate {
            PassGate::ExpirationLane => self.record().storage.lifetime_tracking_enabled(),
            PassGate::AggregateParticleFlags(flags)
            | PassGate::ExtraDampingAggregateFlags(flags) => self
                .record_mut()
                .storage
                .aggregate_particle_flags()
                .intersects(flags),
            PassGate::DirtyParticleFlags
            | PassGate::DirtyGroupFlags
            | PassGate::PauseTerminator
            | PassGate::Always => true,
            PassGate::NeedsGroupDepth => self
                .record_mut()
                .storage
                .aggregate_group_flags()
                .public
                .contains(ParticleGroupFlags::SOLID),
            PassGate::PendingForce => self.record().storage.has_pending_system_force(),
            PassGate::AggregateGroupFlags(flags) => {
                !(self.record_mut().storage.aggregate_group_flags().public & flags).is_empty()
            }
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the closed dispatcher mirrors all 31 manifest entries in one auditable match"
    )]
    fn execute(&mut self, pass: PassId, maybe_iteration: Option<u32>) -> Result<(), Self::Error> {
        let (time_step, inverse_time_step) = self.substep();
        match pass {
            PassId::Lifetime | PassId::ZombieCompaction | PassId::PauseGate => Ok(()),
            PassId::RefreshParticleFlags => {
                self.record_mut().storage.aggregate_particle_flags();
                Ok(())
            }
            PassId::RefreshGroupFlags => {
                self.record_mut().storage.aggregate_group_flags();
                Ok(())
            }
            PassId::ParticleContacts => {
                World::update_particle_contacts(self.system, self.systems, self.hook_run)
            }
            PassId::BodyContacts => {
                let timestamp = self
                    .record()
                    .timestamp
                    .checked_add(1)
                    .ok_or(StepError::ParticleLifecycleInvariant)?;
                let sources = self.world.fixture_contact_sources(self.bodies);
                World::update_body_contacts(
                    self.system,
                    self.systems,
                    &sources,
                    timestamp,
                    self.hook_run,
                )?;
                self.record_mut().timestamp = timestamp;
                Ok(())
            }
            PassId::Weight => {
                preparation::weight(&mut self.record_mut().storage);
                Ok(())
            }
            PassId::SolidDepth => {
                let diameter = 2.0 * self.record().definition.radius();
                preparation::solid_depth(&mut self.record_mut().storage, diameter)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::ReactiveTopology => {
                let diameter = 2.0 * self.record().definition.radius();
                let count = self.record().storage.declared_capacity();
                let limits = VoronoiLimits::new(
                    count,
                    count.saturating_mul(count).max(1),
                    count.saturating_mul(count).saturating_mul(4).max(1),
                    2_000_000,
                    count.saturating_mul(count).saturating_mul(2).max(1),
                );
                preparation::reactive_topology(&mut self.record_mut().storage, diameter, limits)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Force => {
                let definition = self.record().definition;
                preparation::force(&mut self.record_mut().storage, definition, time_step)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Viscous => {
                let definition = self.record().definition;
                self.with_body_coupling(|system, bodies| {
                    material::viscous(&mut system.storage, definition, bodies)
                        .map_err(|_error| StepError::ParticleLifecycleInvariant)
                })
            }
            PassId::Repulsive => {
                let definition = self.record().definition;
                material::repulsive(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Powder => {
                let definition = self.record().definition;
                material::powder(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Tensile => {
                let definition = self.record().definition;
                material::tensile(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Solid => {
                let definition = self.record().definition;
                material::solid(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::ColorMixing => {
                let definition = self.record().definition;
                material::color_mixing(&mut self.record_mut().storage, definition)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Gravity => {
                let definition = self.record().definition;
                let gravity = self.world.gravity();
                preparation::gravity(
                    &mut self.record_mut().storage,
                    definition,
                    time_step,
                    gravity,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::StaticPressure => {
                let definition = self.record().definition;
                pressure::static_pressure(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Pressure => {
                let definition = self.record().definition;
                self.with_body_coupling(|system, bodies| {
                    pressure::pressure(
                        &mut system.storage,
                        definition,
                        time_step,
                        inverse_time_step,
                        bodies,
                    )
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)
                })
            }
            PassId::Damping => {
                let definition = self.record().definition;
                self.with_body_coupling(|system, bodies| {
                    pressure::damping(&mut system.storage, definition, inverse_time_step, bodies)
                        .map_err(|_error| StepError::ParticleLifecycleInvariant)
                })
            }
            PassId::ExtraDamping => {
                let definition = self.record().definition;
                self.with_body_coupling(|system, bodies| {
                    pressure::extra_damping(&mut system.storage, definition, bodies)
                        .map_err(|_error| StepError::ParticleLifecycleInvariant)
                })
            }
            PassId::Elastic => {
                let definition = self.record().definition;
                constraints::elastic(
                    &mut self.record_mut().storage,
                    definition,
                    time_step,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::Spring => {
                let definition = self.record().definition;
                constraints::spring(
                    &mut self.record_mut().storage,
                    definition,
                    time_step,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::LimitVelocity => {
                let definition = self.record().definition;
                constraints::limit_velocity(
                    &mut self.record_mut().storage,
                    definition,
                    inverse_time_step,
                )
                .map_err(|_error| StepError::ParticleLifecycleInvariant)
            }
            PassId::RigidDamping => self.run_rigid_damping(),
            PassId::Barrier => self.begin_boundary(),
            PassId::Collision => {
                self.run_collision(maybe_iteration.ok_or(StepError::ParticleLifecycleInvariant)?)
            }
            PassId::Rigid => self.run_rigid_projection(),
            PassId::Wall => self.run_wall(),
            PassId::Integrate => self.run_integrate(),
        }
    }
}

fn rigid_error(error: crate::particle::solver::rigid::RigidSolverError) -> StepError {
    match error {
        crate::particle::solver::rigid::RigidSolverError::ResourceLimit { resource, limit } => {
            StepError::LimitExceeded { resource, limit }
        }
        crate::particle::solver::rigid::RigidSolverError::InvalidInput => {
            StepError::ParticleLifecycleInvariant
        }
    }
}

pub(super) fn boundary_error(
    error: crate::particle::solver::boundary::BoundarySolverError,
) -> StepError {
    match error {
        crate::particle::solver::boundary::BoundarySolverError::ResourceLimit {
            resource,
            limit,
        } => StepError::LimitExceeded { resource, limit },
        crate::particle::solver::boundary::BoundarySolverError::InvalidInput
        | crate::particle::solver::boundary::BoundarySolverError::ReorderedPass { .. } => {
            StepError::ParticleLifecycleInvariant
        }
    }
}

//! Source-ordered world integration for the closed particle solver graph.

mod body_coupling;
mod executor;

use crate::arena::Arena;
use crate::collision::{RayCastInput, Shape};
use crate::particle::body_contact::{self, FixtureContactSource};
use crate::particle::solver::boundary::{
    BoundaryCandidate, FilteredCollisionHit, collision_start_from_previous_transform,
};
use crate::particle::solver::preparation;
use crate::particle::{
    ParticleContactUpdate, ParticleFlags, ParticleNeighborhood, ParticleSystemView,
};
use crate::{BodyId, CollisionDecisionHook, ParticleSystemId, StepConfiguration, StepError, World};

use self::executor::{SystemPassExecutor, boundary_error};
use super::object::{Body, ParticleSystem};
use super::step::ContactHookRun;

impl World {
    pub(super) fn run_particle_solver<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let backup_bodies = self.bodies.clone();
        let backup_systems = self.particle_systems.clone();
        let backup_groups = self.particle_groups.clone();
        let result = (|| {
            self.run_particle_lifecycle_step(configuration.time_step(), hook_run)?;
            let mut candidate_bodies = self.bodies.clone();
            let mut candidate_systems = self.particle_systems.clone();
            let system_order = self.particle_system_order.clone();
            for system in system_order {
                let mut executor = SystemPassExecutor::new(
                    self,
                    system,
                    configuration,
                    &mut candidate_systems,
                    &mut candidate_bodies,
                    hook_run,
                );
                crate::particle::solver::run_particle_solver(configuration, &mut executor)?;
            }
            self.bodies = candidate_bodies;
            self.particle_systems = candidate_systems;
            Ok(())
        })();
        if result.is_err() {
            self.bodies = backup_bodies;
            self.particle_systems = backup_systems;
            self.particle_groups = backup_groups;
        }
        result
    }

    fn fixture_contact_sources(&self, bodies: &Arena<Body, BodyId>) -> Vec<FixtureContactSource> {
        let mut sources = Vec::new();
        for body_id in &self.body_order {
            let body = bodies
                .get(*body_id)
                .expect("world body order contains only live bodies");
            if !body.state.snapshot().is_active() {
                continue;
            }
            for fixture_id in &body.fixtures {
                let fixture = self
                    .fixtures
                    .get(*fixture_id)
                    .expect("body fixture adjacency contains only live fixtures");
                if fixture.definition.is_sensor() {
                    continue;
                }
                sources.push(FixtureContactSource {
                    fixture: *fixture_id,
                    body: *body_id,
                    shape: fixture.definition.shape().clone(),
                    transform: body.state.transform(),
                    center: body.state.sweep().center(),
                    inverse_mass: body.state.inverse_mass(),
                    inverse_inertia: body.state.inverse_inertia(),
                });
            }
        }
        sources
    }

    fn filtered_collision_hits<H: CollisionDecisionHook>(
        &self,
        candidate: &BoundaryCandidate,
        bodies: &Arena<Body, BodyId>,
        time_step: f32,
        particle_iteration: u32,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<FilteredCollisionHit>, StepError> {
        let mut hits = Vec::new();
        for (particle, (position, velocity)) in candidate
            .positions
            .iter()
            .copied()
            .zip(candidate.velocities.iter().copied())
            .enumerate()
        {
            for body_id in &self.body_order {
                let body = bodies
                    .get(*body_id)
                    .expect("world body order contains only live bodies");
                if !body.state.snapshot().is_active() {
                    continue;
                }
                let previous_transform = body
                    .state
                    .sweep()
                    .transform_at(0.0)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
                let current_transform = body.state.transform();
                let body_local_center = body.state.sweep().local_center();
                for fixture_id in &body.fixtures {
                    let fixture = self
                        .fixtures
                        .get(*fixture_id)
                        .expect("body fixture adjacency contains only live fixtures");
                    if fixture.definition.is_sensor() {
                        continue;
                    }
                    let shape = fixture.definition.shape();
                    let is_circle = matches!(shape, Shape::Circle(_));
                    let start = collision_start_from_previous_transform(
                        position,
                        previous_transform,
                        current_transform,
                        body_local_center,
                        is_circle,
                        particle_iteration,
                    )
                    .map_err(boundary_error)?;
                    let end = position + time_step * velocity;
                    if start == end {
                        continue;
                    }
                    let input = RayCastInput::new(start, end, 1.0)
                        .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
                    for child in 0..shape.child_count() {
                        let child = shape
                            .child_index(child)
                            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
                        let Some(hit) = shape
                            .ray_cast(input, current_transform, child)
                            .map_err(|_error| StepError::ParticleLifecycleInvariant)?
                        else {
                            continue;
                        };
                        let filter_contact = crate::ParticleBodyContact::new_internal(
                            candidate.particle_ids[particle],
                            *body_id,
                            *fixture_id,
                            0.0,
                            hit.normal(),
                            0.0,
                        );
                        if candidate.flags[particle].contains(ParticleFlags::FIXTURE_CONTACT_FILTER)
                            && !hook_run.should_collide_fixture_particle(&filter_contact)
                        {
                            continue;
                        }
                        hits.try_reserve(1)
                            .map_err(|_error| StepError::LimitExceeded {
                                resource: "filtered collision hits",
                                limit: hits.len(),
                            })?;
                        hits.push(FilteredCollisionHit {
                            particle,
                            body: *body_id,
                            previous_transform,
                            current_transform,
                            body_local_center,
                            is_circle,
                            fraction: hit.fraction(),
                            normal: hit.normal(),
                        });
                    }
                }
            }
        }
        Ok(hits)
    }

    fn update_particle_contacts<H: CollisionDecisionHook>(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let record = systems
            .get(system)
            .expect("system remains live during particle contact update");
        let diameter = 2.0 * record.definition.radius();
        let view = ParticleSystemView::new(&record.storage);
        let neighborhood =
            ParticleNeighborhood::from_view(&view, diameter).map_err(StepError::ParticleProxy)?;
        let previous = record.storage.semantic_particle_contacts();
        let update = ParticleContactUpdate::generate(&view, &neighborhood, &previous, |contact| {
            hook_run.should_collide_particle_pair(contact)
        })
        .map_err(StepError::ParticleContact)?;
        hook_run.ensure_lifecycle_capacity(update.effects().len())?;
        for effect in update.effects().iter().copied() {
            hook_run.record_particle_contact(effect)?;
        }
        preparation::particle_contacts(
            &mut systems
                .get_mut(system)
                .expect("system remains live during particle contact commit")
                .storage,
            update.contacts(),
        )
        .map_err(|_error| StepError::ParticleLifecycleInvariant)
    }

    fn update_body_contacts<H: CollisionDecisionHook>(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        sources: &[FixtureContactSource],
        timestamp: u32,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let record = systems
            .get(system)
            .expect("system remains live during body contact update");
        let previous = record.storage.semantic_body_contacts();
        let diameter = 2.0 * record.definition.radius();
        let update = body_contact::generate(
            &ParticleSystemView::new(&record.storage),
            sources,
            &previous,
            diameter,
            record.definition.density(),
            record.definition.uses_strict_contact_check(),
            |contact| hook_run.should_collide_fixture_particle(contact),
        );
        hook_run.ensure_lifecycle_capacity(update.effects().len())?;
        for effect in update.effects().iter().copied() {
            hook_run.record_particle_body_contact(effect)?;
        }
        let record = systems
            .get_mut(system)
            .expect("system remains live during body contact commit");
        preparation::body_contacts(
            &mut record.storage,
            update.contacts(),
            timestamp,
            record.definition.stuck_threshold(),
        )
        .map_err(|_error| StepError::ParticleLifecycleInvariant)
    }
}

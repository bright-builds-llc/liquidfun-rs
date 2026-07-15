//! Source-timed particle contact preparation and Phase 9 rigid coupling.

use crate::arena::Arena;
use crate::math::{Vec2, max, min, settings};
use crate::particle::body_contact::{self, FixtureContactSource};
use crate::particle::{ParticleContactUpdate, ParticleNeighborhood, ParticleSystemView};
use crate::{
    BodyId, CollisionDecisionHook, ParticleBodyContact, ParticleFlags, ParticleSystemId,
    StepConfiguration, StepError, WakePolicy, World,
};

use super::object::{Body, ParticleSystem};
use super::step::ContactHookRun;

const PRESSURE_STRENGTH: f32 = 0.05;

impl World {
    pub(super) fn run_particle_contact_prefix<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let mut candidate_bodies = self.bodies.clone();
        let mut candidate_systems = self.particle_systems.clone();
        let system_order = self.particle_system_order.clone();
        for system in system_order {
            self.run_system_contact_prefix(
                system,
                configuration,
                &mut candidate_systems,
                &mut candidate_bodies,
                hook_run,
            )?;
        }
        self.bodies = candidate_bodies;
        self.particle_systems = candidate_systems;
        Ok(())
    }

    fn run_system_contact_prefix<H: CollisionDecisionHook>(
        &self,
        system: ParticleSystemId,
        configuration: StepConfiguration,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        bodies: &mut Arena<Body, BodyId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        if systems
            .get(system)
            .expect("particle system order contains only live systems")
            .definition
            .is_paused()
        {
            return Ok(());
        }
        let iterations = configuration.particle_iterations();
        #[allow(
            clippy::cast_precision_loss,
            reason = "the checked iteration maximum is exactly representable as f32"
        )]
        let iteration_scale = iterations as f32;
        let sub_dt = configuration.time_step() / iteration_scale;
        let sub_inv_dt = if configuration.time_step() > 0.0 {
            (1.0 / configuration.time_step()) * iteration_scale
        } else {
            0.0
        };
        for _ in 0..iterations {
            let timestamp = systems
                .get(system)
                .expect("system remains live during one contact prefix")
                .timestamp
                .checked_add(1)
                .ok_or(StepError::ParticleLifecycleInvariant)?;
            Self::update_particle_contacts(system, systems, hook_run)?;
            let sources = self.fixture_contact_sources(bodies);
            Self::update_body_contacts(system, systems, &sources, timestamp, hook_run)?;
            Self::apply_body_contact_pressure_and_damping(
                system, systems, bodies, sub_dt, sub_inv_dt,
            )?;
            systems
                .get_mut(system)
                .expect("system remains live after one contact prefix")
                .timestamp = timestamp;
        }
        Ok(())
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
        systems
            .get_mut(system)
            .expect("system remains live during particle contact commit")
            .storage
            .replace_particle_contacts(update.contacts())
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
        record
            .storage
            .replace_body_contacts(update.contacts())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        record
            .storage
            .update_stuck_candidates(timestamp, record.definition.stuck_threshold());
        Ok(())
    }

    fn apply_body_contact_pressure_and_damping(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        bodies: &mut Arena<Body, BodyId>,
        sub_dt: f32,
        sub_inv_dt: f32,
    ) -> Result<(), StepError> {
        let record = systems
            .get(system)
            .expect("system remains live during body contact coupling");
        let definition = record.definition;
        let contacts = record.storage.semantic_body_contacts();
        let diameter = 2.0 * definition.radius();
        let inverse_diameter = 1.0 / diameter;
        let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
        let particle_inverse_mass = (1.0 / definition.density()) * inverse_stride * inverse_stride;
        let critical_velocity = diameter * sub_inv_dt;
        let critical_pressure = definition.density() * critical_velocity * critical_velocity;
        let pressure_per_weight = PRESSURE_STRENGTH * critical_pressure;
        let max_pressure = settings::MAX_PARTICLE_PRESSURE * critical_pressure;
        let velocity_per_pressure = sub_dt / (definition.density() * diameter);

        for contact in contacts.iter().copied() {
            Self::apply_pressure(
                system,
                systems,
                bodies,
                contact,
                particle_inverse_mass,
                pressure_per_weight,
                max_pressure,
                velocity_per_pressure,
            )?;
        }
        for contact in contacts {
            Self::apply_damping(
                system,
                systems,
                bodies,
                contact,
                particle_inverse_mass,
                definition.damping(),
                diameter,
                sub_inv_dt,
            )?;
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the pressure kernel keeps each pinned coefficient explicit"
    )]
    fn apply_pressure(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        bodies: &mut Arena<Body, BodyId>,
        contact: ParticleBodyContact,
        particle_inverse_mass: f32,
        pressure_per_weight: f32,
        max_pressure: f32,
        velocity_per_pressure: f32,
    ) -> Result<(), StepError> {
        let record = systems
            .get_mut(system)
            .expect("system remains live during pressure coupling");
        let weight = record
            .storage
            .particle_weight(contact.particle())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        let input = record
            .storage
            .input(contact.particle())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        let pressure = if input
            .flags
            .intersects(ParticleFlags::POWDER | ParticleFlags::TENSILE)
        {
            0.0
        } else {
            min(
                pressure_per_weight * max(0.0, weight - settings::MIN_PARTICLE_WEIGHT),
                max_pressure,
            )
        };
        let contact_pressure = pressure + pressure_per_weight * contact.weight();
        let impulse = velocity_per_pressure
            * contact.weight()
            * contact.mass()
            * contact_pressure
            * contact.normal();
        let velocity = record
            .storage
            .particle_velocity(contact.particle())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?
            - particle_inverse_mass * impulse;
        record
            .storage
            .set_particle_velocity_internal(contact.particle(), velocity)
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        Self::apply_body_impulse(bodies, contact, input.position, impulse)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the damping kernel keeps each pinned coefficient explicit"
    )]
    fn apply_damping(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        bodies: &mut Arena<Body, BodyId>,
        contact: ParticleBodyContact,
        particle_inverse_mass: f32,
        damping_strength: f32,
        diameter: f32,
        sub_inv_dt: f32,
    ) -> Result<(), StepError> {
        let body = bodies
            .get(contact.body())
            .expect("body contacts retain live body identities");
        let position = systems
            .get(system)
            .expect("system remains live during damping coupling")
            .storage
            .input(contact.particle())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?
            .position;
        let offset = position - body.state.sweep().center();
        let body_velocity =
            body.state.solver_linear() + Vec2::scalar_cross(body.state.solver_angular(), offset);
        let record = systems
            .get_mut(system)
            .expect("system remains live during damping coupling");
        let particle_velocity = record
            .storage
            .particle_velocity(contact.particle())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        let normal_velocity = (body_velocity - particle_velocity).dot(contact.normal());
        if normal_velocity >= 0.0 {
            return Ok(());
        }
        let quadratic_damping = 1.0 / (diameter * sub_inv_dt);
        let damping = max(
            damping_strength * contact.weight(),
            min(-quadratic_damping * normal_velocity, 0.5),
        );
        let impulse = damping * contact.mass() * normal_velocity * contact.normal();
        record
            .storage
            .set_particle_velocity_internal(
                contact.particle(),
                particle_velocity + particle_inverse_mass * impulse,
            )
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        Self::apply_body_impulse(bodies, contact, position, -impulse)
    }

    fn apply_body_impulse(
        bodies: &mut Arena<Body, BodyId>,
        contact: ParticleBodyContact,
        position: Vec2,
        impulse: Vec2,
    ) -> Result<(), StepError> {
        let body = bodies
            .get_mut(contact.body())
            .expect("body contacts retain live body identities");
        body.state = body
            .state
            .candidate_apply_linear_impulse(impulse, position, WakePolicy::Wake)
            .map_err(StepError::ParticleCoupling)?;
        Ok(())
    }
}

//! Source-timed particle contact preparation and Phase 9 rigid coupling.

use crate::particle::ParticleSystemView;
use crate::particle::body_contact::{self, FixtureContactSource};
use crate::{CollisionDecisionHook, ParticleSystemId, StepError, World};

use super::step::ContactHookRun;

impl World {
    pub(super) fn refresh_particle_body_contacts<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let sources = self.fixture_contact_sources();
        let systems = self.particle_system_order.clone();
        for system in systems {
            self.refresh_system_body_contacts(system, &sources, hook_run)?;
        }
        Ok(())
    }

    fn fixture_contact_sources(&self) -> Vec<FixtureContactSource> {
        let mut sources = Vec::new();
        for body_id in &self.body_order {
            let body = self
                .bodies
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

    fn refresh_system_body_contacts<H: CollisionDecisionHook>(
        &mut self,
        system: ParticleSystemId,
        sources: &[FixtureContactSource],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let record = self
            .particle_systems
            .get(system)
            .expect("particle system order contains only live systems");
        if record.definition.is_paused() {
            return Ok(());
        }
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
        let record = self
            .particle_systems
            .get_mut(system)
            .expect("particle system remains live during one refresh");
        record.timestamp = record.timestamp.wrapping_add(1);
        record
            .storage
            .replace_body_contacts(update.contacts())
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        record
            .storage
            .update_stuck_candidates(record.timestamp, record.definition.stuck_threshold());
        Ok(())
    }
}

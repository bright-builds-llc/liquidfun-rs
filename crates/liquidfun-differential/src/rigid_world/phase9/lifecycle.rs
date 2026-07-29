//! Source-ordered Phase 9 lifecycle collection.

use super::{
    DestroyedId, NativeRigidWorldError, ParticleBodyContactEffect, ParticleContactEffect,
    ParticleId, Phase9Occurrence, Phase9OccurrenceKind, ScenarioId, StepLifecycleEvent, StepReport,
    TimelineExecutor,
};

pub(crate) fn collect_step_occurrences(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    for event in report.lifecycle() {
        let maybe_occurrence = match event {
            StepLifecycleEvent::ParticleContact(effect) => {
                let (kind, particles) = match effect {
                    ParticleContactEffect::Begin(contact) => {
                        (Phase9OccurrenceKind::ContactCreated, contact.particles())
                    }
                    ParticleContactEffect::End(particles) => {
                        (Phase9OccurrenceKind::ContactDestroyed, *particles)
                    }
                };
                let (system_id, first) = semantic_particle_owner(executor, particles[0])?;
                let (_, second) = semantic_particle_owner(executor, particles[1])?;
                Some((kind, system_id, Some(first), Some(second), None))
            }
            StepLifecycleEvent::ParticleBodyContact(effect) => {
                let (kind, particle, maybe_fixture) = match effect {
                    ParticleBodyContactEffect::Begin(contact) => (
                        Phase9OccurrenceKind::ContactCreated,
                        contact.particle(),
                        Some(contact.fixture()),
                    ),
                    ParticleBodyContactEffect::End { fixture, particle } => (
                        Phase9OccurrenceKind::ContactDestroyed,
                        *particle,
                        Some(*fixture),
                    ),
                };
                let (system_id, particle_id) = semantic_particle_owner(executor, particle)?;
                let fixture_id = maybe_fixture.and_then(|fixture| {
                    executor
                        .fixtures
                        .iter()
                        .find_map(|(id, candidate)| (*candidate == fixture).then(|| id.clone()))
                });
                Some((kind, system_id, Some(particle_id), None, fixture_id))
            }
            StepLifecycleEvent::ParticleDestruction(record) => {
                let DestroyedId::Particle(particle) = record.destroyed() else {
                    continue;
                };
                let (system_id, particle_id) = semantic_particle_owner(executor, particle)?;
                Some((
                    Phase9OccurrenceKind::ParticleDestroyed,
                    system_id,
                    Some(particle_id),
                    None,
                    None,
                ))
            }
            _ => None,
        };
        let Some((kind, system_id, maybe_particle_id, maybe_other_particle_id, maybe_fixture_id)) =
            maybe_occurrence
        else {
            continue;
        };
        let ordinal = u32::try_from(executor.phase9_occurrences.len()).map_err(|_| {
            NativeRigidWorldError::Declaration {
                checkpoint_id: "phase9-occurrence".into(),
                message: "Phase 9 contact occurrence ordinal overflow".into(),
            }
        })?;
        executor.phase9_occurrences.push(Phase9Occurrence {
            ordinal,
            kind,
            system_id,
            maybe_particle_id,
            maybe_other_particle_id,
            maybe_fixture_id,
        });
    }
    Ok(())
}

fn semantic_particle_owner(
    executor: &TimelineExecutor,
    particle: ParticleId,
) -> Result<(ScenarioId, ScenarioId), NativeRigidWorldError> {
    let (_, system, _) = executor
        .particles
        .iter()
        .find(|(_, _, candidate)| *candidate == particle)
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "phase9-occurrence".into(),
            message: "Phase 9 occurrence has no semantic particle identity".into(),
        })?;
    let system_id = executor
        .particle_systems
        .iter()
        .find_map(|(id, candidate)| (*candidate == *system).then(|| id.clone()))
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "phase9-occurrence".into(),
            message: "Phase 9 occurrence has no semantic system identity".into(),
        })?;
    let particle_id = executor
        .particles
        .iter()
        .find_map(|(id, _, candidate)| (*candidate == particle).then(|| id.clone()))
        .expect("the particle identity was found above");
    Ok((system_id, particle_id))
}

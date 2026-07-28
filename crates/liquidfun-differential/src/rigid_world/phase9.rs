//! Native Phase 9 particle adapter and closed policy declarations.

mod adapter;
mod comparator;
mod evidence;
mod lifecycle;
mod policy;
pub use comparator::{
    Phase9ComparatorError, Phase9ComparisonOutcome, Phase9Mismatch, Phase9ObservationComparison,
    compare_phase9_particle_observations, compare_phase9_rigid_world_results,
    validate_phase9_policy_registry,
};
pub use evidence::{
    Phase9CaseEvidenceError, Phase9CrossRunProof, Phase9CrossRunProofRecord,
    Phase9EvidenceBindingError, Phase9EvidenceMismatch, Phase9EvidencePayloadRef,
    validate_phase9_cross_run_proofs, validate_phase9_evidence_bindings,
};
pub use policy::{
    PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9PolicyKind,
    phase9_observation_is_declared, phase9_policy_for_path,
};

use liquidfun::{
    DestroyedId, ParticleBodyContactEffect, ParticleCapacity, ParticleColor, ParticleContactEffect,
    ParticleDef, ParticleFlags, ParticleId, ParticleSystemDef, ParticleSystemId, QueryDirective,
    RayCastDirective, RayCastFraction, StepLifecycleEvent, StepReport, WorldQueryOccurrence,
    WorldRayCastOccurrence,
};
use liquidfun_test_protocol::{
    FloatBits, Phase9BodyContactObservation, Phase9Occurrence, Phase9OccurrenceKind,
    Phase9ParticleAction, Phase9ParticleBufferMode, Phase9ParticleContactObservation,
    Phase9ParticleObservation, Phase9ParticleSnapshot, Phase9ParticleSystemDeclaration,
    Phase9QueryControl, Phase9RayControl, Phase9StatisticsObservation, RigidWorldAction,
    RigidWorldActionRecord, RigidWorldTimeline, ScenarioId, Vec2Bits,
};

use adapter::{
    execute_spatial_query, mixed_state_observation, particle_declaration,
    particle_system_definition, phase9_checked_u32, phase9_lifecycle_observation, semantic_body_id,
    semantic_fixture_id, semantic_particle_id, system_declaration,
};
pub(super) use lifecycle::collect_step_occurrences;

use super::{NativeRigidWorldError, TimelineExecutor};
use crate::rigid_world::model::{action_error, vec2, vec2_bits};

pub(super) fn execute_action(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    record: &RigidWorldActionRecord,
) -> Result<bool, NativeRigidWorldError> {
    let RigidWorldAction::Particle { action } = record.action() else {
        return Ok(false);
    };
    let maybe_observation = match action {
        Phase9ParticleAction::CreateSystem { system_id } => {
            create_system(executor, timeline, system_id, record)?;
            None
        }
        Phase9ParticleAction::DestroySystem { system_id } => {
            Some(destroy_system(executor, system_id, record)?)
        }
        Phase9ParticleAction::CreateParticle { particle_id } => {
            create_particle(executor, timeline, particle_id, record)?
        }
        Phase9ParticleAction::InspectSystem { system_id } => {
            Some(inspect_system(executor, timeline, system_id, record)?)
        }
        Phase9ParticleAction::InspectParticle { particle_id } => {
            Some(inspect_particle(executor, timeline, particle_id, record)?)
        }
        Phase9ParticleAction::InspectParticleContact {
            system_id,
            contact_index,
        } => Some(inspect_particle_pair(
            executor,
            system_id,
            *contact_index,
            record,
        )?),
        Phase9ParticleAction::InspectBodyContact {
            system_id,
            contact_index,
        } => Some(inspect_body_pair(
            executor,
            system_id,
            *contact_index,
            record,
        )?),
        Phase9ParticleAction::InspectOccurrence { occurrence_index } => Some(
            executor
                .phase9_occurrences
                .get(*occurrence_index)
                .cloned()
                .map(|occurrence| Phase9ParticleObservation::Lifecycle { occurrence })
                .ok_or_else(|| action_error(record, "unknown Phase 9 occurrence index"))?,
        ),
        Phase9ParticleAction::SetPaused { system_id, paused } => {
            set_paused(executor, system_id, *paused, record)?;
            None
        }
        Phase9ParticleAction::SetPosition {
            particle_id,
            position,
        } => {
            set_position(executor, particle_id, *position, record)?;
            None
        }
        Phase9ParticleAction::SetVelocity {
            particle_id,
            velocity,
        } => {
            set_velocity(executor, particle_id, *velocity, record)?;
            None
        }
        Phase9ParticleAction::MarkForDestruction { particle_id } => {
            mark_for_destruction(executor, particle_id, record)?;
            None
        }
        Phase9ParticleAction::Compact { system_id } => compact(executor, system_id, record)?,
        Phase9ParticleAction::ApplyForce {
            particle_ids,
            force,
        } => {
            apply_force(executor, particle_ids, *force, record)?;
            None
        }
        Phase9ParticleAction::ApplyImpulse {
            particle_ids,
            impulse,
        } => {
            apply_impulse(executor, particle_ids, *impulse, record)?;
            None
        }
        Phase9ParticleAction::RequestStatistics { system_id } => {
            Some(request_statistics(executor, system_id, record)?)
        }
        Phase9ParticleAction::QueryAabb { .. } | Phase9ParticleAction::RayCast { .. } => {
            Some(execute_spatial_query(executor, action, record)?)
        }
    };
    record_observation(executor, timeline, maybe_observation);
    Ok(true)
}

fn record_observation(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    maybe_observation: Option<Phase9ParticleObservation>,
) {
    let observation =
        maybe_observation.unwrap_or_else(|| mixed_state_observation(executor, timeline));
    executor
        .semantic_observations
        .push(liquidfun_test_protocol::RigidWorldObservation::Particle { observation });
}

fn create_system(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    system_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let declaration = system_declaration(timeline, system_id, record)?;
    let definition =
        particle_system_definition(declaration).map_err(|message| action_error(record, message))?;
    let system = executor
        .world
        .create_particle_system_with_def(&definition)
        .map_err(|error| action_error(record, error))?;
    executor.particle_systems.push((system_id.clone(), system));
    Ok(())
}

fn destroy_system(
    executor: &mut TimelineExecutor,
    system_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    executor
        .world
        .destroy_particle_system(system)
        .map_err(|error| action_error(record, error))?;
    executor
        .particle_systems
        .retain(|(_, candidate)| *candidate != system);
    executor
        .particles
        .retain(|(_, candidate_system, _)| *candidate_system != system);
    phase9_lifecycle_observation(
        executor,
        Phase9OccurrenceKind::SystemDestroyed,
        system_id.clone(),
        None,
        None,
        None,
        record,
    )
}

fn create_particle(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    particle_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Option<Phase9ParticleObservation>, NativeRigidWorldError> {
    let declaration = particle_declaration(timeline, particle_id, record)?;
    let system = executor.particle_system(&declaration.system_id, record)?;
    let definition = ParticleDef::default()
        .with_position(vec2(declaration.position))
        .and_then(|definition| definition.with_velocity(vec2(declaration.velocity)))
        .and_then(|definition| definition.with_lifetime(declaration.lifetime_bits.to_f32()))
        .map_err(|error| action_error(record, error))?
        .with_flags(ParticleFlags::from_bits_retain(declaration.flags_bits))
        .with_color(ParticleColor::new(
            declaration.color[0],
            declaration.color[1],
            declaration.color[2],
            declaration.color[3],
        ));
    let receipt = executor
        .world
        .create_particle_with_def(system, None, &definition)
        .map_err(|error| action_error(record, error))?;
    let particle = receipt.created_particle();
    let maybe_evicted_id = receipt
        .destruction_occurrences()
        .first()
        .map(|occurrence| semantic_particle_id(executor, system, occurrence.particle(), record))
        .transpose()?;
    if receipt.destruction_occurrences().len() > 1 {
        return Err(action_error(
            record,
            "one creation may emit at most one Phase 9 destruction occurrence",
        ));
    }
    executor
        .particles
        .retain(|(_, candidate_system, candidate)| {
            *candidate_system != system || executor.world.particle_snapshot(*candidate).is_ok()
        });
    executor
        .particles
        .push((particle_id.clone(), system, particle));
    maybe_evicted_id
        .map(|evicted_id| {
            phase9_lifecycle_observation(
                executor,
                Phase9OccurrenceKind::ParticleDestroyed,
                declaration.system_id.clone(),
                Some(evicted_id),
                None,
                None,
                record,
            )
        })
        .transpose()
}

fn inspect_system(
    executor: &TimelineExecutor,
    timeline: &RigidWorldTimeline,
    system_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    let snapshot = executor
        .world
        .particle_system_snapshot(system)
        .map_err(|error| action_error(record, error))?;
    let particle_ids = timeline
        .particles()
        .iter()
        .map(|declaration| &declaration.particle_id)
        .filter(|particle_id| {
            executor.particles.iter().any(
                |(live_particle_id, candidate_system, candidate_particle)| {
                    live_particle_id == *particle_id
                        && *candidate_system == system
                        && executor
                            .world
                            .particle_snapshot(*candidate_particle)
                            .is_ok()
                },
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(Phase9ParticleObservation::System {
        system_id: system_id.clone(),
        paused: snapshot.is_paused(),
        particle_ids: particle_ids.into_boxed_slice(),
    })
}

fn inspect_particle(
    executor: &TimelineExecutor,
    timeline: &RigidWorldTimeline,
    particle_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let (system, particle) = executor.particle(particle_id, record)?;
    let snapshot = executor
        .world
        .particle_snapshot(particle)
        .map_err(|error| action_error(record, error))?;
    let view = executor
        .world
        .particle_system_view(system)
        .map_err(|error| action_error(record, error))?;
    let dense_index = view
        .particle_ids()
        .iter()
        .position(|candidate| *candidate == particle)
        .ok_or_else(|| action_error(record, "particle view omitted the live particle"))?;
    Ok(Phase9ParticleObservation::Particle {
        snapshot: Phase9ParticleSnapshot {
            particle_id: particle_id.clone(),
            system_id: particle_declaration(timeline, particle_id, record)?
                .system_id
                .clone(),
            position: vec2_bits(snapshot.position()),
            velocity: vec2_bits(snapshot.velocity()),
            flags_bits: snapshot.flags().bits(),
            color: snapshot.color().components(),
            weight_bits: FloatBits::new(view.weights()[dense_index].to_bits()),
            force: vec2_bits(view.forces()[dense_index]),
            pending_destruction: false,
        },
    })
}

fn inspect_particle_pair(
    executor: &TimelineExecutor,
    system_id: &ScenarioId,
    contact_index: usize,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    let view = executor
        .world
        .particle_system_view(system)
        .map_err(|error| action_error(record, error))?;
    let contact = view
        .particle_contacts()
        .nth(contact_index)
        .ok_or_else(|| action_error(record, "particle contact index is not live"))?;
    let [particle_a, particle_b] = contact.particles();
    Ok(Phase9ParticleObservation::ParticleContact {
        contact: Phase9ParticleContactObservation {
            system_id: system_id.clone(),
            particle_a_id: semantic_particle_id(executor, system, particle_a, record)?,
            particle_b_id: semantic_particle_id(executor, system, particle_b, record)?,
            flags_bits: contact.flags().bits(),
            weight_bits: FloatBits::new(contact.weight().to_bits()),
            normal: vec2_bits(contact.normal()),
        },
    })
}

fn inspect_body_pair(
    executor: &TimelineExecutor,
    system_id: &ScenarioId,
    contact_index: usize,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    let view = executor
        .world
        .particle_system_view(system)
        .map_err(|error| action_error(record, error))?;
    let contact = view
        .body_contacts()
        .nth(contact_index)
        .ok_or_else(|| action_error(record, "body contact index is not live"))?;
    Ok(Phase9ParticleObservation::BodyContact {
        contact: Phase9BodyContactObservation {
            system_id: system_id.clone(),
            particle_id: semantic_particle_id(executor, system, contact.particle(), record)?,
            body_id: semantic_body_id(executor, contact.body(), record)?,
            fixture_id: semantic_fixture_id(executor, contact.fixture(), record)?,
            weight_bits: FloatBits::new(contact.weight().to_bits()),
            normal: vec2_bits(contact.normal()),
            mass_bits: FloatBits::new(contact.mass().to_bits()),
        },
    })
}

fn set_paused(
    executor: &mut TimelineExecutor,
    system_id: &ScenarioId,
    paused: bool,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    executor
        .world
        .set_particle_system_paused(system, paused)
        .map_err(|error| action_error(record, error))
}

fn set_position(
    executor: &mut TimelineExecutor,
    particle_id: &ScenarioId,
    position: Vec2Bits,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let (_, particle) = executor.particle(particle_id, record)?;
    executor
        .world
        .set_particle_position(particle, vec2(position))
        .map_err(|error| action_error(record, error))
}

fn set_velocity(
    executor: &mut TimelineExecutor,
    particle_id: &ScenarioId,
    velocity: Vec2Bits,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let (_, particle) = executor.particle(particle_id, record)?;
    executor
        .world
        .set_particle_velocity(particle, vec2(velocity))
        .map_err(|error| action_error(record, error))
}

fn mark_for_destruction(
    executor: &mut TimelineExecutor,
    particle_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let (_, particle) = executor.particle(particle_id, record)?;
    executor
        .world
        .mark_particle_for_destruction(particle)
        .map(|_| ())
        .map_err(|error| action_error(record, error))
}

fn compact(
    executor: &mut TimelineExecutor,
    system_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Option<Phase9ParticleObservation>, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    let report = executor
        .world
        .compact_pending_particles(system)
        .map_err(|error| action_error(record, error))?;
    let requested_particles = report
        .lifecycle()
        .iter()
        .filter_map(|event| match event {
            liquidfun::LifecycleEvent::ParticleDestruction(record) => {
                let liquidfun::DestroyedId::Particle(particle) = record.destroyed() else {
                    return None;
                };
                executor
                    .particles
                    .iter()
                    .find_map(|(id, candidate_system, candidate)| {
                        (*candidate_system == system && *candidate == particle).then(|| id.clone())
                    })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if requested_particles.len() > 1 {
        return Err(action_error(
            record,
            "one corpus compaction may emit at most one Phase 9 occurrence",
        ));
    }
    executor
        .particles
        .retain(|(_, candidate_system, particle)| {
            *candidate_system != system || executor.world.particle_snapshot(*particle).is_ok()
        });
    requested_particles
        .into_iter()
        .next()
        .map(|particle_id| {
            phase9_lifecycle_observation(
                executor,
                Phase9OccurrenceKind::ParticleDestroyed,
                system_id.clone(),
                Some(particle_id),
                None,
                None,
                record,
            )
        })
        .transpose()
}

fn apply_force(
    executor: &mut TimelineExecutor,
    particle_ids: &[ScenarioId],
    force: Vec2Bits,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let (system, particles) = executor.particle_range(particle_ids, record)?;
    executor
        .world
        .apply_particle_force_range(system, &particles, vec2(force))
        .map_err(|error| action_error(record, error))
}

fn apply_impulse(
    executor: &mut TimelineExecutor,
    particle_ids: &[ScenarioId],
    impulse: Vec2Bits,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let (system, particles) = executor.particle_range(particle_ids, record)?;
    executor
        .world
        .apply_particle_linear_impulse_range(system, &particles, vec2(impulse))
        .map_err(|error| action_error(record, error))
}

fn request_statistics(
    executor: &TimelineExecutor,
    system_id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let system = executor.particle_system(system_id, record)?;
    let statistics = executor
        .world
        .particle_system_statistics(system)
        .map_err(|error| action_error(record, error))?;
    let stuck_particle_ids = statistics
        .stuck_candidates()
        .iter()
        .map(|particle| semantic_particle_id(executor, system, *particle, record))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Phase9ParticleObservation::Statistics {
        statistics: Phase9StatisticsObservation {
            maybe_system_id: Some(system_id.clone()),
            system_count: phase9_checked_u32(executor.particle_systems.len(), record)?,
            particle_count: phase9_checked_u32(statistics.particle_count(), record)?,
            pending_particle_count: phase9_checked_u32(
                statistics.pending_particle_count(),
                record,
            )?,
            particle_contact_count: phase9_checked_u32(
                statistics.particle_contact_count(),
                record,
            )?,
            body_contact_count: phase9_checked_u32(statistics.body_contact_count(), record)?,
            stuck_particle_ids: stuck_particle_ids.into_boxed_slice(),
            collision_energy_bits: FloatBits::new(statistics.collision_energy().to_bits()),
            declared_capacity: phase9_checked_u32(statistics.declared_capacity(), record)?,
            effective_capacity: phase9_checked_u32(statistics.effective_capacity(), record)?,
        },
    })
}

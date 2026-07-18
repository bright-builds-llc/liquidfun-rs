//! Native Phase 9 particle adapter and closed policy declarations.

mod comparator;
mod evidence;
pub use comparator::{
    Phase9ComparatorError, Phase9ComparisonOutcome, Phase9Mismatch, Phase9ObservationComparison,
    compare_phase9_particle_observations, compare_phase9_rigid_world_results,
    validate_phase9_policy_registry,
};
pub use evidence::{Phase9EvidenceBindingError, validate_phase9_evidence_bindings};

use liquidfun::collision::{Aabb, RayCastInput};
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

use super::{NativeRigidWorldError, TimelineExecutor};
use crate::rigid_world::model::{action_error, vec2, vec2_bits};

/// Closed identity of the reviewed Phase 9 declaration and policy registry.
pub const PHASE9_REGISTRY_ID: &str = "phase9-v1";

/// Named comparison class assigned to a reviewed Phase 9 semantic path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase9PolicyKind {
    /// Identity, order, multiplicity, branch, or count equality.
    ExactDiscrete,
    /// IEEE-754 or byte field equality.
    ExactBits,
    /// Reviewed ULP distance for iterative vector state.
    Ulps,
    /// Reviewed absolute-relative bound for accumulated values.
    AbsoluteRelative,
    /// Unit-specific absolute bound for ray or mass values.
    DimensionedAbsolute,
}

/// Every required Phase 9 policy path. Absence from this list fails closed.
pub const PHASE9_REQUIRED_POLICY_PATHS: &[&str] = &[
    "particle.storage.identity",
    "particle.capacity.mode",
    "particle.permutation.order",
    "particle.lifetime.order",
    "particle.zombie.lifecycle",
    "particle.contact.identity",
    "particle.strict_contact.branch",
    "particle.filter.decision",
    "particle.listener.occurrence",
    "particle.force.range",
    "particle.statistics.counts",
    "particle.query.order",
    "particle.query.culling",
    "particle.coupling.identity",
    "particle.configuration.bits",
    "particle.position",
    "particle.velocity",
    "particle.contact.normal",
    "particle.contact.weight",
    "particle.statistics.collision_energy",
    "particle.ray.fraction",
    "particle.body_contact.mass",
];

/// Returns the reviewed policy for a closed Phase 9 path.
#[must_use]
pub fn phase9_policy_for_path(path: &str) -> Option<Phase9PolicyKind> {
    match path {
        "particle.storage.identity"
        | "particle.capacity.mode"
        | "particle.permutation.order"
        | "particle.lifetime.order"
        | "particle.zombie.lifecycle"
        | "particle.contact.identity"
        | "particle.strict_contact.branch"
        | "particle.filter.decision"
        | "particle.listener.occurrence"
        | "particle.force.range"
        | "particle.statistics.counts"
        | "particle.query.order"
        | "particle.query.culling"
        | "particle.coupling.identity" => Some(Phase9PolicyKind::ExactDiscrete),
        "particle.configuration.bits" => Some(Phase9PolicyKind::ExactBits),
        "particle.position" | "particle.velocity" | "particle.contact.normal" => {
            Some(Phase9PolicyKind::Ulps)
        }
        "particle.contact.weight" | "particle.statistics.collision_energy" => {
            Some(Phase9PolicyKind::AbsoluteRelative)
        }
        "particle.ray.fraction" | "particle.body_contact.mass" => {
            Some(Phase9PolicyKind::DimensionedAbsolute)
        }
        _ => None,
    }
}

/// Returns whether an observation belongs to the closed Phase 9 registry.
#[must_use]
pub const fn phase9_observation_is_declared(observation: &Phase9ParticleObservation) -> bool {
    match observation {
        Phase9ParticleObservation::System { .. }
        | Phase9ParticleObservation::Particle { .. }
        | Phase9ParticleObservation::Lifecycle { .. }
        | Phase9ParticleObservation::ParticleContact { .. }
        | Phase9ParticleObservation::BodyContact { .. }
        | Phase9ParticleObservation::Statistics { .. }
        | Phase9ParticleObservation::Query { .. }
        | Phase9ParticleObservation::RayCast { .. }
        | Phase9ParticleObservation::MixedState { .. } => true,
    }
}

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

pub(super) fn collect_step_occurrences(
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

fn execute_spatial_query(
    executor: &TimelineExecutor,
    action: &Phase9ParticleAction,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    match action {
        Phase9ParticleAction::QueryAabb {
            system_id,
            lower,
            upper,
            control,
        } => query(
            executor,
            system_id.as_ref(),
            *lower,
            *upper,
            *control,
            record,
        ),
        Phase9ParticleAction::RayCast {
            system_id,
            start,
            end,
            control,
        } => ray(executor, system_id.as_ref(), *start, *end, *control, record),
        _ => Err(action_error(
            record,
            "non-spatial action reached the Phase 9 spatial dispatcher",
        )),
    }
}

fn query(
    executor: &TimelineExecutor,
    maybe_system_id: Option<&ScenarioId>,
    lower: Vec2Bits,
    upper: Vec2Bits,
    control: Phase9QueryControl,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let aabb = Aabb::new(vec2(lower), vec2(upper)).map_err(|error| action_error(record, error))?;
    let mut visited = Vec::new();
    let mut terminated = false;
    let mut directive = || match control {
        Phase9QueryControl::Continue => QueryDirective::Continue,
        Phase9QueryControl::Terminate => {
            terminated = true;
            QueryDirective::Terminate
        }
    };
    if let Some(system_id) = maybe_system_id {
        let system = executor.particle_system(system_id, record)?;
        executor
            .world
            .query_particle_system_aabb(system, aabb, |occurrence| {
                visited.push((occurrence.system(), occurrence.particle()));
                directive()
            })
            .map_err(|error| action_error(record, error))?;
    } else {
        executor
            .world
            .query_aabb_with_particles(aabb, |occurrence| {
                if let WorldQueryOccurrence::Particle(particle) = occurrence {
                    visited.push((particle.system(), particle.particle()));
                }
                directive()
            })
            .map_err(|error| action_error(record, error))?;
    }
    let particle_ids = visited
        .into_iter()
        .map(|(system, particle)| semantic_particle_id(executor, system, particle, record))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Phase9ParticleObservation::Query {
        terminated,
        particle_ids: particle_ids.into_boxed_slice(),
    })
}

fn ray(
    executor: &TimelineExecutor,
    maybe_system_id: Option<&ScenarioId>,
    start: Vec2Bits,
    end: Vec2Bits,
    control: Phase9RayControl,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let input = RayCastInput::new(vec2(start), vec2(end), 1.0)
        .map_err(|error| action_error(record, error))?;
    let mut hits = Vec::new();
    let mut terminated = false;
    if let Some(system_id) = maybe_system_id {
        let system = executor.particle_system(system_id, record)?;
        executor
            .world
            .ray_cast_particle_system(system, input, |hit| {
                hits.push((hit.system(), hit.particle(), hit.fraction().get()));
                ray_directive(control, hit.fraction().get(), &mut terminated)
            })
            .map_err(|error| action_error(record, error))?;
    } else {
        executor
            .world
            .ray_cast_with_particles(input, |occurrence| {
                if let WorldRayCastOccurrence::Particle(hit) = occurrence {
                    hits.push((hit.system(), hit.particle(), hit.fraction().get()));
                }
                match occurrence {
                    WorldRayCastOccurrence::Particle(hit) => {
                        ray_directive(control, hit.fraction().get(), &mut terminated)
                    }
                    WorldRayCastOccurrence::Fixture(_) => RayCastDirective::Continue,
                }
            })
            .map_err(|error| action_error(record, error))?;
    }
    let (particle_ids, fractions_bits): (Vec<_>, Vec<_>) = hits
        .into_iter()
        .map(|(system, particle, fraction)| {
            semantic_particle_id(executor, system, particle, record)
                .map(|id| (id, FloatBits::new(fraction.to_bits())))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();
    Ok(Phase9ParticleObservation::RayCast {
        terminated,
        particle_ids: particle_ids.into_boxed_slice(),
        fractions_bits: fractions_bits.into_boxed_slice(),
    })
}

fn mixed_state_observation(
    executor: &TimelineExecutor,
    timeline: &RigidWorldTimeline,
) -> Phase9ParticleObservation {
    Phase9ParticleObservation::MixedState {
        body_ids: timeline
            .bodies()
            .iter()
            .map(liquidfun_test_protocol::RigidBodyDeclaration::body_id)
            .filter(|id| executor.bodies.iter().any(|(live, _)| live == *id))
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        particle_ids: timeline
            .particles()
            .iter()
            .map(|declaration| &declaration.particle_id)
            .filter(|id| executor.particles.iter().any(|(live, _, _)| live == *id))
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

fn ray_directive(
    control: Phase9RayControl,
    fraction: f32,
    terminated: &mut bool,
) -> RayCastDirective {
    match control {
        Phase9RayControl::Ignore => RayCastDirective::Ignore,
        Phase9RayControl::Continue => RayCastDirective::Continue,
        Phase9RayControl::Clip => RayCastDirective::Clip(
            RayCastFraction::new(fraction).expect("a reported hit fraction is always a valid clip"),
        ),
        Phase9RayControl::Terminate => {
            *terminated = true;
            RayCastDirective::Terminate
        }
    }
}

fn semantic_particle_id(
    executor: &TimelineExecutor,
    system: ParticleSystemId,
    particle: ParticleId,
    record: &RigidWorldActionRecord,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .particles
        .iter()
        .find(|(_, candidate_system, candidate)| {
            *candidate_system == system && *candidate == particle
        })
        .map(|(id, _, _)| id.clone())
        .ok_or_else(|| action_error(record, "particle observation has no semantic identity"))
}

fn semantic_body_id(
    executor: &TimelineExecutor,
    body: liquidfun::BodyId,
    record: &RigidWorldActionRecord,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .bodies
        .iter()
        .find_map(|(id, candidate)| (*candidate == body).then(|| id.clone()))
        .ok_or_else(|| action_error(record, "body contact has no semantic body identity"))
}

fn semantic_fixture_id(
    executor: &TimelineExecutor,
    fixture: liquidfun::FixtureId,
    record: &RigidWorldActionRecord,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .fixtures
        .iter()
        .find_map(|(id, candidate)| (*candidate == fixture).then(|| id.clone()))
        .ok_or_else(|| action_error(record, "body contact has no semantic fixture identity"))
}

#[allow(clippy::too_many_arguments)]
fn phase9_lifecycle_observation(
    executor: &mut TimelineExecutor,
    kind: Phase9OccurrenceKind,
    system_id: ScenarioId,
    maybe_particle_id: Option<ScenarioId>,
    maybe_other_particle_id: Option<ScenarioId>,
    maybe_fixture_id: Option<ScenarioId>,
    record: &RigidWorldActionRecord,
) -> Result<Phase9ParticleObservation, NativeRigidWorldError> {
    let ordinal = executor.next_phase9_occurrence_ordinal;
    executor.next_phase9_occurrence_ordinal = ordinal
        .checked_add(1)
        .ok_or_else(|| action_error(record, "Phase 9 lifecycle ordinal overflow"))?;
    Ok(Phase9ParticleObservation::Lifecycle {
        occurrence: Phase9Occurrence {
            ordinal,
            kind,
            system_id,
            maybe_particle_id,
            maybe_other_particle_id,
            maybe_fixture_id,
        },
    })
}

fn phase9_checked_u32(
    value: usize,
    record: &RigidWorldActionRecord,
) -> Result<u32, NativeRigidWorldError> {
    u32::try_from(value).map_err(|error| action_error(record, error))
}

fn system_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<&'a Phase9ParticleSystemDeclaration, NativeRigidWorldError> {
    timeline
        .particle_systems()
        .iter()
        .find(|declaration| &declaration.system_id == id)
        .ok_or_else(|| action_error(record, format!("missing particle system `{id}`")))
}

fn particle_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    record: &RigidWorldActionRecord,
) -> Result<&'a liquidfun_test_protocol::Phase9ParticleDeclaration, NativeRigidWorldError> {
    timeline
        .particles()
        .iter()
        .find(|declaration| &declaration.particle_id == id)
        .ok_or_else(|| action_error(record, format!("missing particle `{id}`")))
}

fn particle_system_definition(
    declaration: &Phase9ParticleSystemDeclaration,
) -> Result<ParticleSystemDef, String> {
    let capacity = match declaration.buffer_mode {
        Phase9ParticleBufferMode::Growable { initial_capacity } => {
            ParticleCapacity::growable(initial_capacity)
        }
        Phase9ParticleBufferMode::Fixed { capacity } => ParticleCapacity::fixed(capacity),
    }
    .map_err(|error| error.to_string())?;
    let mut definition = ParticleSystemDef::default()
        .with_paused(declaration.paused)
        .with_strict_contact_check(declaration.strict_contact_check)
        .with_stuck_threshold(declaration.stuck_threshold)
        .with_destruction_by_age(declaration.destruction_by_age)
        .with_density(declaration.density_bits.to_f32())
        .and_then(|definition| {
            definition.with_gravity_scale(declaration.gravity_scale_bits.to_f32())
        })
        .and_then(|definition| definition.with_radius(declaration.radius_bits.to_f32()))
        .and_then(|definition| definition.with_damping(declaration.damping_bits.to_f32()))
        .and_then(|definition| {
            definition.with_lifetime_granularity(declaration.lifetime_granularity_bits.to_f32())
        })
        .and_then(|definition| definition.with_capacity(capacity))
        .map_err(|error| error.to_string())?;
    if let Some(maximum_count) = declaration.maximum_count {
        definition = definition
            .with_maximum_count(maximum_count)
            .map_err(|error| error.to_string())?;
    }
    Ok(definition)
}

impl TimelineExecutor {
    fn particle_system(
        &self,
        id: &ScenarioId,
        record: &RigidWorldActionRecord,
    ) -> Result<ParticleSystemId, NativeRigidWorldError> {
        self.particle_systems
            .iter()
            .find_map(|(candidate, system)| (candidate == id).then_some(*system))
            .ok_or_else(|| action_error(record, format!("unknown particle system `{id}`")))
    }

    fn particle(
        &self,
        id: &ScenarioId,
        record: &RigidWorldActionRecord,
    ) -> Result<(ParticleSystemId, ParticleId), NativeRigidWorldError> {
        self.particles
            .iter()
            .find_map(|(candidate, system, particle)| {
                (candidate == id).then_some((*system, *particle))
            })
            .ok_or_else(|| action_error(record, format!("unknown particle `{id}`")))
    }

    fn particle_range(
        &self,
        ids: &[ScenarioId],
        record: &RigidWorldActionRecord,
    ) -> Result<(ParticleSystemId, Vec<ParticleId>), NativeRigidWorldError> {
        let mut systems_and_particles = ids
            .iter()
            .map(|id| self.particle(id, record))
            .collect::<Result<Vec<_>, _>>()?;
        let Some((system, _)) = systems_and_particles.first().copied() else {
            return Err(action_error(record, "particle range must not be empty"));
        };
        if systems_and_particles
            .iter()
            .any(|(candidate, _)| *candidate != system)
        {
            return Err(action_error(
                record,
                "particle range must belong to one system",
            ));
        }
        Ok((
            system,
            systems_and_particles
                .drain(..)
                .map(|(_, particle)| particle)
                .collect(),
        ))
    }
}

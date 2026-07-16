//! Native Phase 9 particle adapter and closed policy declarations.

use liquidfun::collision::{Aabb, RayCastInput};
use liquidfun::{
    ParticleCapacity, ParticleColor, ParticleDef, ParticleFlags, ParticleId, ParticleSystemDef,
    ParticleSystemId, QueryDirective, RayCastDirective, WorldQueryOccurrence,
    WorldRayCastOccurrence,
};
use liquidfun_test_protocol::{
    FloatBits, Phase9ParticleAction, Phase9ParticleBufferMode, Phase9ParticleObservation,
    Phase9ParticleSystemDeclaration, Phase9StatisticsObservation, RigidWorldAction,
    RigidWorldActionRecord, RigidWorldTimeline, ScenarioId,
};

use super::{NativeRigidWorldError, TimelineExecutor};
use crate::rigid_world::model::{action_error, vec2};

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
    let mut maybe_observation = None;
    match action {
        Phase9ParticleAction::CreateSystem { system_id } => {
            let declaration = system_declaration(timeline, system_id, record)?;
            let definition = particle_system_definition(declaration)
                .map_err(|message| action_error(record, message))?;
            let system = executor
                .world
                .create_particle_system_with_def(&definition)
                .map_err(|error| action_error(record, error))?;
            executor.particle_systems.push((system_id.clone(), system));
        }
        Phase9ParticleAction::DestroySystem { system_id } => {
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
        }
        Phase9ParticleAction::CreateParticle { particle_id } => {
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
            executor
                .particles
                .push((particle_id.clone(), system, particle));
        }
        Phase9ParticleAction::InspectSystem { system_id } => {
            let system = executor.particle_system(system_id, record)?;
            executor
                .world
                .particle_system_view(system)
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::InspectParticle { particle_id } => {
            let (_, particle) = executor.particle(particle_id, record)?;
            executor
                .world
                .particle_snapshot(particle)
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::SetPaused { system_id, paused } => {
            let system = executor.particle_system(system_id, record)?;
            executor
                .world
                .set_particle_system_paused(system, *paused)
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::SetPosition {
            particle_id,
            position,
        } => {
            let (_, particle) = executor.particle(particle_id, record)?;
            executor
                .world
                .set_particle_position(particle, vec2(*position))
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::SetVelocity {
            particle_id,
            velocity,
        } => {
            let (_, particle) = executor.particle(particle_id, record)?;
            executor
                .world
                .set_particle_velocity(particle, vec2(*velocity))
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::MarkForDestruction { particle_id } => {
            let (_, particle) = executor.particle(particle_id, record)?;
            executor
                .world
                .mark_particle_for_destruction(particle)
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::Compact { system_id } => {
            let system = executor.particle_system(system_id, record)?;
            executor
                .world
                .compact_pending_particles(system)
                .map_err(|error| action_error(record, error))?;
            executor
                .particles
                .retain(|(_, candidate_system, particle)| {
                    *candidate_system != system
                        || executor.world.particle_snapshot(*particle).is_ok()
                });
        }
        Phase9ParticleAction::ApplyForce {
            particle_ids,
            force,
        } => {
            let (system, particles) = executor.particle_range(particle_ids, record)?;
            executor
                .world
                .apply_particle_force_range(system, &particles, vec2(*force))
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::ApplyImpulse {
            particle_ids,
            impulse,
        } => {
            let (system, particles) = executor.particle_range(particle_ids, record)?;
            executor
                .world
                .apply_particle_linear_impulse_range(system, &particles, vec2(*impulse))
                .map_err(|error| action_error(record, error))?;
        }
        Phase9ParticleAction::RequestStatistics { system_id } => {
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
            maybe_observation = Some(Phase9ParticleObservation::Statistics {
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
                    body_contact_count: phase9_checked_u32(
                        statistics.body_contact_count(),
                        record,
                    )?,
                    stuck_particle_ids: stuck_particle_ids.into_boxed_slice(),
                    collision_energy_bits: FloatBits::new(statistics.collision_energy().to_bits()),
                    declared_capacity: phase9_checked_u32(statistics.declared_capacity(), record)?,
                    effective_capacity: phase9_checked_u32(
                        statistics.effective_capacity(),
                        record,
                    )?,
                },
            });
        }
        Phase9ParticleAction::QueryAabb {
            system_id,
            lower,
            upper,
        } => {
            let aabb = Aabb::new(vec2(*lower), vec2(*upper))
                .map_err(|error| action_error(record, error))?;
            let mut visited = Vec::new();
            if let Some(system_id) = system_id {
                let system = executor.particle_system(system_id, record)?;
                executor
                    .world
                    .query_particle_system_aabb(system, aabb, |occurrence| {
                        visited.push((occurrence.system(), occurrence.particle()));
                        QueryDirective::Continue
                    })
                    .map_err(|error| action_error(record, error))?;
            } else {
                executor
                    .world
                    .query_aabb_with_particles(aabb, |occurrence| {
                        if let WorldQueryOccurrence::Particle(particle) = occurrence {
                            visited.push((particle.system(), particle.particle()));
                        }
                        QueryDirective::Continue
                    })
                    .map_err(|error| action_error(record, error))?;
            }
            let particle_ids = visited
                .into_iter()
                .map(|(system, particle)| semantic_particle_id(executor, system, particle, record))
                .collect::<Result<Vec<_>, _>>()?;
            maybe_observation = Some(Phase9ParticleObservation::Query {
                terminated: false,
                particle_ids: particle_ids.into_boxed_slice(),
            });
        }
        Phase9ParticleAction::RayCast {
            system_id,
            start,
            end,
        } => {
            let input = RayCastInput::new(vec2(*start), vec2(*end), 1.0)
                .map_err(|error| action_error(record, error))?;
            let mut hits = Vec::new();
            if let Some(system_id) = system_id {
                let system = executor.particle_system(system_id, record)?;
                executor
                    .world
                    .ray_cast_particle_system(system, input, |hit| {
                        hits.push((hit.system(), hit.particle(), hit.fraction().get()));
                        RayCastDirective::Continue
                    })
                    .map_err(|error| action_error(record, error))?;
            } else {
                executor
                    .world
                    .ray_cast_with_particles(input, |occurrence| {
                        if let WorldRayCastOccurrence::Particle(hit) = occurrence {
                            hits.push((hit.system(), hit.particle(), hit.fraction().get()));
                        }
                        RayCastDirective::Continue
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
            maybe_observation = Some(Phase9ParticleObservation::RayCast {
                terminated: false,
                particle_ids: particle_ids.into_boxed_slice(),
                fractions_bits: fractions_bits.into_boxed_slice(),
            });
        }
    }
    executor
        .semantic_observations
        .push(liquidfun_test_protocol::RigidWorldObservation::Particle {
            observation: maybe_observation.unwrap_or_else(|| {
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
            }),
        });
    Ok(true)
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

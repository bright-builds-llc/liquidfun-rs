//! Phase 9 spatial queries and semantic identity helpers.

use super::{
    FloatBits, NativeRigidWorldError, ParticleCapacity, ParticleId, ParticleSystemDef,
    ParticleSystemId, Phase9Occurrence, Phase9OccurrenceKind, Phase9ParticleAction,
    Phase9ParticleBufferMode, Phase9ParticleObservation, Phase9ParticleSystemDeclaration,
    Phase9QueryControl, Phase9RayControl, QueryDirective, RayCastDirective, RayCastFraction,
    RigidWorldActionRecord, RigidWorldTimeline, ScenarioId, TimelineExecutor, Vec2Bits,
    WorldQueryOccurrence, WorldRayCastOccurrence, action_error, vec2,
};
use liquidfun::collision::{Aabb, RayCastInput};

pub(super) fn execute_spatial_query(
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

pub(super) fn mixed_state_observation(
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

pub(super) fn semantic_particle_id(
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

pub(super) fn semantic_body_id(
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

pub(super) fn semantic_fixture_id(
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
pub(super) fn phase9_lifecycle_observation(
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

pub(super) fn phase9_checked_u32(
    value: usize,
    record: &RigidWorldActionRecord,
) -> Result<u32, NativeRigidWorldError> {
    u32::try_from(value).map_err(|error| action_error(record, error))
}

pub(super) fn system_declaration<'a>(
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

pub(super) fn particle_declaration<'a>(
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

pub(super) fn particle_system_definition(
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
    pub(super) fn particle_system(
        &self,
        id: &ScenarioId,
        record: &RigidWorldActionRecord,
    ) -> Result<ParticleSystemId, NativeRigidWorldError> {
        self.particle_systems
            .iter()
            .find_map(|(candidate, system)| (candidate == id).then_some(*system))
            .ok_or_else(|| action_error(record, format!("unknown particle system `{id}`")))
    }

    pub(super) fn particle(
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

    pub(super) fn particle_range(
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

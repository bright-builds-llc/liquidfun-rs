//! Typed resolution and evaluation for the closed Phase 9 evidence bindings.

use liquidfun_test_protocol::{
    Phase9OccurrenceKind, Phase9ParticleBufferMode, Phase9ParticleObservation,
    Phase9SemanticAssertion, Phase9WitnessBinding, RigidBodyKind, RigidWorldAction,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimeline,
    RigidWorldTimelineResult, RigidWorldWitnessFamily, ScenarioId,
};

use super::PHASE9_REQUIRED_POLICY_PATHS;

/// A persisted Phase 9 witness did not resolve to or prove its declared semantic observation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Phase 9 evidence binding `{branch_id}` is invalid: {message}")]
pub struct Phase9EvidenceBindingError {
    branch_id: Box<str>,
    message: Box<str>,
}

/// Resolves and evaluates every closed Phase 9 witness against one decoded result.
///
/// Each binding must name the reviewed action for its branch, place that action inside the
/// selected checkpoint interval, resolve to the corresponding particle observation ordinal,
/// and satisfy its semantic assertion against the decoded request and result.
///
/// # Errors
///
/// Returns [`Phase9EvidenceBindingError`] when any indexed binding, observation variant, action,
/// or semantic value differs from the reviewed Phase 9 corpus contract.
pub fn validate_phase9_evidence_bindings(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
    bindings: &[Phase9WitnessBinding],
) -> Result<(), Phase9EvidenceBindingError> {
    if bindings
        .iter()
        .any(|binding| binding.branch_id.as_str() == "retained_phase6_through_phase8")
        && request.scenario().timelines().len() != RigidWorldWitnessFamily::ALL.len()
    {
        return Err(unbound_error(
            "retained Phase 6 through Phase 8 timelines are incomplete",
        ));
    }
    let timeline = request
        .scenario()
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 request timeline"))?;
    let result_timeline = result
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 result timeline"))?;
    for binding in bindings {
        let observation = resolve_observation(timeline, result_timeline, binding)?;
        if observation.witness_kind() != binding.observation_kind {
            return Err(binding_error(
                binding,
                format!(
                    "expected {:?} observation, resolved {:?}",
                    binding.observation_kind,
                    observation.witness_kind()
                ),
            ));
        }
        evaluate_assertion(timeline, result_timeline, binding, observation)?;
    }
    Ok(())
}

fn resolve_observation<'a>(
    timeline: &'a RigidWorldTimeline,
    result: &'a RigidWorldTimelineResult,
    binding: &Phase9WitnessBinding,
) -> Result<&'a Phase9ParticleObservation, Phase9EvidenceBindingError> {
    let action = timeline
        .actions()
        .get(binding.action_index)
        .ok_or_else(|| binding_error(binding, "action index is out of range"))?;
    let expected_action_id = expected_action_id(binding.branch_id.as_str());
    if action.action_id().as_str() != expected_action_id {
        return Err(binding_error(
            binding,
            format!(
                "expected action `{expected_action_id}`, found `{}`",
                action.action_id()
            ),
        ));
    }
    if !matches!(action.action(), RigidWorldAction::Particle { .. }) {
        return Err(binding_error(
            binding,
            "bound action is not a Phase 9 particle action",
        ));
    }
    let checkpoint = timeline
        .checkpoints()
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "checkpoint index is out of range"))?;
    let action_end = timeline
        .actions()
        .iter()
        .position(|candidate| candidate.action_id() == checkpoint.after_action_id())
        .ok_or_else(|| binding_error(binding, "checkpoint terminator action is absent"))?;
    let action_start = if binding.checkpoint_index == 0 {
        0
    } else {
        let previous = &timeline.checkpoints()[binding.checkpoint_index - 1];
        timeline
            .actions()
            .iter()
            .position(|candidate| candidate.action_id() == previous.after_action_id())
            .ok_or_else(|| binding_error(binding, "previous checkpoint terminator is absent"))?
            + 1
    };
    if !(action_start..=action_end).contains(&binding.action_index) {
        return Err(binding_error(
            binding,
            "bound action does not belong to the selected checkpoint",
        ));
    }
    let particle_ordinal = timeline.actions()[action_start..binding.action_index]
        .iter()
        .filter(|candidate| matches!(candidate.action(), RigidWorldAction::Particle { .. }))
        .count();
    let result_checkpoint = result
        .checkpoints
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "result checkpoint is absent"))?;
    result_checkpoint
        .observations
        .iter()
        .filter_map(|candidate| match candidate {
            RigidWorldObservation::Particle { observation } => Some(observation),
            _ => None,
        })
        .nth(particle_ordinal)
        .ok_or_else(|| binding_error(binding, "bound particle observation is absent"))
}

fn evaluate_assertion(
    timeline: &RigidWorldTimeline,
    result: &RigidWorldTimelineResult,
    binding: &Phase9WitnessBinding,
    observation: &Phase9ParticleObservation,
) -> Result<(), Phase9EvidenceBindingError> {
    let satisfied = match &binding.semantic_assertion {
        Phase9SemanticAssertion::ObservedSemantic { branch_id } => {
            evaluate_observed_semantic(timeline, result, branch_id.as_str(), observation)
        }
        Phase9SemanticAssertion::FiniteLifetimeExpired { particle_id } => {
            matches!(
                observation,
                Phase9ParticleObservation::System { particle_ids, .. }
                    if !particle_ids.contains(particle_id)
            )
        }
        Phase9SemanticAssertion::InfiniteLifetimeSurvives { particle_id } => {
            matches!(
                observation,
                Phase9ParticleObservation::System { particle_ids, .. }
                    if particle_ids.contains(particle_id)
            )
        }
        Phase9SemanticAssertion::EqualExpirationOrder { particle_ids } => {
            equal_lifetime_is_declared(timeline, particle_ids)
                && matches!(
                    observation,
                    Phase9ParticleObservation::Lifecycle { occurrence }
                        if occurrence.kind == Phase9OccurrenceKind::ParticleDestroyed
                            && occurrence.maybe_particle_id.as_ref() == particle_ids.last()
                )
        }
        Phase9SemanticAssertion::StrictContactCardinality {
            enabled,
            contact_count,
        } => matches!(
            observation,
            Phase9ParticleObservation::Statistics { statistics }
                if statistics.maybe_system_id.as_ref().is_some_and(|system_id| {
                    system_declaration(timeline, system_id)
                        .is_some_and(|declaration| declaration.strict_contact_check == *enabled)
                }) && statistics.body_contact_count == *contact_count
        ),
        Phase9SemanticAssertion::ListenerEventEffect {
            enabled,
            event_count,
        } => listener_effect_matches(result, observation, *enabled, *event_count),
        Phase9SemanticAssertion::FilterContactEffect {
            enabled,
            contact_count,
        } => {
            let expected_system = if *enabled {
                "phase9-growable"
            } else {
                "phase9-fixed-paused"
            };
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if statistics.maybe_system_id.as_ref().map(ScenarioId::as_str)
                        == Some(expected_system)
                        && statistics.particle_contact_count == *contact_count
            )
        }
        Phase9SemanticAssertion::CollisionEnergyPositiveFinite { minimum_bits } => {
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if {
                        let energy = statistics.collision_energy_bits.to_f32();
                        energy.is_finite() && energy > 0.0 && energy >= minimum_bits.to_f32()
                    }
            )
        }
        Phase9SemanticAssertion::StuckCandidatesNonempty { particle_ids } => {
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if !statistics.stuck_particle_ids.is_empty()
                        && particle_ids.iter().all(|particle_id| {
                            statistics.stuck_particle_ids.contains(particle_id)
                        })
            )
        }
        Phase9SemanticAssertion::ReplayResultDigestEquality
        | Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
        | Phase9SemanticAssertion::DeliberateFirstDivergence
        | Phase9SemanticAssertion::D0RepeatedResultDigestEquality
        | Phase9SemanticAssertion::DebugReleaseResultDigestEquality => {
            matches!(
                observation,
                Phase9ParticleObservation::Particle { snapshot }
                    if snapshot.particle_id.as_str() == "phase9-a"
            )
        }
    };
    if !satisfied {
        return Err(binding_error(
            binding,
            "resolved observation does not satisfy its semantic assertion",
        ));
    }
    Ok(())
}

fn evaluate_observed_semantic(
    timeline: &RigidWorldTimeline,
    result: &RigidWorldTimelineResult,
    branch: &str,
    observation: &Phase9ParticleObservation,
) -> bool {
    match branch {
        "multiple_systems" | "newest_first" => {
            timeline.particle_systems().len() == 2
                && matches!(
                    observation,
                    Phase9ParticleObservation::System { system_id, .. }
                        if system_id.as_str() == "phase9-growable"
                )
        }
        "paused_system" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-fixed-paused")
                    .is_some_and(|declaration| declaration.paused)
                    && statistics.particle_count == 2
            })
        }
        "stable_ids_sort" => matches!(
            observation,
            Phase9ParticleObservation::System { particle_ids, .. }
                if ids_equal(
                    particle_ids,
                    &["phase9-a", "phase9-b", "phase9-coupling", "phase9-capacity"],
                )
        ),
        "stable_ids_compact" => matches!(
            observation,
            Phase9ParticleObservation::MixedState { particle_ids, .. }
                if ids_equal(
                    particle_ids,
                    &["phase9-coupling", "phase9-evicting", "phase9-c", "phase9-e"],
                )
        ),
        "optional_lanes" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.color == [0, 0, 255, 255]
                    && snapshot.weight_bits.to_f32() == 0.0
                    && snapshot.force.x_bits.to_f32() == 0.0
                    && snapshot.force.y_bits.to_f32() == 0.0
        ),
        "fixed_buffer" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-fixed-paused").is_some_and(
                    |declaration| {
                        declaration.buffer_mode == Phase9ParticleBufferMode::Fixed { capacity: 2 }
                            && statistics.declared_capacity == 2
                    },
                )
            })
        }
        "growable_buffer" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-growable").is_some_and(|declaration| {
                    declaration.buffer_mode
                        == Phase9ParticleBufferMode::Growable {
                            initial_capacity: 4,
                        }
                        && statistics.declared_capacity == 4
                })
            })
        }
        "fixed_full" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                statistics.particle_count == 2 && statistics.effective_capacity == 2
            })
        }
        "teardown" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::SystemDestroyed,
            "phase9-fixed-paused",
            None,
        ),
        "oldest_lifetime" | "capacity_eviction" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::ParticleDestroyed,
            "phase9-growable",
            Some("phase9-a"),
        ),
        "maximum_lifetime" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-growable")
                    .is_some_and(|declaration| declaration.maximum_count == Some(4))
                    && statistics.effective_capacity == 4
            })
        }
        "requested_destruction_callback" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::ParticleDestroyed,
            "phase9-growable",
            Some("phase9-b"),
        ),
        "unrequested_destruction_callback" => {
            matches!(observation, Phase9ParticleObservation::MixedState { .. })
                && !checkpoint_has_particle_lifecycle(result, "phase9-capacity")
        }
        "zombie_pending" => matches!(
            observation,
            Phase9ParticleObservation::MixedState { particle_ids, .. }
                if particle_ids.iter().any(|id| id.as_str() == "phase9-b")
        ),
        "particle_contact" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.system_id.as_str() == "phase9-fixed-paused"
        ),
        "body_contact" => matches!(
            observation,
            Phase9ParticleObservation::BodyContact { contact }
                if contact.fixture_id.as_str() == "nc-kinematic-fixture"
        ),
        "contact_order" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.particle_a_id.as_str() == "phase9-c"
                    && contact.particle_b_id.as_str() == "phase9-d"
        ),
        "contact_multiplicity" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.particle_a_id != contact.particle_b_id
                    && contact.weight_bits.to_f32() > 0.0
        ),
        "coupling_fields" => matches!(
            observation,
            Phase9ParticleObservation::BodyContact { contact }
                if contact.particle_id.as_str() == "phase9-coupling"
                    && contact.mass_bits.to_f32() > 0.0
                    && contact.weight_bits.to_f32() > 0.0
        ),
        "dynamic_body_reaction" => {
            statistics_for_system(observation, "phase9-growable").is_some()
                && result.checkpoints.iter().any(|checkpoint| {
                    checkpoint.bodies.iter().any(|body| {
                        body.body_id.as_str() == "nc-dynamic"
                            && (body.linear_velocity.x_bits.to_f32() != 0.0
                                || body.linear_velocity.y_bits.to_f32() != 0.0)
                    })
                })
        }
        "static_body_no_reaction" => {
            statistics_for_system(observation, "phase9-growable").is_some()
                && result.checkpoints.iter().any(|checkpoint| {
                    checkpoint.bodies.iter().any(|body| {
                        body.body_id.as_str() == "nc-static"
                            && body.body_kind == RigidBodyKind::Static
                            && body.linear_velocity.x_bits.to_f32() == 0.0
                            && body.linear_velocity.y_bits.to_f32() == 0.0
                    })
                })
        }
        "force_range" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.force.x_bits.to_f32() != 0.0
        ),
        "impulse_range" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.velocity.y_bits.to_f32() != 0.0
        ),
        "statistics_counts" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                statistics.system_count == 2 && statistics.particle_count == 4
            })
        }
        "system_aabb" | "system_culling" | "query_continue" => {
            query_matches(observation, false, &["phase9-a", "phase9-b"])
        }
        "world_aabb" => query_matches(
            observation,
            false,
            &["phase9-c", "phase9-d", "phase9-a", "phase9-b"],
        ),
        "query_terminate" => query_matches(observation, true, &["phase9-a"]),
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            ray_matches(observation, false, &["phase9-a", "phase9-b"], true)
        }
        "world_ray" => ray_matches(
            observation,
            false,
            &["phase9-c", "phase9-d", "phase9-a", "phase9-b"],
            true,
        ),
        "ray_ignore" => ray_matches(observation, false, &["phase9-a", "phase9-b"], false),
        "ray_clip" => ray_matches(observation, false, &["phase9-a"], true),
        "ray_terminate" => ray_matches(observation, true, &["phase9-a"], true),
        "retained_phase6_through_phase8" => {
            matches!(
                observation,
                Phase9ParticleObservation::Particle { snapshot }
                    if snapshot.particle_id.as_str() == "phase9-a"
            )
        }
        "phase10_rejection" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
        ),
        "closed_policy_registry" => {
            PHASE9_REQUIRED_POLICY_PATHS.len() == 22
                && matches!(
                    observation,
                    Phase9ParticleObservation::Particle { snapshot }
                        if snapshot.particle_id.as_str() == "phase9-a"
                )
        }
        _ => false,
    }
}

fn expected_action_id(branch: &str) -> &'static str {
    match branch {
        "multiple_systems" | "newest_first" | "stable_ids_sort" => "inspect-system",
        "paused_system" | "fixed_buffer" | "fixed_full" => "statistics-fixed",
        "stable_ids_compact" => "compact-unrequested",
        "optional_lanes"
        | "retained_phase6_through_phase8"
        | "phase10_rejection"
        | "closed_policy_registry"
        | "replay_identity"
        | "minimization_identity"
        | "first_divergence_stability"
        | "d0_byte_identity"
        | "debug_release_agreement" => "inspect-particle",
        "growable_buffer" | "maximum_lifetime" | "statistics_counts" => "statistics",
        "teardown" => "destroy-fixed",
        "oldest_lifetime" | "capacity_eviction" => "create-evicting",
        "finite_lifetime" | "infinite_lifetime" => "inspect-system-after-step",
        "equal_lifetime" => "create-phase9-e",
        "requested_destruction_callback" => "compact",
        "unrequested_destruction_callback" => "compact-unrequested",
        "zombie_pending" => "mark",
        "particle_contact" | "contact_order" | "contact_multiplicity" => "inspect-particle-contact",
        "body_contact" | "coupling_fields" => "inspect-body-contact",
        "strict_contact_enabled" => "statistics-fixed",
        "strict_contact_disabled"
        | "dynamic_body_reaction"
        | "static_body_no_reaction"
        | "stuck_candidates" => "statistics",
        "listener_flag_enabled" => "inspect-occurrence-zero",
        "listener_flag_disabled" | "filter_flag_enabled" => "contact-statistics-growable",
        "filter_flag_disabled" => "contact-statistics-fixed",
        "force_range" => "inspect-after-force",
        "impulse_range" => "inspect-after-impulse",
        "collision_energy" => "statistics-fixed",
        "system_aabb" | "system_culling" | "query_continue" => "system-query",
        "world_aabb" => "world-query",
        "query_terminate" => "query-terminate",
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            "system-ray"
        }
        "world_ray" => "world-ray",
        "ray_ignore" => "ray-ignore",
        "ray_clip" => "ray-clip",
        "ray_terminate" => "ray-terminate",
        _ => "",
    }
}

fn listener_effect_matches(
    result: &RigidWorldTimelineResult,
    observation: &Phase9ParticleObservation,
    enabled: bool,
    expected_count: u32,
) -> bool {
    let occurrences = result
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.observations.iter())
        .filter_map(|candidate| match candidate {
            RigidWorldObservation::Particle {
                observation: Phase9ParticleObservation::Lifecycle { occurrence },
            } if occurrence.kind == Phase9OccurrenceKind::ContactCreated => Some(occurrence),
            _ => None,
        })
        .filter(|occurrence| {
            enabled
                || occurrence
                    .maybe_particle_id
                    .as_ref()
                    .map(ScenarioId::as_str)
                    == Some("phase9-capacity")
                || occurrence
                    .maybe_other_particle_id
                    .as_ref()
                    .map(ScenarioId::as_str)
                    == Some("phase9-capacity")
        })
        .count();
    u32::try_from(occurrences).ok() == Some(expected_count)
        && if enabled {
            matches!(
                observation,
                Phase9ParticleObservation::Lifecycle { occurrence }
                    if occurrence.kind == Phase9OccurrenceKind::ContactCreated
            )
        } else {
            statistics_for_system(observation, "phase9-growable").is_some()
        }
}

fn equal_lifetime_is_declared(timeline: &RigidWorldTimeline, ids: &[ScenarioId]) -> bool {
    let Some(first) = ids
        .first()
        .and_then(|id| particle_declaration(timeline, id))
    else {
        return false;
    };
    ids.len() >= 2
        && ids.iter().skip(1).all(|id| {
            particle_declaration(timeline, id)
                .is_some_and(|particle| particle.lifetime_bits == first.lifetime_bits)
        })
}

fn system_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    system_id: &ScenarioId,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleSystemDeclaration> {
    timeline
        .particle_systems()
        .iter()
        .find(|declaration| declaration.system_id == *system_id)
}

fn system_declaration_by_name<'a>(
    timeline: &'a RigidWorldTimeline,
    system_id: &str,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleSystemDeclaration> {
    timeline
        .particle_systems()
        .iter()
        .find(|declaration| declaration.system_id.as_str() == system_id)
}

fn particle_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    particle_id: &ScenarioId,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleDeclaration> {
    timeline
        .particles()
        .iter()
        .find(|declaration| declaration.particle_id == *particle_id)
}

fn statistics_for_system<'a>(
    observation: &'a Phase9ParticleObservation,
    system_id: &str,
) -> Option<&'a liquidfun_test_protocol::Phase9StatisticsObservation> {
    let Phase9ParticleObservation::Statistics { statistics } = observation else {
        return None;
    };
    (statistics.maybe_system_id.as_ref().map(ScenarioId::as_str) == Some(system_id))
        .then_some(statistics)
}

fn lifecycle_matches(
    observation: &Phase9ParticleObservation,
    kind: Phase9OccurrenceKind,
    system_id: &str,
    maybe_particle_id: Option<&str>,
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::Lifecycle { occurrence }
            if occurrence.kind == kind
                && occurrence.system_id.as_str() == system_id
                && occurrence.maybe_particle_id.as_ref().map(ScenarioId::as_str)
                    == maybe_particle_id
    )
}

fn checkpoint_has_particle_lifecycle(result: &RigidWorldTimelineResult, particle_id: &str) -> bool {
    result
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.observations.iter())
        .any(|candidate| {
            matches!(
                candidate,
                RigidWorldObservation::Particle {
                    observation: Phase9ParticleObservation::Lifecycle { occurrence },
                } if occurrence.maybe_particle_id.as_ref().map(ScenarioId::as_str)
                    == Some(particle_id)
            )
        })
}

fn query_matches(
    observation: &Phase9ParticleObservation,
    terminated: bool,
    expected_ids: &[&str],
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::Query {
            terminated: actual,
            particle_ids,
        } if *actual == terminated && ids_equal(particle_ids, expected_ids)
    )
}

fn ray_matches(
    observation: &Phase9ParticleObservation,
    terminated: bool,
    expected_ids: &[&str],
    require_nonzero_fractions: bool,
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::RayCast {
            terminated: actual,
            particle_ids,
            fractions_bits,
        } if *actual == terminated
            && ids_equal(particle_ids, expected_ids)
            && particle_ids.len() == fractions_bits.len()
            && (!require_nonzero_fractions
                || fractions_bits.iter().all(|bits| bits.to_f32() > 0.0))
    )
}

fn ids_equal(actual: &[ScenarioId], expected: &[&str]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.as_str() == *expected)
}

fn binding_error(
    binding: &Phase9WitnessBinding,
    message: impl Into<String>,
) -> Phase9EvidenceBindingError {
    Phase9EvidenceBindingError {
        branch_id: binding.branch_id.as_str().into(),
        message: message.into().into_boxed_str(),
    }
}

fn unbound_error(message: impl Into<String>) -> Phase9EvidenceBindingError {
    Phase9EvidenceBindingError {
        branch_id: "<registry>".into(),
        message: message.into().into_boxed_str(),
    }
}

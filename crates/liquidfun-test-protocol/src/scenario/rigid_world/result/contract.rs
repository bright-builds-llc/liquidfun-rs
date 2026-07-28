use std::collections::{HashMap, HashSet};

use super::phase8::{
    ExpectedObservation, expected_observation, validate_phase8_observation_contract,
};
use super::phase9::Phase9ResultState;
use super::phase10::Phase10ResultState;
use super::{
    MAXIMUM_MANIFOLD_POINTS, MAXIMUM_QUERY_OCCURRENCES, MAXIMUM_RAY_HITS, MAXIMUM_RESULT_AGGREGATE,
    MAXIMUM_RESULT_BODIES, MAXIMUM_RESULT_CHECKPOINTS, MAXIMUM_RESULT_CONTACTS,
    MAXIMUM_RESULT_DESTRUCTIONS, MAXIMUM_RESULT_EVENTS, MAXIMUM_RESULT_FIXTURES,
    MAXIMUM_RESULT_OBSERVATIONS, MAXIMUM_RESULT_TIMELINES, Phase10Observation,
    RigidCheckpointLiveIdentities, RigidExpectedCounts, RigidWorldCheckpointResult,
    RigidWorldObservation, RigidWorldResultRecord, fixture_belongs_to_live_body,
};
use crate::{
    HarnessLimits, PHASE9_MAXIMUM_IDENTITIES, Phase9ParticleObservation, Phase10Operation,
    RecordLimit, RigidWorldAction, RigidWorldDecodeError, RigidWorldErrorKind,
    RigidWorldRequestRecord, ScenarioId, decode_jsonl,
};

/// Decodes one newline-complete bounded rigid-world result record.
///
/// # Errors
///
/// Returns [`RigidWorldDecodeError`] for framing, closed-field, version, result
/// count, sensor-manifold, or aggregate-limit failures.
pub fn decode_rigid_world_result_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<RigidWorldResultRecord, RigidWorldDecodeError> {
    let result = decode_jsonl::<RigidWorldResultRecord>(bytes, limits, RecordLimit::Output)?;
    validate_result_bounds(&result)?;
    Ok(result)
}

/// Validates result identity, checkpoints, counts, and declaration ordering.
///
/// # Errors
///
/// Returns [`RigidWorldDecodeError`] when the result disagrees with the request
/// declaration or checkpoint contract.
pub fn validate_rigid_world_result_against_request(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
) -> Result<(), RigidWorldDecodeError> {
    if result.request_id() != request.request_id()
        || result.scenario_id() != request.scenario().scenario_id()
        || result.timelines().len() != request.scenario().timelines().len()
    {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::ResultTimelineMismatch,
        ));
    }

    for (expected_timeline, actual_timeline) in request
        .scenario()
        .timelines()
        .iter()
        .zip(result.timelines())
    {
        if expected_timeline.witness_family() != actual_timeline.witness_family
            || expected_timeline.checkpoints().len() != actual_timeline.checkpoints.len()
        {
            return Err(crate::scenario::rigid_world::validation(
                RigidWorldErrorKind::ResultTimelineMismatch,
            ));
        }

        for (checkpoint_index, (expected, actual)) in expected_timeline
            .checkpoints()
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            if expected.checkpoint_id() != &actual.checkpoint_id
                || expected.phase() != actual.phase.as_ref()
                || expected.counts() != actual.counts
            {
                return Err(crate::scenario::rigid_world::validation(
                    RigidWorldErrorKind::ResultCheckpointMismatch,
                ));
            }
            let live_identities =
                rigid_world_checkpoint_live_identities(expected_timeline, checkpoint_index)
                    .ok_or_else(|| {
                        crate::scenario::rigid_world::validation(
                            RigidWorldErrorKind::ResultCheckpointMismatch,
                        )
                    })?;
            validate_checkpoint_declaration_order(&live_identities, actual)?;
            validate_checkpoint_observations(
                expected_timeline,
                checkpoint_index,
                &actual.observations,
            )?;
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "the retained phase observation ordering contract is kept in one explicit walk"
)]
fn validate_checkpoint_observations(
    timeline: &crate::RigidWorldTimeline,
    checkpoint_index: usize,
    observations: &[RigidWorldObservation],
) -> Result<(), RigidWorldDecodeError> {
    let actions =
        rigid_world_checkpoint_action_window(timeline, checkpoint_index).ok_or_else(|| {
            crate::scenario::rigid_world::validation(RigidWorldErrorKind::ResultCheckpointMismatch)
        })?;
    validate_phase8_observation_contract(timeline.witness_family(), actions, observations)?;
    let first_action = actions.first().ok_or_else(|| {
        crate::scenario::rigid_world::validation(RigidWorldErrorKind::ResultCheckpointMismatch)
    })?;
    let action_start = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == first_action.action_id())
        .ok_or_else(|| {
            crate::scenario::rigid_world::validation(RigidWorldErrorKind::ResultCheckpointMismatch)
        })?;
    let fixture_owners = rigid_fixture_owners(timeline);
    let mut live_bodies = HashSet::new();
    let mut live_fixtures = HashSet::new();
    let mut created_body_ids = HashSet::new();
    let mut phase9_state = Phase9ResultState::new(timeline);
    let mut phase10_state = Phase10ResultState::default();
    for action in &timeline.actions()[..action_start] {
        crate::scenario::rigid_world::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
        if let RigidWorldAction::CreateBody { body_id } = action.action() {
            created_body_ids.insert(body_id.clone());
        }
        if let RigidWorldAction::Particle { action } = action.action() {
            phase9_state.apply(action);
        } else if matches!(action.action(), RigidWorldAction::Step { .. }) {
            phase9_state.advance_step();
        }
        if let RigidWorldAction::ParticleGroup { operation } = action.action() {
            phase10_state.apply(operation);
        }
    }

    let mut actual_observations = observations.iter().peekable();
    for action in actions {
        crate::scenario::rigid_world::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
        if let RigidWorldAction::CreateBody { body_id } = action.action() {
            created_body_ids.insert(body_id.clone());
        }
        if let RigidWorldAction::Particle {
            action: particle_action,
        } = action.action()
        {
            phase9_state.apply(particle_action);
            let Some(actual) = actual_observations.next() else {
                return Err(crate::scenario::rigid_world::validation(
                    RigidWorldErrorKind::ResultObservationMismatch,
                ));
            };
            let live_identities =
                rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
            phase9_state.validate(particle_action, &live_identities, actual)?;
            continue;
        }
        if let RigidWorldAction::ParticleGroup { operation } = action.action() {
            phase10_state.apply(operation);
            if matches!(operation, Phase10Operation::InspectState) {
                let Some(actual) = actual_observations.next() else {
                    return Err(crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::ResultObservationMismatch,
                    ));
                };
                let RigidWorldObservation::ParticleGroup { observation } = actual else {
                    return Err(crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::ResultObservationMismatch,
                    ));
                };
                let Phase10Observation::State { state } = observation;
                let live_identities =
                    rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
                if state.body_contacts.iter().any(|contact| {
                    !fixture_belongs_to_live_body(
                        &live_identities.body_ids,
                        live_identities
                            .fixtures
                            .iter()
                            .map(|fixture| (fixture.fixture_id(), fixture.owner_body_id())),
                        &contact.fixture_id,
                        &contact.body_id,
                    )
                }) {
                    return Err(crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::ResultObservationMismatch,
                    ));
                }
                phase10_state
                    .validate(state, &created_body_ids)
                    .map_err(|_| {
                        crate::scenario::rigid_world::validation(
                            RigidWorldErrorKind::ResultObservationMismatch,
                        )
                    })?;
            }
            continue;
        }
        if matches!(action.action(), RigidWorldAction::Step { .. }) {
            phase9_state.advance_step();
        }
        let Some(expected) = expected_observation(action.action()) else {
            continue;
        };
        let live_identities = rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
        while matches!(
            actual_observations.peek(),
            Some(RigidWorldObservation::Lifecycle { .. })
        ) {
            actual_observations.next();
        }
        if matches!(expected, ExpectedObservation::Reconstruction) {
            let mut ordinal = 0_u32;
            while let Some(RigidWorldObservation::Reconstruction { record }) =
                actual_observations.peek()
            {
                if record.ordinal != ordinal {
                    return Err(crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::ResultObservationMismatch,
                    ));
                }
                ordinal = ordinal.checked_add(1).ok_or_else(|| {
                    crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::AggregateLimitExceeded,
                    )
                })?;
                actual_observations.next();
            }
            if ordinal == 0 {
                return Err(crate::scenario::rigid_world::validation(
                    RigidWorldErrorKind::ResultObservationMismatch,
                ));
            }
            continue;
        }
        let Some(actual) = actual_observations.next() else {
            return Err(crate::scenario::rigid_world::validation(
                RigidWorldErrorKind::ResultObservationMismatch,
            ));
        };
        if !expected.matches(&live_identities, actual) {
            return Err(crate::scenario::rigid_world::validation(
                RigidWorldErrorKind::ResultObservationMismatch,
            ));
        }
    }
    if actual_observations
        .any(|observation| !matches!(observation, RigidWorldObservation::Lifecycle { .. }))
    {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::ResultObservationMismatch,
        ));
    }
    Ok(())
}

fn validate_checkpoint_declaration_order(
    expected: &RigidCheckpointLiveIdentities<'_>,
    checkpoint: &RigidWorldCheckpointResult,
) -> Result<(), RigidWorldDecodeError> {
    let actual_bodies = checkpoint
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    if expected.body_ids != actual_bodies {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }

    let actual_fixtures = checkpoint
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    if expected.fixture_ids != actual_fixtures {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }
    Ok(())
}

/// Returns the actions owned by one checkpoint, excluding actions owned by the prior checkpoint.
#[must_use]
pub fn rigid_world_checkpoint_action_window(
    timeline: &crate::RigidWorldTimeline,
    checkpoint_index: usize,
) -> Option<&[crate::RigidWorldActionRecord]> {
    let checkpoint = timeline.checkpoints().get(checkpoint_index)?;
    let action_end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == checkpoint.after_action_id())?;
    let action_start = if checkpoint_index == 0 {
        0
    } else {
        let previous = timeline.checkpoints().get(checkpoint_index - 1)?;
        timeline
            .actions()
            .iter()
            .position(|action| action.action_id() == previous.after_action_id())?
            .checked_add(1)?
    };
    timeline.actions().get(action_start..=action_end)
}

/// Replays lifecycle actions through one checkpoint and returns exact live identities.
#[must_use]
pub fn rigid_world_checkpoint_live_identities(
    timeline: &crate::RigidWorldTimeline,
    checkpoint_index: usize,
) -> Option<RigidCheckpointLiveIdentities<'_>> {
    let action_window = rigid_world_checkpoint_action_window(timeline, checkpoint_index)?;
    let action_end = action_window.last()?.action_id();
    let action_end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == action_end)?;
    let fixture_owners = rigid_fixture_owners(timeline);
    let mut live_bodies = HashSet::new();
    let mut live_fixtures = HashSet::new();
    for action in &timeline.actions()[..=action_end] {
        crate::scenario::rigid_world::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
    }
    Some(rigid_world_live_identities(
        timeline,
        &live_bodies,
        &live_fixtures,
    ))
}

fn rigid_fixture_owners(timeline: &crate::RigidWorldTimeline) -> HashMap<ScenarioId, ScenarioId> {
    timeline
        .fixtures()
        .iter()
        .map(|fixture| {
            (
                fixture.fixture_id().clone(),
                fixture.owner_body_id().clone(),
            )
        })
        .collect()
}

fn rigid_world_live_identities<'a>(
    timeline: &'a crate::RigidWorldTimeline,
    live_bodies: &HashSet<ScenarioId>,
    live_fixtures: &HashSet<ScenarioId>,
) -> RigidCheckpointLiveIdentities<'a> {
    let fixtures = timeline
        .fixtures()
        .iter()
        .filter(|fixture| live_fixtures.contains(fixture.fixture_id()))
        .collect::<Vec<_>>();
    RigidCheckpointLiveIdentities {
        body_ids: timeline
            .bodies()
            .iter()
            .map(crate::RigidBodyDeclaration::body_id)
            .filter(|body_id| live_bodies.contains(*body_id))
            .collect(),
        fixture_ids: fixtures
            .iter()
            .map(|fixture| fixture.fixture_id())
            .collect(),
        fixtures,
    }
}

pub(super) fn validate_result_bounds(
    result: &RigidWorldResultRecord,
) -> Result<(), RigidWorldDecodeError> {
    if result.timelines.is_empty() || result.timelines.len() > MAXIMUM_RESULT_TIMELINES {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::AggregateLimitExceeded,
        ));
    }
    let mut aggregate = 0_usize;
    for timeline in &result.timelines {
        if timeline.checkpoints.is_empty()
            || timeline.checkpoints.len() > MAXIMUM_RESULT_CHECKPOINTS
        {
            return Err(crate::scenario::rigid_world::validation(
                RigidWorldErrorKind::AggregateLimitExceeded,
            ));
        }
        for checkpoint in &timeline.checkpoints {
            let manifold_points = validate_checkpoint_bounds(checkpoint)?;
            aggregate = aggregate
                .checked_add(checkpoint_aggregate_size(checkpoint, manifold_points))
                .ok_or_else(|| {
                    crate::scenario::rigid_world::validation(
                        RigidWorldErrorKind::AggregateLimitExceeded,
                    )
                })?;
            if aggregate > MAXIMUM_RESULT_AGGREGATE {
                return Err(crate::scenario::rigid_world::validation(
                    RigidWorldErrorKind::AggregateLimitExceeded,
                ));
            }
        }
    }
    Ok(())
}

fn validate_checkpoint_bounds(
    checkpoint: &RigidWorldCheckpointResult,
) -> Result<usize, RigidWorldDecodeError> {
    if checkpoint.bodies.len() > MAXIMUM_RESULT_BODIES
        || checkpoint.fixtures.len() > MAXIMUM_RESULT_FIXTURES
        || checkpoint.contacts.len() > MAXIMUM_RESULT_CONTACTS
        || checkpoint.events.len() > MAXIMUM_RESULT_EVENTS
        || checkpoint.destructions.len() > MAXIMUM_RESULT_DESTRUCTIONS
        || checkpoint.observations.len() > MAXIMUM_RESULT_OBSERVATIONS
    {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::AggregateLimitExceeded,
        ));
    }
    for observation in &checkpoint.observations {
        if let RigidWorldObservation::ParticleGroup { observation } = observation {
            observation.validate().map_err(|_| {
                crate::scenario::rigid_world::validation(
                    RigidWorldErrorKind::InvalidParticleGroupResult,
                )
            })?;
        }
    }
    let manifold_points = checkpoint
        .contacts
        .iter()
        .map(|contact| {
            contact
                .maybe_manifold
                .as_ref()
                .map_or(0, |manifold| manifold.points.len())
        })
        .sum::<usize>();
    if checkpoint
        .observations
        .iter()
        .any(observation_exceeds_bounds)
    {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::AggregateLimitExceeded,
        ));
    }
    if checkpoint.contacts.iter().any(|contact| {
        contact
            .maybe_manifold
            .as_ref()
            .is_some_and(|manifold| manifold.points.len() > MAXIMUM_MANIFOLD_POINTS)
            || contact.sensor && contact.maybe_manifold.is_some()
    }) {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::InvalidGeometry,
        ));
    }
    let actual_counts = RigidExpectedCounts {
        bodies: checked_u32(checkpoint.bodies.len())?,
        fixtures: checked_u32(checkpoint.fixtures.len())?,
        contacts: checked_u32(checkpoint.contacts.len())?,
        manifold_points: checked_u32(manifold_points)?,
        events: checked_u32(checkpoint.events.len())?,
        destructions: checked_u32(checkpoint.destructions.len())?,
    };
    if actual_counts != checkpoint.counts {
        return Err(crate::scenario::rigid_world::validation(
            RigidWorldErrorKind::ExpectedCountMismatch,
        ));
    }
    Ok(manifold_points)
}

fn observation_exceeds_bounds(observation: &RigidWorldObservation) -> bool {
    match observation {
        RigidWorldObservation::Query { observation } => {
            observation.occurrences.len() > MAXIMUM_QUERY_OCCURRENCES
        }
        RigidWorldObservation::RayCast { observation } => observation.hits.len() > MAXIMUM_RAY_HITS,
        RigidWorldObservation::Rope { snapshot } => {
            snapshot.vertices.len() > crate::RIGID_WORLD_MAXIMUM_ROPE_VERTICES
        }
        RigidWorldObservation::Particle { observation } => {
            phase9_observation_exceeds_bounds(observation)
        }
        _ => false,
    }
}

fn checkpoint_aggregate_size(
    checkpoint: &RigidWorldCheckpointResult,
    manifold_points: usize,
) -> usize {
    checkpoint.bodies.len()
        + checkpoint.fixtures.len()
        + checkpoint.contacts.len()
        + manifold_points
        + checkpoint.events.len()
        + checkpoint.destructions.len()
        + checkpoint.observations.len()
}

fn phase9_observation_exceeds_bounds(observation: &Phase9ParticleObservation) -> bool {
    match observation {
        Phase9ParticleObservation::System { particle_ids, .. }
        | Phase9ParticleObservation::Query { particle_ids, .. }
        | Phase9ParticleObservation::MixedState { particle_ids, .. } => {
            particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
        }
        Phase9ParticleObservation::RayCast {
            particle_ids,
            fractions_bits,
            ..
        } => {
            particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
                || fractions_bits.len() != particle_ids.len()
        }
        Phase9ParticleObservation::Statistics { statistics } => {
            statistics.stuck_particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
        }
        Phase9ParticleObservation::Particle { .. }
        | Phase9ParticleObservation::Lifecycle { .. }
        | Phase9ParticleObservation::ParticleContact { .. }
        | Phase9ParticleObservation::BodyContact { .. } => false,
    }
}

pub(super) fn observations_are_empty(observations: &[RigidWorldObservation]) -> bool {
    observations.is_empty()
}

fn checked_u32(value: usize) -> Result<u32, RigidWorldDecodeError> {
    u32::try_from(value).map_err(|_| {
        crate::scenario::rigid_world::validation(RigidWorldErrorKind::AggregateLimitExceeded)
    })
}

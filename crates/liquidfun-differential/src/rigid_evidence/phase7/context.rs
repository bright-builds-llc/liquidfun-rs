//! Maps checkpoint observations back to their validated emitting actions.

use liquidfun_test_protocol::{
    RigidWorldAction, RigidWorldCheckpointResult, RigidWorldObservation, RigidWorldRequestRecord,
};

use super::super::{EvidenceContext, Location, RigidCompletionContext};

pub(super) fn checkpoint_context<'a>(
    request: &'a RigidWorldRequestRecord,
    location: Location,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> EvidenceContext<'a> {
    let (action_id, stage) = checkpoint_step_action(request, location)
        .unwrap_or_else(|| checkpoint_action(request, location));
    EvidenceContext {
        location,
        maybe_action_id: Some(action_id),
        maybe_stage: Some(stage),
        maybe_entity: None,
        maybe_completion_context: last_completion(expected, actual),
    }
}

fn checkpoint_step_action(
    request: &RigidWorldRequestRecord,
    location: Location,
) -> Option<(&str, &str)> {
    let timeline = &request.scenario().timelines()[location.timeline_index];
    let checkpoint = &timeline.checkpoints()[location.checkpoint_index];
    let end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == checkpoint.after_action_id())?;
    timeline.actions()[..=end]
        .iter()
        .rev()
        .find(|action| matches!(action.action(), RigidWorldAction::ConfiguredStep { .. }))
        .map(|action| (action.action_id().as_str(), action.phase()))
}

pub(super) fn checkpoint_action(
    request: &RigidWorldRequestRecord,
    location: Location,
) -> (&str, &str) {
    let timeline = &request.scenario().timelines()[location.timeline_index];
    let checkpoint = &timeline.checkpoints()[location.checkpoint_index];
    let action = timeline
        .actions()
        .iter()
        .find(|action| action.action_id() == checkpoint.after_action_id())
        .expect("validated checkpoint action exists");
    (action.action_id().as_str(), action.phase())
}

pub(super) fn observation_action(
    request: &RigidWorldRequestRecord,
    location: Location,
    observation_index: usize,
) -> Option<(&str, &str)> {
    let timeline = &request.scenario().timelines()[location.timeline_index];
    let checkpoint = &timeline.checkpoints()[location.checkpoint_index];
    let end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == checkpoint.after_action_id())?;
    timeline.actions()[..=end]
        .iter()
        .filter(|action| action_emits_observation(action.action()))
        .nth(observation_index)
        .map(|action| (action.action_id().as_str(), action.phase()))
}

fn action_emits_observation(action: &RigidWorldAction) -> bool {
    matches!(
        action,
        RigidWorldAction::SetLinearVelocity { .. }
            | RigidWorldAction::SetAngularVelocity { .. }
            | RigidWorldAction::ApplyForce { .. }
            | RigidWorldAction::ApplyTorque { .. }
            | RigidWorldAction::ApplyLinearImpulse { .. }
            | RigidWorldAction::ApplyAngularImpulse { .. }
            | RigidWorldAction::SetBodyDamping { .. }
            | RigidWorldAction::SetGravityScale { .. }
            | RigidWorldAction::SetFixedRotation { .. }
            | RigidWorldAction::SetSleepingAllowed { .. }
            | RigidWorldAction::SetAwake { .. }
            | RigidWorldAction::SetBullet { .. }
            | RigidWorldAction::ConfiguredStep { .. }
            | RigidWorldAction::QueryAabb { .. }
            | RigidWorldAction::RayCast { .. }
            | RigidWorldAction::ShiftOrigin { .. }
    )
}

pub(super) fn observation_entity(observation: &RigidWorldObservation) -> Option<&str> {
    match observation {
        RigidWorldObservation::BodyState { state } => Some(state.body_id.as_str()),
        _ => None,
    }
}

fn last_completion(
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Option<RigidCompletionContext> {
    expected
        .observations
        .iter()
        .rev()
        .zip(actual.observations.iter().rev())
        .find_map(|(expected, actual)| match (expected, actual) {
            (
                RigidWorldObservation::Step { outcome: expected },
                RigidWorldObservation::Step { outcome: actual },
            ) => Some(RigidCompletionContext::new(*expected, *actual)),
            _ => None,
        })
}

mod primitives;

use liquidfun::{DebugDrawOptions, WorldObservationLimits};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointPosition, FloatBits, HarnessLimits, RequestId, ResolvedScenario,
    ScenarioId, StructuralObservation, StructuralValue, decode_canonical_checkpoint_jsonl,
    encode_canonical_checkpoint_jsonl,
};

use crate::session::SessionCheckpointIdentity;
use crate::{SessionBackendError, SessionBackendErrorCategory};

use super::executor::NativeSession;
use primitives::encode_debug_primitives;

pub(super) fn capture_checkpoint(
    maybe_request_id: Option<&RequestId>,
    resolved: &ResolvedScenario,
    session: &NativeSession,
    checkpoint: &SessionCheckpointIdentity,
) -> Result<CanonicalCheckpoint, SessionBackendError> {
    let observation = session
        .world
        .world_observation(WorldObservationLimits::reviewed())
        .map_err(|_error| resource_failure())?;
    let primitives = session
        .world
        .collect_debug_primitives(DebugDrawOptions::all())
        .map_err(|_error| resource_failure())?;
    let diagnostics = observation.diagnostics();
    let mut observations = vec![
        structural("world-body-count", diagnostics.body_count())?,
        structural("world-contact-count", diagnostics.contact_count())?,
        structural("world-debug-primitive-count", primitives.primitives().len())?,
        structural("world-fixture-count", diagnostics.fixture_count())?,
        structural("world-joint-count", diagnostics.joint_count())?,
        structural("world-particle-count", observation.particles().len())?,
    ];
    observations.sort_unstable_by(|left, right| left.observation_id().cmp(right.observation_id()));
    let debug_primitives = encode_debug_primitives(session, &primitives)?;
    let checkpoint = CanonicalCheckpoint::new(
        maybe_request_id.cloned().unwrap_or(
            RequestId::new("catalog-native-request").map_err(|_error| capture_failure())?,
        ),
        resolved.identity().content_sha256().clone(),
        checkpoint.checkpoint_id().clone(),
        CheckpointPosition::LogicalStep {
            ordinal: checkpoint.logical_step(),
        },
        FloatBits::from_f32(session.simulation_time),
        observations,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        debug_primitives,
        Vec::new(),
    )
    .map_err(|_error| capture_failure())?;

    let limits = HarnessLimits::phase2_default_v1();
    let bytes = encode_canonical_checkpoint_jsonl(&checkpoint, &limits)
        .map_err(|_error| capture_failure())?;
    let decoded =
        decode_canonical_checkpoint_jsonl(&bytes, &limits).map_err(|_error| capture_failure())?;
    if decoded != checkpoint {
        return Err(capture_failure());
    }
    Ok(checkpoint)
}

fn structural(id: &str, count: usize) -> Result<StructuralObservation, SessionBackendError> {
    let id = ScenarioId::new(id).map_err(|_error| capture_failure())?;
    let count = u64::try_from(count).map_err(|_error| resource_failure())?;
    Ok(StructuralObservation::new(
        id,
        StructuralValue::Count(count),
    ))
}

const fn capture_failure() -> SessionBackendError {
    SessionBackendError::harness(SessionBackendErrorCategory::Capture)
}

const fn resource_failure() -> SessionBackendError {
    SessionBackendError::harness(SessionBackendErrorCategory::ResourceLimit)
}

use liquidfun_test_protocol::{
    BenchmarkRunRequest, CanonicalCheckpoint, HarnessLimits, SemanticCheckpointIdentity, Sha256Hex,
    encode_canonical_checkpoint_jsonl,
};
use sha2::{Digest, Sha256};

use super::{PerformanceExecutionError, PerformanceExecutionErrorKind, PreparedNativeBenchmark};

pub(super) fn validate_request(
    prepared: &PreparedNativeBenchmark,
    request: &BenchmarkRunRequest,
) -> Result<(), PerformanceExecutionError> {
    let identity = request.identity();
    if request.resolved_bytes() != prepared.resolved.canonical_bytes()
        || identity.resolved_sha256() != prepared.resolved.identity().content_sha256()
        || identity.settings() != prepared.resolved.identity().settings()
    {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::ResolvedIdentity,
        ));
    }
    if identity.measured_horizon() != prepared.logical_horizon {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::HorizonMismatch,
        ));
    }
    Ok(())
}

pub(super) fn semantic_checkpoint_identity(
    checkpoint: &CanonicalCheckpoint,
) -> Result<SemanticCheckpointIdentity, PerformanceExecutionError> {
    let mut bytes =
        encode_canonical_checkpoint_jsonl(checkpoint, &HarnessLimits::phase2_default_v1())
            .map_err(|_error| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::CheckpointMismatch)
            })?;
    if bytes.pop() != Some(b'\n') {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::CheckpointMismatch,
        ));
    }
    Ok(SemanticCheckpointIdentity::new(
        checkpoint.request_id().clone(),
        checkpoint.resolved_sha256().clone(),
        checkpoint.checkpoint_id().clone(),
        Sha256Hex::from_digest(Sha256::digest(bytes).into()),
    ))
}

/// Compares only authoritative non-visual checkpoint identity and semantic observation lanes.
///
/// Renderer debug primitives and diagnostic profile names cannot accept or reject a physics timing
/// sample. Exact resolved identity, checkpoint position/time, structural and numeric observations,
/// ordered occurrences, and unordered sets remain mandatory.
#[must_use]
pub fn benchmark_semantics_match(
    expected: &CanonicalCheckpoint,
    candidate: &CanonicalCheckpoint,
) -> bool {
    expected.protocol_version() == candidate.protocol_version()
        && expected.schema_version() == candidate.schema_version()
        && expected.record_kind() == candidate.record_kind()
        && expected.request_id() == candidate.request_id()
        && expected.resolved_sha256() == candidate.resolved_sha256()
        && expected.checkpoint_id() == candidate.checkpoint_id()
        && expected.position() == candidate.position()
        && expected.simulation_time_bits() == candidate.simulation_time_bits()
        && expected.observations() == candidate.observations()
        && expected.numeric_observations() == candidate.numeric_observations()
        && expected.ordered_occurrences() == candidate.ordered_occurrences()
        && expected.unordered_sets() == candidate.unordered_sets()
}

pub(super) fn benchmark_semantics_match_except_request_id(
    expected: &CanonicalCheckpoint,
    candidate: &CanonicalCheckpoint,
) -> bool {
    expected.protocol_version() == candidate.protocol_version()
        && expected.schema_version() == candidate.schema_version()
        && expected.record_kind() == candidate.record_kind()
        && expected.resolved_sha256() == candidate.resolved_sha256()
        && expected.checkpoint_id() == candidate.checkpoint_id()
        && expected.position() == candidate.position()
        && expected.simulation_time_bits() == candidate.simulation_time_bits()
        && expected.observations() == candidate.observations()
        && expected.numeric_observations() == candidate.numeric_observations()
        && expected.ordered_occurrences() == candidate.ordered_occurrences()
        && expected.unordered_sets() == candidate.unordered_sets()
}

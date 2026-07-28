use super::{
    BuildIdentity, CheckpointRecord, Digest, HarnessFailureKind, HarnessLimits,
    ScenarioRequestRecord, Sha256, Sha256Hex, TraceBegin, TraceEnd, TraceHashError, TraceRecord,
    TraceValidationError, ValidatedTrace,
};

enum TraceState {
    AwaitingBegin,
    Streaming {
        begin: TraceBegin,
        checkpoints: Vec<CheckpointRecord>,
    },
    Complete(ValidatedTrace),
}

/// Consuming state-machine validator for one streamed response.
pub struct TraceValidator;

impl TraceValidator {
    /// Validates record size, order, identities, payload hash, counts, and reset proof.
    ///
    /// # Errors
    ///
    /// Returns the exact harness failure category for the first invalid transition or invariant.
    pub fn validate(
        request: &ScenarioRequestRecord,
        identity: &BuildIdentity,
        expected_reset_epoch: u64,
        records: Vec<TraceRecord>,
        limits: &HarnessLimits,
    ) -> Result<ValidatedTrace, TraceValidationError> {
        validate_trace_size(&records, limits)?;
        let mut state = TraceState::AwaitingBegin;
        for record in records {
            state = match (state, record) {
                (TraceState::AwaitingBegin, TraceRecord::Begin(begin)) => {
                    validate_begin(&begin, request, identity)?;
                    TraceState::Streaming {
                        begin,
                        checkpoints: Vec::new(),
                    }
                }
                (
                    TraceState::Streaming {
                        begin,
                        mut checkpoints,
                    },
                    TraceRecord::Checkpoint(checkpoint),
                ) => {
                    validate_checkpoint(&checkpoint, request, identity, checkpoints.len())?;
                    checkpoints.push(checkpoint);
                    TraceState::Streaming { begin, checkpoints }
                }
                (TraceState::Streaming { begin, checkpoints }, TraceRecord::End(end)) => {
                    TraceState::Complete(validate_end(
                        begin,
                        checkpoints,
                        end,
                        request,
                        identity,
                        expected_reset_epoch,
                    )?)
                }
                (TraceState::Complete(_), _) => {
                    return Err(TraceValidationError::new(
                        HarnessFailureKind::SequenceViolation,
                        "records may not follow trace_end",
                    ));
                }
                _ => {
                    return Err(TraceValidationError::new(
                        HarnessFailureKind::SequenceViolation,
                        "trace record appeared outside the begin/checkpoint/end state machine",
                    ));
                }
            };
        }
        let TraceState::Complete(trace) = state else {
            return Err(TraceValidationError::new(
                HarnessFailureKind::UnexpectedEof,
                "stream ended before trace_end",
            ));
        };
        Ok(trace)
    }
}

fn validate_begin(
    begin: &TraceBegin,
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
) -> Result<(), TraceValidationError> {
    if begin.request_id != *request.request_id() {
        return Err(request_mismatch());
    }
    if begin.identity_sha256 != *identity.identity_sha256() {
        return Err(identity_mismatch());
    }
    if begin.protocol_version != request.protocol_version()
        || begin.trace_schema_version != request.requested_trace_schema_version()
        || begin.scenario_id != *request.scenario().scenario_id()
        || begin.scenario_sha256 != scenario_sha256(request)?
        || begin.source != *request.scenario().source()
        || begin.tolerance_profile_version != request.tolerance_profile_version()
        || begin.tolerance_profile_sha256 != *request.tolerance_profile_sha256()
    {
        return Err(TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "trace_begin does not match the validated request contract",
        ));
    }
    Ok(())
}

fn validate_checkpoint(
    checkpoint: &CheckpointRecord,
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
    expected_ordinal: usize,
) -> Result<(), TraceValidationError> {
    if checkpoint.request_id != *request.request_id() {
        return Err(request_mismatch());
    }
    if checkpoint.identity_sha256 != *identity.identity_sha256() {
        return Err(identity_mismatch());
    }
    let ordinal = usize::try_from(checkpoint.ordinal).map_err(|_| {
        TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "checkpoint ordinal cannot be represented on this target",
        )
    })?;
    if ordinal != expected_ordinal {
        return Err(TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "checkpoint ordinals must be contiguous and ordered",
        ));
    }
    let Some(expected) = request.scenario().checkpoints().get(expected_ordinal) else {
        return Err(TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "trace emitted an unrequested checkpoint",
        ));
    };
    if checkpoint.checkpoint_id != *expected.checkpoint_id()
        || checkpoint.phase.as_ref() != expected.phase()
        || !checkpoint.world_counts.is_zero()
    {
        return Err(TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "checkpoint identity, phase, or empty-world counts differ from the request",
        ));
    }
    Ok(())
}

fn validate_end(
    begin: TraceBegin,
    checkpoints: Vec<CheckpointRecord>,
    end: TraceEnd,
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
    expected_reset_epoch: u64,
) -> Result<ValidatedTrace, TraceValidationError> {
    if end.request_id != *request.request_id() {
        return Err(request_mismatch());
    }
    if end.identity_sha256 != *identity.identity_sha256() {
        return Err(identity_mismatch());
    }
    if !end.reset_verified || end.reset_epoch != expected_reset_epoch {
        return Err(TraceValidationError::new(
            HarnessFailureKind::AdapterResetFailure,
            "trace_end lacks the exact successful reset proof",
        ));
    }
    let checkpoint_count = u32::try_from(checkpoints.len()).map_err(|_| {
        TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "checkpoint count cannot be represented on the wire",
        )
    })?;
    if checkpoint_count != end.checkpoint_count
        || checkpoints.len() != request.scenario().checkpoints().len()
        || trace_payload_sha256(&checkpoints).map_err(|error| {
            TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
        })? != end.trace_payload_sha256
    {
        return Err(TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "trace_end count or payload hash does not match ordered checkpoints",
        ));
    }
    Ok(ValidatedTrace {
        begin,
        checkpoints: checkpoints.into_boxed_slice(),
        end,
        evidence_tier: identity.evidence_tier(),
    })
}

fn validate_trace_size(
    records: &[TraceRecord],
    limits: &HarnessLimits,
) -> Result<(), TraceValidationError> {
    let total = records.iter().try_fold(0_usize, |total, record| {
        let bytes = serde_json::to_vec(record).map_err(|error| {
            TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
        })?;
        let record_bytes = bytes.len().checked_add(1).ok_or_else(|| {
            TraceValidationError::new(
                HarnessFailureKind::RecordTooLarge,
                "record byte count overflowed",
            )
        })?;
        if record_bytes > limits.output_record_bytes() {
            return Err(TraceValidationError::new(
                HarnessFailureKind::RecordTooLarge,
                "output record exceeds the reviewed limit",
            ));
        }
        total.checked_add(record_bytes).ok_or_else(|| {
            TraceValidationError::new(
                HarnessFailureKind::TraceTooLarge,
                "trace byte count overflowed",
            )
        })
    })?;
    if total > limits.complete_trace_bytes() {
        return Err(TraceValidationError::new(
            HarnessFailureKind::TraceTooLarge,
            "complete trace exceeds the reviewed limit",
        ));
    }
    Ok(())
}

pub(in crate::trace) fn scenario_sha256(
    request: &ScenarioRequestRecord,
) -> Result<Sha256Hex, TraceValidationError> {
    let bytes = serde_json::to_vec(request.scenario()).map_err(|error| {
        TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
    })?;
    Ok(Sha256Hex::from_digest(Sha256::digest(bytes).into()))
}

/// Hashes ordered checkpoint payloads using length-prefixed deterministic JSON bytes.
///
/// # Errors
///
/// Returns [`TraceHashError`] if a typed checkpoint cannot be serialized.
pub fn trace_payload_sha256(checkpoints: &[CheckpointRecord]) -> Result<Sha256Hex, TraceHashError> {
    let mut hasher = Sha256::new();
    for checkpoint in checkpoints {
        let bytes = serde_json::to_vec(checkpoint).map_err(TraceHashError)?;
        hasher.update(bytes.len().to_be_bytes());
        hasher.update(bytes);
    }
    Ok(Sha256Hex::from_digest(hasher.finalize().into()))
}

fn request_mismatch() -> TraceValidationError {
    TraceValidationError::new(
        HarnessFailureKind::RequestIdMismatch,
        "trace request identity differs from the in-flight request",
    )
}

fn identity_mismatch() -> TraceValidationError {
    TraceValidationError::new(
        HarnessFailureKind::TraceIdentityMismatch,
        "trace build identity differs from the validated handshake",
    )
}

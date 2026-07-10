use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    BuildIdentity, BuildIdentityFields, CheckpointId, CodecError, FloatBits, HarnessFailureKind,
    HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId, ScenarioRequestRecord,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_SUPPORTED_VERSIONS: usize = 16;

/// Error produced while validating handshake or trace semantics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("trace validation failed: {kind:?}: {message}")]
pub struct TraceValidationError {
    kind: HarnessFailureKind,
    message: Box<str>,
}

impl TraceValidationError {
    fn new(kind: HarnessFailureKind, message: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Returns the exact non-physics harness classification.
    #[must_use]
    pub const fn kind(&self) -> HarnessFailureKind {
        self.kind
    }
}

/// Error produced while hashing canonical trace payload bytes.
#[derive(Debug, thiserror::Error)]
#[error("trace payload serialization failed: {0}")]
pub struct TraceHashError(serde_json::Error);

/// Error produced while strictly decoding a handshake or trace record.
#[derive(Debug, thiserror::Error)]
pub enum TraceDecodeError {
    /// Strict JSONL framing or shape validation failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// Typed provenance or record validation failed.
    #[error(transparent)]
    Validation(#[from] TraceValidationError),
}

/// Stable engine implementation identity carried by a semantic trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    /// Independent native Rust engine.
    NativeRust,
    /// Pinned development-only C++ oracle.
    CppOracle,
}

/// Validated startup handshake emitted before any request is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeRecord {
    protocol_version: ProtocolVersion,
    supported_scenario_versions: Box<[ScenarioSchemaVersion]>,
    supported_trace_versions: Box<[TraceSchemaVersion]>,
    supported_tolerance_versions: Box<[ToleranceProfileVersion]>,
    build_identity: BuildIdentity,
}

impl HandshakeRecord {
    /// Creates the complete supported phase-2 handshake.
    #[must_use]
    pub fn phase2(build_identity: BuildIdentity) -> Self {
        Self {
            protocol_version: ProtocolVersion::CURRENT,
            supported_scenario_versions: vec![ScenarioSchemaVersion::CURRENT].into_boxed_slice(),
            supported_trace_versions: vec![TraceSchemaVersion::CURRENT].into_boxed_slice(),
            supported_tolerance_versions: vec![ToleranceProfileVersion::CURRENT].into_boxed_slice(),
            build_identity,
        }
    }

    /// Returns the independently recomputed build identity.
    #[must_use]
    pub const fn build_identity(&self) -> &BuildIdentity {
        &self.build_identity
    }
}

enum SessionState {
    AwaitingHandshake,
    Ready(BuildIdentity),
}

/// Enforces startup handshake ordering and expected pinned provenance.
pub struct ProtocolSessionValidator {
    expected_oracle_revision: Box<str>,
    state: SessionState,
}

impl ProtocolSessionValidator {
    /// Creates a session that trusts only the supplied full pinned revision.
    #[must_use]
    pub fn new(expected_oracle_revision: impl Into<Box<str>>) -> Self {
        Self {
            expected_oracle_revision: expected_oracle_revision.into(),
            state: SessionState::AwaitingHandshake,
        }
    }

    /// Accepts exactly one compatible handshake before requests.
    ///
    /// # Errors
    ///
    /// Returns a sequence, version, or provenance harness failure when the handshake is invalid.
    pub fn accept_handshake(
        &mut self,
        handshake: HandshakeRecord,
    ) -> Result<(), TraceValidationError> {
        if matches!(self.state, SessionState::Ready(_)) {
            return Err(TraceValidationError::new(
                HarnessFailureKind::SequenceViolation,
                "handshake may appear only once before requests",
            ));
        }
        if handshake.protocol_version.get() != ProtocolVersion::SUPPORTED
            || !handshake
                .supported_scenario_versions
                .iter()
                .any(|version| version.get() == ScenarioSchemaVersion::SUPPORTED)
            || !handshake
                .supported_trace_versions
                .iter()
                .any(|version| version.get() == TraceSchemaVersion::SUPPORTED)
            || !handshake
                .supported_tolerance_versions
                .iter()
                .any(|version| version.get() == ToleranceProfileVersion::SUPPORTED)
        {
            return Err(TraceValidationError::new(
                HarnessFailureKind::UnsupportedVersion,
                "handshake does not support every phase-2 version axis",
            ));
        }
        if handshake.build_identity.oracle_revision() != self.expected_oracle_revision.as_ref() {
            return Err(TraceValidationError::new(
                HarnessFailureKind::WrongProvenance,
                "handshake oracle revision differs from the pinned revision",
            ));
        }
        self.state = SessionState::Ready(handshake.build_identity);
        Ok(())
    }

    /// Verifies that a request is sent only after a valid handshake.
    ///
    /// # Errors
    ///
    /// Returns [`HarnessFailureKind::HandshakeMalformed`] before startup completes.
    pub fn begin_request(
        &self,
        _request: &ScenarioRequestRecord,
    ) -> Result<(), TraceValidationError> {
        if matches!(self.state, SessionState::Ready(_)) {
            return Ok(());
        }
        Err(TraceValidationError::new(
            HarnessFailureKind::HandshakeMalformed,
            "scenario request cannot precede the startup handshake",
        ))
    }

    /// Returns the validated session identity after the handshake.
    #[must_use]
    pub fn maybe_build_identity(&self) -> Option<&BuildIdentity> {
        let SessionState::Ready(identity) = &self.state else {
            return None;
        };
        Some(identity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceBeginKind {
    TraceBegin,
}

/// Provenance-bound beginning of one streamed semantic trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceBegin {
    protocol_version: ProtocolVersion,
    record_kind: TraceBeginKind,
    request_id: RequestId,
    trace_schema_version: TraceSchemaVersion,
    scenario_id: ScenarioId,
    scenario_sha256: Sha256Hex,
    source: ScenarioSource,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    engine_kind: EngineKind,
    identity_sha256: Sha256Hex,
}

impl TraceBegin {
    /// Creates a trace begin record from one validated request and engine build.
    ///
    /// # Errors
    ///
    /// Returns [`TraceValidationError`] only if deterministic scenario serialization fails.
    pub fn for_request(
        request: &ScenarioRequestRecord,
        engine_kind: EngineKind,
        identity: &BuildIdentity,
    ) -> Result<Self, TraceValidationError> {
        let scenario_sha256 = scenario_sha256(request)?;
        Ok(Self {
            protocol_version: request.protocol_version(),
            record_kind: TraceBeginKind::TraceBegin,
            request_id: request.request_id().clone(),
            trace_schema_version: request.requested_trace_schema_version(),
            scenario_id: request.scenario().scenario_id().clone(),
            scenario_sha256,
            source: request.scenario().source().clone(),
            tolerance_profile_version: request.tolerance_profile_version(),
            tolerance_profile_sha256: request.tolerance_profile_sha256().clone(),
            engine_kind,
            identity_sha256: identity.identity_sha256().clone(),
        })
    }
}

/// Exact semantic empty-world counts at one checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldCounts {
    bodies: u32,
    fixtures: u32,
    joints: u32,
    contacts: u32,
    particle_systems: u32,
    particle_groups: u32,
    particles: u32,
}

impl WorldCounts {
    /// Returns exact zero counts for every phase-2 empty-world entity class.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            bodies: 0,
            fixtures: 0,
            joints: 0,
            contacts: 0,
            particle_systems: 0,
            particle_groups: 0,
            particles: 0,
        }
    }

    /// Returns the exact body count.
    #[must_use]
    pub const fn bodies(self) -> u32 {
        self.bodies
    }

    /// Returns the exact fixture count.
    #[must_use]
    pub const fn fixtures(self) -> u32 {
        self.fixtures
    }

    /// Returns the exact joint count.
    #[must_use]
    pub const fn joints(self) -> u32 {
        self.joints
    }

    /// Returns the exact contact count.
    #[must_use]
    pub const fn contacts(self) -> u32 {
        self.contacts
    }

    /// Returns the exact particle-system count.
    #[must_use]
    pub const fn particle_systems(self) -> u32 {
        self.particle_systems
    }

    /// Returns the exact particle-group count.
    #[must_use]
    pub const fn particle_groups(self) -> u32 {
        self.particle_groups
    }

    /// Returns the exact particle count.
    #[must_use]
    pub const fn particles(self) -> u32 {
        self.particles
    }

    const fn is_zero(self) -> bool {
        self.bodies == 0
            && self.fixtures == 0
            && self.joints == 0
            && self.contacts == 0
            && self.particle_systems == 0
            && self.particle_groups == 0
            && self.particles == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CheckpointKind {
    Checkpoint,
}

/// One ordered semantic checkpoint emitted by an engine adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckpointRecord {
    protocol_version: ProtocolVersion,
    record_kind: CheckpointKind,
    request_id: RequestId,
    checkpoint_id: CheckpointId,
    ordinal: u32,
    phase: Box<str>,
    simulation_time_bits: FloatBits,
    world_counts: WorldCounts,
    identity_sha256: Sha256Hex,
}

impl CheckpointRecord {
    /// Creates a bounded typed checkpoint record.
    ///
    /// # Errors
    ///
    /// Returns [`TraceValidationError`] when the phase label is empty or oversized.
    #[allow(
        clippy::too_many_arguments,
        reason = "checkpoint wire records have eight fixed fields"
    )]
    pub fn new(
        request_id: RequestId,
        checkpoint_id: CheckpointId,
        ordinal: u32,
        phase: impl Into<String>,
        simulation_time_bits: FloatBits,
        world_counts: WorldCounts,
        identity_sha256: Sha256Hex,
    ) -> Result<Self, TraceValidationError> {
        let phase = phase.into();
        if phase.is_empty() || phase.len() > MAXIMUM_STRING_BYTES {
            return Err(TraceValidationError::new(
                HarnessFailureKind::MalformedRecord,
                "checkpoint phase must be nonempty and bounded",
            ));
        }
        Ok(Self {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: CheckpointKind::Checkpoint,
            request_id,
            checkpoint_id,
            ordinal,
            phase: phase.into_boxed_str(),
            simulation_time_bits,
            world_counts,
            identity_sha256,
        })
    }

    #[cfg(test)]
    pub(crate) fn set_request_id_for_test(&mut self, request_id: RequestId) {
        self.request_id = request_id;
    }

    #[cfg(test)]
    pub(crate) fn set_identity_for_test(&mut self, identity_sha256: Sha256Hex) {
        self.identity_sha256 = identity_sha256;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceEndKind {
    TraceEnd,
}

/// Terminal count, payload hash, and adapter reset proof for one request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceEnd {
    protocol_version: ProtocolVersion,
    record_kind: TraceEndKind,
    request_id: RequestId,
    checkpoint_count: u32,
    trace_payload_sha256: Sha256Hex,
    reset_epoch: u64,
    reset_verified: bool,
    identity_sha256: Sha256Hex,
}

impl TraceEnd {
    /// Creates one terminal trace record.
    #[must_use]
    pub fn new(
        request_id: RequestId,
        checkpoint_count: u32,
        trace_payload_sha256: Sha256Hex,
        reset_epoch: u64,
        reset_verified: bool,
        identity_sha256: Sha256Hex,
    ) -> Self {
        Self {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: TraceEndKind::TraceEnd,
            request_id,
            checkpoint_count,
            trace_payload_sha256,
            reset_epoch,
            reset_verified,
            identity_sha256,
        }
    }

    #[cfg(test)]
    pub(crate) fn set_reset_verified_for_test(&mut self, reset_verified: bool) {
        self.reset_verified = reset_verified;
    }

    #[cfg(test)]
    pub(crate) fn set_checkpoint_count_for_test(&mut self, checkpoint_count: u32) {
        self.checkpoint_count = checkpoint_count;
    }

    #[cfg(test)]
    pub(crate) fn set_payload_hash_for_test(&mut self, trace_payload_sha256: Sha256Hex) {
        self.trace_payload_sha256 = trace_payload_sha256;
    }

    #[cfg(test)]
    pub(crate) fn set_reset_epoch_for_test(&mut self, reset_epoch: u64) {
        self.reset_epoch = reset_epoch;
    }
}

/// Closed streamed record variants after strict typed decoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum TraceRecord {
    /// First record for a request.
    Begin(TraceBegin),
    /// Ordered semantic checkpoint.
    Checkpoint(CheckpointRecord),
    /// Exactly one terminal record.
    End(TraceEnd),
}

/// A complete provenance-checked, reset-proven semantic trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTrace {
    begin: TraceBegin,
    checkpoints: Box<[CheckpointRecord]>,
    end: TraceEnd,
}

impl ValidatedTrace {
    /// Returns checkpoints in preserved solver-significant order.
    #[must_use]
    pub fn checkpoints(&self) -> &[CheckpointRecord] {
        &self.checkpoints
    }

    /// Returns the validated reset epoch.
    #[must_use]
    pub const fn reset_epoch(&self) -> u64 {
        self.end.reset_epoch
    }
}

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

fn scenario_sha256(request: &ScenarioRequestRecord) -> Result<Sha256Hex, TraceValidationError> {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHandshakeRecord {
    protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: HandshakeKind,
    supported_scenario_versions: BoundedVec<ScenarioSchemaVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    supported_trace_versions: BoundedVec<TraceSchemaVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    supported_tolerance_versions: BoundedVec<ToleranceProfileVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    build_identity: RawBuildIdentity,
    identity_sha256: Sha256Hex,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HandshakeKind {
    Handshake,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuildIdentity {
    oracle_revision: BoundedString<MAXIMUM_STRING_BYTES>,
    adapter_revision: BoundedString<MAXIMUM_STRING_BYTES>,
    adapter_content_sha256: BoundedString<MAXIMUM_STRING_BYTES>,
    cmake_preset: BoundedString<MAXIMUM_STRING_BYTES>,
    compiler_id: BoundedString<MAXIMUM_STRING_BYTES>,
    compiler_version: BoundedString<MAXIMUM_STRING_BYTES>,
    target: BoundedString<MAXIMUM_STRING_BYTES>,
    build_type: BoundedString<MAXIMUM_STRING_BYTES>,
    effective_compile_flags: BoundedString<MAXIMUM_STRING_BYTES>,
    effective_link_flags: BoundedString<MAXIMUM_STRING_BYTES>,
    sanitizer_mode: BoundedString<MAXIMUM_STRING_BYTES>,
}

/// Strictly decodes and independently recomputes one startup handshake.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, shape, limit, or provenance failure.
pub fn decode_handshake_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<HandshakeRecord, TraceDecodeError> {
    let raw = decode_jsonl::<RawHandshakeRecord>(bytes, limits, RecordLimit::Output)?;
    let RawHandshakeRecord {
        protocol_version,
        _record_kind: _,
        supported_scenario_versions,
        supported_trace_versions,
        supported_tolerance_versions,
        build_identity: raw_identity,
        identity_sha256,
    } = raw;
    let fields = BuildIdentityFields::new(
        raw_identity.oracle_revision.into_string(),
        raw_identity.adapter_revision.into_string(),
        raw_identity.adapter_content_sha256.into_string(),
        raw_identity.cmake_preset.into_string(),
        raw_identity.compiler_id.into_string(),
        raw_identity.compiler_version.into_string(),
        raw_identity.target.into_string(),
        raw_identity.build_type.into_string(),
        raw_identity.effective_compile_flags.into_string(),
        raw_identity.effective_link_flags.into_string(),
        raw_identity.sanitizer_mode.into_string(),
    );
    let build_identity =
        BuildIdentity::from_reported(fields, &identity_sha256).map_err(|error| {
            TraceValidationError::new(HarnessFailureKind::WrongProvenance, error.to_string())
        })?;
    Ok(HandshakeRecord {
        protocol_version,
        supported_scenario_versions: supported_scenario_versions.into_vec().into_boxed_slice(),
        supported_trace_versions: supported_trace_versions.into_vec().into_boxed_slice(),
        supported_tolerance_versions: supported_tolerance_versions.into_vec().into_boxed_slice(),
        build_identity,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "record_kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawTraceRecord {
    TraceBegin {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        trace_schema_version: TraceSchemaVersion,
        scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
        scenario_sha256: Sha256Hex,
        source: RawTraceSource,
        tolerance_profile_version: ToleranceProfileVersion,
        tolerance_profile_sha256: Sha256Hex,
        engine_kind: EngineKind,
        identity_sha256: Sha256Hex,
    },
    Checkpoint {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        checkpoint_id: BoundedString<MAXIMUM_ID_BYTES>,
        ordinal: u32,
        phase: BoundedString<MAXIMUM_STRING_BYTES>,
        simulation_time_bits: FloatBits,
        world_counts: WorldCounts,
        identity_sha256: Sha256Hex,
    },
    TraceEnd {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        checkpoint_count: u32,
        trace_payload_sha256: Sha256Hex,
        reset_epoch: u64,
        reset_verified: bool,
        identity_sha256: Sha256Hex,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawTraceSource {
    Named {
        name: BoundedString<MAXIMUM_STRING_BYTES>,
    },
    Seeded {
        generator_id: BoundedString<MAXIMUM_STRING_BYTES>,
        generator_version: u32,
        seed: u64,
    },
}

/// Strictly decodes one streamed trace record into its closed typed variant.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, shape, limit, ID, or phase failure.
pub fn decode_trace_record_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<TraceRecord, TraceDecodeError> {
    let raw = decode_jsonl::<RawTraceRecord>(bytes, limits, RecordLimit::Output)?;
    match raw {
        RawTraceRecord::TraceBegin {
            protocol_version,
            request_id,
            trace_schema_version,
            scenario_id,
            scenario_sha256,
            source,
            tolerance_profile_version,
            tolerance_profile_sha256,
            engine_kind,
            identity_sha256,
        } => Ok(TraceRecord::Begin(TraceBegin {
            protocol_version,
            record_kind: TraceBeginKind::TraceBegin,
            request_id: parse_request_id(request_id)?,
            trace_schema_version,
            scenario_id: ScenarioId::new(scenario_id.into_string()).map_err(|error| {
                TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
            })?,
            scenario_sha256,
            source: convert_source(source)?,
            tolerance_profile_version,
            tolerance_profile_sha256,
            engine_kind,
            identity_sha256,
        })),
        RawTraceRecord::Checkpoint {
            protocol_version,
            request_id,
            checkpoint_id,
            ordinal,
            phase,
            simulation_time_bits,
            world_counts,
            identity_sha256,
        } => {
            let mut checkpoint = CheckpointRecord::new(
                parse_request_id(request_id)?,
                CheckpointId::new(checkpoint_id.into_string()).map_err(|error| {
                    TraceValidationError::new(
                        HarnessFailureKind::MalformedRecord,
                        error.to_string(),
                    )
                })?,
                ordinal,
                phase.into_string(),
                simulation_time_bits,
                world_counts,
                identity_sha256,
            )?;
            checkpoint.protocol_version = protocol_version;
            Ok(TraceRecord::Checkpoint(checkpoint))
        }
        RawTraceRecord::TraceEnd {
            protocol_version,
            request_id,
            checkpoint_count,
            trace_payload_sha256,
            reset_epoch,
            reset_verified,
            identity_sha256,
        } => {
            let mut end = TraceEnd::new(
                parse_request_id(request_id)?,
                checkpoint_count,
                trace_payload_sha256,
                reset_epoch,
                reset_verified,
                identity_sha256,
            );
            end.protocol_version = protocol_version;
            Ok(TraceRecord::End(end))
        }
    }
}

fn parse_request_id(
    raw: BoundedString<MAXIMUM_ID_BYTES>,
) -> Result<RequestId, TraceValidationError> {
    RequestId::new(raw.into_string()).map_err(|error| {
        TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
    })
}

fn convert_source(raw: RawTraceSource) -> Result<ScenarioSource, TraceValidationError> {
    match raw {
        RawTraceSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(invalid_trace_source());
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawTraceSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(invalid_trace_source());
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

fn invalid_trace_source() -> TraceValidationError {
    TraceValidationError::new(
        HarnessFailureKind::MalformedRecord,
        "trace source identity must be nonempty and versioned",
    )
}

#[cfg(test)]
mod tests;

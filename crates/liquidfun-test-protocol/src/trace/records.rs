use super::{
    BuildEvidenceTier, BuildIdentity, CheckpointId, Deserialize, EngineKind, FloatBits,
    HarnessFailureKind, MAXIMUM_STRING_BYTES, ProtocolVersion, RequestId, ScenarioId,
    ScenarioRequestRecord, ScenarioSource, Serialize, Sha256Hex, ToleranceProfileVersion,
    TraceSchemaVersion, TraceValidationError, scenario_sha256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::trace) enum TraceBeginKind {
    TraceBegin,
}

/// Provenance-bound beginning of one streamed semantic trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceBegin {
    pub(in crate::trace) protocol_version: ProtocolVersion,
    pub(in crate::trace) record_kind: TraceBeginKind,
    pub(in crate::trace) request_id: RequestId,
    pub(in crate::trace) trace_schema_version: TraceSchemaVersion,
    pub(in crate::trace) scenario_id: ScenarioId,
    pub(in crate::trace) scenario_sha256: Sha256Hex,
    pub(in crate::trace) source: ScenarioSource,
    pub(in crate::trace) tolerance_profile_version: ToleranceProfileVersion,
    pub(in crate::trace) tolerance_profile_sha256: Sha256Hex,
    pub(in crate::trace) engine_kind: EngineKind,
    pub(in crate::trace) identity_sha256: Sha256Hex,
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
    pub(in crate::trace) bodies: u32,
    pub(in crate::trace) fixtures: u32,
    pub(in crate::trace) joints: u32,
    pub(in crate::trace) contacts: u32,
    pub(in crate::trace) particle_systems: u32,
    pub(in crate::trace) particle_groups: u32,
    pub(in crate::trace) particles: u32,
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

    pub(in crate::trace) const fn is_zero(self) -> bool {
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
pub(in crate::trace) enum CheckpointKind {
    Checkpoint,
}

/// One ordered semantic checkpoint emitted by an engine adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckpointRecord {
    pub(in crate::trace) protocol_version: ProtocolVersion,
    pub(in crate::trace) record_kind: CheckpointKind,
    pub(in crate::trace) request_id: RequestId,
    pub(in crate::trace) checkpoint_id: CheckpointId,
    pub(in crate::trace) ordinal: u32,
    pub(in crate::trace) phase: Box<str>,
    pub(in crate::trace) simulation_time_bits: FloatBits,
    pub(in crate::trace) world_counts: WorldCounts,
    pub(in crate::trace) identity_sha256: Sha256Hex,
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

    /// Returns the stable semantic checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the preserved checkpoint ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Returns the named semantic phase.
    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    /// Returns the authoritative simulation-time bits.
    #[must_use]
    pub const fn simulation_time_bits(&self) -> FloatBits {
        self.simulation_time_bits
    }

    /// Returns exact typed world counts.
    #[must_use]
    pub const fn world_counts(&self) -> WorldCounts {
        self.world_counts
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
pub(in crate::trace) enum TraceEndKind {
    TraceEnd,
}

/// Terminal count, payload hash, and adapter reset proof for one request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceEnd {
    pub(in crate::trace) protocol_version: ProtocolVersion,
    pub(in crate::trace) record_kind: TraceEndKind,
    pub(in crate::trace) request_id: RequestId,
    pub(in crate::trace) checkpoint_count: u32,
    pub(in crate::trace) trace_payload_sha256: Sha256Hex,
    pub(in crate::trace) reset_epoch: u64,
    pub(in crate::trace) reset_verified: bool,
    pub(in crate::trace) identity_sha256: Sha256Hex,
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
    pub(in crate::trace) begin: TraceBegin,
    pub(in crate::trace) checkpoints: Box<[CheckpointRecord]>,
    pub(in crate::trace) end: TraceEnd,
    pub(in crate::trace) evidence_tier: BuildEvidenceTier,
}

impl ValidatedTrace {
    /// Returns the validated transport protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> ProtocolVersion {
        self.begin.protocol_version
    }

    /// Returns the validated trace schema version.
    #[must_use]
    pub const fn trace_schema_version(&self) -> TraceSchemaVersion {
        self.begin.trace_schema_version
    }

    /// Returns the in-flight request identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.begin.request_id
    }

    /// Returns the stable scenario identity.
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.begin.scenario_id
    }

    /// Returns the canonical scenario identity hash.
    #[must_use]
    pub const fn scenario_sha256(&self) -> &Sha256Hex {
        &self.begin.scenario_sha256
    }

    /// Returns named or seeded source metadata.
    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.begin.source
    }

    /// Returns the independently versioned tolerance profile version.
    #[must_use]
    pub const fn tolerance_profile_version(&self) -> ToleranceProfileVersion {
        self.begin.tolerance_profile_version
    }

    /// Returns the reviewed tolerance profile identity.
    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.begin.tolerance_profile_sha256
    }

    /// Returns the engine role carried by the trace.
    #[must_use]
    pub const fn engine_kind(&self) -> EngineKind {
        self.begin.engine_kind
    }

    /// Returns the validated engine-build identity hash.
    #[must_use]
    pub const fn identity_sha256(&self) -> &Sha256Hex {
        &self.begin.identity_sha256
    }

    /// Returns the evidence authority derived from the validated build identity.
    #[must_use]
    pub const fn evidence_tier(&self) -> BuildEvidenceTier {
        self.evidence_tier
    }

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

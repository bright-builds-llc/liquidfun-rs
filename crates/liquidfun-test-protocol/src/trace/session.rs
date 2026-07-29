use super::{
    BuildIdentity, Deserialize, HarnessFailureKind, ProtocolVersion, ScenarioRequestRecord,
    ScenarioSchemaVersion, Serialize, ToleranceProfileVersion, TraceSchemaVersion,
    TraceValidationError,
};

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
    pub(in crate::trace) protocol_version: ProtocolVersion,
    pub(in crate::trace) supported_scenario_versions: Box<[ScenarioSchemaVersion]>,
    pub(in crate::trace) supported_trace_versions: Box<[TraceSchemaVersion]>,
    pub(in crate::trace) supported_tolerance_versions: Box<[ToleranceProfileVersion]>,
    pub(in crate::trace) build_identity: BuildIdentity,
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
    Ready(Box<BuildIdentity>),
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
        self.state = SessionState::Ready(Box::new(handshake.build_identity));
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

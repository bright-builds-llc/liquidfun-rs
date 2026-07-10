use std::time::Duration;

use crate::{HarnessLimits, RequestId, Sha256Hex};

/// Exhaustive non-physics failure categories produced by the harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessFailureKind {
    /// The child did not complete its startup handshake before the deadline.
    StartupTimeout,
    /// The startup handshake was syntactically or semantically malformed.
    HandshakeMalformed,
    /// A protocol, scenario, trace, or tolerance version is unsupported.
    UnsupportedVersion,
    /// The reported build does not match the expected oracle provenance.
    WrongProvenance,
    /// An in-flight request exceeded its deadline.
    RequestTimeout,
    /// The child exited with an unsuccessful status code.
    ChildNonZeroExit,
    /// The child terminated because of a signal.
    ChildSignaled,
    /// Bounded stderr contained a recognized sanitizer report.
    SanitizerReport,
    /// The child closed stdout before a complete trace finished.
    UnexpectedEof,
    /// EOF arrived after bytes that did not end in a complete record.
    PartialRecord,
    /// A complete record could not be decoded or validated.
    MalformedRecord,
    /// A record kind is not part of the negotiated protocol version.
    UnknownRecordKind,
    /// One output record exceeded the reviewed limit.
    RecordTooLarge,
    /// A complete trace exceeded the reviewed limit.
    TraceTooLarge,
    /// Combined child output exceeded the reviewed request limit.
    TotalOutputExceeded,
    /// Valid record kinds appeared in an invalid order.
    SequenceViolation,
    /// A response carried a different request identity.
    RequestIdMismatch,
    /// Trace identity changed within one validated response.
    TraceIdentityMismatch,
    /// The adapter rejected a validated scenario request.
    ScenarioRejected,
    /// The native Rust adapter failed before producing a valid trace.
    RustAdapterFailure,
    /// The C++ oracle adapter failed before producing a valid trace.
    CppAdapterFailure,
    /// The adapter could not prove complete reset after a request.
    AdapterResetFailure,
}

impl HarnessFailureKind {
    /// Every harness-failure category, intentionally excluding physics mismatch.
    pub const ALL: [Self; 22] = [
        Self::StartupTimeout,
        Self::HandshakeMalformed,
        Self::UnsupportedVersion,
        Self::WrongProvenance,
        Self::RequestTimeout,
        Self::ChildNonZeroExit,
        Self::ChildSignaled,
        Self::SanitizerReport,
        Self::UnexpectedEof,
        Self::PartialRecord,
        Self::MalformedRecord,
        Self::UnknownRecordKind,
        Self::RecordTooLarge,
        Self::TraceTooLarge,
        Self::TotalOutputExceeded,
        Self::SequenceViolation,
        Self::RequestIdMismatch,
        Self::TraceIdentityMismatch,
        Self::ScenarioRejected,
        Self::RustAdapterFailure,
        Self::CppAdapterFailure,
        Self::AdapterResetFailure,
    ];

    /// Returns the stable machine-readable failure spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StartupTimeout => "startup_timeout",
            Self::HandshakeMalformed => "handshake_malformed",
            Self::UnsupportedVersion => "unsupported_version",
            Self::WrongProvenance => "wrong_provenance",
            Self::RequestTimeout => "request_timeout",
            Self::ChildNonZeroExit => "child_non_zero_exit",
            Self::ChildSignaled => "child_signaled",
            Self::SanitizerReport => "sanitizer_report",
            Self::UnexpectedEof => "unexpected_eof",
            Self::PartialRecord => "partial_record",
            Self::MalformedRecord => "malformed_record",
            Self::UnknownRecordKind => "unknown_record_kind",
            Self::RecordTooLarge => "record_too_large",
            Self::TraceTooLarge => "trace_too_large",
            Self::TotalOutputExceeded => "total_output_exceeded",
            Self::SequenceViolation => "sequence_violation",
            Self::RequestIdMismatch => "request_id_mismatch",
            Self::TraceIdentityMismatch => "trace_identity_mismatch",
            Self::ScenarioRejected => "scenario_rejected",
            Self::RustAdapterFailure => "rust_adapter_failure",
            Self::CppAdapterFailure => "cpp_adapter_failure",
            Self::AdapterResetFailure => "adapter_reset_failure",
        }
    }
}

/// Error returned when retained stderr metadata violates its reviewed bounds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StderrEvidenceError {
    /// Retained bytes exceed the active profile's retention cap.
    #[error("retained stderr exceeds the active limit")]
    RetainedBytesExceeded,
    /// Total drained bytes cannot be smaller than the retained subset.
    #[error("total stderr bytes are smaller than retained stderr bytes")]
    InvalidTotalBytes,
}

/// Bounded retained stderr plus total and truncation metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StderrEvidence {
    retained: Box<[u8]>,
    total_bytes: usize,
    truncated_bytes: usize,
}

impl StderrEvidence {
    /// Validates retained stderr against the active immutable profile.
    ///
    /// # Errors
    ///
    /// Returns [`StderrEvidenceError`] when retention exceeds the profile or totals conflict.
    pub fn new(
        retained: Vec<u8>,
        total_bytes: usize,
        limits: &HarnessLimits,
    ) -> Result<Self, StderrEvidenceError> {
        if retained.len() > limits.retained_stderr_bytes() {
            return Err(StderrEvidenceError::RetainedBytesExceeded);
        }
        if total_bytes < retained.len() {
            return Err(StderrEvidenceError::InvalidTotalBytes);
        }
        let truncated_bytes = total_bytes - retained.len();
        Ok(Self {
            retained: retained.into_boxed_slice(),
            total_bytes,
            truncated_bytes,
        })
    }

    /// Returns the bounded retained byte window.
    #[must_use]
    pub fn retained(&self) -> &[u8] {
        &self.retained
    }

    /// Returns all bytes drained from stderr.
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Returns the number of drained bytes omitted from retention.
    #[must_use]
    pub const fn truncated_bytes(&self) -> usize {
        self.truncated_bytes
    }
}

/// Last validated protocol record retained for failure diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastValidRecord {
    /// Startup handshake.
    Handshake,
    /// Trace begin record.
    TraceBegin,
    /// Semantic checkpoint record.
    Checkpoint,
    /// Trace end record.
    TraceEnd,
}

/// Bounded diagnostic context attached to a harness failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessFailureEvidence {
    maybe_request_id: Option<RequestId>,
    maybe_request_sha256: Option<Sha256Hex>,
    maybe_scenario_sha256: Option<Sha256Hex>,
    maybe_session_identity_sha256: Option<Sha256Hex>,
    maybe_exit_status: Option<i32>,
    elapsed: Duration,
    maybe_last_valid_record: Option<LastValidRecord>,
    stderr: StderrEvidence,
    child_killed: bool,
    child_reaped: bool,
    limit_profile_id: &'static str,
    limit_profile_sha256: Sha256Hex,
}

impl HarnessFailureEvidence {
    /// Creates bounded failure evidence before optional request/process context is known.
    #[must_use]
    pub fn new(
        elapsed: Duration,
        stderr: StderrEvidence,
        child_killed: bool,
        child_reaped: bool,
        limits: &HarnessLimits,
    ) -> Self {
        Self {
            maybe_request_id: None,
            maybe_request_sha256: None,
            maybe_scenario_sha256: None,
            maybe_session_identity_sha256: None,
            maybe_exit_status: None,
            elapsed,
            maybe_last_valid_record: None,
            stderr,
            child_killed,
            child_reaped,
            limit_profile_id: limits.profile_id(),
            limit_profile_sha256: limits.profile_sha256(),
        }
    }

    /// Adds validated request and scenario identities.
    #[must_use]
    pub fn with_request(
        mut self,
        request_id: RequestId,
        request_sha256: Sha256Hex,
        scenario_sha256: Sha256Hex,
    ) -> Self {
        self.maybe_request_id = Some(request_id);
        self.maybe_request_sha256 = Some(request_sha256);
        self.maybe_scenario_sha256 = Some(scenario_sha256);
        self
    }

    /// Adds the validated process-session build identity.
    #[must_use]
    pub fn with_session_identity(mut self, identity_sha256: Sha256Hex) -> Self {
        self.maybe_session_identity_sha256 = Some(identity_sha256);
        self
    }

    /// Adds an available child exit status.
    #[must_use]
    pub const fn with_exit_status(mut self, exit_status: i32) -> Self {
        self.maybe_exit_status = Some(exit_status);
        self
    }

    /// Adds the last completely validated protocol record.
    #[must_use]
    pub const fn with_last_valid_record(mut self, record: LastValidRecord) -> Self {
        self.maybe_last_valid_record = Some(record);
        self
    }

    /// Returns the optional request identity.
    #[must_use]
    pub const fn maybe_request_id(&self) -> Option<&RequestId> {
        self.maybe_request_id.as_ref()
    }

    /// Returns the optional request content hash.
    #[must_use]
    pub const fn maybe_request_sha256(&self) -> Option<&Sha256Hex> {
        self.maybe_request_sha256.as_ref()
    }

    /// Returns the optional scenario content hash.
    #[must_use]
    pub const fn maybe_scenario_sha256(&self) -> Option<&Sha256Hex> {
        self.maybe_scenario_sha256.as_ref()
    }

    /// Returns the optional process-session identity hash.
    #[must_use]
    pub const fn maybe_session_identity_sha256(&self) -> Option<&Sha256Hex> {
        self.maybe_session_identity_sha256.as_ref()
    }

    /// Returns an available child exit status.
    #[must_use]
    pub const fn maybe_exit_status(&self) -> Option<i32> {
        self.maybe_exit_status
    }

    /// Returns elapsed time at failure classification.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the last completely validated record, when available.
    #[must_use]
    pub const fn maybe_last_valid_record(&self) -> Option<LastValidRecord> {
        self.maybe_last_valid_record
    }

    /// Returns bounded stderr evidence.
    #[must_use]
    pub const fn stderr(&self) -> &StderrEvidence {
        &self.stderr
    }

    /// Reports whether the harness killed the child.
    #[must_use]
    pub const fn child_killed(&self) -> bool {
        self.child_killed
    }

    /// Reports whether the harness waited for and reaped the child.
    #[must_use]
    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }

    /// Returns the immutable limit profile identifier.
    #[must_use]
    pub const fn limit_profile_id(&self) -> &'static str {
        self.limit_profile_id
    }

    /// Returns the immutable limit profile identity hash.
    #[must_use]
    pub const fn limit_profile_sha256(&self) -> &Sha256Hex {
        &self.limit_profile_sha256
    }
}

/// A classified harness failure with bounded diagnostic evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessFailure {
    kind: HarnessFailureKind,
    evidence: Box<HarnessFailureEvidence>,
}

impl HarnessFailure {
    /// Combines a non-physics failure category with bounded evidence.
    #[must_use]
    pub fn new(kind: HarnessFailureKind, evidence: HarnessFailureEvidence) -> Self {
        Self {
            kind,
            evidence: Box::new(evidence),
        }
    }

    /// Returns the classified harness failure kind.
    #[must_use]
    pub const fn kind(&self) -> HarnessFailureKind {
        self.kind
    }

    /// Returns the bounded failure evidence.
    #[must_use]
    pub const fn evidence(&self) -> &HarnessFailureEvidence {
        &self.evidence
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        HarnessFailure, HarnessFailureEvidence, HarnessFailureKind, LastValidRecord, StderrEvidence,
    };
    use crate::{HarnessLimits, RequestId, Sha256Hex};

    #[test]
    fn harness_failure_taxonomy_covers_every_non_physics_condition() {
        // Arrange
        let expected = [
            "startup_timeout",
            "handshake_malformed",
            "unsupported_version",
            "wrong_provenance",
            "request_timeout",
            "child_non_zero_exit",
            "child_signaled",
            "sanitizer_report",
            "unexpected_eof",
            "partial_record",
            "malformed_record",
            "unknown_record_kind",
            "record_too_large",
            "trace_too_large",
            "total_output_exceeded",
            "sequence_violation",
            "request_id_mismatch",
            "trace_identity_mismatch",
            "scenario_rejected",
            "rust_adapter_failure",
            "cpp_adapter_failure",
            "adapter_reset_failure",
        ];

        // Act
        let actual = HarnessFailureKind::ALL.map(HarnessFailureKind::as_str);

        // Assert
        assert_eq!(actual, expected);
        assert!(!actual.contains(&"physics_mismatch"));
    }

    #[test]
    fn stderr_evidence_rejects_retention_over_the_reviewed_limit() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let retained = vec![b'x'; limits.retained_stderr_bytes() + 1];

        // Act
        let result = StderrEvidence::new(retained, limits.retained_stderr_bytes() + 1, &limits);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn stderr_evidence_accepts_retention_at_the_reviewed_limit() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let retained = vec![b'x'; limits.retained_stderr_bytes()];

        // Act
        let evidence = StderrEvidence::new(retained, limits.retained_stderr_bytes(), &limits)
            .expect("retention at the reviewed limit should pass");

        // Assert
        assert_eq!(evidence.retained().len(), limits.retained_stderr_bytes());
        assert_eq!(evidence.truncated_bytes(), 0);
    }

    #[test]
    fn harness_failure_preserves_bounded_process_and_identity_evidence() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let stderr = StderrEvidence::new(b"diagnostic".to_vec(), 20, &limits)
            .expect("bounded stderr should be valid");
        let request_id = RequestId::new("request-1").expect("request ID should be valid");
        let request_sha256 = Sha256Hex::new("11".repeat(32)).expect("hash should be valid");
        let scenario_sha256 = Sha256Hex::new("22".repeat(32)).expect("hash should be valid");
        let identity_sha256 = Sha256Hex::new("33".repeat(32)).expect("hash should be valid");

        // Act
        let evidence =
            HarnessFailureEvidence::new(Duration::from_millis(250), stderr, true, true, &limits)
                .with_request(request_id, request_sha256, scenario_sha256)
                .with_session_identity(identity_sha256)
                .with_exit_status(9)
                .with_last_valid_record(LastValidRecord::Checkpoint);
        let failure = HarnessFailure::new(HarnessFailureKind::SanitizerReport, evidence);

        // Assert
        assert_eq!(failure.kind(), HarnessFailureKind::SanitizerReport);
        assert_eq!(
            failure.evidence().maybe_request_id().map(RequestId::as_str),
            Some("request-1")
        );
        assert_eq!(failure.evidence().maybe_exit_status(), Some(9));
        assert_eq!(failure.evidence().elapsed(), Duration::from_millis(250));
        assert_eq!(
            failure.evidence().maybe_last_valid_record(),
            Some(LastValidRecord::Checkpoint)
        );
        assert_eq!(failure.evidence().stderr().truncated_bytes(), 10);
        assert!(failure.evidence().child_killed());
        assert!(failure.evidence().child_reaped());
        assert_eq!(failure.evidence().limit_profile_id(), "phase2-default-v1");
    }
}

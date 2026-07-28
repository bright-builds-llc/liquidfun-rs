//! Strict bounded wire records for paired benchmark execution.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{PerformanceEngineRole, PerformancePolicy};
use crate::{
    CheckpointId, CodecError, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, Sha256Hex,
    codec::BoundedVec, decode_jsonl, encode_jsonl,
};

const MAXIMUM_RESOLVED_BYTES: usize = 1024 * 1024;
const MAXIMUM_COMMON_PARENT_DIAGNOSTICS: usize = 16;
const MAXIMUM_MEASURED_HORIZON: u32 = 4_096;

mod request;
pub use request::{BenchmarkRunIdentity, BenchmarkRunRequest};

/// Stable validation categories for the paired benchmark wire contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkWireErrorKind {
    /// Exact resolved bytes do not match their declared SHA-256 identity.
    ResolvedHashMismatch,
    /// Exact resolved bytes exceed the reviewed typed bound.
    ResolvedBytesTooLarge,
    /// Warm-up count differs from the reviewed measurement policy.
    InvalidWarmupCount,
    /// Measured logical horizon is zero or exceeds the reviewed bound.
    InvalidMeasuredHorizon,
    /// Sample ordinal is zero or exceeds the reviewed per-engine count.
    InvalidSampleOrdinal,
    /// Policy hash does not identify the reviewed Phase 12 policy.
    PolicyMismatch,
    /// A reset epoch or authoritative duration is zero.
    InvalidMeasurement,
    /// Common-parent diagnostic phases contain duplicates.
    DuplicateDiagnosticPhase,
    /// Diagnostics were supplied while profiling was disabled.
    UnexpectedProfileDiagnostics,
    /// Semantic checkpoint identity contradicts the enclosing run.
    CheckpointIdentityMismatch,
    /// Request and result identities differ.
    RunIdentityMismatch,
}

/// Strict framing, decoding, or typed benchmark validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BenchmarkWireError {
    /// Generic strict JSONL framing or schema failure.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// A decoded record violated the paired benchmark contract.
    #[error("benchmark wire validation failure: {0:?}")]
    Validation(BenchmarkWireErrorKind),
}

impl BenchmarkWireError {
    /// Returns the strict JSONL failure category, when decoding failed before typed validation.
    #[must_use]
    pub const fn codec_kind(&self) -> Option<crate::CodecErrorKind> {
        match self {
            Self::Codec(error) => Some(error.kind()),
            Self::Validation(_) => None,
        }
    }

    /// Returns the benchmark validation category, when typed validation failed.
    #[must_use]
    pub const fn validation_kind(&self) -> Option<BenchmarkWireErrorKind> {
        match self {
            Self::Codec(_) => None,
            Self::Validation(kind) => Some(*kind),
        }
    }
}

/// SHA-256 identity of the exact reviewed Phase 12 performance policy.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] if deterministic policy serialization fails.
pub fn benchmark_policy_sha256() -> Result<Sha256Hex, BenchmarkWireError> {
    let bytes = serde_json::to_vec(&PerformancePolicy::reviewed_v1())
        .map_err(|_| validation(BenchmarkWireErrorKind::PolicyMismatch))?;
    Ok(Sha256Hex::from_digest(Sha256::digest(bytes).into()))
}

/// Closed common parent phases comparable across both engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCommonParentPhase {
    /// Broad-phase update and pair discovery.
    BroadPhase,
    /// Narrow-phase manifold generation.
    NarrowPhase,
    /// Contact constraint solving.
    ContactSolve,
    /// Joint constraint solving.
    JointSolve,
    /// Particle-system solving.
    ParticleSolve,
    /// Query or ray-cast traversal.
    QueryTraversal,
}

/// Optional non-authoritative duration for one common parent phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCommonParentDiagnostic {
    phase: BenchmarkCommonParentPhase,
    nanoseconds: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBenchmarkCommonParentDiagnostic {
    phase: BenchmarkCommonParentPhase,
    nanoseconds: u64,
}

impl<'de> Deserialize<'de> for BenchmarkCommonParentDiagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawBenchmarkCommonParentDiagnostic::deserialize(deserializer)?;
        Self::new(raw.phase, raw.nanoseconds).map_err(serde::de::Error::custom)
    }
}

impl BenchmarkCommonParentDiagnostic {
    /// Creates a non-authoritative diagnostic duration.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkWireError`] when the duration is zero.
    pub fn new(
        phase: BenchmarkCommonParentPhase,
        nanoseconds: u64,
    ) -> Result<Self, BenchmarkWireError> {
        if nanoseconds == 0 {
            return Err(validation(BenchmarkWireErrorKind::InvalidMeasurement));
        }
        Ok(Self { phase, nanoseconds })
    }

    /// Returns the closed common parent phase.
    #[must_use]
    pub const fn phase(self) -> BenchmarkCommonParentPhase {
        self.phase
    }

    /// Returns the raw diagnostic duration.
    #[must_use]
    pub const fn nanoseconds(self) -> u64 {
        self.nanoseconds
    }
}

/// Identity of the semantic checkpoint protecting one timing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticCheckpointIdentity {
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    checkpoint_id: CheckpointId,
    checkpoint_sha256: Sha256Hex,
}

impl SemanticCheckpointIdentity {
    /// Creates a semantic checkpoint identity without exposing engine storage.
    #[must_use]
    pub const fn new(
        request_id: RequestId,
        resolved_sha256: Sha256Hex,
        checkpoint_id: CheckpointId,
        checkpoint_sha256: Sha256Hex,
    ) -> Self {
        Self {
            request_id,
            resolved_sha256,
            checkpoint_id,
            checkpoint_sha256,
        }
    }

    /// Returns the stable checkpoint ID.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the semantic checkpoint hash.
    #[must_use]
    pub const fn checkpoint_sha256(&self) -> &Sha256Hex {
        &self.checkpoint_sha256
    }
}

/// Successful authoritative unprofiled measurement plus optional diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPerformanceResult {
    unprofiled_nanoseconds: u64,
    maybe_common_parent_diagnostics: Option<Box<[BenchmarkCommonParentDiagnostic]>>,
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBenchmarkPerformanceResult {
    unprofiled_nanoseconds: u64,
    maybe_common_parent_diagnostics:
        Option<BoundedVec<BenchmarkCommonParentDiagnostic, MAXIMUM_COMMON_PARENT_DIAGNOSTICS>>,
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
}

impl<'de> Deserialize<'de> for BenchmarkPerformanceResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawBenchmarkPerformanceResult::deserialize(deserializer)?;
        Self::new(
            raw.unprofiled_nanoseconds,
            raw.maybe_common_parent_diagnostics
                .map(BoundedVec::into_vec),
            raw.semantic_checkpoint_identity,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl BenchmarkPerformanceResult {
    /// Validates authoritative time and bounded, unique common-parent diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkWireError`] for zero time, too many diagnostics, or duplicate phases.
    pub fn new(
        unprofiled_nanoseconds: u64,
        maybe_common_parent_diagnostics: Option<Vec<BenchmarkCommonParentDiagnostic>>,
        semantic_checkpoint_identity: SemanticCheckpointIdentity,
    ) -> Result<Self, BenchmarkWireError> {
        if unprofiled_nanoseconds == 0 {
            return Err(validation(BenchmarkWireErrorKind::InvalidMeasurement));
        }
        let diagnostics = maybe_common_parent_diagnostics
            .map(|diagnostics| {
                if diagnostics.len() > MAXIMUM_COMMON_PARENT_DIAGNOSTICS {
                    return Err(validation(BenchmarkWireErrorKind::InvalidMeasurement));
                }
                let unique = diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.phase)
                    .collect::<BTreeSet<_>>();
                if unique.len() != diagnostics.len() {
                    return Err(validation(BenchmarkWireErrorKind::DuplicateDiagnosticPhase));
                }
                Ok(diagnostics.into_boxed_slice())
            })
            .transpose()?;
        Ok(Self {
            unprofiled_nanoseconds,
            maybe_common_parent_diagnostics: diagnostics,
            semantic_checkpoint_identity,
        })
    }

    /// Returns the only authoritative wall-clock duration.
    #[must_use]
    pub const fn unprofiled_nanoseconds(&self) -> u64 {
        self.unprofiled_nanoseconds
    }

    /// Returns optional non-authoritative common-parent diagnostics.
    #[must_use]
    pub fn maybe_common_parent_diagnostics(&self) -> Option<&[BenchmarkCommonParentDiagnostic]> {
        self.maybe_common_parent_diagnostics.as_deref()
    }

    /// Returns the semantic checkpoint identity protecting the measurement.
    #[must_use]
    pub const fn semantic_checkpoint_identity(&self) -> &SemanticCheckpointIdentity {
        &self.semantic_checkpoint_identity
    }
}

/// Semantic divergence reported separately from performance evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPhysicsMismatch {
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
}

impl BenchmarkPhysicsMismatch {
    /// Creates a physics-mismatch result bound to its semantic checkpoint.
    #[must_use]
    pub const fn new(semantic_checkpoint_identity: SemanticCheckpointIdentity) -> Self {
        Self {
            semantic_checkpoint_identity,
        }
    }

    /// Returns the first divergent semantic checkpoint identity.
    #[must_use]
    pub const fn semantic_checkpoint_identity(&self) -> &SemanticCheckpointIdentity {
        &self.semantic_checkpoint_identity
    }
}

/// Non-physics harness failure reported separately from measurements and mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkHarnessFailure {
    kind: BenchmarkHarnessFailureKind,
}

impl BenchmarkHarnessFailure {
    /// Creates a classified non-physics benchmark failure.
    #[must_use]
    pub const fn new(kind: BenchmarkHarnessFailureKind) -> Self {
        Self { kind }
    }

    /// Returns the closed harness failure category.
    #[must_use]
    pub const fn kind(self) -> BenchmarkHarnessFailureKind {
        self.kind
    }
}

/// Closed non-physics failures that a paired benchmark adapter may report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkHarnessFailureKind {
    /// Request exceeded its execution deadline.
    RequestTimeout,
    /// Child process exited unsuccessfully.
    ChildNonZeroExit,
    /// Child process terminated because of a signal.
    ChildSignaled,
    /// Sanitizer output invalidated the run.
    SanitizerReport,
    /// Protocol record was malformed.
    MalformedRecord,
    /// Child output exceeded a reviewed bound.
    OutputLimitExceeded,
    /// Child response contradicted request or build provenance.
    IdentityMismatch,
    /// Engine adapter failed before a valid result.
    AdapterFailure,
    /// Complete reset could not be proven.
    AdapterResetFailure,
}

/// Mutually exclusive benchmark terminal outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "outcome_kind",
    content = "outcome",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum BenchmarkRunOutcome {
    /// Authoritative unprofiled performance result.
    Performance(BenchmarkPerformanceResult),
    /// Semantic physics divergence, never a timing result.
    PhysicsMismatch(BenchmarkPhysicsMismatch),
    /// Process, protocol, provenance, or adapter failure.
    HarnessFailure(BenchmarkHarnessFailure),
}

/// Strict engine result echoing the complete run identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRunResult {
    identity: BenchmarkRunIdentity,
    engine_role: PerformanceEngineRole,
    reset_epoch: u64,
    outcome: BenchmarkRunOutcome,
}

impl BenchmarkRunResult {
    /// Validates one engine result against its own run and checkpoint identities.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkWireError`] for zero reset epochs, unexpected diagnostics, or
    /// checkpoint identity contradictions.
    pub fn new(
        identity: BenchmarkRunIdentity,
        engine_role: PerformanceEngineRole,
        reset_epoch: u64,
        outcome: BenchmarkRunOutcome,
    ) -> Result<Self, BenchmarkWireError> {
        if reset_epoch == 0 {
            return Err(validation(BenchmarkWireErrorKind::InvalidMeasurement));
        }
        let maybe_checkpoint = match &outcome {
            BenchmarkRunOutcome::Performance(result) => {
                if !identity.profile_enabled() && result.maybe_common_parent_diagnostics().is_some()
                {
                    return Err(validation(
                        BenchmarkWireErrorKind::UnexpectedProfileDiagnostics,
                    ));
                }
                Some(result.semantic_checkpoint_identity())
            }
            BenchmarkRunOutcome::PhysicsMismatch(mismatch) => {
                Some(mismatch.semantic_checkpoint_identity())
            }
            BenchmarkRunOutcome::HarnessFailure(_) => None,
        };
        if maybe_checkpoint.is_some_and(|checkpoint| {
            checkpoint.request_id != *identity.request_id()
                || checkpoint.resolved_sha256 != *identity.resolved_sha256()
        }) {
            return Err(validation(
                BenchmarkWireErrorKind::CheckpointIdentityMismatch,
            ));
        }
        Ok(Self {
            identity,
            engine_role,
            reset_epoch,
            outcome,
        })
    }

    /// Returns the complete echoed run identity.
    #[must_use]
    pub const fn identity(&self) -> &BenchmarkRunIdentity {
        &self.identity
    }

    /// Returns the engine that produced this result.
    #[must_use]
    pub const fn engine_role(&self) -> PerformanceEngineRole {
        self.engine_role
    }

    /// Returns the child-provided monotonic reset epoch.
    #[must_use]
    pub const fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }

    /// Returns the mutually exclusive terminal outcome.
    #[must_use]
    pub const fn outcome(&self) -> &BenchmarkRunOutcome {
        &self.outcome
    }
}

#[derive(Serialize)]
struct BenchmarkRunResultRef<'a> {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    identity: &'a BenchmarkRunIdentity,
    engine_role: PerformanceEngineRole,
    reset_epoch: u64,
    outcome: &'a BenchmarkRunOutcome,
}

impl Serialize for BenchmarkRunResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BenchmarkRunResultRef {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: "benchmark_run_result",
            identity: &self.identity,
            engine_role: self.engine_role,
            reset_epoch: self.reset_epoch,
            outcome: &self.outcome,
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBenchmarkRunResult {
    #[serde(rename = "protocol_version")]
    _protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: BenchmarkRunResultRecordKind,
    identity: BenchmarkRunIdentity,
    engine_role: PerformanceEngineRole,
    reset_epoch: u64,
    outcome: BenchmarkRunOutcome,
}

#[derive(Deserialize)]
enum BenchmarkRunResultRecordKind {
    #[serde(rename = "benchmark_run_result")]
    BenchmarkRunResult,
}

/// Encodes one strict newline-complete benchmark request.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] when serialization or the input record bound fails.
pub fn encode_benchmark_run_request_jsonl(
    request: &BenchmarkRunRequest,
    limits: &HarnessLimits,
) -> Result<Vec<u8>, BenchmarkWireError> {
    encode_jsonl(request, limits, RecordLimit::Input).map_err(Into::into)
}

/// Strictly decodes one bounded exact-byte benchmark request.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] for framing, schema, bounds, policy, or hash failures.
pub fn decode_benchmark_run_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<BenchmarkRunRequest, BenchmarkWireError> {
    request::decode(bytes, limits)
}

/// Encodes one strict newline-complete benchmark result.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] when serialization or the output record bound fails.
pub fn encode_benchmark_run_result_jsonl(
    result: &BenchmarkRunResult,
    limits: &HarnessLimits,
) -> Result<Vec<u8>, BenchmarkWireError> {
    encode_jsonl(result, limits, RecordLimit::Output).map_err(Into::into)
}

/// Strictly decodes one bounded benchmark result.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] for framing, schema, bounds, or identity failures.
pub fn decode_benchmark_run_result_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<BenchmarkRunResult, BenchmarkWireError> {
    let raw: RawBenchmarkRunResult = decode_jsonl(bytes, limits, RecordLimit::Output)?;
    BenchmarkRunResult::new(raw.identity, raw.engine_role, raw.reset_epoch, raw.outcome)
}

/// Confirms that a result echoes the exact identity of its request.
///
/// # Errors
///
/// Returns [`BenchmarkWireError`] when any request/result identity field differs.
pub fn validate_benchmark_run_pair(
    request: &BenchmarkRunRequest,
    result: &BenchmarkRunResult,
) -> Result<(), BenchmarkWireError> {
    if request.identity() != result.identity() {
        return Err(validation(BenchmarkWireErrorKind::RunIdentityMismatch));
    }
    Ok(())
}

const fn validation(kind: BenchmarkWireErrorKind) -> BenchmarkWireError {
    BenchmarkWireError::Validation(kind)
}

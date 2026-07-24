//! Immutable raw paired performance reports and fail-closed report assembly.

use std::collections::BTreeSet;

use liquidfun::{DiagnosticProfileChild, DiagnosticProfileSchema};
use liquidfun_test_protocol::{
    RequestId, RunSettings, Sha256Hex,
    performance::{
        BenchmarkCommonParentDiagnostic, BenchmarkHarnessFailureKind, BenchmarkRunIdentity,
        BenchmarkRunOutcome, BenchmarkRunRequest, BenchmarkRunResult, CompatibilityStatus,
        PerformanceEngineRole, PerformancePolicy, PerformanceReportIdentity, PerformanceSizePoint,
        PerformanceWorkloadKind, ScalarOptimizationMode, SemanticCheckpointIdentity,
        benchmark_policy_sha256,
    },
};
use serde::{Serialize, ser::SerializeStruct};
use sha2::{Digest, Sha256};

mod runner;
pub use runner::run_paired_benchmark;

/// Stable paired-report construction failures that occur before engine execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairedPerformanceErrorKind {
    /// A request ID, resolved payload, or report identity was invalid.
    Identity,
    /// A reviewed benchmark wire field could not be constructed.
    Wire,
    /// Rust-only diagnostics contradicted their engine, profile, or outcome.
    Diagnostic,
}

/// Redacted paired-report construction failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("paired performance construction failed: {kind:?}")]
pub struct PairedPerformanceError {
    kind: PairedPerformanceErrorKind,
}

impl PairedPerformanceError {
    const fn new(kind: PairedPerformanceErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable construction-failure category.
    #[must_use]
    pub const fn kind(self) -> PairedPerformanceErrorKind {
        self.kind
    }
}

/// One optional Rust-only child duration, retained strictly as a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustChildProfileDiagnostic {
    phase: DiagnosticProfileChild,
    nanoseconds: u64,
}

impl RustChildProfileDiagnostic {
    /// Creates one nonzero diagnostic child duration.
    ///
    /// # Errors
    ///
    /// Returns [`PairedPerformanceError`] when the duration is zero.
    pub const fn new(
        phase: DiagnosticProfileChild,
        nanoseconds: u64,
    ) -> Result<Self, PairedPerformanceError> {
        if nanoseconds == 0 {
            return Err(PairedPerformanceError::new(
                PairedPerformanceErrorKind::Diagnostic,
            ));
        }
        Ok(Self { phase, nanoseconds })
    }

    /// Returns the closed Rust-only child phase.
    #[must_use]
    pub const fn phase(self) -> DiagnosticProfileChild {
        self.phase
    }

    /// Returns the diagnostic duration.
    #[must_use]
    pub const fn nanoseconds(self) -> u64 {
        self.nanoseconds
    }
}

impl Serialize for RustChildProfileDiagnostic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RustChildProfileDiagnostic", 2)?;
        state.serialize_field("phase", self.phase.as_str())?;
        state.serialize_field("nanoseconds", &self.nanoseconds)?;
        state.end()
    }
}

/// One validated adapter response plus optional Rust-only diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkAdapterOutput {
    result: BenchmarkRunResult,
    rust_child_diagnostics: Box<[RustChildProfileDiagnostic]>,
    process_generation: u64,
}

impl BenchmarkAdapterOutput {
    /// Validates the diagnostic authority boundary for one adapter response.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkHarnessFailureKind::IdentityMismatch`] when diagnostics appear on the
    /// oracle, profiling is disabled, or child phases are duplicated.
    pub fn new(
        result: BenchmarkRunResult,
        rust_child_diagnostics: Vec<RustChildProfileDiagnostic>,
    ) -> Result<Self, BenchmarkHarnessFailureKind> {
        Self::new_with_process_generation(result, rust_child_diagnostics, 1)
    }

    /// Validates one response together with its raw supervisor process generation.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkHarnessFailureKind::AdapterResetFailure`] for generation zero, or
    /// [`BenchmarkHarnessFailureKind::IdentityMismatch`] for contradictory diagnostics.
    pub fn new_with_process_generation(
        result: BenchmarkRunResult,
        rust_child_diagnostics: Vec<RustChildProfileDiagnostic>,
        process_generation: u64,
    ) -> Result<Self, BenchmarkHarnessFailureKind> {
        if process_generation == 0 {
            return Err(BenchmarkHarnessFailureKind::AdapterResetFailure);
        }
        if !rust_child_diagnostics.is_empty()
            && (result.engine_role() != PerformanceEngineRole::NativeRust
                || !result.identity().profile_enabled()
                || !matches!(result.outcome(), BenchmarkRunOutcome::Performance(_)))
        {
            return Err(BenchmarkHarnessFailureKind::IdentityMismatch);
        }
        let unique = rust_child_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.phase())
            .collect::<BTreeSet<_>>();
        if unique.len() != rust_child_diagnostics.len() {
            return Err(BenchmarkHarnessFailureKind::IdentityMismatch);
        }
        Ok(Self {
            result,
            rust_child_diagnostics: rust_child_diagnostics.into_boxed_slice(),
            process_generation,
        })
    }

    /// Returns the validated typed terminal result.
    #[must_use]
    pub const fn result(&self) -> &BenchmarkRunResult {
        &self.result
    }

    /// Returns optional Rust-only diagnostic children.
    #[must_use]
    pub fn rust_child_diagnostics(&self) -> &[RustChildProfileDiagnostic] {
        &self.rust_child_diagnostics
    }

    /// Returns the supervisor generation that owns the child-provided reset epoch.
    #[must_use]
    pub const fn process_generation(&self) -> u64 {
        self.process_generation
    }
}

/// Exact engine order for one paired sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PairedEngineOrder {
    /// Native Rust runs before the pinned C++ oracle.
    NativeThenOracle,
    /// The pinned C++ oracle runs before native Rust.
    OracleThenNative,
}

/// Complete immutable input needed to execute one workload across the reviewed session policy.
pub struct PairedBenchmarkPlan {
    request_id_prefix: Box<str>,
    resolved_bytes: Box<[u8]>,
    resolved_sha256: Sha256Hex,
    settings: RunSettings,
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    optimization_mode: ScalarOptimizationMode,
    measured_horizon: u32,
    profile_enabled: bool,
    report_identity: PerformanceReportIdentity,
    compatibility_status: CompatibilityStatus,
    policy: PerformancePolicy,
    policy_sha256: Sha256Hex,
}

impl PairedBenchmarkPlan {
    /// Validates one identity-complete workload plan before either engine executes.
    ///
    /// # Errors
    ///
    /// Returns [`PairedPerformanceError`] for an invalid request prefix, resolved payload, report
    /// identity, or wire bound.
    #[allow(
        clippy::too_many_arguments,
        reason = "all ten paired workload and immutable report fields are mandatory"
    )]
    pub fn new(
        request_id_prefix: impl Into<String>,
        resolved_bytes: Vec<u8>,
        settings: RunSettings,
        workload: PerformanceWorkloadKind,
        size_point: PerformanceSizePoint,
        optimization_mode: ScalarOptimizationMode,
        measured_horizon: u32,
        profile_enabled: bool,
        report_identity: PerformanceReportIdentity,
        compatibility_status: CompatibilityStatus,
    ) -> Result<Self, PairedPerformanceError> {
        let request_id_prefix = request_id_prefix.into();
        let probe_id = format!("{request_id_prefix}-run-05-sample-30");
        RequestId::new(probe_id)
            .map_err(|_error| PairedPerformanceError::new(PairedPerformanceErrorKind::Identity))?;
        if resolved_bytes.is_empty() {
            return Err(PairedPerformanceError::new(
                PairedPerformanceErrorKind::Identity,
            ));
        }
        let resolved_sha256 = Sha256Hex::from_digest(Sha256::digest(&resolved_bytes).into());
        let policy = PerformancePolicy::reviewed_v1();
        let policy_sha256 = benchmark_policy_sha256()
            .map_err(|_error| PairedPerformanceError::new(PairedPerformanceErrorKind::Wire))?;
        if !report_identity_matches(&report_identity, &resolved_sha256, &policy_sha256) {
            return Err(PairedPerformanceError::new(
                PairedPerformanceErrorKind::Identity,
            ));
        }
        let plan = Self {
            request_id_prefix: request_id_prefix.into_boxed_str(),
            resolved_bytes: resolved_bytes.into_boxed_slice(),
            resolved_sha256,
            settings,
            workload,
            size_point,
            optimization_mode,
            measured_horizon,
            profile_enabled,
            report_identity,
            compatibility_status,
            policy,
            policy_sha256,
        };
        plan.request(1, 1)?;
        Ok(plan)
    }

    fn request(
        &self,
        baseline_run: u8,
        sample_ordinal: u16,
    ) -> Result<BenchmarkRunRequest, PairedPerformanceError> {
        let request_id = RequestId::new(format!(
            "{}-run-{baseline_run:02}-sample-{sample_ordinal:02}",
            self.request_id_prefix
        ))
        .map_err(|_error| PairedPerformanceError::new(PairedPerformanceErrorKind::Identity))?;
        let identity = BenchmarkRunIdentity::new(
            request_id,
            self.resolved_sha256.clone(),
            self.settings,
            self.workload,
            self.size_point,
            self.optimization_mode,
            self.policy.warmup_runs(),
            self.measured_horizon,
            sample_ordinal,
            self.policy_sha256.clone(),
            self.profile_enabled,
        )
        .map_err(|_error| PairedPerformanceError::new(PairedPerformanceErrorKind::Wire))?;
        BenchmarkRunRequest::new(identity, self.resolved_bytes.to_vec())
            .map_err(|_error| PairedPerformanceError::new(PairedPerformanceErrorKind::Wire))
    }
}

fn report_identity_matches(
    identity: &PerformanceReportIdentity,
    resolved_sha256: &Sha256Hex,
    policy_sha256: &Sha256Hex,
) -> bool {
    let Ok(value) = serde_json::to_value(identity) else {
        return false;
    };
    value
        .get("resolved_sha256")
        .and_then(serde_json::Value::as_str)
        == Some(resolved_sha256.as_str())
        && value
            .get("policy_sha256")
            .and_then(serde_json::Value::as_str)
            == Some(policy_sha256.as_str())
}

/// One raw paired sample with exact execution order, reset identities, and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PairedRawSample {
    baseline_run: u8,
    sample_ordinal: u16,
    engine_order: PairedEngineOrder,
    native_nanoseconds: u64,
    oracle_nanoseconds: u64,
    native_process_generation: u64,
    oracle_process_generation: u64,
    native_reset_epoch: u64,
    oracle_reset_epoch: u64,
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
    native_common_parent_diagnostics: Box<[BenchmarkCommonParentDiagnostic]>,
    oracle_common_parent_diagnostics: Box<[BenchmarkCommonParentDiagnostic]>,
    rust_child_diagnostics: Box<[RustChildProfileDiagnostic]>,
}

impl PairedRawSample {
    /// Returns the one-based independent baseline run.
    #[must_use]
    pub const fn baseline_run(&self) -> u8 {
        self.baseline_run
    }

    /// Returns the one-based sample ordinal within its independent run.
    #[must_use]
    pub const fn sample_ordinal(&self) -> u16 {
        self.sample_ordinal
    }

    /// Returns the exact execution order used for this pair.
    #[must_use]
    pub const fn engine_order(&self) -> PairedEngineOrder {
        self.engine_order
    }

    /// Returns the authoritative native Rust wall-clock total.
    #[must_use]
    pub const fn native_nanoseconds(&self) -> u64 {
        self.native_nanoseconds
    }

    /// Returns the authoritative pinned-oracle wall-clock total.
    #[must_use]
    pub const fn oracle_nanoseconds(&self) -> u64 {
        self.oracle_nanoseconds
    }

    /// Returns the native supervisor generation owning the native reset epoch.
    #[must_use]
    pub const fn native_process_generation(&self) -> u64 {
        self.native_process_generation
    }

    /// Returns the oracle supervisor generation owning the oracle reset epoch.
    #[must_use]
    pub const fn oracle_process_generation(&self) -> u64 {
        self.oracle_process_generation
    }

    /// Returns the native child-provided reset epoch.
    #[must_use]
    pub const fn native_reset_epoch(&self) -> u64 {
        self.native_reset_epoch
    }

    /// Returns the oracle child-provided reset epoch.
    #[must_use]
    pub const fn oracle_reset_epoch(&self) -> u64 {
        self.oracle_reset_epoch
    }

    /// Returns non-authoritative native common-parent diagnostics.
    #[must_use]
    pub fn native_common_parent_diagnostics(&self) -> &[BenchmarkCommonParentDiagnostic] {
        &self.native_common_parent_diagnostics
    }

    /// Returns non-authoritative oracle common-parent diagnostics.
    #[must_use]
    pub fn oracle_common_parent_diagnostics(&self) -> &[BenchmarkCommonParentDiagnostic] {
        &self.oracle_common_parent_diagnostics
    }

    /// Returns optional non-authoritative Rust-only child diagnostics.
    #[must_use]
    pub fn rust_child_diagnostics(&self) -> &[RustChildProfileDiagnostic] {
        &self.rust_child_diagnostics
    }
}

/// Complete immutable same-host raw report, before statistical interval analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PairedPerformanceReport {
    identity: PerformanceReportIdentity,
    compatibility_status: CompatibilityStatus,
    policy: PerformancePolicy,
    profile_schema: &'static str,
    raw_samples: Box<[PairedRawSample]>,
}

impl PairedPerformanceReport {
    /// Returns the immutable D-05 source, tool, flags, hardware, and hash identity.
    #[must_use]
    pub const fn identity(&self) -> &PerformanceReportIdentity {
        &self.identity
    }

    /// Returns the explicitly scoped compatibility status.
    #[must_use]
    pub const fn compatibility_status(&self) -> CompatibilityStatus {
        self.compatibility_status
    }

    /// Returns the complete reviewed measurement policy.
    #[must_use]
    pub const fn policy(&self) -> &PerformancePolicy {
        &self.policy
    }

    /// Returns the reviewed independent-run count.
    #[must_use]
    pub const fn independent_runs(&self) -> u8 {
        self.policy.baseline_runs()
    }

    /// Returns every raw pair in run-major, sample-major order.
    #[must_use]
    pub fn raw_samples(&self) -> &[PairedRawSample] {
        &self.raw_samples
    }

    /// Returns the structural diagnostic profile schema.
    #[must_use]
    pub const fn profile_schema(&self) -> DiagnosticProfileSchema {
        DiagnosticProfileSchema::Phase12V1
    }
}

/// Stable location and category of the first paired harness failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PairedHarnessFailure {
    baseline_run: u8,
    sample_ordinal: u16,
    engine_role: PerformanceEngineRole,
    kind: BenchmarkHarnessFailureKind,
}

impl PairedHarnessFailure {
    /// Returns the one-based independent run containing the first failure.
    #[must_use]
    pub const fn baseline_run(self) -> u8 {
        self.baseline_run
    }

    /// Returns the one-based sample ordinal containing the first failure.
    #[must_use]
    pub const fn sample_ordinal(self) -> u16 {
        self.sample_ordinal
    }

    /// Returns the engine whose execution first failed.
    #[must_use]
    pub const fn engine_role(self) -> PerformanceEngineRole {
        self.engine_role
    }

    /// Returns the closed non-physics failure category.
    #[must_use]
    pub const fn kind(self) -> BenchmarkHarnessFailureKind {
        self.kind
    }
}

/// Stable location and checkpoint identity of the first physics divergence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PairedPhysicsMismatch {
    baseline_run: u8,
    sample_ordinal: u16,
    engine_role: PerformanceEngineRole,
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
}

impl PairedPhysicsMismatch {
    /// Returns the one-based independent run containing the first mismatch.
    #[must_use]
    pub const fn baseline_run(&self) -> u8 {
        self.baseline_run
    }

    /// Returns the one-based sample ordinal containing the first mismatch.
    #[must_use]
    pub const fn sample_ordinal(&self) -> u16 {
        self.sample_ordinal
    }

    /// Returns the engine whose result first established divergence.
    #[must_use]
    pub const fn engine_role(&self) -> PerformanceEngineRole {
        self.engine_role
    }
}

/// Mutually exclusive terminal state for one paired workload session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome_kind", content = "outcome", rename_all = "snake_case")]
pub enum PairedBenchmarkOutcome {
    /// Every planned pair produced identity-complete raw timings.
    Performance(Box<PairedPerformanceReport>),
    /// A semantic checkpoint diverged; no duration report is accepted.
    PhysicsMismatch(PairedPhysicsMismatch),
    /// A process, protocol, provenance, identity, or reset failure occurred.
    HarnessFailure(PairedHarnessFailure),
}

//! One-resolution native catalog execution and semantic comparison.

use std::path::Path;

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogRunRequest, CheckpointDeclaration, HarnessLimits,
    Phase4PolicyProfile, ScenarioActionId, Sha256Hex, decode_catalog_run_request_jsonl,
    encode_canonical_checkpoint_jsonl,
};

use crate::{
    CatalogOracleSupervisor, ComparisonLimits, ComparisonModel, ComparisonState,
    NativeCatalogBackend, OracleExecutable, OraclePreset, SessionCommand, SessionController,
    SessionControllerError, SessionProfile, compare_canonical_checkpoints,
};

/// Stable catalog harness-failure categories, always distinct from physics mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogFailureKind {
    /// Bundle or terminal classification for a semantic physics divergence.
    PhysicsMismatch,
    /// Bundle or terminal classification for a non-physics harness failure.
    HarnessFailure,
    /// Exact request or resolved bytes failed strict validation.
    Protocol,
    /// Required child build or limits identity did not match.
    Provenance,
    /// Native execution or capture failed transactionally.
    NativeExecution,
    /// Child startup or request deadline elapsed.
    Timeout,
    /// Child exited or crashed before a complete result.
    ChildProcess,
    /// Child emitted malformed, incomplete, or unknown records.
    MalformedRecord,
    /// Child reset proof was absent or contradictory.
    ResetFailure,
    /// Reviewed byte, action, checkpoint, or diagnostic bounds were exceeded.
    ResourceLimit,
    /// Failure evidence was absent, contradictory, or unsafe to persist.
    Evidence,
}

/// Failure before two validated checkpoint sequences can become physics evidence.
#[derive(Debug, thiserror::Error)]
#[error("catalog harness failure: {kind:?}")]
pub struct CatalogRunnerError {
    kind: CatalogFailureKind,
}

impl CatalogRunnerError {
    pub(crate) const fn new(kind: CatalogFailureKind) -> Self {
        Self { kind }
    }

    /// Returns the stable non-physics failure category.
    #[must_use]
    pub const fn kind(&self) -> CatalogFailureKind {
        self.kind
    }
}

/// One engine's complete deterministic execution of exact resolved bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRunCapture {
    resolved_bytes: Box<[u8]>,
    resolved_sha256: Sha256Hex,
    action_log: Box<[ScenarioActionId]>,
    checkpoint_schedule: Box<[CheckpointDeclaration]>,
    checkpoints: Box<[CanonicalCheckpoint]>,
    canonical_checkpoint_bytes: Box<[Box<[u8]>]>,
}

impl CatalogRunCapture {
    /// Returns the only authoritative replay bytes.
    #[must_use]
    pub fn resolved_bytes(&self) -> &[u8] {
        &self.resolved_bytes
    }

    /// Returns the verified content identity of the exact replay bytes.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns every stable action identity in resolved source order.
    #[must_use]
    pub fn action_log(&self) -> &[ScenarioActionId] {
        &self.action_log
    }

    /// Returns every declared capture boundary in logical order.
    #[must_use]
    pub fn checkpoint_schedule(&self) -> &[CheckpointDeclaration] {
        &self.checkpoint_schedule
    }

    /// Returns validated semantic checkpoints in schedule order.
    #[must_use]
    pub fn checkpoints(&self) -> &[CanonicalCheckpoint] {
        &self.checkpoints
    }

    /// Returns canonical newline-complete checkpoint records in schedule order.
    #[must_use]
    pub fn canonical_checkpoint_bytes(&self) -> &[Box<[u8]>] {
        &self.canonical_checkpoint_bytes
    }

    /// Reconstructs one capture from strict canonical checkpoint records and exact request bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogRunnerError`] for malformed records or any request/hash/schedule conflict.
    pub fn from_checkpoint_jsonl(
        request: &CatalogRunRequest,
        records: &[Vec<u8>],
    ) -> Result<Self, CatalogRunnerError> {
        let limits = HarnessLimits::phase2_default_v1();
        let checkpoints = records
            .iter()
            .map(|bytes| {
                liquidfun_test_protocol::decode_canonical_checkpoint_jsonl(bytes, &limits)
                    .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::MalformedRecord))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (declaration, checkpoint) in request.resolved().checkpoints().iter().zip(&checkpoints) {
            if checkpoint.request_id() != request.request_id()
                || checkpoint.resolved_sha256() != request.resolved().identity().content_sha256()
                || checkpoint.checkpoint_id() != declaration.checkpoint_id()
            {
                return Err(CatalogRunnerError::new(CatalogFailureKind::Protocol));
            }
        }
        Self::from_parts(request, checkpoints)
    }

    pub(crate) fn from_parts(
        request: &CatalogRunRequest,
        checkpoints: Vec<CanonicalCheckpoint>,
    ) -> Result<Self, CatalogRunnerError> {
        if checkpoints.len() != request.resolved().checkpoints().len() {
            return Err(CatalogRunnerError::new(CatalogFailureKind::MalformedRecord));
        }
        let limits = HarnessLimits::phase2_default_v1();
        let canonical_checkpoint_bytes = checkpoints
            .iter()
            .map(|checkpoint| {
                encode_canonical_checkpoint_jsonl(checkpoint, &limits)
                    .map(Vec::into_boxed_slice)
                    .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::Protocol))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            resolved_bytes: request.resolved().canonical_bytes().into(),
            resolved_sha256: request.resolved().identity().content_sha256().clone(),
            action_log: request
                .resolved()
                .actions()
                .iter()
                .map(|action| action.action_id().clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            checkpoint_schedule: request.resolved().checkpoints().into(),
            checkpoints: checkpoints.into_boxed_slice(),
            canonical_checkpoint_bytes: canonical_checkpoint_bytes.into_boxed_slice(),
        })
    }
}

/// Successful equality evidence for two exact resolved-byte executions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMatch {
    comparisons: Box<[ComparisonModel]>,
}

impl CatalogMatch {
    /// Returns every checkpoint comparison in declared schedule order.
    #[must_use]
    pub fn comparisons(&self) -> &[ComparisonModel] {
        &self.comparisons
    }
}

/// First physics divergence plus all comparisons built before classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPhysicsMismatch {
    comparisons: Box<[ComparisonModel]>,
    first_mismatch: usize,
}

impl CatalogPhysicsMismatch {
    /// Returns the first mismatching checkpoint comparison.
    #[must_use]
    pub fn first_mismatch(&self) -> &ComparisonModel {
        &self.comparisons[self.first_mismatch]
    }
}

/// Closed distinction between equality, physics divergence, and harness failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogRunOutcome {
    /// Both engines produced equal or within-policy semantic checkpoints.
    Match(CatalogMatch),
    /// Both engines completed but semantic state diverged.
    PhysicsMismatch(CatalogPhysicsMismatch),
    /// Validation, process, provenance, reset, or resource handling failed first.
    HarnessFailure(CatalogFailureKind),
}

/// Executes one validated request natively without regenerating its resolved plan.
///
/// # Errors
///
/// Returns [`CatalogRunnerError`] for transactional native controller failures.
pub fn execute_catalog_native(
    request: &CatalogRunRequest,
) -> Result<CatalogRunCapture, CatalogRunnerError> {
    let mut backend = NativeCatalogBackend::new();
    backend.set_request_id(request.request_id().clone());
    let mut controller = SessionController::new(backend);
    submit(
        &mut controller,
        SessionCommand::Select {
            resolved: request.resolved().clone(),
        },
    )?;
    for declaration in request.resolved().checkpoints() {
        submit(&mut controller, SessionCommand::StepOnce)?;
        submit(
            &mut controller,
            SessionCommand::CaptureCheckpoint {
                checkpoint_id: declaration.checkpoint_id().clone(),
            },
        )?;
    }
    let checkpoints = controller
        .captures()
        .iter()
        .map(|capture| capture.value().clone())
        .collect();
    CatalogRunCapture::from_parts(request, checkpoints)
}

/// Strictly decodes exact request bytes and executes their embedded resolved bytes natively.
///
/// # Errors
///
/// Returns [`CatalogRunnerError`] for framing, identity, or native execution failure.
pub fn replay_catalog_exact_native(bytes: &[u8]) -> Result<CatalogRunCapture, CatalogRunnerError> {
    let request = decode_catalog_run_request_jsonl(bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::Protocol))?;
    execute_catalog_native(&request)
}

/// Executes one already-resolved request through native Rust and the supervised C++ child.
///
/// # Errors
///
/// Returns [`CatalogRunnerError`] only for local executable resolution or native setup. Child,
/// provenance, protocol, timeout, reset, and comparison failures remain classified outcomes.
pub fn run_catalog_resolved(
    repository_root: &Path,
    request: &CatalogRunRequest,
    preset: OraclePreset,
    profile: SessionProfile,
    expected_oracle_revision: &str,
) -> Result<CatalogRunOutcome, CatalogRunnerError> {
    let native = execute_catalog_native(request)?;
    let executable = OracleExecutable::resolve(repository_root, preset)
        .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::ChildProcess))?;
    let mut supervisor =
        CatalogOracleSupervisor::new(executable, profile, expected_oracle_revision);
    let oracle = match supervisor.execute(request) {
        Ok(captured) => captured,
        Err(error) => return Ok(CatalogRunOutcome::HarnessFailure(error.kind())),
    };
    compare_catalog(&native, oracle.capture())
}

/// Strictly decodes exact request JSONL before running both engines without regeneration.
///
/// # Errors
///
/// Returns [`CatalogRunnerError`] for request framing or local execution setup failure.
pub fn replay_catalog_resolved_exact(
    repository_root: &Path,
    bytes: &[u8],
    preset: OraclePreset,
    profile: SessionProfile,
    expected_oracle_revision: &str,
) -> Result<CatalogRunOutcome, CatalogRunnerError> {
    let request = decode_catalog_run_request_jsonl(bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::Protocol))?;
    run_catalog_resolved(
        repository_root,
        &request,
        preset,
        profile,
        expected_oracle_revision,
    )
}

/// Compares two already validated captures without reading either engine again.
///
/// # Errors
///
/// Returns [`CatalogRunnerError`] when exact replay authority or checkpoint structure conflicts.
pub fn compare_catalog(
    native: &CatalogRunCapture,
    oracle: &CatalogRunCapture,
) -> Result<CatalogRunOutcome, CatalogRunnerError> {
    if native.resolved_bytes != oracle.resolved_bytes
        || native.resolved_sha256 != oracle.resolved_sha256
        || native.action_log != oracle.action_log
        || native.checkpoint_schedule != oracle.checkpoint_schedule
        || native.checkpoints.len() != oracle.checkpoints.len()
    {
        return Ok(CatalogRunOutcome::HarnessFailure(
            CatalogFailureKind::Protocol,
        ));
    }
    let comparisons = native
        .checkpoints
        .iter()
        .zip(&oracle.checkpoints)
        .map(|(rust, cpp)| {
            let policies = Phase4PolicyProfile::parse_toml(include_str!(
                "../../../../protocol/tolerances/phase4-v1.toml"
            ))
            .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::Protocol))?;
            compare_canonical_checkpoints(rust, cpp, &policies, ComparisonLimits::phase11_default())
                .map_err(|_error| CatalogRunnerError::new(CatalogFailureKind::Protocol))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let maybe_first_mismatch = comparisons.iter().position(|model| {
        matches!(
            model.state(),
            ComparisonState::PhysicsMismatch
                | ComparisonState::RustOnly
                | ComparisonState::OracleOnly
        )
    });
    if let Some(first_mismatch) = maybe_first_mismatch {
        return Ok(CatalogRunOutcome::PhysicsMismatch(CatalogPhysicsMismatch {
            comparisons: comparisons.into_boxed_slice(),
            first_mismatch,
        }));
    }
    Ok(CatalogRunOutcome::Match(CatalogMatch {
        comparisons: comparisons.into_boxed_slice(),
    }))
}

fn submit(
    controller: &mut SessionController<NativeCatalogBackend>,
    command: SessionCommand,
) -> Result<(), CatalogRunnerError> {
    let command_id = controller
        .next_command_id()
        .ok_or_else(|| CatalogRunnerError::new(CatalogFailureKind::ResourceLimit))?;
    controller
        .submit(command_id, command)
        .map(|_outcome| ())
        .map_err(map_controller_error)
}

fn map_controller_error(error: SessionControllerError) -> CatalogRunnerError {
    let kind = error
        .maybe_backend()
        .map_or(CatalogFailureKind::Protocol, |backend| {
            match backend.category() {
                crate::SessionBackendErrorCategory::ResourceLimit => {
                    CatalogFailureKind::ResourceLimit
                }
                crate::SessionBackendErrorCategory::Protocol => CatalogFailureKind::Protocol,
                _ => CatalogFailureKind::NativeExecution,
            }
        });
    CatalogRunnerError::new(kind)
}

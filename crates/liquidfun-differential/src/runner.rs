//! Library orchestration for named/exact differential runs.

use std::{fs, io, path::Path};

use liquidfun_test_protocol::{
    HarnessFailure, HarnessLimits, RecordLimit, RequestId, ScenarioDecodeError,
    ScenarioRequestRecord, decode_scenario_request_jsonl, encode_jsonl,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    DifferentialOutcome, EmptyWorldAdapter, EmptyWorldAdapterError, MismatchReport,
    OracleExecutable, OracleExecutableError, OraclePreset, OracleSupervisor, SessionProfile,
    compare,
};

/// Error before two validated engine traces can produce a classified outcome.
#[derive(Debug, thiserror::Error)]
pub enum DifferentialRunnerError {
    /// Scenario name is not in the checked-in allowlist.
    #[error("unknown checked-in scenario `{0}`")]
    UnknownScenario(String),
    /// Checked-in scenario request is a symbolic link.
    #[error("checked-in scenario request must not be a symbolic link")]
    ScenarioSymlink,
    /// Scenario request bytes could not be read.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Strict scenario request validation failed.
    #[error(transparent)]
    Scenario(#[from] ScenarioDecodeError),
    /// Reviewed oracle executable resolution failed.
    #[error(transparent)]
    Executable(#[from] OracleExecutableError),
    /// Native adapter construction or execution failed.
    #[error(transparent)]
    NativeAdapter(#[from] EmptyWorldAdapterError),
    /// A validated request could not be encoded for a distinct reuse identity.
    #[error("validated reuse request could not be encoded: {0}")]
    Encode(String),
}

/// Reset epochs and request identity for one successful two-engine comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedRequest {
    request_id: Box<str>,
    cpp_reset_epoch: u64,
    rust_reset_epoch: u64,
}

impl MatchedRequest {
    /// Returns the distinguishable request identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Returns the C++ adapter reset epoch.
    #[must_use]
    pub const fn cpp_reset_epoch(&self) -> u64 {
        self.cpp_reset_epoch
    }

    /// Returns the native Rust adapter reset epoch.
    #[must_use]
    pub const fn rust_reset_epoch(&self) -> u64 {
        self.rust_reset_epoch
    }
}

/// Complete successful result for one command, possibly containing two reused requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchRun {
    requests: Box<[MatchedRequest]>,
}

impl MatchRun {
    /// Returns successful requests in execution order.
    #[must_use]
    pub fn requests(&self) -> &[MatchedRequest] {
        &self.requests
    }
}

/// Top-level distinction between match, physics mismatch, and harness failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifferentialRunOutcome {
    /// Every executed request matched.
    Match(MatchRun),
    /// Compatible traces contained a semantic divergence.
    PhysicsMismatch(MismatchReport),
    /// Process/protocol/provenance validation failed before physics comparison.
    HarnessFailure(HarnessFailure),
}

/// Runs an allowlisted checked-in named scenario.
///
/// Reuse and sanitizer modes execute two distinguishable requests through one child.
///
/// # Errors
///
/// Returns [`DifferentialRunnerError`] for scenario lookup/validation, executable resolution, or
/// native adapter failures. Process and comparator failures remain classified outcomes.
pub fn run_named(
    repository_root: &Path,
    scenario_name: &str,
    preset: OraclePreset,
    profile: SessionProfile,
    expected_oracle_revision: &str,
) -> Result<DifferentialRunOutcome, DifferentialRunnerError> {
    let relative = match scenario_name {
        "empty-world" => "protocol/fixtures/accepted/empty-world-request.jsonl",
        other => return Err(DifferentialRunnerError::UnknownScenario(other.to_owned())),
    };
    let request_path = repository_root.join(relative);
    if fs::symlink_metadata(&request_path)?
        .file_type()
        .is_symlink()
    {
        return Err(DifferentialRunnerError::ScenarioSymlink);
    }
    let bytes = fs::read(request_path)?;
    let request = decode_scenario_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())?;
    run_request(
        repository_root,
        request,
        preset,
        profile,
        expected_oracle_revision,
    )
}

/// Replays one exact newline-complete validated request record.
///
/// # Errors
///
/// Returns [`DifferentialRunnerError`] for strict validation, executable resolution, or native
/// adapter failures. Process and comparator failures remain classified outcomes.
pub fn replay_exact(
    repository_root: &Path,
    request_bytes: &[u8],
    preset: OraclePreset,
    profile: SessionProfile,
    expected_oracle_revision: &str,
) -> Result<DifferentialRunOutcome, DifferentialRunnerError> {
    let request =
        decode_scenario_request_jsonl(request_bytes, &HarnessLimits::phase2_default_v1())?;
    run_request(
        repository_root,
        request,
        preset,
        profile,
        expected_oracle_revision,
    )
}

fn run_request(
    repository_root: &Path,
    request: ScenarioRequestRecord,
    preset: OraclePreset,
    profile: SessionProfile,
    expected_oracle_revision: &str,
) -> Result<DifferentialRunOutcome, DifferentialRunnerError> {
    let executable = OracleExecutable::resolve(repository_root, preset)?;
    let mut supervisor = OracleSupervisor::new(executable, profile, expected_oracle_revision);
    let mut native = EmptyWorldAdapter::new(expected_oracle_revision)?;
    let requests = requests_for_profile(request, profile)?;
    let mut matches = Vec::with_capacity(requests.len());

    for request in requests {
        let rust_trace = native.execute(&request)?;
        let cpp_trace = match supervisor.execute(&request) {
            Ok(trace) => trace,
            Err(failure) => return Ok(DifferentialRunOutcome::HarnessFailure(failure)),
        };
        let outcome = match compare(
            &cpp_trace,
            &rust_trace,
            &liquidfun_test_protocol::ToleranceProfile::phase2_v1(),
        ) {
            Ok(outcome) => outcome,
            Err(failure) => return Ok(DifferentialRunOutcome::HarnessFailure(failure)),
        };
        match outcome {
            DifferentialOutcome::Match => matches.push(MatchedRequest {
                request_id: request.request_id().as_str().into(),
                cpp_reset_epoch: cpp_trace.reset_epoch(),
                rust_reset_epoch: rust_trace.reset_epoch(),
            }),
            DifferentialOutcome::PhysicsMismatch(report) => {
                return Ok(DifferentialRunOutcome::PhysicsMismatch(report));
            }
        }
    }

    Ok(DifferentialRunOutcome::Match(MatchRun {
        requests: matches.into_boxed_slice(),
    }))
}

fn requests_for_profile(
    request: ScenarioRequestRecord,
    profile: SessionProfile,
) -> Result<Vec<ScenarioRequestRecord>, DifferentialRunnerError> {
    if profile == SessionProfile::OneShot {
        return Ok(vec![request]);
    }
    let digest = Sha256::digest(request.request_id().as_str().as_bytes());
    let second_id = RequestId::new(format!("reuse-{digest:x}"))
        .map_err(|error| DifferentialRunnerError::Encode(error.to_string()))?;
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_jsonl(&request, &limits, RecordLimit::Input)
        .map_err(|error| DifferentialRunnerError::Encode(error.to_string()))?;
    let text = String::from_utf8(encoded)
        .map_err(|error| DifferentialRunnerError::Encode(error.to_string()))?;
    let original = format!("\"request_id\":\"{}\"", request.request_id().as_str());
    let replacement = format!("\"request_id\":\"{}\"", second_id.as_str());
    let second_bytes = text.replacen(&original, &replacement, 1);
    let second = decode_scenario_request_jsonl(second_bytes.as_bytes(), &limits)?;
    Ok(vec![request, second])
}

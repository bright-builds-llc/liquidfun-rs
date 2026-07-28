//! Shared typed contracts for Phase 12 safety, regression, and coverage evidence.

use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    fs,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[path = "contract/validation.rs"]
mod validation;

#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use validation::*;

const REGRESSION_SCHEMA_VERSION: u32 = 1;
const COVERAGE_SCHEMA_VERSION: u32 = 1;
const RESULT_SCHEMA_VERSION: u32 = 1;
const MAXIMUM_CONTRACT_BYTES: usize = 1024 * 1024;
const REGRESSION_FIELDS: [&str; 14] = [
    "id",
    "minimized_path",
    "minimized_sha256",
    "target",
    "generator",
    "toolchain",
    "candidate_commit",
    "fix_commit",
    "oracle_identity",
    "tolerance_identity",
    "first_divergence_signature",
    "failure_class",
    "review_status",
    "named_test_path",
];
const COVERAGE_IDENTITY_FIELDS: [&str; 4] = [
    "candidate_commit",
    "toolchain_identity",
    "artifact_path",
    "artifact_sha256",
];
const COVERAGE_SUBSYSTEM_FIELDS: [&str; 3] = [
    "name",
    "exercised_files_or_leaves",
    "missed_files_or_leaves",
];

/// Fail-closed typed contract error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContractError {
    message: String,
}

impl ContractError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ContractError {}

/// Validated reviewed regression registry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegressionManifest {
    schema_version: u32,
    record_fields: Vec<String>,
    regressions: Vec<RegressionRecord>,
}

impl RegressionManifest {
    /// Returns the reviewed records in deterministic manifest order.
    pub(crate) fn regressions(&self) -> &[RegressionRecord] {
        &self.regressions
    }
}

/// One reviewed minimized regression and its complete provenance.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegressionRecord {
    id: String,
    minimized_path: String,
    minimized_sha256: String,
    target: String,
    generator: String,
    toolchain: String,
    candidate_commit: String,
    fix_commit: String,
    oracle_identity: Option<String>,
    tolerance_identity: Option<String>,
    first_divergence_signature: String,
    failure_class: FailureClass,
    review_status: ReviewStatus,
    named_test_path: String,
}

/// Closed finding classification shared with the fuzz handoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
enum FailureClass {
    Harness,
    PhysicsMismatch,
    Sanitizer,
    Timeout,
    Schema,
}

/// Explicit review state accepted by the tracked authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReviewStatus {
    Reviewed,
}

/// Validated coverage schema authority.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CoverageContract {
    schema_version: u32,
    parity_authority: bool,
    identity_fields: Vec<String>,
    subsystem_fields: Vec<String>,
    rust: CoverageSection,
    cpp: CoverageSection,
    differential: CoverageSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageSection {
    evidence_kinds: Vec<CoverageEvidenceKind>,
    toolchain_identities: Vec<String>,
    leaf_kind: CoverageLeafKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CoverageEvidenceKind {
    RustSanitizer,
    CppAsanUbsan,
    RustCoverage,
    CppCoverage,
    DifferentialCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CoverageLeafKind {
    Files,
    DifferentialLeaves,
}

/// Measured differential coverage derived from expected and observed leaf IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DifferentialLeafCoverage {
    schema_version: u32,
    leaf_kind: CoverageLeafKind,
    parity_authority: bool,
    exercised: Vec<String>,
    missed: Vec<String>,
}

impl DifferentialLeafCoverage {
    /// Returns leaves that were not observed during the differential run.
    pub(crate) fn missed(&self) -> &[String] {
        &self.missed
    }
}

/// Builds a deterministic differential-leaf report from measured observations.
pub(crate) fn differential_leaf_coverage(
    expected_bytes: &[u8],
    observed_bytes: &[u8],
) -> Result<DifferentialLeafCoverage, ContractError> {
    let expected = decode_leaf_ids(expected_bytes, "expected")?;
    let observed = decode_leaf_ids(observed_bytes, "observed")?;
    if !observed.is_subset(&expected) {
        return Err(ContractError::new(
            "observed differential coverage contains an unknown leaf",
        ));
    }
    Ok(DifferentialLeafCoverage {
        schema_version: COVERAGE_SCHEMA_VERSION,
        leaf_kind: CoverageLeafKind::DifferentialLeaves,
        parity_authority: false,
        exercised: observed.iter().cloned().collect(),
        missed: expected.difference(&observed).cloned().collect(),
    })
}

fn decode_leaf_ids(bytes: &[u8], field: &str) -> Result<BTreeSet<String>, ContractError> {
    if bytes.is_empty() || bytes.len() > MAXIMUM_CONTRACT_BYTES {
        return Err(ContractError::new(format!(
            "{field} differential leaf list violates the reviewed byte bound"
        )));
    }
    let values = serde_json::from_slice::<Vec<String>>(bytes)
        .map_err(|error| ContractError::new(format!("invalid {field} leaf list: {error}")))?;
    if values.is_empty() || values.iter().any(|value| !is_leaf_id(value)) {
        return Err(ContractError::new(format!(
            "{field} differential leaf list contains an invalid ID"
        )));
    }
    let unique = values.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != values.len() {
        return Err(ContractError::new(format!(
            "{field} differential leaf list contains a duplicate ID"
        )));
    }
    Ok(unique)
}

/// One complete produced coverage record set for later release-audit reuse.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    dead_code,
    reason = "Plan 12-15 consumes this shared validated record type directly"
)]
pub(crate) struct CoverageRecordSet {
    schema_version: u32,
    candidate_commit: String,
    parity_authority: bool,
    records: Vec<CoverageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    dead_code,
    reason = "constructed through the Plan 12-15 shared record validator"
)]
struct CoverageRecord {
    evidence_kind: CoverageEvidenceKind,
    candidate_commit: String,
    toolchain_identity: String,
    artifact_path: String,
    artifact_sha256: String,
    subsystems: Vec<CoverageSubsystem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    dead_code,
    reason = "constructed through the Plan 12-15 shared record validator"
)]
struct CoverageSubsystem {
    name: String,
    exercised_files_or_leaves: Vec<String>,
    missed_files_or_leaves: Vec<String>,
}

/// Complete validated result set for one exact candidate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegressionResultSet {
    schema_version: u32,
    candidate_sha: String,
    complete: bool,
    results: Vec<RegressionResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegressionResult {
    regression_id: String,
    candidate_sha: String,
    named_test_path: String,
    minimized_sha256: String,
    outcome: RegressionOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RegressionOutcome {
    Passed,
}

#[derive(Serialize)]
struct ExecutionEntry<'a> {
    regression_id: &'a str,
    named_test_path: &'a str,
    minimized_input: &'a str,
    minimized_sha256: &'a str,
    provenance: ExecutionProvenance<'a>,
}

#[derive(Serialize)]
struct ExecutionProvenance<'a> {
    target: &'a str,
    generator: &'a str,
    toolchain: &'a str,
    candidate_commit: &'a str,
    fix_commit: &'a str,
    oracle_identity: Option<&'a str>,
    tolerance_identity: Option<&'a str>,
    first_divergence_signature: &'a str,
    failure_class: FailureClass,
}

/// Parses and validates the tracked regression registry and its exact minimized bytes.
pub(crate) fn validate_regression_manifest_bytes(
    repository_root: &Path,
    bytes: &[u8],
) -> Result<RegressionManifest, ContractError> {
    enforce_size("regression manifest", bytes)?;
    let manifest: RegressionManifest = toml::from_str(
        std::str::from_utf8(bytes)
            .map_err(|error| ContractError::new(format!("manifest is not UTF-8: {error}")))?,
    )
    .map_err(|error| ContractError::new(format!("manifest TOML is invalid: {error}")))?;
    if manifest.schema_version != REGRESSION_SCHEMA_VERSION
        || manifest.record_fields != REGRESSION_FIELDS
    {
        return Err(ContractError::new(
            "regression schema version or exact record field registry differs",
        ));
    }

    let mut ids = BTreeSet::new();
    let mut named_tests = BTreeSet::new();
    let mut minimized_paths = BTreeSet::new();
    for record in &manifest.regressions {
        validate_regression_record(repository_root, record)?;
        if !ids.insert(record.id.as_str())
            || !named_tests.insert(record.named_test_path.as_str())
            || !minimized_paths.insert(record.minimized_path.as_str())
        {
            return Err(ContractError::new(
                "regression IDs, named tests, and minimized paths must be unique",
            ));
        }
    }
    Ok(manifest)
}

/// Renders the complete reviewed execution projection after typed validation.
pub(crate) fn render_execution_list(
    manifest: &RegressionManifest,
) -> Result<String, ContractError> {
    let entries = manifest
        .regressions
        .iter()
        .map(|record| ExecutionEntry {
            regression_id: &record.id,
            named_test_path: &record.named_test_path,
            minimized_input: &record.minimized_path,
            minimized_sha256: &record.minimized_sha256,
            provenance: ExecutionProvenance {
                target: &record.target,
                generator: &record.generator,
                toolchain: &record.toolchain,
                candidate_commit: &record.candidate_commit,
                fix_commit: &record.fix_commit,
                oracle_identity: record.oracle_identity.as_deref(),
                tolerance_identity: record.tolerance_identity.as_deref(),
                first_divergence_signature: &record.first_divergence_signature,
                failure_class: record.failure_class,
            },
        })
        .collect::<Vec<_>>();
    let mut rendered = serde_json::to_string_pretty(&entries)
        .map_err(|error| ContractError::new(format!("execution list render failed: {error}")))?;
    rendered.push('\n');
    Ok(rendered)
}

/// Parses and validates the tracked non-parity coverage schema.
pub(crate) fn validate_coverage_contract_bytes(
    bytes: &[u8],
) -> Result<CoverageContract, ContractError> {
    enforce_size("coverage contract", bytes)?;
    let contract: CoverageContract = serde_json::from_slice(bytes).map_err(|error| {
        ContractError::new(format!("coverage contract JSON is invalid: {error}"))
    })?;
    if contract.schema_version != COVERAGE_SCHEMA_VERSION
        || contract.parity_authority
        || contract.identity_fields != COVERAGE_IDENTITY_FIELDS
        || contract.subsystem_fields != COVERAGE_SUBSYSTEM_FIELDS
    {
        return Err(ContractError::new(
            "coverage schema, identity fields, subsystem fields, or parity authority differs",
        ));
    }
    validate_coverage_sections(&contract)?;
    Ok(contract)
}

/// Parses and validates one produced coverage record set against the shared contract.
#[allow(
    dead_code,
    reason = "public within xtask for the Plan 12-15 release validator"
)]
pub(crate) fn validate_coverage_record_bytes(
    repository_root: &Path,
    contract: &CoverageContract,
    bytes: &[u8],
) -> Result<CoverageRecordSet, ContractError> {
    enforce_size("coverage record", bytes)?;
    let record_set: CoverageRecordSet = serde_json::from_slice(bytes)
        .map_err(|error| ContractError::new(format!("coverage record JSON is invalid: {error}")))?;
    if record_set.schema_version != COVERAGE_SCHEMA_VERSION
        || record_set.parity_authority
        || !is_full_sha(&record_set.candidate_commit)
    {
        return Err(ContractError::new(
            "coverage record schema, candidate, or parity authority is invalid",
        ));
    }

    let expected = coverage_kinds(contract);
    let mut actual = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for record in &record_set.records {
        validate_coverage_record(
            repository_root,
            contract,
            &record_set.candidate_commit,
            record,
        )?;
        if !actual.insert(record.evidence_kind) {
            return Err(ContractError::new("coverage evidence kind is duplicated"));
        }
        if !artifacts.insert(record.artifact_path.as_str()) {
            return Err(ContractError::new("coverage artifact path is duplicated"));
        }
    }
    if actual != expected {
        return Err(ContractError::new(
            "coverage evidence kinds are incomplete, merged, or unknown",
        ));
    }
    Ok(record_set)
}

/// Parses and validates a complete candidate-bound regression result set.
pub(crate) fn validate_regression_result_bytes(
    manifest: &RegressionManifest,
    candidate_sha: &str,
    bytes: &[u8],
) -> Result<RegressionResultSet, ContractError> {
    enforce_size("regression result set", bytes)?;
    let result_set: RegressionResultSet = serde_json::from_slice(bytes).map_err(|error| {
        ContractError::new(format!("regression result-set JSON is invalid: {error}"))
    })?;
    if result_set.schema_version != RESULT_SCHEMA_VERSION
        || !result_set.complete
        || result_set.candidate_sha != candidate_sha
    {
        return Err(ContractError::new(
            "result-set schema, completion marker, or candidate identity differs",
        ));
    }

    let expected = manifest
        .regressions
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut actual = BTreeSet::new();
    for result in &result_set.results {
        if !actual.insert(result.regression_id.as_str()) {
            return Err(ContractError::new("regression result is duplicated"));
        }
        let Some(record) = expected.get(result.regression_id.as_str()) else {
            return Err(ContractError::new("regression result is unregistered"));
        };
        if result.candidate_sha != candidate_sha
            || result.named_test_path != record.named_test_path
            || result.minimized_sha256 != record.minimized_sha256
            || result.outcome != RegressionOutcome::Passed
        {
            return Err(ContractError::new(
                "regression result carries mixed identity or an invalid outcome",
            ));
        }
    }
    if actual != expected.keys().copied().collect() {
        return Err(ContractError::new("regression result set is incomplete"));
    }
    Ok(result_set)
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

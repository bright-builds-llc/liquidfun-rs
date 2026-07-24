//! Shared typed contracts for Phase 12 safety, regression, and coverage evidence.

use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    fs,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

fn validate_regression_record(
    repository_root: &Path,
    record: &RegressionRecord,
) -> Result<(), ContractError> {
    if !is_identifier(&record.id)
        || record.target.is_empty()
        || record.generator.is_empty()
        || record.toolchain.is_empty()
        || !is_full_sha(&record.candidate_commit)
        || !is_full_sha(&record.fix_commit)
        || record.candidate_commit == record.fix_commit
        || record.first_divergence_signature.is_empty()
        || !record.named_test_path.contains("::")
        || record.review_status != ReviewStatus::Reviewed
    {
        return Err(ContractError::new(
            "regression identity, provenance, review, or named test is invalid",
        ));
    }
    if record.failure_class == FailureClass::PhysicsMismatch
        && (record.oracle_identity.as_deref().is_none_or(str::is_empty)
            || record
                .tolerance_identity
                .as_deref()
                .is_none_or(str::is_empty))
    {
        return Err(ContractError::new(
            "physics mismatch requires oracle and tolerance identities",
        ));
    }
    for maybe_identity in [&record.oracle_identity, &record.tolerance_identity] {
        if maybe_identity.as_deref().is_some_and(str::is_empty) {
            return Err(ContractError::new(
                "optional provenance identities cannot be empty",
            ));
        }
    }
    if !is_sha256(&record.minimized_sha256) {
        return Err(ContractError::new("minimized input SHA-256 is invalid"));
    }
    let relative = normalized_relative(&record.minimized_path)?;
    if !(relative.starts_with("scenarios/regressions")
        || relative.starts_with("fuzz/corpus/regressions"))
    {
        return Err(ContractError::new(
            "minimized input must remain under a reviewed regression root",
        ));
    }
    let bytes = read_confined_regular(repository_root, relative)?;
    if sha256(&bytes) != record.minimized_sha256 {
        return Err(ContractError::new("minimized input SHA-256 mismatch"));
    }
    Ok(())
}

fn validate_coverage_sections(contract: &CoverageContract) -> Result<(), ContractError> {
    if contract.rust.evidence_kinds
        != [
            CoverageEvidenceKind::RustSanitizer,
            CoverageEvidenceKind::RustCoverage,
        ]
        || contract.cpp.evidence_kinds
            != [
                CoverageEvidenceKind::CppAsanUbsan,
                CoverageEvidenceKind::CppCoverage,
            ]
        || contract.differential.evidence_kinds != [CoverageEvidenceKind::DifferentialCoverage]
        || contract.rust.leaf_kind != CoverageLeafKind::Files
        || contract.cpp.leaf_kind != CoverageLeafKind::Files
        || contract.differential.leaf_kind != CoverageLeafKind::DifferentialLeaves
        || contract.rust.toolchain_identities.is_empty()
        || contract.cpp.toolchain_identities.is_empty()
        || contract.differential.toolchain_identities.is_empty()
        || contract
            .rust
            .toolchain_identities
            .iter()
            .chain(&contract.cpp.toolchain_identities)
            .chain(&contract.differential.toolchain_identities)
            .any(String::is_empty)
    {
        return Err(ContractError::new(
            "Rust, C++, and differential coverage identities must remain distinct and complete",
        ));
    }
    Ok(())
}

fn validate_coverage_record(
    repository_root: &Path,
    contract: &CoverageContract,
    candidate_commit: &str,
    record: &CoverageRecord,
) -> Result<(), ContractError> {
    let expected_toolchains = match record.evidence_kind {
        CoverageEvidenceKind::RustSanitizer | CoverageEvidenceKind::RustCoverage => {
            &contract.rust.toolchain_identities
        }
        CoverageEvidenceKind::CppAsanUbsan | CoverageEvidenceKind::CppCoverage => {
            &contract.cpp.toolchain_identities
        }
        CoverageEvidenceKind::DifferentialCoverage => &contract.differential.toolchain_identities,
    };
    let relative_artifact = normalized_relative(&record.artifact_path)?;
    if record.candidate_commit != candidate_commit
        || !expected_toolchains.contains(&record.toolchain_identity)
        || !is_sha256(&record.artifact_sha256)
        || !relative_artifact.starts_with("target")
        || record.subsystems.is_empty()
    {
        return Err(ContractError::new(
            "coverage record identity or subsystem inventory is invalid",
        ));
    }
    let artifact_bytes = read_confined_regular(repository_root, relative_artifact)?;
    if sha256(&artifact_bytes) != record.artifact_sha256 {
        return Err(ContractError::new("coverage artifact SHA-256 mismatch"));
    }
    let mut subsystems = BTreeSet::new();
    for subsystem in &record.subsystems {
        let exercised = subsystem
            .exercised_files_or_leaves
            .iter()
            .collect::<BTreeSet<_>>();
        let missed = subsystem
            .missed_files_or_leaves
            .iter()
            .collect::<BTreeSet<_>>();
        if subsystem.name.is_empty()
            || !subsystems.insert(subsystem.name.as_str())
            || (subsystem.exercised_files_or_leaves.is_empty()
                && subsystem.missed_files_or_leaves.is_empty())
            || exercised.len() != subsystem.exercised_files_or_leaves.len()
            || missed.len() != subsystem.missed_files_or_leaves.len()
            || exercised.iter().any(|leaf| leaf.is_empty())
            || missed.iter().any(|leaf| leaf.is_empty())
            || !exercised.is_disjoint(&missed)
        {
            return Err(ContractError::new(
                "coverage subsystem leaves are incomplete or duplicated",
            ));
        }
    }
    Ok(())
}

fn coverage_kinds(contract: &CoverageContract) -> BTreeSet<CoverageEvidenceKind> {
    contract
        .rust
        .evidence_kinds
        .iter()
        .chain(&contract.cpp.evidence_kinds)
        .chain(&contract.differential.evidence_kinds)
        .copied()
        .collect()
}

fn normalized_relative(value: &str) -> Result<&Path, ContractError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ContractError::new("path is not normalized and relative"));
    }
    Ok(path)
}

fn read_confined_regular(
    repository_root: &Path,
    relative: &Path,
) -> Result<Vec<u8>, ContractError> {
    let canonical_root = fs::canonicalize(repository_root).map_err(|error| {
        ContractError::new(format!(
            "failed to resolve {}: {error}",
            repository_root.display()
        ))
    })?;
    let mut path = repository_root.to_path_buf();
    for component in relative {
        path.push(component);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            ContractError::new(format!("failed to inspect {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ContractError::new(format!(
                "{} contains a symbolic link",
                path.display()
            )));
        }
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        ContractError::new(format!("failed to inspect {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(ContractError::new(format!(
            "{} is not an ordinary file",
            path.display()
        )));
    }
    let canonical = fs::canonicalize(&path).map_err(|error| {
        ContractError::new(format!("failed to resolve {}: {error}", path.display()))
    })?;
    if !canonical.starts_with(canonical_root) {
        return Err(ContractError::new("contract artifact escaped repository"));
    }
    let length = usize::try_from(metadata.len())
        .map_err(|_error| ContractError::new("contract file length exceeds usize"))?;
    if length > MAXIMUM_CONTRACT_BYTES {
        return Err(ContractError::new("contract file exceeds reviewed bound"));
    }
    fs::read(&path)
        .map_err(|error| ContractError::new(format!("failed to read {}: {error}", path.display())))
}

fn enforce_size(field: &str, bytes: &[u8]) -> Result<(), ContractError> {
    if bytes.len() > MAXIMUM_CONTRACT_BYTES {
        return Err(ContractError::new(format!(
            "{field} exceeds reviewed byte bound"
        )));
    }
    Ok(())
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_leaf_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

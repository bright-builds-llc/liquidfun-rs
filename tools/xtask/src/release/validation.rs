//! Pure, bounded, fail-closed release evidence validation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::ReleaseError;
use super::domain::{
    EvidenceArtifact, EvidenceKey, ReleaseEvidenceKind, ReleaseEvidenceRecord, ReleaseManifest,
    ReleaseReadiness, RequiredEvidenceManifest, ValidatedEvidence,
};
use crate::safety_evidence::contract::{
    validate_coverage_contract_bytes, validate_regression_manifest_bytes,
};

const MANIFEST_SCHEMA_VERSION: u8 = 1;
const ARTIFACT_SCHEMA_VERSION: u8 = 1;
const REQUIRED_EVIDENCE_SCHEMA_VERSION: u8 = 1;
const MAXIMUM_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAXIMUM_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAXIMUM_ITEMS: usize = 256;
const REQUIRED_EVIDENCE_PATH: &str = "reference/release/required-evidence.toml";
const PACKAGE_NAME: &str = "liquidfun";
const PACKAGE_RUST_VERSION: &str = "1.92";
const CONDITIONAL_TARGET: &str = "x86_64-apple-darwin";
const CONDITIONAL_SECONDS: u64 = 90 * 86_400;

pub(crate) fn audit(
    repository_root: &Path,
    manifest_path: &Path,
    candidate: &str,
) -> Result<ReleaseReadiness, ReleaseError> {
    if !is_full_sha(candidate) {
        return Err(ReleaseError::new(
            "candidate",
            "candidate must be one lowercase 40-hex commit",
        ));
    }
    let manifest_bytes = read_input_manifest(repository_root, manifest_path)?;
    let manifest: ReleaseManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| ReleaseError::new("manifest-schema", error.to_string()))?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        || manifest.candidate_commit != candidate
        || manifest.items.len() > MAXIMUM_ITEMS
    {
        return Err(ReleaseError::new(
            "manifest",
            "manifest schema, candidate, or item bound is invalid",
        ));
    }

    let required = load_required_evidence(repository_root)?;
    let expected = required
        .evidence
        .iter()
        .map(|entry| (entry.key(), entry))
        .collect::<BTreeMap<_, _>>();
    let actual = unique_records(&manifest.items)?;
    if actual.keys().collect::<BTreeSet<_>>() != expected.keys().collect::<BTreeSet<_>>() {
        return Err(ReleaseError::new(
            "evidence-set",
            "required evidence identities are missing, unknown, or substituted",
        ));
    }

    validate_repository_authorities(repository_root)?;
    let mut validated = Vec::with_capacity(actual.len());
    let mut maybe_package_sha256 = None;
    for (key, requirement) in &expected {
        let record = actual
            .get(key)
            .copied()
            .ok_or_else(|| ReleaseError::new("evidence-set", "required evidence is missing"))?;
        validate_record_identity(record, requirement, candidate)?;
        let artifact_bytes = read_confined_regular(
            repository_root,
            Path::new(&record.artifact_path),
            MAXIMUM_ARTIFACT_BYTES,
            "artifact-path",
        )?;
        if sha256(&artifact_bytes) != record.artifact_sha256 {
            return Err(ReleaseError::new(
                "artifact-hash",
                format!("{key:?} artifact SHA-256 differs"),
            ));
        }
        let artifact: EvidenceArtifact = serde_json::from_slice(&artifact_bytes)
            .map_err(|error| ReleaseError::new("artifact-schema", error.to_string()))?;
        validate_artifact_envelope(record, &artifact, candidate)?;
        let payload_bytes = serde_json::to_vec(&artifact.claims)
            .map_err(|error| ReleaseError::new("payload", error.to_string()))?;
        if sha256(&payload_bytes) != artifact.payload_sha256
            || artifact.payload_sha256 != record.payload_sha256
        {
            return Err(ReleaseError::new(
                "payload-hash",
                format!("{key:?} payload SHA-256 differs"),
            ));
        }
        let package_sha256 = validate_claims(repository_root, record, &artifact)?;
        if let Some(package_sha256) = package_sha256 {
            match &maybe_package_sha256 {
                Some(expected_sha256) if expected_sha256 != &package_sha256 => {
                    return Err(ReleaseError::new(
                        "package-drift",
                        "package evidence differs across release targets",
                    ));
                }
                None => maybe_package_sha256 = Some(package_sha256),
                Some(_) => {}
            }
        }
        validated.push(ValidatedEvidence {
            kind: record.kind,
            target: record.target.clone(),
            workflow: record.producer.workflow.clone(),
            job: record.producer.job.clone(),
            run_id: record.producer.run_id.clone(),
            toolchain: record.toolchain.clone(),
            artifact_sha256: record.artifact_sha256.clone(),
            payload_sha256: record.payload_sha256.clone(),
        });
    }
    if maybe_package_sha256.is_none() {
        return Err(ReleaseError::new(
            "package-drift",
            "package identity was not established",
        ));
    }
    Ok(ReleaseReadiness {
        candidate_commit: candidate.to_owned(),
        evidence: validated,
    })
}

fn load_required_evidence(
    repository_root: &Path,
) -> Result<RequiredEvidenceManifest, ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new(REQUIRED_EVIDENCE_PATH),
        MAXIMUM_MANIFEST_BYTES,
        "required-evidence",
    )?;
    let manifest: RequiredEvidenceManifest = toml::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("required-evidence", error.to_string()))?;
    let unique = manifest
        .evidence
        .iter()
        .map(super::domain::RequiredEvidence::key)
        .collect::<BTreeSet<_>>();
    if manifest.schema_version != REQUIRED_EVIDENCE_SCHEMA_VERSION
        || manifest.evidence.is_empty()
        || manifest.evidence.len() > MAXIMUM_ITEMS
        || unique.len() != manifest.evidence.len()
        || manifest.evidence.iter().any(|entry| {
            entry.target.is_empty()
                || entry.workflow.is_empty()
                || entry.job.is_empty()
                || entry.toolchain.is_empty()
        })
    {
        return Err(ReleaseError::new(
            "required-evidence",
            "tracked required-evidence registry is invalid",
        ));
    }
    Ok(manifest)
}

fn unique_records(
    records: &[ReleaseEvidenceRecord],
) -> Result<BTreeMap<EvidenceKey, &ReleaseEvidenceRecord>, ReleaseError> {
    let mut unique = BTreeMap::new();
    for record in records {
        let key = record.key();
        if unique.insert(key, record).is_some() {
            return Err(ReleaseError::new(
                "duplicate-evidence",
                "evidence kind/target identity is duplicated",
            ));
        }
    }
    Ok(unique)
}

fn validate_record_identity(
    record: &ReleaseEvidenceRecord,
    required: &super::domain::RequiredEvidence,
    candidate: &str,
) -> Result<(), ReleaseError> {
    if record.candidate_commit != candidate {
        return Err(ReleaseError::new(
            "mixed-candidate",
            "evidence candidate differs from the audited commit",
        ));
    }
    if record.producer.workflow != required.workflow
        || record.producer.job != required.job
        || record.toolchain != required.toolchain
        || record.review_status != "reviewed"
        || record.status != "passed"
        || !is_run_id(&record.producer.run_id)
        || !is_sha256(&record.artifact_sha256)
        || !is_sha256(&record.payload_sha256)
    {
        return Err(ReleaseError::new(
            "evidence-identity",
            format!(
                "{}/{} has an unreviewed status or unallowlisted producer identity",
                record.kind, record.target
            ),
        ));
    }
    normalized_relative(&record.artifact_path, "artifact-path")?;
    Ok(())
}

fn validate_artifact_envelope(
    record: &ReleaseEvidenceRecord,
    artifact: &EvidenceArtifact,
    candidate: &str,
) -> Result<(), ReleaseError> {
    if artifact.schema_version != ARTIFACT_SCHEMA_VERSION
        || artifact.kind != record.kind
        || artifact.target != record.target
        || artifact.candidate_commit != candidate
        || artifact.status != "passed"
        || !is_sha256(&artifact.payload_sha256)
    {
        return Err(ReleaseError::new(
            "artifact-identity",
            format!(
                "{}/{} artifact envelope differs",
                record.kind, record.target
            ),
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match keeps every closed evidence kind visibly total"
)]
fn validate_claims(
    repository_root: &Path,
    record: &ReleaseEvidenceRecord,
    artifact: &EvidenceArtifact,
) -> Result<Option<String>, ReleaseError> {
    match record.kind {
        ReleaseEvidenceKind::Package => {
            let claims: PackageClaims = parse_claims(&artifact.claims)?;
            validate_package_claims(repository_root, &claims)?;
            Ok(Some(claims.package_sha256))
        }
        ReleaseEvidenceKind::Msrv => {
            let claims: PackageJoinClaims = parse_claims(&artifact.claims)?;
            if claims.rust_version.as_deref() != Some(PACKAGE_RUST_VERSION) || claims.package_drift
            {
                return Err(ReleaseError::new(
                    "package-drift",
                    "MSRV package identity drifted",
                ));
            }
            require_sha256(&claims.package_sha256, "package-drift")?;
            Ok(Some(claims.package_sha256))
        }
        ReleaseEvidenceKind::Platform => {
            let claims: PlatformClaims = parse_claims(&artifact.claims)?;
            validate_durable_platform(repository_root, &record.target, &claims)?;
            Ok(Some(claims.package_sha256))
        }
        ReleaseEvidenceKind::ConditionalPlatform => {
            let claims: ConditionalPlatformClaims = parse_claims(&artifact.claims)?;
            validate_conditional_platform(repository_root, &record.target, &claims)?;
            Ok(Some(claims.package_sha256))
        }
        ReleaseEvidenceKind::CanonicalDifferential => {
            let claims: DifferentialClaims = parse_claims(&artifact.claims)?;
            if claims.parity_tier != "d1_canonical"
                || claims.coverage_authority
                || claims.performance_authority
                || claims.gap_count != 0
            {
                return Err(ReleaseError::new(
                    "parity-authority",
                    "canonical differential evidence has a gap or non-parity authority",
                ));
            }
            Ok(None)
        }
        ReleaseEvidenceKind::RustSafety => {
            let claims: RustSafetyClaims = parse_claims(&artifact.claims)?;
            if claims.unsafe_waivers != 0
                || claims.advisory_waivers != 0
                || claims.unsafe_code != "forbid"
            {
                return Err(ReleaseError::new(
                    "safety",
                    "unsafe or advisory policy was weakened",
                ));
            }
            validate_safety_authorities(repository_root)?;
            Ok(None)
        }
        ReleaseEvidenceKind::CppSanitizer => {
            let claims: FindingsClaims = parse_claims(&artifact.claims)?;
            if claims.findings != 0 {
                return Err(ReleaseError::new(
                    "cpp-sanitizer",
                    "C++ sanitizer findings remain",
                ));
            }
            Ok(None)
        }
        ReleaseEvidenceKind::Fuzz => {
            let claims: FuzzClaims = parse_claims(&artifact.claims)?;
            if claims.findings != 0 || claims.target_count != 5 {
                return Err(ReleaseError::new(
                    "fuzz",
                    "fuzz evidence is incomplete or has findings",
                ));
            }
            Ok(None)
        }
        ReleaseEvidenceKind::Regressions => {
            let claims: RegressionClaims = parse_claims(&artifact.claims)?;
            validate_regressions(repository_root, &claims)?;
            Ok(None)
        }
        ReleaseEvidenceKind::RustCoverage | ReleaseEvidenceKind::CppCoverage => {
            let claims: CoverageClaims = parse_claims(&artifact.claims)?;
            validate_coverage(repository_root, &claims)?;
            Ok(None)
        }
        ReleaseEvidenceKind::Performance => {
            let claims: PerformanceClaims = parse_claims(&artifact.claims)?;
            validate_performance(repository_root, &claims)?;
            Ok(None)
        }
        ReleaseEvidenceKind::Docs => {
            let claims: DocsClaims = parse_claims(&artifact.claims)?;
            if !claims.docs_complete || claims.rustdoc_warnings != 0 {
                return Err(ReleaseError::new(
                    "docs",
                    "documentation or rustdoc evidence is incomplete",
                ));
            }
            require_repository_files(
                repository_root,
                &[
                    "README.md",
                    "RELEASE.md",
                    "COMPATIBILITY.md",
                    "BENCHMARKING.md",
                    "SAFETY.md",
                ],
                "docs",
            )?;
            Ok(None)
        }
        ReleaseEvidenceKind::Notices => {
            let claims: NoticeClaims = parse_claims(&artifact.claims)?;
            if !claims.notices_complete || claims.license != "MIT" || claims.advisory_waivers != 0 {
                return Err(ReleaseError::new(
                    "notices",
                    "license, notices, or advisory policy is incomplete",
                ));
            }
            require_repository_files(
                repository_root,
                &["LICENSE", "THIRD_PARTY_NOTICES.md"],
                "notices",
            )?;
            Ok(None)
        }
        ReleaseEvidenceKind::CorpusClosure => {
            let claims: CorpusClosureClaims = parse_claims(&artifact.claims)?;
            validate_corpus_closure(repository_root, &claims)?;
            Ok(None)
        }
        ReleaseEvidenceKind::CompatibilityClosure => {
            let claims: CompatibilityClosureClaims = parse_claims(&artifact.claims)?;
            validate_compatibility_closure(repository_root, &claims)?;
            Ok(None)
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageClaims {
    package_name: String,
    package_sha256: String,
    archive_path: String,
    archive_sha256: String,
    rust_version: String,
    scalar_mode: String,
    package_drift: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageJoinClaims {
    package_sha256: String,
    package_drift: bool,
    rust_version: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformClaims {
    package_sha256: String,
    package_drift: bool,
    evidence_tier: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConditionalDisposition {
    Supported,
    Unsupported,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConditionalPlatformClaims {
    package_sha256: String,
    package_drift: bool,
    disposition: ConditionalDisposition,
    recorded_at_unix: Option<u64>,
    expires_at_unix: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DifferentialClaims {
    parity_tier: String,
    coverage_authority: bool,
    performance_authority: bool,
    gap_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RustSafetyClaims {
    unsafe_waivers: u64,
    advisory_waivers: u64,
    unsafe_code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FindingsClaims {
    findings: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FuzzClaims {
    findings: u64,
    target_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegressionClaims {
    manifest_sha256: String,
    missing_results: u64,
    unreviewed_results: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageClaims {
    contract_sha256: String,
    parity_authority: bool,
    missing_subsystems: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PerformanceClaims {
    policy_sha256: String,
    timing_authority: String,
    claim_scope: String,
    claim_status: String,
    profile_authority: bool,
    reviewed_report_count: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocsClaims {
    docs_complete: bool,
    rustdoc_warnings: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoticeClaims {
    notices_complete: bool,
    license: String,
    advisory_waivers: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusClosureClaims {
    authority_sha256: String,
    item_count: usize,
    unresolved_count: usize,
    nonterminal_count: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityClosureClaims {
    authority_sha256: String,
    gap_count: usize,
    unexplained_count: usize,
    mixed_commit_count: usize,
    coverage_promoted_to_parity: bool,
    platform_promoted_to_parity: bool,
}

fn parse_claims<T: for<'de> Deserialize<'de>>(
    value: &serde_json::Value,
) -> Result<T, ReleaseError> {
    serde_json::from_value(value.clone())
        .map_err(|error| ReleaseError::new("claims-schema", error.to_string()))
}

fn validate_package_claims(
    repository_root: &Path,
    claims: &PackageClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "package-drift")?;
    require_sha256(&claims.archive_sha256, "package-drift")?;
    if claims.package_name != PACKAGE_NAME
        || claims.rust_version != PACKAGE_RUST_VERSION
        || claims.scalar_mode != "strict_f32"
        || claims.package_drift
        || claims.package_sha256 != claims.archive_sha256
    {
        return Err(ReleaseError::new(
            "package-drift",
            "package archive identity differs from the reviewed consumer contract",
        ));
    }
    let archive = read_confined_regular(
        repository_root,
        normalized_relative(&claims.archive_path, "package-drift")?,
        64 * 1024 * 1024,
        "package-drift",
    )?;
    if sha256(&archive) != claims.archive_sha256 {
        return Err(ReleaseError::new(
            "package-drift",
            "package archive bytes differ from their identity",
        ));
    }
    Ok(())
}

fn validate_durable_platform(
    repository_root: &Path,
    target: &str,
    claims: &PlatformClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "platform")?;
    let support = platform_support(repository_root)?;
    let durable = support
        .get("durable_targets")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("platform", "durable target policy is invalid"))?;
    if claims.package_drift
        || claims.evidence_tier != "d2_supported"
        || !durable.iter().any(|value| value.as_str() == Some(target))
    {
        return Err(ReleaseError::new(
            "platform",
            "durable platform evidence differs from the support policy",
        ));
    }
    Ok(())
}

fn validate_conditional_platform(
    repository_root: &Path,
    target: &str,
    claims: &ConditionalPlatformClaims,
) -> Result<(), ReleaseError> {
    require_sha256(&claims.package_sha256, "conditional-platform")?;
    let support = platform_support(repository_root)?;
    let conditional = support
        .get("conditional_targets")
        .and_then(serde_json::Value::as_array)
        .and_then(|values| values.first())
        .ok_or_else(|| ReleaseError::new("conditional-platform", "conditional policy is absent"))?;
    if target != CONDITIONAL_TARGET
        || conditional
            .get("target")
            .and_then(serde_json::Value::as_str)
            != Some(target)
        || claims.package_drift
    {
        return Err(ReleaseError::new(
            "conditional-platform",
            "conditional platform identity differs",
        ));
    }
    let maybe_native = conditional.get("native_evidence");
    match (&claims.disposition, maybe_native) {
        (ConditionalDisposition::Unsupported, Some(value)) if value.is_null() => {
            if claims.recorded_at_unix.is_some() || claims.expires_at_unix.is_some() {
                return Err(ReleaseError::new(
                    "conditional-platform",
                    "unsupported disposition cannot carry native freshness",
                ));
            }
        }
        (ConditionalDisposition::Supported, Some(value)) if !value.is_null() => {
            let recorded = claims.recorded_at_unix.ok_or_else(|| {
                ReleaseError::new(
                    "conditional-platform",
                    "native evidence timestamp is missing",
                )
            })?;
            let expires = claims.expires_at_unix.ok_or_else(|| {
                ReleaseError::new("conditional-platform", "native evidence expiry is missing")
            })?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| ReleaseError::new("conditional-platform", error.to_string()))?
                .as_secs();
            if recorded.checked_add(CONDITIONAL_SECONDS) != Some(expires)
                || recorded > now
                || expires < now
            {
                return Err(ReleaseError::new(
                    "conditional-platform",
                    "conditional platform evidence is stale",
                ));
            }
        }
        _ => {
            return Err(ReleaseError::new(
                "conditional-platform",
                "conditional disposition disagrees with tracked native evidence",
            ));
        }
    }
    Ok(())
}

fn validate_safety_authorities(repository_root: &Path) -> Result<(), ReleaseError> {
    let deny = read_confined_regular(
        repository_root,
        Path::new("deny.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "safety",
    )?;
    let deny: toml::Value =
        toml::from_slice(&deny).map_err(|error| ReleaseError::new("safety", error.to_string()))?;
    let ignores = deny
        .get("advisories")
        .and_then(|value| value.get("ignore"))
        .and_then(toml::Value::as_array);
    let cargo = read_confined_regular(
        repository_root,
        Path::new("Cargo.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "safety",
    )?;
    let cargo = std::str::from_utf8(&cargo)
        .map_err(|error| ReleaseError::new("safety", error.to_string()))?;
    if ignores.is_none_or(|values| !values.is_empty())
        || !cargo.contains("unsafe_code = \"forbid\"")
    {
        return Err(ReleaseError::new(
            "safety",
            "tracked unsafe or advisory policy was weakened",
        ));
    }
    Ok(())
}

fn validate_regressions(
    repository_root: &Path,
    claims: &RegressionClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/regressions/manifest.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "regressions",
    )?;
    validate_regression_manifest_bytes(repository_root, &bytes)
        .map_err(|error| ReleaseError::new("regressions", error.to_string()))?;
    if claims.manifest_sha256 != sha256(&bytes)
        || claims.missing_results != 0
        || claims.unreviewed_results != 0
    {
        return Err(ReleaseError::new(
            "regressions",
            "regression evidence is incomplete, stale, or unreviewed",
        ));
    }
    Ok(())
}

fn validate_coverage(repository_root: &Path, claims: &CoverageClaims) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/coverage/contract.json"),
        MAXIMUM_MANIFEST_BYTES,
        "coverage",
    )?;
    validate_coverage_contract_bytes(&bytes)
        .map_err(|error| ReleaseError::new("coverage", error.to_string()))?;
    if claims.contract_sha256 != sha256(&bytes)
        || claims.parity_authority
        || claims.missing_subsystems != 0
    {
        return Err(ReleaseError::new(
            "coverage",
            "coverage evidence is incomplete or promoted into parity",
        ));
    }
    Ok(())
}

fn validate_performance(
    repository_root: &Path,
    claims: &PerformanceClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/performance/manifest.toml"),
        MAXIMUM_MANIFEST_BYTES,
        "performance",
    )?;
    let manifest: toml::Value = toml::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("performance", error.to_string()))?;
    let policy_sha256 = manifest.get("policy_sha256").and_then(toml::Value::as_str);
    let reviewed_count = manifest
        .get("reviewed_reports")
        .and_then(toml::Value::as_array)
        .map(Vec::len);
    if policy_sha256 != Some(claims.policy_sha256.as_str())
        || reviewed_count != Some(claims.reviewed_report_count)
        || claims.timing_authority != "unprofiled_wall_clock"
        || claims.claim_scope != "workload_only"
        || claims.claim_status != "no_generalized_performance_claim"
        || claims.profile_authority
    {
        return Err(ReleaseError::new(
            "performance",
            "performance evidence overclaims or differs from the reviewed policy",
        ));
    }
    Ok(())
}

fn validate_corpus_closure(
    repository_root: &Path,
    claims: &CorpusClosureClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/upstream-corpus.json"),
        MAXIMUM_ARTIFACT_BYTES,
        "corpus-closure",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("corpus-closure", error.to_string()))?;
    let items = value
        .get("items")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("corpus-closure", "corpus items are absent"))?;
    let unresolved = items
        .iter()
        .filter(|item| item.get("review").is_none() || item.get("evidence").is_none())
        .count();
    let nonterminal = items
        .iter()
        .filter(|item| {
            item.get("disposition")
                .is_none_or(serde_json::Value::is_null)
        })
        .count();
    if claims.authority_sha256 != sha256(&bytes)
        || claims.item_count != items.len()
        || claims.unresolved_count != 0
        || claims.nonterminal_count != 0
        || unresolved != 0
        || nonterminal != 0
    {
        return Err(ReleaseError::new(
            "corpus-closure",
            "semantic corpus contains unresolved or nonterminal items",
        ));
    }
    Ok(())
}

fn validate_compatibility_closure(
    repository_root: &Path,
    claims: &CompatibilityClosureClaims,
) -> Result<(), ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/compatibility.json"),
        MAXIMUM_ARTIFACT_BYTES,
        "compatibility-closure",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("compatibility-closure", error.to_string()))?;
    let entries = value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("compatibility-closure", "entries are absent"))?;
    let dispositions = value
        .get("release_dispositions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ReleaseError::new("compatibility-closure", "dispositions are absent"))?;
    let entry_ids = ids(entries)?;
    let disposition_ids = ids(dispositions)?;
    if claims.authority_sha256 != sha256(&bytes)
        || entry_ids != disposition_ids
        || claims.gap_count != 0
        || claims.unexplained_count != 0
        || claims.mixed_commit_count != 0
        || claims.coverage_promoted_to_parity
        || claims.platform_promoted_to_parity
    {
        return Err(ReleaseError::new(
            "compatibility-closure",
            "compatibility release closure contains gaps or invalid authority promotion",
        ));
    }
    Ok(())
}

fn ids(values: &[serde_json::Value]) -> Result<BTreeSet<&str>, ReleaseError> {
    values
        .iter()
        .map(|value| {
            value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ReleaseError::new("compatibility-closure", "identity is absent"))
        })
        .collect()
}

fn validate_repository_authorities(repository_root: &Path) -> Result<(), ReleaseError> {
    require_repository_files(
        repository_root,
        &[
            "reference/platform/support.json",
            "reference/performance/manifest.toml",
            "reference/coverage/contract.json",
            "reference/regressions/manifest.toml",
            "reference/upstream-corpus.json",
            "reference/compatibility.json",
        ],
        "authority",
    )
}

fn platform_support(repository_root: &Path) -> Result<serde_json::Value, ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/platform/support.json"),
        MAXIMUM_MANIFEST_BYTES,
        "platform",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("platform", error.to_string()))?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
        || value
            .get("evidence_tier")
            .and_then(serde_json::Value::as_str)
            != Some("d2_supported")
    {
        return Err(ReleaseError::new(
            "platform",
            "tracked platform authority is invalid",
        ));
    }
    Ok(value)
}

fn require_repository_files(
    repository_root: &Path,
    paths: &[&str],
    category: &'static str,
) -> Result<(), ReleaseError> {
    for relative in paths {
        let _bytes = read_confined_regular(
            repository_root,
            Path::new(relative),
            MAXIMUM_ARTIFACT_BYTES,
            category,
        )?;
    }
    Ok(())
}

fn read_input_manifest(
    repository_root: &Path,
    manifest_path: &Path,
) -> Result<Vec<u8>, ReleaseError> {
    let path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repository_root.join(manifest_path)
    };
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|error| ReleaseError::new("manifest-path", error.to_string()))?;
    reject_symlink_components(&path, "manifest-path")?;
    let canonical = fs::canonicalize(&path)
        .map_err(|error| ReleaseError::new("manifest-path", error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ReleaseError::new(
            "manifest-path",
            "manifest escaped the repository",
        ));
    }
    read_regular(&canonical, MAXIMUM_MANIFEST_BYTES, "manifest-path")
}

fn read_confined_regular(
    repository_root: &Path,
    relative: &Path,
    maximum_bytes: u64,
    category: &'static str,
) -> Result<Vec<u8>, ReleaseError> {
    let relative = normalized_relative_path(relative, category)?;
    let path = repository_root.join(relative);
    reject_symlink_components(&path, category)?;
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    let canonical =
        fs::canonicalize(&path).map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ReleaseError::new(category, "path escaped the repository"));
    }
    read_regular(&canonical, maximum_bytes, category)
}

fn read_regular(
    path: &Path,
    maximum_bytes: u64,
    category: &'static str,
) -> Result<Vec<u8>, ReleaseError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(ReleaseError::new(
            category,
            "input must be a bounded ordinary file",
        ));
    }
    fs::read(path).map_err(|error| ReleaseError::new(category, error.to_string()))
}

fn reject_symlink_components(path: &Path, category: &'static str) -> Result<(), ReleaseError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            return Err(ReleaseError::new(category, "path contains a symbolic link"));
        }
    }
    Ok(())
}

fn normalized_relative<'a>(
    value: &'a str,
    category: &'static str,
) -> Result<&'a Path, ReleaseError> {
    normalized_relative_path(Path::new(value), category)
}

fn normalized_relative_path<'a>(
    path: &'a Path,
    category: &'static str,
) -> Result<&'a Path, ReleaseError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ReleaseError::new(
            category,
            "path must be normalized and repository-relative",
        ));
    }
    Ok(path)
}

fn require_sha256(value: &str, category: &'static str) -> Result<(), ReleaseError> {
    if !is_sha256(value) {
        return Err(ReleaseError::new(category, "SHA-256 identity is invalid"));
    }
    Ok(())
}

fn is_run_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && !value.starts_with('0')
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

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

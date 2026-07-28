//! Pure, bounded, fail-closed release evidence validation.

#[path = "validation/claims.rs"]
mod claims;
#[path = "validation/repository.rs"]
mod repository;

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
use claims::{
    CompatibilityClosureClaims, ConditionalPlatformClaims, CorpusClosureClaims, CoverageClaims,
    DifferentialClaims, DocsClaims, FindingsClaims, FuzzClaims, NoticeClaims, PackageClaims,
    PackageJoinClaims, PerformanceClaims, PlatformClaims, RegressionClaims, RustSafetyClaims,
    parse_claims, validate_compatibility_closure, validate_conditional_platform,
    validate_corpus_closure, validate_coverage, validate_durable_platform, validate_package_claims,
    validate_performance, validate_regressions, validate_safety_authorities,
};
use repository::{
    ids, is_full_sha, is_run_id, is_sha256, normalized_relative, platform_support,
    read_confined_regular, read_input_manifest, require_repository_files, require_sha256, sha256,
    validate_repository_authorities,
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

//! Strict immutable capability fixture loading.

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use super::{CapabilityError, hex_sha256};

const MAXIMUM_MANIFEST_BYTES: u64 = 128 * 1024;
const MAXIMUM_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024;
const MAXIMUM_REFERENCES: usize = 32;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    schema_version: u32,
    profile: String,
    upstream_revision: String,
    catalog: RawArtifact,
    mapping: RawArtifact,
    payloads: Vec<RawPayload>,
    inherited_proofs: Vec<RawProof>,
    cases: Vec<RawCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArtifact {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPayload {
    case_id: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProof {
    proof_id: String,
    phase: u32,
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCase {
    case_id: String,
    families: Vec<String>,
    payload_path: String,
    payload_sha256: String,
    inherited_proof_ids: Vec<String>,
    eligibility: RawEligibility,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEligibility {
    regression: bool,
    benchmark: bool,
    visual: bool,
}

/// Validated immutable input consumed by the renderer gate.
#[derive(Debug)]
pub(super) struct FixtureSnapshot {
    pub(super) sha256: String,
    pub(super) profile: String,
    pub(super) upstream_revision: String,
    pub(super) case_ids: Vec<String>,
    pub(super) families: Vec<String>,
    pub(super) verified_artifacts: usize,
}

pub(super) fn load_fixture_snapshot(
    repository: &Path,
    requested: &Path,
) -> Result<FixtureSnapshot, CapabilityError> {
    let fixture_path = resolve_input(repository, requested)?;
    let bytes = read_bounded_regular(&fixture_path, MAXIMUM_MANIFEST_BYTES)?;
    let raw: RawManifest =
        serde_json::from_slice(&bytes).map_err(|_| CapabilityError::InvalidFixture)?;
    validate_manifest(repository, raw, hex_sha256(&bytes))
}

fn validate_manifest(
    repository: &Path,
    raw: RawManifest,
    sha256: String,
) -> Result<FixtureSnapshot, CapabilityError> {
    if raw.schema_version != 1
        || raw.profile != "phase11-v1"
        || !is_lower_hex(&raw.upstream_revision, 40)
        || raw.payloads.is_empty()
        || raw.payloads.len() > MAXIMUM_REFERENCES
        || raw.inherited_proofs.len() > MAXIMUM_REFERENCES
        || raw.cases.is_empty()
        || raw.cases.len() > MAXIMUM_REFERENCES
    {
        return Err(CapabilityError::InvalidFixture);
    }

    let mut verified_artifacts = 0_usize;
    verify_artifact(repository, &raw.catalog.path, &raw.catalog.sha256)?;
    verified_artifacts += 1;
    verify_artifact(repository, &raw.mapping.path, &raw.mapping.sha256)?;
    verified_artifacts += 1;

    let mut payload_ids = HashSet::new();
    let mut payload_paths = HashSet::new();
    for payload in &raw.payloads {
        if !is_safe_id(&payload.case_id)
            || !payload_ids.insert(payload.case_id.as_str())
            || !payload_paths.insert(payload.path.as_str())
        {
            return Err(CapabilityError::InvalidFixture);
        }
        verify_artifact(repository, &payload.path, &payload.sha256)?;
        verified_artifacts += 1;
    }

    let mut proof_ids = HashSet::new();
    for proof in &raw.inherited_proofs {
        if !is_safe_id(&proof.proof_id)
            || proof.phase == 0
            || !proof_ids.insert(proof.proof_id.as_str())
        {
            return Err(CapabilityError::InvalidFixture);
        }
        verify_artifact(repository, &proof.path, &proof.sha256)?;
        verified_artifacts += 1;
    }

    let mut case_ids = Vec::with_capacity(raw.cases.len());
    let mut families = Vec::new();
    let mut seen_cases = HashSet::new();
    for case in &raw.cases {
        let Some(payload) = raw
            .payloads
            .iter()
            .find(|payload| payload.case_id == case.case_id)
        else {
            return Err(CapabilityError::InvalidFixture);
        };
        if !seen_cases.insert(case.case_id.as_str())
            || case.payload_path != payload.path
            || case.payload_sha256 != payload.sha256
            || !case.eligibility.regression
            || !case.eligibility.benchmark
            || !case.eligibility.visual
            || case.families.is_empty()
            || case
                .inherited_proof_ids
                .iter()
                .any(|proof_id| !proof_ids.contains(proof_id.as_str()))
        {
            return Err(CapabilityError::InvalidFixture);
        }
        case_ids.push(case.case_id.clone());
        for family in &case.families {
            if !is_safe_id(family) {
                return Err(CapabilityError::InvalidFixture);
            }
            if !families.contains(family) {
                families.push(family.clone());
            }
        }
    }
    if seen_cases != payload_ids {
        return Err(CapabilityError::InvalidFixture);
    }
    families.sort();
    Ok(FixtureSnapshot {
        sha256,
        profile: raw.profile,
        upstream_revision: raw.upstream_revision,
        case_ids,
        families,
        verified_artifacts,
    })
}

fn verify_artifact(
    repository: &Path,
    relative: &str,
    expected_sha256: &str,
) -> Result<(), CapabilityError> {
    if !is_lower_hex(expected_sha256, 64) {
        return Err(CapabilityError::InvalidFixture);
    }
    let path = resolve_relative(repository, Path::new(relative))?;
    let bytes = read_bounded_regular(&path, MAXIMUM_ARTIFACT_BYTES)?;
    if hex_sha256(&bytes) != expected_sha256 {
        return Err(CapabilityError::InvalidFixture);
    }
    Ok(())
}

fn resolve_input(repository: &Path, requested: &Path) -> Result<PathBuf, CapabilityError> {
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        repository.join(requested)
    };
    let metadata = fs::symlink_metadata(&candidate).map_err(|_| CapabilityError::InvalidFixture)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CapabilityError::InvalidFixture);
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|_| CapabilityError::InvalidFixture)?;
    if !canonical.starts_with(repository) {
        return Err(CapabilityError::InvalidFixture);
    }
    Ok(canonical)
}

fn resolve_relative(repository: &Path, relative: &Path) -> Result<PathBuf, CapabilityError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CapabilityError::InvalidFixture);
    }
    resolve_input(repository, &repository.join(relative))
}

fn read_bounded_regular(path: &Path, limit: u64) -> Result<Vec<u8>, CapabilityError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| CapabilityError::InvalidFixture)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit {
        return Err(CapabilityError::InvalidFixture);
    }
    let bytes = fs::read(path).map_err(|_| CapabilityError::InvalidFixture)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(CapabilityError::InvalidFixture);
    }
    Ok(bytes)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

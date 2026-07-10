//! Strict versioned provenance for reviewed semantic traces and regressions.

use std::{collections::BTreeSet, env, ffi::OsStr, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    ConfinedPaths, ProvenanceError, SourceMap, is_lower_hex, read_json, read_toml,
    require_nonempty, require_revision, require_revision_format, run_git, sha256,
};

mod trace;

use trace::validate_trace;

const MANIFEST_SCHEMA_VERSION: u64 = 2;
const RECORD_SCHEMA_VERSION: u64 = 2;
const RECORD_FIELDS: [&str; 27] = [
    "artifact_kind",
    "path",
    "sha256",
    "generator_revision",
    "request_sha256",
    "scenario_content_sha256",
    "scenario_sha256",
    "protocol_version",
    "scenario_schema_version",
    "trace_schema_version",
    "tolerance_profile_version",
    "tolerance_profile_sha256",
    "oracle_revision",
    "adapter_revision",
    "adapter_content_sha256",
    "build_identity_sha256",
    "preset",
    "compiler",
    "target",
    "flags",
    "source",
    "trace_payload_sha256",
    "failure_signature",
    "notice_refs",
    "reviewer",
    "reviewed_at",
    "review_status",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactManifest {
    schema_version: u64,
    record_schema_version: u64,
    oracle_revision: String,
    record_fields: Vec<String>,
    artifacts: Vec<RawArtifactRecord>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArtifactKind {
    Trace,
    Regression,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ReviewStatus {
    Pending,
    Reviewed,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ArtifactSource {
    Named {
        name: String,
    },
    Seeded {
        generator_id: String,
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FailureSignature {
    checkpoint_id: String,
    phase: String,
    semantic_path: serde_json::Value,
    kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArtifactRecord {
    artifact_kind: ArtifactKind,
    path: String,
    sha256: String,
    generator_revision: String,
    request_sha256: String,
    scenario_content_sha256: String,
    scenario_sha256: String,
    protocol_version: u32,
    scenario_schema_version: u32,
    trace_schema_version: u32,
    tolerance_profile_version: u32,
    tolerance_profile_sha256: String,
    oracle_revision: String,
    adapter_revision: String,
    adapter_content_sha256: String,
    build_identity_sha256: String,
    preset: String,
    compiler: String,
    target: String,
    flags: Vec<String>,
    source: ArtifactSource,
    trace_payload_sha256: Option<String>,
    failure_signature: Option<FailureSignature>,
    notice_refs: Vec<String>,
    reviewer: String,
    reviewed_at: String,
    review_status: ReviewStatus,
}

enum ArtifactRecord<'a> {
    Trace {
        common: &'a RawArtifactRecord,
        trace_payload_sha256: &'a str,
    },
    Regression {
        common: &'a RawArtifactRecord,
        failure_signature: &'a FailureSignature,
    },
}

impl<'a> ArtifactRecord<'a> {
    fn parse(raw: &'a RawArtifactRecord) -> Result<Self, ProvenanceError> {
        match (
            raw.artifact_kind,
            raw.trace_payload_sha256.as_deref(),
            raw.failure_signature.as_ref(),
        ) {
            (ArtifactKind::Trace, Some(trace_payload_sha256), None) => Ok(Self::Trace {
                common: raw,
                trace_payload_sha256,
            }),
            (ArtifactKind::Regression, None, Some(failure_signature)) => Ok(Self::Regression {
                common: raw,
                failure_signature,
            }),
            _ => Err(ProvenanceError::new(
                "schema",
                format!(
                    "artifact `{}` does not match its strict trace/regression variant",
                    raw.path
                ),
            )),
        }
    }

    const fn common(&self) -> &'a RawArtifactRecord {
        match self {
            Self::Trace { common, .. } | Self::Regression { common, .. } => common,
        }
    }
}

pub(super) fn validate_manifest(
    repository_root: &Path,
    confined_paths: &ConfinedPaths,
    source_map: &SourceMap,
    oracle_revision: &str,
) -> Result<usize, ProvenanceError> {
    let manifest: ArtifactManifest = read_toml(
        &repository_root.join("reference/artifacts/manifest.toml"),
        "artifact manifest",
    )?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        || manifest.record_schema_version != RECORD_SCHEMA_VERSION
        || manifest.record_fields != RECORD_FIELDS
    {
        return Err(ProvenanceError::new(
            "schema",
            "artifact manifest must use the exact schema version 2 field contract",
        ));
    }
    require_revision(
        "artifact manifest",
        oracle_revision,
        &manifest.oracle_revision,
    )?;
    let mapped_paths = source_map
        .mapping
        .iter()
        .map(|mapping| mapping.local_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut artifact_paths = BTreeSet::new();
    for raw in &manifest.artifacts {
        let artifact = ArtifactRecord::parse(raw)?;
        let common = artifact.common();
        if !artifact_paths.insert(common.path.as_str()) {
            return Err(ProvenanceError::new(
                "schema",
                format!("duplicate artifact path `{}`", common.path),
            ));
        }
        validate_artifact(repository_root, confined_paths, &artifact, oracle_revision)?;
        if !mapped_paths.contains(common.path.as_str()) {
            return Err(ProvenanceError::new(
                "notice",
                format!("artifact `{}` has no source-map record", common.path),
            ));
        }
    }
    Ok(manifest.artifacts.len())
}

fn validate_artifact(
    repository_root: &Path,
    confined_paths: &ConfinedPaths,
    artifact: &ArtifactRecord<'_>,
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    let common = artifact.common();
    let artifact_path = confined_paths.file(&common.path, "artifact path")?;
    validate_artifact_kind_path(common.artifact_kind, &common.path)?;
    validate_sha256("artifact content", &common.sha256)?;
    let actual = sha256(&artifact_path)?;
    if actual != common.sha256 {
        return Err(ProvenanceError::new(
            "hash",
            format!(
                "artifact `{}` SHA-256 mismatch: expected `{}`, actual `{actual}`",
                common.path, common.sha256
            ),
        ));
    }
    require_revision("artifact", oracle_revision, &common.oracle_revision)?;
    require_revision_format("generator_revision", &common.generator_revision)?;
    validate_generator_revision(repository_root, &common.generator_revision)?;
    validate_review(common)?;
    validate_notices(confined_paths, common)?;
    validate_common_identity(common)?;
    let scenario = validate_scenario(confined_paths, common)?;
    validate_tolerance(repository_root, common)?;
    match artifact {
        ArtifactRecord::Trace {
            trace_payload_sha256,
            ..
        } => validate_trace(&artifact_path, common, trace_payload_sha256, &scenario),
        ArtifactRecord::Regression {
            failure_signature, ..
        } => validate_regression(common, failure_signature),
    }
}

fn validate_artifact_kind_path(kind: ArtifactKind, path: &str) -> Result<(), ProvenanceError> {
    let valid = match kind {
        ArtifactKind::Trace => {
            path.starts_with("reference/artifacts/traces/")
                && Path::new(path).extension() == Some(OsStr::new("jsonl"))
        }
        ArtifactKind::Regression => {
            path.starts_with("scenarios/regressions/")
                && Path::new(path).extension() == Some(OsStr::new("json"))
        }
    };
    if valid {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "path",
        format!("artifact path `{path}` does not match its artifact kind"),
    ))
}

fn validate_common_identity(common: &RawArtifactRecord) -> Result<(), ProvenanceError> {
    for (field, value) in [
        ("adapter_revision", common.adapter_revision.as_str()),
        ("preset", common.preset.as_str()),
        ("compiler", common.compiler.as_str()),
        ("target", common.target.as_str()),
    ] {
        require_nonempty(field, value)?;
    }
    for (field, value) in [
        ("request_sha256", common.request_sha256.as_str()),
        (
            "scenario_content_sha256",
            common.scenario_content_sha256.as_str(),
        ),
        ("scenario_sha256", common.scenario_sha256.as_str()),
        (
            "tolerance_profile_sha256",
            common.tolerance_profile_sha256.as_str(),
        ),
        (
            "adapter_content_sha256",
            common.adapter_content_sha256.as_str(),
        ),
        (
            "build_identity_sha256",
            common.build_identity_sha256.as_str(),
        ),
    ] {
        validate_sha256(field, value)?;
    }
    if common.flags.is_empty() || common.flags.iter().any(|flag| flag.trim().is_empty()) {
        return Err(ProvenanceError::new(
            "identity",
            "artifact flags must contain complete nonempty compile and link identities",
        ));
    }
    if [
        common.protocol_version,
        common.scenario_schema_version,
        common.trace_schema_version,
        common.tolerance_profile_version,
    ] != [1, 1, 1, 1]
    {
        return Err(ProvenanceError::new(
            "schema",
            "Phase-2 artifact versions must all be exactly 1",
        ));
    }
    Ok(())
}

fn validate_review(common: &RawArtifactRecord) -> Result<(), ProvenanceError> {
    if common.review_status != ReviewStatus::Reviewed {
        return Err(ProvenanceError::new(
            "review",
            format!("artifact `{}` is not reviewed", common.path),
        ));
    }
    require_nonempty("reviewer", &common.reviewer)?;
    require_nonempty("reviewed_at", &common.reviewed_at)?;
    if !common.reviewed_at.contains('T') || !common.reviewed_at.ends_with('Z') {
        return Err(ProvenanceError::new(
            "review",
            "artifact reviewed_at must be explicit UTC RFC3339 form",
        ));
    }
    Ok(())
}

fn validate_notices(
    confined_paths: &ConfinedPaths,
    common: &RawArtifactRecord,
) -> Result<(), ProvenanceError> {
    if common.notice_refs.is_empty() {
        return Err(ProvenanceError::new(
            "notice",
            format!("artifact `{}` has no notice references", common.path),
        ));
    }
    for notice_ref in &common.notice_refs {
        let path_part = notice_ref
            .split_once('#')
            .map_or(notice_ref.as_str(), |pair| pair.0);
        let _notice_path = confined_paths.file(path_part, "artifact notice reference")?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScenarioDocument {
    scenario_id: String,
    source: ArtifactSource,
    gravity_x_bits: u32,
    gravity_y_bits: u32,
    entities: Vec<serde_json::Value>,
    commands: Vec<ScenarioCommand>,
    checkpoints: Vec<ScenarioCheckpoint>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScenarioCommand {
    kind: String,
    command_id: String,
    timestep_bits: u32,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScenarioCheckpoint {
    checkpoint_id: String,
    after_command_id: String,
    phase: String,
    observables: Vec<String>,
}

fn validate_scenario(
    confined_paths: &ConfinedPaths,
    common: &RawArtifactRecord,
) -> Result<ScenarioDocument, ProvenanceError> {
    let scenario_path = match &common.source {
        ArtifactSource::Named { name } => {
            validate_identifier("source name", name)?;
            format!("scenarios/phase-02/{name}.json")
        }
        ArtifactSource::Seeded { generator_id, .. } => {
            validate_identifier("generator_id", generator_id)?;
            common.path.clone()
        }
    };
    let scenario_file = confined_paths.file(&scenario_path, "scenario source")?;
    validate_sha256("scenario_content_sha256", &common.scenario_content_sha256)?;
    let scenario: ScenarioDocument = read_json(&scenario_file, "artifact scenario")?;
    if scenario.scenario_id != source_name(&common.source)
        || scenario.source != common.source
        || !scenario.entities.is_empty()
    {
        return Err(ProvenanceError::new(
            "scenario",
            "artifact scenario identity, source, or Phase-2 entity contract is invalid",
        ));
    }
    let canonical = serde_json::to_vec(&scenario).map_err(|error| {
        ProvenanceError::new("scenario", format!("failed to encode scenario: {error}"))
    })?;
    let canonical_sha256 = digest_bytes(&canonical);
    if canonical_sha256 != common.scenario_sha256
        || canonical_sha256 != common.scenario_content_sha256
    {
        return Err(ProvenanceError::new(
            "hash",
            "artifact canonical scenario content/identity SHA-256 mismatch",
        ));
    }
    if let ArtifactSource::Named { name } = &common.source {
        let request_path = format!("protocol/fixtures/accepted/{name}-request.jsonl");
        let request_file = confined_paths.file(&request_path, "scenario request")?;
        validate_file_hash(&request_file, &common.request_sha256, "scenario request")?;
    } else {
        validate_sha256("request_sha256", &common.request_sha256)?;
    }
    Ok(scenario)
}

fn source_name(source: &ArtifactSource) -> &str {
    match source {
        ArtifactSource::Named { name } => name,
        ArtifactSource::Seeded { generator_id, .. } => generator_id,
    }
}

fn validate_tolerance(
    repository_root: &Path,
    common: &RawArtifactRecord,
) -> Result<(), ProvenanceError> {
    #[derive(Deserialize)]
    struct ToleranceHeader {
        profile_id: String,
        version: u32,
        profile_sha256: String,
    }
    let header: ToleranceHeader = read_toml(
        &repository_root.join("protocol/tolerances/phase2-v1.toml"),
        "tolerance profile",
    )?;
    if header.profile_id != "phase2-v1"
        || header.version != common.tolerance_profile_version
        || header.profile_sha256 != common.tolerance_profile_sha256
    {
        return Err(ProvenanceError::new(
            "policy",
            "artifact tolerance profile identity mismatch",
        ));
    }
    Ok(())
}

fn validate_regression(
    common: &RawArtifactRecord,
    signature: &FailureSignature,
) -> Result<(), ProvenanceError> {
    let semantic_path = signature.semantic_path.to_string();
    for (field, value) in [
        ("checkpoint_id", signature.checkpoint_id.as_str()),
        ("phase", signature.phase.as_str()),
        ("semantic_path", semantic_path.as_str()),
        ("kind", signature.kind.as_str()),
    ] {
        require_nonempty(field, value)?;
    }
    if common.sha256 != common.scenario_content_sha256 {
        return Err(ProvenanceError::new(
            "hash",
            "regression content and scenario content SHA-256 must agree",
        ));
    }
    Ok(())
}

fn validate_file_hash(path: &Path, expected: &str, label: &str) -> Result<(), ProvenanceError> {
    validate_sha256(label, expected)?;
    let actual = sha256(path)?;
    if actual == expected {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "hash",
        format!("{label} SHA-256 mismatch: expected `{expected}`, actual `{actual}`"),
    ))
}

fn validate_sha256(label: &str, value: &str) -> Result<(), ProvenanceError> {
    if value.len() == 64 && is_lower_hex(value) {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "hash",
        format!("{label} must be a lowercase 64-hex SHA-256"),
    ))
}

fn validate_identifier(label: &str, value: &str) -> Result<(), ProvenanceError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            (index > 0 && matches!(byte, b'.' | b'_' | b'-'))
                || byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
        });
    if valid {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "scenario",
        format!("{label} `{value}` is not a normalized identifier"),
    ))
}

fn validate_generator_revision(
    repository_root: &Path,
    revision: &str,
) -> Result<(), ProvenanceError> {
    let git = env::var_os("LIQUIDFUN_XTASK_GIT").unwrap_or_else(|| "git".into());
    let object = format!("{revision}^{{commit}}");
    let _output = run_git(
        &git,
        [
            OsStr::new("-C"),
            repository_root.as_os_str(),
            OsStr::new("cat-file"),
            OsStr::new("-e"),
            OsStr::new(&object),
        ],
        "verify artifact generator revision",
    )
    .map_err(|error| ProvenanceError::new("generator", error.to_string()))?;
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

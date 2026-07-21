use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
};

use liquidfun_differential::{
    PHASE10_EVIDENCE_SCHEMA_VERSION, Phase10EvidenceBinding, validate_phase10_evidence_contract,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Phase10EvidenceError,
    authority::ExactRun,
    paths::{
        MAXIMUM_LOG_BYTES, canonical_sha256, checked_payload_path, read_json, read_regular_file,
        regular_files, require_digest, resolve_descendant, resolve_target_path, sha256,
    },
};

pub(super) const MANIFEST_FILE: &str = "phase10-manifest.json";
pub(super) const IDENTITY_FILE: &str = "identity.json";
const TRACE_FILE: &str = "phase10-trace.log";
const FIXTURE_MANIFEST: &str =
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PROTOCOL_VERSION: &str = "rigid-world-phase10-v1";
const GENERATOR_VERSION: &str = "phase10-corpus-v1";
const REQUIRED_LOGS: [&str; 4] = [
    TRACE_FILE,
    "provenance.log",
    "inventory.log",
    "read-only.log",
];
const PROOF_ROLES: [&str; 10] = [
    "native",
    "oracle",
    "comparison",
    "replay-native",
    "replay-oracle",
    "debug-oracle",
    "release-oracle",
    "minimized",
    "copied",
    "inherited",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EvidenceKind {
    Canonical,
    Sanitizer,
}

impl EvidenceKind {
    pub(super) fn parse(value: &str) -> Result<Self, Phase10EvidenceError> {
        match value {
            "canonical" => Ok(Self::Canonical),
            "sanitizer" => Ok(Self::Sanitizer),
            _ => Err(Phase10EvidenceError::new(
                "usage",
                format!("unsupported evidence kind `{value}`"),
            )),
        }
    }

    pub(super) const fn local_job(self) -> &'static str {
        match self {
            Self::Canonical => "phase10-canonical-local",
            Self::Sanitizer => "phase10-sanitizer-local",
        }
    }
}

#[derive(Debug)]
pub(super) struct ValidatedDirectory {
    pub(super) semantic_manifest_sha256: String,
    pub(super) identity: EvidenceIdentity,
    pub(super) expected_files: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceManifest {
    schema_version: u32,
    profile: String,
    upstream_revision: String,
    protocol_version: String,
    generator_version: String,
    fixture_manifest_sha256: String,
    semantic_manifest_sha256: String,
    bindings: Vec<Phase10EvidenceBinding>,
    cases: Vec<EvidenceCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCase {
    case_id: String,
    action_count: usize,
    checkpoint_count: usize,
    observation_count: usize,
    proofs: BTreeMap<String, FileRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofEnvelope {
    schema_version: u32,
    case_id: String,
    role: String,
    outcome: String,
    payload_sha256: String,
    payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceIdentity {
    schema_version: u32,
    pub(super) mode: String,
    pub(super) run_id: u64,
    pub(super) head_sha: String,
    pub(super) job_name: String,
    pub(super) artifact_id: u64,
    pub(super) artifact_name: String,
    pub(super) platform: String,
    pub(super) rust_version: String,
    pub(super) clang_version: String,
    pub(super) upstream_revision: String,
    pub(super) protocol_version: String,
    pub(super) generator_version: String,
    semantic_manifest_sha256: String,
    files: Vec<FileRef>,
}

#[derive(Serialize)]
struct SemanticManifest<'a> {
    bindings: &'a [Phase10EvidenceBinding],
    cases: &'a [EvidenceCase],
}

pub(super) fn validate_directory(
    repository_root: &Path,
    relative: &Path,
    kind: EvidenceKind,
    maybe_run: Option<&ExactRun>,
) -> Result<ValidatedDirectory, Phase10EvidenceError> {
    let root = resolve_target_path(repository_root, relative, "evidence root")?;
    let manifest = validate_content(repository_root, &root)?;
    let identity: EvidenceIdentity = read_json(&root.join(IDENTITY_FILE), "identity")?;
    validate_identity(&root, kind, &identity, maybe_run, &manifest)?;
    let expected_files = expected_files(&manifest);
    if regular_files(&root)? != expected_files {
        return Err(Phase10EvidenceError::new(
            "files",
            "evidence regular-file set differs from the closed manifest",
        ));
    }
    Ok(ValidatedDirectory {
        semantic_manifest_sha256: manifest.semantic_manifest_sha256,
        identity,
        expected_files,
    })
}

pub(super) fn validate_generated_directory(
    repository_root: &Path,
    relative: &Path,
    _kind: EvidenceKind,
) -> Result<(), Phase10EvidenceError> {
    let root = resolve_target_path(repository_root, relative, "evidence root")?;
    let manifest = validate_content(repository_root, &root)?;
    let mut expected = expected_files(&manifest);
    expected.remove(IDENTITY_FILE);
    if regular_files(&root)? != expected {
        return Err(Phase10EvidenceError::new(
            "files",
            "generated evidence file set is not identity-last and closed",
        ));
    }
    Ok(())
}

fn validate_content(
    repository_root: &Path,
    root: &Path,
) -> Result<EvidenceManifest, Phase10EvidenceError> {
    let manifest: EvidenceManifest = read_json(&root.join(MANIFEST_FILE), "manifest")?;
    if manifest.schema_version != PHASE10_EVIDENCE_SCHEMA_VERSION
        || manifest.profile != "phase10-v1"
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.protocol_version != PROTOCOL_VERSION
        || manifest.generator_version != GENERATOR_VERSION
        || manifest.cases.len() != 5
    {
        return Err(Phase10EvidenceError::new(
            "manifest",
            "Phase 10 schema, provenance, or exact five-case cardinality is invalid",
        ));
    }
    let fixture = repository_root.join(FIXTURE_MANIFEST);
    require_digest(
        "fixture manifest",
        &manifest.fixture_manifest_sha256,
        &sha256(&read_regular_file(
            &fixture,
            "fixture manifest",
            MAXIMUM_LOG_BYTES,
        )?),
    )?;
    require_digest(
        "semantic manifest",
        &manifest.semantic_manifest_sha256,
        &canonical_sha256(&SemanticManifest {
            bindings: &manifest.bindings,
            cases: &manifest.cases,
        })?,
    )?;
    validate_cases(root, &manifest)?;
    validate_bindings(&manifest)?;
    validate_logs(root)?;
    Ok(manifest)
}

fn validate_bindings(manifest: &EvidenceManifest) -> Result<(), Phase10EvidenceError> {
    let bounds = manifest
        .cases
        .iter()
        .map(|case| {
            liquidfun_test_protocol::ScenarioId::new(&case.case_id)
                .map(|id| {
                    (
                        id,
                        (
                            case.action_count,
                            case.checkpoint_count,
                            case.observation_count,
                        ),
                    )
                })
                .map_err(|error| Phase10EvidenceError::new("case", error.to_string()))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    validate_phase10_evidence_contract(&manifest.bindings, &bounds)
        .map_err(|error| Phase10EvidenceError::new("leaf-contract", error.to_string()))
}

fn validate_cases(root: &Path, manifest: &EvidenceManifest) -> Result<(), Phase10EvidenceError> {
    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        let proof_roles = case
            .proofs
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let required_roles = PROOF_ROLES.into_iter().collect::<BTreeSet<_>>();
        if !case_ids.insert(case.case_id.as_str())
            || case.action_count == 0
            || case.checkpoint_count == 0
            || case.observation_count == 0
            || proof_roles != required_roles
        {
            return Err(Phase10EvidenceError::new(
                "case",
                "case identity, bounds, or proof-role topology is invalid",
            ));
        }
        let mut payloads = BTreeMap::new();
        let mut paths = BTreeSet::new();
        for role in PROOF_ROLES {
            let reference = &case.proofs[role];
            let expected = format!("cases/{}/proofs/{role}.json", case.case_id);
            if reference.path != expected || !paths.insert(reference.path.as_str()) {
                return Err(Phase10EvidenceError::new(
                    "proof",
                    "proof paths are aliased or noncanonical",
                ));
            }
            let path = resolve_descendant(root, checked_payload_path(&reference.path)?, "proof")?;
            let bytes = read_regular_file(&path, "proof", MAXIMUM_LOG_BYTES)?;
            require_digest("proof", &reference.sha256, &sha256(&bytes))?;
            let proof: ProofEnvelope = serde_json::from_slice(&bytes)
                .map_err(|error| Phase10EvidenceError::new("proof", error.to_string()))?;
            validate_proof(case, role, &proof)?;
            payloads.insert(role, proof.payload);
        }
        require_equal(&payloads, "native", "replay-native")?;
        require_equal(&payloads, "oracle", "replay-oracle")?;
        require_equal(&payloads, "oracle", "debug-oracle")?;
        require_equal(&payloads, "oracle", "release-oracle")?;
        require_equal(&payloads, "minimized", "copied")?;
    }
    Ok(())
}

fn validate_proof(
    case: &EvidenceCase,
    role: &str,
    proof: &ProofEnvelope,
) -> Result<(), Phase10EvidenceError> {
    let expected_outcome = if matches!(role, "minimized" | "copied") {
        "deliberate-divergence"
    } else {
        "match"
    };
    if proof.schema_version != 1
        || proof.case_id != case.case_id
        || proof.role != role
        || proof.outcome != expected_outcome
    {
        return Err(Phase10EvidenceError::new(
            "proof",
            "proof identity, role, or passing outcome is invalid",
        ));
    }
    require_digest(
        "proof payload",
        &proof.payload_sha256,
        &canonical_sha256(&proof.payload)?,
    )
}

fn require_equal(
    payloads: &BTreeMap<&str, Value>,
    left: &str,
    right: &str,
) -> Result<(), Phase10EvidenceError> {
    if payloads[left] != payloads[right] {
        return Err(Phase10EvidenceError::new(
            "replay",
            format!("{left} and {right} semantic bytes differ"),
        ));
    }
    Ok(())
}

fn validate_logs(root: &Path) -> Result<(), Phase10EvidenceError> {
    for name in REQUIRED_LOGS {
        let bytes = read_regular_file(&root.join(name), "log", MAXIMUM_LOG_BYTES)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| Phase10EvidenceError::new("log", error.to_string()))?;
        if !text.contains("status: ok") || text.contains("FAILED") {
            return Err(Phase10EvidenceError::new(
                "log",
                format!("{name} lacks a passing marker or contains FAILED"),
            ));
        }
    }
    Ok(())
}

fn validate_identity(
    root: &Path,
    kind: EvidenceKind,
    identity: &EvidenceIdentity,
    maybe_run: Option<&ExactRun>,
    manifest: &EvidenceManifest,
) -> Result<(), Phase10EvidenceError> {
    if identity.schema_version != 1
        || identity.upstream_revision != UPSTREAM_REVISION
        || identity.protocol_version != PROTOCOL_VERSION
        || identity.generator_version != GENERATOR_VERSION
        || identity.semantic_manifest_sha256 != manifest.semantic_manifest_sha256
    {
        return Err(Phase10EvidenceError::new(
            "identity",
            "identity schema or semantic provenance differs from the manifest",
        ));
    }
    if maybe_run.is_none()
        && (identity.mode != "local"
            || identity.run_id != 0
            || identity.head_sha != "local"
            || identity.job_name != kind.local_job()
            || identity.artifact_id != 0
            || identity.artifact_name != kind.local_job())
    {
        return Err(Phase10EvidenceError::new(
            "identity",
            "local evidence carries promotable or substituted authority",
        ));
    }
    let expected = expected_files(manifest);
    let actual = identity
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    let expected_without_identity = expected
        .iter()
        .filter(|path| path.as_str() != IDENTITY_FILE)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual != expected_without_identity {
        return Err(Phase10EvidenceError::new(
            "identity",
            "identity file inventory is incomplete or contains extras",
        ));
    }
    for file in &identity.files {
        let path = resolve_descendant(root, checked_payload_path(&file.path)?, "identity")?;
        require_digest(
            "identity file",
            &file.sha256,
            &sha256(&read_regular_file(
                &path,
                "identity file",
                MAXIMUM_LOG_BYTES,
            )?),
        )?;
    }
    Ok(())
}

fn expected_files(manifest: &EvidenceManifest) -> BTreeSet<String> {
    let mut files = REQUIRED_LOGS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    files.insert(MANIFEST_FILE.to_owned());
    files.insert(IDENTITY_FILE.to_owned());
    for case in &manifest.cases {
        files.extend(case.proofs.values().map(|reference| reference.path.clone()));
    }
    files
}

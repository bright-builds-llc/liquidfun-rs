//! One semantic evaluator shared by local and exact-reference authority modes.

mod corpus;
mod model;

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use corpus::{evaluate_cases, payload_filename, validate_manifest, validate_mapping_authority};
use model::{CaseSemantic, CorpusManifest, ProofRecord, ScenarioMappings};

use super::{
    Phase11EvidenceError,
    paths::{
        MAX_JSON_BYTES, canonical_sha256, read_json, read_regular, regular_files, require_sha256,
        resolve_input,
    },
};

pub(super) const CORPUS_DIRECTORY: &str = "crates/liquidfun-differential/tests/fixtures/catalog";
pub(super) const MANIFEST_FILE: &str = "phase11-v1.json";
pub(super) const IDENTITY_FILE: &str = "identity.json";
pub(super) const PROTOCOL_VERSION: &str = "catalog-phase11-v1";
pub(super) const GENERATOR_VERSION: &str = "phase11-evidence-v1";
pub(super) const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const MAPPINGS: &str = "reference/artifacts/phase11/scenario-mappings.json";
const PHASE6_SHA256: &str = "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_SHA256: &str = "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const CASE_IDS: [&str; 3] = [
    "rigid-joint-rope",
    "particle-groups",
    "queries-callbacks-mutations",
];
const ROLES: [&str; 4] = ["debug", "release", "replay", "sanitizer"];
const FORBIDDEN_PARTS: [&str; 11] = [
    "pixel",
    "frame",
    "framerate",
    "frame_rate",
    "duration",
    "wall_clock",
    "pass_id",
    "private",
    "render_order",
    "renderer_order",
    "dense_index",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EvidenceKind {
    Canonical,
    Sanitizer,
}

impl EvidenceKind {
    pub(super) fn parse(value: &str) -> Result<Self, Phase11EvidenceError> {
        match value {
            "canonical" => Ok(Self::Canonical),
            "sanitizer" => Ok(Self::Sanitizer),
            _ => Err(Phase11EvidenceError::new(
                "usage",
                format!("unsupported evidence kind `{value}`"),
            )),
        }
    }

    pub(super) const fn local_name(self) -> &'static str {
        match self {
            Self::Canonical => "phase11-canonical-local",
            Self::Sanitizer => "phase11-sanitizer-local",
        }
    }
}

#[derive(Debug)]
pub(super) struct AcceptedContent {
    pub(super) root: PathBuf,
    pub(super) semantic_sha256: String,
    pub(super) expected_files: BTreeSet<String>,
    pub(super) source_only: bool,
}

pub(super) fn evaluate_directory(
    repository_root: &Path,
    relative: &Path,
) -> Result<AcceptedContent, Phase11EvidenceError> {
    let root = resolve_input(repository_root, relative, "evidence root")?;
    let source_root = resolve_input(repository_root, Path::new(CORPUS_DIRECTORY), "corpus")?;
    let source_only = root == source_root;
    let accepted = evaluate_content(repository_root, root, source_only)?;
    if regular_files(&accepted.root)? != accepted.expected_files {
        return Err(Phase11EvidenceError::new(
            "files",
            "evidence file set differs from the closed Phase 11 topology",
        ));
    }
    Ok(accepted)
}

pub(super) fn evaluate_generated_before_identity(
    repository_root: &Path,
    relative: &Path,
) -> Result<AcceptedContent, Phase11EvidenceError> {
    let root = resolve_input(repository_root, relative, "evidence root")?;
    let accepted = evaluate_content(repository_root, root, false)?;
    let mut expected = accepted.expected_files.clone();
    expected.remove(IDENTITY_FILE);
    if regular_files(&accepted.root)? != expected {
        return Err(Phase11EvidenceError::new(
            "files",
            "generated content must be complete, closed, and identity-last",
        ));
    }
    Ok(accepted)
}

pub(super) fn render_source_records(
    repository_root: &Path,
    role: &str,
) -> Result<Vec<String>, Phase11EvidenceError> {
    if !ROLES.contains(&role) {
        return Err(Phase11EvidenceError::new(
            "usage",
            format!("unsupported Phase 11 proof role `{role}`"),
        ));
    }
    let root = resolve_input(repository_root, Path::new(CORPUS_DIRECTORY), "corpus")?;
    let (manifest, mappings) = load_authorities(repository_root, &root)?;
    evaluate_cases(repository_root, &root, &manifest, &mappings)?
        .iter()
        .map(|semantic| render_record(role, semantic))
        .collect()
}

fn evaluate_content(
    repository_root: &Path,
    root: PathBuf,
    source_only: bool,
) -> Result<AcceptedContent, Phase11EvidenceError> {
    let (manifest, mappings) = load_authorities(repository_root, &root)?;
    let semantics = evaluate_cases(repository_root, &root, &manifest, &mappings)?;
    if !source_only {
        validate_records(&root, &semantics)?;
    }
    Ok(AcceptedContent {
        root,
        semantic_sha256: canonical_sha256(&semantics)?,
        expected_files: expected_files(&manifest, source_only),
        source_only,
    })
}

fn load_authorities(
    repository_root: &Path,
    root: &Path,
) -> Result<(CorpusManifest, ScenarioMappings), Phase11EvidenceError> {
    let manifest: CorpusManifest = read_json(&root.join(MANIFEST_FILE), "corpus manifest")?;
    validate_manifest(repository_root, root, &manifest)?;
    let mappings: ScenarioMappings =
        read_json(&repository_root.join(MAPPINGS), "scenario mappings")?;
    validate_mapping_authority(repository_root, &manifest, &mappings)?;
    Ok((manifest, mappings))
}

fn render_record(role: &str, semantic: &CaseSemantic) -> Result<String, Phase11EvidenceError> {
    let record = serde_json::json!({
        "schema_version": 1,
        "role": role,
        "outcome": "match",
        "semantic_sha256": canonical_sha256(&semantic)?,
        "semantic": semantic,
    });
    serde_json::to_string(&record)
        .map_err(|error| Phase11EvidenceError::new("json", error.to_string()))
}

fn validate_records(root: &Path, semantics: &[CaseSemantic]) -> Result<(), Phase11EvidenceError> {
    for role in ROLES {
        let bytes = read_regular(&root.join(format!("{role}.jsonl")), "proof", MAX_JSON_BYTES)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| Phase11EvidenceError::new("proof", error.to_string()))?;
        let records = text
            .lines()
            .map(|line| {
                serde_json::from_str::<ProofRecord>(line)
                    .map_err(|error| Phase11EvidenceError::new("proof", error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if records.len() != semantics.len() {
            return Err(Phase11EvidenceError::new(
                "proof",
                format!("{role} proof omits or adds cases"),
            ));
        }
        for (record, expected) in records.iter().zip(semantics) {
            if record.schema_version != 1
                || record.role != role
                || record.outcome != "match"
                || record.semantic != *expected
            {
                return Err(Phase11EvidenceError::new(
                    "proof",
                    format!("{role} proof reinterprets semantic content"),
                ));
            }
            require_sha256(
                "proof semantic",
                &record.semantic_sha256,
                &canonical_sha256(&record.semantic)?,
            )?;
        }
    }
    Ok(())
}

fn expected_files(manifest: &CorpusManifest, source_only: bool) -> BTreeSet<String> {
    let mut files = BTreeSet::from([MANIFEST_FILE.to_owned()]);
    for payload in &manifest.payloads {
        if let Ok(filename) = payload_filename(&payload.path) {
            files.insert(format!("cases/{filename}"));
        }
    }
    if !source_only {
        files.extend(ROLES.map(|role| format!("{role}.jsonl")));
        files.insert(IDENTITY_FILE.to_owned());
    }
    files
}

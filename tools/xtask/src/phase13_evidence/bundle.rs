//! Pure immutable bundle construction and read-only validation.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CanonicalEnvironment, ProductionGate, valid_digest, valid_revision};

pub(crate) const MANIFEST_NAME: &str = "phase13-bundle.json";
const SCHEMA_VERSION: u32 = 1;
const REQUIRED_NOTICE: &str = "THIRD_PARTY_NOTICES.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BundleErrorKind {
    Closure,
    Digest,
    FileSet,
    Identity,
    Metadata,
    Path,
    Schema,
    Symlink,
    Write,
}

#[derive(Debug)]
pub(crate) struct BundleError {
    kind: BundleErrorKind,
    message: String,
}

impl BundleError {
    fn new(kind: BundleErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[allow(
        dead_code,
        reason = "integration contract tests inspect stable categories"
    )]
    pub(crate) const fn kind(&self) -> BundleErrorKind {
        self.kind
    }
}

impl Display for BundleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase13 bundle/{:?}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for BundleError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceMetadata {
    pub(crate) record_class: String,
    pub(crate) source_revision: String,
    pub(crate) source_path: String,
    pub(crate) derivation_kind: String,
    pub(crate) alteration_summary: String,
    pub(crate) notice_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClosureEntry {
    pub(crate) path: String,
    pub(crate) sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClosureIdentity {
    pub(crate) schema_version: u32,
    pub(crate) label: String,
    pub(crate) digest: String,
    pub(crate) entries: Vec<ClosureEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BundleFile {
    pub(crate) path: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) metadata: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BundleDraft {
    pub(crate) producer: ProductionGate,
    pub(crate) witness_closure: ClosureIdentity,
    pub(crate) replay_closure: ClosureIdentity,
    pub(crate) materials_manifest_sha256: String,
    pub(crate) materials_sha256: String,
    pub(crate) probe_source_sha256: String,
    pub(crate) schema_identity: String,
    pub(crate) tolerance_identity: String,
    pub(crate) witness_invocation: Vec<String>,
    pub(crate) replay_invocations: Vec<String>,
    pub(crate) d1_oracle_identity_sha256: String,
    pub(crate) d1_result: String,
    pub(crate) diagnosis: serde_json::Value,
    pub(crate) bundle_metadata: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BundleIdentity {
    pub(crate) producer_sha: String,
    pub(crate) bundle_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileEntry {
    path: String,
    sha256: String,
    bytes: u64,
    #[serde(flatten)]
    metadata: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleManifest {
    schema_version: u32,
    producer_sha: String,
    upstream_revision: String,
    bundle_sha256: String,
    environment: CanonicalEnvironment,
    witness_closure: ClosureIdentity,
    replay_closure: ClosureIdentity,
    materials_manifest_sha256: String,
    materials_sha256: String,
    probe_source_sha256: String,
    schema_identity: String,
    tolerance_identity: String,
    witness_invocation: Vec<String>,
    replay_invocations: Vec<String>,
    witness_repeat_sha256: [String; 2],
    native_d0_repeat_sha256: [String; 2],
    sealed_input_sha256: String,
    d1_input_sha256: String,
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
    #[serde(flatten)]
    metadata: EvidenceMetadata,
    files: Vec<FileEntry>,
}

pub(crate) fn write_bundle(
    root: &Path,
    draft: BundleDraft,
    mut files: Vec<BundleFile>,
) -> Result<BundleIdentity, BundleError> {
    draft
        .producer
        .validate()
        .map_err(|error| BundleError::new(BundleErrorKind::Identity, error.to_string()))?;
    validate_draft(&draft)?;

    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut paths = BTreeSet::new();
    let mut entries = Vec::with_capacity(files.len());
    for file in &files {
        validate_relative_path(&file.path)?;
        if file.path == MANIFEST_NAME || !paths.insert(file.path.as_str()) {
            return Err(BundleError::new(
                BundleErrorKind::Path,
                "bundle file paths must be unique and cannot name the manifest",
            ));
        }
        validate_metadata(&file.metadata, &draft.producer.upstream_revision)?;
        entries.push(FileEntry {
            path: file.path.clone(),
            sha256: sha256(&file.bytes),
            bytes: u64::try_from(file.bytes.len()).map_err(|_error| {
                BundleError::new(BundleErrorKind::Schema, "bundle file is too large")
            })?,
            metadata: file.metadata.clone(),
        });
    }
    if entries.is_empty() {
        return Err(BundleError::new(
            BundleErrorKind::FileSet,
            "bundle must contain evidence files",
        ));
    }

    reject_existing_or_linked_root(root)?;
    fs::create_dir(root).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Write,
            format!("failed to create staging root: {error}"),
        )
    })?;
    for file in &files {
        write_file(root, &file.path, &file.bytes)?;
    }

    let mut manifest = BundleManifest {
        schema_version: SCHEMA_VERSION,
        producer_sha: draft.producer.producer_sha.clone(),
        upstream_revision: draft.producer.upstream_revision.clone(),
        bundle_sha256: String::new(),
        environment: draft.producer.environment,
        witness_closure: draft.witness_closure,
        replay_closure: draft.replay_closure,
        materials_manifest_sha256: draft.materials_manifest_sha256,
        materials_sha256: draft.materials_sha256,
        probe_source_sha256: draft.probe_source_sha256,
        schema_identity: draft.schema_identity,
        tolerance_identity: draft.tolerance_identity,
        witness_invocation: draft.witness_invocation,
        replay_invocations: draft.replay_invocations,
        witness_repeat_sha256: draft.producer.witness_repeat_sha256,
        native_d0_repeat_sha256: draft.producer.native_d0_repeat_sha256,
        sealed_input_sha256: draft.producer.sealed_input_sha256,
        d1_input_sha256: draft.producer.d1_input_sha256,
        d1_oracle_identity_sha256: draft.d1_oracle_identity_sha256,
        d1_result: draft.d1_result,
        diagnosis: draft.diagnosis,
        metadata: draft.bundle_metadata,
        files: entries,
    };
    manifest.bundle_sha256 = manifest_digest(&manifest)?;
    let manifest_bytes = canonical_json(&manifest)?;
    write_file(root, MANIFEST_NAME, &manifest_bytes)?;
    check_bundle(
        root,
        &manifest.producer_sha,
        &manifest.bundle_sha256,
        Some(&manifest.witness_closure.digest),
        Some(&manifest.replay_closure.digest),
    )?;
    Ok(BundleIdentity {
        producer_sha: manifest.producer_sha,
        bundle_sha256: manifest.bundle_sha256,
    })
}

pub(crate) fn check_bundle(
    root: &Path,
    expected_producer_sha: &str,
    expected_bundle_sha256: &str,
    maybe_expected_witness_closure: Option<&str>,
    maybe_expected_replay_closure: Option<&str>,
) -> Result<BundleIdentity, BundleError> {
    if !valid_revision(expected_producer_sha) || !valid_digest(expected_bundle_sha256) {
        return Err(BundleError::new(
            BundleErrorKind::Identity,
            "expected P and B must be full lowercase identities",
        ));
    }
    reject_symlink(root)?;
    let manifest_path = root.join(MANIFEST_NAME);
    reject_symlink(&manifest_path)?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        BundleError::new(
            BundleErrorKind::FileSet,
            format!("failed to read bundle manifest: {error}"),
        )
    })?;
    let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Schema,
            format!("invalid bundle manifest: {error}"),
        )
    })?;
    validate_manifest(&manifest)?;
    if manifest.producer_sha != expected_producer_sha
        || manifest.bundle_sha256 != expected_bundle_sha256
        || manifest_digest(&manifest)? != manifest.bundle_sha256
    {
        return Err(BundleError::new(
            BundleErrorKind::Digest,
            "bundle P or B does not match its expected identity",
        ));
    }
    if maybe_expected_witness_closure
        .is_some_and(|expected| expected != manifest.witness_closure.digest)
        || maybe_expected_replay_closure
            .is_some_and(|expected| expected != manifest.replay_closure.digest)
    {
        return Err(BundleError::new(
            BundleErrorKind::Closure,
            "bundle producer-affecting closure changed",
        ));
    }

    let actual_paths = collect_regular_files(root)?;
    let expected_paths = std::iter::once(MANIFEST_NAME.to_owned())
        .chain(manifest.files.iter().map(|entry| entry.path.clone()))
        .collect::<BTreeSet<_>>();
    if actual_paths != expected_paths {
        return Err(BundleError::new(
            BundleErrorKind::FileSet,
            "bundle has extra or missing files",
        ));
    }
    for entry in &manifest.files {
        let path = root.join(&entry.path);
        reject_symlink(&path)?;
        let bytes = fs::read(&path).map_err(|error| {
            BundleError::new(
                BundleErrorKind::FileSet,
                format!("failed to read staged file: {error}"),
            )
        })?;
        if sha256(&bytes) != entry.sha256 || u64::try_from(bytes.len()).ok() != Some(entry.bytes) {
            return Err(BundleError::new(
                BundleErrorKind::Digest,
                format!("staged file `{}` changed", entry.path),
            ));
        }
    }
    Ok(BundleIdentity {
        producer_sha: manifest.producer_sha,
        bundle_sha256: manifest.bundle_sha256,
    })
}

fn validate_manifest(manifest: &BundleManifest) -> Result<(), BundleError> {
    if manifest.schema_version != SCHEMA_VERSION
        || !valid_revision(&manifest.producer_sha)
        || !valid_revision(&manifest.upstream_revision)
        || !valid_digest(&manifest.bundle_sha256)
        || manifest.environment.operating_system != "linux"
        || manifest.environment.architecture != "x86_64"
        || manifest.environment.rust_target != "x86_64-unknown-linux-gnu"
        || manifest.environment.rust_version != "1.97.0"
        || manifest.environment.cmake_version != "4.3.3"
        || manifest.environment.ninja_version != "1.13.2"
        || manifest.environment.clang_version != "22.1.8"
        || manifest.environment.cmake_preset != "oracle-debug"
        || manifest.d1_result != "match"
        || manifest.sealed_input_sha256 != manifest.d1_input_sha256
        || manifest.witness_repeat_sha256[0] != manifest.witness_repeat_sha256[1]
        || manifest.native_d0_repeat_sha256[0] != manifest.native_d0_repeat_sha256[1]
    {
        return Err(BundleError::new(
            BundleErrorKind::Schema,
            "bundle manifest has invalid identity or evidence gates",
        ));
    }
    validate_closure(&manifest.witness_closure, "witness")?;
    validate_closure(&manifest.replay_closure, "replay")?;
    validate_metadata(&manifest.metadata, &manifest.upstream_revision)?;
    let mut previous: Option<&str> = None;
    for entry in &manifest.files {
        validate_relative_path(&entry.path)?;
        validate_metadata(&entry.metadata, &manifest.upstream_revision)?;
        require_digest(&entry.sha256)?;
        if previous.is_some_and(|value| value >= entry.path.as_str()) {
            return Err(BundleError::new(
                BundleErrorKind::FileSet,
                "manifest file set must be strictly ordered and unique",
            ));
        }
        previous = Some(&entry.path);
    }
    Ok(())
}

fn validate_draft(draft: &BundleDraft) -> Result<(), BundleError> {
    validate_closure(&draft.witness_closure, "witness")?;
    validate_closure(&draft.replay_closure, "replay")?;
    validate_metadata(&draft.bundle_metadata, &draft.producer.upstream_revision)?;
    for digest in [
        draft.materials_manifest_sha256.as_str(),
        draft.materials_sha256.as_str(),
        draft.probe_source_sha256.as_str(),
        draft.tolerance_identity.as_str(),
        draft.d1_oracle_identity_sha256.as_str(),
    ] {
        require_digest(digest)?;
    }
    if draft.schema_identity.trim().is_empty()
        || draft.witness_invocation.is_empty()
        || draft.replay_invocations.len() != 3
        || draft.d1_result != "match"
    {
        return Err(BundleError::new(
            BundleErrorKind::Schema,
            "producer metadata is incomplete",
        ));
    }
    Ok(())
}

fn validate_closure(closure: &ClosureIdentity, expected_label: &str) -> Result<(), BundleError> {
    if closure.schema_version != 1
        || closure.label != expected_label
        || !valid_digest(&closure.digest)
        || closure.entries.is_empty()
    {
        return Err(BundleError::new(
            BundleErrorKind::Closure,
            format!("invalid {expected_label} closure identity"),
        ));
    }
    let mut previous: Option<&str> = None;
    for entry in &closure.entries {
        validate_relative_path(&entry.path)?;
        require_digest(&entry.sha256)?;
        if previous.is_some_and(|value| value >= entry.path.as_str()) {
            return Err(BundleError::new(
                BundleErrorKind::Closure,
                "closure entries must be strictly ordered and unique",
            ));
        }
        previous = Some(&entry.path);
    }
    let actual = closure_digest(&closure.label, &closure.entries);
    if actual != closure.digest {
        return Err(BundleError::new(
            BundleErrorKind::Closure,
            format!("{expected_label} closure digest mismatch"),
        ));
    }
    Ok(())
}

pub(crate) fn closure_digest(label: &str, entries: &[ClosureEntry]) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-closure-v1");
    update_field(&mut hasher, label.as_bytes());
    for entry in entries {
        update_field(&mut hasher, entry.path.as_bytes());
        update_field(&mut hasher, entry.sha256.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn manifest_digest(manifest: &BundleManifest) -> Result<String, BundleError> {
    let mut value = serde_json::to_value(manifest).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Schema,
            format!("failed to encode bundle identity: {error}"),
        )
    })?;
    let object = value.as_object_mut().ok_or_else(|| {
        BundleError::new(BundleErrorKind::Schema, "bundle manifest is not an object")
    })?;
    object.remove("bundle_sha256");
    let bytes = serde_json::to_vec(&value).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Schema,
            format!("failed to encode bundle identity: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-bundle-v1");
    update_field(&mut hasher, &bytes);
    for entry in &manifest.files {
        update_field(&mut hasher, entry.path.as_bytes());
        update_field(&mut hasher, entry.sha256.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_metadata(metadata: &EvidenceMetadata, upstream: &str) -> Result<(), BundleError> {
    if !matches!(
        metadata.record_class.as_str(),
        "witness" | "replay_evidence" | "staged_bundle"
    ) || metadata.source_revision != upstream
        || !valid_relative_or_dot(&metadata.source_path)
        || metadata.derivation_kind.trim().is_empty()
        || metadata.alteration_summary.trim().is_empty()
        || metadata.notice_refs != [REQUIRED_NOTICE]
    {
        return Err(BundleError::new(
            BundleErrorKind::Metadata,
            "staged evidence does not satisfy the FND-04 metadata contract",
        ));
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), BundleError> {
    if !valid_relative_or_dot(value) || value == "." {
        return Err(BundleError::new(
            BundleErrorKind::Path,
            format!("unsafe bundle path `{value}`"),
        ));
    }
    Ok(())
}

fn valid_relative_or_dot(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn reject_existing_or_linked_root(root: &Path) -> Result<(), BundleError> {
    if root.exists() {
        return Err(BundleError::new(
            BundleErrorKind::Write,
            "staging root must not already exist",
        ));
    }
    let Some(parent) = root.parent() else {
        return Err(BundleError::new(
            BundleErrorKind::Path,
            "staging root must have a parent",
        ));
    };
    reject_symlink(parent)
}

fn write_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), BundleError> {
    let path = root.join(relative);
    let parent = path
        .parent()
        .ok_or_else(|| BundleError::new(BundleErrorKind::Path, "bundle file must have a parent"))?;
    create_directories(root, parent)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            BundleError::new(
                BundleErrorKind::Write,
                format!("failed to create staged file: {error}"),
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Write,
            format!("failed to write staged file: {error}"),
        )
    })
}

fn create_directories(root: &Path, target: &Path) -> Result<(), BundleError> {
    let relative = target.strip_prefix(root).map_err(|_error| {
        BundleError::new(
            BundleErrorKind::Path,
            "bundle directory escaped staging root",
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(BundleError::new(
                BundleErrorKind::Path,
                "bundle directory has an unsafe component",
            ));
        };
        current.push(part);
        if current.exists() {
            reject_symlink(&current)?;
        } else {
            fs::create_dir(&current).map_err(|error| {
                BundleError::new(
                    BundleErrorKind::Write,
                    format!("failed to create bundle directory: {error}"),
                )
            })?;
        }
    }
    Ok(())
}

fn collect_regular_files(root: &Path) -> Result<BTreeSet<String>, BundleError> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        reject_symlink(&directory)?;
        for entry in fs::read_dir(&directory).map_err(|error| {
            BundleError::new(
                BundleErrorKind::FileSet,
                format!("failed to enumerate bundle: {error}"),
            )
        })? {
            let entry = entry.map_err(|error| {
                BundleError::new(
                    BundleErrorKind::FileSet,
                    format!("failed to enumerate bundle: {error}"),
                )
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                BundleError::new(
                    BundleErrorKind::FileSet,
                    format!("failed to inspect bundle entry: {error}"),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(BundleError::new(
                    BundleErrorKind::Symlink,
                    "bundle contains a symbolic link",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_error| {
                    BundleError::new(BundleErrorKind::Path, "bundle entry escaped root")
                })?;
                paths.insert(path_text(relative)?);
            } else {
                return Err(BundleError::new(
                    BundleErrorKind::FileSet,
                    "bundle contains a non-regular entry",
                ));
            }
        }
    }
    Ok(paths)
}

fn reject_symlink(path: &Path) -> Result<(), BundleError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        BundleError::new(
            BundleErrorKind::FileSet,
            format!("failed to inspect bundle path: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(BundleError::new(
            BundleErrorKind::Symlink,
            "bundle path contains a symbolic link",
        ));
    }
    Ok(())
}

fn path_text(path: &Path) -> Result<String, BundleError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| BundleError::new(BundleErrorKind::Path, "bundle path is not valid UTF-8"))
}

fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, BundleError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        BundleError::new(
            BundleErrorKind::Schema,
            format!("failed to encode bundle manifest: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn require_digest(value: &str) -> Result<(), BundleError> {
    if valid_digest(value) {
        return Ok(());
    }
    Err(BundleError::new(
        BundleErrorKind::Digest,
        "expected a lowercase SHA-256 identity",
    ))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn update_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}

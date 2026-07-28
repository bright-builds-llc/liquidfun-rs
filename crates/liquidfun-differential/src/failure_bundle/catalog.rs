//! Exact Phase 11 catalog failure bundles with atomic confined publication.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogRunRequest, HarnessLimits, Sha256Hex, decode_resolved_scenario,
    encode_canonical_checkpoint_jsonl,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{CatalogFailureKind, CatalogRunCapture};

use super::FailureBundleError;

mod comparison;
mod replay;

use comparison::{comparison_json, first_divergence_json};
pub use replay::CatalogBundleReplay;
pub(crate) use replay::replay_catalog_bundle;

pub(super) const SCHEMA_VERSION: u32 = 2;
pub(super) const MAXIMUM_FIELD_BYTES: usize = 1024 * 1024;
const MAXIMUM_PUBLICATION_ATTEMPTS: usize = 100;
pub(super) const AUTHORITY_FILES: [&str; 7] = [
    "resolved.json",
    "action-log.json",
    "checkpoint-schedule.json",
    "native-identity.json",
    "oracle-identity.json",
    "stderr.txt",
    "controller-state.json",
];
pub(super) const CAPTURE_FILES: [&str; 3] = [
    "native-checkpoints.json",
    "oracle-checkpoints.json",
    "comparison.json",
];

/// Closed semantic surface used to compare a persisted capture pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogComparisonSurface {
    /// The complete expanded checkpoint schema, including renderer-neutral debug capture.
    ExpandedCheckpointV1,
    /// Parity-bearing physics fields after the reviewed debug-capture projection.
    LegacyPhysicsV1,
}

/// Complete owned catalog failure evidence. Construction rejects incomplete authority.
pub struct CatalogFailureBundleRequest {
    result_kind: CatalogFailureKind,
    request_id: Box<str>,
    resolved_bytes: Box<[u8]>,
    resolved_sha256: Sha256Hex,
    action_log_json: Box<[u8]>,
    checkpoint_schedule_json: Box<[u8]>,
    maybe_comparison_surface: Option<CatalogComparisonSurface>,
    maybe_native_checkpoints_json: Option<Box<[u8]>>,
    maybe_oracle_checkpoints_json: Option<Box<[u8]>>,
    maybe_comparison_json: Option<Box<[u8]>>,
    maybe_first_divergence_json: Option<Box<[u8]>>,
    native_identity_json: Box<[u8]>,
    oracle_identity_json: Box<[u8]>,
    stderr: Box<[u8]>,
    controller_state_json: Box<[u8]>,
}

impl CatalogFailureBundleRequest {
    /// Builds one complete bundle from two captures of the same exact resolved bytes.
    ///
    /// # Errors
    ///
    /// Returns [`FailureBundleError`] when captures conflict or evidence is incomplete.
    pub fn from_captures(
        result_kind: CatalogFailureKind,
        request: &CatalogRunRequest,
        native: &CatalogRunCapture,
        oracle: &CatalogRunCapture,
        stderr: &[u8],
        controller_state_json: &[u8],
    ) -> Result<Self, FailureBundleError> {
        Self::from_projection_captures(
            result_kind,
            CatalogComparisonSurface::ExpandedCheckpointV1,
            request,
            native,
            oracle,
            stderr,
            controller_state_json,
        )
    }

    /// Builds one complete bundle from two captures and a named comparison surface.
    ///
    /// # Errors
    ///
    /// Returns [`FailureBundleError`] when captures conflict or evidence is incomplete.
    #[allow(clippy::too_many_arguments)]
    pub fn from_projection_captures(
        result_kind: CatalogFailureKind,
        comparison_surface: CatalogComparisonSurface,
        request: &CatalogRunRequest,
        native: &CatalogRunCapture,
        oracle: &CatalogRunCapture,
        stderr: &[u8],
        controller_state_json: &[u8],
    ) -> Result<Self, FailureBundleError> {
        if !matches!(
            result_kind,
            CatalogFailureKind::HarnessFailure | CatalogFailureKind::PhysicsMismatch
        ) || native.resolved_bytes() != request.resolved().canonical_bytes()
            || oracle.resolved_bytes() != request.resolved().canonical_bytes()
            || native.resolved_sha256() != request.resolved().identity().content_sha256()
            || oracle.resolved_sha256() != request.resolved().identity().content_sha256()
            || native.action_log() != oracle.action_log()
            || native.checkpoint_schedule() != oracle.checkpoint_schedule()
        {
            return Err(evidence_error("catalog capture authority mismatch"));
        }
        let comparison_json = comparison_json(comparison_surface, native, oracle)?;
        let maybe_first_divergence_json = if result_kind == CatalogFailureKind::PhysicsMismatch {
            first_divergence_json(comparison_surface, native, oracle)?.map(Vec::into_boxed_slice)
        } else {
            None
        };
        if result_kind == CatalogFailureKind::PhysicsMismatch
            && maybe_first_divergence_json.is_none()
        {
            return Err(evidence_error("physics mismatch has no first divergence"));
        }
        let action_log_json = json_line(native.action_log())?;
        let checkpoint_schedule_json = json_line(native.checkpoint_schedule())?;
        let native_checkpoints_json = json_line(native.checkpoints())?;
        let oracle_checkpoints_json = json_line(oracle.checkpoints())?;
        let native_identity_json = identity_json("native_rust", request)?;
        let oracle_identity_json = identity_json("cpp_oracle", request)?;
        validate_json(controller_state_json, "controller-state.json")?;
        let request = Self {
            result_kind,
            request_id: request.request_id().as_str().into(),
            resolved_bytes: request.resolved().canonical_bytes().into(),
            resolved_sha256: request.resolved().identity().content_sha256().clone(),
            action_log_json: action_log_json.into_boxed_slice(),
            checkpoint_schedule_json: checkpoint_schedule_json.into_boxed_slice(),
            maybe_comparison_surface: Some(comparison_surface),
            maybe_native_checkpoints_json: Some(native_checkpoints_json.into_boxed_slice()),
            maybe_oracle_checkpoints_json: Some(oracle_checkpoints_json.into_boxed_slice()),
            maybe_comparison_json: Some(comparison_json.into_boxed_slice()),
            maybe_first_divergence_json,
            native_identity_json: native_identity_json.into_boxed_slice(),
            oracle_identity_json: oracle_identity_json.into_boxed_slice(),
            stderr: stderr.into(),
            controller_state_json: controller_state_json.into(),
        };
        request.validate()?;
        Ok(request)
    }

    /// Builds exact replay authority for a typed harness failure before comparable captures exist.
    ///
    /// # Errors
    ///
    /// Returns [`FailureBundleError`] when the category, request, or bounded diagnostics are invalid.
    pub fn from_harness_failure(
        result_kind: CatalogFailureKind,
        request: &CatalogRunRequest,
        stderr: &[u8],
        controller_state_json: &[u8],
    ) -> Result<Self, FailureBundleError> {
        if matches!(
            result_kind,
            CatalogFailureKind::PhysicsMismatch | CatalogFailureKind::HarnessFailure
        ) {
            return Err(evidence_error(
                "capture-free failure requires a specific harness category",
            ));
        }
        validate_json(controller_state_json, "controller-state.json")?;
        let request = Self {
            result_kind,
            request_id: request.request_id().as_str().into(),
            resolved_bytes: request.resolved().canonical_bytes().into(),
            resolved_sha256: request.resolved().identity().content_sha256().clone(),
            action_log_json: json_line(
                &request
                    .resolved()
                    .actions()
                    .iter()
                    .map(|action| action.action_id().clone())
                    .collect::<Vec<_>>(),
            )?
            .into_boxed_slice(),
            checkpoint_schedule_json: json_line(request.resolved().checkpoints())?
                .into_boxed_slice(),
            maybe_comparison_surface: None,
            maybe_native_checkpoints_json: None,
            maybe_oracle_checkpoints_json: None,
            maybe_comparison_json: None,
            maybe_first_divergence_json: None,
            native_identity_json: identity_json("native_rust", request)?.into_boxed_slice(),
            oracle_identity_json: identity_json("cpp_oracle", request)?.into_boxed_slice(),
            stderr: stderr.into(),
            controller_state_json: controller_state_json.into(),
        };
        request.validate()?;
        Ok(request)
    }

    /// Explicitly rejects seed-only replay authority.
    ///
    /// # Errors
    ///
    /// Always returns [`FailureBundleError`] because a seed is never replay authority.
    pub fn from_seed_only(
        _result_kind: CatalogFailureKind,
        _maybe_seed: Option<u64>,
    ) -> Result<Self, FailureBundleError> {
        Err(evidence_error(
            "seed-only catalog replay authority is prohibited",
        ))
    }

    fn validate(&self) -> Result<(), FailureBundleError> {
        let limits = HarnessLimits::phase2_default_v1();
        if self.resolved_bytes.is_empty() || self.resolved_bytes.len() > limits.input_record_bytes()
        {
            return Err(size_error("resolved.json", limits.input_record_bytes()));
        }
        decode_resolved_scenario(&self.resolved_bytes, &self.resolved_sha256)
            .map_err(|_error| evidence_error("resolved bytes failed exact replay"))?;
        for (name, bytes, limit) in self.files_without_signature() {
            if (bytes.is_empty() && name != "stderr.txt") || bytes.len() > limit {
                return Err(size_error(name, limit));
            }
        }
        if self.stderr.len() > limits.retained_stderr_bytes() {
            return Err(size_error("stderr.txt", limits.retained_stderr_bytes()));
        }
        for (name, bytes) in [
            ("action-log.json", self.action_log_json.as_ref()),
            (
                "checkpoint-schedule.json",
                self.checkpoint_schedule_json.as_ref(),
            ),
            ("native-identity.json", self.native_identity_json.as_ref()),
            ("oracle-identity.json", self.oracle_identity_json.as_ref()),
            ("controller-state.json", self.controller_state_json.as_ref()),
        ] {
            validate_json(bytes, name)?;
        }
        for (name, maybe_bytes) in [
            (
                "native-checkpoints.json",
                self.maybe_native_checkpoints_json.as_deref(),
            ),
            (
                "oracle-checkpoints.json",
                self.maybe_oracle_checkpoints_json.as_deref(),
            ),
            ("comparison.json", self.maybe_comparison_json.as_deref()),
        ] {
            if let Some(bytes) = maybe_bytes {
                validate_json(bytes, name)?;
            }
        }
        let capture_count = [
            self.maybe_native_checkpoints_json.is_some(),
            self.maybe_oracle_checkpoints_json.is_some(),
            self.maybe_comparison_json.is_some(),
            self.maybe_comparison_surface.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if capture_count != 0 && capture_count != 4 {
            return Err(evidence_error("catalog capture evidence is incomplete"));
        }
        Ok(())
    }

    fn files_without_signature(&self) -> Vec<(&'static str, &[u8], usize)> {
        let mut files: Vec<(&'static str, &[u8], usize)> = vec![
            (
                "resolved.json",
                self.resolved_bytes.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
            (
                "action-log.json",
                self.action_log_json.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
            (
                "checkpoint-schedule.json",
                self.checkpoint_schedule_json.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
            (
                "native-identity.json",
                self.native_identity_json.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
            (
                "oracle-identity.json",
                self.oracle_identity_json.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
            (
                "stderr.txt",
                self.stderr.as_ref(),
                HarnessLimits::phase2_default_v1().retained_stderr_bytes(),
            ),
            (
                "controller-state.json",
                self.controller_state_json.as_ref(),
                MAXIMUM_FIELD_BYTES,
            ),
        ];
        for (name, maybe_bytes) in [
            (
                "native-checkpoints.json",
                self.maybe_native_checkpoints_json.as_deref(),
            ),
            (
                "oracle-checkpoints.json",
                self.maybe_oracle_checkpoints_json.as_deref(),
            ),
            ("comparison.json", self.maybe_comparison_json.as_deref()),
        ] {
            if let Some(bytes) = maybe_bytes {
                files.push((name, bytes, MAXIMUM_FIELD_BYTES));
            }
        }
        files
    }
}

/// Receipt for an atomically published catalog failure bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogFailureBundleReceipt {
    directory: PathBuf,
}

impl CatalogFailureBundleReceipt {
    /// Returns the confined published directory.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Manifest {
    pub(super) schema_version: u32,
    pub(super) result_kind: CatalogFailureKind,
    pub(super) maybe_comparison_surface: Option<CatalogComparisonSurface>,
    pub(super) request_id: String,
    pub(super) resolved_sha256: Sha256Hex,
    pub(super) files: BTreeMap<String, FileEntry>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct FileEntry {
    pub(super) bytes: usize,
    pub(super) sha256: String,
}

/// Atomically publishes one complete bounded catalog failure bundle.
///
/// # Errors
///
/// Returns [`FailureBundleError`] for incomplete evidence, unsafe paths, bounds, or I/O failure.
pub fn persist_catalog_failure_bundle(
    repository_root: &Path,
    request: &CatalogFailureBundleRequest,
) -> Result<CatalogFailureBundleReceipt, FailureBundleError> {
    request.validate()?;
    let root = ensure_catalog_root(repository_root)?;
    let (temporary, final_path) = allocate_paths(&root, &request.request_id)?;
    fs::create_dir(&temporary)?;
    let result = write_bundle(&temporary, request).and_then(|()| {
        sync_directory(&temporary)?;
        fs::rename(&temporary, &final_path)?;
        sync_directory(&root)
    });
    if let Err(error) = result {
        let _ignored = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    Ok(CatalogFailureBundleReceipt {
        directory: final_path,
    })
}

fn write_bundle(
    directory: &Path,
    request: &CatalogFailureBundleRequest,
) -> Result<(), FailureBundleError> {
    let mut files = BTreeMap::new();
    for (name, bytes, _limit) in request.files_without_signature() {
        write_new(&directory.join(name), bytes)?;
        files.insert(name.to_owned(), file_entry(bytes));
    }
    if let Some(bytes) = request.maybe_first_divergence_json.as_deref() {
        write_new(&directory.join("first-divergence.json"), bytes)?;
        files.insert("first-divergence.json".to_owned(), file_entry(bytes));
    }
    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        result_kind: request.result_kind,
        maybe_comparison_surface: request.maybe_comparison_surface,
        request_id: request.request_id.to_string(),
        resolved_sha256: request.resolved_sha256.clone(),
        files,
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    write_new(&directory.join("manifest.json"), &bytes)
}

fn identity_json(engine: &str, request: &CatalogRunRequest) -> Result<Vec<u8>, FailureBundleError> {
    json_line(&serde_json::json!({
        "engine": engine,
        "required_identity_sha256": request
            .provenance_requirements()
            .required_identity_sha256(),
        "limits_profile_sha256": request
            .provenance_requirements()
            .limits_profile_sha256(),
        "evidence_tier": request.provenance_requirements().evidence_tier(),
    }))
}

fn encode_checkpoint(checkpoint: &CanonicalCheckpoint) -> Result<Vec<u8>, FailureBundleError> {
    encode_canonical_checkpoint_jsonl(checkpoint, &HarnessLimits::phase2_default_v1())
        .map_err(|error| evidence_error(&error.to_string()))
}

fn json_line<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, FailureBundleError> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_json(bytes: &[u8], field: &'static str) -> Result<(), FailureBundleError> {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .map(|_value| ())
        .map_err(|_error| size_error(field, MAXIMUM_FIELD_BYTES))
}

pub(super) fn ensure_catalog_root(repository_root: &Path) -> Result<PathBuf, FailureBundleError> {
    let canonical_repository = fs::canonicalize(repository_root)?;
    let mut path = repository_root.to_path_buf();
    for component in ["target", "differential", "catalog-failures"] {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(evidence_error("catalog evidence boundary is unsafe"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&path)?,
            Err(error) => return Err(error.into()),
        }
    }
    let canonical = fs::canonicalize(path)?;
    if !canonical.starts_with(canonical_repository) {
        return Err(evidence_error("catalog evidence root escaped repository"));
    }
    Ok(canonical)
}

fn allocate_paths(root: &Path, request_id: &str) -> Result<(PathBuf, PathBuf), FailureBundleError> {
    for ordinal in 0..MAXIMUM_PUBLICATION_ATTEMPTS {
        let suffix = if ordinal == 0 {
            String::new()
        } else {
            format!("-{ordinal}")
        };
        let final_path = root.join(format!("{request_id}{suffix}"));
        let temporary = root.join(format!(".{request_id}{suffix}.tmp"));
        if !final_path.exists() && !temporary.exists() {
            return Ok((temporary, final_path));
        }
    }
    Err(FailureBundleError::DirectoryExhausted {
        root: root.to_path_buf(),
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), FailureBundleError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), FailureBundleError> {
    fs::File::open(path)?.sync_all()?;
    Ok(())
}

pub(super) fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, FailureBundleError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(evidence_error("catalog bundle entry is unsafe"));
    }
    let length = usize::try_from(metadata.len())
        .map_err(|_error| size_error("catalog bundle entry", limit))?;
    if length > limit {
        return Err(size_error("catalog bundle entry", limit));
    }
    Ok(fs::read(path)?)
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), FailureBundleError> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(evidence_error("catalog bundle path is a symlink"));
    }
    Ok(())
}

fn file_entry(bytes: &[u8]) -> FileEntry {
    FileEntry {
        bytes: bytes.len(),
        sha256: digest(bytes),
    }
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn evidence_error(message: &str) -> FailureBundleError {
    FailureBundleError::Io(io::Error::other(message.to_owned()))
}

const fn size_error(field: &'static str, limit: usize) -> FailureBundleError {
    FailureBundleError::SizeLimit { field, limit }
}

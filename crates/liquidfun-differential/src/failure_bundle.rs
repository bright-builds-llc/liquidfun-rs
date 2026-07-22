//! Bounded, hash-indexed failure evidence persisted below the ignored target tree.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use liquidfun_test_protocol::{HarnessLimits, RequestId};
use serde::Serialize;
use sha2::{Digest, Sha256};

mod catalog;
pub use catalog::*;

const MAXIMUM_REPORT_BYTES: usize = 1024 * 1024;
const MAXIMUM_IDENTITY_BYTES: usize = 64 * 1024;
const MAXIMUM_BUNDLE_ATTEMPTS: usize = 100;

/// Exact bounded evidence supplied by the command after classifying a failed run.
pub struct FailureBundleRequest<'a> {
    /// Stable result category (`physics_mismatch` or `harness_failure`).
    pub result_kind: &'static str,
    /// Validated request identity used to derive a confined directory name.
    pub request_id: &'a RequestId,
    /// Exact newline-complete scenario request.
    pub request_jsonl: &'a [u8],
    /// Exact machine-readable command report.
    pub report_json: &'a [u8],
    /// Build and session identity summary.
    pub identity_json: &'a [u8],
    /// Bounded retained stderr, empty when no stderr was produced.
    pub stderr: &'a [u8],
    /// Optional exact typed first-divergence signature for replay and reduction.
    pub maybe_failure_signature_json: Option<&'a [u8]>,
}

/// Successfully persisted evidence directory and its manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureBundleReceipt {
    directory: PathBuf,
}

/// Bounded minimized scenario and its machine report.
pub struct MinimizationArtifactRequest<'a> {
    /// Validated request identity used to derive a confined directory name.
    pub request_id: &'a RequestId,
    /// Canonical minimized scenario JSON.
    pub scenario_json: &'a [u8],
    /// Machine-readable minimization result.
    pub report_json: &'a [u8],
}

/// Canonical minimized rigid request and its machine report.
pub struct RigidMinimizationArtifactRequest<'a> {
    /// Validated request identity used to derive a confined directory name.
    pub request_id: &'a RequestId,
    /// Canonical newline-complete minimized rigid request.
    pub request_jsonl: &'a [u8],
    /// Machine-readable minimization result including status and transforms.
    pub report_json: &'a [u8],
}

/// Successfully persisted minimized scenario evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimizationArtifactReceipt {
    directory: PathBuf,
}

impl MinimizationArtifactReceipt {
    /// Returns the newly created minimized-artifact directory.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

impl FailureBundleReceipt {
    /// Returns the newly created bundle directory.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

/// Failure while validating or atomically persisting a bounded evidence bundle.
#[derive(Debug, thiserror::Error)]
pub enum FailureBundleError {
    /// A supplied evidence field exceeded its reviewed cap.
    #[error("failure bundle field `{field}` exceeds {limit} bytes")]
    SizeLimit {
        /// Bounded evidence field.
        field: &'static str,
        /// Maximum accepted bytes.
        limit: usize,
    },
    /// The failed result category was not one of the reviewed classes.
    #[error("unsupported failure bundle result kind `{0}`")]
    ResultKind(String),
    /// All bounded no-clobber directory names already existed.
    #[error("failure bundle directory allocation exhausted below {}", root.display())]
    DirectoryExhausted {
        /// Failure evidence root.
        root: PathBuf,
    },
    /// Filesystem persistence failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Manifest or identity JSON serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct BundleManifest {
    schema_version: u32,
    result_kind: &'static str,
    request_id: String,
    files: BTreeMap<&'static str, BundleFile>,
}

#[derive(Serialize)]
struct BundleFile {
    bytes: usize,
    sha256: String,
}

/// Persists one no-clobber failure bundle below `target/differential/failures`.
///
/// # Errors
///
/// Returns [`FailureBundleError`] for unsupported categories, oversized evidence, exhausted
/// no-clobber names, serialization failures, or filesystem errors.
pub fn persist_failure_bundle(
    repository_root: &Path,
    request: &FailureBundleRequest<'_>,
) -> Result<FailureBundleReceipt, FailureBundleError> {
    if !matches!(request.result_kind, "physics_mismatch" | "harness_failure") {
        return Err(FailureBundleError::ResultKind(
            request.result_kind.to_owned(),
        ));
    }
    let limits = HarnessLimits::phase2_default_v1();
    enforce_size(
        "request.jsonl",
        request.request_jsonl,
        limits.input_record_bytes(),
    )?;
    enforce_size("report.json", request.report_json, MAXIMUM_REPORT_BYTES)?;
    enforce_size(
        "identity.json",
        request.identity_json,
        MAXIMUM_IDENTITY_BYTES,
    )?;
    enforce_size("stderr.txt", request.stderr, limits.retained_stderr_bytes())?;
    if let Some(signature) = request.maybe_failure_signature_json {
        enforce_size("failure-signature.json", signature, MAXIMUM_IDENTITY_BYTES)?;
    }

    let root = ensure_evidence_root(repository_root, "failures")?;
    let directory = create_bundle_directory(&root, request.request_id, request.result_kind)?;
    let result = (|| {
        let evidence = [
            ("request.jsonl", request.request_jsonl),
            ("report.json", request.report_json),
            ("identity.json", request.identity_json),
            ("stderr.txt", request.stderr),
        ];
        let mut files = BTreeMap::new();
        for (name, bytes) in evidence {
            write_create_new(&directory.join(name), bytes)?;
            files.insert(
                name,
                BundleFile {
                    bytes: bytes.len(),
                    sha256: format!("{:x}", Sha256::digest(bytes)),
                },
            );
        }
        if let Some(bytes) = request.maybe_failure_signature_json {
            let name = "failure-signature.json";
            write_create_new(&directory.join(name), bytes)?;
            files.insert(
                name,
                BundleFile {
                    bytes: bytes.len(),
                    sha256: format!("{:x}", Sha256::digest(bytes)),
                },
            );
        }
        let manifest = BundleManifest {
            schema_version: 1,
            result_kind: request.result_kind,
            request_id: request.request_id.as_str().to_owned(),
            files,
        };
        let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        manifest_bytes.push(b'\n');
        write_create_new(&directory.join("manifest.json"), &manifest_bytes)
    })();
    if let Err(error) = result {
        let _ignored = fs::remove_dir_all(&directory);
        return Err(error);
    }
    Ok(FailureBundleReceipt { directory })
}

/// Persists a canonical minimized scenario and hash-indexed report below the target tree.
///
/// # Errors
///
/// Returns [`FailureBundleError`] for oversized values, exhausted no-clobber names,
/// serialization failures, or filesystem errors.
pub fn persist_minimization_artifact(
    repository_root: &Path,
    request: &MinimizationArtifactRequest<'_>,
) -> Result<MinimizationArtifactReceipt, FailureBundleError> {
    persist_minimization_files(
        repository_root,
        request.request_id,
        "minimized",
        "scenario.json",
        request.scenario_json,
        request.report_json,
    )
}

/// Persists a canonical minimized rigid request and reducer report below the target tree.
///
/// # Errors
///
/// Returns [`FailureBundleError`] for oversized values, exhausted no-clobber names,
/// serialization failures, or filesystem errors.
pub fn persist_rigid_minimization_artifact(
    repository_root: &Path,
    request: &RigidMinimizationArtifactRequest<'_>,
) -> Result<MinimizationArtifactReceipt, FailureBundleError> {
    persist_minimization_files(
        repository_root,
        request.request_id,
        "minimized-rigid",
        "request.jsonl",
        request.request_jsonl,
        request.report_json,
    )
}

fn persist_minimization_files(
    repository_root: &Path,
    request_id: &RequestId,
    leaf: &str,
    value_name: &'static str,
    value_bytes: &[u8],
    report_json: &[u8],
) -> Result<MinimizationArtifactReceipt, FailureBundleError> {
    let limits = HarnessLimits::phase2_default_v1();
    enforce_size(value_name, value_bytes, limits.input_record_bytes())?;
    enforce_size("report.json", report_json, MAXIMUM_REPORT_BYTES)?;
    let root = ensure_evidence_root(repository_root, leaf)?;
    let directory = create_bundle_directory(&root, request_id, "minimization")?;
    let result = (|| {
        let evidence = [(value_name, value_bytes), ("report.json", report_json)];
        let mut files = BTreeMap::new();
        for (name, bytes) in evidence {
            write_create_new(&directory.join(name), bytes)?;
            files.insert(
                name,
                BundleFile {
                    bytes: bytes.len(),
                    sha256: format!("{:x}", Sha256::digest(bytes)),
                },
            );
        }
        let manifest = BundleManifest {
            schema_version: 1,
            result_kind: "minimization",
            request_id: request_id.as_str().to_owned(),
            files,
        };
        let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        manifest_bytes.push(b'\n');
        write_create_new(&directory.join("manifest.json"), &manifest_bytes)
    })();
    if let Err(error) = result {
        let _ignored = fs::remove_dir_all(&directory);
        return Err(error);
    }
    Ok(MinimizationArtifactReceipt { directory })
}

fn ensure_evidence_root(repository_root: &Path, leaf: &str) -> Result<PathBuf, FailureBundleError> {
    let canonical_root = fs::canonicalize(repository_root)?;
    let mut path = repository_root.to_path_buf();
    for component in ["target", "differential", leaf] {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(FailureBundleError::Io(io::Error::other(format!(
                    "failure evidence boundary is not a regular directory: {}",
                    path.display()
                ))));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&path)?,
            Err(error) => return Err(FailureBundleError::Io(error)),
        }
    }
    let canonical = fs::canonicalize(path)?;
    if !canonical.starts_with(canonical_root) {
        return Err(FailureBundleError::Io(io::Error::other(
            "failure evidence root escaped the repository",
        )));
    }
    Ok(canonical)
}

fn enforce_size(field: &'static str, bytes: &[u8], limit: usize) -> Result<(), FailureBundleError> {
    if bytes.len() <= limit {
        return Ok(());
    }
    Err(FailureBundleError::SizeLimit { field, limit })
}

fn create_bundle_directory(
    root: &Path,
    request_id: &RequestId,
    result_kind: &str,
) -> Result<PathBuf, FailureBundleError> {
    let stem = format!("{}-{result_kind}", request_id.as_str());
    for sequence in 0..MAXIMUM_BUNDLE_ATTEMPTS {
        let name = if sequence == 0 {
            stem.clone()
        } else {
            format!("{stem}-{sequence}")
        };
        let directory = root.join(name);
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(FailureBundleError::Io(error)),
        }
    }
    Err(FailureBundleError::DirectoryExhausted {
        root: root.to_path_buf(),
    })
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), FailureBundleError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

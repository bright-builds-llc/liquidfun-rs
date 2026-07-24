//! Confined repository and runtime paths for Phase 12 performance evidence.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const MAXIMUM_EVIDENCE_FILE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct PerformancePaths {
    repository_root: PathBuf,
    output_root: PathBuf,
}

impl PerformancePaths {
    pub(crate) fn production(repository_root: &Path) -> Result<Self, String> {
        let output_root = repository_root.join("target/phase12-performance");
        Self::new(repository_root, &output_root)
    }

    pub(crate) fn new(repository_root: &Path, output_root: &Path) -> Result<Self, String> {
        let repository_root = repository_root
            .canonicalize()
            .map_err(|error| format!("failed to resolve repository root: {error}"))?;
        let target_root = repository_root.join("target");
        fs::create_dir_all(&target_root)
            .map_err(|error| format!("failed to prepare target directory: {error}"))?;
        let target_root = target_root
            .canonicalize()
            .map_err(|error| format!("failed to resolve target directory: {error}"))?;
        reject_symlink_components(output_root)?;
        let normalized_output = normalize_new_path(output_root)?;
        if !normalized_output.starts_with(&target_root) {
            return Err("performance output must remain below target/".to_owned());
        }
        Ok(Self {
            repository_root,
            output_root: normalized_output,
        })
    }

    pub(crate) fn repository_root(&self) -> &Path {
        &self.repository_root
    }

    pub(crate) fn output_root(&self) -> &Path {
        &self.output_root
    }

    pub(crate) fn policy(&self) -> PathBuf {
        self.repository_root
            .join("reference/performance/policy.json")
    }

    pub(crate) fn manifest(&self) -> PathBuf {
        self.repository_root
            .join("reference/performance/manifest.toml")
    }

    pub(crate) fn upstream_lock(&self) -> PathBuf {
        self.repository_root.join("reference/upstream-lock.toml")
    }

    pub(crate) fn raw_directory(&self) -> PathBuf {
        self.output_root.join("raw")
    }

    pub(crate) fn optimization_record(&self) -> PathBuf {
        self.output_root.join("optimization-record.json")
    }

    pub(crate) fn calibration(&self) -> PathBuf {
        self.output_root.join("calibration.json")
    }
}

pub(crate) fn read_bounded_regular_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{} must be a regular non-symlink file",
            path.display()
        ));
    }
    if metadata.len() > MAXIMUM_EVIDENCE_FILE_BYTES {
        return Err(format!(
            "{} exceeds the evidence size limit",
            path.display()
        ));
    }
    fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

pub(crate) fn write_json_atomically(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    reject_symlink_components(parent)?;
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize evidence: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("failed to publish {}: {error}", path.display()))
}

fn normalize_new_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()));
    }
    let Some(parent) = path.parent() else {
        return Err(format!("{} has no parent", path.display()));
    };
    let parent = if parent.exists() {
        parent
            .canonicalize()
            .map_err(|error| format!("failed to resolve {}: {error}", parent.display()))?
    } else {
        normalize_new_path(parent)?
    };
    let Some(name) = path.file_name() else {
        return Err(format!("{} has no file name", path.display()));
    };
    Ok(parent.join(name))
}

fn reject_symlink_components(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("{} must not contain symlinks", path.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("failed to inspect {}: {error}", current.display()));
            }
        }
    }
    Ok(())
}

use std::{
    collections::BTreeSet,
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::Phase10EvidenceError;

pub(super) const MAXIMUM_JSON_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const MAXIMUM_LOG_BYTES: u64 = 32 * 1024 * 1024;
const MAXIMUM_FILES: usize = 256;
const MAXIMUM_DEPTH: usize = 6;

pub(super) fn checked_relative_path(value: &str) -> Result<PathBuf, Phase10EvidenceError> {
    let path = PathBuf::from(value);
    if path.is_absolute()
        || !path.starts_with("target")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase10EvidenceError::new(
            "usage",
            format!("path `{value}` must be normalized, relative, and under target/"),
        ));
    }
    Ok(path)
}

pub(super) fn checked_payload_path(value: &str) -> Result<&Path, Phase10EvidenceError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || path.is_absolute()
        || path.components().count() > MAXIMUM_DEPTH
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase10EvidenceError::new(
            "path",
            format!("unsafe evidence path `{value}`"),
        ));
    }
    Ok(path)
}

pub(super) fn repository_root() -> Result<PathBuf, Phase10EvidenceError> {
    let current = std::env::current_dir()
        .map_err(|error| Phase10EvidenceError::new("root", error.to_string()))?;
    let root = current
        .ancestors()
        .find(|candidate| candidate.join("Cargo.toml").is_file())
        .ok_or_else(|| Phase10EvidenceError::new("root", "workspace root not found"))?;
    fs::canonicalize(root).map_err(|error| Phase10EvidenceError::new("root", error.to_string()))
}

pub(super) fn resolve_target_path(
    repository_root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase10EvidenceError> {
    let suffix = relative
        .strip_prefix("target")
        .map_err(|_| Phase10EvidenceError::new(label, "path is outside target"))?;
    let target = resolve_descendant(repository_root, Path::new("target"), label)?;
    let result = resolve_descendant(&target, suffix, label)?;
    if !result.starts_with(&target) {
        return Err(Phase10EvidenceError::new(label, "path escapes target"));
    }
    Ok(result)
}

pub(super) fn resolve_descendant(
    root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase10EvidenceError> {
    reject_symlink(root, label)?;
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| Phase10EvidenceError::new(label, error.to_string()))?;
    let mut current = canonical_root.clone();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(Phase10EvidenceError::new(label, "unsafe path component"));
        };
        current.push(component);
        reject_symlink(&current, label)?;
    }
    let canonical = fs::canonicalize(&current)
        .map_err(|error| Phase10EvidenceError::new(label, error.to_string()))?;
    if !canonical.starts_with(canonical_root) {
        return Err(Phase10EvidenceError::new(
            label,
            "canonical path escapes root",
        ));
    }
    Ok(canonical)
}

fn reject_symlink(path: &Path, label: &'static str) -> Result<(), Phase10EvidenceError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Phase10EvidenceError::new(label, error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(Phase10EvidenceError::new(
            label,
            format!("symlink component `{}` is forbidden", path.display()),
        ));
    }
    Ok(())
}

pub(super) fn regular_files(root: &Path) -> Result<BTreeSet<String>, Phase10EvidenceError> {
    let mut files = BTreeSet::new();
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    while let Some((directory, prefix)) = pending.pop() {
        for entry in fs::read_dir(directory)
            .map_err(|error| Phase10EvidenceError::new("files", error.to_string()))?
        {
            let entry =
                entry.map_err(|error| Phase10EvidenceError::new("files", error.to_string()))?;
            let relative = prefix.join(entry.file_name());
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| Phase10EvidenceError::new("files", error.to_string()))?;
            if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
                return Err(Phase10EvidenceError::new(
                    "files",
                    format!("non-regular entry `{}` is forbidden", relative.display()),
                ));
            }
            if metadata.is_dir() {
                if relative.components().count() >= MAXIMUM_DEPTH {
                    return Err(Phase10EvidenceError::new(
                        "files",
                        "directory depth exceeds bound",
                    ));
                }
                pending.push((entry.path(), relative));
            } else {
                let path = relative
                    .to_str()
                    .ok_or_else(|| Phase10EvidenceError::new("files", "non-UTF-8 path"))?;
                files.insert(path.to_owned());
                if files.len() > MAXIMUM_FILES {
                    return Err(Phase10EvidenceError::new(
                        "files",
                        "file count exceeds bound",
                    ));
                }
            }
        }
    }
    Ok(files)
}

pub(super) fn read_regular_file(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<Vec<u8>, Phase10EvidenceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        let message = if error.kind() == ErrorKind::NotFound {
            "is missing".to_owned()
        } else {
            error.to_string()
        };
        Phase10EvidenceError::new(label, format!("{} {message}", path.display()))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(Phase10EvidenceError::new(
            label,
            format!("{} must be a bounded regular file", path.display()),
        ));
    }
    fs::read(path).map_err(|error| Phase10EvidenceError::new(label, error.to_string()))
}

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &'static str,
) -> Result<T, Phase10EvidenceError> {
    let bytes = read_regular_file(path, label, MAXIMUM_JSON_BYTES)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| Phase10EvidenceError::new(label, error.to_string()))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn canonical_sha256(value: &impl Serialize) -> Result<String, Phase10EvidenceError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| Phase10EvidenceError::new("json", error.to_string()))
}

pub(super) fn require_digest(
    label: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), Phase10EvidenceError> {
    if !is_sha256(expected) || expected != actual {
        return Err(Phase10EvidenceError::new(
            "digest",
            format!("{label} SHA-256 mismatch"),
        ));
    }
    Ok(())
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

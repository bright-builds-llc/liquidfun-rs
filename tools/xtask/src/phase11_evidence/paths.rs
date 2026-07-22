use std::{
    collections::{BTreeSet, HashSet},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::Phase11EvidenceError;

pub(super) const MAX_JSON_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const MAX_ARCHIVE_BYTES: u64 = 32 * 1024 * 1024;
pub(super) const MAX_FILES: usize = 64;
pub(super) const MAX_DEPTH: usize = 4;

pub(super) fn repository_root() -> Result<PathBuf, Phase11EvidenceError> {
    let current = std::env::current_dir()
        .map_err(|error| Phase11EvidenceError::new("root", error.to_string()))?;
    let root = current
        .ancestors()
        .find(|candidate| candidate.join("crates/liquidfun/Cargo.toml").is_file())
        .ok_or_else(|| Phase11EvidenceError::new("root", "workspace root not found"))?;
    canonical_regular_directory(root, "root")
}

pub(super) fn checked_input_path(value: &str) -> Result<PathBuf, Phase11EvidenceError> {
    let path = PathBuf::from(value);
    if value.is_empty()
        || value.contains('\\')
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase11EvidenceError::new(
            "usage",
            format!("path `{value}` must be normalized and repository-relative"),
        ));
    }
    Ok(path)
}

pub(super) fn checked_target_path(value: &str) -> Result<PathBuf, Phase11EvidenceError> {
    let path = checked_input_path(value)?;
    if !path.starts_with("target") {
        return Err(Phase11EvidenceError::new(
            "usage",
            format!("path `{value}` must remain under target/"),
        ));
    }
    Ok(path)
}

pub(super) fn resolve_input(
    root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase11EvidenceError> {
    resolve_descendant(root, relative, label)
}

pub(super) fn resolve_descendant(
    root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase11EvidenceError> {
    let canonical_root = canonical_regular_directory(root, label)?;
    let mut current = canonical_root.clone();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(Phase11EvidenceError::new(label, "unsafe path component"));
        };
        current.push(component);
        reject_symlink(&current, label)?;
    }
    let canonical = fs::canonicalize(&current)
        .map_err(|error| Phase11EvidenceError::new(label, error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(Phase11EvidenceError::new(label, "path escapes its root"));
    }
    Ok(canonical)
}

fn canonical_regular_directory(
    path: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase11EvidenceError> {
    reject_symlink(path, label)?;
    let canonical = fs::canonicalize(path)
        .map_err(|error| Phase11EvidenceError::new(label, error.to_string()))?;
    if !canonical.is_dir() {
        return Err(Phase11EvidenceError::new(label, "expected a directory"));
    }
    Ok(canonical)
}

fn reject_symlink(path: &Path, label: &'static str) -> Result<(), Phase11EvidenceError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Phase11EvidenceError::new(label, error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(Phase11EvidenceError::new(
            label,
            format!("symlink `{}` is forbidden", path.display()),
        ));
    }
    Ok(())
}

pub(super) fn read_regular(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<Vec<u8>, Phase11EvidenceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        let message = if error.kind() == ErrorKind::NotFound {
            "is missing".to_owned()
        } else {
            error.to_string()
        };
        Phase11EvidenceError::new(label, format!("{} {message}", path.display()))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(Phase11EvidenceError::new(
            label,
            format!("{} must be a bounded regular file", path.display()),
        ));
    }
    fs::read(path).map_err(|error| Phase11EvidenceError::new(label, error.to_string()))
}

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &'static str,
) -> Result<T, Phase11EvidenceError> {
    let bytes = read_regular(path, label, MAX_JSON_BYTES)?;
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
    let value = T::deserialize(&mut deserializer)
        .map_err(|error| Phase11EvidenceError::new(label, error.to_string()))?;
    deserializer
        .end()
        .map_err(|error| Phase11EvidenceError::new(label, error.to_string()))?;
    Ok(value)
}

pub(super) fn regular_files(root: &Path) -> Result<BTreeSet<String>, Phase11EvidenceError> {
    let mut result = BTreeSet::new();
    let mut folded = HashSet::new();
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    while let Some((directory, prefix)) = pending.pop() {
        for entry in fs::read_dir(directory)
            .map_err(|error| Phase11EvidenceError::new("files", error.to_string()))?
        {
            let entry =
                entry.map_err(|error| Phase11EvidenceError::new("files", error.to_string()))?;
            let relative = prefix.join(entry.file_name());
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| Phase11EvidenceError::new("files", error.to_string()))?;
            if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
                return Err(Phase11EvidenceError::new(
                    "files",
                    format!("non-regular entry `{}` is forbidden", relative.display()),
                ));
            }
            if metadata.is_dir() {
                if relative.components().count() >= MAX_DEPTH {
                    return Err(Phase11EvidenceError::new(
                        "files",
                        "directory depth exceeds bound",
                    ));
                }
                pending.push((entry.path(), relative));
                continue;
            }
            let value = relative
                .to_str()
                .ok_or_else(|| Phase11EvidenceError::new("files", "non-UTF-8 path"))?
                .to_owned();
            if !folded.insert(value.to_ascii_lowercase()) {
                return Err(Phase11EvidenceError::new(
                    "files",
                    "duplicate or case-colliding paths are forbidden",
                ));
            }
            result.insert(value);
            if result.len() > MAX_FILES {
                return Err(Phase11EvidenceError::new(
                    "files",
                    "file count exceeds bound",
                ));
            }
        }
    }
    Ok(result)
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn canonical_sha256(value: &impl Serialize) -> Result<String, Phase11EvidenceError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| Phase11EvidenceError::new("json", error.to_string()))
}

pub(super) fn require_sha256(
    label: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), Phase11EvidenceError> {
    if !is_sha256(expected) || expected != actual {
        return Err(Phase11EvidenceError::new(
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

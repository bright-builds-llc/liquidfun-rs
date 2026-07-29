use super::{
    BTreeSet, Component, Digest, MAXIMUM_ARTIFACT_BYTES, MAXIMUM_MANIFEST_BYTES, Path, PathBuf,
    ReleaseError, Sha256, fs,
};

pub(super) fn ids(values: &[serde_json::Value]) -> Result<BTreeSet<&str>, ReleaseError> {
    values
        .iter()
        .map(|value| {
            value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| ReleaseError::new("compatibility-closure", "identity is absent"))
        })
        .collect()
}

pub(super) fn validate_repository_authorities(repository_root: &Path) -> Result<(), ReleaseError> {
    require_repository_files(
        repository_root,
        &[
            "reference/platform/support.json",
            "reference/performance/manifest.toml",
            "reference/coverage/contract.json",
            "reference/regressions/manifest.toml",
            "reference/upstream-corpus.json",
            "reference/compatibility.json",
        ],
        "authority",
    )
}

pub(super) fn platform_support(repository_root: &Path) -> Result<serde_json::Value, ReleaseError> {
    let bytes = read_confined_regular(
        repository_root,
        Path::new("reference/platform/support.json"),
        MAXIMUM_MANIFEST_BYTES,
        "platform",
    )?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| ReleaseError::new("platform", error.to_string()))?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
        || value
            .get("evidence_tier")
            .and_then(serde_json::Value::as_str)
            != Some("d2_supported")
    {
        return Err(ReleaseError::new(
            "platform",
            "tracked platform authority is invalid",
        ));
    }
    Ok(value)
}

pub(super) fn require_repository_files(
    repository_root: &Path,
    paths: &[&str],
    category: &'static str,
) -> Result<(), ReleaseError> {
    for relative in paths {
        let _bytes = read_confined_regular(
            repository_root,
            Path::new(relative),
            MAXIMUM_ARTIFACT_BYTES,
            category,
        )?;
    }
    Ok(())
}

pub(super) fn read_input_manifest(
    repository_root: &Path,
    manifest_path: &Path,
) -> Result<Vec<u8>, ReleaseError> {
    let path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repository_root.join(manifest_path)
    };
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|error| ReleaseError::new("manifest-path", error.to_string()))?;
    reject_symlink_components(&path, "manifest-path")?;
    let canonical = fs::canonicalize(&path)
        .map_err(|error| ReleaseError::new("manifest-path", error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ReleaseError::new(
            "manifest-path",
            "manifest escaped the repository",
        ));
    }
    read_regular(&canonical, MAXIMUM_MANIFEST_BYTES, "manifest-path")
}

pub(super) fn read_confined_regular(
    repository_root: &Path,
    relative: &Path,
    maximum_bytes: u64,
    category: &'static str,
) -> Result<Vec<u8>, ReleaseError> {
    let relative = normalized_relative_path(relative, category)?;
    let path = repository_root.join(relative);
    reject_symlink_components(&path, category)?;
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    let canonical =
        fs::canonicalize(&path).map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ReleaseError::new(category, "path escaped the repository"));
    }
    read_regular(&canonical, maximum_bytes, category)
}

pub(super) fn read_regular(
    path: &Path,
    maximum_bytes: u64,
    category: &'static str,
) -> Result<Vec<u8>, ReleaseError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(ReleaseError::new(
            category,
            "input must be a bounded ordinary file",
        ));
    }
    fs::read(path).map_err(|error| ReleaseError::new(category, error.to_string()))
}

pub(super) fn reject_symlink_components(
    path: &Path,
    category: &'static str,
) -> Result<(), ReleaseError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            return Err(ReleaseError::new(category, "path contains a symbolic link"));
        }
    }
    Ok(())
}

pub(super) fn normalized_relative<'a>(
    value: &'a str,
    category: &'static str,
) -> Result<&'a Path, ReleaseError> {
    normalized_relative_path(Path::new(value), category)
}

fn normalized_relative_path<'a>(
    path: &'a Path,
    category: &'static str,
) -> Result<&'a Path, ReleaseError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ReleaseError::new(
            category,
            "path must be normalized and repository-relative",
        ));
    }
    Ok(path)
}

pub(super) fn require_sha256(value: &str, category: &'static str) -> Result<(), ReleaseError> {
    if !is_sha256(value) {
        return Err(ReleaseError::new(category, "SHA-256 identity is invalid"));
    }
    Ok(())
}

pub(super) fn is_run_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && !value.starts_with('0')
}

pub(super) fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

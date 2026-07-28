use super::{
    BTreeSet, Component, Deserialize, Digest, ErrorKind, EvidenceKind, EvidenceManifest, ExactRun,
    IDENTITY_FILE, INVENTORY_FILE, MANIFEST_FILE, MAXIMUM_LOG_BYTES, PROVENANCE_FILE, Path,
    PathBuf, Phase9EvidenceError, READ_ONLY_FILE, Serialize, Sha256, TRACE_FILE, UPSTREAM_REVISION,
    cross_run_payload_refs, fs, resolve_existing_descendant,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceIdentity {
    run_id: u64,
    pub(super) job: String,
    head_sha: String,
    upstream_revision: String,
    rust: String,
    cmake: String,
    ninja: String,
    clang: String,
    target: String,
    policy: String,
    trace: IdentityFile,
    manifest: IdentityFile,
    files: Vec<IdentityFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityFile {
    path: String,
    sha256: String,
}

pub(super) fn validate_identity(
    root: &Path,
    kind: EvidenceKind,
    identity: &EvidenceIdentity,
    maybe_run: Option<&ExactRun>,
    denied_run_ids: &BTreeSet<u64>,
    manifest: &EvidenceManifest,
) -> Result<(), Phase9EvidenceError> {
    if denied_run_ids.contains(&identity.run_id) {
        return Err(Phase9EvidenceError::new(
            "identity",
            format!("identity run {} is denylisted", identity.run_id),
        ));
    }
    let exact_ref = maybe_run.is_some();
    if identity.job != kind.identity_job(exact_ref)
        || identity.upstream_revision != UPSTREAM_REVISION
        || identity.rust != "1.97.0"
        || identity.cmake != "4.3.3"
        || identity.ninja != "1.13.2"
        || identity.clang != "22.1.8"
        || identity.target != "x86_64-unknown-linux-gnu"
        || identity.policy != "phase9-v1"
        || manifest.profile != identity.policy
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity does not match the reviewed job, toolchain, target, or policy",
        ));
    }
    if let Some(run) = maybe_run {
        if identity.run_id != run.run_id || identity.head_sha != run.approved_sha {
            return Err(Phase9EvidenceError::new(
                "identity",
                "identity does not match exact-ref run and approved head",
            ));
        }
    } else if identity.run_id != 0 || identity.head_sha != "local" {
        return Err(Phase9EvidenceError::new(
            "identity",
            "local evidence must use run 0 and local head identity",
        ));
    }
    if identity.trace.path != TRACE_FILE || identity.manifest.path != MANIFEST_FILE {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity trace or manifest path mismatch",
        ));
    }
    require_file_digest(root, &identity.trace)?;
    require_file_digest(root, &identity.manifest)
}

pub(super) fn validate_exact_file_set(
    root: &Path,
    manifest: &EvidenceManifest,
    identity: &EvidenceIdentity,
) -> Result<(), Phase9EvidenceError> {
    let mut expected = BTreeSet::from([
        IDENTITY_FILE.to_owned(),
        MANIFEST_FILE.to_owned(),
        TRACE_FILE.to_owned(),
        PROVENANCE_FILE.to_owned(),
        INVENTORY_FILE.to_owned(),
        READ_ONLY_FILE.to_owned(),
    ]);
    for case in &manifest.cases {
        expected.insert(case.request_path.clone());
        expected.insert(case.native_result_path.clone());
        expected.insert(case.oracle_result_path.clone());
        expected.insert(case.complete_comparison_path.clone());
        expected.extend(
            cross_run_payload_refs(&case.cross_run_proofs)
                .into_iter()
                .map(|reference| reference.path.to_string()),
        );
    }
    let actual = regular_files(root)?;
    if actual != expected {
        return Err(Phase9EvidenceError::new(
            "files",
            format!("evidence regular-file set mismatch: expected {expected:?}, actual {actual:?}"),
        ));
    }
    let expected_identity_files = expected
        .iter()
        .filter(|path| path.as_str() != IDENTITY_FILE)
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_identity_files = identity
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    if identity.files.len() != actual_identity_files.len()
        || actual_identity_files != expected_identity_files
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity file inventory is incomplete, duplicated, or substituted",
        ));
    }
    for file in &identity.files {
        require_file_digest(root, file)?;
    }
    Ok(())
}

pub(super) fn regular_files(root: &Path) -> Result<BTreeSet<String>, Phase9EvidenceError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Phase9EvidenceError::new(
            "files",
            "evidence root must be an ordinary directory",
        ));
    }
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    let mut files = BTreeSet::new();
    while let Some((directory, relative_directory)) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?
        {
            let entry =
                entry.map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
            let name = entry.file_name();
            let relative = relative_directory.join(name);
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
            if metadata.file_type().is_symlink() {
                return Err(Phase9EvidenceError::new(
                    "files",
                    format!("symlink `{}` is forbidden", relative.display()),
                ));
            }
            if metadata.is_dir() {
                pending.push((entry.path(), relative));
            } else if metadata.is_file() {
                files.insert(
                    relative
                        .to_str()
                        .ok_or_else(|| {
                            Phase9EvidenceError::new("files", "non-UTF-8 evidence path")
                        })?
                        .to_owned(),
                );
            } else {
                return Err(Phase9EvidenceError::new(
                    "files",
                    format!("non-regular entry `{}` is forbidden", relative.display()),
                ));
            }
        }
    }
    Ok(files)
}

pub(super) fn validate_trace(path: &Path) -> Result<(), Phase9EvidenceError> {
    let bytes = read_regular_file(path, "trace", MAXIMUM_LOG_BYTES)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| Phase9EvidenceError::new("trace", error.to_string()))?;
    if !text.contains("test result: ok.") || text.contains("FAILED") {
        return Err(Phase9EvidenceError::new(
            "trace",
            "trace lacks a passing marker or contains FAILED",
        ));
    }
    Ok(())
}

fn require_file_digest(root: &Path, file: &IdentityFile) -> Result<(), Phase9EvidenceError> {
    let relative = Path::new(&file.path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            format!("unsafe identity file path `{}`", file.path),
        ));
    }
    let path = resolve_existing_descendant(root, relative, "identity")?;
    let bytes = read_regular_file(&path, "identity file", MAXIMUM_LOG_BYTES)?;
    require_digest("identity file", &file.sha256, &sha256(&bytes))
}

pub(super) fn require_digest(
    label: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), Phase9EvidenceError> {
    if !is_sha256(expected) || expected != actual {
        return Err(Phase9EvidenceError::new(
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

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn canonical_sha256(value: &impl Serialize) -> Result<String, Phase9EvidenceError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| Phase9EvidenceError::new("json", error.to_string()))
}

pub(super) fn read_json_absolute<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<T, Phase9EvidenceError> {
    let bytes = read_regular_file(path, label, maximum)?;
    parse_json_bytes(&bytes, label)
}

pub(super) fn parse_json_bytes<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    label: &'static str,
) -> Result<T, Phase9EvidenceError> {
    serde_json::from_slice(bytes)
        .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))
}

pub(super) fn read_regular_file(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<Vec<u8>, Phase9EvidenceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        let detail = if error.kind() == ErrorKind::NotFound {
            "is missing".to_owned()
        } else {
            error.to_string()
        };
        Phase9EvidenceError::new(label, format!("{} {detail}", path.display()))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(Phase9EvidenceError::new(
            label,
            format!("{} must be a bounded regular file", path.display()),
        ));
    }
    fs::read(path).map_err(|error| Phase9EvidenceError::new(label, error.to_string()))
}

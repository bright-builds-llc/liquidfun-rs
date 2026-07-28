use super::{
    Acquisition, BTreeMap, BTreeSet, ClosureReview, Command, Component, Deserialize, Digest,
    NEXT_STAGE, OpenOptions, Ordering, Output, Path, PathBuf, PromotionError, PromotionErrorKind,
    ReviewPacket, Serialize, Sha256, Write, env, fs,
};

#[derive(Serialize)]
struct ReviewSubject<'a> {
    schema_version: u32,
    producer_sha: &'a str,
    bundle_sha256: &'a str,
    promotion_base_sha: &'a str,
    acquisition: &'a Acquisition,
    witness_closure: &'a ClosureReview,
    replay_closure: &'a ClosureReview,
    promoted_paths: &'a [String],
    promoted_path_set_sha256: &'a str,
    promoted_content_sha256: &'a str,
    changed_paths: &'a [String],
    unchanged_paths: &'a [String],
    changed_path_set_sha256: &'a str,
    changed_content_sha256: &'a str,
    replacement_sha256: &'a BTreeMap<String, String>,
    diff: &'a str,
}

pub(super) fn review_sha256(packet: &ReviewPacket) -> Result<String, PromotionError> {
    let subject = ReviewSubject {
        schema_version: 2,
        producer_sha: &packet.producer_sha,
        bundle_sha256: &packet.bundle_sha256,
        promotion_base_sha: &packet.promotion_base_sha,
        acquisition: &packet.acquisition,
        witness_closure: &packet.witness_closure,
        replay_closure: &packet.replay_closure,
        promoted_paths: &packet.promoted_paths,
        promoted_path_set_sha256: &packet.promoted_path_set_sha256,
        promoted_content_sha256: &packet.promoted_content_sha256,
        changed_paths: &packet.changed_paths,
        unchanged_paths: &packet.unchanged_paths,
        changed_path_set_sha256: &packet.changed_path_set_sha256,
        changed_content_sha256: &packet.changed_content_sha256,
        replacement_sha256: &packet.replacement_sha256,
        diff: &packet.diff,
    };
    serde_json::to_vec(&subject)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Schema,
                format!("failed to encode review subject: {error}"),
            )
        })
}

#[allow(
    dead_code,
    reason = "integration contract tests hash a packet subject without running promotion"
)]
pub(crate) fn review_sha256_for_test(packet: &ReviewPacket) -> Result<String, PromotionError> {
    review_sha256(packet)
}

pub(super) fn collect_regular_files(root: &Path) -> Result<BTreeSet<String>, PromotionError> {
    let mut pending = vec![root.to_path_buf()];
    let mut owned = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        reject_symlink(&directory)?;
        for entry in fs::read_dir(&directory).map_err(filesystem_error)? {
            let entry = entry.map_err(filesystem_error)?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(filesystem_error)?;
            if metadata.file_type().is_symlink() {
                return Err(PromotionError::new(
                    PromotionErrorKind::Path,
                    "staging tree contains a symbolic link",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_error| {
                    PromotionError::new(
                        PromotionErrorKind::Path,
                        "staged file escaped the staging root",
                    )
                })?;
                owned.insert(path_text(relative)?);
            } else {
                return Err(PromotionError::new(
                    PromotionErrorKind::Path,
                    "staging tree contains a non-regular entry",
                ));
            }
        }
    }
    Ok(owned)
}

pub(super) fn new_staging_root(
    repository_root: &Path,
    promotion_base_sha: &str,
) -> Result<PathBuf, PromotionError> {
    let parent = repository_root.join("target/phase13/promotion-staged");
    fs::create_dir_all(&parent).map_err(filesystem_error)?;
    reject_symlink(&parent)?;
    let ordinal = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
    let root = parent.join(format!(
        "{promotion_base_sha}-{}-{ordinal}",
        std::process::id()
    ));
    fs::create_dir(&root).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!("failed to create new promotion staging root: {error}"),
        )
    })?;
    Ok(root)
}

pub(super) fn require_clean_worktree(repository_root: &Path) -> Result<(), PromotionError> {
    let status = git_text(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if status.is_empty() {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Git,
        "promotion preparation requires a completely clean worktree",
    ))
}

pub(super) fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, PromotionError> {
    if !args.len().is_multiple_of(2) {
        return Err(PromotionError::usage(
            "every option requires exactly one value",
        ));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(PromotionError::usage(
                "options must be unique option/value pairs",
            ));
        }
    }
    Ok(options)
}

pub(super) fn require_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
) -> Result<(), PromotionError> {
    if options.len() == required_names.len()
        && required_names
            .iter()
            .all(|name| options.contains_key(*name))
    {
        return Ok(());
    }
    Err(PromotionError::usage(
        "command options do not match the closed contract",
    ))
}

pub(super) fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, PromotionError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| PromotionError::usage(format!("missing `{name}`")))
}

pub(super) fn validate_reviewer_id(value: &str) -> Result<(), PromotionError> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
        && !value.to_ascii_lowercase().contains("codex");
    if valid {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Acknowledgement,
        "reviewer ID must identify an independent human",
    ))
}

pub(super) fn validate_relative_path(value: &str) -> Result<(), PromotionError> {
    if !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Path,
        format!("unsafe relative path `{value}`"),
    ))
}

pub(super) fn repository_root() -> Result<PathBuf, PromotionError> {
    let current = env::current_dir().map_err(filesystem_error)?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/liquidfun/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Filesystem,
                "repository root is unavailable",
            )
        })
}

pub(super) fn absolute_path(repository_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository_root.join(path)
    }
}

pub(super) fn relative_path_text(
    repository_root: &Path,
    path: &Path,
) -> Result<String, PromotionError> {
    path.strip_prefix(repository_root)
        .map_err(|_error| {
            PromotionError::new(
                PromotionErrorKind::Path,
                "generated path is outside the repository",
            )
        })
        .and_then(path_text)
}

pub(super) fn git_text(repository_root: &Path, args: &[&str]) -> Result<String, PromotionError> {
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(args),
        "query Git identity",
    )?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

pub(super) fn git_success(repository_root: &Path, args: &[&str]) -> Result<bool, PromotionError> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(args)
        .status()
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("failed to query Git ancestry: {error}"),
            )
        })?;
    Ok(status.success())
}

pub(super) fn git_file(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Vec<u8>, PromotionError> {
    git_maybe_file(repository_root, revision, path)?.ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Git,
            format!("`{path}` is absent at revision `{revision}`"),
        )
    })
}

pub(super) fn git_maybe_file(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Option<Vec<u8>>, PromotionError> {
    let object = format!("{revision}:{path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["show", &object])
        .output()
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("failed to read Git object `{object}`: {error}"),
            )
        })?;
    if output.status.success() {
        return Ok(Some(output.stdout));
    }
    if output.status.code() == Some(128) {
        return Ok(None);
    }
    Err(PromotionError::new(
        PromotionErrorKind::Git,
        format!(
            "failed to read Git object `{object}`: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    ))
}

pub(super) fn run_process(command: &mut Command, action: &str) -> Result<Output, PromotionError> {
    let output = command.output().map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!("failed to {action}: {error}"),
        )
    })?;
    if !output.status.success() {
        return Err(PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!(
                "failed to {action} with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output)
}

pub(super) fn write_json(
    path: &Path,
    value: &impl Serialize,
    create_new: bool,
) -> Result<(), PromotionError> {
    let bytes = json_bytes(value)?;
    if create_new {
        return write_new_file(path, &bytes);
    }
    let parent = path.parent().ok_or_else(|| {
        PromotionError::new(PromotionErrorKind::Path, "JSON output has no parent")
    })?;
    fs::create_dir_all(parent).map_err(filesystem_error)?;
    fs::write(path, bytes).map_err(filesystem_error)
}

pub(super) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), PromotionError> {
    let parent = path.parent().ok_or_else(|| {
        PromotionError::new(PromotionErrorKind::Path, "output file has no parent")
    })?;
    fs::create_dir_all(parent).map_err(filesystem_error)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(filesystem_error)?;
    file.write_all(bytes).map_err(filesystem_error)
}

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, PromotionError> {
    let bytes = fs::read(path).map_err(filesystem_error)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("invalid JSON `{}`: {error}", path.display()),
        )
    })
}

pub(super) fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, PromotionError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("failed to encode JSON: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), PromotionError> {
    let metadata = fs::symlink_metadata(path).map_err(filesystem_error)?;
    if metadata.file_type().is_symlink() {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            format!("symbolic link is forbidden: {}", path.display()),
        ));
    }
    Ok(())
}

pub(super) fn file_sha256(path: &Path) -> Result<String, PromotionError> {
    fs::read(path)
        .map(|bytes| sha256(&bytes))
        .map_err(filesystem_error)
}

pub(super) fn path_text(path: &Path) -> Result<String, PromotionError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| PromotionError::new(PromotionErrorKind::Path, "path is not valid UTF-8"))
}

pub(super) fn filesystem_error(error: impl std::fmt::Display) -> PromotionError {
    PromotionError::new(PromotionErrorKind::Filesystem, error.to_string())
}

pub(super) fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(lower_hex)
}

pub(super) fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(lower_hex)
}

pub(super) fn lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

pub(super) fn valid_utc_timestamp(value: &str) -> bool {
    value.len() == 20
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value.as_bytes().get(10) == Some(&b'T')
        && value.as_bytes().get(13) == Some(&b':')
        && value.as_bytes().get(16) == Some(&b':')
        && value.ends_with('Z')
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn update_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}

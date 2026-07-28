use super::execution::{git_file, git_file_exists, git_text, successful_output};
use super::identity::{MaterialsManifest, Receipt};
use super::{
    AcceptanceError, AcceptanceErrorKind, BTreeMap, BTreeSet, Command, Digest, MATERIALS_MANIFEST,
    ORACLE_REVISION, PROMOTED_PATHS, Path, RECEIPT_PATH, REPLAY_REPOSITORY_PREFIXES, Sha256,
    WITNESS_REPOSITORY_PREFIXES, fs, sha256, update_field, validate_relative_path,
};

pub(super) fn derive_witness_closure(
    repository_root: &Path,
    revision: &str,
) -> Result<String, AcceptanceError> {
    let bytes = git_file(repository_root, revision, MATERIALS_MANIFEST)?;
    let manifest: MaterialsManifest = serde_json::from_slice(&bytes).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("invalid witness materials at {revision}: {error}"),
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != "oracle-debug"
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            "witness materials have the wrong schema, target, or preset",
        ));
    }
    let mut entries = git_entries(repository_root, revision, &WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let maybe_bytes = if material.identity.starts_with("third_party/liquidfun/") {
            require_pinned_upstream(repository_root, revision)?;
            Some(
                fs::read(repository_root.join(&material.identity))
                    .map_err(AcceptanceError::from)?,
            )
        } else if git_file_exists(repository_root, revision, &material.identity)? {
            Some(git_file(repository_root, revision, &material.identity)?)
        } else {
            None
        };
        if let Some(bytes) = maybe_bytes {
            entries.insert(material.identity, sha256(&bytes));
        }
    }
    closure_digest("witness", entries)
}

pub(super) fn derive_replay_closure(
    repository_root: &Path,
    revision: &str,
    promotion_base_sha: &str,
) -> Result<String, AcceptanceError> {
    let mut entries = git_entries(repository_root, revision, &REPLAY_REPOSITORY_PREFIXES)?;
    // Q's seven reviewed paths are promotion outputs, not fresh producer inputs. Project those
    // paths back to R while every other input remains read from the revision under acceptance.
    for path in PROMOTED_PATHS {
        if entries.contains_key(path) {
            let base_bytes = git_file(repository_root, promotion_base_sha, path)?;
            entries.insert(path.to_owned(), sha256(&base_bytes));
        }
    }
    closure_digest("replay", entries)
}

fn require_pinned_upstream(repository_root: &Path, revision: &str) -> Result<(), AcceptanceError> {
    let gitlink = git_text(
        repository_root,
        &["ls-tree", revision, "third_party/liquidfun"],
    )?;
    let expected = format!("160000 commit {ORACLE_REVISION}\tthird_party/liquidfun");
    let checked_out = git_text(
        &repository_root.join("third_party/liquidfun"),
        &["rev-parse", "HEAD"],
    )?;
    if gitlink != expected || checked_out != ORACLE_REVISION {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "witness closure requires the exact pinned and initialized oracle gitlink",
        ));
    }
    Ok(())
}

fn git_entries(
    repository_root: &Path,
    revision: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", revision, "--"])
        .args(prefixes);
    let output = successful_output(&mut command, "enumerate closure inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for name in names.lines() {
        validate_relative_path(name)?;
        entries.insert(
            name.to_owned(),
            sha256(&git_file(repository_root, revision, name)?),
        );
    }
    Ok(entries)
}

fn closure_digest(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<String, AcceptanceError> {
    if entries.is_empty() {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Closure,
            format!("{label} closure is empty"),
        ));
    }
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-closure-v1");
    update_field(&mut hasher, label.as_bytes());
    for (path, digest) in entries {
        update_field(&mut hasher, path.as_bytes());
        update_field(&mut hasher, digest.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn receipt_semantic_sha256(receipt: &[u8]) -> Result<String, AcceptanceError> {
    let mut normalized: Receipt = serde_json::from_slice(receipt).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("invalid promotion receipt for semantic hashing: {error}"),
        )
    })?;
    normalized.promoted_content_sha256.clear();
    normalized.changed_content_sha256.clear();
    let canonical = serde_json::to_vec(&normalized).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("failed to normalize promotion receipt: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-promotion-receipt-semantic-leaf-v2");
    update_field(&mut hasher, &canonical);
    Ok(format!("{:x}", hasher.finalize()))
}

fn reviewed_content_digest(
    domain: &str,
    root: &Path,
    paths: &[String],
) -> Result<String, AcceptanceError> {
    let unique_paths = paths.iter().collect::<BTreeSet<_>>();
    if unique_paths.len() != paths.len() {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "content digest path set contains duplicates",
        ));
    }
    let mut hasher = Sha256::new();
    update_field(&mut hasher, domain.as_bytes());
    for path in unique_paths {
        let bytes = fs::read(root.join(path)).map_err(AcceptanceError::from)?;
        let leaf = if path == RECEIPT_PATH {
            receipt_semantic_sha256(&bytes)?
        } else {
            sha256(&bytes)
        };
        update_field(&mut hasher, path.as_bytes());
        update_field(&mut hasher, leaf.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn reviewed_content_digests_from_root(
    root: &Path,
    changed_paths: &[String],
) -> Result<(String, String), AcceptanceError> {
    let promoted_paths = PROMOTED_PATHS.map(str::to_owned).to_vec();
    Ok((
        reviewed_content_digest("phase13-promoted-content-set-v2", root, &promoted_paths)?,
        reviewed_content_digest("phase13-changed-content-set-v2", root, changed_paths)?,
    ))
}

pub(super) fn promoted_path_set_sha256(paths: &[String]) -> Result<String, AcceptanceError> {
    let actual = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = PROMOTED_PATHS.into_iter().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "promoted path set is incomplete or contains extra paths",
        ));
    }
    Ok(path_set_sha256("phase13-promoted-path-set-v2", paths))
}

pub(super) fn changed_path_set_sha256(paths: &[String]) -> Result<String, AcceptanceError> {
    if paths.is_empty()
        || paths
            .iter()
            .any(|path| !PROMOTED_PATHS.contains(&path.as_str()))
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "changed path set is empty or outside the promoted set",
        ));
    }
    Ok(path_set_sha256("phase13-changed-path-set-v2", paths))
}

fn path_set_sha256(domain: &str, paths: &[String]) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, domain.as_bytes());
    for path in paths {
        update_field(&mut hasher, path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub(super) fn valid_path_classification(changed: &[String], unchanged: &[String]) -> bool {
    let changed = changed.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unchanged = unchanged
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let promoted = PROMOTED_PATHS.into_iter().collect::<BTreeSet<_>>();
    changed.len() + unchanged.len() == PROMOTED_PATHS.len()
        && changed.is_disjoint(&unchanged)
        && changed.union(&unchanged).copied().collect::<BTreeSet<_>>() == promoted
}

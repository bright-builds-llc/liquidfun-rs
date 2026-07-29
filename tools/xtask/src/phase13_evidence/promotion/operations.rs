use super::{
    Acquisition, BTreeMap, BTreeSet, BundleClosure, BundleManifest, CATALOG_PATH, ClosureReview,
    Command, Digest, NEXT_STAGE, Ordering, PRODUCER_SHA, PROMOTED_PATHS, PROVIDER_ARTIFACT_ID,
    PROVIDER_ARTIFACT_NAME, PROVIDER_DIGEST, PROVIDER_REPOSITORY, PROVIDER_RUN_ID, Path,
    PromotionError, PromotionErrorKind, ProviderArtifact, Sha256, filesystem_error, fs,
    git_maybe_file, read_json, relative_path_text, run_process, transaction, update_field,
    valid_utc_timestamp, validate_exact_paths, write_new_file,
};

#[allow(
    dead_code,
    reason = "integration tests inject a bounded transaction failure"
)]
pub(crate) fn replace_with_injected_failure(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
    maybe_fail_after: Option<usize>,
) -> Result<(), PromotionError> {
    transaction::replace_all(repository_root, staging_root, paths, maybe_fail_after)
}

#[allow(
    dead_code,
    reason = "integration tests inject a failing post-write validator"
)]
pub(crate) fn replace_with_failing_validation(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
) -> Result<(), PromotionError> {
    transaction::replace_all_and_validate(repository_root, staging_root, paths, None, || {
        Err(PromotionError::new(
            PromotionErrorKind::Transaction,
            "injected post-write validation failure",
        ))
    })
}

pub(super) fn write_replacements(
    staging_root: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<(), PromotionError> {
    let actual = replacements.keys().cloned().collect::<BTreeSet<_>>();
    let expected = PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    validate_exact_paths(&actual, &expected)?;
    for (path, bytes) in replacements {
        write_new_file(&staging_root.join(path), bytes)?;
    }
    Ok(())
}

pub(super) fn format_staged_catalog(staging_root: &Path) -> Result<(), PromotionError> {
    run_process(
        Command::new("rustfmt")
            .args(["--edition", "2024", "--config", "skip_children=true"])
            .arg(staging_root.join(CATALOG_PATH)),
        "format staged catalog binding",
    )
    .map(|_output| ())
}

pub(super) fn canonical_diff(
    repository_root: &Path,
    base_revision: &str,
    staging_root: &Path,
    paths: &[&str],
) -> Result<String, PromotionError> {
    let scratch = repository_root
        .join("target/phase13/promotion-diff")
        .join(format!(
            "{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&scratch).map_err(filesystem_error)?;
    let mut complete = String::new();
    for path in paths {
        let maybe_old = git_maybe_file(repository_root, base_revision, path)?;
        let staged = relative_path_text(repository_root, &staging_root.join(path))?;
        let old = if let Some(ref bytes) = maybe_old {
            let old_path = scratch.join(path);
            write_new_file(&old_path, bytes)?;
            relative_path_text(repository_root, &old_path)?
        } else {
            "/dev/null".to_owned()
        };
        let output = Command::new("git")
            .current_dir(repository_root)
            .args([
                "diff",
                "--no-index",
                "--no-ext-diff",
                "--text",
                "--src-prefix=a/",
                "--dst-prefix=b/",
                "--",
                &old,
                &staged,
            ])
            .output()
            .map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Diff,
                    format!("failed to render review diff: {error}"),
                )
            })?;
        if output.status.code() != Some(1) {
            return Err(PromotionError::new(
                PromotionErrorKind::Diff,
                format!("promoted path `{path}` did not produce one changed-file diff"),
            ));
        }
        let text = String::from_utf8(output.stdout).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Diff,
                format!("review diff is not UTF-8: {error}"),
            )
        })?;
        complete.push_str(&normalize_diff_headers(
            &text,
            &old,
            &staged,
            path,
            maybe_old.is_none(),
        ));
    }
    Ok(complete)
}

pub(super) fn normalize_diff_headers(
    diff: &str,
    old: &str,
    staged: &str,
    promoted_path: &str,
    is_new: bool,
) -> String {
    let old_label = if is_new {
        "/dev/null".to_owned()
    } else {
        format!("a/{promoted_path}")
    };
    diff.lines()
        .map(|line| {
            if line.starts_with("diff --git ") {
                return format!("diff --git a/{promoted_path} b/{promoted_path}");
            }
            if line == format!("--- a/{old}") || line == "--- /dev/null" {
                return format!("--- {old_label}");
            }
            if line == format!("+++ b/{staged}") {
                return format!("+++ b/{promoted_path}");
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub(super) fn acquire_provider_metadata() -> Result<Acquisition, PromotionError> {
    let endpoint = format!("repos/{PROVIDER_REPOSITORY}/actions/artifacts/{PROVIDER_ARTIFACT_ID}");
    let output = run_process(
        Command::new("gh").args(["api", &endpoint]),
        "read immutable artifact provider metadata",
    )?;
    let artifact: ProviderArtifact = serde_json::from_slice(&output.stdout).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Provider,
            format!("invalid provider artifact metadata: {error}"),
        )
    })?;
    let acquisition = Acquisition {
        repository: PROVIDER_REPOSITORY.to_owned(),
        run_id: artifact.workflow_run.id,
        artifact_id: artifact.id,
        artifact_name: artifact.name,
        provider_digest: artifact.digest,
        artifact_created_at: artifact.created_at,
        artifact_expires_at: artifact.expires_at,
    };
    if artifact.workflow_run.head_sha != PRODUCER_SHA {
        return Err(PromotionError::new(
            PromotionErrorKind::Provider,
            "provider artifact head SHA does not equal P",
        ));
    }
    validate_acquisition(&acquisition)?;
    Ok(acquisition)
}

pub(super) fn validate_acquisition(acquisition: &Acquisition) -> Result<(), PromotionError> {
    if acquisition.repository != PROVIDER_REPOSITORY
        || acquisition.run_id != PROVIDER_RUN_ID
        || acquisition.artifact_id != PROVIDER_ARTIFACT_ID
        || acquisition.artifact_name != PROVIDER_ARTIFACT_NAME
        || acquisition.provider_digest != PROVIDER_DIGEST
        || !valid_utc_timestamp(&acquisition.artifact_created_at)
        || !valid_utc_timestamp(&acquisition.artifact_expires_at)
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Provider,
            "artifact provider metadata does not equal the canonical acquisition tuple",
        ));
    }
    Ok(())
}

pub(super) fn read_bundle_manifest(root: &Path) -> Result<BundleManifest, PromotionError> {
    read_json(&root.join("phase13-bundle.json"))
}

pub(super) fn closure_review(
    recorded: &BundleClosure,
    recomputed: &BundleClosure,
) -> ClosureReview {
    ClosureReview {
        recorded_sha256: recorded.digest.clone(),
        recomputed_sha256: recomputed.digest.clone(),
        byte_identical: recorded == recomputed,
    }
}

pub(super) fn promoted_paths() -> Vec<String> {
    PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
}

pub(super) fn promoted_path_set_sha256() -> String {
    path_set_sha256("phase13-promoted-path-set-v2", &promoted_paths())
}

pub(super) fn changed_path_set_sha256(paths: &[String]) -> String {
    path_set_sha256("phase13-changed-path-set-v2", paths)
}

pub(super) fn path_set_sha256(domain: &str, paths: &[String]) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, domain.as_bytes());
    for path in paths {
        update_field(&mut hasher, path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub(crate) fn classify_reviewed_paths(
    baseline_sha256: &BTreeMap<String, String>,
    replacement_sha256: &BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>), PromotionError> {
    if baseline_sha256.keys().collect::<Vec<_>>() != replacement_sha256.keys().collect::<Vec<_>>() {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            "baseline and replacement path sets differ",
        ));
    }
    let (changed, unchanged): (Vec<_>, Vec<_>) = replacement_sha256
        .iter()
        .partition(|(path, replacement)| baseline_sha256.get(*path) != Some(*replacement));
    let changed = changed
        .into_iter()
        .map(|(path, _digest)| path.clone())
        .collect();
    let unchanged = unchanged
        .into_iter()
        .map(|(path, _digest)| path.clone())
        .collect();
    Ok((changed, unchanged))
}

use super::{
    BTreeSet, Command, Component, Deserialize, EvidenceKind, EvidenceManifest, IDENTITY_FILE,
    INVENTORY_FILE, MANIFEST_FILE, MAXIMUM_LOG_BYTES, PROVENANCE_FILE, Path, Phase9EvidenceError,
    READ_ONLY_FILE, TRACE_FILE, checked_relative_path, cross_run_payload_refs, is_sha256,
    read_regular_file, require_digest, resolve_existing_target_path, sha256,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExactRun {
    repository: String,
    branch: String,
    pub(super) approved_sha: String,
    head_sha: String,
    dispatched_at: String,
    pub(super) run_id: u64,
    run_url: String,
    workflow_name: String,
    event: String,
    conclusion: String,
    created_at: String,
    updated_at: String,
    jobs: ExactJobs,
    artifacts: ExactArtifacts,
    live_run: LiveRun,
    live_jobs: Vec<LiveJob>,
    live_artifacts: Vec<LiveArtifact>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactJobs {
    canonical: ExactJob,
    sanitizer: ExactJob,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactJob {
    id: u64,
    name: String,
    url: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactArtifacts {
    canonical: ExactArtifact,
    sanitizer: ExactArtifact,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactArtifact {
    id: u64,
    name: String,
    api_url: String,
    archive_download_url: String,
    digest: String,
    size_in_bytes: u64,
    expired: bool,
    created_at: String,
    expires_at: String,
    archive_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveRun {
    id: u64,
    head_sha: String,
    name: String,
    event: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveJob {
    id: u64,
    name: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveArtifact {
    id: u64,
    name: String,
    digest: String,
    expired: bool,
}

pub(super) fn parse_exact_run(
    value: serde_json::Value,
    denied_run_ids: &BTreeSet<u64>,
) -> Result<ExactRun, Phase9EvidenceError> {
    let run: ExactRun = serde_json::from_value(value)
        .map_err(|error| Phase9EvidenceError::new("run", error.to_string()))?;
    if run.run_id == 0 || denied_run_ids.contains(&run.run_id) {
        return Err(Phase9EvidenceError::new(
            "run",
            format!("run {} is absent or denylisted", run.run_id),
        ));
    }
    if run.repository != "bright-builds-llc/liquidfun-rs"
        || run.branch != "main"
        || !is_full_sha(&run.approved_sha)
        || run.approved_sha != run.head_sha
        || run.workflow_name != "Oracle CI"
        || run.event != "workflow_dispatch"
        || run.conclusion != "success"
        || run.dispatched_at.is_empty()
        || run.run_url.is_empty()
        || run.created_at.is_empty()
        || run.updated_at.is_empty()
    {
        return Err(Phase9EvidenceError::new(
            "run",
            "run does not match the approved head or Oracle CI dispatch authority",
        ));
    }
    validate_exact_job(
        &run.jobs.canonical,
        "Canonical Linux oracle",
        &run.live_jobs,
    )?;
    validate_exact_job(
        &run.jobs.sanitizer,
        "Scheduled fail-fast sanitizer and reset corpus",
        &run.live_jobs,
    )?;
    if run.jobs.canonical.id == run.jobs.sanitizer.id {
        return Err(Phase9EvidenceError::new(
            "jobs",
            "canonical and sanitizer job IDs must be unique",
        ));
    }
    validate_exact_artifact(
        run.run_id,
        &run.approved_sha,
        EvidenceKind::Canonical,
        &run.artifacts.canonical,
        &run.live_artifacts,
    )?;
    validate_exact_artifact(
        run.run_id,
        &run.approved_sha,
        EvidenceKind::Sanitizer,
        &run.artifacts.sanitizer,
        &run.live_artifacts,
    )?;
    if run.artifacts.canonical.id == run.artifacts.sanitizer.id {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            "canonical and sanitizer artifact IDs must be unique",
        ));
    }
    if run.live_jobs.len() != 2 || run.live_artifacts.len() != 2 {
        return Err(Phase9EvidenceError::new(
            "run",
            "live metadata must contain exactly two jobs and two artifacts",
        ));
    }
    if run.live_run.id != run.run_id
        || run.live_run.head_sha != run.approved_sha
        || run.live_run.name != run.workflow_name
        || run.live_run.event != run.event
        || run.live_run.conclusion != run.conclusion
    {
        return Err(Phase9EvidenceError::new(
            "run",
            "live run snapshot does not match run.json",
        ));
    }
    Ok(run)
}

fn validate_exact_job(
    job: &ExactJob,
    expected_name: &str,
    live_jobs: &[LiveJob],
) -> Result<(), Phase9EvidenceError> {
    if job.name != expected_name || job.conclusion != "success" || job.url.is_empty() {
        return Err(Phase9EvidenceError::new(
            "jobs",
            format!("required successful job `{expected_name}` is absent"),
        ));
    }
    let matches = live_jobs
        .iter()
        .filter(|live| {
            live.id == job.id && live.name == job.name && live.conclusion == job.conclusion
        })
        .count();
    if matches != 1
        || live_jobs
            .iter()
            .filter(|live| live.name == expected_name)
            .count()
            != 1
    {
        return Err(Phase9EvidenceError::new(
            "jobs",
            format!("live job `{expected_name}` is missing or duplicated"),
        ));
    }
    Ok(())
}

fn validate_exact_artifact(
    run_id: u64,
    approved_sha: &str,
    kind: EvidenceKind,
    artifact: &ExactArtifact,
    live_artifacts: &[LiveArtifact],
) -> Result<(), Phase9EvidenceError> {
    let expected_name = format!("{}-{run_id}-{approved_sha}", kind.artifact_prefix());
    if artifact.name != expected_name
        || artifact.expired
        || artifact.api_url.is_empty()
        || artifact.archive_download_url.is_empty()
        || artifact.size_in_bytes == 0
        || artifact.created_at.is_empty()
        || artifact.expires_at.is_empty()
        || artifact
            .digest
            .strip_prefix("sha256:")
            .is_none_or(|digest| !is_sha256(digest))
    {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            format!("artifact `{expected_name}` is absent, expired, or malformed"),
        ));
    }
    let matches = live_artifacts
        .iter()
        .filter(|live| {
            live.id == artifact.id
                && live.name == artifact.name
                && live.digest == artifact.digest
                && live.expired == artifact.expired
        })
        .count();
    if matches != 1
        || live_artifacts
            .iter()
            .filter(|live| live.name == expected_name)
            .count()
            != 1
    {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            format!("live artifact `{expected_name}` is missing or duplicated"),
        ));
    }
    Ok(())
}

pub(super) fn validate_archive(
    repository_root: &Path,
    kind: EvidenceKind,
    run: &ExactRun,
    manifest: &EvidenceManifest,
) -> Result<(), Phase9EvidenceError> {
    let artifact = match kind {
        EvidenceKind::Canonical => &run.artifacts.canonical,
        EvidenceKind::Sanitizer => &run.artifacts.sanitizer,
    };
    let archive_relative = checked_relative_path(&artifact.archive_path)?;
    let archive = resolve_existing_target_path(repository_root, &archive_relative, "archive")?;
    let bytes = read_regular_file(&archive, "archive", MAXIMUM_LOG_BYTES)?;
    if u64::try_from(bytes.len()).ok() != Some(artifact.size_in_bytes) {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive size does not match recorded artifact metadata",
        ));
    }
    let expected = artifact
        .digest
        .strip_prefix("sha256:")
        .expect("artifact digest validated during run parsing");
    require_digest("archive", expected, &sha256(&bytes))?;
    let output = Command::new("unzip")
        .arg("-Z1")
        .arg(&archive)
        .output()
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    if !output.status.success() {
        return Err(Phase9EvidenceError::new(
            "archive",
            "unzip could not inspect the artifact archive",
        ));
    }
    let entries = std::str::from_utf8(&output.stdout)
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    let expected_files = expected_evidence_files(manifest);
    let mut archive_files = BTreeSet::new();
    for entry in entries.lines() {
        if entry.ends_with('/') {
            validate_archive_entry(entry.trim_end_matches('/'))?;
            continue;
        }
        validate_archive_entry(entry)?;
        archive_files.insert(entry.to_owned());
    }
    if archive_files != expected_files {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive entries do not match extracted evidence files",
        ));
    }
    let modes = Command::new("unzip")
        .args(["-Z", "-l"])
        .arg(&archive)
        .output()
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    if !modes.status.success()
        || std::str::from_utf8(&modes.stdout)
            .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?
            .lines()
            .any(|line| line.starts_with('l'))
    {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive contains a symlink or unreadable mode listing",
        ));
    }
    Ok(())
}

pub(super) fn validate_archive_entry(entry: &str) -> Result<(), Phase9EvidenceError> {
    let path = Path::new(entry);
    if entry.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "archive",
            format!("unsafe archive entry `{entry}`"),
        ));
    }
    Ok(())
}

pub(super) fn expected_evidence_files(manifest: &EvidenceManifest) -> BTreeSet<String> {
    let mut files = BTreeSet::from([
        IDENTITY_FILE.to_owned(),
        MANIFEST_FILE.to_owned(),
        TRACE_FILE.to_owned(),
        PROVENANCE_FILE.to_owned(),
        INVENTORY_FILE.to_owned(),
        READ_ONLY_FILE.to_owned(),
    ]);
    for case in &manifest.cases {
        files.insert(case.request_path.clone());
        files.insert(case.native_result_path.clone());
        files.insert(case.oracle_result_path.clone());
        files.insert(case.complete_comparison_path.clone());
        files.extend(
            cross_run_payload_refs(&case.cross_run_proofs)
                .into_iter()
                .map(|reference| reference.path.to_string()),
        );
    }
    files
}

pub(super) fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

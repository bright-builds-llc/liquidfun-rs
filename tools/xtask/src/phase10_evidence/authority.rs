use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
    process::Command,
};

use serde::Deserialize;

use super::{
    Phase10EvidenceError,
    content::{EvidenceKind, ValidatedDirectory},
    paths::{
        MAXIMUM_LOG_BYTES, checked_relative_path, is_sha256, read_regular_file, require_digest,
        resolve_target_path, sha256,
    },
};

const REPOSITORY: &str = "bright-builds-llc/liquidfun-rs";
const WORKFLOW: &str = "Oracle CI";
const PLATFORM: &str = "linux-x86_64";
const RUST_VERSION: &str = "1.97.0";
const CLANG_VERSION: &str = "22.1.8";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PROTOCOL_VERSION: &str = "rigid-world-phase10-v1";
const GENERATOR_VERSION: &str = "phase10-corpus-v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExactRun {
    repository: String,
    branch: String,
    approved_sha: String,
    head_sha: String,
    pub(super) run_id: u64,
    workflow_name: String,
    event: String,
    conclusion: String,
    run_url: String,
    dispatched_at: String,
    created_at: String,
    updated_at: String,
    captured_at: String,
    platform: String,
    rust_version: String,
    clang_version: String,
    upstream_revision: String,
    protocol_version: String,
    generator_version: String,
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
) -> Result<ExactRun, Phase10EvidenceError> {
    let run: ExactRun = serde_json::from_value(value)
        .map_err(|error| Phase10EvidenceError::new("run", error.to_string()))?;
    if run.run_id == 0
        || denied_run_ids.contains(&run.run_id)
        || run.repository != REPOSITORY
        || run.branch != "main"
        || !is_full_sha(&run.approved_sha)
        || run.approved_sha != run.head_sha
        || run.workflow_name != WORKFLOW
        || run.event != "workflow_dispatch"
        || run.conclusion != "success"
        || run.platform != PLATFORM
        || run.rust_version != RUST_VERSION
        || run.clang_version != CLANG_VERSION
        || run.upstream_revision != UPSTREAM_REVISION
        || run.protocol_version != PROTOCOL_VERSION
        || run.generator_version != GENERATOR_VERSION
        || !valid_url(&run.run_url)
        || !valid_timestamp(&run.dispatched_at)
        || !valid_timestamp(&run.created_at)
        || !valid_timestamp(&run.updated_at)
        || !valid_timestamp(&run.captured_at)
        || run.created_at > run.updated_at
        || run.updated_at > run.captured_at
    {
        return Err(Phase10EvidenceError::new(
            "run",
            "run is stale, unsuccessful, or outside the locked D1 stack",
        ));
    }
    validate_live_run(&run)?;
    validate_job(&run.jobs.canonical, EvidenceKind::Canonical, &run.live_jobs)?;
    validate_job(&run.jobs.sanitizer, EvidenceKind::Sanitizer, &run.live_jobs)?;
    if run.jobs.canonical.id == run.jobs.sanitizer.id || run.live_jobs.len() != 2 {
        return Err(Phase10EvidenceError::new(
            "jobs",
            "canonical and sanitizer jobs must be one distinct pair",
        ));
    }
    validate_artifact_metadata(&run, &run.artifacts.canonical, EvidenceKind::Canonical)?;
    validate_artifact_metadata(&run, &run.artifacts.sanitizer, EvidenceKind::Sanitizer)?;
    if run.artifacts.canonical.id == run.artifacts.sanitizer.id || run.live_artifacts.len() != 2 {
        return Err(Phase10EvidenceError::new(
            "artifacts",
            "canonical and sanitizer artifacts must be one distinct pair",
        ));
    }
    Ok(run)
}

pub(super) fn validate_exact_pair(
    repository_root: &Path,
    run: &ExactRun,
    canonical: &ValidatedDirectory,
    sanitizer: &ValidatedDirectory,
    denied_artifact_ids: &BTreeSet<u64>,
) -> Result<(), Phase10EvidenceError> {
    validate_identity(run, canonical, EvidenceKind::Canonical)?;
    validate_identity(run, sanitizer, EvidenceKind::Sanitizer)?;
    for (kind, directory, artifact) in [
        (EvidenceKind::Canonical, canonical, &run.artifacts.canonical),
        (EvidenceKind::Sanitizer, sanitizer, &run.artifacts.sanitizer),
    ] {
        if denied_artifact_ids.contains(&artifact.id) {
            return Err(Phase10EvidenceError::new(
                "artifacts",
                format!("artifact {} is denylisted", artifact.id),
            ));
        }
        validate_archive(repository_root, kind, directory, artifact)?;
    }
    Ok(())
}

fn validate_live_run(run: &ExactRun) -> Result<(), Phase10EvidenceError> {
    if run.live_run.id != run.run_id
        || run.live_run.head_sha != run.approved_sha
        || run.live_run.name != run.workflow_name
        || run.live_run.event != run.event
        || run.live_run.conclusion != run.conclusion
    {
        return Err(Phase10EvidenceError::new(
            "run",
            "independently captured live run metadata differs",
        ));
    }
    Ok(())
}

fn validate_job(
    job: &ExactJob,
    kind: EvidenceKind,
    live_jobs: &[LiveJob],
) -> Result<(), Phase10EvidenceError> {
    let expected = job_name(kind);
    let matches = live_jobs
        .iter()
        .filter(|live| {
            live.id == job.id
                && live.name == job.name
                && live.conclusion == job.conclusion
                && live.name == expected
        })
        .count();
    if job.name != expected || job.conclusion != "success" || !valid_url(&job.url) || matches != 1 {
        return Err(Phase10EvidenceError::new(
            "jobs",
            format!("required successful job `{expected}` is absent or mixed"),
        ));
    }
    Ok(())
}

fn validate_artifact_metadata(
    run: &ExactRun,
    artifact: &ExactArtifact,
    kind: EvidenceKind,
) -> Result<(), Phase10EvidenceError> {
    let expected = artifact_name(run.run_id, &run.approved_sha, kind);
    let matches = run
        .live_artifacts
        .iter()
        .filter(|live| {
            live.id == artifact.id
                && live.name == artifact.name
                && live.digest == artifact.digest
                && live.expired == artifact.expired
        })
        .count();
    let maybe_digest = artifact.digest.strip_prefix("sha256:");
    if artifact.name != expected
        || artifact.expired
        || maybe_digest.is_none_or(|digest| !is_sha256(digest))
        || artifact.created_at > run.captured_at
        || artifact.expires_at <= run.captured_at
        || !valid_url(&artifact.api_url)
        || !valid_url(&artifact.archive_download_url)
        || !valid_timestamp(&artifact.created_at)
        || !valid_timestamp(&artifact.expires_at)
        || matches != 1
    {
        return Err(Phase10EvidenceError::new(
            "artifacts",
            format!("artifact `{expected}` is stale, expired, malformed, or mixed"),
        ));
    }
    Ok(())
}

fn validate_identity(
    run: &ExactRun,
    directory: &ValidatedDirectory,
    kind: EvidenceKind,
) -> Result<(), Phase10EvidenceError> {
    let artifact = match kind {
        EvidenceKind::Canonical => &run.artifacts.canonical,
        EvidenceKind::Sanitizer => &run.artifacts.sanitizer,
    };
    if directory.identity.mode != "exact-ref"
        || directory.identity.run_id != run.run_id
        || directory.identity.head_sha != run.approved_sha
        || directory.identity.job_name != job_name(kind)
        || directory.identity.artifact_id != artifact.id
        || directory.identity.artifact_name != artifact.name
        || directory.identity.platform != PLATFORM
        || directory.identity.rust_version != RUST_VERSION
        || directory.identity.clang_version != CLANG_VERSION
        || directory.identity.upstream_revision != UPSTREAM_REVISION
        || directory.identity.protocol_version != PROTOCOL_VERSION
        || directory.identity.generator_version != GENERATOR_VERSION
    {
        return Err(Phase10EvidenceError::new(
            "identity",
            "extracted identity differs from same-run job/artifact authority",
        ));
    }
    Ok(())
}

fn validate_archive(
    repository_root: &Path,
    _kind: EvidenceKind,
    directory: &ValidatedDirectory,
    artifact: &ExactArtifact,
) -> Result<(), Phase10EvidenceError> {
    let relative = checked_relative_path(&artifact.archive_path)?;
    let path = resolve_target_path(repository_root, &relative, "archive")?;
    let bytes = read_regular_file(&path, "archive", MAXIMUM_LOG_BYTES)?;
    if bytes.len() as u64 != artifact.size_in_bytes {
        return Err(Phase10EvidenceError::new(
            "archive",
            "archive size differs from API metadata",
        ));
    }
    require_digest(
        "archive",
        artifact
            .digest
            .strip_prefix("sha256:")
            .expect("artifact digest validated"),
        &sha256(&bytes),
    )?;
    let listing = Command::new("unzip")
        .arg("-Z1")
        .arg(&path)
        .output()
        .map_err(|error| Phase10EvidenceError::new("archive", error.to_string()))?;
    if !listing.status.success() {
        return Err(Phase10EvidenceError::new(
            "archive",
            "unzip could not inspect archive",
        ));
    }
    let text = std::str::from_utf8(&listing.stdout)
        .map_err(|error| Phase10EvidenceError::new("archive", error.to_string()))?;
    let mut files = BTreeSet::new();
    let mut casefold = HashSet::new();
    for entry in text.lines() {
        let trimmed = entry.trim_end_matches('/');
        validate_archive_entry(trimmed)?;
        if !casefold.insert(trimmed.to_ascii_lowercase()) {
            return Err(Phase10EvidenceError::new(
                "archive",
                "archive has duplicate or case-colliding entries",
            ));
        }
        if !entry.ends_with('/') {
            files.insert(entry.to_owned());
        }
    }
    if files != directory.expected_files {
        return Err(Phase10EvidenceError::new(
            "archive",
            "archive entries differ from extracted closed file set",
        ));
    }
    let modes = Command::new("unzip")
        .args(["-Z", "-l"])
        .arg(path)
        .output()
        .map_err(|error| Phase10EvidenceError::new("archive", error.to_string()))?;
    let mode_listing = std::str::from_utf8(&modes.stdout)
        .map_err(|error| Phase10EvidenceError::new("archive", error.to_string()))?;
    if !modes.status.success() || unsafe_archive_modes(mode_listing)? {
        return Err(Phase10EvidenceError::new(
            "archive",
            "archive contains links or unreadable entry types",
        ));
    }
    Ok(())
}

fn validate_archive_entry(entry: &str) -> Result<(), Phase10EvidenceError> {
    let path = Path::new(entry);
    if entry.is_empty()
        || entry.contains('\\')
        || path.is_absolute()
        || path.components().count() > 6
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Phase10EvidenceError::new(
            "archive",
            format!("unsafe archive entry `{entry}`"),
        ));
    }
    Ok(())
}

const fn job_name(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::Canonical => "Phase 10 canonical Linux oracle",
        EvidenceKind::Sanitizer => "Phase 10 fail-fast sanitizer",
    }
}

fn artifact_name(run_id: u64, sha: &str, kind: EvidenceKind) -> String {
    let prefix = match kind {
        EvidenceKind::Canonical => "phase10-canonical",
        EvidenceKind::Sanitizer => "phase10-sanitizer",
    };
    format!("{prefix}-{run_id}-{sha}")
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_url(value: &str) -> bool {
    value.starts_with("https://") && !value.contains(char::is_whitespace)
}

fn valid_timestamp(value: &str) -> bool {
    value.len() == 20 && value.ends_with('Z') && value.as_bytes().get(10) == Some(&b'T')
}

fn unsafe_archive_modes(listing: &str) -> Result<bool, Phase10EvidenceError> {
    let mut total_uncompressed = 0_u64;
    for line in listing
        .lines()
        .filter(|line| line.starts_with('-') || line.starts_with('d') || line.starts_with('l'))
    {
        if line.starts_with('l') || (!line.starts_with('-') && !line.starts_with('d')) {
            return Ok(true);
        }
        let Some(size) = line
            .split_whitespace()
            .nth(3)
            .and_then(|value| value.parse::<u64>().ok())
        else {
            return Err(Phase10EvidenceError::new(
                "archive",
                "archive mode listing has an unreadable entry size",
            ));
        };
        if size > MAXIMUM_LOG_BYTES {
            return Ok(true);
        }
        total_uncompressed = total_uncompressed
            .checked_add(size)
            .ok_or_else(|| Phase10EvidenceError::new("archive", "archive size overflow"))?;
        if total_uncompressed > MAXIMUM_LOG_BYTES {
            return Ok(true);
        }
    }
    Ok(false)
}

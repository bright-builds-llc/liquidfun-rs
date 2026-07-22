//! Exact-reference authority layered only over accepted semantic content.

use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
    process::Command,
};

use serde::Deserialize;

use super::{
    EvidenceIdentity, Phase11EvidenceError,
    content::{
        AcceptedContent, EvidenceKind, GENERATOR_VERSION, PROTOCOL_VERSION, UPSTREAM_REVISION,
    },
    paths::{
        MAX_ARCHIVE_BYTES, MAX_FILES, checked_target_path, is_sha256, read_regular, require_sha256,
        resolve_input, sha256,
    },
};

const REPOSITORY: &str = "bright-builds-llc/liquidfun-rs";
const WORKFLOW: &str = "Oracle CI";
const PLATFORM: &str = "linux-x86_64";
const RUST_VERSION: &str = "1.97.0";
const CLANG_VERSION: &str = "22.1.8";
const PRE_UPLOAD_ARTIFACT_ID: u64 = 0;

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
    metadata_source: String,
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
    head_sha: String,
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
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveJob {
    id: u64,
    name: String,
    conclusion: String,
    head_sha: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveArtifact {
    id: u64,
    name: String,
    digest: String,
    size_in_bytes: u64,
    expired: bool,
    created_at: String,
}

pub(super) fn parse_exact_run(
    value: serde_json::Value,
    denied_run_ids: &BTreeSet<u64>,
) -> Result<ExactRun, Phase11EvidenceError> {
    let run: ExactRun = serde_json::from_value(value)
        .map_err(|error| Phase11EvidenceError::new("run", error.to_string()))?;
    if run.run_id == 0
        || denied_run_ids.contains(&run.run_id)
        || run.repository != REPOSITORY
        || run.branch != "main"
        || !full_sha(&run.approved_sha)
        || run.approved_sha != run.head_sha
        || run.workflow_name != WORKFLOW
        || run.event != "workflow_dispatch"
        || run.conclusion != "success"
        || run.metadata_source != "github-api-live"
        || run.platform != PLATFORM
        || run.rust_version != RUST_VERSION
        || run.clang_version != CLANG_VERSION
        || run.upstream_revision != UPSTREAM_REVISION
        || run.protocol_version != PROTOCOL_VERSION
        || run.generator_version != GENERATOR_VERSION
        || !url(&run.run_url)
        || !timestamp(&run.dispatched_at)
        || !timestamp(&run.created_at)
        || !timestamp(&run.updated_at)
        || !timestamp(&run.captured_at)
        || run.created_at > run.updated_at
        || run.updated_at > run.captured_at
    {
        return Err(Phase11EvidenceError::new(
            "run",
            "run is historical, stale, unsuccessful, or outside the locked D1 stack",
        ));
    }
    validate_live_run(&run)?;
    validate_job(&run, &run.jobs.canonical, EvidenceKind::Canonical)?;
    validate_job(&run, &run.jobs.sanitizer, EvidenceKind::Sanitizer)?;
    if run.jobs.canonical.id == run.jobs.sanitizer.id || run.live_jobs.len() != 2 {
        return Err(Phase11EvidenceError::new(
            "jobs",
            "canonical and sanitizer jobs must be one distinct same-run pair",
        ));
    }
    validate_artifact(&run, &run.artifacts.canonical, EvidenceKind::Canonical)?;
    validate_artifact(&run, &run.artifacts.sanitizer, EvidenceKind::Sanitizer)?;
    if run.artifacts.canonical.id == run.artifacts.sanitizer.id || run.live_artifacts.len() != 2 {
        return Err(Phase11EvidenceError::new(
            "artifacts",
            "canonical and sanitizer artifacts must be one distinct same-run pair",
        ));
    }
    Ok(run)
}

pub(super) fn validate_exact_pair(
    repository_root: &Path,
    run: &ExactRun,
    canonical: &AcceptedContent,
    canonical_identity: &EvidenceIdentity,
    sanitizer: &AcceptedContent,
    sanitizer_identity: &EvidenceIdentity,
    denied_artifact_ids: &BTreeSet<u64>,
) -> Result<(), Phase11EvidenceError> {
    if canonical.source_only || sanitizer.source_only {
        return Err(Phase11EvidenceError::new(
            "identity",
            "identity-free tracked corpus cannot promote",
        ));
    }
    validate_identity(run, canonical, canonical_identity, EvidenceKind::Canonical)?;
    validate_identity(run, sanitizer, sanitizer_identity, EvidenceKind::Sanitizer)?;
    for (content, artifact) in [
        (canonical, &run.artifacts.canonical),
        (sanitizer, &run.artifacts.sanitizer),
    ] {
        if denied_artifact_ids.contains(&artifact.id) {
            return Err(Phase11EvidenceError::new(
                "artifacts",
                format!("artifact {} is denylisted", artifact.id),
            ));
        }
        inspect_archive(repository_root, content, artifact)?;
    }
    Ok(())
}

fn validate_live_run(run: &ExactRun) -> Result<(), Phase11EvidenceError> {
    if run.live_run.id != run.run_id
        || run.live_run.head_sha != run.approved_sha
        || run.live_run.name != run.workflow_name
        || run.live_run.event != run.event
        || run.live_run.conclusion != run.conclusion
        || run.live_run.updated_at != run.updated_at
    {
        return Err(Phase11EvidenceError::new(
            "run",
            "fresh live run metadata differs from the approval envelope",
        ));
    }
    Ok(())
}

fn validate_job(
    run: &ExactRun,
    job: &ExactJob,
    kind: EvidenceKind,
) -> Result<(), Phase11EvidenceError> {
    let expected = job_name(kind);
    let matches = run
        .live_jobs
        .iter()
        .filter(|live| {
            live.id == job.id
                && live.name == job.name
                && live.conclusion == job.conclusion
                && live.head_sha == job.head_sha
        })
        .count();
    if job.id == 0
        || job.name != expected
        || job.conclusion != "success"
        || job.head_sha != run.approved_sha
        || !url(&job.url)
        || matches != 1
    {
        return Err(Phase11EvidenceError::new(
            "jobs",
            format!("required successful same-SHA job `{expected}` is absent or mixed"),
        ));
    }
    Ok(())
}

fn validate_artifact(
    run: &ExactRun,
    artifact: &ExactArtifact,
    kind: EvidenceKind,
) -> Result<(), Phase11EvidenceError> {
    let expected = artifact_name(run, kind);
    let matches = run
        .live_artifacts
        .iter()
        .filter(|live| {
            live.id == artifact.id
                && live.name == artifact.name
                && live.digest == artifact.digest
                && live.size_in_bytes == artifact.size_in_bytes
                && live.expired == artifact.expired
                && live.created_at == artifact.created_at
        })
        .count();
    let maybe_digest = artifact.digest.strip_prefix("sha256:");
    if artifact.id == 0
        || artifact.name != expected
        || artifact.expired
        || artifact.size_in_bytes == 0
        || artifact.size_in_bytes > MAX_ARCHIVE_BYTES
        || maybe_digest.is_none_or(|digest| !is_sha256(digest))
        || artifact.created_at > run.captured_at
        || artifact.expires_at <= run.captured_at
        || !timestamp(&artifact.created_at)
        || !timestamp(&artifact.expires_at)
        || !url(&artifact.api_url)
        || !url(&artifact.archive_download_url)
        || matches != 1
    {
        return Err(Phase11EvidenceError::new(
            "artifacts",
            format!("artifact `{expected}` is stale, zero-ID, malformed, or mixed"),
        ));
    }
    Ok(())
}

fn validate_identity(
    run: &ExactRun,
    content: &AcceptedContent,
    identity: &EvidenceIdentity,
    kind: EvidenceKind,
) -> Result<(), Phase11EvidenceError> {
    let artifact = match kind {
        EvidenceKind::Canonical => &run.artifacts.canonical,
        EvidenceKind::Sanitizer => &run.artifacts.sanitizer,
    };
    if identity.mode != "exact-ref"
        || identity.run_id != run.run_id
        || identity.head_sha != run.approved_sha
        || identity.job_name != job_name(kind)
        || identity.artifact_id != PRE_UPLOAD_ARTIFACT_ID
        || identity.artifact_name != artifact.name
        || identity.platform != PLATFORM
        || identity.rust_version != RUST_VERSION
        || identity.clang_version != CLANG_VERSION
        || identity.semantic_sha256 != content.semantic_sha256
    {
        return Err(Phase11EvidenceError::new(
            "identity",
            "artifact identity differs from accepted content or same-run authority",
        ));
    }
    Ok(())
}

fn inspect_archive(
    repository_root: &Path,
    content: &AcceptedContent,
    artifact: &ExactArtifact,
) -> Result<(), Phase11EvidenceError> {
    let relative = checked_target_path(&artifact.archive_path)?;
    let path = resolve_input(repository_root, &relative, "archive")?;
    let bytes = read_regular(&path, "archive", MAX_ARCHIVE_BYTES)?;
    if bytes.len() as u64 != artifact.size_in_bytes {
        return Err(Phase11EvidenceError::new(
            "archive",
            "archive size differs from live API metadata",
        ));
    }
    require_sha256(
        "archive",
        artifact
            .digest
            .strip_prefix("sha256:")
            .expect("artifact digest was validated"),
        &sha256(&bytes),
    )?;
    let names = Command::new("unzip")
        .arg("-Z1")
        .arg(&path)
        .output()
        .map_err(|error| Phase11EvidenceError::new("archive", error.to_string()))?;
    if !names.status.success() {
        return Err(Phase11EvidenceError::new(
            "archive",
            "archive inspection failed before extraction",
        ));
    }
    let text = std::str::from_utf8(&names.stdout)
        .map_err(|error| Phase11EvidenceError::new("archive", error.to_string()))?;
    let mut files = BTreeSet::new();
    let mut folded = HashSet::new();
    for entry in text.lines() {
        let value = entry.trim_end_matches('/');
        validate_entry(value)?;
        if !folded.insert(value.to_ascii_lowercase()) {
            return Err(Phase11EvidenceError::new(
                "archive",
                "archive contains duplicate or case-colliding entries",
            ));
        }
        if !entry.ends_with('/') {
            files.insert(entry.to_owned());
        }
        if files.len() > MAX_FILES {
            return Err(Phase11EvidenceError::new(
                "archive",
                "archive file count exceeds bound",
            ));
        }
    }
    if files != content.expected_files {
        return Err(Phase11EvidenceError::new(
            "archive",
            "archive entries differ from the accepted closed file set",
        ));
    }
    validate_modes_and_sizes(&path, bytes.len() as u64)
}

fn validate_entry(value: &str) -> Result<(), Phase11EvidenceError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || path.is_absolute()
        || path.components().count() > 4
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Phase11EvidenceError::new(
            "archive",
            format!("unsafe archive entry `{value}`"),
        ));
    }
    Ok(())
}

fn validate_modes_and_sizes(path: &Path, compressed_size: u64) -> Result<(), Phase11EvidenceError> {
    let output = Command::new("unzip")
        .args(["-Z", "-l"])
        .arg(path)
        .output()
        .map_err(|error| Phase11EvidenceError::new("archive", error.to_string()))?;
    if !output.status.success() {
        return Err(Phase11EvidenceError::new(
            "archive",
            "archive mode inspection failed",
        ));
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|error| Phase11EvidenceError::new("archive", error.to_string()))?;
    let mut total = 0_u64;
    for line in text
        .lines()
        .filter(|line| line.starts_with('-') || line.starts_with('d') || line.starts_with('l'))
    {
        if line.starts_with('l') || (!line.starts_with('-') && !line.starts_with('d')) {
            return Err(Phase11EvidenceError::new(
                "archive",
                "links and device-like entries are forbidden",
            ));
        }
        let size = line
            .split_whitespace()
            .nth(3)
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| Phase11EvidenceError::new("archive", "unreadable entry size"))?;
        total = total
            .checked_add(size)
            .ok_or_else(|| Phase11EvidenceError::new("archive", "size overflow"))?;
        if size > MAX_ARCHIVE_BYTES || total > MAX_ARCHIVE_BYTES {
            return Err(Phase11EvidenceError::new(
                "archive",
                "archive uncompressed bytes exceed bound",
            ));
        }
    }
    if total
        > compressed_size
            .saturating_mul(100)
            .saturating_add(1_048_576)
    {
        return Err(Phase11EvidenceError::new(
            "archive",
            "archive compression ratio exceeds bound",
        ));
    }
    Ok(())
}

const fn job_name(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::Canonical => "Phase 11 canonical Linux oracle",
        EvidenceKind::Sanitizer => "Phase 11 fail-fast sanitizer",
    }
}

fn artifact_name(run: &ExactRun, kind: EvidenceKind) -> String {
    let prefix = match kind {
        EvidenceKind::Canonical => "phase11-canonical",
        EvidenceKind::Sanitizer => "phase11-sanitizer",
    };
    format!("{prefix}-{}-{}", run.run_id, run.approved_sha)
}

fn full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn url(value: &str) -> bool {
    value.starts_with("https://") && !value.contains(char::is_whitespace)
}

fn timestamp(value: &str) -> bool {
    value.len() == 20 && value.ends_with('Z') && value.as_bytes().get(10) == Some(&b'T')
}

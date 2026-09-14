//! Fresh provider authority, separate from durable reviewed receipt validation.

use std::io::Read;

use super::{
    Acquisition, Command, Deserialize, PROVIDER_REPOSITORY, Path, PromotionError,
    PromotionErrorKind, filesystem_error, fs, new_staging_root, run_process, valid_digest,
    valid_revision, valid_utc_timestamp, write_new_file,
};

const WORKFLOW_PATH: &str = ".github/workflows/phase13-evidence-producer.yml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema_version: u32,
    run_id: u64,
    run_attempt: u64,
    artifact_id: u64,
    archive_path: String,
}

#[derive(Debug, Deserialize)]
struct ProviderRun {
    id: u64,
    run_attempt: u64,
    head_sha: String,
    path: String,
    event: String,
    status: String,
    conclusion: String,
    run_started_at: String,
    repository: Repository,
    head_repository: Repository,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    id: u64,
    name: String,
    digest: String,
    created_at: String,
    expires_at: String,
    expired: bool,
    workflow_run: ArtifactRun,
}

#[derive(Debug, Deserialize)]
struct ArtifactRun {
    id: u64,
    head_sha: String,
}

#[derive(Debug, Deserialize)]
struct Jobs {
    total_count: usize,
    jobs: Vec<Job>,
}

#[derive(Debug, Deserialize)]
struct Job {
    run_id: u64,
    run_attempt: u64,
    name: String,
    head_sha: String,
    status: String,
    conclusion: String,
}

fn validate_jobs(jobs: &Jobs, request: &Request, producer_sha: &str) -> Result<(), PromotionError> {
    if jobs.total_count != 1
        || jobs.jobs.len() != 1
        || jobs.jobs.iter().any(|job| {
            job.run_id != request.run_id
                || job.run_attempt != request.run_attempt
                || job.name != "Produce one immutable canonical Linux bundle"
                || job.head_sha != producer_sha
                || job.status != "completed"
                || job.conclusion != "success"
        })
    {
        return Err(provider_error(
            "exact canonical producer job is absent or unsuccessful",
        ));
    }
    Ok(())
}

fn provider_error(message: &str) -> PromotionError {
    PromotionError::new(PromotionErrorKind::Provider, message)
}

pub(super) fn validate_acquisition(
    acquisition: &Acquisition,
    producer_sha: &str,
) -> Result<(), PromotionError> {
    if !valid_revision(producer_sha)
        || acquisition.repository != PROVIDER_REPOSITORY
        || acquisition.run_id == 0
        || acquisition.artifact_id == 0
        || acquisition.artifact_name
            != format!("phase13-staged-{}-{producer_sha}", acquisition.run_id)
        || !acquisition
            .provider_digest
            .strip_prefix("sha256:")
            .is_some_and(valid_digest)
        || !valid_utc_timestamp(&acquisition.artifact_created_at)
        || !valid_utc_timestamp(&acquisition.artifact_expires_at)
        || acquisition.artifact_created_at >= acquisition.artifact_expires_at
    {
        return Err(provider_error(
            "artifact acquisition does not bind exact P/run/digest",
        ));
    }
    Ok(())
}

fn validate_provider(
    request: &Request,
    run: &ProviderRun,
    artifact: &Artifact,
    producer_sha: &str,
) -> Result<Acquisition, PromotionError> {
    if request.schema_version != 1
        || request.run_id == 0
        || request.run_attempt == 0
        || request.artifact_id == 0
        || request.archive_path.is_empty()
        || run.id != request.run_id
        || run.run_attempt != request.run_attempt
        || run.repository.full_name != PROVIDER_REPOSITORY
        || run.head_repository.full_name != PROVIDER_REPOSITORY
        || run.head_sha != producer_sha
        || run.path != WORKFLOW_PATH
        || run.event != "workflow_dispatch"
        || run.status != "completed"
        || run.conclusion != "success"
        || !valid_utc_timestamp(&run.run_started_at)
        || artifact.id != request.artifact_id
        || artifact.workflow_run.id != run.id
        || artifact.workflow_run.head_sha != producer_sha
        || artifact.expired
        || artifact.created_at < run.run_started_at
    {
        return Err(provider_error(
            "provider run/artifact does not match the successful current producer attempt",
        ));
    }
    let acquisition = Acquisition {
        repository: PROVIDER_REPOSITORY.to_owned(),
        run_id: run.id,
        artifact_id: artifact.id,
        artifact_name: artifact.name.clone(),
        provider_digest: artifact.digest.clone(),
        artifact_created_at: artifact.created_at.clone(),
        artifact_expires_at: artifact.expires_at.clone(),
    };
    validate_acquisition(&acquisition, producer_sha)?;
    Ok(acquisition)
}

fn query<T: serde::de::DeserializeOwned>(
    repository_root: &Path,
    endpoint: &str,
    retained: &Path,
) -> Result<T, PromotionError> {
    run_process(
        Command::new("python3")
            .arg("-B")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .arg(repository_root.join("scripts/phase13-evidence-query.py"))
            .arg(endpoint)
            .arg(retained),
        "query provider with bounded output and deadline",
    )?;
    let bytes = fs::read(retained).map_err(filesystem_error)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| provider_error(&format!("invalid provider metadata: {error}")))
}

pub(super) fn acquire_provider_metadata(
    repository_root: &Path,
    request_path: &Path,
    bundle_root: &Path,
    producer_sha: &str,
) -> Result<Acquisition, PromotionError> {
    // Keep acquisition evidence beside, never inside, the seven-path replacement tree.
    let retained = new_staging_root(repository_root, "acquisition")?;
    for ancestor in request_path.ancestors() {
        if fs::symlink_metadata(ancestor)
            .map_err(filesystem_error)?
            .file_type()
            .is_symlink()
        {
            return Err(provider_error(
                "acquisition request contains a symlink component",
            ));
        }
    }
    let metadata = fs::metadata(request_path).map_err(filesystem_error)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err(provider_error(
            "acquisition request must be a regular file of at most 64 KiB",
        ));
    }
    let mut request_bytes = Vec::new();
    fs::File::open(request_path)
        .map_err(filesystem_error)?
        .take(64 * 1024 + 1)
        .read_to_end(&mut request_bytes)
        .map_err(filesystem_error)?;
    if request_bytes.len() > 64 * 1024 {
        return Err(provider_error("acquisition request grew beyond 64 KiB"));
    }
    write_new_file(&retained.join("request.json"), &request_bytes)?;
    let request: Request = serde_json::from_slice(&request_bytes)
        .map_err(|error| provider_error(&format!("invalid acquisition request: {error}")))?;
    if request.schema_version != 1
        || request.run_id == 0
        || request.run_attempt == 0
        || request.artifact_id == 0
    {
        return Err(provider_error(
            "acquisition request requires positive run, attempt and artifact identities",
        ));
    }
    let run_endpoint = format!(
        "repos/{PROVIDER_REPOSITORY}/actions/runs/{}",
        request.run_id
    );
    let run: ProviderRun = query(
        repository_root,
        &run_endpoint,
        &retained.join("run-before.json"),
    )?;
    let artifact: Artifact = query(
        repository_root,
        &format!(
            "repos/{PROVIDER_REPOSITORY}/actions/artifacts/{}",
            request.artifact_id
        ),
        &retained.join("artifact.json"),
    )?;
    let acquisition = validate_provider(&request, &run, &artifact, producer_sha)?;
    let jobs: Jobs = query(
        repository_root,
        &format!(
            "{run_endpoint}/attempts/{}/jobs?per_page=100",
            request.run_attempt
        ),
        &retained.join("jobs.json"),
    )?;
    validate_jobs(&jobs, &request, producer_sha)?;
    let archive = super::absolute_path(repository_root, &request.archive_path);
    let output = run_process(
        Command::new("python3")
            .arg("-B")
            .arg(repository_root.join("scripts/phase13-evidence-archive.py"))
            .arg(&archive)
            .arg(bundle_root)
            .arg(&acquisition.provider_digest)
            .arg(&acquisition.artifact_expires_at),
        "validate retained provider archive against extracted bundle",
    )?;
    write_new_file(&retained.join("archive-validation.json"), &output.stdout)?;
    let current: ProviderRun = query(
        repository_root,
        &run_endpoint,
        &retained.join("run-after.json"),
    )?;
    validate_provider(&request, &current, &artifact, producer_sha)?;
    if current.run_started_at != run.run_started_at {
        return Err(provider_error(
            "producer attempt changed during acquisition",
        ));
    }
    Ok(acquisition)
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;

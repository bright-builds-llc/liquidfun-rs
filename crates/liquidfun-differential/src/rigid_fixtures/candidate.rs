//! Atomic rigid candidate publication.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_test_protocol::{BuildIdentity, RigidWorldRequestRecord};

use super::super::{
    domain::{
        ArtifactCandidate, ArtifactKind, CANDIDATE_SCHEMA_VERSION, CandidateMetadata, FixtureError,
        ReviewStatus,
    },
    storage::{candidate_sha256, ensure_directory_chain, sha256, sync_directory, write_create_new},
};

static NEXT_TEMPORARY_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one post-validation write seam keeps candidate creation atomic and auditable"
)]
pub(super) fn write_rigid_candidate(
    repository_root: &Path,
    artifact_id: &str,
    artifact_kind: ArtifactKind,
    preset: &str,
    session_profile: &str,
    generator_revision: &str,
    request: &RigidWorldRequestRecord,
    request_bytes: &[u8],
    identity: &BuildIdentity,
    trace_bytes: &[u8],
    report_bytes: &[u8],
    maybe_failure_signature_json: Option<String>,
) -> Result<ArtifactCandidate, FixtureError> {
    let scenario_bytes = serde_json::to_vec(request.scenario())?;
    let identity_bytes = trace_bytes
        .split_inclusive(|byte| *byte == b'\n')
        .next()
        .ok_or_else(|| FixtureError::Replay("rigid handshake is missing".to_owned()))?;
    let result_bytes = trace_bytes
        .split_inclusive(|byte| *byte == b'\n')
        .nth(1)
        .ok_or_else(|| FixtureError::Replay("rigid result is missing".to_owned()))?;
    let mut metadata = CandidateMetadata {
        schema_version: CANDIDATE_SCHEMA_VERSION,
        artifact_id: artifact_id.to_owned(),
        artifact_kind,
        scenario_id: request.scenario().scenario_id().as_str().to_owned(),
        scenario_sha256: sha256(&scenario_bytes),
        source_json: serde_json::to_string(request.scenario().source())?,
        protocol_version: 1,
        scenario_schema_version: 1,
        trace_schema_version: 1,
        tolerance_profile_version: 1,
        tolerance_profile_sha256: request.tolerance_profile_sha256().as_str().to_owned(),
        oracle_revision: identity.oracle_revision().to_owned(),
        adapter_revision: identity.adapter_revision().to_owned(),
        adapter_content_sha256: identity.adapter_content_sha256().as_str().to_owned(),
        build_identity_sha256: identity.identity_sha256().as_str().to_owned(),
        preset: preset.to_owned(),
        session_profile: session_profile.to_owned(),
        compiler: format!("{} {}", identity.compiler_id(), identity.compiler_version()),
        target: identity.target().to_owned(),
        flags: vec![
            identity.effective_compile_flags().to_owned(),
            identity.effective_link_flags().to_owned(),
        ],
        generator_revision: generator_revision.to_owned(),
        review_status: ReviewStatus::Pending,
        request_sha256: sha256(request_bytes),
        trace_sha256: sha256(trace_bytes),
        report_sha256: sha256(report_bytes),
        identity_sha256: sha256(identity_bytes),
        stderr_sha256: sha256(b""),
        scenario_bytes_sha256: sha256(&scenario_bytes),
        trace_payload_sha256: sha256(result_bytes),
        failure_signature_json: maybe_failure_signature_json,
        candidate_sha256: String::new(),
    };
    metadata.candidate_sha256 = candidate_sha256(&metadata);
    let metadata_bytes = toml::to_string_pretty(&metadata)?.into_bytes();
    let files = [
        ("request.jsonl", request_bytes),
        ("trace.jsonl", trace_bytes),
        ("report.json", report_bytes),
        ("identity.jsonl", identity_bytes),
        ("stderr.txt", b"".as_slice()),
        ("scenario.json", scenario_bytes.as_slice()),
        ("candidate.toml", metadata_bytes.as_slice()),
    ];
    let staging = ensure_directory_chain(repository_root, &["target", "differential", "staging"])?;
    let directory = publish_candidate_directory(
        &staging,
        artifact_id,
        &files,
        CandidatePublishOperations::REAL,
    )?;
    Ok(ArtifactCandidate {
        artifact_id: artifact_id.into(),
        directory: fs::canonicalize(directory)?,
    })
}

#[derive(Clone, Copy)]
struct CandidatePublishOperations {
    write_file: fn(&Path, &[u8]) -> Result<(), FixtureError>,
    sync_directory: fn(&Path) -> Result<(), FixtureError>,
    rename_directory: fn(&Path, &Path) -> Result<(), FixtureError>,
    cleanup_directory: fn(&Path) -> io::Result<()>,
}

impl CandidatePublishOperations {
    const REAL: Self = Self {
        write_file: write_create_new,
        sync_directory,
        rename_directory,
        cleanup_directory,
    };
}

fn publish_candidate_directory(
    staging: &Path,
    artifact_id: &str,
    files: &[(&str, &[u8])],
    operations: CandidatePublishOperations,
) -> Result<PathBuf, FixtureError> {
    let final_directory = staging.join(artifact_id);
    match fs::symlink_metadata(&final_directory) {
        Ok(_) => {
            return Err(FixtureError::CandidateExists {
                path: final_directory,
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(FixtureError::Io(error)),
    }

    let temporary_directory = create_temporary_candidate_directory(staging, artifact_id)?;
    for (name, bytes) in files {
        if let Err(error) = (operations.write_file)(&temporary_directory.join(name), bytes) {
            return cleanup_failed_publish(&temporary_directory, error, operations);
        }
    }
    if let Err(error) = (operations.sync_directory)(&temporary_directory) {
        return cleanup_failed_publish(&temporary_directory, error, operations);
    }
    if let Err(error) = (operations.rename_directory)(&temporary_directory, &final_directory) {
        let failure = if fs::symlink_metadata(&final_directory).is_ok() {
            FixtureError::CandidateExists {
                path: final_directory,
            }
        } else {
            error
        };
        return cleanup_failed_publish(&temporary_directory, failure, operations);
    }
    (operations.sync_directory)(staging).map_err(|error| {
        FixtureError::Replay(format!(
            "candidate committed at {} but staging directory sync failed: {error}",
            final_directory.display()
        ))
    })?;
    Ok(final_directory)
}

fn create_temporary_candidate_directory(
    staging: &Path,
    artifact_id: &str,
) -> Result<PathBuf, FixtureError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    for _attempt in 0..128 {
        let sequence = NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = staging.join(format!(
            ".{artifact_id}.tmp-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(FixtureError::Io(error)),
        }
    }
    Err(FixtureError::Replay(
        "could not allocate a unique candidate staging directory".to_owned(),
    ))
}

fn cleanup_failed_publish<T>(
    temporary_directory: &Path,
    staging_error: FixtureError,
    operations: CandidatePublishOperations,
) -> Result<T, FixtureError> {
    match (operations.cleanup_directory)(temporary_directory) {
        Ok(()) => Err(staging_error),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(staging_error),
        Err(cleanup_error) => Err(FixtureError::Replay(format!(
            "candidate staging failed: {staging_error}; cleanup of {} also failed: {cleanup_error}",
            temporary_directory.display()
        ))),
    }
}

fn rename_directory(source: &Path, destination: &Path) -> Result<(), FixtureError> {
    fs::rename(source, destination)?;
    Ok(())
}

fn cleanup_directory(path: &Path) -> io::Result<()> {
    fs::remove_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_candidate_publish_cleans_temporary_state_and_allows_retry() {
        // Arrange
        let staging = std::env::temp_dir().join(format!(
            "liquidfun-rigid-publish-{}-{}",
            std::process::id(),
            NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&staging).expect("isolated staging directory should be created");
        let files = [
            ("request.jsonl", b"request\n".as_slice()),
            ("trace.jsonl", b"trace\n".as_slice()),
        ];
        let interrupted = CandidatePublishOperations {
            write_file: fail_trace_write,
            ..CandidatePublishOperations::REAL
        };

        // Act
        let error = publish_candidate_directory(&staging, "retryable", &files, interrupted)
            .expect_err("injected second-file interruption should fail publishing");

        // Assert
        assert!(error.to_string().contains("injected write interruption"));
        assert!(
            !staging.join("retryable").exists(),
            "an interrupted transaction must not expose its final directory"
        );
        assert_eq!(
            fs::read_dir(&staging)
                .expect("staging directory should be readable after interruption")
                .count(),
            0,
            "the interrupted temporary directory must be removed"
        );

        // Act
        let published = publish_candidate_directory(
            &staging,
            "retryable",
            &files,
            CandidatePublishOperations::REAL,
        )
        .expect("retry should atomically publish a complete candidate");

        // Assert
        assert_eq!(
            fs::read(published.join("request.jsonl"))
                .expect("published request should be readable"),
            b"request\n"
        );
        assert_eq!(
            fs::read(published.join("trace.jsonl")).expect("published trace should be readable"),
            b"trace\n"
        );
        let entries = fs::read_dir(&staging)
            .expect("staging directory should be readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("staging entries should be readable");
        assert_eq!(
            entries.len(),
            1,
            "no interrupted temporary entry may remain"
        );
        fs::remove_dir_all(&staging).expect("isolated staging directory should clean up");
    }

    fn fail_trace_write(path: &Path, bytes: &[u8]) -> Result<(), FixtureError> {
        if path.file_name().is_some_and(|name| name == "trace.jsonl") {
            return Err(FixtureError::Io(io::Error::other(
                "injected write interruption",
            )));
        }
        write_create_new(path, bytes)
    }
}

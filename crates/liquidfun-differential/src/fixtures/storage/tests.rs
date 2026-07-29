use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::fixtures::artifact_schema::ArtifactSchemas;
use crate::fixtures::domain::{CANDIDATE_SCHEMA_VERSION, ReviewStatus};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

#[test]
fn manifest_accepts_the_registered_phase11_evidence_schema() {
    // Arrange
    let (repository_root, _, _, _) = manifest_fixture("phase11-schema");

    // Act
    let manifest = read_manifest(&repository_root);

    // Assert
    assert!(manifest.is_ok());
    fs::remove_dir_all(repository_root).expect("fixture should be removed");
}

#[test]
fn manifest_rejects_an_unregistered_artifact_schema() {
    // Arrange
    let (repository_root, _, _, _) = manifest_fixture("unknown-schema");
    let path = repository_root.join("reference/artifacts/manifest.toml");
    let mut contents = fs::read_to_string(&path).expect("manifest should be readable");
    contents.push_str("\n[artifact_schemas.unregistered]\nschema_version = 1\n");
    fs::write(&path, contents).expect("manifest should be writable");

    // Act
    let error = read_manifest(&repository_root).expect_err("unknown schemas must fail closed");

    // Assert
    assert!(error.to_string().contains("unknown field `unregistered`"));
    fs::remove_dir_all(repository_root).expect("fixture should be removed");
}

#[test]
fn manifest_rejects_drifted_phase13_evidence_contract() {
    // Arrange
    let (repository_root, _, _, _) = manifest_fixture("phase13-schema-drift");
    let path = repository_root.join("reference/artifacts/manifest.toml");
    let contents = fs::read_to_string(&path).expect("manifest should be readable");
    assert!(contents.contains("\"record_class\""));
    fs::write(
        &path,
        contents.replacen("\"record_class\"", "\"record_kind\"", 1),
    )
    .expect("manifest should be writable");

    // Act
    let error =
        read_manifest(&repository_root).expect_err("Phase 13 schema drift must fail closed");

    // Assert
    assert!(
        error
            .to_string()
            .contains("schema, fields, or oracle revision mismatch")
    );
    fs::remove_dir_all(repository_root).expect("fixture should be removed");
}

#[test]
fn committed_manifest_survives_lock_cleanup_failure() {
    // Arrange
    let (repository_root, destination, metadata, review) = manifest_fixture("lock-cleanup");

    // Act
    let commit = update_manifest_atomically_with_operations(
        &repository_root,
        &metadata,
        &review,
        &destination,
        &sha256(b"trace\n"),
        &metadata.artifact_id,
        ManifestOperations {
            sync_directory,
            cleanup_lock: |_path| {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected lock cleanup failure",
                ))
            },
        },
    )
    .expect("manifest replacement is committed despite cleanup failure");

    // Assert
    let committed = read_manifest(&repository_root).expect("committed manifest should parse");
    assert_eq!(committed.artifacts.len(), 1);
    assert!(destination.is_file());
    assert_eq!(commit.post_commit_warnings.len(), 1);
    assert!(commit.post_commit_warnings[0].contains("lock cleanup failed"));
    assert!(
        repository_root
            .join("reference/artifacts/manifest.toml.lock")
            .is_file()
    );
    fs::remove_dir_all(repository_root).expect("fixture should be removed");
}

#[test]
fn committed_manifest_survives_directory_sync_failure() {
    // Arrange
    let (repository_root, destination, metadata, review) = manifest_fixture("directory-sync");

    // Act
    let commit = update_manifest_atomically_with_operations(
        &repository_root,
        &metadata,
        &review,
        &destination,
        &sha256(b"trace\n"),
        &metadata.artifact_id,
        ManifestOperations {
            sync_directory: |_path| {
                Err(FixtureError::Io(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "injected post-rename directory sync failure",
                )))
            },
            cleanup_lock: |path| fs::remove_file(path),
        },
    )
    .expect("manifest replacement is committed despite directory sync failure");

    // Assert
    let committed = read_manifest(&repository_root).expect("committed manifest should parse");
    assert_eq!(committed.artifacts.len(), 1);
    assert!(destination.is_file());
    assert_eq!(commit.post_commit_warnings.len(), 1);
    assert!(commit.post_commit_warnings[0].contains("directory sync failed"));
    assert!(
        !repository_root
            .join("reference/artifacts/manifest.toml.lock")
            .exists()
    );
    fs::remove_dir_all(repository_root).expect("fixture should be removed");
}

fn manifest_fixture(test_name: &str) -> (PathBuf, PathBuf, CandidateMetadata, StoredReview) {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let repository_root = std::env::temp_dir().join(format!(
        "liquidfun-manifest-{test_name}-{}-{sequence}",
        std::process::id()
    ));
    let artifact_directory = repository_root.join("reference/artifacts/traces");
    fs::create_dir_all(&artifact_directory).expect("artifact directory should be created");
    let repository_root =
        fs::canonicalize(repository_root).expect("fixture root should canonicalize");
    let manifest = ArtifactManifest {
        schema_version: 2,
        record_schema_version: 2,
        oracle_revision: REVISION.to_owned(),
        record_fields: MANIFEST_FIELDS.into_iter().map(str::to_owned).collect(),
        artifact_schemas: ArtifactSchemas::current(REVISION),
        artifacts: Vec::new(),
    };
    fs::write(
        repository_root.join("reference/artifacts/manifest.toml"),
        toml::to_string_pretty(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should be written");
    let destination = repository_root.join("reference/artifacts/traces/empty-world-v1.jsonl");
    fs::write(&destination, b"trace\n").expect("destination should be written");
    let metadata = candidate_metadata();
    let review = StoredReview {
        schema_version: CANDIDATE_SCHEMA_VERSION,
        artifact_id: metadata.artifact_id.clone(),
        candidate_sha256: metadata.candidate_sha256.clone(),
        reviewer: "reviewer".to_owned(),
        reviewed_at: "2026-07-10T12:50:00Z".to_owned(),
        review_status: ReviewStatus::Approved,
    };
    (repository_root, destination, metadata, review)
}

fn candidate_metadata() -> CandidateMetadata {
    CandidateMetadata {
        schema_version: CANDIDATE_SCHEMA_VERSION,
        artifact_id: "cleanup-failure".to_owned(),
        artifact_kind: ArtifactKind::ReviewedTrace,
        scenario_id: "empty-world".to_owned(),
        scenario_sha256: "0".repeat(64),
        source_json: r#"{"kind":"named","name":"empty-world"}"#.to_owned(),
        protocol_version: 1,
        scenario_schema_version: 1,
        trace_schema_version: 1,
        tolerance_profile_version: 1,
        tolerance_profile_sha256: "1".repeat(64),
        oracle_revision: REVISION.to_owned(),
        adapter_revision: "fixture-adapter-v1".to_owned(),
        adapter_content_sha256: "2".repeat(64),
        build_identity_sha256: "3".repeat(64),
        preset: "oracle-debug".to_owned(),
        session_profile: "one-shot".to_owned(),
        compiler: "fixture compiler".to_owned(),
        target: "fixture-target".to_owned(),
        flags: Vec::new(),
        generator_revision: REVISION.to_owned(),
        review_status: ReviewStatus::Pending,
        request_sha256: "4".repeat(64),
        trace_sha256: "5".repeat(64),
        report_sha256: "6".repeat(64),
        identity_sha256: "7".repeat(64),
        stderr_sha256: "8".repeat(64),
        scenario_bytes_sha256: "9".repeat(64),
        trace_payload_sha256: "a".repeat(64),
        failure_signature_json: None,
        candidate_sha256: "b".repeat(64),
    }
}

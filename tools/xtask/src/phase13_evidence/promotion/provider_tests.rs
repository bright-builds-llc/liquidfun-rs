use super::{Artifact, ProviderRun, Request, validate_provider};

fn fixture() -> (Request, ProviderRun, Artifact, String) {
    let producer = "a".repeat(40);
    let request = Request {
        schema_version: 1,
        run_id: 99,
        run_attempt: 2,
        artifact_id: 123,
        archive_path: "target/archive.zip".to_owned(),
    };
    let run = serde_json::from_value(serde_json::json!({
        "id":99,"run_attempt":2,"head_sha":producer,"path":super::WORKFLOW_PATH,
        "event":"workflow_dispatch","status":"completed","conclusion":"success",
        "run_started_at":"2026-09-14T00:00:00Z",
        "repository":{"full_name":super::PROVIDER_REPOSITORY},
        "head_repository":{"full_name":super::PROVIDER_REPOSITORY}
    }))
    .expect("run fixture decodes");
    let artifact = serde_json::from_value(serde_json::json!({
        "id":123,"name":format!("phase13-staged-99-{producer}"),
        "digest":format!("sha256:{}", "b".repeat(64)),
        "created_at":"2026-09-14T00:01:00Z","expires_at":"2026-12-14T00:00:00Z",
        "expired":false,"workflow_run":{"id":99,"head_sha":producer}
    }))
    .expect("artifact fixture decodes");
    (request, run, artifact, producer)
}

#[test]
fn fresh_provider_tuple_passes() {
    // Arrange
    let (request, run, artifact, producer) = fixture();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn rejects_wrong_source() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.head_sha = "c".repeat(40);
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_wrong_workflow() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.path = ".github/workflows/other.yml".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_wrong_repository() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.repository = super::Repository {
        full_name: "attacker/repo".to_owned(),
    };
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_nonterminal_run() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.status = "in_progress".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_failed_run() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.conclusion = "failure".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_stale_attempt() {
    // Arrange
    let (request, mut run, artifact, producer) = fixture();
    run.run_attempt = 3;
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_wrong_artifact() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.id = 124;
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_wrong_artifact_name() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.name = "arbitrary".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_wrong_artifact_run() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.workflow_run = super::ArtifactRun {
        id: 100,
        head_sha: "a".repeat(40),
    };
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_expired_artifact() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.expired = true;
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_old_attempt_artifact() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.created_at = "2026-09-13T00:00:00Z".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

#[test]
fn rejects_malformed_provider_digest() {
    // Arrange
    let (request, run, mut artifact, producer) = fixture();
    artifact.digest = "sha256:bad".to_owned();
    // Act
    let result = validate_provider(&request, &run, &artifact, &producer);
    // Assert
    assert!(result.is_err(), "mismatched provider tuple accepted");
}

fn jobs() -> super::Jobs {
    serde_json::from_value(serde_json::json!({"total_count":1,"jobs":[{
        "run_id":99,"run_attempt":2,"head_sha":"a".repeat(40),
        "name":"Produce one immutable canonical Linux bundle",
        "status":"completed","conclusion":"success"
    }]}))
    .expect("job fixture decodes")
}

#[test]
fn exact_producer_job_passes() {
    // Arrange
    let (request, _, _, producer) = fixture();
    // Act
    let result = super::validate_jobs(&jobs(), &request, &producer);
    // Assert
    assert!(result.is_ok());
}

#[test]
fn unrelated_successful_job_fails() {
    // Arrange
    let (request, _, _, producer) = fixture();
    let mut jobs = jobs();
    jobs.jobs[0].name = "unrelated success".to_owned();
    // Act
    let result = super::validate_jobs(&jobs, &request, &producer);
    // Assert
    assert!(result.is_err());
}

#[test]
fn wrong_job_source_fails() {
    // Arrange
    let (request, _, _, producer) = fixture();
    let mut jobs = jobs();
    jobs.jobs[0].head_sha = "b".repeat(40);
    // Act
    let result = super::validate_jobs(&jobs, &request, &producer);
    // Assert
    assert!(result.is_err());
}

#[test]
fn incomplete_job_listing_fails() {
    // Arrange
    let (request, _, _, producer) = fixture();
    let mut jobs = jobs();
    jobs.total_count = 2;
    // Act
    let result = super::validate_jobs(&jobs, &request, &producer);
    // Assert
    assert!(result.is_err());
}

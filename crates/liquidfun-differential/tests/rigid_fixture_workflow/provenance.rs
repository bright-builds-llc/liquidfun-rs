use std::{fs, io, path::Path, process::Output};

use liquidfun_differential::{ArtifactKind, OraclePreset, stage_rigid_candidate};
use sha2::{Digest, Sha256};

use super::{RigidFixtureRepository, git_head, rigid_minimization, stderr};

#[test]
fn replay_rejects_malformed_transform_after_metadata_rehash()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let candidate = stage_minimized_candidate(&repository, "malformed-transform")?;
    rewrite_transform_report(&candidate, |report| {
        report["accepted_transforms"][0] = serde_json::json!({"not_a_transform": {}});
    })?;

    // Act
    let reviewed = repository.review("malformed-transform")?;

    // Assert
    assert_transform_report_rejected(&repository, "malformed-transform", &reviewed);
    assert!(stderr(&reviewed).contains("unknown variant"));
    Ok(())
}

#[test]
fn replay_rejects_unrelated_transform_after_metadata_rehash()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let candidate = stage_minimized_candidate(&repository, "unrelated-transform")?;
    rewrite_transform_report(&candidate, |report| {
        report["accepted_transforms"][0] = serde_json::json!({
            "remove_actions": {
                "timeline_index": 999,
                "start": 0,
                "end": 1
            }
        });
    })?;

    // Act
    let reviewed = repository.review("unrelated-transform")?;

    // Assert
    assert_transform_report_rejected(&repository, "unrelated-transform", &reviewed);
    assert!(stderr(&reviewed).contains("failure signature changed"));
    Ok(())
}

#[test]
fn replay_rejects_reordered_transforms_after_metadata_rehash()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let candidate = stage_minimized_candidate(&repository, "reordered-transforms")?;
    rewrite_transform_report(&candidate, |report| {
        let accepted = report["accepted_transforms"]
            .as_array_mut()
            .expect("staged accepted transforms should be an array");
        assert!(
            accepted.len() >= 2,
            "fixture should retain multiple transforms"
        );
        accepted.reverse();
    })?;

    // Act
    let reviewed = repository.review("reordered-transforms")?;

    // Assert
    assert_transform_report_rejected(&repository, "reordered-transforms", &reviewed);
    assert!(stderr(&reviewed).contains("failure signature changed"));
    Ok(())
}

#[test]
fn replay_rejects_excess_duplicate_after_metadata_rehash() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let candidate = stage_minimized_candidate(&repository, "duplicate-transform")?;
    rewrite_transform_report(&candidate, |report| {
        let attempted = report["attempted_transforms"]
            .as_array()
            .expect("staged attempted transforms should be an array");
        let transform = report["accepted_transforms"][0].clone();
        let attempted_occurrences = attempted
            .iter()
            .filter(|attempt| **attempt == transform)
            .count();
        report["accepted_transforms"] =
            serde_json::Value::Array(vec![transform; attempted_occurrences + 1]);
    })?;

    // Act
    let reviewed = repository.review("duplicate-transform")?;

    // Assert
    assert_transform_report_rejected(&repository, "duplicate-transform", &reviewed);
    assert!(stderr(&reviewed).contains("failure signature changed"));
    Ok(())
}

fn stage_minimized_candidate(
    repository: &RigidFixtureRepository,
    artifact_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let minimization = rigid_minimization(&repository.root, 4_096)?;
    let candidate = stage_rigid_candidate(
        &repository.root,
        artifact_id,
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        Some(&minimization),
    )?;
    Ok(candidate.directory().to_path_buf())
}

fn rewrite_transform_report(
    candidate: &Path,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = candidate.join("report.json");
    let mut report: serde_json::Value = serde_json::from_slice(&fs::read(&report_path)?)?;
    mutate(&mut report);
    let mut report_bytes = serde_json::to_vec(&report)?;
    report_bytes.push(b'\n');
    fs::write(&report_path, &report_bytes)?;

    let metadata_path = candidate.join("candidate.toml");
    let mut metadata: toml::Value = toml::from_str(&fs::read_to_string(&metadata_path)?)?;
    let metadata_table = metadata
        .as_table_mut()
        .ok_or_else(|| io::Error::other("candidate metadata should be a TOML table"))?;
    metadata_table.insert(
        "report_sha256".to_owned(),
        toml::Value::String(sha256(&report_bytes)),
    );
    metadata_table.insert(
        "candidate_sha256".to_owned(),
        toml::Value::String(String::new()),
    );
    let candidate_hash = candidate_sha256(metadata_table)?;
    metadata_table.insert(
        "candidate_sha256".to_owned(),
        toml::Value::String(candidate_hash),
    );
    fs::write(metadata_path, toml::to_string_pretty(&metadata)?)?;
    Ok(())
}

fn candidate_sha256(metadata: &toml::value::Table) -> Result<String, Box<dyn std::error::Error>> {
    let mut digest = Sha256::new();
    for field in [
        "artifact_id",
        "artifact_kind",
        "scenario_id",
        "scenario_sha256",
        "source_json",
        "tolerance_profile_sha256",
        "oracle_revision",
        "adapter_revision",
        "adapter_content_sha256",
        "build_identity_sha256",
        "preset",
        "session_profile",
        "compiler",
        "target",
        "generator_revision",
        "review_status",
        "request_sha256",
        "trace_sha256",
        "report_sha256",
        "identity_sha256",
        "stderr_sha256",
        "scenario_bytes_sha256",
        "trace_payload_sha256",
    ] {
        update_length_prefixed(&mut digest, metadata_string(metadata, field)?);
    }
    update_length_prefixed(
        &mut digest,
        metadata
            .get("failure_signature_json")
            .and_then(toml::Value::as_str)
            .unwrap_or(""),
    );
    for field in [
        "schema_version",
        "protocol_version",
        "scenario_schema_version",
        "trace_schema_version",
        "tolerance_profile_version",
    ] {
        let version = metadata
            .get(field)
            .and_then(toml::Value::as_integer)
            .ok_or_else(|| {
                io::Error::other(format!("candidate field {field} is not an integer"))
            })?;
        digest.update(u32::try_from(version)?.to_be_bytes());
    }
    let flags = metadata
        .get("flags")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| io::Error::other("candidate flags should be an array"))?;
    for flag in flags {
        let flag = flag
            .as_str()
            .ok_or_else(|| io::Error::other("candidate flag should be a string"))?;
        update_length_prefixed(&mut digest, flag);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn metadata_string<'a>(
    metadata: &'a toml::value::Table,
    field: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    metadata
        .get(field)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| io::Error::other(format!("candidate field {field} is not a string")).into())
}

fn update_length_prefixed(digest: &mut Sha256, value: &str) {
    digest.update(value.len().to_be_bytes());
    digest.update(value.as_bytes());
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_transform_report_rejected(
    repository: &RigidFixtureRepository,
    artifact_id: &str,
    reviewed: &Output,
) {
    assert!(!reviewed.status.success());
    assert!(!stderr(reviewed).contains("SHA-256 mismatch"));
    assert!(
        !repository
            .candidate(artifact_id)
            .join("review.toml")
            .exists()
    );
}

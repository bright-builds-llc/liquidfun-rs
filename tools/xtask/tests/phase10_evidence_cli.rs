//! Adversarial command coverage for the Phase 10 evidence validator.

#[path = "phase10_evidence_cli/support.rs"]
mod support;

use std::fs;

use serde_json::{Value, json};

use support::{TestResult, TestRoot, assert_failure, assert_success, refresh_identity, write_json};

#[test]
fn content_accepts_one_shared_complete_local_pair() -> TestResult {
    // Arrange
    let root = TestRoot::new("valid-content")?;
    root.write_local_pair()?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("80 semantic leaves"));
    Ok(())
}

#[test]
fn content_rejects_recomputed_manifest_leaf_and_policy_substitution() -> TestResult {
    // Arrange
    let missing = TestRoot::new("missing-leaf")?;
    missing.write_local_pair()?;
    missing.mutate_manifest(|manifest| {
        manifest["bindings"].as_array_mut().expect("bindings").pop();
    })?;
    let policy = TestRoot::new("unknown-policy")?;
    policy.write_local_pair()?;
    policy.mutate_manifest(|manifest| {
        manifest["bindings"][0]["policy_path"] = json!("phase10/*");
    })?;

    // Act
    let missing_output = missing.run_local()?;
    let policy_output = policy.run_local()?;

    // Assert
    assert_failure(&missing_output);
    assert_failure(&policy_output);
    Ok(())
}

#[test]
fn content_rejects_recomputed_proof_alias_replay_and_failed_outcome() -> TestResult {
    // Arrange
    let alias = TestRoot::new("alias")?;
    alias.write_local_pair()?;
    alias.mutate_manifest(|manifest| {
        let native = manifest["cases"][0]["proofs"]["native"].clone();
        manifest["cases"][0]["proofs"]["oracle"] = native;
    })?;
    let replay = TestRoot::new("replay")?;
    replay.write_local_pair()?;
    replay.mutate_proof("replay-native", |proof| {
        proof["payload"]["semantic"] = json!("different");
    })?;
    let failed = TestRoot::new("failed")?;
    failed.write_local_pair()?;
    failed.mutate_proof("comparison", |proof| {
        proof["outcome"] = json!("mismatch");
    })?;

    // Act
    let outputs = [alias.run_local()?, replay.run_local()?, failed.run_local()?];

    // Assert
    assert!(outputs.iter().all(|output| !output.status.success()));
    Ok(())
}

#[test]
fn content_rejects_digest_log_extra_missing_and_symlink_corruption() -> TestResult {
    // Arrange
    let digest = TestRoot::new("digest")?;
    digest.write_local_pair()?;
    fs::write(
        digest
            .path
            .join("canonical/cases/group-construction-and-mutation/proofs/native.json"),
        b"{}",
    )?;
    let log = TestRoot::new("log")?;
    log.write_local_pair()?;
    fs::write(
        log.path.join("canonical/phase10-trace.log"),
        b"status: FAILED\n",
    )?;
    refresh_identity(&log.path.join("canonical"))?;
    let extra = TestRoot::new("extra")?;
    extra.write_local_pair()?;
    fs::write(extra.path.join("canonical/extra"), b"extra")?;
    let missing = TestRoot::new("missing")?;
    missing.write_local_pair()?;
    fs::remove_file(missing.path.join("canonical/read-only.log"))?;
    let link = TestRoot::new("link")?;
    link.write_local_pair()?;
    #[cfg(unix)]
    std::os::unix::fs::symlink("inventory.log", link.path.join("canonical/forbidden-link"))?;

    // Act
    let outputs = [
        digest.run_local()?,
        log.run_local()?,
        extra.run_local()?,
        missing.run_local()?,
        link.run_local()?,
    ];

    // Assert
    assert!(outputs.iter().all(|output| !output.status.success()));
    Ok(())
}

#[test]
fn content_rejects_unsafe_paths_resource_depth_and_local_authority_substitution() -> TestResult {
    // Arrange
    let unsafe_path = TestRoot::new("unsafe-path")?;
    unsafe_path.write_local_pair()?;
    unsafe_path.mutate_manifest(|manifest| {
        manifest["cases"][0]["proofs"]["native"]["path"] = json!("../escape.json");
    })?;
    let deep = TestRoot::new("deep")?;
    deep.write_local_pair()?;
    fs::create_dir_all(deep.path.join("canonical/a/b/c/d/e/f/g"))?;
    let authority = TestRoot::new("local-authority")?;
    authority.write_local_pair()?;
    let identity_path = authority.path.join("canonical/identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    identity["run_id"] = json!(99);
    write_json(&identity_path, &identity)?;

    // Act
    let outputs = [
        unsafe_path.run_local()?,
        deep.run_local()?,
        authority.run_local()?,
    ];

    // Assert
    assert!(outputs.iter().all(|output| !output.status.success()));
    Ok(())
}

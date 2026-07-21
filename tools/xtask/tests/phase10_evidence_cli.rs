//! Adversarial command coverage for the Phase 10 evidence validator.

#[path = "phase10_evidence_cli/exact.rs"]
mod exact;
#[path = "phase10_evidence_cli/support.rs"]
mod support;

use std::fs;

use serde_json::{Value, json};

use exact::{APPROVED_SHA, CANONICAL_ARTIFACT, EXACT_RUN};
use support::{
    TestResult, TestRoot, assert_failure, assert_success, refresh_identity, workspace_root,
    write_json,
};

#[test]
fn workflow_contract_defines_one_same_run_phase10_pair() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;
    let canonical_run =
        "bash scripts/phase10-evidence.sh canonical target/oracle-evidence/phase10-canonical";
    let sanitizer_run =
        "bash scripts/phase10-evidence.sh sanitizer target/oracle-evidence/phase10-sanitizer";

    // Act
    let action_refs = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses: "))
        .collect::<Vec<_>>();

    // Assert
    assert!(workflow.contains("          - phase10\n"));
    assert!(workflow.contains("'Phase 10 canonical Linux oracle'"));
    assert!(workflow.contains("'Phase 10 fail-fast sanitizer'"));
    assert_eq!(workflow.matches(canonical_run).count(), 1);
    assert_eq!(workflow.matches(sanitizer_run).count(), 1);
    assert!(workflow.contains("name: phase10-canonical-${{ github.run_id }}-${{ github.sha }}"));
    assert!(workflow.contains("name: phase10-sanitizer-${{ github.run_id }}-${{ github.sha }}"));
    assert_eq!(workflow.matches("retention-days: 30").count(), 6);
    assert!(workflow.contains("permissions:\n  contents: read"));
    assert!(workflow.contains(
        "if: github.event_name == 'workflow_dispatch' && inputs.evidence_phase == 'phase10'"
    ));
    assert!(!action_refs.is_empty());
    assert!(action_refs.iter().all(|action_ref| {
        let Some((_, revision)) = action_ref.rsplit_once('@') else {
            return false;
        };
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    }));
    Ok(())
}

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

#[test]
fn exact_ref_accepts_one_current_same_run_authority_pair() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-valid")?;
    let run = root.write_exact_pair()?;
    root.write_run(&run)?;

    // Act
    let output = root.run_exact_ref(&[])?;

    // Assert
    assert_success(&output);
    Ok(())
}

#[test]
fn exact_ref_rejects_repeatable_run_and_artifact_denylists_before_promotion() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-denylists")?;
    let run = root.write_exact_pair()?;
    root.write_run(&run)?;

    // Act
    let run_output = root.run_exact_ref(&[("--deny-run-id", 1), ("--deny-run-id", EXACT_RUN)])?;
    let artifact_output = root.run_exact_ref(&[
        ("--deny-artifact-id", 2),
        ("--deny-artifact-id", CANONICAL_ARTIFACT),
    ])?;

    // Assert
    assert_failure(&run_output);
    assert_failure(&artifact_output);
    Ok(())
}

#[test]
fn exact_ref_rejects_stale_head_toolchain_expiry_and_live_metadata_mutation() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-stale")?;
    let valid = root.write_exact_pair()?;
    let mut stale = valid.clone();
    stale["head_sha"] = json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    root.write_run(&stale)?;
    let stale_output = root.run_exact_ref(&[])?;
    let mut toolchain = valid.clone();
    toolchain["clang_version"] = json!("22.1.7");
    root.write_run(&toolchain)?;
    let toolchain_output = root.run_exact_ref(&[])?;
    let mut expired = valid.clone();
    expired["artifacts"]["canonical"]["expires_at"] = json!("2026-07-21T11:30:00Z");
    root.write_run(&expired)?;
    let expired_output = root.run_exact_ref(&[])?;
    let mut live = valid;
    live["live_jobs"][0]["id"] = json!(9999);
    root.write_run(&live)?;

    // Act
    let live_output = root.run_exact_ref(&[])?;

    // Assert
    assert!(
        [stale_output, toolchain_output, expired_output, live_output]
            .iter()
            .all(|output| !output.status.success())
    );
    Ok(())
}

#[test]
fn exact_ref_rejects_mixed_job_artifact_and_extracted_identity() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-mixing")?;
    let valid = root.write_exact_pair()?;
    let mut mixed_job = valid.clone();
    mixed_job["jobs"]["sanitizer"]["id"] = mixed_job["jobs"]["canonical"]["id"].clone();
    root.write_run(&mixed_job)?;
    let job_output = root.run_exact_ref(&[])?;
    let mut mixed_artifact = valid.clone();
    mixed_artifact["artifacts"]["sanitizer"]["id"] = json!(CANONICAL_ARTIFACT);
    root.write_run(&mixed_artifact)?;
    let artifact_output = root.run_exact_ref(&[])?;
    root.write_run(&valid)?;
    let identity_path = root.path.join("canonical/identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    identity["head_sha"] = json!(APPROVED_SHA.replace('a', "b"));
    write_json(&identity_path, &identity)?;
    let identity_output = root.run_exact_ref(&[])?;
    identity["head_sha"] = json!(APPROVED_SHA);
    identity["artifact_id"] = json!(CANONICAL_ARTIFACT);
    write_json(&identity_path, &identity)?;

    // Act
    let post_upload_identity_output = root.run_exact_ref(&[])?;

    // Assert
    assert_failure(&job_output);
    assert_failure(&artifact_output);
    assert_failure(&identity_output);
    assert_failure(&post_upload_identity_output);
    Ok(())
}

#[test]
fn exact_ref_rejects_archive_digest_extra_entry_and_symlink_ancestor() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-archive")?;
    let valid = root.write_exact_pair()?;
    let mut digest = valid.clone();
    digest["artifacts"]["canonical"]["digest"] = json!(format!("sha256:{}", "0".repeat(64)));
    digest["live_artifacts"][0]["digest"] = digest["artifacts"]["canonical"]["digest"].clone();
    root.write_run(&digest)?;
    let digest_output = root.run_exact_ref(&[])?;
    let mut extra = valid.clone();
    root.add_extra_archive_entry(&mut extra)?;
    root.write_run(&extra)?;
    let extra_output = root.run_exact_ref(&[])?;

    // Act
    #[cfg(unix)]
    let link_output = {
        use std::os::unix::fs::symlink;

        let external = root.path.join("archive-source");
        fs::create_dir_all(&external)?;
        fs::copy(
            root.path.join("sanitizer.zip"),
            external.join("sanitizer.zip"),
        )?;
        symlink(&external, root.path.join("archive-link"))?;
        let mut linked = valid;
        linked["artifacts"]["sanitizer"]["archive_path"] =
            json!(root.relative("archive-link/sanitizer.zip"));
        root.write_run(&linked)?;
        root.run_exact_ref(&[])?
    };

    // Assert
    assert_failure(&digest_output);
    assert_failure(&extra_output);
    #[cfg(unix)]
    assert_failure(&link_output);
    Ok(())
}

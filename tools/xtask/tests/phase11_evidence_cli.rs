//! Fail-closed Phase 11 evidence CLI contracts.

use std::{path::Path, process::Command};

#[path = "phase11_evidence_cli/exact.rs"]
mod exact;
#[path = "phase11_evidence_cli/support.rs"]
mod support;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask remains two levels below the workspace")
}

fn run(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()
}

#[test]
fn generated_local_pair_is_complete_but_non_promotable() -> TestResult {
    // Arrange
    let root = support::TestRoot::new("local")?;
    root.write_local_pair()?;

    // Act
    let output = root.run_local()?;

    // Assert
    support::assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("non-promotable"));
    Ok(())
}

#[test]
fn malformed_private_partial_and_unknown_content_fails_closed() -> TestResult {
    // Arrange / Act / Assert: unknown/private proof field
    let private = support::TestRoot::new("private")?;
    private.write_local_pair()?;
    private.mutate_record("debug", |record| {
        record["pixel_frame_duration"] = serde_json::json!(16);
    })?;
    support::assert_failure(&private.run_local()?);

    // Arrange / Act / Assert: omitted leaf and unknown policy
    let policy = support::TestRoot::new("policy")?;
    policy.write_local_pair()?;
    policy.mutate_payload(|payload| {
        payload["observation_leaves"] = serde_json::json!([]);
        payload["numeric_policies"] = serde_json::json!(["unknown.open.policy"]);
    })?;
    support::assert_failure(&policy.run_local()?);

    // Arrange / Act / Assert: partial output and unknown file
    let partial = support::TestRoot::new("partial")?;
    partial.write_local_pair()?;
    std::fs::remove_file(partial.path.join("canonical/replay.jsonl"))?;
    support::assert_failure(&partial.run_local()?);
    let extra = support::TestRoot::new("extra")?;
    extra.write_local_pair()?;
    std::fs::write(extra.path.join("canonical/unexpected.bin"), b"unexpected")?;
    support::assert_failure(&extra.run_local()?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_local_content_is_rejected() -> TestResult {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = support::TestRoot::new("symlink")?;
    root.write_local_pair()?;
    std::fs::remove_file(root.path.join("canonical/release.jsonl"))?;
    symlink("debug.jsonl", root.path.join("canonical/release.jsonl"))?;

    // Act / Assert
    support::assert_failure(&root.run_local()?);
    Ok(())
}

#[test]
fn tracked_local_corpus_uses_the_closed_non_promotable_path() -> TestResult {
    // Arrange
    let corpus = "crates/liquidfun-differential/tests/fixtures/catalog";

    // Act
    let output = run(&[
        "phase11-evidence",
        "validate",
        "--mode",
        "local",
        "--canonical-dir",
        corpus,
        "--sanitizer-dir",
        corpus,
    ])?;

    // Assert
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("non-promotable"));
    Ok(())
}

#[test]
fn exact_ref_rejects_the_identity_free_tracked_corpus() -> TestResult {
    // Arrange
    let corpus = "crates/liquidfun-differential/tests/fixtures/catalog";

    // Act
    let output = run(&[
        "phase11-evidence",
        "validate",
        "--mode",
        "exact-ref",
        "--canonical-dir",
        corpus,
        "--sanitizer-dir",
        corpus,
    ])?;

    // Assert
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("run-json"));
    Ok(())
}

#[test]
fn unknown_or_partial_options_fail_before_filesystem_access() -> TestResult {
    // Arrange / Act
    let unknown = run(&[
        "phase11-evidence",
        "validate",
        "--mode",
        "local",
        "--canonical-dir",
        "target/missing",
        "--surprise",
        "value",
    ])?;
    let partial = run(&["phase11-evidence", "validate", "--mode", "local"])?;

    // Assert
    assert!(!unknown.status.success());
    assert!(!partial.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("usage"));
    assert!(String::from_utf8_lossy(&partial.stderr).contains("sanitizer-dir"));
    Ok(())
}

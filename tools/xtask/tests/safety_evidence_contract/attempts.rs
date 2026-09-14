use super::*;

#[test]
fn producers_preserve_failures_and_bound_diagnostics() -> TestResult {
    // Arrange
    let root = workspace_root()
        .join("target/phase15-attempt-tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&root)?;

    // Act
    let output = Command::new("timeout")
        .args(["120", "bash", "-x"])
        .arg(workspace_root().join("tools/xtask/tests/safety_evidence_contract/attempts.sh"))
        .arg(&root)
        .arg(workspace_root())
        .output()?;
    fs::write(root.join("stdout.log"), &output.stdout)?;
    fs::write(root.join("stderr.log"), &output.stderr)?;

    // Assert
    assert!(
        output.status.success(),
        "attempt fixture exited with {}; retained at {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        root.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
